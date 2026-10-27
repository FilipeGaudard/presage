//! Descoberta das credenciais do cliente em execução.
//!
//! O parsing mora em `presage_core::lockfile`; aqui fica só o que toca o
//! sistema.
//!
//! # Por que isto não lança PowerShell
//!
//! A primeira versão perguntava ao WMI a linha de comando do `LeagueClientUx`
//! rodando `powershell -NoProfile -NonInteractive -Command "Get-CimInstance
//! Win32_Process ..."` com janela oculta. Funcionava, e o Kaspersky marcou o
//! binário como `VHO:Trojan.Win32.Convagent.gen`.
//!
//! A detecção foi heurística, mas não foi injusta: um executável compilado que
//! lança um shell escondido para interrogar outro processo é exatamente o que
//! um ladrão de credenciais faz. Não dá para pedir ao antivírus que confie na
//! nossa intenção.
//!
//! Então a ordem de busca aqui vai do mais silencioso para o mais invasivo:
//!
//! 1. **Caminhos conhecidos** — leitura de arquivo, sinal zero.
//! 2. **Configuração da própria Riot** — leitura de arquivo, sinal zero.
//! 3. **Caminho do executável do processo** — só se os dois primeiros
//!    falharem, e via API que lê o caminho da imagem, nunca a memória dele.
//!
//! Na prática os dois primeiros resolvem quase sempre, e o terceiro quase
//! nunca roda.

use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};

use presage_core::lockfile::{self, Credentials};

const LOCKFILE: &str = "lockfile";

/// Onde o League costuma ficar.
const KNOWN_DIRS: &[&str] = &[
    r"C:\Riot Games\League of Legends",
    r"C:\Program Files\Riot Games\League of Legends",
    r"C:\Program Files (x86)\Riot Games\League of Legends",
];

pub fn discover() -> Result<Credentials> {
    for dir in candidate_dirs() {
        let caminho = dir.join(LOCKFILE);
        let Ok(conteudo) = std::fs::read_to_string(&caminho) else {
            continue;
        };
        if let Some(creds) = lockfile::parse_lockfile(&conteudo) {
            tracing::debug!(port = creds.port, pasta = %dir.display(), "credenciais lidas");
            return Ok(creds);
        }
    }

    Err(anyhow!(
        "cliente do League of Legends não encontrado (LeagueClientUx.exe não está rodando?)"
    ))
}

/// Pastas onde pode haver um lockfile, da busca mais barata para a mais cara.
fn candidate_dirs() -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = KNOWN_DIRS.iter().map(PathBuf::from).collect();
    dirs.extend(dirs_from_riot_config());

    // Só vale enumerar processos se nada acima deu lockfile legível.
    if !dirs.iter().any(|d| d.join(LOCKFILE).is_file()) {
        dirs.extend(dir_from_running_client());
    }

    dirs.sort();
    dirs.dedup();
    dirs
}

/// Pastas citadas nos arquivos de configuração do Riot Client.
///
/// Lê sem conhecer o esquema: varre o texto atrás de qualquer caminho do
/// Windows e testa cada um. O formato desses arquivos já mudou mais de uma vez
/// e vai mudar de novo; depender de nome de campo seria quebrar na próxima.
fn dirs_from_riot_config() -> Vec<PathBuf> {
    let mut arquivos = Vec::new();

    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        arquivos.push(
            PathBuf::from(&local)
                .join(r"Riot Games\Riot Client\Config\LolClientInstalls.json"),
        );
    }
    if let Ok(todos) = std::env::var("ALLUSERSPROFILE") {
        arquivos.push(PathBuf::from(&todos).join(
            r"Riot Games\Metadata\league_of_legends.live\league_of_legends.live.product_settings.yaml",
        ));
    }

    arquivos
        .into_iter()
        .filter_map(|arquivo| std::fs::read_to_string(arquivo).ok())
        .flat_map(|texto| paths_in(&texto))
        .collect()
}

