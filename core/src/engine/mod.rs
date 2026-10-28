pub mod build;
pub mod champion;
pub mod coach;
pub mod comp;
pub mod context;
pub mod damage;
pub mod dataset;
pub mod gold;
pub mod items;
pub mod lanes;
pub mod minimax;
pub mod poolgap;
pub mod profile;
pub mod roles;
pub mod priority;
pub mod runes;
pub mod stats;
pub mod timeline;
pub mod wincon;

use serde::Serialize;

use crate::draft::DraftState;
use champion::{Champion, ChampionIndex, Role};
use context::DraftContext;
use stats::{
    expected_against, expected_together, posterior_sd, shrink, shrunk_delta, strength_for_sd,
    StatsStore,
};

const PRIOR_RATE: f64 = 0.50;

const BASE_STRENGTH: f64 = 400.0;

/// Dispersão típica de cada efeito na população. Sem amostra, é esta a nossa
/// incerteza sobre o componente — não zero. A força do shrinkage sai daqui
/// (`strength_for_sd`), para a estimativa e o `±` contarem a mesma história.
const MATCHUP_PRIOR_SD: f64 = 0.030;
const SYNERGY_PRIOR_SD: f64 = 0.012;
const PERSONAL_PRIOR_SD: f64 = 0.035;
const COMP_SD: f64 = 0.010;

/// Pesos escolhidos à mão, calibrados para que uma correção forte valha ~1,5pp
/// de winrate. São priors, não coeficientes ajustados — saem quando a regressão
/// sobre partidas reais entrar no lugar.
const W_DAMAGE_BALANCE: f64 = 0.050;
const W_FRONTLINE: f64 = 0.016;

const MAX_REASONS: usize = 3;

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Scored {
    pub champion_id: i64,
    pub score: f64,
    /// Desvio em pontos percentuais contra uma partida neutra. É este número
    /// que aparece embaixo do retrato.
    pub delta: f64,
    pub winrate: f64,
    pub confidence: f64,
    pub sample: u32,
    pub in_pool: bool,
    pub reasons: Vec<String>,
}

/// O ranking com a rota que o cliente informa. O app usa `rank_with`, que
/// recebe a rota já resolvida (cliente, escolha na tela ou pool).
pub fn rank(
    draft: &DraftState,
    index: &ChampionIndex,
    store: &dyn StatsStore,
    pool: &[i64],
) -> Vec<Scored> {
    let role = Role::from_lcu(&draft.local_position);
    rank_with(&DraftContext::new(draft, index, store, pool, role), index, store)
}

/// Só os candidatos do contexto entram: disponíveis, e que jogam a sua rota
/// ou estão na sua pool.
pub fn rank_with(ctx: &DraftContext, index: &ChampionIndex, store: &dyn StatsStore) -> Vec<Scored> {
    if index.is_empty() {
        return Vec::new();
    }

    let role = ctx.my_role;
    let team = comp::profile(&ctx.ally_ids(), index);
    // Cada aliado com a winrate base dele na rota dele — a referência contra a
    // qual a sinergia é medida.
    let ally_bases: Vec<(i64, f64)> = ctx
        .allies
        .iter()
        .map(|p| (p.champion_id, base_rate(store, p.champion_id, p.role)))
        .collect();
    let lane_opponent = ctx.lane_opponent().and_then(|p| {
        index.get(p.champion_id).map(|champion| LaneOpponent {
            id: p.champion_id,
            name: champion.name.clone(),
            base: base_rate(store, p.champion_id, role),
        })
    });
    let lanes = store.lane_profile();

    let mut scored: Vec<Scored> = ctx
        .candidates
        .iter()
        .filter_map(|&id| index.get(id).map(|champion| (id, champion)))
        .map(|(id, champion)| {
            score_one(
                champion,
                role,
                &team,
                &ally_bases,
                lane_opponent.as_ref(),
                store,
                ctx.pool.contains(&id),
                lanes.as_ref().and_then(|profile| profile.champion(id)),
            )
        })
        .collect();

    scored.sort_by(|a, b| {
        b.winrate
            .partial_cmp(&a.winrate)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(b.in_pool.cmp(&a.in_pool))
            .then(a.champion_id.cmp(&b.champion_id))
    });

    scored
}

