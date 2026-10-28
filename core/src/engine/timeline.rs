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
//!
//! Inibidor e torre do nexo derrubados voltam depois de um tempo fixo
//! (`Respawn`). O buff de Barão e de Ancião é de quem estava vivo no abate e
//! some de quem morre: saber quem ainda tem pede lembrar da volta anterior, e
//! por isso ele mora num rastreador com estado (`BuffTracker`), não em `build`.
//! Tudo vem dos eventos da Live Client, que o jogo anuncia na tela de todo
//! mundo ([C-4](../../../docs/produto/requisitos.md)).

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
    pub inhibitor_respawn: f64,
    pub nexus_turret_respawn: f64,
    pub baron_buff: f64,
    pub elder_buff: f64,
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
            inhibitor_respawn: 300.0,
            nexus_turret_respawn: 180.0,
            baron_buff: 180.0,
            elder_buff: 150.0,
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
    #[serde(rename = "InhibKilled", default)]
    pub inhibitor: String,
    /// Só o `ChampionKill` traz: quem morreu.
    #[serde(rename = "VictimName", default)]
    pub victim: String,
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
    /// Inibidores e torres do nexo derrubados que ainda não voltaram.
    pub respawns: Vec<Respawn>,
}

/// Estrutura derrubada esperando voltar.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Respawn {
    /// `Inibidor do topo`, `Torre do nexo`.
    pub name: String,
    /// A estrutura é do seu time.
    pub ours: bool,
    pub at: f64,
    pub seconds_left: f64,
}

#[derive(Debug, Clone, Copy)]
enum Estrutura {
    Inibidor(&'static str),
    TorreDoNexo,
}

/// Lê o nome da estrutura. Há dois formatos no mundo, e os dois são aceitos:
///
/// - `Inhib_TOrder_L2`, `Turret_TOrder_L1_P5` — o que apps em produção leem
///   hoje: `L2`, `L1`, `L0` são topo, meio e bot; `L1_P4`/`L1_P5`, as torres do
///   nexo.
/// - `Barracks_T1_L1`, `Turret_T1_C_01_A` — o da documentação da Riot: `L`,
///   `C`, `R` são topo, meio e bot; `C_01`/`C_02`, as torres do nexo.
///
/// Devolve o time dono da estrutura (`ORDER`/`CHAOS`, como na Live Client).
fn estrutura(nome: &str) -> Option<(&'static str, Estrutura)> {
    let partes: Vec<&str> = nome.split('_').collect();
    let time = match partes.get(1).copied()? {
        "TOrder" | "T1" => "ORDER",
        "TChaos" | "T2" => "CHAOS",
        _ => return None,
    };
    let estrutura = match (partes[0], partes.get(2).copied()?, partes.get(3).copied()) {
        ("Inhib", "L2", _) | ("Barracks", "L1", _) => Estrutura::Inibidor("topo"),
        ("Inhib", "L1", _) | ("Barracks", "C1", _) => Estrutura::Inibidor("meio"),
        ("Inhib", "L0", _) | ("Barracks", "R1", _) => Estrutura::Inibidor("bot"),
        ("Turret", "L1", Some("P4" | "P5")) | ("Turret", "C", Some("01" | "02")) => Estrutura::TorreDoNexo,
        _ => return None,
    };
    Some((time, estrutura))
}

fn respawns(events: &[RawEvent], game: &LiveGame, timings: &Timings) -> Vec<Respawn> {
    let my_team = game.me().map(|me| me.team.as_str()).unwrap_or("");
    let now = game.game_time;
    let mut saida: Vec<Respawn> = events
        .iter()
        .filter_map(|event| {
            let (nome, tempo) = match event.name.as_str() {
                "InhibKilled" => (&event.inhibitor, timings.inhibitor_respawn),
                "TurretKilled" => (&event.turret, timings.nexus_turret_respawn),
                _ => return None,
            };
            let (time, estrutura) = estrutura(nome)?;
            let at = event.time + tempo;
            (at > now).then(|| Respawn {
                name: match estrutura {
                    Estrutura::Inibidor(rota) => format!("Inibidor do {rota}"),
                    Estrutura::TorreDoNexo => "Torre do nexo".to_string(),
                },
                ours: time == my_team,
                at,
                seconds_left: at - now,
            })
        })
        .collect();
    saida.sort_by(|a, b| a.at.partial_cmp(&b.at).unwrap_or(std::cmp::Ordering::Equal));
    saida
}

/// Buff de Barão ou de Ancião em andamento.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Buff {
    /// `Barão` ou `Ancião`.
    pub name: String,
    pub ours: bool,
    pub started: f64,
    pub until: f64,
    pub seconds_left: f64,
    /// Quem ainda tem o buff, por campeão. `None` quando o abate aconteceu
    /// antes de o app olhar e não dá para saber quem estava vivo.
    pub holders: Option<Vec<i64>>,
}

