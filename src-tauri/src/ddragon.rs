use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::time::Duration;

use presage_core::engine::champion::{BaseStats, Champion, ChampionIndex};
use presage_core::engine::items::{parse_stats, Item, ItemIndex};
use presage_core::engine::runes::{Rune, RuneIndex};

const BASE: &str = "https://ddragon.leagueoflegends.com";
const CHAMPION_CACHE: &str = "champions.json";
const ITEM_CACHE: &str = "items.json";
const RUNE_CACHE: &str = "runas.json";

/// Os atributos de item só existem na descrição, e os rótulos lá são a chave do
/// parser. Por isso lemos sempre em inglês e usamos o idioma do usuário apenas
/// para o nome exibido.
const STAT_LOCALE: &str = "en_US";

#[derive(Debug, Deserialize)]
struct ChampionPayload {
    data: HashMap<String, ChampionEntry>,
}

#[derive(Debug, Deserialize)]
struct ChampionEntry {
    id: String,
    key: String,
    name: String,
    #[serde(default)]
    tags: Vec<String>,
    info: Info,
    stats: ChampionStats,
}

#[derive(Debug, Deserialize)]
struct Info {
    attack: f64,
    defense: f64,
    magic: f64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
struct ChampionStats {
    hp: f64,
    hpperlevel: f64,
    armor: f64,
    armorperlevel: f64,
    spellblock: f64,
    spellblockperlevel: f64,
    attackdamage: f64,
    attackdamageperlevel: f64,
    attackrange: f64,
}

#[derive(Debug, Deserialize)]
struct ItemPayload {
    data: HashMap<String, ItemEntry>,
}

#[derive(Debug, Deserialize)]
struct ItemEntry {
    name: String,
    #[serde(default)]
    description: String,
    gold: Gold,
}

#[derive(Debug, Deserialize)]
struct Gold {
    total: f64,
    purchasable: bool,
}

/// `runesReforged.json` é uma lista de árvores, cada uma com fileiras de runas.
#[derive(Debug, Deserialize)]
struct RuneTree {
    #[serde(default)]
    key: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    slots: Vec<RuneSlot>,
}

#[derive(Debug, Deserialize)]
struct RuneSlot {
    #[serde(default)]
    runes: Vec<RuneEntry>,
}

#[derive(Debug, Deserialize)]
struct RuneEntry {
    id: i64,
    #[serde(default)]
    key: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    icon: String,
}

fn client() -> Result<reqwest::Client> {
    Ok(reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .build()?)
}

async fn latest_version(http: &reqwest::Client) -> Result<String> {
    let versions: Vec<String> = http
        .get(format!("{BASE}/api/versions.json"))
        .send()
        .await?
        .json()
        .await?;
    versions
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("lista de versões vazia"))
}

async fn payload<T: serde::de::DeserializeOwned>(
    http: &reqwest::Client,
    version: &str,
    locale: &str,
    file: &str,
) -> Result<T> {
    let url = format!("{BASE}/cdn/{version}/data/{locale}/{file}");
    let response = http.get(&url).send().await?;
    if !response.status().is_success() {
        return Err(anyhow!("{url} respondeu {}", response.status()));
    }
    Ok(response.json().await?)
}

pub async fn load_champions(locale: &str) -> Result<ChampionIndex> {
    match fetch_champions(locale).await {
        Ok(index) => {
            if let Err(err) = crate::storage::write_json(CHAMPION_CACHE, &index) {
                tracing::warn!(%err, "não consegui gravar o cache de campeões");
            }
            Ok(index)
        }
        Err(err) => {
            tracing::warn!(%err, "Data Dragon indisponível, tentando o cache de campeões");
            crate::storage::read_json(CHAMPION_CACHE).ok_or(err)
        }
    }
}