fn score_one(
    champion: &Champion,
    role: Role,
    team: &comp::TeamProfile,
    allies: &[(i64, f64)],
    lane_opponent: Option<&LaneOpponent>,
    store: &dyn StatsStore,
    in_pool: bool,
    lane: Option<&profile::ChampionLane>,
) -> Scored {
    let base = shrink(store.base(champion.id, role), PRIOR_RATE, BASE_STRENGTH);

    // Matchup e sinergia são medidos contra o que as duas forças já explicam,
    // não contra a sua base — senão a força do outro vira efeito, na proporção
    // da amostra.
    let matchup = lane_opponent.map(|opponent| {
        shrunk_delta(
            store.matchup(champion.id, opponent.id, role),
            expected_against(base.rate, opponent.base),
            strength_for_sd(MATCHUP_PRIOR_SD),
        )
    });

    let synergies: Vec<stats::Estimate> = allies
        .iter()
        .map(|&(ally, ally_base)| {
            shrunk_delta(
                store.synergy(champion.id, ally),
                expected_together(base.rate, ally_base),
                strength_for_sd(SYNERGY_PRIOR_SD),
            )
        })
        .collect();

    // Vazamento conhecido, mantido de propósito: o registro pessoal carrega a
    // fase da conta, e uma sequência boa entra em cada campeão na proporção
    // do quanto ele foi jogado. Medir contra a fase da conta resolve isso mas
    // zera o campeão principal de quem só joga um, empatando-o com campeões
    // nunca jogados — falta um termo de experiência (nunca jogado < sua
    // média) que nenhum dado calibra ainda. Entre os dois erros, este ordena
    // melhor.
    let personal_record = store.personal(champion.id);
    let personal = shrunk_delta(
        personal_record,
        base.rate,
        strength_for_sd(PERSONAL_PRIOR_SD),
    );

    let balance_gain = comp::damage_balance_gain(team, champion);
    let frontline_fit = if champion.is_frontline() {
        comp::frontline_need(team)
    } else {
        0.0
    };
    let comp_delta = W_DAMAGE_BALANCE * balance_gain + W_FRONTLINE * frontline_fit;

    let matchup_delta = matchup.map(|m| m.rate).unwrap_or(0.0);
    let synergy_delta: f64 = synergies.iter().map(|s| s.rate).sum();

    let winrate =
        (base.rate + matchup_delta + synergy_delta + personal.rate + comp_delta).clamp(0.25, 0.75);

    let mut variance = base.se().powi(2) + COMP_SD.powi(2);
    variance += posterior_sd(matchup.map_or(0, |m| m.games), MATCHUP_PRIOR_SD).powi(2);
    for synergy in &synergies {
        variance += posterior_sd(synergy.games, SYNERGY_PRIOR_SD).powi(2);
    }
    variance += posterior_sd(personal.games, PERSONAL_PRIOR_SD).powi(2);

    let reasons = build_reasons(
        champion,
        team,
        balance_gain,
        frontline_fit,
        matchup.as_ref().zip(lane_opponent),
        personal_record,
        &personal,
        lane,
        store.has_global(),
    );

    Scored {
        champion_id: champion.id,
        score: (winrate * 1000.0).round() / 10.0,
        delta: ((winrate - PRIOR_RATE) * 1000.0).round() / 10.0,
        winrate,
        confidence: (1.96 * variance.sqrt() * 1000.0).round() / 10.0,
        sample: base.games,
        in_pool,
        reasons,
    }
}

fn build_reasons(
    champion: &Champion,
    team: &comp::TeamProfile,
    balance_gain: f64,
    frontline_fit: f64,
    matchup: Option<(&stats::Estimate, &LaneOpponent)>,
    personal_record: Option<stats::Record>,
    personal: &stats::Estimate,
    lane: Option<&profile::ChampionLane>,
    has_global: bool,
) -> Vec<String> {
    let mut reasons = Vec::new();

    if let Some((estimate, opponent)) = matchup {
        if estimate.games > 0 && estimate.rate.abs() > 0.005 {
            let verbo = if estimate.rate > 0.0 { "Vence" } else { "Perde para" };
            reasons.push(format!(
                "{verbo} {} na rota ({:+.1}pp, n={})",
                opponent.name,
                estimate.rate * 100.0,
                estimate.games
            ));
        }
    }

    if balance_gain > 0.01 {
        reasons.push(format!(
            "Equilibra o dano: {:.0}% → {:.0}% físico",
            team.physical_share * 100.0,
            comp::share_after(team, champion) * 100.0
        ));
    }

    if frontline_fit > 0.0 {
        reasons.push("Entra como a frontline que o time não tem".to_string());
    }

    if let Some(record) = personal_record {
        if record.games >= 5 {
            reasons.push(format!(
                "Seus jogos: {}V-{}D ({:+.1}pp)",
                record.wins,
                record.games - record.wins,
                personal.rate * 100.0
            ));
        }
    }

    // Só o que sai do ruído vira razão, e só como texto: sem a base para
    // converter tropa em vitória, a rota não entra no número.
    if let Some(reading) = lane.and_then(|l| l.clearest()) {
        reasons.push(format!(
            "Sua rota com ele: {} contra o oponente (n={})",
            reading.metric.phrase(reading.value),
            reading.games
        ));
    }

    if reasons.is_empty() && !has_global {
        reasons.push("Sem base estatística ainda — ordenado só por composição".to_string());
    }

    reasons.truncate(MAX_REASONS);
    reasons
}

