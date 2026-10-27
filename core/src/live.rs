//! Estado da partida em andamento, vindo da Live Client Data API.
//!
//! É a API oficial da Riot, roda em `127.0.0.1:2999` na máquina do jogador e
//! não pede chave nem autenticação. Entrega, para os dez: itens, nível, KDA,
//! CS, feitiços, se está morto e o tempo de renascimento.
//!
//! Aqui ficam só os tipos e a tradução. O HTTP está em src-tauri.

use serde::{Deserialize, Serialize};

use crate::engine::build::Combatant;
use crate::engine::champion::ChampionIndex;

const RAW_NAME_PREFIX: &str = "game_character_displayname_";

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct RawPlayer {
    pub champion_name: String,
    /// `game_character_displayname_Annie`. Independe de idioma, então é daqui
    /// que tiramos a chave do campeão.
    pub raw_champion_name: String,
    pub summoner_name: String,
    pub riot_id: String,
    pub team: String,
    pub position: String,
    pub level: u8,
    pub is_dead: bool,
    pub is_bot: bool,
    pub respawn_timer: f64,
    pub items: Vec<RawItem>,
    pub scores: RawScores,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct RawItem {
    #[serde(rename = "itemID")]
    pub item_id: i64,
    pub price: f64,
    pub slot: i64,
}

#[derive(Debug, Clone, Deserialize, Default, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct RawScores {
    pub kills: i64,
    pub deaths: i64,
    pub assists: i64,
    pub creep_score: i64,
    pub ward_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LivePlayer {
    pub champion_key: String,
    pub champion_id: i64,
    pub display_name: String,
    pub team: String,
    pub position: String,
    pub level: u8,
    pub items: Vec<i64>,
    /// Ouro investido em itens. Soma exata, não estimativa.
    pub gold_invested: f64,
    pub is_dead: bool,
    pub respawn_timer: f64,
    pub is_local: bool,
    pub scores: RawScores,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LiveGame {
    pub game_time: f64,
    pub players: Vec<LivePlayer>,
}

impl LiveGame {
    pub fn me(&self) -> Option<&LivePlayer> {
        self.players.iter().find(|player| player.is_local)
    }

    pub fn allies(&self) -> Vec<&LivePlayer> {
        match self.me() {
            Some(me) => self
                .players
                .iter()
                .filter(|player| player.team == me.team && !player.is_local)
                .collect(),
            None => Vec::new(),
        }
    }

    pub fn enemies(&self) -> Vec<&LivePlayer> {
        match self.me() {
            Some(me) => self
                .players
                .iter()
                .filter(|player| player.team != me.team)
                .collect(),
            None => Vec::new(),
        }
    }

    /// Ouro investido por time, do ponto de vista do jogador local.
    pub fn gold_invested(&self) -> (f64, f64) {
        let mine: f64 = self
            .me()
            .map(|me| me.gold_invested)
            .unwrap_or(0.0)
            + self.allies().iter().map(|p| p.gold_invested).sum::<f64>();
        let theirs: f64 = self.enemies().iter().map(|p| p.gold_invested).sum();
        (mine, theirs)
    }
}

impl LivePlayer {
    pub fn as_combatant(&self) -> Combatant {
        Combatant {
            champion_id: self.champion_id,
            level: self.level,
            items: self.items.clone(),
        }
    }
}

/// `game_character_displayname_Annie` -> `Annie`
pub fn champion_key(raw_champion_name: &str) -> &str {
    raw_champion_name
        .strip_prefix(RAW_NAME_PREFIX)
        .unwrap_or(raw_champion_name)
}

/// Monta o estado do jogo a partir da resposta crua.
///
/// `active` é o que `/activeplayername` devolve — pode vir como Riot ID
/// (`nome#tag`) ou como nome de invocador, dependendo da versão do cliente,
/// então comparamos com os dois campos.
pub fn to_live_game(
    raw: &[RawPlayer],
    active: &str,
    game_time: f64,
    champions: &ChampionIndex,
) -> LiveGame {
    let players = raw
        .iter()
        .filter(|player| !player.is_bot)
        .map(|player| {
            let key = champion_key(&player.raw_champion_name);
            let champion_id = champions
                .by_key(key)
                .map(|champion| champion.id)
                .unwrap_or(0);

            let display_name = if player.riot_id.is_empty() {
                player.summoner_name.clone()
            } else {
                player.riot_id.clone()
            };

            LivePlayer {
                champion_key: key.to_string(),
                champion_id,
                display_name,
                team: player.team.clone(),
                position: player.position.clone(),
                level: player.level,
                items: player.items.iter().map(|item| item.item_id).collect(),
                gold_invested: player.items.iter().map(|item| item.price).sum(),
                is_dead: player.is_dead,
                respawn_timer: player.respawn_timer,
                is_local: !active.is_empty()
                    && (player.riot_id == active || player.summoner_name == active),
                scores: player.scores.clone(),
            }
        })
        .collect();

    LiveGame { game_time, players }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::champion::fake;
    use std::collections::HashMap;

    fn champions() -> ChampionIndex {
        let annie = fake(1, "Annie", "Mage", 2.0, 9.0, 4.0);
        let mut lee = fake(64, "LeeSin", "Fighter", 8.0, 3.0, 5.0);
        lee.key = "LeeSin".into();
        let mut a = annie;
        a.key = "Annie".into();

        ChampionIndex {
            version: "test".into(),
            by_id: HashMap::from([(1, a), (64, lee)]),
        }
    }

    fn player(raw_name: &str, riot_id: &str, team: &str, items: Vec<(i64, f64)>) -> RawPlayer {
        RawPlayer {
            raw_champion_name: raw_name.to_string(),
            riot_id: riot_id.to_string(),
            summoner_name: riot_id.split('#').next().unwrap_or("").to_string(),
            team: team.to_string(),
            level: 11,
            items: items
                .into_iter()
                .map(|(item_id, price)| RawItem {
                    item_id,
                    price,
                    slot: 0,
                })
                .collect(),
            ..Default::default()
        }
    }

    #[test]
    fn chave_do_campeao_ignora_o_prefixo() {
        assert_eq!(champion_key("game_character_displayname_Annie"), "Annie");
        assert_eq!(champion_key("Annie"), "Annie");
    }

    #[test]
    fn identifica_o_jogador_local_pelo_riot_id() {
        let raw = vec![
            player("game_character_displayname_Annie", "eu#BR1", "ORDER", vec![]),
            player("game_character_displayname_LeeSin", "outro#BR1", "CHAOS", vec![]),
        ];
        let game = to_live_game(&raw, "eu#BR1", 600.0, &champions());

        assert_eq!(game.me().unwrap().champion_key, "Annie");
        assert_eq!(game.enemies().len(), 1);
    }

    #[test]
    fn identifica_pelo_nome_de_invocador_quando_nao_ha_riot_id() {
        let raw = vec![player("game_character_displayname_Annie", "eu#BR1", "ORDER", vec![])];
        let game = to_live_game(&raw, "eu", 0.0, &champions());
        assert!(game.me().is_some());
    }

    #[test]
    fn mapeia_a_chave_para_o_id_do_campeao() {
        let raw = vec![player("game_character_displayname_LeeSin", "x#BR1", "ORDER", vec![])];
        let game = to_live_game(&raw, "x#BR1", 0.0, &champions());
        assert_eq!(game.players[0].champion_id, 64);
    }

    #[test]
    fn campeao_desconhecido_vira_zero_sem_quebrar() {
        let raw = vec![player("game_character_displayname_Inexistente", "x#BR1", "ORDER", vec![])];
        let game = to_live_game(&raw, "x#BR1", 0.0, &champions());
        assert_eq!(game.players[0].champion_id, 0);
    }

    #[test]
    fn ouro_investido_e_soma_exata_dos_precos() {
        let raw = vec![
            player("game_character_displayname_Annie", "eu#BR1", "ORDER", vec![(3020, 1100.0), (3135, 3000.0)]),
            player("game_character_displayname_LeeSin", "ini#BR1", "CHAOS", vec![(3071, 3000.0)]),
        ];
        let game = to_live_game(&raw, "eu#BR1", 0.0, &champions());
        let (meu, deles) = game.gold_invested();

        assert_eq!(meu, 4100.0);
        assert_eq!(deles, 3000.0);
    }

    #[test]
    fn bots_ficam_de_fora() {
        let mut bot = player("game_character_displayname_Annie", "bot#BR1", "CHAOS", vec![]);
        bot.is_bot = true;
        let raw = vec![
            player("game_character_displayname_LeeSin", "eu#BR1", "ORDER", vec![]),
            bot,
        ];
        let game = to_live_game(&raw, "eu#BR1", 0.0, &champions());
        assert_eq!(game.players.len(), 1);
    }

    #[test]
    fn sem_jogador_local_as_listas_ficam_vazias_em_vez_de_mentir() {
        let raw = vec![player("game_character_displayname_Annie", "alguem#BR1", "ORDER", vec![])];
        let game = to_live_game(&raw, "ninguem#BR1", 0.0, &champions());

        assert!(game.me().is_none());
        assert!(game.enemies().is_empty());
        assert!(game.allies().is_empty());
    }

    #[test]
    fn vira_combatente_para_o_motor_de_build() {
        let raw = vec![player("game_character_displayname_Annie", "eu#BR1", "ORDER", vec![(3135, 3000.0)])];
        let game = to_live_game(&raw, "eu#BR1", 0.0, &champions());
        let combatant = game.me().unwrap().as_combatant();

        assert_eq!(combatant.champion_id, 1);
        assert_eq!(combatant.level, 11);
        assert_eq!(combatant.items, vec![3135]);
    }
}
