pub mod analysis;
pub mod ddragon;
pub mod lcu;
pub mod live;
pub mod overlay;
pub mod profile;
pub mod recorder;
pub mod riot;
pub mod storage;
pub mod watcher;

use serde::Serialize;
use std::sync::Arc;
use tauri::{Emitter, Manager};

use analysis::LaneReport;
use lcu::models::{ConnStatus, DraftState};
use presage_core::engine::build::{self, BuildContext, ItemSuggestion};
use presage_core::engine::champion::{ChampionIndex, Role};
use presage_core::engine::context::{self, DraftContext, Leitura};
use presage_core::engine::stats::StatsStore;
use presage_core::engine::coach::Report;
use presage_core::engine::dataset::{Dataset, Layered};
use presage_core::engine::gold::{self, GoldReport};
use presage_core::engine::lanes;
use presage_core::engine::poolgap;
use presage_core::engine::profile::LaneProfile;
use presage_core::engine::runes;
use presage_core::engine::minimax;
use presage_core::engine::stats::PersonalStats;
use presage_core::engine::{self, Scored};
use live::LiveSnapshot;
use presage_core::pool::{Pool, Tier};
use presage_core::prefs::{DraftPrefs, OverlayPrefs};
use watcher::Shared;

const EVENT_ENGINE: &str = "presage://engine";
const POOL_CACHE: &str = "pool.json";
const PREFS_CACHE: &str = "overlay.json";
const DRAFT_PREFS_CACHE: &str = "draft.json";
/// Base de partidas agregada. Não vem no instalador; quando existir, é aqui
/// que ela mora.
const DATASET_CACHE: &str = "base-de-partidas.json";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct EngineStatus {
    ready: bool,
    version: String,
    champions: usize,
    has_global: bool,
    has_personal: bool,
    personal_games: u32,
    account: Option<String>,
    /// Patch de onde a base de partidas saiu, quando existe. A interface mostra
    /// para ninguém confiar numa base de dois patches atrás sem saber.
    global_patch: Option<String>,
    global_matches: u64,
}

fn engine_status_of(shared: &Shared) -> EngineStatus {
    let champions = shared.champions.read().unwrap();
    let stats = shared.stats.read().unwrap();
    let dataset = shared.dataset.read().unwrap();

    EngineStatus {
        ready: !champions.is_empty(),
        version: champions.version.clone(),
        champions: champions.by_id.len(),
        has_global: stats.has_global(),
        has_personal: stats.has_personal(),
        personal_games: stats.personal_games(),
        account: shared
            .identity
            .lock()
            .unwrap()
            .as_ref()
            .map(|id| id.riot_id.clone()),
        global_patch: dataset.as_ref().map(|d| d.patch.clone()),
        global_matches: dataset.as_ref().map_or(0, |d| d.matches),
    }
}

/// Reconstrói a loja de estatísticas a partir das duas camadas.
///
/// As duas são independentes: a base agregada responde winrate, matchup e
/// sinergia; o histórico da conta responde só o componente pessoal. Trocar uma
/// não pode apagar a outra, e é por isso que a montagem fica num lugar só.
fn rebuild_stats(shared: &Shared, personal: PersonalStats) {
    let global = shared.dataset.read().unwrap().clone();
    *shared.stats.write().unwrap() = Arc::new(Layered { global, personal });
}

#[tauri::command]
fn get_snapshot(shared: tauri::State<'_, Arc<Shared>>) -> (Option<ConnStatus>, Option<DraftState>) {
    (
        shared.status.lock().unwrap().clone(),
        shared.draft.lock().unwrap().clone(),
    )
}

#[tauri::command]
fn engine_status(shared: tauri::State<'_, Arc<Shared>>) -> EngineStatus {
    engine_status_of(&shared)
}

