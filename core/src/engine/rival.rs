//! Você contra o oponente direto, durante a partida.
//!
//! A mesma regra do pós-jogo (`coach::cobranca`): a rota decide o que se cobra.
//! Quem farma é comparado em tropas por minuto; o suporte, em placar de visão
//! por minuto, e nunca em tropas. O ouro em itens vale para todas as rotas: é
//! exato — o preço dos itens que estão na tela de todo mundo — e diz quem saiu
//! na frente.
//!
//! O oponente vem da posição que a Live Client informa para os dez, então aqui
//! não há rota presumida.

use serde::Serialize;

use crate::live::{LiveGame, LivePlayer};

use super::champion::Role;
use super::coach::cobranca;

/// Antes disso a taxa por minuto é ruído: uma tropa a mais no primeiro minuto
/// vira uma tropa por minuto de diferença.
pub const MIN_SECONDS: f64 = 90.0;

/// Diferença mínima de ouro em itens para alguém estar na frente. **Regra, não
/// estatística**: o maior entre este piso e `AHEAD_SHARE` do que o oponente
/// investiu. Um item pequeno de vantagem no começo conta; no fim do jogo, não.
pub const AHEAD_FLOOR: f64 = 300.0;
pub const AHEAD_SHARE: f64 = 0.10;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Metric {
    /// Tropas por minuto.
    Farm,
    /// Placar de visão por minuto.
    Vision,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Standing {
    Behind,
    Even,
    Ahead,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Rival {
    pub role: Role,
    pub opponent_id: i64,
    pub opponent_key: String,
    /// O que a rota cobra. `None` quando a rota não cobra nem farm nem visão.
    pub metric: Option<Metric>,
    pub mine_per_min: f64,
    pub theirs_per_min: f64,
    /// O seu ouro em itens menos o dele.
    pub gold_diff: f64,
    pub standing: Standing,
}

fn per_min(player: &LivePlayer, metric: Metric, minutes: f64) -> f64 {
    let total = match metric {
        Metric::Farm => player.scores.creep_score as f64,
        Metric::Vision => player.scores.ward_score,
    };
    ((total / minutes) * 10.0).round() / 10.0
}

pub fn standing(gold_diff: f64, theirs: f64) -> Standing {
    let limite = AHEAD_FLOOR.max(AHEAD_SHARE * theirs);
    if gold_diff >= limite {
        Standing::Ahead
    } else if gold_diff <= -limite {
        Standing::Behind
    } else {
        Standing::Even
    }
}

pub fn report(game: &LiveGame) -> Option<Rival> {
    if game.game_time < MIN_SECONDS {
        return None;
    }
    let me = game.me()?;
    let role = Role::from_lcu(&me.position);
    if role == Role::Unknown {
        return None;
    }
    let opponent = game
        .enemies()
        .into_iter()
        .find(|p| Role::from_lcu(&p.position) == role)?;

    let regra = cobranca(role);
    let metric = if regra.farm {
        Some(Metric::Farm)
    } else if regra.vision {
        Some(Metric::Vision)
    } else {
        None
    };
    let minutes = game.game_time / 60.0;
    let (mine_per_min, theirs_per_min) = match metric {
        Some(m) => (per_min(me, m, minutes), per_min(opponent, m, minutes)),
        None => (0.0, 0.0),
    };
    let gold_diff = me.gold_invested - opponent.gold_invested;

    Some(Rival {
        role,
        opponent_id: opponent.champion_id,
        opponent_key: opponent.champion_key.clone(),
        metric,
        mine_per_min,
        theirs_per_min,
        gold_diff,
        standing: standing(gold_diff, opponent.gold_invested),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::live::RawScores;

    fn jogador(position: &str, team: &str, local: bool, cs: i64, visao: f64, ouro: f64) -> LivePlayer {
        LivePlayer {
            champion_key: format!("{position}-{team}"),
            champion_id: if local { 1 } else { 2 },
            display_name: format!("{position}#{team}"),
            team: team.into(),
            position: position.into(),
            level: 9,
            items: vec![],
            gold_invested: ouro,
            is_dead: false,
            respawn_timer: 0.0,
            is_local: local,
            scores: RawScores {
                creep_score: cs,
                ward_score: visao,
                ..Default::default()
            },
        }
    }

    fn partida(minutos: f64, players: Vec<LivePlayer>) -> LiveGame {
        LiveGame {
            game_time: minutos * 60.0,
            players,
        }
    }

    #[test]
    fn suporte_ao_vivo_e_comparado_pela_visao_e_nunca_por_tropas() {
        // Você tem 10 tropas contra 40 do outro suporte, e mais visão: o painel
        // fala de visão.
        let g = partida(
            10.0,
            vec![
                jogador("UTILITY", "ORDER", true, 10, 15.0, 2000.0),
                jogador("UTILITY", "CHAOS", false, 40, 8.0, 2000.0),
            ],
        );
        let r = report(&g).unwrap();
        assert_eq!(r.role, Role::Support);
        assert_eq!(r.metric, Some(Metric::Vision));
        assert_eq!(r.mine_per_min, 1.5);
        assert_eq!(r.theirs_per_min, 0.8);
    }

    #[test]
    fn quem_farma_e_comparado_em_tropas_contra_o_da_mesma_rota() {
        let g = partida(
            10.0,
            vec![
                jogador("MIDDLE", "ORDER", true, 70, 5.0, 3000.0),
                jogador("TOP", "CHAOS", false, 95, 5.0, 3000.0),
                jogador("MIDDLE", "CHAOS", false, 85, 5.0, 3000.0),
            ],
        );
        let r = report(&g).unwrap();
        assert_eq!(r.metric, Some(Metric::Farm));
        assert_eq!(r.opponent_key, "MIDDLE-CHAOS");
        assert_eq!(r.mine_per_min, 7.0);
        assert_eq!(r.theirs_per_min, 8.5);
    }

    #[test]
    fn sem_oponente_na_rota_ou_sem_rota_nao_ha_painel() {
        let sem_oponente = partida(
            10.0,
            vec![
                jogador("MIDDLE", "ORDER", true, 70, 5.0, 3000.0),
                jogador("", "CHAOS", false, 85, 5.0, 3000.0),
            ],
        );
        assert!(report(&sem_oponente).is_none());

        let sem_rota = partida(
            10.0,
            vec![
                jogador("", "ORDER", true, 70, 5.0, 3000.0),
                jogador("MIDDLE", "CHAOS", false, 85, 5.0, 3000.0),
            ],
        );
        assert!(report(&sem_rota).is_none());
    }

    #[test]
    fn antes_de_um_minuto_e_meio_a_taxa_e_ruido() {
        let g = partida(
            1.0,
            vec![
                jogador("MIDDLE", "ORDER", true, 3, 0.0, 500.0),
                jogador("MIDDLE", "CHAOS", false, 1, 0.0, 500.0),
            ],
        );
        assert!(report(&g).is_none());
    }

    #[test]
    fn na_frente_pelo_ouro_em_itens_com_piso_e_proporcao() {
        // Cedo vale o piso de 300.
        assert_eq!(standing(300.0, 1000.0), Standing::Ahead);
        assert_eq!(standing(299.0, 1000.0), Standing::Even);
        assert_eq!(standing(-300.0, 1000.0), Standing::Behind);
        // Tarde vale 10% do que ele investiu: 500 de 10.000 é igual.
        assert_eq!(standing(500.0, 10_000.0), Standing::Even);
        assert_eq!(standing(1000.0, 10_000.0), Standing::Ahead);
    }
}
