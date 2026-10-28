//! Cliente da Riot Web API.
//!
//! A chave nunca fica no repositório nem no código: vem da variável de
//! ambiente `PRESAGE_RIOT_API_KEY`, que o usuário define na própria máquina.
//! Sem ela o Presage segue funcionando, só sem o componente pessoal.
//!
//! Chave de desenvolvedor expira a cada 24h e aceita 20 req/s, 100 req/2min.
//! Respeitamos isso espaçando as chamadas; ingestão em massa precisaria de uma
//! chave de produção, que é outro processo.

use anyhow::{anyhow, Result};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::time::Duration;

pub use presage_core::history::MatchDto;
pub use presage_core::riot::{split_riot_id, urlencode, Region};

pub const ENV_API_KEY: &str = "PRESAGE_RIOT_API_KEY";
pub const ENV_RIOT_ID: &str = "PRESAGE_RIOT_ID";

/// 1,3s entre chamadas mantém ~92 requisições em 2 minutos, dentro do limite
/// da chave de desenvolvedor com folga.
const CALL_SPACING: Duration = Duration::from_millis(1300);

const RANKED_SOLO_QUEUE: u32 = 420;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub puuid: String,
    #[serde(default)]
    pub game_name: String,
    #[serde(default)]
    pub tag_line: String,
}

pub struct RiotClient {
    http: reqwest::Client,
    key: String,
    region: Region,
}

impl RiotClient {
    pub fn from_env(region: Region) -> Option<Self> {
        let key = std::env::var(ENV_API_KEY).ok()?;
        if key.trim().is_empty() {
            return None;
        }
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .ok()?;
        Some(Self { http, key, region })
    }

    async fn get<T: DeserializeOwned>(&self, host: &str, path: &str) -> Result<T> {
        let url = format!("https://{host}.api.riotgames.com{path}");

        for attempt in 0..3 {
            let response = self
                .http
                .get(&url)
                .header("X-Riot-Token", &self.key)
                .send()
                .await?;

            let status = response.status();

            if status.as_u16() == 429 {
                let wait = response
                    .headers()
                    .get("Retry-After")
                    .and_then(|value| value.to_str().ok())
                    .and_then(|value| value.parse::<u64>().ok())
                    .unwrap_or(10);
                tracing::warn!(wait, attempt, "rate limit da Riot, aguardando");
                tokio::time::sleep(Duration::from_secs(wait)).await;
                continue;
            }

            if status.as_u16() == 401 || status.as_u16() == 403 {
                return Err(anyhow!(
                    "a Riot recusou a chave ({status}). Chave de desenvolvedor expira a cada 24h."
                ));
            }

            if !status.is_success() {
                // O path carrega o puuid, então a URL completa não entra no erro.
                return Err(anyhow!("Riot API respondeu {status}"));
            }

            return Ok(response.json::<T>().await?);
        }

        Err(anyhow!("Riot API seguiu limitando após 3 tentativas"))
    }

    pub async fn account_by_riot_id(&self, game_name: &str, tag_line: &str) -> Result<Account> {
        let path = format!(
            "/riot/account/v1/accounts/by-riot-id/{}/{}",
            urlencode(game_name),
            urlencode(tag_line)
        );
        self.get(self.region.routing, &path).await
    }

    pub async fn ranked_match_ids(&self, puuid: &str, count: u8) -> Result<Vec<String>> {
        let path = format!(
            "/lol/match/v5/matches/by-puuid/{puuid}/ids?queue={RANKED_SOLO_QUEUE}&start=0&count={count}"
        );
        self.get(self.region.routing, &path).await
    }

    /// A partida inteira. Quem lê é `presage_core::history::played`: a chamada
    /// já paga pelo payload completo, e é dele que sai a comparação de rota.
    pub async fn match_detail(&self, match_id: &str) -> Result<MatchDto> {
        tokio::time::sleep(CALL_SPACING).await;
        self.get(
            self.region.routing,
            &format!("/lol/match/v5/matches/{match_id}"),
        )
        .await
    }
}
