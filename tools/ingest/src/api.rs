//! Cliente da Riot API para o coletor.
//!
//! Três coisas que não são detalhe:
//!
//! 1. **A chave vai no cabeçalho `X-Riot-Token`**, nunca na URL. URL aparece em
//!    log de proxy, em histórico e em mensagem de erro.
//! 2. **Dois limites separados.** `league-v4` fala com o host de plataforma
//!    (`br1.api.riotgames.com`) e `match-v5` com o regional
//!    (`americas.api.riotgames.com`). A Riot conta as janelas por host, então
//!    somar os dois num limitador só desperdiça metade da cota.
//! 3. **429 é aviso, não erro.** Quem insiste depois de um 429 perde a chave.
//!    Aqui ele para o limitador inteiro pelo tempo que o servidor pediu.

use std::sync::Mutex;
use std::time::{Duration, Instant};

use anyhow::{anyhow, bail, Context, Result};
use serde::de::DeserializeOwned;
use serde::Deserialize;

use crate::aggregate::Match;
use crate::limiter::Limiter;

/// Tentativas por chamada antes de desistir dela.
const MAX_TRIES: u32 = 5;

/// Quando o 429 vem sem `Retry-After`.
const FALLBACK_PENALTY: f64 = 10.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Host {
    /// `br1.api.riotgames.com` — league-v4.
    Platform,
    /// `americas.api.riotgames.com` — match-v5.
    Regional,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct LeagueEntry {
    pub puuid: String,
}

/// As ligas de topo (mestre e acima) vêm numa lista única, não paginadas por
/// divisão.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct LeagueList {
    pub entries: Vec<LeagueEntry>,
}

pub struct Riot {
    http: reqwest::Client,
    key: String,
    platform: String,
    regional: String,
    limite_plataforma: Mutex<Limiter>,
    limite_regional: Mutex<Limiter>,
    start: Instant,
}

impl Riot {
    pub fn new(platform: &str, regional: &str, key: String, limite: &str) -> Result<Self> {
        let inicial = Limiter::parse(limite)
            .ok_or_else(|| anyhow!("limite inválido: {limite:?} (esperado algo como 20:1,100:120)"))?;

        Ok(Riot {
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(20))
                .user_agent("Presage-ingest/0.1")
                .build()
                .context("montando o cliente HTTP")?,
            key,
            platform: format!("https://{platform}.api.riotgames.com"),
            regional: format!("https://{regional}.api.riotgames.com"),
            limite_plataforma: Mutex::new(inicial.clone()),
            limite_regional: Mutex::new(inicial),
            start: Instant::now(),
        })
    }

    fn now(&self) -> f64 {
        self.start.elapsed().as_secs_f64()
    }

    fn limiter(&self, host: Host) -> &Mutex<Limiter> {
        match host {
            Host::Platform => &self.limite_plataforma,
            Host::Regional => &self.limite_regional,
        }
    }

    fn base(&self, host: Host) -> &str {
        match host {
            Host::Platform => &self.platform,
            Host::Regional => &self.regional,
        }
    }

    /// Espera a vez. O cadeado nunca é mantido através de um `await`.
    async fn gate(&self, host: Host) {
        loop {
            let espera = {
                let mut limiter = self.limiter(host).lock().unwrap();
                let agora = self.now();
                let espera = limiter.wait(agora);
                if espera <= 0.0 {
                    limiter.record(agora);
                    return;
                }
                espera
            };
            tokio::time::sleep(Duration::from_secs_f64(espera.min(60.0))).await;
        }
    }

    /// `None` quando a Riot responde 404 — recurso que não existe não é falha.
    async fn get<T: DeserializeOwned>(&self, host: Host, path: &str) -> Result<Option<T>> {
        let url = format!("{}{path}", self.base(host));

        for tentativa in 1..=MAX_TRIES {
            self.gate(host).await;

            let resposta = self
                .http
                .get(&url)
                .header("X-Riot-Token", &self.key)
                .send()
                .await;

            let resposta = match resposta {
                Ok(r) => r,
                Err(err) if tentativa < MAX_TRIES => {
                    tracing::debug!(%err, tentativa, "rede falhou, tentando de novo");
                    tokio::time::sleep(backoff(tentativa)).await;
                    continue;
                }
                Err(err) => return Err(err).context(format!("GET {path}")),
            };

            let status = resposta.status();

            // O servidor declara o limite real dele. Adotar isso é melhor que
            // qualquer número que a gente chute na linha de comando.
            if let Some(declarado) = resposta
                .headers()
                .get("X-App-Rate-Limit")
                .and_then(|v| v.to_str().ok())
                .and_then(Limiter::parse)
            {
                self.limiter(host).lock().unwrap().adopt(declarado);
            }

            if status == reqwest::StatusCode::NOT_FOUND {
                return Ok(None);
            }

            if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                let segundos = resposta
                    .headers()
                    .get("Retry-After")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse::<f64>().ok())
                    .unwrap_or(FALLBACK_PENALTY);
                let tipo = resposta
                    .headers()
                    .get("X-Rate-Limit-Type")
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or("?")
                    .to_string();

                tracing::warn!(segundos, tipo, "429: parando até liberar");
                self.limiter(host)
                    .lock()
                    .unwrap()
                    .penalize(self.now(), segundos);
                continue;
            }

            if status == reqwest::StatusCode::UNAUTHORIZED
                || status == reqwest::StatusCode::FORBIDDEN
            {
                // Não adianta tentar de novo: ou a chave está errada, ou
                // expirou. Chave de desenvolvedor expira a cada 24 horas.
                bail!(
                    "a Riot recusou a chave ({status}). \
                     Confira PRESAGE_RIOT_API_KEY — a chave de desenvolvedor expira a cada 24h."
                );
            }

            if status.is_server_error() && tentativa < MAX_TRIES {
                tracing::debug!(%status, tentativa, "erro no servidor, tentando de novo");
                tokio::time::sleep(backoff(tentativa)).await;
                continue;
            }

            if !status.is_success() {
                bail!("GET {path} respondeu {status}");
            }

            return resposta
                .json::<T>()
                .await
                .map(Some)
                .context(format!("lendo a resposta de {path}"));
        }

        bail!("GET {path} falhou em {MAX_TRIES} tentativas")
    }

    /// Uma página de jogadores de uma divisão. Página vazia = acabou.
    pub async fn league_entries(
        &self,
        queue: &str,
        tier: &str,
        division: &str,
        page: u32,
    ) -> Result<Vec<LeagueEntry>> {
        let path = format!("/lol/league/v4/entries/{queue}/{tier}/{division}?page={page}");
        Ok(self.get(Host::Platform, &path).await?.unwrap_or_default())
    }

    /// Mestre, grão-mestre e desafiante não têm divisão: vêm numa lista só.
    pub async fn apex_league(&self, tier: &str, queue: &str) -> Result<Vec<LeagueEntry>> {
        let endpoint = match tier {
            "CHALLENGER" => "challengerleagues",
            "GRANDMASTER" => "grandmasterleagues",
            "MASTER" => "masterleagues",
            outro => bail!("{outro} não é uma liga de topo"),
        };
        let path = format!("/lol/league/v4/{endpoint}/by-queue/{queue}");
        let lista: Option<LeagueList> = self.get(Host::Platform, &path).await?;
        Ok(lista.unwrap_or_default().entries)
    }

    pub async fn match_ids(&self, puuid: &str, queue: i64, count: u32) -> Result<Vec<String>> {
        let path = format!(
            "/lol/match/v5/matches/by-puuid/{puuid}/ids?queue={queue}&type=ranked&start=0&count={count}"
        );
        Ok(self.get(Host::Regional, &path).await?.unwrap_or_default())
    }

    pub async fn match_detail(&self, id: &str) -> Result<Option<Match>> {
        self.get(Host::Regional, &format!("/lol/match/v5/matches/{id}"))
            .await
    }
}

