//! Perfil de rota do jogador: quanto você tira do oponente direto, no geral,
//! por campeão e por confronto.
//!
//! Três níveis, cada um encolhido para o de cima:
//!
//! - **você**: a média de todas as suas partidas. A referência é zero — a
//!   diferença contra o oponente some na população —, e com dezenas de
//!   partidas essa média já é medida, não palpite.
//! - **você com o campeão**: encolhido para a sua média.
//! - **você com o campeão contra aquele oponente**: encolhido para o campeão.
//!
//! Quanto cada nível encolhe não é escolhido à mão: sai do quanto as médias se
//! espalham além do que o ruído explicaria (DerSimonian–Laird). Se os seus
//! campeões não se distinguem acima do ruído, o perfil diz isso — todos
//! colapsam na sua média — em vez de inventar um especialista.
//!
//! O que ele não separa: quanto um campeão rende na rota por natureza. Quem
//! perde tropa aos 10 para quase todo mundo por desenho aparece aqui como
//! "você com ele". Separar exige a referência por campeão e por confronto que
//! só a base de partidas dá — o mesmo erro de referência que o ranking de
//! draft tinha.
//!
//! Nenhuma nota única sai daqui, de propósito: a política da Riot proíbe
//! substituto de ranking, e uma média das métricas seria exatamente isso.

use serde::Serialize;
use std::collections::BTreeMap;

use crate::history::{LaneGame, Metric};

/// Abaixo disto nem a média geral é medida, e a métrica fica de fora.
const MIN_GAMES: usize = 5;

/// Piso de variância: partidas todas iguais não podem dividir por zero.
const EPS: f64 = 1e-9;

#[derive(Debug, Clone, Copy, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Reading {
    pub metric: Metric,
    pub games: u32,
    /// Média crua das suas partidas.
    pub raw: f64,
    /// Depois de encolher para o nível de cima. É o número para mostrar.
    pub value: f64,
    /// Desvio padrão de `value`.
    pub sd: f64,
}

impl Reading {
    /// O intervalo de 95% não cruza o zero.
    pub fn is_clear(&self) -> bool {
        self.value.abs() > 1.96 * self.sd
    }

