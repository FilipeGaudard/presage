//! Cliente da Live Client Data API.
//!
//! Oficial, local e sem autenticação. Só responde com uma partida em andamento;
//! fora de jogo a conexão é recusada, e isso é o normal, não um erro.

use anyhow::{anyhow, Result};
use serde::de::DeserializeOwned;
use std::time::Duration;

use presage_core::engine::priority::PriorityReport;
use presage_core::engine::rival::Rival;
use presage_core::engine::timeline::{Buff, EventFeed, RawEvent, Timeline};
use presage_core::live::{LiveGame, RawPlayer};

/// O que o front recebe de uma vez. Juntos de propósito: jogo, objetivos e
/// prioridade são lidos no mesmo instante e não podem divergir na tela.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveSnapshot {
    pub game: LiveGame,
    pub timeline: Timeline,
    pub priority: PriorityReport,
    /// Barão e Ancião em andamento, com quem ainda tem.
    pub buffs: Vec<Buff>,
    /// Você contra o oponente direto, no que a sua rota cobra.
    pub rival: Option<Rival>,
}

const BASE: &str = "https://127.0.0.1:2999/liveclientdata";

#[derive(Debug, serde::Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
struct GameStats {
    game_time: f64,
}

pub struct LiveClient {
    http: reqwest::Client,
}

impl LiveClient {
    pub fn new() -> Result<Self> {
        // Mesmo certificado autoassinado do cliente da Riot. Ver a nota em
        // lcu/client.rs: o destino é loopback, e o pinning do root CA entra
        // antes de qualquer release.
        let http = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .timeout(Duration::from_secs(3))
            .build()?;
        Ok(Self { http })
    }

    async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let response = self.http.get(format!("{BASE}{path}")).send().await?;
        if !response.status().is_success() {
            return Err(anyhow!("Live Client respondeu {} em {path}", response.status()));
        }
        Ok(response.json::<T>().await?)
    }

    /// Resposta crua da partida. Usa os endpoints estreitos em vez de
    /// `allgamedata`, que a própria doc da Riot marca como só para teste.
    ///
    /// Devolve cru de propósito: a tradução precisa do índice de campeões, que
    /// vive atrás de um lock, e segurar lock durante requisição é pedir
    /// problema.
    pub async fn snapshot(&self) -> Result<RawSnapshot> {
        let players: Vec<RawPlayer> = self.get("/playerlist").await?;
        let active: String = self.get("/activeplayername").await?;
        let stats: GameStats = self.get("/gamestats").await.unwrap_or_default();
        // Sem eventos a partida ainda roda; só perdemos os timers.
        let feed: EventFeed = self.get("/eventdata").await.unwrap_or_default();

        Ok(RawSnapshot {
            players,
            active,
            game_time: stats.game_time,
            events: feed.events,
        })
    }
}

pub struct RawSnapshot {
    pub players: Vec<RawPlayer>,
    pub active: String,
    pub game_time: f64,
    pub events: Vec<RawEvent>,
}