/// Até quanto depois do abate ainda dá para ler quem estava vivo. A Live Client
/// é lida a cada segundo; acima disso alguém pode ter morrido ou renascido no
/// meio.
const BUFF_FRESH: f64 = 5.0;

/// Lembra os buffs entre uma leitura e a próxima.
#[derive(Debug, Clone, Default)]
pub struct BuffTracker {
    buffs: Vec<Buff>,
}

impl BuffTracker {
    pub fn update(&mut self, events: &[RawEvent], game: &LiveGame, timings: &Timings) -> Vec<Buff> {
        let now = game.game_time;
        let my_team = game.me().map(|me| me.team.clone()).unwrap_or_default();

        for event in events {
            let (nome, duracao) = match event.name.as_str() {
                "BaronKill" => ("Barão", timings.baron_buff),
                "DragonKill" if event.dragon_type.eq_ignore_ascii_case("Elder") => ("Ancião", timings.elder_buff),
                _ => continue,
            };
            let visto = self
                .buffs
                .iter()
                .any(|b| b.name == nome && (b.started - event.time).abs() < 0.5);
            if visto || event.time + duracao <= now {
                continue;
            }
            let Some(time) = team_of(game, &event.killer).map(str::to_string) else {
                continue;
            };
            let holders = (now - event.time <= BUFF_FRESH).then(|| {
                game.players
                    .iter()
                    .filter(|p| p.team == time && !p.is_dead)
                    .map(|p| p.champion_id)
                    .collect()
            });
            self.buffs.push(Buff {
                name: nome.to_string(),
                ours: time == my_team,
                started: event.time,
                until: event.time + duracao,
                seconds_left: event.time + duracao - now,
                holders,
            });
        }

        for buff in &mut self.buffs {
            buff.seconds_left = buff.until - now;
            if let Some(holders) = &mut buff.holders {
                holders.retain(|id| {
                    let Some(p) = game.players.iter().find(|p| p.champion_id == *id) else {
                        return false;
                    };
                    let morreu = events.iter().any(|e| {
                        e.name == "ChampionKill"
                            && e.time > buff.started
                            && player_named(game, &e.victim).is_some_and(|v| v.champion_id == *id)
                    });
                    !p.is_dead && !morreu
                });
            }
        }
        self.buffs
            .retain(|b| b.until > now && b.holders.as_ref().map_or(true, |h| !h.is_empty()));
        self.buffs.clone()
    }
}

fn player_named<'a>(game: &'a LiveGame, summoner: &str) -> Option<&'a crate::live::LivePlayer> {
    if summoner.is_empty() {
        return None;
    }
    game.players.iter().find(|player| {
        player.display_name == summoner
            || player
                .display_name
                .split('#')
                .next()
                .is_some_and(|name| name == summoner)
    })
}

