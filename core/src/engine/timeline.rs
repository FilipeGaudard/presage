//! Estado de objetivos a partir dos eventos da partida.
//!
//! Há duas naturezas de número aqui, e a diferença importa:
//!
//! - **Observado**: vimos o abate no fluxo de eventos. O respawn sai de uma
//!   soma exata.
//! - **Assumido**: ninguém matou ainda, então o primeiro spawn vem de uma
//!   constante de regra do jogo.
//!
//! `Spawn::observed` marca qual é qual, e a interface mostra isso — um número
//! assumido que o patch mudou não pode se passar por certeza.

use serde::{Deserialize, Serialize};

use crate::live::LiveGame;

/// Regras do jogo, não dados. **Mudam a cada temporada** e precisam ser
/// conferidas contra o patch: na 16, Barão voltou de 25:00 para 20:00, Arauto
/// passou a ser único às 15:00 e o Atakhan foi removido.
///
/// Ficam todas aqui de propósito: a lógica é testada contra valores
/// arbitrários, então trocar uma constante não quebra nada além dela mesma.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Timings {
    pub dragon_first: f64,
    pub dragon_respawn: f64,
    pub elder_respawn: f64,
    pub baron_first: f64,
    pub baron_respawn: f64,
    /// Arauto e Grubs aparecem uma vez e não voltam.
    pub herald_spawn: f64,
    pub grubs_spawn: f64,
    pub dragons_for_soul: u32,
}