/// O draft que o motor lê: o do simulador, quando ele está aberto, ou o do
/// champ select do cliente.
fn current_draft(shared: &Shared) -> Option<DraftState> {
    let simulado = shared.simulated.lock().unwrap().clone();
    simulado.or_else(|| shared.draft.lock().unwrap().clone())
}

/// O contexto do draft com a rota resolvida: a do cliente, a escolhida na tela,
/// ou a da pool. Ranking, minimax e análise de rota leem o mesmo.
///
/// Entram também as rotas que você marcou nos inimigos e, fora do simulador e
/// sem "mostrar todos", a lista de quem a conta pode escolher.
fn draft_context(
    shared: &Shared,
    draft: &DraftState,
    champions: &ChampionIndex,
    stats: &dyn StatsStore,
) -> DraftContext {
    let pool = shared.pool.read().unwrap();
    let chosen = *shared.draft_role.lock().unwrap();
    let role = context::resolve_role(&draft.local_position, chosen, &pool);
    let simulando = shared.simulated.lock().unwrap().is_some();
    let todos = shared.draft_prefs.read().unwrap().show_all_champions;
    let leitura = Leitura {
        enemy_roles: shared.enemy_roles.lock().unwrap().clone(),
        owned: if simulando || todos {
            None
        } else {
            shared.owned.lock().unwrap().clone()
        },
    };
    DraftContext::new_with(draft, champions, stats, &pool.ids_for_role(role), role, &leitura)
}

#[tauri::command]
fn rank_draft(shared: tauri::State<'_, Arc<Shared>>) -> Vec<Scored> {
    let draft = current_draft(&shared);
    let Some(draft) = draft else {
        return Vec::new();
    };

    let champions = shared.champions.read().unwrap();
    let stats = shared.stats.read().unwrap();
    let ctx = draft_context(&shared, &draft, &champions, &**stats);
    engine::rank_with(&ctx, &champions, &**stats)
}

/// A rota para quando o cliente não informa. `None` volta para a da pool.
#[tauri::command]
fn set_draft_role(role: Option<String>, shared: tauri::State<'_, Arc<Shared>>) {
    *shared.draft_role.lock().unwrap() = role
        .map(|r| Role::from_lcu(&r))
        .filter(|r| *r != Role::Unknown);
}

/// A rota de um inimigo, marcada à mão quando a inferida está errada. `None`
/// desfaz a marca e devolve o campeão à inferência.
#[tauri::command]
fn set_enemy_role(champion_id: i64, role: Option<String>, shared: tauri::State<'_, Arc<Shared>>) {
    let mut marcadas = shared.enemy_roles.lock().unwrap();
    match role.map(|r| Role::from_lcu(&r)).filter(|r| *r != Role::Unknown) {
        Some(role) => {
            marcadas.insert(champion_id, role);
        }
        None => {
            marcadas.remove(&champion_id);
        }
    }
}

/// As rotas marcadas agora, para a interface mostrar o que é marca.
#[tauri::command]
fn get_enemy_roles(shared: tauri::State<'_, Arc<Shared>>) -> Vec<(i64, Role)> {
    shared
        .enemy_roles
        .lock()
        .unwrap()
        .iter()
        .map(|(id, role)| (*id, *role))
        .collect()
}

/// O simulador de draft. `Some` passa a ser o draft que o motor lê, com os
/// indisponíveis refeitos; `None` volta para o champ select do cliente.
#[tauri::command]
fn set_simulated_draft(draft: Option<DraftState>, shared: tauri::State<'_, Arc<Shared>>) {
    *shared.simulated.lock().unwrap() = draft.map(DraftState::normalized);
}

#[tauri::command]
fn get_draft_prefs(shared: tauri::State<'_, Arc<Shared>>) -> DraftPrefs {
    shared.draft_prefs.read().unwrap().clone()
}

