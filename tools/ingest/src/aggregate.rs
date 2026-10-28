//! Partida crua da match-v5 → contagens de vitória.
//!
//! Sem rede e sem disco. É aqui que a correção do dado mora, então é aqui que
//! estão os testes.
//!
//! O que entra na conta de cada partida aceita:
//!
//! - **base**: uma linha por participante, `campeão:rota`;
//! - **matchup**: o par de cada rota, nas duas direções — `A:B:rota` conta a
//!   vitória de A, `B:A:rota` a de B. É direcional de propósito, porque é
//!   assim que a chave é lida no app;
//! - **sinergia**: todo par de companheiros, sem direção (o menor id primeiro),
//!   dez por time, vinte por partida;
//! - **contra**: todo par de adversários, em qualquer rota, nas duas direções —
//!   `A:B` conta a vitória de A com B no time inimigo. Vinte e cinco pares,
//!   cinquenta entradas por partida;
//! - **duração**: uma linha por participante, `campeão:rota:faixa`, pela
//!   duração da partida. É o que mede a fase do jogo em que um campeão rende.
//!
//! O que *não* entra importa tanto quanto. Remake, partida terminada por erro,
//! modo que não é a Fenda, rota em branco: tudo isso é descartado com motivo,
//! porque uma base que engole remake mede a taxa de remake, não de vitória.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use presage_core::engine::champion::Role;
use presage_core::engine::dataset::{
    base_key, length_key, matchup_key, synergy_key, versus_key, Dataset,
};
use presage_core::engine::stats::{expected_against, Length, Record};

/// Abaixo disto é remake ou abandono precoce, independente das flags.
const MIN_DURATION: i64 = 300;

const TEAM_SIZE: usize = 5;
const PARTICIPANTS: usize = 10;

// ---- o que a API devolve ----
//
// Tudo com `default`: uma partida com um campo a menos é melhor descartada
// pela regra de negócio do que por um erro de desserialização no meio de um
// crawl de horas.

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Match {
    pub info: Info,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Info {
    pub game_duration: i64,
    pub game_mode: String,
    /// `15.21.704.1234`. As duas primeiras partes são o patch.
    pub game_version: String,
    pub queue_id: i64,
    /// `GameComplete` numa partida normal. Vazio em respostas antigas.
    pub end_of_game_result: String,
    pub participants: Vec<Participant>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Participant {
    pub champion_id: i64,
    pub team_id: i64,
    /// `TOP`, `JUNGLE`, `MIDDLE`, `BOTTOM`, `UTILITY`. A própria Riot recomenda
    /// este campo em vez de `individualPosition`.
    pub team_position: String,
    pub win: bool,
    pub game_ended_in_early_surrender: bool,
}

/// Por que uma partida não entrou.
///
/// Serializável porque o relatório de descartes sobrevive a uma retomada: um
/// crawl de horas que recomeça não pode perder a conta do que recusou.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Skip {
    Fila,
    Modo,
    Terminada,
    Remake,
    Curta,
    Incompleta,
    SemRota,
    RotaRepetida,
    Patch,
}

impl Skip {
    pub fn motivo(self) -> &'static str {
        match self {
            Skip::Fila => "fila diferente",
            Skip::Modo => "modo não é a Fenda",
            Skip::Terminada => "partida terminada por erro",
            Skip::Remake => "remake",
            Skip::Curta => "curta demais",
            Skip::Incompleta => "não tinha dez participantes",
            Skip::SemRota => "rota em branco",
            Skip::RotaRepetida => "rotas não fecham cinco por time",
            Skip::Patch => "outro patch",
        }
    }
}

/// `15.21.704.1234` → `15.21`
pub fn patch_of(game_version: &str) -> String {
    let mut partes = game_version.split('.');
    match (partes.next(), partes.next()) {
        (Some(maior), Some(menor)) if !maior.is_empty() && !menor.is_empty() => {
            format!("{maior}.{menor}")
        }
        _ => String::new(),
    }
}