fn base_rate(store: &dyn StatsStore, champion: i64, role: Role) -> f64 {
    shrink(store.base(champion, role), PRIOR_RATE, BASE_STRENGTH).rate
}

struct LaneOpponent {
    id: i64,
    name: String,
    /// Winrate base dele na rota: a referência do matchup.
    base: f64,
}

#[cfg(test)]
mod tests {
    use super::champion::fake;
    use super::stats::Record;
    use super::*;
    use crate::draft::DraftSlot;
    use std::collections::HashMap;

    fn slot(cell_id: i64, position: &str, champion_id: i64, is_local: bool) -> DraftSlot {
        DraftSlot {
            cell_id,
            position: position.to_string(),
            champion_id,
            intent_id: 0,
            spell1: 4,
            spell2: 14,
            is_local,
            is_acting: false,
            locked: champion_id > 0,
        }
    }

    fn index() -> ChampionIndex {
        let champions = vec![
            fake(1, "Marks", "Marksman", 9.0, 2.0, 3.0),
            fake(2, "Lutador", "Fighter", 8.0, 3.0, 5.0),
            fake(3, "Assassino", "Assassin", 9.0, 3.0, 3.0),
            fake(4, "Mago", "Mage", 2.0, 9.0, 4.0),
            // Tanque de meio, com Mage como segunda classe — sem ela, a regra de
            // rota o tiraria da lista do meio.
            {
                let mut tanque = fake(5, "Tanque", "Tank", 5.0, 6.0, 9.0);
                tanque.tags.push("Mage".into());
                tanque
            },
            fake(6, "Inimigo", "Mage", 3.0, 8.0, 4.0),
            fake(7, "AtiradorLivre", "Marksman", 9.0, 2.0, 3.0),
        ];
        ChampionIndex {
            version: "test".into(),
            by_id: champions.into_iter().map(|c| (c.id, c)).collect::<HashMap<_, _>>(),
        }
    }

    fn draft() -> DraftState {
        DraftState {
            phase: "BAN_PICK".into(),
            time_left_ms: 27_000,
            total_time_ms: 30_000,
            local_cell_id: 0,
            local_position: "middle".into(),
            my_team: vec![
                slot(0, "middle", 0, true),
                slot(1, "top", 2, false),
                slot(2, "jungle", 3, false),
                slot(3, "bottom", 1, false),
            ],
            their_team: vec![slot(5, "middle", 6, false)],
            my_bans: vec![],
            their_bans: vec![],
            current_action: Some("pick".into()),
            is_local_turn: true,
            unavailable: vec![1, 2, 3, 6],
            }
    }

    struct FakeStats;
    impl StatsStore for FakeStats {
        fn base(&self, champion: i64, _role: Role) -> Option<Record> {
            (champion == 4).then(|| Record::new(20_000, 11_000))
        }
        fn matchup(&self, champion: i64, opponent: i64, _role: Role) -> Option<Record> {
            (champion == 4 && opponent == 6).then(|| Record::new(5_000, 3_000))
        }
        fn synergy(&self, _champion: i64, _ally: i64) -> Option<Record> {
            None
        }
        fn personal(&self, _champion: i64) -> Option<Record> {
            None
        }
        fn has_global(&self) -> bool {
            true
        }
    }

    #[test]
    fn quem_nao_joga_a_rota_fica_de_fora() {
        // O atirador está disponível, mas o seu lugar é o meio.
        let scored = rank(&draft(), &index(), &stats::NoStats, &[]);
        assert!(!scored.iter().any(|s| s.champion_id == 7), "{scored:?}");
    }