#[tauri::command]
fn set_draft_prefs(prefs: DraftPrefs, shared: tauri::State<'_, Arc<Shared>>) -> Result<(), String> {
    storage::write_json(DRAFT_PREFS_CACHE, &prefs).map_err(|err| err.to_string())?;
    *shared.draft_prefs.write().unwrap() = prefs;
    Ok(())
}

/// Quanto você tira do oponente direto, do seu histórico. `None` sem chave da
/// Riot ou antes da primeira atualização.
#[tauri::command]
fn lane_profile(shared: tauri::State<'_, Arc<Shared>>) -> Option<LaneProfile> {
    shared.stats.read().unwrap().lane_profile()
}

/// Ranking com uma camada de resposta adversária.
#[tauri::command]
fn minimax_draft(shared: tauri::State<'_, Arc<Shared>>) -> Vec<minimax::MinimaxPick> {
    let draft = current_draft(&shared);
    let Some(draft) = draft else {
        return Vec::new();
    };

    let champions = shared.champions.read().unwrap();
    let stats = shared.stats.read().unwrap();
    let ctx = draft_context(&shared, &draft, &champions, &**stats);
    minimax::minimax_with(&ctx, &champions, &**stats, 12)
}

#[tauri::command]
fn ban_suggestions(shared: tauri::State<'_, Arc<Shared>>) -> Vec<minimax::BanSuggestion> {
    let draft = current_draft(&shared);
    let Some(draft) = draft else {
        return Vec::new();
    };
    let champions = shared.champions.read().unwrap();
    let pool = shared.pool.read().unwrap();
    minimax::bans_with_pool(&draft, &champions, &pool.bans, 10)
}

#[tauri::command]
fn get_live(shared: tauri::State<'_, Arc<Shared>>) -> Option<LiveSnapshot> {
    shared.live.lock().unwrap().clone()
}

/// Build adaptativa para a partida em andamento.
#[tauri::command]
fn build_suggestions(
    limit: Option<usize>,
    shared: tauri::State<'_, Arc<Shared>>,
) -> Option<(BuildContext, Vec<ItemSuggestion>)> {
    let snapshot = shared.live.lock().unwrap().clone()?;
    let game = snapshot.game;
    let me = game.me()?.as_combatant();
    let enemies: Vec<_> = game
        .enemies()
        .into_iter()
        .map(|enemy| enemy.as_combatant())
        .collect();

    let champions = shared.champions.read().unwrap();
    let items = shared.items.read().unwrap();

    Some(build::suggest(
        &me,
        &enemies,
        &champions,
        &items,
        limit.unwrap_or(8),
    ))
}

/// Chance de vitória por rota e em cada escaramuça.
///
/// Serve o draft e a partida com a mesma chamada: durante o jogo os níveis e
/// itens são reais, antes dele é projeção em igualdade.
/// A análise de rota. Com `candidate`, a prévia do draft com aquele campeão no
/// seu lugar — a mesma conta de quando ele estiver travado. A prévia ignora a
/// partida: ela só existe no champ select.
#[tauri::command]
fn lane_analysis(
    candidate: Option<i64>,
    shared: tauri::State<'_, Arc<Shared>>,
) -> Option<LaneReport> {
    let live = shared.live.lock().unwrap().clone();
    let draft = current_draft(&shared);
    let champions = shared.champions.read().unwrap();
    let items = shared.items.read().unwrap();
    let stats = shared.stats.read().unwrap();

    let ctx = draft.as_ref().map(|d| {
        let ctx = draft_context(&shared, d, &champions, &**stats);
        match candidate {
            Some(id) => ctx.with_candidate(id),
            None => ctx,
        }
    });
    let game = live
        .as_ref()
        .map(|snapshot| &snapshot.game)
        .filter(|_| candidate.is_none());

    analysis::report(game, ctx.as_ref(), &champions, &items, &**stats)
}

