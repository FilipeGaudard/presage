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

/// Grava `value` em `dir/name` sem nunca deixar o arquivo pela metade: escreve
/// tudo em `name.parcial` e só então troca pelo nome final. Uma escrita que morre
/// no meio (disco cheio, energia que cai) deixa o arquivo anterior inteiro, e a
/// pool não volta vazia na abertura seguinte (A7). É a mesma disciplina do
/// coletor (`tools/ingest`).
fn write_json_in<T: Serialize>(dir: &Path, name: &str, value: &T) -> Result<()> {
    std::fs::create_dir_all(dir)?;
    let parcial = dir.join(format!("{name}.parcial"));
    std::fs::write(&parcial, serde_json::to_vec(value)?)?;
    std::fs::rename(&parcial, dir.join(name))?;
    Ok(())
}

/// O arquivo de registro do app, na pasta dos outros arquivos dele.
pub const LOG_FILE: &str = "presage.log";

/// Abre o registro do zero: cada abertura do app começa um arquivo novo, e ele
/// não cresce para sempre (A14).
pub fn abrir_log() -> Option<std::fs::File> {
    abrir_log_em(&cache_dir()?)
}

fn abrir_log_em(dir: &Path) -> Option<std::fs::File> {
    std::fs::create_dir_all(dir).ok()?;
    std::fs::File::create(dir.join(LOG_FILE)).ok()
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
    fn o_registro_comeca_do_zero_a_cada_abertura() {
        let dir = pasta("log");
        std::fs::write(dir.join(LOG_FILE), "da abertura anterior").unwrap();
        let arquivo = abrir_log_em(&dir).expect("o registro abre");
        assert_eq!(arquivo.metadata().unwrap().len(), 0);
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
