//! Histórico do próprio jogador.
//!
//! Alimenta só o componente pessoal do motor. Winrate base, matchup e sinergia
//! continuam vazios de propósito: os três precisam de muitas partidas de muitos
//! jogadores, e uma conta sozinha não produz nenhum deles.

use anyhow::{anyhow, Result};
use std::sync::Arc;
use std::time::Duration;

use presage_core::engine::stats::PersonalStats;
use presage_core::history;

use crate::riot::{self, Region, RiotClient};
use crate::storage;
use crate::watcher::{Identity, Shared};

/// O `personal.json` antigo marca partidas como vistas sem a comparação de
/// rota; reaproveitá-lo deixaria essas partidas sem rota para sempre. Arquivo
/// novo, e a primeira atualização refaz o histórico.
const CACHE: &str = "personal-v2.json";

/// Teto por atualização. Com o espaçamento entre chamadas isso leva ~80s na
/// primeira vez; depois só as partidas novas são buscadas.
const MATCHES_PER_REFRESH: u8 = 60;

/// Quanto esperamos o cliente da Riot abrir antes de cair para a variável de
/// ambiente.
const IDENTITY_TIMEOUT: Duration = Duration::from_secs(120);
const IDENTITY_POLL: Duration = Duration::from_secs(2);

pub async fn refresh(shared: Arc<Shared>) -> Result<PersonalStats> {
    if std::env::var(riot::ENV_API_KEY).is_err() {
        return Err(anyhow!(
            "{} não definida — componente pessoal fica desligado",
            riot::ENV_API_KEY
        ));
    }

    let identity = resolve_identity(&shared)
        .await
        .ok_or_else(|| anyhow!("não consegui identificar a conta"))?;

    let region = Region::from_lcu(&identity.region);
    let client = RiotClient::from_env(region).ok_or_else(|| anyhow!("chave da Riot vazia"))?;

    let mut stats: PersonalStats = storage::read_json(CACHE).unwrap_or_default();
    if stats.puuid != identity.puuid {
        stats = PersonalStats {
            puuid: identity.puuid.clone(),
            ..Default::default()
        };
    }

    let ids = client
        .ranked_match_ids(&identity.puuid, MATCHES_PER_REFRESH)
        .await?;

    let pending: Vec<String> = ids
        .into_iter()
        .filter(|id| !stats.seen.iter().any(|seen| seen == id))
        .collect();

    tracing::info!(
        conta = %identity.riot_id,
        ja_tinha = stats.total_games(),
        novas = pending.len(),
        "atualizando histórico pessoal"
    );

    let mut added = 0;
    let mut skipped = 0;
    for id in &pending {
        match client.match_detail(id).await {
            Ok(detail) => match history::played(&detail, &identity.puuid) {
                Ok(game) => {
                    if stats.add_played(id, game) {
                        added += 1;
                    }
                }
                Err(skip) => {
                    // Remake, curta, fora da Fenda: não muda com o tempo, então
                    // fica marcada para não ser buscada de novo.
                    tracing::debug!(partida = %id, ?skip, "partida fora da conta");
                    stats.mark_seen(id);
                    skipped += 1;
                }
            },
            Err(err) => {
                // Chave expirada ou rate limit teimoso: para e guarda o que deu.
                tracing::warn!(%err, "parando a atualização do histórico");
                break;
            }
        }
    }

    if added > 0 || skipped > 0 {
        if let Err(err) = storage::write_json(CACHE, &stats) {
            tracing::warn!(%err, "não consegui gravar o cache do histórico");
        }
    }

    Ok(stats)
}

async fn resolve_identity(shared: &Shared) -> Option<Identity> {
    let deadline = tokio::time::Instant::now() + IDENTITY_TIMEOUT;

    while tokio::time::Instant::now() < deadline {
        let current = shared.identity.lock().unwrap().clone();
        if current.is_some() {
            return current;
        }
        tokio::time::sleep(IDENTITY_POLL).await;
    }

    identity_from_env().await
}

/// Caminho alternativo para quando o cliente não está aberto: o usuário define
/// `PRESAGE_RIOT_ID` como `nome#tag` e a conta é resolvida pela Riot API.
async fn identity_from_env() -> Option<Identity> {
    let riot_id = std::env::var(riot::ENV_RIOT_ID).ok()?;
    let (game_name, tag_line) = riot::split_riot_id(&riot_id)?;

    let region_name = std::env::var("PRESAGE_REGION").unwrap_or_else(|_| "BR".into());
    let client = RiotClient::from_env(Region::from_lcu(&region_name))?;

    match client.account_by_riot_id(&game_name, &tag_line).await {
        Ok(account) => Some(Identity {
            puuid: account.puuid,
            riot_id,
            region: region_name,
        }),
        Err(err) => {
            tracing::warn!(%err, "não consegui resolver o Riot ID da variável de ambiente");
            None
        }
    }
}