    #[test]
    fn a_pool_passa_por_cima_da_regra_de_rota() {
        let scored = rank(&draft(), &index(), &stats::NoStats, &[7]);
        assert!(scored.iter().any(|s| s.champion_id == 7 && s.in_pool));
    }

    #[test]
    fn o_oponente_de_rota_e_achado_sem_posicao() {
        // Na ranqueada o cliente não diz a posição do inimigo. O mago deles
        // é inferido no meio, e o matchup passa a valer.
        let mut d = draft();
        d.their_team[0].position = String::new();
        let scored = rank(&d, &index(), &FakeStats, &[]);
        let mago = scored.iter().find(|s| s.champion_id == 4).unwrap();
        assert!(
            mago.reasons.iter().any(|r| r.starts_with("Vence Inimigo")),
            "{:?}",
            mago.reasons
        );
    }

    #[test]
    fn indisponiveis_ficam_de_fora() {
        let scored = rank(&draft(), &index(), &stats::NoStats, &[]);
        let mut ids: Vec<i64> = scored.iter().map(|s| s.champion_id).collect();
        ids.sort_unstable();
        assert_eq!(ids, vec![4, 5]);
    }

    #[test]
    fn frontline_pesa_mais_que_dano_em_time_sem_tanque() {
        // Aliados são Fighter + Assassin + Marksman: físico demais e sem
        // frontline. O tanque resolve o buraco maior, então vem primeiro.
        let scored = rank(&draft(), &index(), &stats::NoStats, &[]);
        assert_eq!(scored[0].champion_id, 5);
    }

    #[test]
    fn indice_vazio_nao_entra_em_panico() {
        assert!(rank(&draft(), &ChampionIndex::default(), &stats::NoStats, &[]).is_empty());
    }

    #[test]
    fn sem_estatistica_o_ranking_vem_da_composicao() {
        // Time aliado é Fighter + Assassin + Marksman: quase todo físico e sem
        // frontline. O mago equilibra o dano, o tanque preenche a frontline;
        // ambos têm de ficar acima de 50%.
        let scored = rank(&draft(), &index(), &stats::NoStats, &[]);
        for item in &scored {
            assert!(item.winrate > 0.5, "{:?}", item);
        }
    }

    #[test]
    fn sem_estatistica_a_razao_diz_isso() {
        let scored = rank(&draft(), &index(), &stats::NoStats, &[]);
        let tanque = scored.iter().find(|s| s.champion_id == 5).unwrap();
        assert!(tanque.reasons.iter().any(|r| r.contains("frontline")));
    }

    #[test]
    fn pool_e_marcado_sem_alterar_a_ordem() {
        let com_pool = rank(&draft(), &index(), &stats::NoStats, &[5]);
        let sem_pool = rank(&draft(), &index(), &stats::NoStats, &[]);
        assert!(com_pool.iter().find(|s| s.champion_id == 5).unwrap().in_pool);
        let ordem_a: Vec<i64> = com_pool.iter().map(|s| s.champion_id).collect();
        let ordem_b: Vec<i64> = sem_pool.iter().map(|s| s.champion_id).collect();
        assert_eq!(ordem_a, ordem_b);
    }

    #[test]
    fn sem_dados_o_intervalo_e_largo() {
        let scored = rank(&draft(), &index(), &stats::NoStats, &[]);
        assert!(scored[0].confidence > 8.0, "{}", scored[0].confidence);
        assert_eq!(scored[0].sample, 0);
    }

    #[test]
    fn com_dados_o_intervalo_encolhe() {
        let sem = rank(&draft(), &index(), &stats::NoStats, &[]);
        let com = rank(&draft(), &index(), &FakeStats, &[]);
        let mago_sem = sem.iter().find(|s| s.champion_id == 4).unwrap();
        let mago_com = com.iter().find(|s| s.champion_id == 4).unwrap();
        assert!(mago_com.confidence < mago_sem.confidence);
        assert_eq!(mago_com.sample, 20_000);
    }

    #[test]
    fn matchup_favoravel_vira_razao() {
        let scored = rank(&draft(), &index(), &FakeStats, &[]);
        let mago = scored.iter().find(|s| s.champion_id == 4).unwrap();
        assert!(
            mago.reasons.iter().any(|r| r.starts_with("Vence Inimigo")),
            "{:?}",
            mago.reasons
        );
    }