/// A escaramuça que importa agora — uma só, para o overlay.
///
/// No painel cabem as cinco; por cima do jogo cabe uma, e ela tem de ser a
/// certa. Quem escolhe é `lanes::focus`, pelo objetivo mais próximo e pela
/// rota do jogador.
#[tauri::command]
fn skirmish_focus(shared: tauri::State<'_, Arc<Shared>>) -> Option<lanes::Focus> {
    let snapshot = shared.live.lock().unwrap().clone()?;

    let champions = shared.champions.read().unwrap();
    let items = shared.items.read().unwrap();
    let stats = shared.stats.read().unwrap();

    let report = analysis::report(
        Some(&snapshot.game),
        None,
        &champions,
        &items,
        &**stats,
    )?;

    let role = Role::from_lcu(&snapshot.game.me()?.position);
    lanes::focus(&report.skirmishes, &snapshot.timeline.upcoming, role)
}

/// Ouro dos dez. Investido é exato; ganho é estimativa, e o relatório diz qual
/// é qual.
#[tauri::command]
fn gold_report(shared: tauri::State<'_, Arc<Shared>>) -> Option<GoldReport> {
    let snapshot = shared.live.lock().unwrap().clone()?;
    Some(gold::report(&snapshot.game))
}

/// A leitura da última partida terminada.
#[tauri::command]
fn last_review(shared: tauri::State<'_, Arc<Shared>>) -> Option<Report> {
    shared.last_review.lock().unwrap().clone()
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct PoolPlan {
    /// Pontos fortes e fracos dos core da rota.
    analysis: poolgap::Analysis,
    /// Quem entra no próximo lugar, dada a pool inteira como está.
    candidates: Vec<poolgap::Candidate>,
}

/// O retrato dos core desta rota e os candidatos para o próximo lugar.
///
/// O front chama de novo a cada campeão aceito: cada lugar é escolhido pelo
/// que os anteriores ainda não cobrem.
#[tauri::command]
fn pool_suggestions(
    role: Option<String>,
    limit: Option<usize>,
    shared: tauri::State<'_, Arc<Shared>>,
) -> PoolPlan {
    let role = role.map(|r| Role::from_lcu(&r)).unwrap_or(Role::Unknown);
    let pool = shared.pool.read().unwrap();
    let ids = pool.ids_for_role(role);
    let core = pool.ids_for_role_tier(role, Tier::Core);
    let champions = shared.champions.read().unwrap();
    let stats = shared.stats.read().unwrap();

    PoolPlan {
        analysis: poolgap::analyze(&core, &champions, role),
        candidates: poolgap::suggest(&ids, &champions, role, &**stats, limit.unwrap_or(6)),
    }
}

/// Runa principal para o confronto da sua rota.
///
/// Sai de regras mecânicas sobre o matchup, não de winrate — `from_data` no
/// resultado é falso enquanto a base de partidas não existir, e é por ele que
/// a interface decide o que escrever.
#[tauri::command]
fn rune_plan(shared: tauri::State<'_, Arc<Shared>>) -> Option<runes::RunePlan> {
    let live = shared.live.lock().unwrap().clone();
    let draft = current_draft(&shared);

    let champions = shared.champions.read().unwrap();
    let items = shared.items.read().unwrap();
    let stats = shared.stats.read().unwrap();
    let rune_index = shared.runes.read().unwrap();

    let ctx = draft
        .as_ref()
        .map(|d| draft_context(&shared, d, &champions, &**stats));
    let report = analysis::report(
        live.as_ref().map(|s| &s.game),
        ctx.as_ref(),
        &champions,
        &items,
        &**stats,
    )?;

    // A rota do jogador é a que interessa; as outras quatro não mudam a runa
    // dele.
    let minha = report.lanes.iter().find(|lane| {
        live.as_ref()
            .and_then(|s| s.game.me())
            .map(|me| Role::from_lcu(&me.position) == lane.role)
            .unwrap_or_else(|| ctx.as_ref().is_some_and(|c| c.my_role == lane.role))
    })?;

    runes::plan(minha.ally, minha.enemy, minha.role, &champions, &rune_index)
}

#[tauri::command]
fn get_pool(shared: tauri::State<'_, Arc<Shared>>) -> Pool {
    shared.pool.read().unwrap().clone()
}

#[tauri::command]
fn set_pool(pool: Pool, shared: tauri::State<'_, Arc<Shared>>) -> Result<(), String> {
    storage::write_json(POOL_CACHE, &pool).map_err(|err| err.to_string())?;
    *shared.pool.write().unwrap() = pool;
    Ok(())
}

#[tauri::command]
fn get_overlay_prefs(shared: tauri::State<'_, Arc<Shared>>) -> OverlayPrefs {
    shared.overlay_prefs.read().unwrap().clone()
}

/// Troca o que o overlay mostra e avisa a janela na hora.
///
/// O evento é o que faz a mudança ser imediata: sem ele o jogador mexeria no
/// painel durante o draft e só veria o efeito na partida seguinte.
#[tauri::command]
fn set_overlay_prefs(
    prefs: OverlayPrefs,
    app: tauri::AppHandle,
    shared: tauri::State<'_, Arc<Shared>>,
) -> Result<(), String> {
    storage::write_json(PREFS_CACHE, &prefs).map_err(|err| err.to_string())?;
    *shared.overlay_prefs.write().unwrap() = prefs.clone();
    let _ = app.emit(watcher::EVENT_PREFS, &prefs);
    Ok(())
}

/// Iniciar com o Windows. Desligado até o jogador ligar: o Presage não se
/// instala no boot de ninguém por conta própria.
#[tauri::command]
fn get_autostart(app: tauri::AppHandle) -> bool {
    use tauri_plugin_autostart::ManagerExt;
    app.autolaunch().is_enabled().unwrap_or(false)
}

#[tauri::command]
fn set_autostart(enabled: bool, app: tauri::AppHandle) -> Result<(), String> {
    use tauri_plugin_autostart::ManagerExt;
    let gerente = app.autolaunch();
    let feito = if enabled { gerente.enable() } else { gerente.disable() };
    feito.map_err(|err| err.to_string())
}

fn mostrar_janela(app: &tauri::AppHandle) {
    if let Some(janela) = app.get_webview_window("main") {
        let _ = janela.unminimize();
        let _ = janela.show();
        let _ = janela.set_focus();
    }
}

/// Ícone na bandeja: trazer a janela de volta e sair.
fn bandeja(app: &tauri::App) -> tauri::Result<()> {
    use tauri::menu::{Menu, MenuItem};
    use tauri::tray::TrayIconBuilder;

    let mostrar = MenuItem::with_id(app, "mostrar", "Mostrar o Presage", true, None::<&str>)?;
    let sair = MenuItem::with_id(app, "sair", "Sair", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&mostrar, &sair])?;

    let mut icone = TrayIconBuilder::with_id("presage")
        .tooltip("Presage")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "mostrar" => mostrar_janela(app),
            "sair" => app.exit(0),
            _ => {}
        });
    if let Some(imagem) = app.default_window_icon() {
        icone = icone.icon(imagem.clone());
    }
    icone.build(app)?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "presage_lib=debug,warn".into()),
        )
        .init();

    // O atualizador não pode impedir o app de abrir.
    //
    // O Tauri recusa endpoint que não seja https, e a recusa acontece na
    // inicialização do plugin — ou seja, derruba o processo inteiro. Numa
    // ferramenta que alguém instalou para usar, isso é o pior resultado
    // possível: um erro de digitação no endereço deixa todo mundo sem app, e
    // como o atualizador morreu junto não há como consertar remotamente.
    //
    // Então: tenta com ele, e se a configuração dele estiver errada, sobe sem.
    // Fica sem atualização automática, que é ruim — mas abre, que é o que
    // importa.
    if let Err(err) = montar(true).run(tauri::generate_context!()) {
        let texto = err.to_string();
        if !erro_do_atualizador(&texto) {
            panic!("falha ao iniciar o Presage: {texto}");
        }
        tracing::error!(
            erro = %texto,
            "configuração do atualizador recusada; abrindo sem atualização automática"
        );
        montar(false)
            .run(tauri::generate_context!())
            .expect("falha ao iniciar o Presage");
    }
}