#[derive(Debug, Clone)]
pub struct Filter {
    pub queue: i64,
    /// `None` trava no patch da primeira partida aceita.
    pub patch: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Counts {
    pub base: HashMap<String, Record>,
    pub matchup: HashMap<String, Record>,
    pub synergy: HashMap<String, Record>,
    pub versus: HashMap<String, Record>,
    pub length: HashMap<String, Record>,
    pub matches: u64,
    /// Partidas que entraram também em `versus` e `length`. Um estado salvo
    /// antes delas existirem tem menos que `matches`, e retomar dele deixaria
    /// as tabelas novas contando menos partidas do que o cabeçalho diz.
    pub versus_matches: u64,
    /// O patch travado, assim que a primeira partida é aceita.
    pub patch: String,
    pub skipped: HashMap<Skip, u64>,
}

fn bump(map: &mut HashMap<String, Record>, key: String, win: bool) {
    let entry = map.entry(key).or_insert(Record { games: 0, wins: 0 });
    entry.games += 1;
    if win {
        entry.wins += 1;
    }
}

impl Counts {
    /// Soma uma partida, ou diz por que ela ficou de fora.
    pub fn add(&mut self, info: &Info, filter: &Filter) -> Result<(), Skip> {
        let skip = |me: &mut Self, motivo: Skip| {
            *me.skipped.entry(motivo).or_insert(0) += 1;
            Err(motivo)
        };

        if info.queue_id != filter.queue {
            return skip(self, Skip::Fila);
        }
        if !info.game_mode.eq_ignore_ascii_case("CLASSIC") {
            return skip(self, Skip::Modo);
        }
        // Campo novo: vazio em respostas antigas, e aí a regra não se aplica.
        if !info.end_of_game_result.is_empty()
            && !info.end_of_game_result.eq_ignore_ascii_case("GameComplete")
        {
            return skip(self, Skip::Terminada);
        }
        if info.participants.len() != PARTICIPANTS {
            return skip(self, Skip::Incompleta);
        }
        if info
            .participants
            .iter()
            .any(|p| p.game_ended_in_early_surrender)
        {
            return skip(self, Skip::Remake);
        }
        if info.game_duration < MIN_DURATION {
            return skip(self, Skip::Curta);
        }

        let patch = patch_of(&info.game_version);
        let esperado = filter.patch.as_deref().or(if self.patch.is_empty() {
            None
        } else {
            Some(self.patch.as_str())
        });
        if let Some(esperado) = esperado {
            if patch != esperado {
                return skip(self, Skip::Patch);
            }
        }
        if patch.is_empty() {
            return skip(self, Skip::Patch);
        }

        // Resolve as rotas antes de mexer nos mapas: uma partida ou entra
        // inteira ou não entra, senão um descarte no meio deixa meia partida
        // contada.
        let mut por_rota: HashMap<Role, Vec<&Participant>> = HashMap::new();
        for p in &info.participants {
            let role = Role::from_lcu(&p.team_position);
            if role == Role::Unknown {
                return skip(self, Skip::SemRota);
            }
            por_rota.entry(role).or_default().push(p);
        }
        if por_rota.len() != TEAM_SIZE || por_rota.values().any(|lado| lado.len() != 2) {
            return skip(self, Skip::RotaRepetida);
        }

        let mut times: HashMap<i64, Vec<&Participant>> = HashMap::new();
        for p in &info.participants {
            times.entry(p.team_id).or_default().push(p);
        }
        if times.len() != 2 || times.values().any(|t| t.len() != TEAM_SIZE) {
            return skip(self, Skip::Incompleta);
        }

        // Daqui para baixo a partida está aceita e nada mais pode falhar.
        if self.patch.is_empty() {
            self.patch = patch;
        }
        self.matches += 1;

        for p in &info.participants {
            let role = Role::from_lcu(&p.team_position);
            bump(&mut self.base, base_key(p.champion_id, role), p.win);
        }

        for (role, dupla) in &por_rota {
            let (a, b) = (dupla[0], dupla[1]);
            if a.team_id == b.team_id {
                continue; // os dois da mesma rota no mesmo time: não é matchup
            }
            bump(
                &mut self.matchup,
                matchup_key(a.champion_id, b.champion_id, *role),
                a.win,
            );
            bump(
                &mut self.matchup,
                matchup_key(b.champion_id, a.champion_id, *role),
                b.win,
            );
        }

        for lado in times.values() {
            for (i, a) in lado.iter().enumerate() {
                for b in &lado[i + 1..] {
                    if a.champion_id == b.champion_id {
                        continue;
                    }
                    bump(
                        &mut self.synergy,
                        synergy_key(a.champion_id, b.champion_id),
                        a.win,
                    );
                }
            }
        }

        let lados: Vec<&Vec<&Participant>> = times.values().collect();
        for a in lados[0].iter() {
            for b in lados[1].iter() {
                bump(&mut self.versus, versus_key(a.champion_id, b.champion_id), a.win);
                bump(&mut self.versus, versus_key(b.champion_id, a.champion_id), b.win);
            }
        }

        let faixa = Length::of(info.game_duration);
        for p in &info.participants {
            let role = Role::from_lcu(&p.team_position);
            bump(&mut self.length, length_key(p.champion_id, role, faixa), p.win);
        }
        self.versus_matches += 1;

        Ok(())
    }