fn team_of<'a>(game: &'a LiveGame, summoner: &str) -> Option<&'a str> {
    player_named(game, summoner).map(|player| player.team.as_str())
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
        respawns: respawns(events, game, timings),
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

    fn estrutura_caiu(name: &str, time: f64, campo: &str) -> RawEvent {
        RawEvent {
            name: name.into(),
            time,
            inhibitor: if name == "InhibKilled" { campo.into() } else { String::new() },
            turret: if name == "TurretKilled" { campo.into() } else { String::new() },
            ..Default::default()
        }
    }

    #[test]
    fn inibidor_volta_em_cinco_minutos_nos_dois_formatos_de_nome() {
        // Você é ORDER: o inibidor TChaos/T2 é do inimigo.
        for nome in ["Inhib_TChaos_L1", "Barracks_T2_C1"] {
            let t = build(&[estrutura_caiu("InhibKilled", 1500.0, nome)], &game(1600.0), &Timings::default());
            assert_eq!(t.respawns.len(), 1, "{nome}");
            let r = &t.respawns[0];
            assert_eq!(r.name, "Inibidor do meio");
            assert!(!r.ours);
            assert_eq!(r.at, 1800.0);
            assert_eq!(r.seconds_left, 200.0);
        }
    }

    #[test]
    fn a_rota_do_inibidor_sai_do_nome() {
        let caso = |nome: &str| {
            build(&[estrutura_caiu("InhibKilled", 1500.0, nome)], &game(1501.0), &Timings::default()).respawns[0]
                .name
                .clone()
        };
        assert_eq!(caso("Inhib_TOrder_L2"), "Inibidor do topo");
        assert_eq!(caso("Inhib_TOrder_L0"), "Inibidor do bot");
        assert_eq!(caso("Barracks_T1_L1"), "Inibidor do topo");
        assert_eq!(caso("Barracks_T1_R1"), "Inibidor do bot");
    }

    #[test]
    fn torre_do_nexo_volta_e_torre_comum_nao() {
        let eventos = [
            estrutura_caiu("TurretKilled", 1700.0, "Turret_TOrder_L1_P5"),
            estrutura_caiu("TurretKilled", 1700.0, "Turret_T1_C_02_A"),
            estrutura_caiu("TurretKilled", 1700.0, "Turret_TOrder_L1_P2"),
            estrutura_caiu("TurretKilled", 1700.0, "Turret_T1_L_03_A"),
        ];
        let t = build(&eventos, &game(1750.0), &Timings::default());
        assert_eq!(t.respawns.len(), 2);
        assert!(t.respawns.iter().all(|r| r.name == "Torre do nexo" && r.ours));
        assert!(t.respawns.iter().all(|r| r.at == 1880.0));
    }

    #[test]
    fn estrutura_que_ja_voltou_sai_da_lista() {
        let t = build(&[estrutura_caiu("InhibKilled", 1000.0, "Inhib_TChaos_L1")], &game(1300.0), &Timings::default());
        assert!(t.respawns.is_empty());
    }

    #[test]
    fn nome_desconhecido_nao_vira_timer() {
        let t = build(&[estrutura_caiu("InhibKilled", 1000.0, "Algo_Novo_X")], &game(1001.0), &Timings::default());
        assert!(t.respawns.is_empty());
    }

    fn partida(time: f64, mortos: &[&str]) -> LiveGame {
        let mut g = LiveGame {
            game_time: time,
            players: vec![
                player("eu#BR1", "ORDER", true),
                player("aliado#BR1", "ORDER", false),
                player("outro#BR1", "ORDER", false),
                player("inimigo#BR1", "CHAOS", false),
            ],
        };
        for (i, p) in g.players.iter_mut().enumerate() {
            p.champion_id = 10 + i as i64;
            p.is_dead = mortos.contains(&p.display_name.as_str());
        }
        g
    }

    fn morte(time: f64, vitima: &str) -> RawEvent {
        RawEvent {
            name: "ChampionKill".into(),
            time,
            victim: vitima.into(),
            ..Default::default()
        }
    }

    #[test]
    fn buff_de_barao_e_de_quem_estava_vivo() {
        // O aliado está morto quando o time mata o Barão: ele não leva o buff.
        let mut rastreador = BuffTracker::default();
        let eventos = [event("BaronKill", 1300.0, "eu#BR1")];
        let buffs = rastreador.update(&eventos, &partida(1301.0, &["aliado#BR1"]), &Timings::default());
        assert_eq!(buffs.len(), 1);
        assert_eq!(buffs[0].name, "Barão");
        assert!(buffs[0].ours);
        assert_eq!(buffs[0].until, 1480.0);
        assert_eq!(buffs[0].holders, Some(vec![10, 12]));
    }

    #[test]
    fn quem_morre_perde_o_buff_mesmo_depois_de_renascer() {
        let mut rastreador = BuffTracker::default();
        let mut eventos = vec![event("BaronKill", 1300.0, "eu#BR1")];
        rastreador.update(&eventos, &partida(1301.0, &[]), &Timings::default());

        // `outro` morre e já renasceu na leitura seguinte: o evento conta.
        eventos.push(morte(1340.0, "outro"));
        let buffs = rastreador.update(&eventos, &partida(1380.0, &[]), &Timings::default());
        assert_eq!(buffs[0].holders, Some(vec![10, 11]));
    }

    #[test]
    fn buff_some_quando_ninguem_mais_tem_ou_quando_acaba() {
        let mut rastreador = BuffTracker::default();
        let eventos = [dragon(2000.0, "inimigo#BR1", "Elder")];
        let buffs = rastreador.update(&eventos, &partida(2001.0, &[]), &Timings::default());
        assert_eq!(buffs[0].name, "Ancião");
        assert!(!buffs[0].ours);
        assert_eq!(buffs[0].until, 2150.0);

        let todos_mortos = rastreador.update(&eventos, &partida(2010.0, &["inimigo#BR1"]), &Timings::default());
        assert!(todos_mortos.is_empty());

        let mut outro = BuffTracker::default();
        outro.update(&eventos, &partida(2001.0, &[]), &Timings::default());
        assert!(outro.update(&eventos, &partida(2151.0, &[]), &Timings::default()).is_empty());
    }

    #[test]
    fn abate_visto_tarde_mostra_o_tempo_sem_inventar_quem_tem() {
        let mut rastreador = BuffTracker::default();
        let eventos = [event("BaronKill", 1300.0, "eu#BR1")];
        let buffs = rastreador.update(&eventos, &partida(1340.0, &[]), &Timings::default());
        assert_eq!(buffs[0].holders, None);
        assert_eq!(buffs[0].seconds_left, 140.0);
    }

    #[test]
    fn dragao_comum_nao_da_buff_de_anciao() {
        let mut rastreador = BuffTracker::default();
        let buffs = rastreador.update(&[dragon(600.0, "eu#BR1", "Fire")], &partida(601.0, &[]), &Timings::default());
        assert!(buffs.is_empty());
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
