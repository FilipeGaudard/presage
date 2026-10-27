//! Ouro dos dez jogadores durante a partida.
//!
//! Duas colunas, e a diferença entre elas é o ponto.
//!
//! **Investido** é exato. Cada item carrega o próprio preço na resposta da
//! Live Client Data API, então somar dá o número certo, sem margem.
//!
//! **Ganho** é estimativa. A API só entrega o ouro em mão do jogador local; dos
//! outros nove não entrega nada. O que dá para fazer é reconstruir a renda a
//! partir do que é observável — tempo, tropas, abates e assistências — e dizer
//! na cara que é estimativa. Torres, objetivos e placas ficam de fora, então o
//! número é um piso.
//!
//! Há um conserto barato que vale a pena: ninguém gasta o que não ganhou. Se a
//! estimativa sai abaixo do investido, o investido manda.

use serde::Serialize;

use crate::live::LiveGame;

/// Ouro inicial no Rift.
const STARTING: f64 = 500.0;

/// A renda passiva começa em 1:50 e rende 20,4 a cada 10 segundos.
const PASSIVE_START: f64 = 110.0;
const PASSIVE_PER_SECOND: f64 = 2.04;

/// Média por tropa misturando corpo a corpo, à distância e canhão. No jogo o
/// valor sobe com o tempo; aqui é constante, e é por isso que a coluna é
/// estimativa e não medida.
const PER_CS: f64 = 21.0;

/// Abate sem sequência nem recompensa por cabeça.
const PER_KILL: f64 = 300.0;
const PER_ASSIST: f64 = 150.0;