impl Default for Timings {
    fn default() -> Self {
        Timings {
            dragon_first: 300.0,
            dragon_respawn: 300.0,
            elder_respawn: 360.0,
            baron_first: 1200.0,
            baron_respawn: 360.0,
            herald_spawn: 900.0,
            grubs_spawn: 480.0,
            dragons_for_soul: 4,
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct RawEvent {
    #[serde(rename = "EventName", default)]
    pub name: String,
    #[serde(rename = "EventTime", default)]
    pub time: f64,
    #[serde(rename = "DragonType", default)]
    pub dragon_type: String,
    #[serde(rename = "KillerName", default)]
    pub killer: String,
    #[serde(rename = "TurretKilled", default)]
    pub turret: String,
    /// Só o `GameEnd` traz: `Win` ou `Lose`, do ponto de vista do jogador
    /// local. É a única fonte do resultado que não exige a chave da Riot.
    #[serde(rename = "Result", default)]
    pub result: String,
}

impl RawEvent {
    /// `Some(true)` numa vitória, `Some(false)` numa derrota, `None` em
    /// qualquer outro evento.
    pub fn outcome(&self) -> Option<bool> {
        if !self.name.eq_ignore_ascii_case("GameEnd") {
            return None;
        }
        match self.result.to_ascii_lowercase().as_str() {
            "win" => Some(true),
            "lose" | "loss" => Some(false),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct EventFeed {
    #[serde(rename = "Events", default)]
    pub events: Vec<RawEvent>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize)]
pub struct Sides {
    pub ally: u32,
    pub enemy: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Spawn {
    pub name: String,
    /// Instante do spawn, em segundos de partida.
    pub at: f64,
    pub seconds_left: f64,
    /// `true` quando sai de um abate que vimos; `false` quando é a constante.
    pub observed: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Timeline {
    pub game_time: f64,
    pub dragons: Sides,
    pub dragon_types: Vec<String>,
    /// Quem fechou alma, se alguém fechou.
    pub soul: Option<String>,
    pub turrets: Sides,
    pub kills: Sides,
    pub upcoming: Vec<Spawn>,
}

fn team_of<'a>(game: &'a LiveGame, summoner: &str) -> Option<&'a str> {
    game.players
        .iter()
        .find(|player| {
            player.display_name == summoner
                || player
                    .display_name
                    .split('#')
                    .next()
                    .is_some_and(|name| name == summoner)
        })
        .map(|player| player.team.as_str())
}

pub fn build(events: &[RawEvent], game: &LiveGame, timings: &Timings) -> Timeline {
    let my_team = game.me().map(|me| me.team.clone()).unwrap_or_default();
    let now = game.game_time;

    let mut dragons = Sides::default();
    let mut dragon_types = Vec::new();
    let mut turrets = Sides::default();
    let mut kills = Sides::default();
    let mut last_dragon: Option<f64> = None;
    let mut last_baron: Option<f64> = None;
    let mut herald_taken = false;
    let mut grubs_taken = false;

    let tally = |sides: &mut Sides, killer: &str| match team_of(game, killer) {
        Some(team) if team == my_team => sides.ally += 1,
        Some(_) => sides.enemy += 1,
        // Torre derrubada por minions vem sem dono. Conta como ninguém.
        None => {}
    };

    for event in events {
        match event.name.as_str() {
            "DragonKill" => {
                last_dragon = Some(event.time);
                if !event.dragon_type.is_empty() {
                    dragon_types.push(event.dragon_type.clone());
                }
                tally(&mut dragons, &event.killer);
            }
            "BaronKill" => last_baron = Some(event.time),
            "HeraldKill" => herald_taken = true,
            "VoidgrubKill" | "HordeKill" => grubs_taken = true,
            "TurretKilled" => tally(&mut turrets, &event.killer),
            "ChampionKill" => tally(&mut kills, &event.killer),
            _ => {}
        }
    }

    let soul = if dragons.ally >= timings.dragons_for_soul {
        Some("ally".to_string())
    } else if dragons.enemy >= timings.dragons_for_soul {
        Some("enemy".to_string())
    } else {
        None
    };

    let mut upcoming = Vec::new();

    // Com alma fechada, o pit passa a ser do Ancião.
    let dragon_interval = if soul.is_some() {
        timings.elder_respawn
    } else {
        timings.dragon_respawn
    };
    let dragon_name = if soul.is_some() { "Ancião" } else { "Dragão" };

    match last_dragon {
        Some(at) => upcoming.push(spawn(dragon_name, at + dragon_interval, now, true)),
        None => upcoming.push(spawn(dragon_name, timings.dragon_first, now, false)),
    }

    match last_baron {
        Some(at) => upcoming.push(spawn("Barão", at + timings.baron_respawn, now, true)),
        None => upcoming.push(spawn("Barão", timings.baron_first, now, false)),
    }

    if !herald_taken && now < timings.herald_spawn + 300.0 {
        upcoming.push(spawn("Arauto", timings.herald_spawn, now, false));
    }
    if !grubs_taken && now < timings.grubs_spawn + 300.0 {
        upcoming.push(spawn("Grubs", timings.grubs_spawn, now, false));
    }

    upcoming.sort_by(|a, b| {
        a.seconds_left
            .partial_cmp(&b.seconds_left)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Timeline {
        game_time: now,
        dragons,
        dragon_types,
        soul,
        turrets,
        kills,
        upcoming,
    }
}

fn spawn(name: &str, at: f64, now: f64, observed: bool) -> Spawn {
    Spawn {
        name: name.to_string(),
        at,
        seconds_left: (at - now).max(0.0),
        observed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::live::{LivePlayer, RawScores};

    fn player(name: &str, team: &str, is_local: bool) -> LivePlayer {
        LivePlayer {
            champion_key: "X".into(),
            champion_id: 1,
            display_name: name.into(),
            team: team.into(),
            position: "MIDDLE".into(),
            level: 11,
            items: vec![],
            gold_invested: 0.0,
            is_dead: false,
            respawn_timer: 0.0,
            is_local,
            scores: RawScores::default(),
        }
    }

    fn game(time: f64) -> LiveGame {
        LiveGame {
            game_time: time,
            players: vec![
                player("eu#BR1", "ORDER", true),
                player("aliado#BR1", "ORDER", false),
                player("inimigo#BR1", "CHAOS", false),
            ],
        }
    }

    fn event(name: &str, time: f64, killer: &str) -> RawEvent {
        RawEvent {
            name: name.into(),
            time,
            killer: killer.into(),
            ..Default::default()
        }
    }

    fn dragon(time: f64, killer: &str, kind: &str) -> RawEvent {
        RawEvent {
            name: "DragonKill".into(),
            time,
            killer: killer.into(),
            dragon_type: kind.into(),
            ..Default::default()
        }
    }

    fn find<'a>(timeline: &'a Timeline, name: &str) -> &'a Spawn {
        timeline
            .upcoming
            .iter()
            .find(|spawn| spawn.name == name)
            .expect("spawn na lista")
    }

    #[test]
    fn sem_eventos_usa_as_constantes_e_marca_como_assumido() {
        let timeline = build(&[], &game(60.0), &Timings::default());
        let dragao = find(&timeline, "Dragão");

        assert_eq!(dragao.at, 300.0);
        assert_eq!(dragao.seconds_left, 240.0);
        assert!(!dragao.observed, "sem abate, o número é assumido");
    }

    #[test]
    fn abate_observado_vira_soma_exata() {
        let eventos = vec![dragon(400.0, "inimigo#BR1", "Fire")];
        let timeline = build(&eventos, &game(500.0), &Timings::default());
        let dragao = find(&timeline, "Dragão");

        assert_eq!(dragao.at, 700.0);
        assert_eq!(dragao.seconds_left, 200.0);
        assert!(dragao.observed);
    }

    #[test]
    fn atribui_o_dragao_ao_time_certo() {
        let eventos = vec![
            dragon(400.0, "aliado#BR1", "Fire"),
            dragon(700.0, "inimigo#BR1", "Ocean"),
        ];
        let timeline = build(&eventos, &game(800.0), &Timings::default());

        assert_eq!(timeline.dragons, Sides { ally: 1, enemy: 1 });
        assert_eq!(timeline.dragon_types, vec!["Fire", "Ocean"]);
    }

    #[test]
    fn alma_fechada_troca_o_pit_para_anciao() {
        let eventos: Vec<RawEvent> = (0..4)
            .map(|i| dragon(300.0 + 300.0 * i as f64, "inimigo#BR1", "Fire"))
            .collect();
        let timeline = build(&eventos, &game(1600.0), &Timings::default());

        assert_eq!(timeline.soul.as_deref(), Some("enemy"));
        assert!(timeline.upcoming.iter().all(|s| s.name != "Dragão"));
        // Quarto dragão aos 1200, Ancião 6 minutos depois.
        assert_eq!(find(&timeline, "Ancião").at, 1200.0 + 360.0);
    }

    #[test]
    fn barao_sem_abate_usa_o_primeiro_spawn() {
        let timeline = build(&[], &game(600.0), &Timings::default());
        let barao = find(&timeline, "Barão");

        assert_eq!(barao.at, 1200.0);
        assert!(!barao.observed);
    }

    #[test]
    fn barao_abatido_respawna_seis_minutos_depois() {
        let eventos = vec![event("BaronKill", 1300.0, "inimigo#BR1")];
        let timeline = build(&eventos, &game(1400.0), &Timings::default());
        let barao = find(&timeline, "Barão");

        assert_eq!(barao.at, 1660.0);
        assert!(barao.observed);
    }

    #[test]
    fn arauto_e_grubs_somem_da_lista_depois_de_tomados() {
        let eventos = vec![
            event("HeraldKill", 920.0, "aliado#BR1"),
            event("VoidgrubKill", 500.0, "aliado#BR1"),
        ];
        let timeline = build(&eventos, &game(1000.0), &Timings::default());

        assert!(timeline.upcoming.iter().all(|s| s.name != "Arauto"));
        assert!(timeline.upcoming.iter().all(|s| s.name != "Grubs"));
    }

    #[test]
    fn objetivo_que_ja_passou_da_janela_nao_aparece() {
        // Grubs às 8:00 somem do mapa; não faz sentido listar aos 30 minutos.
        let timeline = build(&[], &game(1800.0), &Timings::default());
        assert!(timeline.upcoming.iter().all(|s| s.name != "Grubs"));
    }

    #[test]
    fn torre_derrubada_por_minions_nao_conta_para_ninguem() {
        let eventos = vec![event("TurretKilled", 700.0, "Minion_T100")];
        let timeline = build(&eventos, &game(800.0), &Timings::default());
        assert_eq!(timeline.turrets, Sides::default());
    }

    #[test]
    fn abates_sao_separados_por_time() {
        let eventos = vec![
            event("ChampionKill", 100.0, "aliado#BR1"),
            event("ChampionKill", 200.0, "eu#BR1"),
            event("ChampionKill", 300.0, "inimigo#BR1"),
        ];
        let timeline = build(&eventos, &game(400.0), &Timings::default());
        assert_eq!(timeline.kills, Sides { ally: 2, enemy: 1 });
    }

    #[test]
    fn killer_pelo_nome_curto_tambem_resolve() {
        // Os eventos usam o nome de invocador, sem a tag.
        let eventos = vec![event("ChampionKill", 100.0, "aliado")];
        let timeline = build(&eventos, &game(200.0), &Timings::default());
        assert_eq!(timeline.kills.ally, 1);
    }

    #[test]
    fn tempo_restante_nunca_fica_negativo() {
        let eventos = vec![dragon(100.0, "aliado#BR1", "Fire")];
        let timeline = build(&eventos, &game(9999.0), &Timings::default());
        assert!(timeline.upcoming.iter().all(|s| s.seconds_left >= 0.0));
    }

    #[test]
    fn lista_sai_ordenada_pelo_mais_proximo() {
        let timeline = build(&[], &game(60.0), &Timings::default());
        let tempos: Vec<f64> = timeline.upcoming.iter().map(|s| s.seconds_left).collect();
        let mut ordenado = tempos.clone();
        ordenado.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert_eq!(tempos, ordenado);
    }

    #[test]
    fn mudar_a_constante_nao_quebra_a_logica() {
        let custom = Timings {
            baron_first: 1500.0,
            ..Default::default()
        };
        let timeline = build(&[], &game(60.0), &custom);
        assert_eq!(find(&timeline, "Barão").at, 1500.0);
    }
}