    #[test]
    fn historico_pessoal_entra_no_score_e_na_razao() {
        let mut pessoal = stats::PersonalStats::default();
        for i in 0..20 {
            pessoal.add_match(&format!("BR1_{i}"), 4, i < 15);
        }

        let sem = rank(&draft(), &index(), &stats::NoStats, &[]);
        let com = rank(&draft(), &index(), &pessoal, &[]);

        let mago_sem = sem.iter().find(|s| s.champion_id == 4).unwrap();
        let mago_com = com.iter().find(|s| s.champion_id == 4).unwrap();

        assert!(mago_com.winrate > mago_sem.winrate);
        assert!(
            mago_com.reasons.iter().any(|r| r.contains("15V-5D")),
            "{:?}",
            mago_com.reasons
        );
    }

    #[test]
    fn historico_pessoal_nao_preenche_matchup() {
        // Jogar muito não cria base de matchup: isso precisa de muitos
        // jogadores, não de muitas partidas suas.
        let mut pessoal = stats::PersonalStats::default();
        for i in 0..200 {
            pessoal.add_match(&format!("BR1_{i}"), 4, true);
        }
        let scored = rank(&draft(), &index(), &pessoal, &[]);
        let mago = scored.iter().find(|s| s.champion_id == 4).unwrap();

        assert_eq!(mago.sample, 0);
        assert!(!mago.reasons.iter().any(|r| r.contains("na rota")));
    }

    /// Base montada à mão, para testar a conta contra números conhecidos.
    #[derive(Default)]
    struct Montada {
        base: HashMap<(i64, Role), Record>,
        matchup: HashMap<(i64, i64), Record>,
        synergy: HashMap<(i64, i64), Record>,
        personal: HashMap<i64, Record>,
    }

    impl StatsStore for Montada {
        fn base(&self, champion: i64, role: Role) -> Option<Record> {
            self.base.get(&(champion, role)).copied()
        }
        fn matchup(&self, champion: i64, opponent: i64, _role: Role) -> Option<Record> {
            self.matchup.get(&(champion, opponent)).copied()
        }
        fn synergy(&self, champion: i64, ally: i64) -> Option<Record> {
            self.synergy
                .get(&(champion.min(ally), champion.max(ally)))
                .copied()
        }
        fn personal(&self, champion: i64) -> Option<Record> {
            self.personal.get(&champion).copied()
        }
        fn has_global(&self) -> bool {
            true
        }
    }

    fn winrate_de(scored: &[Scored], id: i64) -> f64 {
        scored.iter().find(|s| s.champion_id == id).unwrap().winrate
    }

    #[test]
    fn popularidade_nao_vira_matchup() {
        // O inimigo é fraco (47%), então qualquer um tem ~53% contra ele sem
        // nada de especial no confronto. Mago e Tanque têm exatamente essa taxa
        // — um com 5000 partidas, outro com 100. Nenhum dos dois tem matchup
        // de verdade, e acrescentar essas linhas não pode mexer em nenhum.
        let mut sem_matchup = Montada::default();
        sem_matchup.base.insert((4, Role::Mid), Record::new(20_000, 10_000));
        sem_matchup.base.insert((5, Role::Mid), Record::new(20_000, 10_000));
        sem_matchup.base.insert((6, Role::Mid), Record::new(40_000, 18_800));

        let mut com_matchup = Montada {
            base: sem_matchup.base.clone(),
            ..Default::default()
        };
        com_matchup.matchup.insert((4, 6), Record::new(5_000, 2_650));
        com_matchup.matchup.insert((5, 6), Record::new(100, 53));

        let antes = rank(&draft(), &index(), &sem_matchup, &[]);
        let depois = rank(&draft(), &index(), &com_matchup, &[]);

        for id in [4, 5] {
            let deslocamento = winrate_de(&depois, id) - winrate_de(&antes, id);
            assert!(
                deslocamento.abs() < 0.002,
                "campeão {id} andou {:+.2}pp só por ter amostra contra um inimigo fraco",
                deslocamento * 100.0
            );
        }
    }