/// Todo caminho absoluto do Windows que aparece no texto, mais a subpasta
/// `League of Legends` de cada um — a configuração às vezes guarda a pasta do
/// Riot Client, com o jogo um nível abaixo.
fn paths_in(texto: &str) -> Vec<PathBuf> {
    // Varre atrás de `X:\` em vez de fatiar o texto por separadores: o
    // dois-pontos que separa chave de valor no YAML é o mesmo que vem depois
    // da letra do drive, e fatiar por ele decapita todo caminho.
    let bytes = texto.as_bytes();
    let mut saida = Vec::new();
    let mut i = 0;

    while i + 2 < bytes.len() {
        let comeca_caminho = bytes[i].is_ascii_alphabetic()
            && bytes[i + 1] == b':'
            && bytes[i + 2] == b'\\'
            // Não pode estar no meio de uma palavra.
            && (i == 0 || !bytes[i - 1].is_ascii_alphanumeric());

        if !comeca_caminho {
            i += 1;
            continue;
        }

        let mut fim = i;
        while fim < bytes.len()
            && !matches!(bytes[fim], b'"' | b'\'' | b'\n' | b'\r' | b',')
        {
            fim += 1;
        }

        // `get` em vez de fatiar direto: um acento no caminho faria o índice
        // cair no meio de um caractere e derrubar o app.
        if let Some(bruto) = texto.get(i..fim) {
            let limpo = bruto.replace("\\\\", "\\");
            let limpo = limpo.trim().trim_end_matches('\\');
            if limpo.len() >= 3 {
                let caminho = PathBuf::from(limpo);
                saida.push(caminho.join("League of Legends"));
                saida.push(caminho);
            }
        }

        i = fim.max(i + 1);
    }

    saida
}

/// A pasta do `LeagueClientUx.exe` que está rodando.
///
/// Usa só o caminho da imagem do processo, que é informação pública do
/// sistema — nunca a linha de comando nem a memória dele, que é onde mora o
/// token e é o que faz um antivírus reagir.
fn dir_from_running_client() -> Option<PathBuf> {
    use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, RefreshKind, System, UpdateKind};

    let mut sistema = System::new_with_specifics(
        RefreshKind::nothing()
            .with_processes(ProcessRefreshKind::nothing().with_exe(UpdateKind::Always)),
    );
    sistema.refresh_processes(ProcessesToUpdate::All, true);

    sistema
        .processes()
        .values()
        .find(|processo| {
            processo
                .name()
                .to_str()
                .is_some_and(|nome| nome.eq_ignore_ascii_case("LeagueClientUx.exe"))
        })
        .and_then(|processo| processo.exe())
        .and_then(Path::parent)
        .map(Path::to_path_buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn acha_caminhos_num_json_com_barras_escapadas() {
        let json = r#"{"associated_client":{"C:\\Riot Games\\League of Legends\\":"x"}}"#;
        let achados = paths_in(json);

        assert!(
            achados.contains(&PathBuf::from(r"C:\Riot Games\League of Legends")),
            "{achados:?}"
        );
    }

    #[test]
    fn acrescenta_a_subpasta_do_jogo() {
        // A configuração às vezes guarda a pasta do Riot Client, com o jogo
        // um nível abaixo.
        let achados = paths_in(r#""C:\\Riot Games""#);
        assert!(achados.contains(&PathBuf::from(r"C:\Riot Games\League of Legends")));
        assert!(achados.contains(&PathBuf::from(r"C:\Riot Games")));
    }

    #[test]
    fn acha_caminho_em_yaml_sem_aspas() {
        let yaml = "product_install_full_path: D:\\Jogos\\League of Legends\noutra: coisa";
        let achados = paths_in(yaml);
        assert!(
            achados.contains(&PathBuf::from(r"D:\Jogos\League of Legends")),
            "{achados:?}"
        );
    }

    #[test]
    fn texto_sem_caminho_nao_rende_nada() {
        assert!(paths_in("{}").is_empty());
        assert!(paths_in("").is_empty());
        assert!(paths_in("nada de util aqui").is_empty());
    }

    #[test]
    fn nao_confunde_url_com_caminho() {
        // Um `https://...` tem dois-pontos mas não é caminho do Windows.
        let achados = paths_in(r#""https://riotgames.com/algo""#);
        assert!(achados.is_empty(), "{achados:?}");
    }

    #[test]
    fn os_caminhos_conhecidos_entram_sempre() {
        let dirs = candidate_dirs();
        for conhecido in KNOWN_DIRS {
            assert!(
                dirs.contains(&PathBuf::from(conhecido)),
                "faltou {conhecido}"
            );
        }
    }

    #[test]
    fn sem_cliente_rodando_a_descoberta_falha_com_mensagem_util() {
        // Nesta máquina o League não está instalado. O erro tem de dizer o
        // que fazer, não "None".
        if KNOWN_DIRS.iter().any(|d| Path::new(d).join(LOCKFILE).is_file()) {
            return; // há um cliente de verdade aqui; o teste não se aplica
        }
        let erro = discover().unwrap_err().to_string();
        assert!(erro.contains("League of Legends"), "{erro}");
    }
}