    fn z(&self) -> f64 {
        self.value.abs() / self.sd.max(EPS)
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MatchupLane {
    pub opponent: i64,
    pub games: u32,
    pub readings: Vec<Reading>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ChampionLane {
    pub champion: i64,
    pub games: u32,
    pub readings: Vec<Reading>,
    pub matchups: Vec<MatchupLane>,
}

impl ChampionLane {
    /// A leitura que mais se destaca do ruído, se alguma se destaca.
    pub fn clearest(&self) -> Option<&Reading> {
        self.readings
            .iter()
            .filter(|r| r.is_clear())
            .max_by(|a, b| a.z().total_cmp(&b.z()))
    }
}

#[derive(Debug, Clone, Default, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LaneProfile {
    pub games: u32,
    pub overall: Vec<Reading>,
    /// Do mais jogado para o menos.
    pub champions: Vec<ChampionLane>,
}

impl LaneProfile {
    pub fn champion(&self, id: i64) -> Option<&ChampionLane> {
        self.champions.iter().find(|c| c.champion == id)
    }
}

pub fn build(games: &[LaneGame]) -> LaneProfile {
    let mut overall = Vec::new();
    let mut champions: BTreeMap<i64, Vec<Reading>> = BTreeMap::new();
    let mut matchups: BTreeMap<(i64, i64), Vec<Reading>> = BTreeMap::new();

    for metric in Metric::ALL {
        let Some(levels) = estimate(games, metric) else {
            continue;
        };
        overall.push(levels.overall);
        for (champion, reading) in levels.champions {
            champions.entry(champion).or_default().push(reading);
        }
        for (cell, reading) in levels.matchups {
            matchups.entry(cell).or_default().push(reading);
        }
    }

    let mut champion_games: BTreeMap<i64, u32> = BTreeMap::new();
    let mut matchup_games: BTreeMap<(i64, i64), u32> = BTreeMap::new();
    for game in games {
        *champion_games.entry(game.champion).or_default() += 1;
        *matchup_games.entry((game.champion, game.opponent)).or_default() += 1;
    }

    let mut champions: Vec<ChampionLane> = champions
        .into_iter()
        .map(|(champion, readings)| {
            let mut cells: Vec<MatchupLane> = matchups
                .range((champion, i64::MIN)..=(champion, i64::MAX))
                .map(|(&(_, opponent), readings)| MatchupLane {
                    opponent,
                    games: matchup_games[&(champion, opponent)],
                    readings: readings.clone(),
                })
                .collect();
            cells.sort_by(|a, b| b.games.cmp(&a.games).then(a.opponent.cmp(&b.opponent)));
            ChampionLane {
                champion,
                games: champion_games[&champion],
                readings,
                matchups: cells,
            }
        })
        .collect();
    champions.sort_by(|a, b| b.games.cmp(&a.games).then(a.champion.cmp(&b.champion)));

    LaneProfile {
        games: games.len() as u32,
        overall,
        champions,
    }
}

struct Levels {
    overall: Reading,
    champions: Vec<(i64, Reading)>,
    matchups: Vec<((i64, i64), Reading)>,
}

/// O nível de cima, do ponto de vista de quem encolhe para ele.
struct Parent {
    raw: f64,
    value: f64,
    var: f64,
}

impl Parent {
    fn of(reading: &Reading) -> Self {
        Parent {
            raw: reading.raw,
            value: reading.value,
            var: reading.sd * reading.sd,
        }
    }
}

fn estimate(games: &[LaneGame], metric: Metric) -> Option<Levels> {
    let mut by_champion: BTreeMap<i64, Vec<f64>> = BTreeMap::new();
    let mut by_cell: BTreeMap<(i64, i64), Vec<f64>> = BTreeMap::new();
    let mut all = Vec::new();
    for game in games {
        if let Some(&diff) = game.diffs.get(&metric) {
            by_champion.entry(game.champion).or_default().push(diff);
            by_cell.entry((game.champion, game.opponent)).or_default().push(diff);
            all.push(diff);
        }
    }
    if all.len() < MIN_GAMES {
        return None;
    }

    let n = all.len() as f64;
    let mean = mean_of(&all);
    let spread = (squares(&all, mean) / (n - 1.0)).max(EPS);
    let overall = Reading {
        metric,
        games: all.len() as u32,
        raw: mean,
        value: mean,
        sd: (spread / n).sqrt(),
    };

    // Ruído de uma partida: a variação dentro de cada campeão, somada. Com
    // quase tudo em campeões de uma partida só não há como medir, e a variação
    // total entra no lugar — maior, então o perfil erra para o lado da cautela.
    let within: f64 = by_champion.values().map(|v| squares(v, mean_of(v))).sum();
    let df = all.len() - by_champion.len();
    let noise = if df >= 2 {
        (within / df as f64).max(EPS)
    } else {
        spread
    };

    let champion_spread = between_variance(&[cells_of(by_champion.values())], noise);
    let root = Parent::of(&overall);
    let champions: Vec<(i64, Reading)> = by_champion
        .iter()
        .map(|(&id, values)| (id, shrink(metric, values, &root, noise, champion_spread)))
        .collect();

    let groups: Vec<Vec<(f64, f64)>> = by_champion
        .keys()
        .map(|&id| cells_of(by_cell.range((id, i64::MIN)..=(id, i64::MAX)).map(|(_, v)| v)))
        .collect();
    let matchup_spread = between_variance(&groups, noise);
    let parents: BTreeMap<i64, Parent> = champions
        .iter()
        .map(|(id, reading)| (*id, Parent::of(reading)))
        .collect();
    let matchups = by_cell
        .iter()
        .map(|(&(champion, opponent), values)| {
            let parent = &parents[&champion];
            (
                (champion, opponent),
                shrink(metric, values, parent, noise, matchup_spread),
            )
        })
        .collect();

    Some(Levels {
        overall,
        champions,
        matchups,
    })
}

/// Encolhe a média de um grupo para o nível de cima.
///
/// O desvio do grupo é medido contra a média crua do pai e somado ao valor já
/// encolhido dele. `pull` é a fração que fica com o pai: 1 quando o
/// espalhamento é zero (nada se distingue do ruído), perto de 0 com muita
/// amostra.
fn shrink(metric: Metric, values: &[f64], parent: &Parent, noise: f64, spread: f64) -> Reading {
    let raw = mean_of(values);
    let sampling = noise / values.len() as f64;
    let pull = sampling / (sampling + spread);
    let value = parent.value + (1.0 - pull) * (raw - parent.raw);
    let var = (1.0 - pull) * sampling + pull * pull * parent.var;
    Reading {
        metric,
        games: values.len() as u32,
        raw,
        value,
        sd: var.sqrt(),
    }
}

/// Quanto as médias de verdade se espalham além do que o ruído explica: o τ²
/// de DerSimonian–Laird, generalizado para grupos com centro próprio.
///
/// Cada grupo é uma lista de (partidas, média) e o centro dele é a média
/// ponderada do próprio grupo. Grupo de uma célula só não diz nada sobre
/// espalhamento e fica de fora. Sem nada acima do ruído devolve zero.
fn between_variance(groups: &[Vec<(f64, f64)>], noise: f64) -> f64 {
    let (mut q, mut df, mut scale) = (0.0, 0.0, 0.0);
    for group in groups.iter().filter(|g| g.len() >= 2) {
        let weights: Vec<f64> = group.iter().map(|(n, _)| n / noise).collect();
        let s1: f64 = weights.iter().sum();
        let s2: f64 = weights.iter().map(|w| w * w).sum();
        let center = group.iter().zip(&weights).map(|((_, x), w)| w * x).sum::<f64>() / s1;
        q += group
            .iter()
            .zip(&weights)
            .map(|((_, x), w)| w * (x - center).powi(2))
            .sum::<f64>();
        df += (group.len() - 1) as f64;
        scale += s1 - s2 / s1;
    }
    if scale <= 0.0 {
        return 0.0;
    }
    ((q - df) / scale).max(0.0)
}

fn cells_of<'a>(groups: impl Iterator<Item = &'a Vec<f64>>) -> Vec<(f64, f64)> {
    groups.map(|v| (v.len() as f64, mean_of(v))).collect()
}

fn mean_of(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len() as f64
}

fn squares(values: &[f64], center: f64) -> f64 {
    values.iter().map(|v| (v - center).powi(2)).sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::champion::Role;

    /// Ruído fixo, média zero, para os testes não dependerem de sorteio.
    const RUIDO: [f64; 10] = [3.0, -2.0, 1.0, -3.0, 2.0, -1.0, 0.0, 2.5, -2.5, 0.0];

    fn jogo(campeao: i64, oponente: i64, tropa: f64) -> LaneGame {
        LaneGame {
            champion: campeao,
            opponent: oponente,
            role: Role::Mid,
            won: tropa > 0.0,
            diffs: BTreeMap::from([(Metric::Cs10, tropa), (Metric::Deaths, 0.0)]),
        }
    }

    /// `n` partidas de um campeão em torno de `centro`, contra oponentes que
    /// se revezam.
    fn serie(campeao: i64, centro: f64, n: usize) -> Vec<LaneGame> {
        (0..n)
            .map(|i| jogo(campeao, 900 + (i % 4) as i64, centro + RUIDO[i % RUIDO.len()]))
            .collect()
    }

    fn tropa(leituras: &[Reading]) -> Reading {
        *leituras.iter().find(|r| r.metric == Metric::Cs10).unwrap()
    }

    #[test]
    fn menos_de_cinco_partidas_nao_vira_perfil() {
        let perfil = build(&serie(1, 10.0, 4));
        assert!(perfil.overall.is_empty());
        assert!(perfil.champions.is_empty());
    }

    #[test]
    fn media_geral_e_medida_nao_encolhida() {
        let jogos = serie(1, 6.0, 10);
        let geral = tropa(&build(&jogos).overall);

        let valores: Vec<f64> = jogos.iter().map(|j| j.diffs[&Metric::Cs10]).collect();
        let media = mean_of(&valores);
        let desvio = (squares(&valores, media) / 9.0).sqrt();

        assert!((geral.raw - media).abs() < 1e-12);
        assert_eq!(geral.value, geral.raw);
        assert!((geral.sd - desvio / 10f64.sqrt()).abs() < 1e-12);
    }

    #[test]
    fn um_campeao_so_fica_na_propria_media() {
        let perfil = build(&serie(1, 6.0, 12));
        let geral = tropa(&perfil.overall);
        let campeao = tropa(&perfil.champions[0].readings);
        assert!((campeao.value - geral.value).abs() < 1e-12);
        assert!((campeao.sd - geral.sd).abs() < 1e-12);
    }

    #[test]
    fn campeoes_que_nao_se_distinguem_colapsam_na_media() {
        // Um ponto de diferença com ruído de ±3 em seis partidas: abaixo do
        // que o ruído explica. O perfil não pode apontar um especialista.
        let mut jogos = serie(1, 0.0, 6);
        jogos.extend(serie(2, 1.0, 6));
        let perfil = build(&jogos);
        let geral = tropa(&perfil.overall).value;

        for c in &perfil.champions {
            let leitura = tropa(&c.readings);
            assert!((leitura.value - geral).abs() < 1e-9, "{leitura:?}");
            assert!(!leitura.is_clear());
        }
    }

    #[test]
    fn diferenca_clara_sobrevive() {
        let mut jogos = serie(1, 10.0, 20);
        jogos.extend(serie(2, -10.0, 20));
        let perfil = build(&jogos);

        for c in &perfil.champions {
            let leitura = tropa(&c.readings);
            assert!((leitura.value - leitura.raw).abs() < 0.5, "{leitura:?}");
            assert!(leitura.is_clear());
        }
    }

    #[test]
    fn amostra_pequena_encolhe_mais() {
        let mut jogos = serie(1, 10.0, 20);
        jogos.extend(serie(2, -10.0, 20));
        jogos.push(jogo(3, 950, 25.0));
        jogos.push(jogo(3, 951, 25.0));
        let perfil = build(&jogos);
        let geral = tropa(&perfil.overall).value;

        let encolheu = |id: i64| {
            let l = tropa(&perfil.champion(id).unwrap().readings);
            (l.raw - l.value) / (l.raw - geral)
        };
        assert!(encolheu(3) > 2.0 * encolheu(1), "{} vs {}", encolheu(3), encolheu(1));
    }

    #[test]
    fn confronto_de_uma_partida_fica_perto_do_campeao() {
        let mut jogos = serie(1, 5.0, 20);
        jogos.push(jogo(1, 777, -30.0));
        let perfil = build(&jogos);
        let campeao = &perfil.champions[0];
        let confronto = campeao.matchups.iter().find(|m| m.opponent == 777).unwrap();

        let c = tropa(&campeao.readings).value;
        let m = tropa(&confronto.readings).value;
        assert!((m - c).abs() < (m - -30.0).abs(), "confronto {m}, campeão {c}");
    }

    #[test]
    fn metrica_que_falta_nao_entra_na_conta() {
        // Suporte não tem tropa: as seis partidas dele contam para mortes, não
        // para a média de tropa.
        let mut jogos = serie(1, 6.0, 6);
        for i in 0..6 {
            let mut j = jogo(2, 800, 0.0);
            j.role = Role::Support;
            j.diffs.remove(&Metric::Cs10);
            j.diffs.insert(Metric::Deaths, RUIDO[i]);
            jogos.push(j);
        }
        let perfil = build(&jogos);

        assert_eq!(tropa(&perfil.overall).games, 6);
        let suporte = perfil.champion(2).unwrap();
        assert!(suporte.readings.iter().all(|r| r.metric != Metric::Cs10));
        assert_eq!(suporte.games, 6);
    }

    #[test]
    fn espalhamento_conta_so_o_que_passa_do_ruido() {
        // Médias +5 e −5 com variância amostral 1 cada: a variância observada
        // entre elas é 50, o ruído explica 1, sobram 49.
        let cheio = between_variance(&[vec![(10.0, 5.0), (10.0, -5.0)]], 10.0);
        assert!((cheio - 49.0).abs() < 1e-9, "{cheio}");

        let ruido = between_variance(&[vec![(10.0, 0.5), (10.0, -0.5)]], 10.0);
        assert_eq!(ruido, 0.0);

        assert_eq!(between_variance(&[vec![(10.0, 5.0)]], 10.0), 0.0);
    }

    #[test]
    fn a_razao_sai_da_leitura_mais_segura() {
        let mut jogos = serie(1, 10.0, 20);
        jogos.extend(serie(2, -10.0, 20));
        let perfil = build(&jogos);
        let melhor = perfil.champion(1).unwrap().clearest().unwrap();
        assert_eq!(melhor.metric, Metric::Cs10);
        assert!(melhor.value > 0.0);
    }
}