/// A falha veio do atualizador?
///
/// É um casamento de texto, e por isso é a parte mais frágil do conserto: se a
/// mensagem do Tauri mudar, o app volta a morrer na inicialização. Daí o teste
/// abaixo prender a mensagem real, copiada de um crash de verdade.
///
/// Larga de propósito do lado de reconhecer: ficar sem atualização automática
/// é ruim, o app não abrir é pior. Mas não tão larga a ponto de engolir
/// qualquer erro — um defeito nosso tem de aparecer alto.
fn erro_do_atualizador(texto: &str) -> bool {
    let baixo = texto.to_lowercase();
    baixo.contains("updater") || baixo.contains("plugins.updater")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Copiada de um crash de verdade: endpoint `http://` numa build release.
    const MENSAGEM_REAL: &str = "PluginInitialization(\"updater\", \"Error deserializing \
        'plugins.updater' within your Tauri configuration: The configured updater endpoint \
        must use a secure protocol like `https`.\")";

    #[test]
    fn reconhece_a_mensagem_real_do_tauri() {
        assert!(
            erro_do_atualizador(MENSAGEM_REAL),
            "sem isto o app volta a morrer na inicialização"
        );
    }

    #[test]
    fn reconhece_outras_falhas_do_mesmo_plugin() {
        for texto in [
            "PluginInitialization(\"updater\", \"missing field `pubkey`\")",
            "failed to initialize plugin updater",
            "Error deserializing 'plugins.updater'",
        ] {
            assert!(erro_do_atualizador(texto), "{texto}");
        }
    }

    #[test]
    fn nao_engole_erro_que_nao_e_do_atualizador() {
        // O risco do outro lado: esconder um defeito nosso atrás de uma
        // segunda tentativa silenciosa. Estes têm de continuar derrubando o
        // app, alto e claro.
        for texto in [
            "PluginInitialization(\"global-shortcut\", \"atalho em uso\")",
            "webview2 runtime not found",
            "failed to create window",
            "",
        ] {
            assert!(!erro_do_atualizador(texto), "{texto}");
        }
    }
}

