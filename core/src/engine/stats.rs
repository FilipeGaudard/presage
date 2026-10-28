use super::champion::Role;
use super::profile::{self, LaneProfile};
use crate::history::{LaneGame, PlayedGame};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Record {
    pub games: u32,
    pub wins: u32,
}

impl Record {
    pub fn new(games: u32, wins: u32) -> Self {
        Record {
            games,
            wins: wins.min(games),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Estimate {
    pub rate: f64,
    pub effective_n: f64,
    pub games: u32,
}

impl Estimate {
    /// Erro padrão da proporção, usando o n efetivo (amostra + força do prior).
    pub fn se(&self) -> f64 {
        if self.effective_n <= 0.0 {
            return 0.5;
        }
        let p = self.rate.clamp(0.0, 1.0);
        (p * (1.0 - p) / self.effective_n).sqrt()
    }
}

/// Média posterior de uma Beta(prior * strength, (1 - prior) * strength)
/// atualizada pela amostra. Com zero jogos devolve o prior; com amostra grande
/// converge para a taxa observada.
pub fn shrink(record: Option<Record>, prior: f64, strength: f64) -> Estimate {
    let strength = strength.max(1.0);
    let (games, wins) = record
        .map(|r| (r.games as f64, r.wins as f64))
        .unwrap_or((0.0, 0.0));

    let effective_n = games + strength;
    let rate = ((wins + strength * prior) / effective_n).clamp(0.0, 1.0);

    Estimate {
        rate,
        effective_n,
        games: games as u32,
    }
}

/// Mesma conta, expressa como desvio em relação à linha de base. Sem amostra o
/// desvio é exatamente zero, que é o que queremos: componente sem dado não
/// mexe no ranking.
pub fn shrunk_delta(record: Option<Record>, baseline: f64, strength: f64) -> Estimate {
    let estimate = shrink(record, baseline, strength);
    Estimate {
        rate: estimate.rate - baseline,
        ..estimate
    }
}

/// Força da Beta, em partidas fictícias, cujo desvio padrão em torno de 50% é
/// `sd`.
///
/// Uma Beta de média p e força s tem variância p(1 − p)/(s + 1). Força e
/// desvio são a mesma crença dita de dois jeitos; guardados como constantes
/// separadas, a força puxava a estimativa e o desvio desenhava o `±`, e os dois
/// discordavam. Daqui sai uma só.
pub fn strength_for_sd(sd: f64) -> f64 {
    (0.25 / (sd * sd) - 1.0).max(1.0)
}

/// Desvio posterior de um componente com `games` partidas. Sem amostra é
/// exatamente `prior_sd`; depois cai com a raiz da amostra, como na Beta.
///
/// Usa p = 0,5 na variância: entre 45% e 55% o p(1 − p) muda menos de 1%.
pub fn posterior_sd(games: u32, prior_sd: f64) -> f64 {
    (0.25 / (strength_for_sd(prior_sd) + 1.0 + games as f64)).sqrt()
}

fn logit(p: f64) -> f64 {
    let p = p.clamp(1e-6, 1.0 - 1e-6);
    (p / (1.0 - p)).ln()
}

fn logistic(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}

/// Winrate esperada de um campeão contra outro se o confronto não tiver nada
/// de especial: só a força de cada um, somada em log-odds.
///
/// É contra isto que o matchup tem de ser medido. A taxa crua de A contra B já
/// carrega a fraqueza geral de B; medida contra a base de A, essa fraqueza vira
/// "matchup" na proporção da amostra, e o pick popular sobe por ser popular.
pub fn expected_against(champion: f64, opponent: f64) -> f64 {
    logistic(logit(champion) - logit(opponent))
}

/// Winrate esperada de dois campeões no mesmo time, sem interação entre eles.
/// Mesmo raciocínio: a força do aliado não é sinergia.
pub fn expected_together(champion: f64, ally: f64) -> f64 {
    logistic(logit(champion) + logit(ally))
}

pub trait StatsStore: Send + Sync {
    fn base(&self, champion: i64, role: Role) -> Option<Record>;
    fn matchup(&self, champion: i64, opponent: i64, role: Role) -> Option<Record>;
    fn synergy(&self, champion: i64, ally: i64) -> Option<Record>;
    fn personal(&self, champion: i64) -> Option<Record>;

    /// Base agregada sobre muitos jogadores: winrate por rota, matchup,
    /// sinergia. Vem do pipeline de ingestão.
    fn has_global(&self) -> bool {
        false
    }

    /// Histórico do próprio jogador. Vem da conta dele, e só alimenta o
    /// componente pessoal.
    fn has_personal(&self) -> bool {
        false
    }

    fn personal_games(&self) -> u32 {
        0
    }

    /// Quanto o jogador tira do oponente direto, a partir do próprio
    /// histórico.
    fn lane_profile(&self) -> Option<LaneProfile> {
        None
    }
}

pub struct NoStats;

impl StatsStore for NoStats {
    fn base(&self, _champion: i64, _role: Role) -> Option<Record> {
        None
    }
    fn matchup(&self, _champion: i64, _opponent: i64, _role: Role) -> Option<Record> {
        None
    }
    fn synergy(&self, _champion: i64, _ally: i64) -> Option<Record> {
        None
    }
    fn personal(&self, _champion: i64) -> Option<Record> {
        None
    }
}

/// O histórico do próprio jogador: vitórias e derrotas por campeão, e a
/// comparação com o oponente direto em cada partida.
///
/// Cobre só o componente pessoal, e de propósito: winrate base, matchup e
/// sinergia precisam de milhões de partidas de milhares de jogadores. Uma
/// conta sozinha não produz nenhum dos três — com 100 partidas você teria
/// menos de uma observação por matchup.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PersonalStats {
    pub puuid: String,
    pub by_champion: HashMap<i64, Record>,
    /// Partidas já contabilizadas, para a busca ser incremental.
    pub seen: Vec<String>,
    /// Uma por partida em que havia oponente direto.
    #[serde(default)]
    pub lanes: Vec<LaneGame>,
}

impl PersonalStats {
    /// Soma uma partida lida da match-v5. Vitória e derrota entram sempre; a
    /// comparação de rota, quando existe.
    pub fn add_played(&mut self, match_id: &str, game: PlayedGame) -> bool {
        if !self.add_match(match_id, game.champion, game.won) {
            return false;
        }
        if let Some(lane) = game.lane {
            self.lanes.push(lane);
        }
        true
    }

    /// Partida que não conta (remake, curta), marcada para a próxima
    /// atualização não buscá-la de novo.
    pub fn mark_seen(&mut self, match_id: &str) {
        if !self.seen.iter().any(|id| id == match_id) {
            self.seen.push(match_id.to_string());
        }
    }

    pub fn add_match(&mut self, match_id: &str, champion: i64, won: bool) -> bool {
        if self.seen.iter().any(|id| id == match_id) {
            return false;
        }
        self.seen.push(match_id.to_string());
        let entry = self
            .by_champion
            .entry(champion)
            .or_insert(Record { games: 0, wins: 0 });
        entry.games += 1;
        if won {
            entry.wins += 1;
        }
        true
    }

    pub fn total_games(&self) -> u32 {
        self.by_champion.values().map(|r| r.games).sum()
    }
}

impl StatsStore for PersonalStats {
    fn base(&self, _champion: i64, _role: Role) -> Option<Record> {
        None
    }
    fn matchup(&self, _champion: i64, _opponent: i64, _role: Role) -> Option<Record> {
        None
    }
    fn synergy(&self, _champion: i64, _ally: i64) -> Option<Record> {
        None
    }
    fn personal(&self, champion: i64) -> Option<Record> {
        self.by_champion.get(&champion).copied()
    }
    fn has_personal(&self) -> bool {
        !self.by_champion.is_empty()
    }
    fn personal_games(&self) -> u32 {
        self.total_games()
    }
    fn lane_profile(&self) -> Option<LaneProfile> {
        (!self.lanes.is_empty()).then(|| profile::build(&self.lanes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PRIOR: f64 = 0.5;

    #[test]
    fn sem_amostra_devolve_o_prior() {
        let e = shrink(None, PRIOR, 400.0);
        assert!((e.rate - PRIOR).abs() < 1e-12);
        assert_eq!(e.games, 0);
    }

    #[test]
    fn sem_amostra_o_desvio_e_zero() {
        let e = shrunk_delta(None, 0.52, 800.0);
        assert!(e.rate.abs() < 1e-12);
    }

    #[test]
    fn amostra_grande_domina_o_prior() {
        let e = shrink(Some(Record::new(100_000, 60_000)), PRIOR, 400.0);
        assert!((e.rate - 0.60).abs() < 0.01, "{}", e.rate);
    }

    #[test]
    fn amostra_pequena_fica_perto_do_prior() {
        // 10 jogos, 10 vitórias: sem shrinkage viraria 100%.
        let e = shrink(Some(Record::new(10, 10)), PRIOR, 400.0);
        assert!(e.rate < 0.52, "{}", e.rate);
    }

    #[test]
    fn erro_padrao_encolhe_com_a_amostra() {
        let pouco = shrink(Some(Record::new(50, 30)), PRIOR, 400.0);
        let muito = shrink(Some(Record::new(50_000, 30_000)), PRIOR, 400.0);
        assert!(muito.se() < pouco.se());
    }

    #[test]
    fn vitorias_acima_de_jogos_sao_cortadas() {
        let r = Record::new(10, 99);
        assert_eq!(r.wins, 10);
        let e = shrink(Some(r), PRIOR, 400.0);
        assert!((0.0..=1.0).contains(&e.rate));
    }

    #[test]
    fn forca_zero_nao_divide_por_zero() {
        let e = shrink(Some(Record::new(0, 0)), PRIOR, 0.0);
        assert!(e.rate.is_finite());
        assert!(e.se().is_finite());
    }

    #[test]
    fn forca_e_desvio_sao_a_mesma_crenca() {
        for sd in [0.012, 0.030, 0.035] {
            let s = strength_for_sd(sd);
            let desvio_da_beta = (0.25 / (s + 1.0)).sqrt();
            assert!((desvio_da_beta - sd).abs() < 1e-12, "{sd}: {desvio_da_beta}");
            assert!((posterior_sd(0, sd) - sd).abs() < 1e-12);
        }
    }

    #[test]
    fn desvio_posterior_cai_com_a_raiz_da_amostra() {
        // 4x a amostra, metade do desvio. Cair com 1/n encolheria o `±` quatro
        // vezes, e a tela prometeria uma certeza que a base não tem.
        let razao = posterior_sd(10_000, 0.03) / posterior_sd(40_000, 0.03);
        assert!((1.9..2.1).contains(&razao), "{razao}");
    }

    #[test]
    fn rival_e_aliado_neutros_nao_mudam_a_esperada() {
        assert!((expected_against(0.53, 0.5) - 0.53).abs() < 1e-12);
        assert!((expected_together(0.53, 0.5) - 0.53).abs() < 1e-12);
    }

    #[test]
    fn rival_fraco_sobe_a_esperada_e_aliado_forte_tambem() {
        assert!((expected_against(0.5, 0.47) - 0.53).abs() < 1e-12);
        assert!((expected_together(0.5, 0.52) - 0.52).abs() < 1e-12);
        assert!(expected_against(0.5, 0.53) < 0.5);
    }

    #[test]
    fn nostats_nao_tem_dado() {
        assert!(!NoStats.has_global());
        assert!(!NoStats.has_personal());
        assert!(NoStats.base(103, Role::Mid).is_none());
    }

    #[test]
    fn pessoal_acumula_vitorias_e_derrotas() {
        let mut stats = PersonalStats::default();
        stats.add_match("BR1_1", 103, true);
        stats.add_match("BR1_2", 103, false);
        stats.add_match("BR1_3", 64, true);

        assert_eq!(stats.personal(103), Some(Record { games: 2, wins: 1 }));
        assert_eq!(stats.personal(64), Some(Record { games: 1, wins: 1 }));
        assert_eq!(stats.total_games(), 3);
    }

    #[test]
    fn partida_repetida_nao_conta_duas_vezes() {
        let mut stats = PersonalStats::default();
        assert!(stats.add_match("BR1_1", 103, true));
        assert!(!stats.add_match("BR1_1", 103, true));
        assert_eq!(stats.total_games(), 1);
    }

    #[test]
    fn pessoal_nao_inventa_dado_global() {
        let mut stats = PersonalStats::default();
        stats.add_match("BR1_1", 103, true);

        assert!(stats.has_personal());
        assert!(!stats.has_global());
        assert!(stats.base(103, Role::Mid).is_none());
        assert!(stats.matchup(103, 238, Role::Mid).is_none());
        assert!(stats.synergy(103, 64).is_none());
    }

    #[test]
    fn partida_lida_soma_vitoria_e_rota() {
        use crate::history::Metric;
        use std::collections::BTreeMap;

        let mut stats = PersonalStats::default();
        let rota = LaneGame {
            champion: 103,
            opponent: 238,
            role: Role::Mid,
            won: true,
            diffs: BTreeMap::from([(Metric::Cs10, 7.0)]),
        };
        let jogo = PlayedGame {
            champion: 103,
            won: true,
            lane: Some(rota),
        };

        assert!(stats.add_played("BR1_1", jogo.clone()));
        assert!(!stats.add_played("BR1_1", jogo), "repetida não conta");
        assert_eq!(stats.personal(103), Some(Record::new(1, 1)));
        assert_eq!(stats.lanes.len(), 1);
    }

    #[test]
    fn partida_descartada_fica_vista_sem_contar() {
        let mut stats = PersonalStats::default();
        stats.mark_seen("BR1_9");
        stats.mark_seen("BR1_9");
        assert_eq!(stats.seen, vec!["BR1_9".to_string()]);
        assert_eq!(stats.total_games(), 0);
    }

    #[test]
    fn cache_antigo_carrega_sem_rotas() {
        let antigo = r#"{"puuid":"x","by_champion":{"103":{"games":2,"wins":1}},"seen":["BR1_1"]}"#;
        let stats: PersonalStats = serde_json::from_str(antigo).unwrap();
        assert!(stats.lanes.is_empty());
        assert_eq!(stats.personal(103), Some(Record::new(2, 1)));
        assert!(stats.lane_profile().is_none());
    }

    #[test]
    fn campeao_nunca_jogado_nao_tem_registro() {
        let mut stats = PersonalStats::default();
        stats.add_match("BR1_1", 103, true);
        assert!(stats.personal(777).is_none());
    }
}