    /// O estado pode ser retomado: toda partida contada entrou também nas
    /// tabelas novas.
    pub fn resumable(&self) -> bool {
        self.versus_matches == self.matches
    }

    /// Fecha a base para o disco.
    ///
    /// `min_games` corta as entradas magras. Com a força de prior do motor, um
    /// matchup de dez partidas pesa ~3,5% contra o prior — ou seja, ocupa
    /// espaço sem dizer nada. Cortar não perde informação; só deixa de
    /// carregar ruído.
    pub fn into_dataset(
        self,
        region: String,
        tier: String,
        generated_at: String,
        min_games: u32,
    ) -> Dataset {
        let podar = |map: HashMap<String, Record>| -> HashMap<String, Record> {
            map.into_iter()
                .filter(|(_, r)| r.games >= min_games)
                .collect()
        };

        Dataset {
            patch: self.patch,
            region,
            tier,
            generated_at,
            matches: self.matches,
            base: podar(self.base),
            matchup: podar(self.matchup),
            synergy: podar(self.synergy),
            versus: podar(self.versus),
            length: podar(self.length),
        }
    }
}

/// O que a base cobre de verdade, para a pessoa decidir se ela já serve.
#[derive(Debug, Clone, PartialEq)]
pub struct Coverage {
    pub base: usize,
    pub matchup: usize,
    pub synergy: usize,
    pub versus: usize,
    pub length: usize,
    /// Entradas com amostra grande o bastante para mexer no ranking.
    pub matchup_util: usize,
    pub base_util: usize,
    pub versus_util: usize,
    /// O espalhamento de verdade dos efeitos de contra, em fração. `None` com
    /// poucas entradas úteis.
    pub versus_sd: Option<f64>,
}

/// Abaixo disto de entradas úteis, o espalhamento medido é ruído.
const MIN_SPREAD_ENTRIES: usize = 30;

/// Quanto os efeitos de contra se espalham além do que o ruído explica.
///
/// O resíduo de cada par é a taxa observada menos o que as duas forças já
/// explicam (`expected_against`, com a winrate de cada um somada nas cinco
/// rotas); o espalhamento é `τ² = Var(resíduos) − média(p(1 − p)/n)`. É o
/// número que o motor deve usar como `VERSUS_PRIOR_SD` — medido, não chutado.
fn versus_spread(dataset: &Dataset) -> Option<f64> {
    let mut forca: HashMap<i64, (f64, f64)> = HashMap::new();
    for (chave, r) in &dataset.base {
        if let Some(id) = chave.split(':').next().and_then(|s| s.parse::<i64>().ok()) {
            let e = forca.entry(id).or_insert((0.0, 0.0));
            e.0 += r.games as f64;
            e.1 += r.wins as f64;
        }
    }
    let taxa = |id: i64| forca.get(&id).filter(|(g, _)| *g > 0.0).map(|(g, w)| w / g);

    let mut residuos = Vec::new();
    let mut ruido = Vec::new();
    for (chave, r) in &dataset.versus {
        if r.games < USEFUL {
            continue;
        }
        let mut partes = chave.split(':').filter_map(|s| s.parse::<i64>().ok());
        let (Some(a), Some(b)) = (partes.next(), partes.next()) else {
            continue;
        };
        let (Some(pa), Some(pb)) = (taxa(a), taxa(b)) else {
            continue;
        };
        let p = r.wins as f64 / r.games as f64;
        residuos.push(p - expected_against(pa, pb));
        ruido.push(p * (1.0 - p) / r.games as f64);
    }
    if residuos.len() < MIN_SPREAD_ENTRIES {
        return None;
    }
    let n = residuos.len() as f64;
    let media = residuos.iter().sum::<f64>() / n;
    let var = residuos.iter().map(|d| (d - media).powi(2)).sum::<f64>() / (n - 1.0);
    let ruido_medio = ruido.iter().sum::<f64>() / n;
    Some((var - ruido_medio).max(0.0).sqrt())
}

/// A partir daqui o componente desloca a estimativa mais de um ponto
/// percentual, que é quando ele passa a importar.
pub const USEFUL: u32 = 200;

pub fn coverage(dataset: &Dataset) -> Coverage {
    let acima = |map: &HashMap<String, Record>| map.values().filter(|r| r.games >= USEFUL).count();

    Coverage {
        base: dataset.base.len(),
        matchup: dataset.matchup.len(),
        synergy: dataset.synergy.len(),
        versus: dataset.versus.len(),
        length: dataset.length.len(),
        matchup_util: acima(&dataset.matchup),
        base_util: acima(&dataset.base),
        versus_util: acima(&dataset.versus),
        versus_sd: versus_spread(dataset),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ROTAS: [&str; 5] = ["TOP", "JUNGLE", "MIDDLE", "BOTTOM", "UTILITY"];

    fn participante(champion_id: i64, team_id: i64, rota: &str, win: bool) -> Participant {
        Participant {
            champion_id,
            team_id,
            team_position: rota.into(),
            win,
            game_ended_in_early_surrender: false,
        }
    }

    /// Partida válida: time 100 com os campeões 1-5, time 200 com 11-15.
    fn partida(vitoria_do_100: bool) -> Info {
        let mut participants = Vec::new();
        for (i, rota) in ROTAS.iter().enumerate() {
            participants.push(participante(i as i64 + 1, 100, rota, vitoria_do_100));
            participants.push(participante(i as i64 + 11, 200, rota, !vitoria_do_100));
        }
        Info {
            game_duration: 1800,
            game_mode: "CLASSIC".into(),
            game_version: "15.21.704.1234".into(),
            queue_id: 420,
            end_of_game_result: "GameComplete".into(),
            participants,
        }
    }

    fn filtro() -> Filter {
        Filter {
            queue: 420,
            patch: None,
        }
    }

    #[test]
    fn o_patch_sai_das_duas_primeiras_partes() {
        assert_eq!(patch_of("15.21.704.1234"), "15.21");
        assert_eq!(patch_of("9.3.1"), "9.3");
        assert_eq!(patch_of("lixo"), "");
        assert_eq!(patch_of(""), "");
    }

    #[test]
    fn uma_partida_rende_cinquenta_linhas_de_contra() {
        let mut c = Counts::default();
        c.add(&partida(true), &filtro()).unwrap();

        // 5 × 5 pares, nas duas direções.
        assert_eq!(c.versus.len(), 50);
        // A:B e B:A somam uma vitória por partida.
        let ab = c.versus[&versus_key(1, 11)];
        let ba = c.versus[&versus_key(11, 1)];
        assert_eq!(ab.games, 1);
        assert_eq!(ab.wins + ba.wins, 1);
        // De rotas diferentes também: o topo deles contra o meio de vocês.
        assert!(c.versus.contains_key(&versus_key(3, 11)));
    }

    #[test]
    fn a_duracao_vai_para_a_faixa_da_partida() {
        let mut c = Counts::default();
        c.add(&partida(true), &filtro()).unwrap();
        assert_eq!(c.length.len(), 10);
        assert_eq!(c.length[&length_key(1, Role::Top, Length::Media)], Record::new(1, 1));
    }

    /// Oito campeões de 50%, com todo par de contra a 50% mais `efeito(a, b)`.
    fn base_de_contra(efeito: impl Fn(i64, i64) -> i32) -> Dataset {
        let mut base = HashMap::new();
        let mut versus = HashMap::new();
        for a in 1..=8 {
            base.insert(base_key(a, Role::Top), Record::new(10_000, 5_000));
            for b in 1..=8 {
                if a != b {
                    let vitorias = (5_000 + efeito(a, b)) as u32;
                    versus.insert(versus_key(a, b), Record::new(10_000, vitorias));
                }
            }
        }
        Dataset {
            patch: "15.21".into(),
            matches: 100_000,
            base,
            versus,
            ..Default::default()
        }
    }

    #[test]
    fn o_espalhamento_do_contra_e_medido_alem_do_ruido() {
        // Efeitos de ±3pp plantados: a medida tem de achar ~3pp.
        let plantado = base_de_contra(|a, b| if (a + b) % 2 == 0 { 300 } else { -300 });
        let sd = coverage(&plantado).versus_sd.unwrap();
        assert!((sd - 0.03).abs() < 0.002, "{sd}");

        // Todo par exatamente no esperado: espalhamento zero.
        let neutro = base_de_contra(|_, _| 0);
        assert_eq!(coverage(&neutro).versus_sd, Some(0.0));
    }

    #[test]
    fn estado_sem_as_tabelas_novas_nao_e_retomado() {
        let mut c = Counts::default();
        c.add(&partida(true), &filtro()).unwrap();
        assert!(c.resumable());

        // Como um estado salvo por uma versão anterior do coletor.
        c.versus_matches = 0;
        assert!(!c.resumable());
        assert!(Counts::default().resumable(), "estado vazio sempre retoma");
    }

    #[test]
    fn uma_partida_rende_dez_linhas_de_base() {
        let mut c = Counts::default();
        c.add(&partida(true), &filtro()).unwrap();

        assert_eq!(c.matches, 1);
        assert_eq!(c.base.len(), 10);
        assert_eq!(c.base[&base_key(1, Role::Top)], Record::new(1, 1));
        assert_eq!(c.base[&base_key(11, Role::Top)], Record::new(1, 0));
    }

    #[test]
    fn o_matchup_e_direcional_e_sai_nas_duas_direcoes() {
        let mut c = Counts::default();
        c.add(&partida(true), &filtro()).unwrap();

        // Cinco rotas, duas direções.
        assert_eq!(c.matchup.len(), 10);
        assert_eq!(
            c.matchup[&matchup_key(3, 13, Role::Mid)],
            Record::new(1, 1),
            "o campeão 3 ganhou"
        );
        assert_eq!(
            c.matchup[&matchup_key(13, 3, Role::Mid)],
            Record::new(1, 0),
            "e o 13 perdeu a mesma partida"
        );
    }

    #[test]
    fn a_sinergia_e_so_entre_companheiros() {
        let mut c = Counts::default();
        c.add(&partida(true), &filtro()).unwrap();

        // Dez pares por time, vinte na partida.
        assert_eq!(c.synergy.len(), 20);
        assert_eq!(c.synergy[&synergy_key(1, 2)], Record::new(1, 1));
        assert_eq!(c.synergy[&synergy_key(11, 12)], Record::new(1, 0));
        // 1 e 11 estavam em times opostos: não é observação de sinergia.
        assert!(!c.synergy.contains_key(&synergy_key(1, 11)));
    }

    #[test]
    fn a_sinergia_nao_depende_da_ordem_do_par() {
        let mut c = Counts::default();
        c.add(&partida(true), &filtro()).unwrap();
        assert_eq!(c.synergy[&synergy_key(2, 1)], c.synergy[&synergy_key(1, 2)]);
    }

    #[test]
    fn duas_partidas_somam_na_mesma_chave() {
        let mut c = Counts::default();
        c.add(&partida(true), &filtro()).unwrap();
        c.add(&partida(false), &filtro()).unwrap();

        assert_eq!(c.matches, 2);
        assert_eq!(c.base[&base_key(1, Role::Top)], Record::new(2, 1));
        assert_eq!(c.matchup[&matchup_key(1, 11, Role::Top)], Record::new(2, 1));
    }

    #[test]
    fn remake_fica_de_fora() {
        let mut info = partida(true);
        info.participants[0].game_ended_in_early_surrender = true;

        let mut c = Counts::default();
        assert_eq!(c.add(&info, &filtro()), Err(Skip::Remake));
        assert_eq!(c.matches, 0);
        assert!(c.base.is_empty(), "nada pode ter sido contado");
    }

    #[test]
    fn partida_curta_fica_de_fora_mesmo_sem_a_flag() {
        let mut info = partida(true);
        info.game_duration = 240;

        let mut c = Counts::default();
        assert_eq!(c.add(&info, &filtro()), Err(Skip::Curta));
    }

    #[test]
    fn fila_diferente_fica_de_fora() {
        let mut info = partida(true);
        info.queue_id = 450; // ARAM

        let mut c = Counts::default();
        assert_eq!(c.add(&info, &filtro()), Err(Skip::Fila));
    }

    #[test]
    fn modo_que_nao_e_a_fenda_fica_de_fora() {
        let mut info = partida(true);
        info.game_mode = "ARAM".into();

        let mut c = Counts::default();
        assert_eq!(c.add(&info, &filtro()), Err(Skip::Modo));
    }

    #[test]
    fn partida_terminada_por_erro_fica_de_fora() {
        let mut info = partida(true);
        info.end_of_game_result = "Abort_Unexpected".into();

        let mut c = Counts::default();
        assert_eq!(c.add(&info, &filtro()), Err(Skip::Terminada));
    }

    #[test]
    fn resposta_sem_o_campo_novo_nao_e_recusada_por_causa_dele() {
        // `endOfGameResult` é recente. Partida antiga vem sem, e isso não pode
        // significar "terminada por erro".
        let mut info = partida(true);
        info.end_of_game_result = String::new();

        let mut c = Counts::default();
        assert!(c.add(&info, &filtro()).is_ok());
    }

    #[test]
    fn rota_em_branco_derruba_a_partida_inteira() {
        let mut info = partida(true);
        info.participants[4].team_position = String::new();

        let mut c = Counts::default();
        assert_eq!(c.add(&info, &filtro()), Err(Skip::SemRota));
        assert!(
            c.base.is_empty(),
            "meia partida contada é pior que partida nenhuma"
        );
    }

    #[test]
    fn rotas_que_nao_fecham_cinco_por_time_ficam_de_fora() {
        let mut info = partida(true);
        info.participants[0].team_position = "MIDDLE".into(); // dois mid no time 100

        let mut c = Counts::default();
        assert_eq!(c.add(&info, &filtro()), Err(Skip::RotaRepetida));
    }

    #[test]
    fn partida_sem_dez_participantes_fica_de_fora() {
        let mut info = partida(true);
        info.participants.pop();

        let mut c = Counts::default();
        assert_eq!(c.add(&info, &filtro()), Err(Skip::Incompleta));
    }

    #[test]
    fn o_patch_trava_na_primeira_partida_aceita() {
        let mut c = Counts::default();
        c.add(&partida(true), &filtro()).unwrap();
        assert_eq!(c.patch, "15.21");

        let mut outra = partida(true);
        outra.game_version = "15.22.800.4321".into();
        assert_eq!(c.add(&outra, &filtro()), Err(Skip::Patch));
        assert_eq!(c.matches, 1, "misturar patch é o jeito de a base mentir");
    }

    #[test]
    fn patch_pedido_na_linha_de_comando_manda() {
        let filtro = Filter {
            queue: 420,
            patch: Some("15.22".into()),
        };
        let mut c = Counts::default();

        assert_eq!(c.add(&partida(true), &filtro), Err(Skip::Patch));

        let mut certa = partida(true);
        certa.game_version = "15.22.800.4321".into();
        assert!(c.add(&certa, &filtro).is_ok());
        assert_eq!(c.patch, "15.22");
    }

    #[test]
    fn versao_ilegivel_fica_de_fora() {
        let mut info = partida(true);
        info.game_version = "???".into();

        let mut c = Counts::default();
        assert_eq!(c.add(&info, &filtro()), Err(Skip::Patch));
    }

    #[test]
    fn os_descartes_sao_contados_por_motivo() {
        let mut c = Counts::default();
        let mut aram = partida(true);
        aram.game_mode = "ARAM".into();

        let _ = c.add(&aram, &filtro());
        let _ = c.add(&aram, &filtro());
        let mut curta = partida(true);
        curta.game_duration = 100;
        let _ = c.add(&curta, &filtro());

        assert_eq!(c.skipped[&Skip::Modo], 2);
        assert_eq!(c.skipped[&Skip::Curta], 1);
    }

    #[test]
    fn a_poda_tira_as_entradas_magras() {
        let mut c = Counts::default();
        for _ in 0..5 {
            c.add(&partida(true), &filtro()).unwrap();
        }

        let sem_poda = c.clone().into_dataset("br1".into(), "t".into(), "x".into(), 1);
        assert_eq!(sem_poda.base.len(), 10);

        let podada = c.into_dataset("br1".into(), "t".into(), "x".into(), 20);
        assert!(podada.base.is_empty());
        assert_eq!(podada.matches, 5, "a contagem de partidas não é podada");
    }

    #[test]
    fn a_base_gerada_e_a_que_o_app_le() {
        // O teste que amarra as duas pontas: as chaves saem de `presage-core`,
        // então o formato não pode divergir do leitor.
        use presage_core::engine::stats::StatsStore;

        let mut c = Counts::default();
        for _ in 0..30 {
            c.add(&partida(true), &filtro()).unwrap();
        }
        let dataset = c.into_dataset("br1".into(), "EMERALD".into(), "agora".into(), 20);

        assert!(dataset.is_usable());
        assert!(dataset.has_global());
        assert_eq!(
            dataset.matchup(3, 13, Role::Mid),
            Some(Record::new(30, 30))
        );
        assert_eq!(dataset.base(1, Role::Top), Some(Record::new(30, 30)));
        assert_eq!(dataset.synergy(1, 2), dataset.synergy(2, 1));
    }

    #[test]
    fn a_cobertura_separa_o_que_ja_serve() {
        let mut c = Counts::default();
        for _ in 0..(USEFUL as usize + 10) {
            c.add(&partida(true), &filtro()).unwrap();
        }
        // Uma partida de um par diferente, que fica com amostra minúscula.
        let mut rara = partida(true);
        for p in rara.participants.iter_mut() {
            p.champion_id += 100;
        }
        c.add(&rara, &filtro()).unwrap();

        let dataset = c.into_dataset("br1".into(), "t".into(), "x".into(), 1);
        let cob = coverage(&dataset);

        assert_eq!(cob.matchup, 20, "dez pares comuns mais dez raros");
        assert_eq!(cob.matchup_util, 10, "só os comuns passam do limiar");
    }
}
