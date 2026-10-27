//! Ranking de prioridade em tempo real.
//!
//! A API da Riot não entrega posição de campeão no mapa, e nenhuma via legal
//! entrega. Então isto **não diz onde o inimigo está** — diz onde está o valor.
//!
//! O que alimenta o ranking é tudo dado oficial: quem está morto e por quanto
//! tempo, CS e nível de cada um, os itens dos dez, e os eventos de objetivo. Em
//! cima disso roda a simulação de escaramuça do motor de dano, com as
//! resistências e a penetração reais.
//!
//! A formulação importa tanto quanto a conta: a saída é sempre "esta rota vale
//! mais agora, por estes motivos", nunca "o inimigo está no topo".

use serde::Serialize;

use super::build::{self, Combatant};
use super::champion::ChampionIndex;
use super::items::ItemIndex;
use super::timeline::Timeline;
use crate::live::{LiveGame, LivePlayer};

/// Quanto cada sinal pode somar ao placar. São pesos de exibição: mantêm a
/// escaramuça como base e deixam o resto temperar.
const DEAD_ENEMY_MAX: f64 = 25.0;
const DEAD_JUNGLER: f64 = 12.0;
const BEHIND_MAX: f64 = 10.0;
const OBJECTIVE_MAX: f64 = 15.0;

/// Janela em que um objetivo começa a puxar prioridade.
const OBJECTIVE_WINDOW: f64 = 90.0;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LanePriority {
    pub lane: String,
    pub score: f64,
    /// Chance de ganhar a luta nessa rota, 0 a 1.
    pub skirmish_odds: f64,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PriorityReport {
    pub lanes: Vec<LanePriority>,
    pub focus: Option<String>,
}

struct Lane {
    name: &'static str,
    positions: &'static [&'static str],
    /// Objetivos que ficam desse lado do mapa.
    objectives: &'static [&'static str],
}

const LANES: &[Lane] = &[
    Lane {
        name: "TOP",
        positions: &["TOP"],
        objectives: &["Barão", "Arauto", "Grubs"],
    },
    Lane {
        name: "MID",
        positions: &["MIDDLE"],
        objectives: &[],
    },
    Lane {
        name: "BOT",
        positions: &["BOTTOM", "UTILITY"],
        objectives: &["Dragão", "Ancião"],
    },
];

pub fn report(
    game: &LiveGame,
    timeline: &Timeline,
    champions: &ChampionIndex,
    items: &ItemIndex,
) -> PriorityReport {
    let Some(me) = game.me() else {
        return PriorityReport {
            lanes: Vec::new(),
            focus: None,
        };
    };

    let enemy_jungler_down = game
        .enemies()
        .into_iter()
        .find(|player| player.position == "JUNGLE")
        .filter(|player| player.is_dead)
        .map(|player| player.respawn_timer);

    let mut lanes: Vec<LanePriority> = LANES
        .iter()
        .map(|lane| score_lane(lane, game, me, timeline, enemy_jungler_down, champions, items))
        .collect();

    lanes.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.lane.cmp(&b.lane))
    });

    let focus = timeline
        .upcoming
        .iter()
        .find(|spawn| spawn.seconds_left <= OBJECTIVE_WINDOW)
        .map(|spawn| {
            let certainty = if spawn.observed { "" } else { " (estimado)" };
            format!(
                "{} em {:.0}s{}",
                spawn.name, spawn.seconds_left, certainty
            )
        });

    PriorityReport { lanes, focus }
}