/// 1s, 2s, 4s, 8s.
fn backoff(tentativa: u32) -> Duration {
    Duration::from_secs(1u64 << tentativa.min(4).saturating_sub(1))
}

/// As divisões de cada tier, na ordem em que o coletor as percorre.
pub const DIVISIONS: [&str; 4] = ["I", "II", "III", "IV"];

/// Verdadeiro para os tiers que vêm em lista única, sem divisão.
pub fn is_apex(tier: &str) -> bool {
    matches!(tier, "MASTER" | "GRANDMASTER" | "CHALLENGER")
}

pub const TIERS: [&str; 10] = [
    "IRON",
    "BRONZE",
    "SILVER",
    "GOLD",
    "PLATINUM",
    "EMERALD",
    "DIAMOND",
    "MASTER",
    "GRANDMASTER",
    "CHALLENGER",
];

/// Os tiers de `menor` para cima, como em `EMERALD+`.
pub fn tiers_from(menor: &str) -> Option<Vec<&'static str>> {
    let menor = menor.to_ascii_uppercase();
    let inicio = TIERS.iter().position(|t| *t == menor)?;
    Some(TIERS[inicio..].to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn o_backoff_dobra_e_para_de_dobrar() {
        assert_eq!(backoff(1), Duration::from_secs(1));
        assert_eq!(backoff(2), Duration::from_secs(2));
        assert_eq!(backoff(3), Duration::from_secs(4));
        assert_eq!(backoff(4), Duration::from_secs(8));
        assert_eq!(backoff(9), Duration::from_secs(8));
    }

    #[test]
    fn as_ligas_de_topo_sao_reconhecidas() {
        assert!(is_apex("CHALLENGER"));
        assert!(is_apex("MASTER"));
        assert!(!is_apex("EMERALD"));
        assert!(!is_apex("DIAMOND"));
    }

    #[test]
    fn tier_minimo_inclui_tudo_acima() {
        let de_esmeralda = tiers_from("emerald").unwrap();
        assert_eq!(de_esmeralda[0], "EMERALD");
        assert_eq!(de_esmeralda.last(), Some(&"CHALLENGER"));
        assert_eq!(de_esmeralda.len(), 5);
    }

    #[test]
    fn tier_desconhecido_nao_vira_lista_vazia_silenciosa() {
        assert!(tiers_from("MADEIRA").is_none());
    }

    #[test]
    fn o_tier_mais_baixo_pega_todos() {
        assert_eq!(tiers_from("IRON").unwrap().len(), TIERS.len());
    }

    #[test]
    fn a_chave_nunca_entra_na_url() {
        let riot = Riot::new("br1", "americas", "RGAPI-secreta".into(), "20:1").unwrap();
        assert!(!riot.base(Host::Platform).contains("RGAPI"));
        assert!(!riot.base(Host::Regional).contains("RGAPI"));
    }

    #[test]
    fn plataforma_e_regional_tem_hosts_diferentes() {
        let riot = Riot::new("br1", "americas", "x".into(), "20:1").unwrap();
        assert_eq!(riot.base(Host::Platform), "https://br1.api.riotgames.com");
        assert_eq!(
            riot.base(Host::Regional),
            "https://americas.api.riotgames.com"
        );
    }

    #[test]
    fn limite_invalido_nao_passa_na_construcao() {
        // Melhor falhar na partida do que descobrir no meio do crawl que o
        // coletor estava sem limite nenhum.
        assert!(Riot::new("br1", "americas", "x".into(), "nada").is_err());
    }
}
