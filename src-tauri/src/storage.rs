use anyhow::{anyhow, Result};
use serde::{de::DeserializeOwned, Serialize};
use std::path::PathBuf;

pub fn cache_dir() -> Option<PathBuf> {
    let root = std::env::var("LOCALAPPDATA").ok()?;
    Some(PathBuf::from(root).join("Presage"))
}

pub fn write_json<T: Serialize>(name: &str, value: &T) -> Result<()> {
    let dir = cache_dir().ok_or_else(|| anyhow!("LOCALAPPDATA indisponível"))?;
    std::fs::create_dir_all(&dir)?;
    std::fs::write(dir.join(name), serde_json::to_vec(value)?)?;
    Ok(())
}

pub fn read_json<T: DeserializeOwned>(name: &str) -> Option<T> {
    let raw = std::fs::read(cache_dir()?.join(name)).ok()?;
    serde_json::from_slice(&raw).ok()
}