    #[test]
    fn sinergia_nao_herda_a_forca_do_aliado() {
        // O Lutador (top, aliado já travado) é forte: 52%. Com um campeão de
        // 50%, sem interação nenhuma, a dupla ganha ~52%. Mago tem 900 jogos
        // com ele nessa taxa, Tanque 50: a força do Lutador não é sinergia.
        let mut sem_sinergia = Montada::default();
        sem_sinergia.base.insert((2, Role::Top), Record::new(40_000, 20_800));
        sem_sinergia.base.insert((4, Role::Mid), Record::new(20_000, 10_000));
        sem_sinergia.base.insert((5, Role::Mid), Record::new(20_000, 10_000));

        let mut com_sinergia = Montada {
            base: sem_sinergia.base.clone(),
            ..Default::default()
        };
        com_sinergia.synergy.insert((2, 4), Record::new(900, 468));
        com_sinergia.synergy.insert((2, 5), Record::new(50, 26));

        let antes = rank(&draft(), &index(), &sem_sinergia, &[]);
        let depois = rank(&draft(), &index(), &com_sinergia, &[]);

        for id in [4, 5] {
            let deslocamento = winrate_de(&depois, id) - winrate_de(&antes, id);
            assert!(
                deslocamento.abs() < 0.002,
                "campeão {id} andou {:+.2}pp herdando a força do aliado",
                deslocamento * 100.0
            );
        }
    }

    #[test]
    fn historico_pessoal_curto_nao_domina() {
        // 15V-5D em 20 partidas. Com o espalhamento pessoal que o próprio `±`
        // assume (3,5pp), a Beta move isso ~2pp — não 8.
        let mut pessoal = stats::PersonalStats::default();
        for i in 0..20 {
            pessoal.add_match(&format!("BR1_{i}"), 4, i < 15);
        }

        let sem = rank(&draft(), &index(), &stats::NoStats, &[]);
        let com = rank(&draft(), &index(), &pessoal, &[]);
        let deslocamento = winrate_de(&com, 4) - winrate_de(&sem, 4);

        assert!(
            deslocamento > 0.0 && deslocamento < 0.03,
            "20 partidas moveram {:+.2}pp",
            deslocamento * 100.0
        );
    }

    fn historico_de_rota(campeao: i64, centro: f64, n: usize) -> Vec<crate::history::LaneGame> {
        use crate::history::{LaneGame, Metric};
        const RUIDO: [f64; 6] = [3.0, -2.0, 1.0, -3.0, 2.0, -1.0];
        (0..n)
            .map(|i| LaneGame {
                champion: campeao,
                opponent: 900 + (i % 3) as i64,
                role: Role::Mid,
                won: true,
                diffs: std::collections::BTreeMap::from([(
                    Metric::Cs10,
                    centro + RUIDO[i % RUIDO.len()],
                )]),
            })
            .collect()
    }

    #[test]
    fn rota_clara_vira_razao_sem_mexer_na_ordem() {
        let mut pessoal = stats::PersonalStats::default();
        pessoal.lanes = historico_de_rota(4, 10.0, 20);
        pessoal.lanes.extend(historico_de_rota(5, -10.0, 20));

        let com = rank(&draft(), &index(), &pessoal, &[]);
        let sem = rank(&draft(), &index(), &stats::NoStats, &[]);

        let mago = com.iter().find(|s| s.champion_id == 4).unwrap();
        assert!(
            mago.reasons
                .iter()
                .any(|r| r.contains("tropas aos 10") && r.contains("n=20")),
            "{:?}",
            mago.reasons
        );
        // O perfil de rota é leitura, não peso: sem a base para converter
        // tropa em vitória, ele não entra no número.
        for (a, b) in com.iter().zip(&sem) {
            assert_eq!(a.champion_id, b.champion_id);
            assert_eq!(a.winrate, b.winrate);
        }
    }

    #[test]
    fn rota_dentro_do_ruido_nao_vira_razao() {
        let mut pessoal = stats::PersonalStats::default();
        pessoal.lanes = historico_de_rota(4, 0.5, 6);
        pessoal.lanes.extend(historico_de_rota(5, -0.5, 6));

        let scored = rank(&draft(), &index(), &pessoal, &[]);
        for s in &scored {
            assert!(
                !s.reasons.iter().any(|r| r.contains("Sua rota")),
                "{:?}",
                s.reasons
            );
        }
    }

    #[test]
    fn ordem_e_deterministica() {
        let a = rank(&draft(), &index(), &stats::NoStats, &[]);
        let b = rank(&draft(), &index(), &stats::NoStats, &[]);
        assert_eq!(a, b);
    }

    #[test]
    fn winrate_fica_dentro_dos_limites() {
        for item in rank(&draft(), &index(), &FakeStats, &[]) {
            assert!((0.25..=0.75).contains(&item.winrate), "{:?}", item);
        }
    }
}