/// A partir daqui vale a pena voltar para a base em vez de seguir com o ouro
/// no bolso.
const UNSPENT_ALERT: f64 = 1500.0;

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GoldPlayer {
    pub champion_key: String,
    pub champion_id: i64,
    pub display_name: String,
    pub team: String,
    pub position: String,
    pub is_local: bool,
    pub is_ally: bool,
    /// Soma exata dos preços dos itens no inventário.
    pub invested: f64,
    /// Renda reconstruída. Estimativa, e piso.
    pub earned: f64,
    /// O que provavelmente está no bolso: ganho menos investido.
    pub unspent: f64,
    pub gold_per_minute: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GoldReport {
    pub game_time: f64,
    pub players: Vec<GoldPlayer>,
    pub ours_invested: f64,
    pub theirs_invested: f64,
    /// Positivo quando o seu time tem mais ouro em itens. Este é exato.
    pub invested_lead: f64,
    pub ours_earned: f64,
    pub theirs_earned: f64,
    pub earned_lead: f64,
    pub notes: Vec<String>,
}

/// Renda total reconstruída até `game_time`.
pub fn earned_at(game_time: f64, cs: i64, kills: i64, assists: i64, invested: f64) -> f64 {
    let passive = (game_time - PASSIVE_START).max(0.0) * PASSIVE_PER_SECOND;
    let estimate = STARTING
        + passive
        + cs as f64 * PER_CS
        + kills as f64 * PER_KILL
        + assists as f64 * PER_ASSIST;

    // Ninguém gasta o que não ganhou. Onde a estimativa erra para baixo — e
    // ela erra, porque ignora torres e objetivos — o investido corrige.
    estimate.max(invested)
}

pub fn report(game: &LiveGame) -> GoldReport {
    let my_team = game.me().map(|me| me.team.clone());
    let minutes = (game.game_time / 60.0).max(1.0 / 60.0);

    let players: Vec<GoldPlayer> = game
        .players
        .iter()
        .map(|player| {
            let earned = earned_at(
                game.game_time,
                player.scores.creep_score,
                player.scores.kills,
                player.scores.assists,
                player.gold_invested,
            );

            GoldPlayer {
                champion_key: player.champion_key.clone(),
                champion_id: player.champion_id,
                display_name: player.display_name.clone(),
                team: player.team.clone(),
                position: player.position.clone(),
                is_local: player.is_local,
                is_ally: my_team.as_deref() == Some(player.team.as_str()),
                invested: player.gold_invested,
                earned,
                unspent: (earned - player.gold_invested).max(0.0),
                gold_per_minute: earned / minutes,
            }
        })
        .collect();

    let sum = |ally: bool, pick: fn(&GoldPlayer) -> f64| -> f64 {
        players
            .iter()
            .filter(|p| p.is_ally == ally)
            .map(pick)
            .sum()
    };

    let ours_invested = sum(true, |p| p.invested);
    let theirs_invested = sum(false, |p| p.invested);
    let ours_earned = sum(true, |p| p.earned);
    let theirs_earned = sum(false, |p| p.earned);

    // Sem jogador local não existe "nosso lado": tudo cai em `is_ally = false`
    // e os totais do nosso time ficam em zero, que é o honesto.
    let mut notes = Vec::new();

    if my_team.is_some() {
        let lead = ours_invested - theirs_invested;
        if lead.abs() >= 1000.0 {
            let lado = if lead > 0.0 { "à frente" } else { "atrás" };
            notes.push(format!("Seu time está {:.0} {lado} em itens", lead.abs()));
        }

        if let Some(me) = players.iter().find(|p| p.is_local) {
            if me.unspent >= UNSPENT_ALERT {
                notes.push(format!("Você tem ~{:.0} de ouro para gastar", me.unspent));
            }
        }

        if let Some(rico) = players
            .iter()
            .filter(|p| !p.is_ally)
            .max_by(|a, b| a.invested.total_cmp(&b.invested))
        {
            if rico.invested > 0.0 {
                notes.push(format!(
                    "O mais adiantado deles é {} ({:.0} em itens)",
                    rico.champion_key, rico.invested
                ));
            }
        }
    }

    GoldReport {
        game_time: game.game_time,
        players,
        ours_invested,
        theirs_invested,
        invested_lead: ours_invested - theirs_invested,
        ours_earned,
        theirs_earned,
        earned_lead: ours_earned - theirs_earned,
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::live::{LivePlayer, RawScores};

    fn player(
        key: &str,
        team: &str,
        local: bool,
        invested: f64,
        cs: i64,
        kills: i64,
        assists: i64,
    ) -> LivePlayer {
        LivePlayer {
            champion_key: key.into(),
            champion_id: 1,
            display_name: key.into(),
            team: team.into(),
            position: "MIDDLE".into(),
            level: 11,
            items: vec![],
            gold_invested: invested,
            is_dead: false,
            respawn_timer: 0.0,
            is_local: local,
            scores: RawScores {
                kills,
                deaths: 0,
                assists,
                creep_score: cs,
                ward_score: 0.0,
            },
        }
    }

    fn game(players: Vec<LivePlayer>, t: f64) -> LiveGame {
        LiveGame {
            game_time: t,
            players,
        }
    }

    #[test]
    fn antes_de_um_e_cinquenta_nao_ha_renda_passiva() {
        let cedo = earned_at(60.0, 0, 0, 0, 0.0);
        assert_eq!(cedo, STARTING);
    }

    #[test]
    fn a_renda_passiva_comeca_a_contar_depois() {
        let a = earned_at(PASSIVE_START, 0, 0, 0, 0.0);
        let b = earned_at(PASSIVE_START + 100.0, 0, 0, 0, 0.0);
        assert!((b - a - 100.0 * PASSIVE_PER_SECOND).abs() < 1e-9);
    }

    #[test]
    fn o_investido_e_um_piso_para_a_estimativa() {
        // Alguém que derrubou torres a partida toda tem mais ouro do que a
        // conta de tropas e abates sabe explicar.
        let estimado = earned_at(600.0, 10, 0, 0, 0.0);
        let corrigido = earned_at(600.0, 10, 0, 0, 9000.0);

        assert!(estimado < 9000.0);
        assert_eq!(corrigido, 9000.0);
    }

    #[test]
    fn o_investido_e_soma_exata_e_a_vantagem_tambem() {
        let g = game(
            vec![
                player("Ahri", "ORDER", true, 6200.0, 180, 4, 3),
                player("Zed", "CHAOS", false, 4100.0, 150, 2, 1),
            ],
            1200.0,
        );
        let r = report(&g);

        assert_eq!(r.ours_invested, 6200.0);
        assert_eq!(r.theirs_invested, 4100.0);
        assert_eq!(r.invested_lead, 2100.0);
    }

    #[test]
    fn o_ouro_parado_e_a_diferenca_entre_ganho_e_gasto() {
        let g = game(vec![player("Ahri", "ORDER", true, 1000.0, 200, 5, 5)], 1500.0);
        let r = report(&g);
        let eu = &r.players[0];

        assert!(eu.unspent > 0.0);
        assert!((eu.earned - eu.invested - eu.unspent).abs() < 1e-9);
    }

    #[test]
    fn avisa_quando_ha_ouro_sobrando_no_bolso() {
        let g = game(vec![player("Ahri", "ORDER", true, 0.0, 250, 8, 6)], 1800.0);
        let r = report(&g);

        assert!(
            r.notes.iter().any(|n| n.contains("para gastar")),
            "{:?}",
            r.notes
        );
    }

    #[test]
    fn nao_avisa_quando_o_ouro_esta_todo_investido() {
        let g = game(vec![player("Ahri", "ORDER", true, 99_000.0, 10, 0, 0)], 600.0);
        let r = report(&g);

        assert!(!r.notes.iter().any(|n| n.contains("para gastar")));
        assert_eq!(r.players[0].unspent, 0.0);
    }

    #[test]
    fn sem_jogador_local_nao_inventa_um_lado() {
        let g = game(
            vec![
                player("Ahri", "ORDER", false, 6200.0, 180, 4, 3),
                player("Zed", "CHAOS", false, 4100.0, 150, 2, 1),
            ],
            1200.0,
        );
        let r = report(&g);

        assert_eq!(r.ours_invested, 0.0);
        assert!(r.notes.is_empty());
        assert!(r.players.iter().all(|p| !p.is_ally));
    }

    #[test]
    fn ouro_por_minuto_nao_explode_no_segundo_zero() {
        let g = game(vec![player("Ahri", "ORDER", true, 0.0, 0, 0, 0)], 0.0);
        let r = report(&g);

        assert!(r.players[0].gold_per_minute.is_finite());
    }

    #[test]
    fn abates_e_assistencias_entram_na_renda() {
        let limpo = earned_at(900.0, 100, 0, 0, 0.0);
        let com_abate = earned_at(900.0, 100, 1, 0, 0.0);
        let com_assist = earned_at(900.0, 100, 0, 1, 0.0);

        assert!((com_abate - limpo - PER_KILL).abs() < 1e-9);
        assert!((com_assist - limpo - PER_ASSIST).abs() < 1e-9);
    }

    #[test]
    fn aponta_o_inimigo_mais_adiantado() {
        let g = game(
            vec![
                player("Ahri", "ORDER", true, 3000.0, 100, 0, 0),
                player("Zed", "CHAOS", false, 4000.0, 120, 3, 0),
                player("Viego", "CHAOS", false, 8000.0, 160, 7, 2),
            ],
            1500.0,
        );
        let r = report(&g);

        assert!(
            r.notes.iter().any(|n| n.contains("Viego")),
            "{:?}",
            r.notes
        );
    }
}
