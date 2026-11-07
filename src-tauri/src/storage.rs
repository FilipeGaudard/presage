use anyhow::{anyhow, Result};
use serde::{de::DeserializeOwned, Serialize};
use std::path::{Path, PathBuf};

pub fn cache_dir() -> Option<PathBuf> {
    let root = std::env::var("LOCALAPPDATA").ok()?;
    Some(PathBuf::from(root).join("Presage"))
}

pub fn write_json<T: Serialize>(name: &str, value: &T) -> Result<()> {
    let dir = cache_dir().ok_or_else(|| anyhow!("LOCALAPPDATA indisponível"))?;
    write_json_in(&dir, name, value)
}

/// Grava `value` em `dir/name`.
fn write_json_in<T: Serialize>(dir: &Path, name: &str, value: &T) -> Result<()> {
    std::fs::create_dir_all(dir)?;
    std::fs::write(dir.join(name), serde_json::to_vec(value)?)?;
    Ok(())
}

pub fn read_json<T: DeserializeOwned>(name: &str) -> Option<T> {
    let raw = std::fs::read(cache_dir()?.join(name)).ok()?;
    serde_json::from_slice(&raw).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pasta(nome: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("presage-storage-{nome}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn ler(dir: &Path) -> Vec<i32> {
        serde_json::from_slice(&std::fs::read(dir.join("pool.json")).unwrap()).unwrap()
    }

    #[test]
    fn escrita_que_morre_no_meio_deixa_o_arquivo_anterior_inteiro() {
        let dir = pasta("falha");
        write_json_in(&dir, "pool.json", &vec![1, 2, 3]).unwrap();

        // O temporário não pode ser criado, porque no lugar dele há uma pasta:
        // é a escrita que morre antes de terminar (disco cheio, energia que cai).
        std::fs::create_dir_all(dir.join("pool.json.parcial")).unwrap();
        let r = write_json_in(&dir, "pool.json", &vec![9, 9]);

        assert!(r.is_err(), "a escrita que não terminou volta como erro");
        assert_eq!(ler(&dir), vec![1, 2, 3], "o arquivo anterior continua inteiro (A7)");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn escrita_que_termina_troca_o_arquivo_e_nao_deixa_resto() {
        let dir = pasta("ok");
        write_json_in(&dir, "pool.json", &vec![1]).unwrap();
        write_json_in(&dir, "pool.json", &vec![2]).unwrap();

        assert_eq!(ler(&dir), vec![2]);
        assert!(!dir.join("pool.json.parcial").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