fn score_lane(
    lane: &Lane,
    game: &LiveGame,
    me: &LivePlayer,
    timeline: &Timeline,
    enemy_jungler_down: Option<f64>,
    champions: &ChampionIndex,
    items: &ItemIndex,
) -> LanePriority {
    let allies: Vec<&LivePlayer> = game
        .players
        .iter()
        .filter(|player| player.team == me.team && in_lane(lane, player))
        .collect();

    // Vivos de cada lado: quem está morto não entra na luta.
    let enemies: Vec<&LivePlayer> = game
        .players
        .iter()
        .filter(|player| player.team != me.team && in_lane(lane, player))
        .collect();

    let dead_enemies: Vec<&&LivePlayer> = enemies.iter().filter(|player| player.is_dead).collect();

    let mut our_side: Vec<Combatant> = allies
        .iter()
        .filter(|player| !player.is_dead)
        .map(|player| player.as_combatant())
        .collect();

    // O jogador local entra na conta: é ele que vai decidir ir ou não.
    if !in_lane(lane, me) && !me.is_dead {
        our_side.push(me.as_combatant());
    }

    let their_side: Vec<Combatant> = enemies
        .iter()
        .filter(|player| !player.is_dead)
        .map(|player| player.as_combatant())
        .collect();

    let odds = build::skirmish_odds(&our_side, &their_side, champions, items);
    let mut score = odds * 100.0;
    let mut reasons = Vec::new();

    reasons.push(format!(
        "{}v{} a seu favor em {:.0}%",
        our_side.len(),
        their_side.len(),
        odds * 100.0
    ));

    if let Some(dead) = dead_enemies.first() {
        // Quanto mais tempo morto, maior a janela.
        let bonus = (dead.respawn_timer / 2.0).clamp(0.0, DEAD_ENEMY_MAX);
        score += bonus;
        reasons.push(format!(
            "{} morto por {:.0}s",
            dead.champion_key, dead.respawn_timer
        ));
    }

    if let Some(respawn) = enemy_jungler_down {
        score += DEAD_JUNGLER;
        reasons.push(format!("Jungler inimigo morto por {respawn:.0}s"));
    }

    if let Some(gap) = lane_deficit(&allies, &enemies) {
        if gap > 0.0 {
            score += (gap / 20.0 * BEHIND_MAX).clamp(0.0, BEHIND_MAX);
            reasons.push(format!("Eles estão {gap:.0} de CS atrás na rota"));
        }
    }

    if let Some(spawn) = timeline
        .upcoming
        .iter()
        .filter(|spawn| lane.objectives.contains(&spawn.name.as_str()))
        .find(|spawn| spawn.seconds_left <= OBJECTIVE_WINDOW)
    {
        let urgency = 1.0 - (spawn.seconds_left / OBJECTIVE_WINDOW);
        score += urgency * OBJECTIVE_MAX;
        reasons.push(format!("{} em {:.0}s", spawn.name, spawn.seconds_left));
    }

    reasons.truncate(3);

    LanePriority {
        lane: lane.name.to_string(),
        score: (score.clamp(0.0, 100.0) * 10.0).round() / 10.0,
        skirmish_odds: odds,
        reasons,
    }
}

fn in_lane(lane: &Lane, player: &LivePlayer) -> bool {
    lane.positions.contains(&player.position.as_str())
}

/// Quanto os inimigos daquela rota estão atrás em CS. `None` quando falta
/// alguém dos dois lados para comparar.
fn lane_deficit(allies: &[&LivePlayer], enemies: &[&LivePlayer]) -> Option<f64> {
    if allies.is_empty() || enemies.is_empty() {
        return None;
    }
    let ours: i64 = allies.iter().map(|player| player.scores.creep_score).sum();
    let theirs: i64 = enemies.iter().map(|player| player.scores.creep_score).sum();
    Some((ours - theirs) as f64)
}

#[cfg(test)]
mod tests {
    use super::super::champion::fake;
    use super::super::items::{parse_stats, Item};
    use super::super::timeline::{Sides, Spawn, Timeline};
    use super::*;
    use crate::live::RawScores;
    use std::collections::HashMap;