fn montar(com_atualizador: bool) -> tauri::Builder<tauri::Wry> {
    let mut builder = tauri::Builder::default();

    if com_atualizador {
        // Os dois andam juntos: um baixa e instala, o outro reinicia depois.
        builder = builder
            .plugin(tauri_plugin_updater::Builder::new().build())
            .plugin(tauri_plugin_process::init());
    }

    builder
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    if event.state() == tauri_plugin_global_shortcut::ShortcutState::Pressed {
                        overlay::on_shortcut(app, shortcut);
                    }
                })
                .build(),
        )
        .setup(|app| {
            let shared = Arc::new(Shared::new());
            if let Some(saved) = storage::read_json::<Pool>(POOL_CACHE) {
                tracing::info!(campeoes = saved.entries.len(), "pool carregado do disco");
                *shared.pool.write().unwrap() = saved;
            }

            // Base de partidas, se alguém já a colocou no lugar. `is_usable`
            // recusa arquivo truncado: melhor ficar sem base do que recomendar
            // a partir de meia dúzia de linhas.
            match storage::read_json::<Dataset>(DATASET_CACHE) {
                Some(dataset) if dataset.is_usable() => {
                    tracing::info!(
                        patch = %dataset.patch,
                        partidas = dataset.matches,
                        "base de partidas carregada"
                    );
                    *shared.dataset.write().unwrap() = Some(dataset);
                }
                Some(_) => tracing::warn!("base de partidas incompleta, ignorada"),
                None => tracing::info!("sem base de partidas: matchup e sinergia ficam em zero"),
            }
            rebuild_stats(&shared, PersonalStats::default());

            if let Some(saved) = storage::read_json::<Report>(recorder::LAST_REVIEW) {
                *shared.last_review.lock().unwrap() = Some(saved);
            }

            if let Some(saved) = storage::read_json::<DraftPrefs>(DRAFT_PREFS_CACHE) {
                *shared.draft_prefs.write().unwrap() = saved;
            }

            if let Some(saved) = storage::read_json::<OverlayPrefs>(PREFS_CACHE) {
                tracing::info!(
                    escondidas = saved.hidden.len(),
                    "preferências do overlay carregadas"
                );
                *shared.overlay_prefs.write().unwrap() = saved;
            }

            app.manage(shared.clone());

            if let Err(err) = bandeja(app) {
                tracing::warn!(%err, "sem ícone na bandeja");
            }

            overlay::prepare(app.handle());
            {
                use tauri_plugin_global_shortcut::GlobalShortcutExt;
                let shortcuts = app.global_shortcut();
                for shortcut in [overlay::toggle_shortcut(), overlay::interact_shortcut()] {
                    if let Err(err) = shortcuts.register(shortcut) {
                        tracing::warn!(%err, "atalho global já em uso por outro app");
                    }
                }
            }

            tauri::async_runtime::spawn(watcher::run(app.handle().clone(), shared.clone()));

            let handle = app.handle().clone();
            let champions_shared = shared.clone();
            tauri::async_runtime::spawn(async move {
                match ddragon::load_champions("pt_BR").await {
                    Ok(index) => {
                        tracing::info!(
                            version = %index.version,
                            campeoes = index.by_id.len(),
                            "campeões carregados"
                        );
                        *champions_shared.champions.write().unwrap() = index;
                    }
                    Err(err) => tracing::error!(%err, "sem dados de campeão, motor fica inativo"),
                }

                match ddragon::load_items("pt_BR").await {
                    Ok(index) => {
                        tracing::info!(itens = index.by_id.len(), "itens carregados");
                        *champions_shared.items.write().unwrap() = index;
                    }
                    Err(err) => tracing::error!(%err, "sem dados de item, build adaptativa inativa"),
                }

                match ddragon::load_runes("pt_BR").await {
                    Ok(index) => {
                        tracing::info!(runas = index.by_key.len(), "runas carregadas");
                        *champions_shared.runes.write().unwrap() = index;
                    }
                    Err(err) => tracing::warn!(%err, "sem dados de runa, sugestão de runa inativa"),
                }

                let _ = handle.emit(EVENT_ENGINE, engine_status_of(&champions_shared));
            });

            let handle = app.handle().clone();
            let profile_shared = shared.clone();
            tauri::async_runtime::spawn(async move {
                match profile::refresh(profile_shared.clone()).await {
                    Ok(stats) => {
                        tracing::info!(jogos = stats.total_games(), "histórico pessoal carregado");
                        rebuild_stats(&profile_shared, stats);
                    }
                    Err(err) => tracing::info!(%err, "componente pessoal desligado"),
                }
                let _ = handle.emit(EVENT_ENGINE, engine_status_of(&profile_shared));
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_snapshot,
            engine_status,
            lane_profile,
            rank_draft,
            set_draft_role,
            set_enemy_role,
            get_enemy_roles,
            set_simulated_draft,
            get_draft_prefs,
            set_draft_prefs,
            minimax_draft,
            ban_suggestions,
            get_pool,
            set_pool,
            get_live,
            build_suggestions,
            lane_analysis,
            skirmish_focus,
            gold_report,
            last_review,
            pool_suggestions,
            rune_plan,
            get_overlay_prefs,
            set_overlay_prefs,
            get_autostart,
            set_autostart
        ])
}
