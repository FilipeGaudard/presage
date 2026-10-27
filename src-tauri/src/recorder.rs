//! Grava a partida enquanto ela acontece, para o coach ler quando ela acabar.
//!
//! Este é o caminho que dispensa a chave da Riot. A API de partidas entrega a
//! timeline depois — mas só com chave de produção, e com aprovação que leva
//! semanas. A Live Client Data API entrega o mesmo estado a cada segundo, de
//! graça, na própria máquina. Guardar isso enquanto o jogo roda dá o material
//! do pós-jogo sem depender de ninguém.
//!
//! Tudo fica em `%LOCALAPPDATA%\Presage`. Nada sai da máquina.

use presage_core::engine::coach::{review, Recording, Report};
use presage_core::engine::items::ItemIndex;
use presage_core::engine::timeline::RawEvent;
use presage_core::live::LiveGame;

use crate::storage;

pub const LAST_REVIEW: &str = "ultima-partida.json";
pub const LAST_RECORDING: &str = "gravacao.json";

/// Teto de quadros guardados. A um por segundo, cobre mais de uma hora de
/// jogo; o limite existe para uma sessão esquecida aberta não encher o disco.
const MAX_FRAMES: usize = 6000;

/// Abaixo disto não vale guardar: remake, partida custom de teste, ou a
/// gravação começou quando o jogo já tinha acabado.
const MIN_FRAMES: usize = 60;

pub fn observe(recording: &mut Recording, game: &LiveGame, events: &[RawEvent]) {
    if recording.frames.len() < MAX_FRAMES {
        recording.push(game.clone());
    }

    if recording.won.is_none() {
        recording.won = events.iter().find_map(RawEvent::outcome);
    }
}

/// Fecha a gravação e devolve a leitura.
///
/// Limpa a gravação em todos os casos — inclusive quando ela é curta demais
/// para render relatório. Deixar restos de uma partida para trás contaminaria
/// a próxima.
pub fn finish(recording: &mut Recording, items: &ItemIndex) -> Option<Report> {
    let gravado = std::mem::take(recording);

    if gravado.frames.len() < MIN_FRAMES {
        return None;
    }

    let relatorio = review(&gravado, items);

    if let Err(err) = storage::write_json(LAST_RECORDING, &gravado) {
        tracing::warn!(%err, "não consegui guardar a gravação da partida");
    }
    if let Err(err) = storage::write_json(LAST_REVIEW, &relatorio) {
        tracing::warn!(%err, "não consegui guardar a leitura da partida");
    }

    Some(relatorio)
}

#[cfg(test)]
mod tests {
    use super::*;
    use presage_core::live::{LivePlayer, RawScores};

    fn game(t: f64) -> LiveGame {
        LiveGame {
            game_time: t,
            players: vec![LivePlayer {
                champion_key: "Ahri".into(),
                champion_id: 1,
                display_name: "eu".into(),
                team: "ORDER".into(),
                position: "MIDDLE".into(),
                level: 11,
                items: vec![],
                gold_invested: 0.0,
                is_dead: false,
                respawn_timer: 0.0,
                is_local: true,
                scores: RawScores::default(),
            }],
        }
    }

    fn fim(resultado: &str) -> RawEvent {
        RawEvent {
            name: "GameEnd".into(),
            result: resultado.into(),
            ..Default::default()
        }
    }

    #[test]
    fn o_resultado_sai_do_evento_de_fim_de_jogo() {
        let mut r = Recording::default();
        observe(&mut r, &game(1.0), &[]);
        assert_eq!(r.won, None);

        observe(&mut r, &game(2.0), &[fim("Win")]);
        assert_eq!(r.won, Some(true));
    }

    #[test]
    fn derrota_tambem_e_lida() {
        let mut r = Recording::default();
        observe(&mut r, &game(1.0), &[fim("Lose")]);
        assert_eq!(r.won, Some(false));
    }

    #[test]
    fn evento_que_nao_e_fim_de_jogo_nao_decide_resultado() {
        let mut r = Recording::default();
        let dragao = RawEvent {
            name: "DragonKill".into(),
            ..Default::default()
        };
        observe(&mut r, &game(1.0), &[dragao]);
        assert_eq!(r.won, None);
    }

    #[test]
    fn o_resultado_nao_e_reescrito_depois_de_conhecido() {
        let mut r = Recording::default();
        observe(&mut r, &game(1.0), &[fim("Win")]);
        observe(&mut r, &game(2.0), &[fim("Lose")]);
        assert_eq!(r.won, Some(true));
    }

    #[test]
    fn a_gravacao_tem_teto() {
        let mut r = Recording::default();
        for i in 0..(MAX_FRAMES + 500) {
            observe(&mut r, &game(i as f64), &[]);
        }
        assert_eq!(r.frames.len(), MAX_FRAMES);
    }

    #[test]
    fn partida_curta_nao_rende_relatorio_mas_limpa_a_gravacao() {
        let mut r = Recording::default();
        for i in 0..10 {
            observe(&mut r, &game(i as f64), &[]);
        }

        assert!(finish(&mut r, &ItemIndex::default()).is_none());
        assert!(
            r.frames.is_empty(),
            "restos de uma partida contaminariam a próxima"
        );
    }
}