    fn champions() -> ChampionIndex {
        let all = vec![
            fake(1, "Lutador", "Fighter", 8.0, 3.0, 6.0),
            fake(2, "Atirador", "Marksman", 9.0, 2.0, 3.0),
        ];
        ChampionIndex {
            version: "test".into(),
            by_id: all.into_iter().map(|c| (c.id, c)).collect::<HashMap<_, _>>(),
        }
    }

    fn items() -> ItemIndex {
        let lamina = Item {
            id: 5,
            name: "Lâmina".into(),
            cost: 3000.0,
            purchasable: true,
            stats: parse_stats("<stats><attention>70</attention> Attack Damage</stats>"),
        };
        ItemIndex {
            version: "test".into(),
            by_id: HashMap::from([(5, lamina)]),
        }
    }

    fn player(
        name: &str,
        team: &str,
        position: &str,
        is_local: bool,
        is_dead: bool,
        respawn: f64,
        cs: i64,
    ) -> LivePlayer {
        LivePlayer {
            champion_key: name.into(),
            champion_id: 1,
            display_name: format!("{name}#BR1"),
            team: team.into(),
            position: position.into(),
            level: 11,
            items: vec![],
            gold_invested: 0.0,
            is_dead,
            respawn_timer: respawn,
            is_local,
            scores: RawScores {
                creep_score: cs,
                ..Default::default()
            },
        }
    }

    fn timeline(upcoming: Vec<Spawn>) -> Timeline {
        Timeline {
            game_time: 600.0,
            dragons: Sides::default(),
            dragon_types: vec![],
            soul: None,
            turrets: Sides::default(),
            kills: Sides::default(),
            upcoming,
        }
    }

    fn spawn(name: &str, seconds_left: f64) -> Spawn {
        Spawn {
            name: name.into(),
            at: 600.0 + seconds_left,
            seconds_left,
            observed: true,
        }
    }

    fn base_game() -> LiveGame {
        LiveGame {
            game_time: 600.0,
            players: vec![
                player("Eu", "ORDER", "JUNGLE", true, false, 0.0, 80),
                player("AliadoTop", "ORDER", "TOP", false, false, 0.0, 100),
                player("AliadoMid", "ORDER", "MIDDLE", false, false, 0.0, 100),
                player("AliadoBot", "ORDER", "BOTTOM", false, false, 0.0, 100),
                player("IniJg", "CHAOS", "JUNGLE", false, false, 0.0, 80),
                player("IniTop", "CHAOS", "TOP", false, false, 0.0, 100),
                player("IniMid", "CHAOS", "MIDDLE", false, false, 0.0, 100),
                player("IniBot", "CHAOS", "BOTTOM", false, false, 0.0, 100),
            ],
        }
    }