async fn fetch_champions(locale: &str) -> Result<ChampionIndex> {
    let http = client()?;
    let version = latest_version(&http).await?;

    let data: ChampionPayload = match payload(&http, &version, locale, "champion.json").await {
        Ok(payload) => payload,
        // Nem todo locale existe em todo patch.
        Err(_) => payload(&http, &version, STAT_LOCALE, "champion.json").await?,
    };

    let by_id: HashMap<i64, Champion> = data
        .data
        .into_values()
        .filter_map(|entry| {
            let id = entry.key.parse::<i64>().ok()?;
            Some((
                id,
                Champion {
                    id,
                    key: entry.id,
                    name: entry.name,
                    tags: entry.tags,
                    attack: entry.info.attack,
                    magic: entry.info.magic,
                    defense: entry.info.defense,
                    attack_range: entry.stats.attackrange,
                    base: BaseStats {
                        health: entry.stats.hp,
                        health_per_level: entry.stats.hpperlevel,
                        armor: entry.stats.armor,
                        armor_per_level: entry.stats.armorperlevel,
                        magic_resist: entry.stats.spellblock,
                        magic_resist_per_level: entry.stats.spellblockperlevel,
                        attack_damage: entry.stats.attackdamage,
                        attack_damage_per_level: entry.stats.attackdamageperlevel,
                    },
                },
            ))
        })
        .collect();

    if by_id.is_empty() {
        return Err(anyhow!("Data Dragon devolveu zero campeões"));
    }

    Ok(ChampionIndex { version, by_id })
}

pub async fn load_runes(locale: &str) -> Result<RuneIndex> {
    match fetch_runes(locale).await {
        Ok(index) => {
            if let Err(err) = crate::storage::write_json(RUNE_CACHE, &index) {
                tracing::warn!(%err, "não consegui gravar o cache de runas");
            }
            Ok(index)
        }
        Err(err) => {
            tracing::warn!(%err, "Data Dragon indisponível, tentando o cache de runas");
            crate::storage::read_json(RUNE_CACHE).ok_or(err)
        }
    }
}

async fn fetch_runes(locale: &str) -> Result<RuneIndex> {
    let http = client()?;
    let version = latest_version(&http).await?;

    // O motor procura as runas pela chave textual, que é a mesma em qualquer
    // idioma; o locale só muda o nome mostrado na tela.
    let trees: Vec<RuneTree> = match payload(&http, &version, locale, "runesReforged.json").await {
        Ok(trees) => trees,
        Err(_) => payload(&http, &version, STAT_LOCALE, "runesReforged.json").await?,
    };

    let mut by_key = HashMap::new();
    for tree in trees {
        for slot in tree.slots {
            for entry in slot.runes {
                if entry.key.is_empty() {
                    continue;
                }
                by_key.insert(
                    entry.key.clone(),
                    Rune {
                        id: entry.id,
                        key: entry.key,
                        name: entry.name,
                        icon: entry.icon,
                        tree_key: tree.key.clone(),
                        tree_name: tree.name.clone(),
                    },
                );
            }
        }
    }

    if by_key.is_empty() {
        return Err(anyhow!("Data Dragon devolveu zero runas"));
    }

    Ok(RuneIndex { version, by_key })
}

pub async fn load_items(locale: &str) -> Result<ItemIndex> {
    match fetch_items(locale).await {
        Ok(index) => {
            if let Err(err) = crate::storage::write_json(ITEM_CACHE, &index) {
                tracing::warn!(%err, "não consegui gravar o cache de itens");
            }
            Ok(index)
        }
        Err(err) => {
            tracing::warn!(%err, "Data Dragon indisponível, tentando o cache de itens");
            crate::storage::read_json(ITEM_CACHE).ok_or(err)
        }
    }
}

async fn fetch_items(locale: &str) -> Result<ItemIndex> {
    let http = client()?;
    let version = latest_version(&http).await?;

    let stats_source: ItemPayload = payload(&http, &version, STAT_LOCALE, "item.json").await?;

    // Nomes no idioma do usuário, quando esse idioma existe neste patch.
    let names: HashMap<String, String> = if locale == STAT_LOCALE {
        HashMap::new()
    } else {
        payload::<ItemPayload>(&http, &version, locale, "item.json")
            .await
            .map(|payload| {
                payload
                    .data
                    .into_iter()
                    .map(|(id, entry)| (id, entry.name))
                    .collect()
            })
            .unwrap_or_default()
    };

    let by_id: HashMap<i64, Item> = stats_source
        .data
        .into_iter()
        .filter_map(|(raw_id, entry)| {
            let id = raw_id.parse::<i64>().ok()?;
            Some((
                id,
                Item {
                    id,
                    name: names.get(&raw_id).cloned().unwrap_or(entry.name),
                    cost: entry.gold.total,
                    purchasable: entry.gold.purchasable,
                    stats: parse_stats(&entry.description),
                },
            ))
        })
        .collect();

    if by_id.is_empty() {
        return Err(anyhow!("Data Dragon devolveu zero itens"));
    }

    Ok(ItemIndex { version, by_id })
}
