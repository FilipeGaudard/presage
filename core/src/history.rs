//! Partida crua da match-v5 → a partida do jogador, comparada ao oponente
//! direto.
//!
//! Mesma divisão de `draft.rs`: `Match*` espelha o JSON da Riot, `PlayedGame`
//! e `LaneGame` são o que o resto do Presage consome, e `played` é a única
//! fronteira entre os dois.
//!
//! A comparação é sempre contra o oponente da mesma rota, na mesma partida. O
//! matchmaking escolheu ele para ter o seu nível, e ele jogou a mesma rota pelo
//! mesmo tempo: é o grupo de controle que uma conta sozinha consegue ter sem
//! base de partidas. Somada sobre a população a diferença é exatamente zero —
//! cada confronto entra uma vez com +d e outra com −d —, então a referência
//! existe sem dado global nenhum.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::engine::champion::Role;

/// O mesmo corte do coletor: abaixo disto é remake ou abandono precoce.
const MIN_DURATION: i64 = 300;

// ---------------------------------------------------------------------------
// Formato cru da match-v5
// ---------------------------------------------------------------------------
//
// Tudo com `default`: campo que falta derruba a métrica, não a partida.

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MatchDto {
    pub info: MatchInfo,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MatchInfo {
    pub game_duration: i64,
    pub game_mode: String,
    /// `GameComplete` numa partida normal. Vazio em respostas antigas.
    pub end_of_game_result: String,
    pub participants: Vec<MatchParticipant>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MatchParticipant {
    pub puuid: String,
    pub champion_id: i64,
    pub team_id: i64,
    /// `TOP`, `JUNGLE`, `MIDDLE`, `BOTTOM`, `UTILITY`, ou vazio.
    pub team_position: String,
    pub win: bool,
    pub game_ended_in_early_surrender: bool,
    pub deaths: u32,
    /// Ausente em partidas antigas e em alguns modos.
    pub challenges: Option<Challenges>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Challenges {
    #[serde(rename = "laneMinionsFirst10Minutes")]
    pub lane_minions_first_10: Option<f64>,
    #[serde(rename = "jungleCsBefore10Minutes")]
    pub jungle_cs_before_10: Option<f64>,
    /// Fração de 0 a 1.
    #[serde(rename = "killParticipation")]
    pub kill_participation: Option<f64>,
    /// Fração de 0 a 1.
    #[serde(rename = "teamDamagePercentage")]
    pub team_damage_share: Option<f64>,
    #[serde(rename = "visionScorePerMinute")]
    pub vision_per_minute: Option<f64>,
}

// ---------------------------------------------------------------------------
// Modelo de domínio
// ---------------------------------------------------------------------------

/// O que se compara com o oponente direto.
///
/// O que ficou de fora ficou por um motivo: ouro e dano por minuto de fim de
/// jogo medem mais o resultado do que a rota — quem ganhou farma o mapa
/// inteiro —, então só repetiriam a vitória. Participação e fatia de dano são
/// frações do próprio time, o que tira boa parte do resultado da conta.
/// Mortes é a exceção consciente: mede a partida inteira e carrega parte do
/// resultado, mas é o erro mais acionável que existe.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Metric {
    /// Tropas aos 10 minutos; campos da selva, para o caçador.
    Cs10,
    /// Participação em abates, em pontos percentuais.
    KillParticipation,
    /// Fatia do dano do time em campeões, em pontos percentuais.
    DamageShare,
    /// Placar de visão por minuto.
    VisionPerMinute,
    /// Mortes na partida.
    Deaths,
}

impl Metric {
    pub const ALL: [Metric; 5] = [
        Metric::Cs10,
        Metric::KillParticipation,
        Metric::DamageShare,
        Metric::VisionPerMinute,
        Metric::Deaths,
    ];

    pub fn higher_is_better(self) -> bool {
        !matches!(self, Metric::Deaths)
    }

    /// A diferença em texto, do jeito que entra numa razão do draft.
    pub fn phrase(self, diff: f64) -> String {
        match self {
            Metric::Cs10 => format!("{diff:+.0} tropas aos 10"),
            Metric::KillParticipation => format!("{diff:+.0}pp de participação em abates"),
            Metric::DamageShare => format!("{diff:+.0}pp da fatia de dano"),
            Metric::VisionPerMinute => format!("{diff:+.1} de visão por minuto"),
            Metric::Deaths => format!("{diff:+.1} mortes por partida"),
        }
    }

    fn value(self, participant: &MatchParticipant, role: Role) -> Option<f64> {
        let challenges = participant.challenges.as_ref();
        match self {
            // Suporte não farma por desenho: a diferença de tropa entre dois
            // suportes não mede nada.
            Metric::Cs10 => match role {
                Role::Support | Role::Unknown => None,
                Role::Jungle => challenges?.jungle_cs_before_10,
                _ => challenges?.lane_minions_first_10,
            },
            Metric::KillParticipation => challenges?.kill_participation.map(|v| v * 100.0),
            Metric::DamageShare => challenges?.team_damage_share.map(|v| v * 100.0),
            Metric::VisionPerMinute => challenges?.vision_per_minute,
            Metric::Deaths => Some(participant.deaths as f64),
        }
    }
}

/// Uma partida do jogador, já validada.
#[derive(Debug, Clone, PartialEq)]
pub struct PlayedGame {
    pub champion: i64,
    pub won: bool,
    /// `None` quando a partida conta para vitória e derrota mas não dá para
    /// comparar rota: posição em branco ou oponente ambíguo.
    pub lane: Option<LaneGame>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaneGame {
    pub champion: i64,
    pub opponent: i64,
    pub role: Role,
    pub won: bool,
    /// Você menos o oponente, na unidade da métrica. Só as métricas que os
    /// dois lados têm.
    pub diffs: BTreeMap<Metric, f64>,
}

/// Por que uma partida não conta.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Skip {
    Ausente,
    Modo,
    Terminada,
    Remake,
    Curta,
}

/// Lê a partida do ponto de vista de `puuid`.
///
/// Os cortes são os do coletor, e valem também para vitória e derrota: uma
/// conta que soma remake mede a taxa de remake.
pub fn played(game: &MatchDto, puuid: &str) -> Result<PlayedGame, Skip> {
    let info = &game.info;
    let me = info
        .participants
        .iter()
        .find(|p| p.puuid == puuid)
        .ok_or(Skip::Ausente)?;

    if !info.game_mode.eq_ignore_ascii_case("CLASSIC") {
        return Err(Skip::Modo);
    }
    if !info.end_of_game_result.is_empty()
        && !info.end_of_game_result.eq_ignore_ascii_case("GameComplete")
    {
        return Err(Skip::Terminada);
    }
    if info
        .participants
        .iter()
        .any(|p| p.game_ended_in_early_surrender)
    {
        return Err(Skip::Remake);
    }
    if info.game_duration < MIN_DURATION {
        return Err(Skip::Curta);
    }

    Ok(PlayedGame {
        champion: me.champion_id,
        won: me.win,
        lane: lane_game(info, me),
    })
}

fn lane_game(info: &MatchInfo, me: &MatchParticipant) -> Option<LaneGame> {
    let role = Role::from_lcu(&me.team_position);
    if role == Role::Unknown {
        return None;
    }

    let mut opponents = info.participants.iter().filter(|p| {
        p.team_id != me.team_id && p.team_position.eq_ignore_ascii_case(&me.team_position)
    });
    let opponent = opponents.next()?;
    if opponents.next().is_some() {
        return None; // dois na mesma rota do outro lado: não há oponente direto
    }

    let diffs = Metric::ALL
        .iter()
        .filter_map(|&metric| {
            let mine = metric.value(me, role)?;
            let theirs = metric.value(opponent, role)?;
            Some((metric, mine - theirs))
        })
        .collect();

    Some(LaneGame {
        champion: me.champion_id,
        opponent: opponent.champion_id,
        role,
        won: me.win,
        diffs,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const EU: &str = "eu";

    fn participante(puuid: &str, campeao: i64, time: i64, rota: &str) -> MatchParticipant {
        MatchParticipant {
            puuid: puuid.into(),
            champion_id: campeao,
            team_id: time,
            team_position: rota.into(),
            win: time == 100,
            game_ended_in_early_surrender: false,
            deaths: 3,
            challenges: Some(Challenges {
                lane_minions_first_10: Some(70.0),
                jungle_cs_before_10: Some(50.0),
                kill_participation: Some(0.5),
                team_damage_share: Some(0.2),
                vision_per_minute: Some(1.0),
            }),
        }
    }

    fn partida() -> MatchDto {
        let rotas = ["TOP", "JUNGLE", "MIDDLE", "BOTTOM", "UTILITY"];
        let mut participants = Vec::new();
        for (i, rota) in rotas.iter().enumerate() {
            let puuid = if *rota == "MIDDLE" { EU } else { "aliado" };
            participants.push(participante(puuid, 10 + i as i64, 100, rota));
        }
        for (i, rota) in rotas.iter().enumerate() {
            participants.push(participante("inimigo", 20 + i as i64, 200, rota));
        }
        MatchDto {
            info: MatchInfo {
                game_duration: 1800,
                game_mode: "CLASSIC".into(),
                end_of_game_result: "GameComplete".into(),
                participants,
            },
        }
    }

    fn eu(m: &mut MatchDto) -> &mut MatchParticipant {
        m.info.participants.iter_mut().find(|p| p.puuid == EU).unwrap()
    }

    #[test]
    fn oponente_e_a_mesma_rota_do_outro_time() {
        let jogo = played(&partida(), EU).unwrap();
        assert_eq!(jogo.champion, 12);
        assert!(jogo.won);
        let rota = jogo.lane.unwrap();
        assert_eq!(rota.opponent, 22);
        assert_eq!(rota.role, Role::Mid);
    }

    #[test]
    fn diferenca_e_voce_menos_ele_na_unidade_da_tela() {
        let mut m = partida();
        let me = eu(&mut m);
        me.deaths = 1;
        let c = me.challenges.as_mut().unwrap();
        c.lane_minions_first_10 = Some(82.0);
        c.kill_participation = Some(0.62);

        let diffs = played(&m, EU).unwrap().lane.unwrap().diffs;
        assert_eq!(diffs[&Metric::Cs10], 12.0);
        assert!((diffs[&Metric::KillParticipation] - 12.0).abs() < 1e-9);
        assert_eq!(diffs[&Metric::Deaths], -2.0);
    }

    #[test]
    fn cacador_compara_a_selva() {
        let mut m = partida();
        m.info.participants[1].puuid = EU.into();
        eu(&mut m).challenges.as_mut().unwrap().jungle_cs_before_10 = Some(58.0);
        m.info.participants[2].puuid = "aliado".into();

        let rota = played(&m, EU).unwrap().lane.unwrap();
        assert_eq!(rota.role, Role::Jungle);
        assert_eq!(rota.diffs[&Metric::Cs10], 8.0);
    }

    #[test]
    fn suporte_nao_compara_tropa() {
        let mut m = partida();
        m.info.participants[4].puuid = EU.into();
        m.info.participants[2].puuid = "aliado".into();

        let diffs = played(&m, EU).unwrap().lane.unwrap().diffs;
        assert!(!diffs.contains_key(&Metric::Cs10));
        assert!(diffs.contains_key(&Metric::VisionPerMinute));
    }

    #[test]
    fn remake_nao_conta_nem_para_vitoria() {
        let mut m = partida();
        m.info.participants[7].game_ended_in_early_surrender = true;
        assert_eq!(played(&m, EU), Err(Skip::Remake));
    }

    #[test]
    fn partida_curta_terminada_ou_fora_da_fenda_fica_de_fora() {
        let mut curta = partida();
        curta.info.game_duration = 200;
        assert_eq!(played(&curta, EU), Err(Skip::Curta));

        let mut erro = partida();
        erro.info.end_of_game_result = "Abort_Unexpected".into();
        assert_eq!(played(&erro, EU), Err(Skip::Terminada));

        let mut aram = partida();
        aram.info.game_mode = "ARAM".into();
        assert_eq!(played(&aram, EU), Err(Skip::Modo));
    }

    #[test]
    fn jogador_fora_da_partida() {
        assert_eq!(played(&partida(), "outro"), Err(Skip::Ausente));
    }

    #[test]
    fn rota_em_branco_conta_vitoria_sem_comparar_rota() {
        let mut m = partida();
        eu(&mut m).team_position = String::new();
        let jogo = played(&m, EU).unwrap();
        assert_eq!(jogo.champion, 12);
        assert!(jogo.lane.is_none());
    }

    #[test]
    fn oponente_ambiguo_nao_vira_comparacao() {
        let mut m = partida();
        m.info.participants[8].team_position = "MIDDLE".into();
        assert!(played(&m, EU).unwrap().lane.is_none());
    }

    #[test]
    fn sem_challenges_sobra_so_mortes() {
        let mut m = partida();
        eu(&mut m).challenges = None;
        let diffs = played(&m, EU).unwrap().lane.unwrap().diffs;
        assert_eq!(diffs.keys().copied().collect::<Vec<_>>(), vec![Metric::Deaths]);
    }

    #[test]
    fn le_os_nomes_de_campo_da_riot() {
        // Trava os nomes do JSON: renomear um campo aqui sem a Riot ter mudado
        // zera a métrica em silêncio.
        let json = r#"{
            "metadata": { "matchId": "BR1_1" },
            "info": {
                "gameDuration": 1650,
                "gameMode": "CLASSIC",
                "endOfGameResult": "GameComplete",
                "participants": [
                    { "puuid": "eu", "championId": 103, "teamId": 100,
                      "teamPosition": "MIDDLE", "win": true,
                      "gameEndedInEarlySurrender": false, "deaths": 2,
                      "challenges": { "laneMinionsFirst10Minutes": 81,
                                      "killParticipation": 0.64,
                                      "teamDamagePercentage": 0.31,
                                      "visionScorePerMinute": 0.9 } },
                    { "puuid": "ele", "championId": 238, "teamId": 200,
                      "teamPosition": "MIDDLE", "win": false,
                      "gameEndedInEarlySurrender": false, "deaths": 6,
                      "challenges": { "laneMinionsFirst10Minutes": 74,
                                      "killParticipation": 0.5,
                                      "teamDamagePercentage": 0.25,
                                      "visionScorePerMinute": 0.7 } }
                ]
            }
        }"#;
        let m: MatchDto = serde_json::from_str(json).unwrap();
        let rota = played(&m, "eu").unwrap().lane.unwrap();

        assert_eq!(rota.champion, 103);
        assert_eq!(rota.opponent, 238);
        assert_eq!(rota.diffs[&Metric::Cs10], 7.0);
        assert!((rota.diffs[&Metric::KillParticipation] - 14.0).abs() < 1e-9);
        assert!((rota.diffs[&Metric::DamageShare] - 6.0).abs() < 1e-9);
        assert!((rota.diffs[&Metric::VisionPerMinute] - 0.2).abs() < 1e-9);
        assert_eq!(rota.diffs[&Metric::Deaths], -4.0);
    }
}
