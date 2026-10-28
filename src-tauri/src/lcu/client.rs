//! Cliente HTTP para a LCU API.

use anyhow::{anyhow, Result};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Deserialize)]
struct RegionLocale {
    region: String,
}

use presage_core::lockfile::Credentials;

pub struct LcuClient {
    http: reqwest::Client,
    base_url: String,
    auth_header: String,
}

impl LcuClient {
    pub fn new(creds: &Credentials) -> Result<Self> {
        // O cliente da Riot usa um certificado autoassinado em 127.0.0.1, então
        // a validação de cadeia falharia sempre.
        //
        // TODO(segurança): antes do release público, trocar por pinning do root
        // CA publicado pela Riot (riotgames.pem) via `Certificate::from_pem` +
        // `add_root_certificate`, em vez de desligar a verificação. O risco aqui
        // é baixo porque o destino é loopback, mas desligar a verificação é uma
        // decisão que merece ser revertida e não herdada sem querer.
        let http = reqwest::Client::builder()
            .danger_accept_invalid_certs(true)
            .timeout(Duration::from_secs(5))
            .build()?;

        let auth = BASE64.encode(format!("riot:{}", creds.token));

        Ok(Self {
            http,
            base_url: creds.base_url(),
            auth_header: format!("Basic {auth}"),
        })
    }

    pub async fn get_json<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let response = self
            .http
            .get(format!("{}{}", self.base_url, path))
            .header("Authorization", &self.auth_header)
            .header("Accept", "application/json")
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            return Err(anyhow!("LCU respondeu {status} para {path}"));
        }

        Ok(response.json::<T>().await?)
    }

    /// Fase atual do fluxo de jogo: `None`, `Lobby`, `Matchmaking`,
    /// `ReadyCheck`, `ChampSelect`, `InProgress`, `EndOfGame`...
    ///
    /// O endpoint devolve uma string JSON crua (`"ChampSelect"`).
    pub async fn gameflow_phase(&self) -> Result<String> {
        self.get_json::<String>("/lol-gameflow/v1/gameflow-phase").await
    }

    pub async fn champ_select_session(&self) -> Result<super::models::LcuSession> {
        self.get_json("/lol-champ-select/v1/session").await
    }

    pub async fn current_summoner(&self) -> Result<super::models::LcuSummoner> {
        self.get_json("/lol-summoner/v1/current-summoner").await
    }

    /// A grade do champ select: quem a conta tem, ganhou pelo Game Pass ou
    /// está na rotação. Só responde durante o champ select. Só leitura.
    pub async fn grid_champions(&self) -> Result<Vec<super::models::LcuGridChampion>> {
        self.get_json("/lol-champ-select/v1/all-grid-champions").await
    }

    /// A fila da partida, p.ex. `420` (solo/duo).
    pub async fn queue_id(&self) -> Result<i64> {
        let sessao: super::models::LcuGameflowSession =
            self.get_json("/lol-gameflow/v1/session").await?;
        Ok(sessao.game_data.queue.id)
    }

    /// Região da conta logada, p.ex. `BR`. Evita pedir isso ao usuário.
    pub async fn region(&self) -> Result<String> {
        let locale: RegionLocale = self.get_json("/riot/v1/region-locale").await?;
        Ok(locale.region)
    }
}