    fn lane<'a>(report: &'a PriorityReport, name: &str) -> &'a LanePriority {
        report.lanes.iter().find(|l| l.lane == name).expect("rota")
    }

    #[test]
    fn sem_jogador_local_devolve_vazio() {
        let mut game = base_game();
        for player in &mut game.players {
            player.is_local = false;
        }
        let report = report(&game, &timeline(vec![]), &champions(), &items());
        assert!(report.lanes.is_empty());
    }

    #[test]
    fn as_tres_rotas_aparecem_ordenadas() {
        let report = report(&base_game(), &timeline(vec![]), &champions(), &items());
        assert_eq!(report.lanes.len(), 3);
        for pair in report.lanes.windows(2) {
            assert!(pair[0].score >= pair[1].score);
        }
    }

    #[test]
    fn inimigo_morto_na_rota_sobe_a_prioridade() {
        let mut game = base_game();
        let neutro = report(&game, &timeline(vec![]), &champions(), &items());
        let antes = lane(&neutro, "TOP").score;

        game.players[5] = player("IniTop", "CHAOS", "TOP", false, true, 30.0, 100);
        let depois = report(&game, &timeline(vec![]), &champions(), &items());

        assert!(lane(&depois, "TOP").score > antes);
        assert!(lane(&depois, "TOP")
            .reasons
            .iter()
            .any(|r| r.contains("morto")));
    }

    #[test]
    fn jungler_inimigo_morto_levanta_todas_as_rotas() {
        let mut game = base_game();
        let antes = report(&game, &timeline(vec![]), &champions(), &items());
        game.players[4] = player("IniJg", "CHAOS", "JUNGLE", false, true, 25.0, 80);
        let depois = report(&game, &timeline(vec![]), &champions(), &items());

        for nome in ["TOP", "MID", "BOT"] {
            assert!(
                lane(&depois, nome).score > lane(&antes, nome).score,
                "{nome}"
            );
        }
    }

    #[test]
    fn dragao_proximo_puxa_a_rota_de_baixo() {
        let com_dragao = report(
            &base_game(),
            &timeline(vec![spawn("Dragão", 20.0)]),
            &champions(),
            &items(),
        );
        let sem = report(&base_game(), &timeline(vec![]), &champions(), &items());

        assert!(lane(&com_dragao, "BOT").score > lane(&sem, "BOT").score);
        assert!(lane(&com_dragao, "BOT")
            .reasons
            .iter()
            .any(|r| r.contains("Dragão")));
    }

    #[test]
    fn barao_proximo_puxa_a_rota_de_cima_e_nao_a_de_baixo() {
        let report = report(
            &base_game(),
            &timeline(vec![spawn("Barão", 10.0)]),
            &champions(),
            &items(),
        );
        assert!(lane(&report, "TOP").reasons.iter().any(|r| r.contains("Barão")));
        assert!(lane(&report, "BOT").reasons.iter().all(|r| !r.contains("Barão")));
    }

    #[test]
    fn morto_nao_entra_na_escaramuca() {
        let mut game = base_game();
        game.players[5] = player("IniTop", "CHAOS", "TOP", false, true, 30.0, 100);
        let report = report(&game, &timeline(vec![]), &champions(), &items());

        // Aliado + jungler contra ninguém: 2v0.
        assert!(lane(&report, "TOP").reasons[0].starts_with("2v0"));
        assert_eq!(lane(&report, "TOP").skirmish_odds, 1.0);
    }

    #[test]
    fn nenhuma_razao_revela_posicao() {
        let mut game = base_game();
        game.players[4] = player("IniJg", "CHAOS", "JUNGLE", false, true, 25.0, 80);
        let report = report(
            &game,
            &timeline(vec![spawn("Dragão", 15.0)]),
            &champions(),
            &items(),
        );

        for lane in &report.lanes {
            for reason in &lane.reasons {
                let lower = reason.to_lowercase();
                assert!(!lower.contains("está no"), "{reason}");
                assert!(!lower.contains("posição"), "{reason}");
            }
        }
    }

    #[test]
    fn foco_marca_quando_o_numero_e_estimado() {
        let mut estimado = spawn("Barão", 40.0);
        estimado.observed = false;
        let report = report(&base_game(), &timeline(vec![estimado]), &champions(), &items());
        assert!(report.focus.unwrap().contains("estimado"));
    }

    #[test]
    fn objetivo_distante_nao_vira_foco() {
        let report = report(
            &base_game(),
            &timeline(vec![spawn("Barão", 400.0)]),
            &champions(),
            &items(),
        );
        assert!(report.focus.is_none());
    }

    #[test]
    fn placar_fica_na_faixa() {
        let mut game = base_game();
        game.players[4] = player("IniJg", "CHAOS", "JUNGLE", false, true, 90.0, 80);
        game.players[5] = player("IniTop", "CHAOS", "TOP", false, true, 90.0, 0);
        let report = report(
            &game,
            &timeline(vec![spawn("Barão", 0.0)]),
            &champions(),
            &items(),
        );
        for lane in &report.lanes {
            assert!((0.0..=100.0).contains(&lane.score), "{:?}", lane);
        }
    }
}
