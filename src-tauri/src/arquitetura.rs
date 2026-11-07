//! Testes de arquitetura da casca: o código que ela tem e o que ela não pode ter.
//!
//! Eles leem o próprio código-fonte, o `Cargo.lock` e as capabilities, porque o
//! que provam não é uma conta: é a ausência de algo. Nada de escrever no LCU
//! (ADR-0026), nada de falar com quem não é a Riot ou a própria máquina (C-2),
//! nada de ler memória nem de injetar input (C-2, C-3), autostart só quando o
//! jogador liga (ADR-0026), e cada janela só com o que precisa.

use std::path::{Path, PathBuf};

/// A pasta `src-tauri`. Fora do cargo da casca (o WSL compila estes testes à
/// parte, sem o Tauri), `PRESAGE_CASCA_RAIZ` aponta para ela.
fn raiz() -> PathBuf {
    option_env!("PRESAGE_CASCA_RAIZ")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")))
}

/// O texto de um arquivo, com fim de linha `\n`: no Windows o checkout vem com
/// `\r\n`, e o teste não pode depender disso.
fn ler(p: &Path) -> String {
    std::fs::read_to_string(p).unwrap().replace("\r\n", "\n")
}

/// Todo `.rs` da casca, menos este arquivo, que precisa citar o que proíbe.
fn fontes() -> Vec<(String, String)> {
    fn varrer(dir: &Path, out: &mut Vec<(String, String)>) {
        for entrada in std::fs::read_dir(dir).unwrap() {
            let p = entrada.unwrap().path();
            if p.is_dir() {
                varrer(&p, out);
            } else if p.extension().is_some_and(|e| e == "rs") && !p.ends_with("arquitetura.rs") {
                out.push((p.display().to_string(), ler(&p)));
            }
        }
    }
    let mut out = Vec::new();
    varrer(&raiz().join("src"), &mut out);
    assert!(out.len() >= 10, "achou só {} fontes da casca", out.len());
    out
}

#[test]
fn a_casca_nunca_escreve_no_lcu_nem_em_api_nenhuma() {
    // ADR-0026, R-18: o LCU é só lido. Vale para todo HTTP da casca: Live Client,
    // Riot Web API e Data Dragon também são só leitura.
    for (nome, texto) in fontes() {
        for verbo in [".post(", ".put(", ".patch(", ".delete(", "Method::POST", "Method::PUT", "Method::PATCH", "Method::DELETE"] {
            assert!(!texto.contains(verbo), "{nome} usa {verbo}");
        }
    }
}

#[test]
fn a_casca_so_fala_com_a_riot_e_com_a_propria_maquina() {
    // C-2: só APIs oficiais. Todo endereço de rede escrito num texto da casca é do
    // loopback ou de um domínio da Riot. Só conta o que está entre aspas, com
    // esquema de rede: `presage://` é nome de evento, e comentário não conecta.
    let mut hosts = Vec::new();
    for (nome, texto) in fontes() {
        for esquema in ["\"http://", "\"https://", "\"ws://", "\"wss://"] {
            for (i, _) in texto.match_indices(esquema) {
                let resto = &texto[i + esquema.len()..];
                let fim = resto
                    .find(|c: char| c == '/' || c == '"' || c == ':' || c.is_whitespace())
                    .unwrap_or(resto.len());
                // `{host}.api.riotgames.com{path}`: tira os marcadores do format!.
                let mut host = String::new();
                let mut dentro = false;
                for c in resto[..fim].chars() {
                    match c {
                        '{' => dentro = true,
                        '}' => dentro = false,
                        _ if !dentro => host.push(c),
                        _ => {}
                    }
                }
                hosts.push((nome.clone(), host));
            }
        }
    }
    assert!(hosts.iter().any(|(_, h)| h.ends_with("api.riotgames.com")), "{hosts:?}");
    for (nome, host) in hosts {
        let permitido = host == "127.0.0.1"
            || host == "localhost"
            || host.ends_with("riotgames.com")
            || host.ends_with("leagueoflegends.com");
        assert!(permitido, "{nome} fala com {host}");
    }
}

#[test]
fn nada_le_memoria_do_jogo_nem_injeta_input() {
    // C-2 e C-3: nenhuma chamada do Windows que leia a memória de outro processo
    // ou fabrique tecla e clique, e nenhuma dependência que faça isso.
    for (nome, texto) in fontes() {
        for api in [
            "ReadProcessMemory",
            "WriteProcessMemory",
            "VirtualAllocEx",
            "CreateRemoteThread",
            "SendInput",
            "SetWindowsHookEx",
            "keybd_event",
            "mouse_event",
        ] {
            assert!(!texto.contains(api), "{nome} usa {api}");
        }
    }
    let lock = ler(&raiz().join("..").join("Cargo.lock"));
    for cr in [
        "enigo",
        "rdev",
        "inputbot",
        "winput",
        "autopilot",
        "device_query",
        "read-process-memory",
        "process-memory",
        "memflow",
        "dll-syringe",
    ] {
        assert!(!lock.contains(&format!("name = \"{cr}\"")), "o Cargo.lock traz {cr}");
    }
}

#[test]
fn iniciar_com_o_windows_so_quando_o_jogador_liga() {
    // ADR-0026, R-19: o autostart só é ligado pelo comando que a tela chama
    // quando o jogador marca a caixa. Nada liga sozinho na abertura.
    let lib = ler(&raiz().join("src").join("lib.rs"));
    assert_eq!(lib.matches(".enable()").count(), 1, "quem mais liga o autostart?");
    let comando = lib.find("fn set_autostart").expect("set_autostart");
    let fim = lib[comando..].find("\n}\n").unwrap() + comando;
    assert!(lib[comando..fim].contains(".enable()"), "o enable() está fora do set_autostart");
}

#[test]
fn fechar_a_janela_principal_esconde_em_vez_de_destruir() {
    // A16: o ícone da bandeja traz a janela de volta. Para isso fechar a `main`
    // tem de escondê-la: destruída, "Mostrar o Presage" não acha nada.
    let lib = ler(&raiz().join("src").join("lib.rs"));
    let evento = lib.find("on_window_event").expect("sem on_window_event");
    let trecho = &lib[evento..(evento + 600).min(lib.len())];
    for parte in ["\"main\"", "CloseRequested", "prevent_close", ".hide()"] {
        assert!(trecho.contains(parte), "o fechar da janela principal não usa {parte}");
    }
}

#[test]
fn cada_janela_tem_so_o_que_precisa() {
    // A33: o overlay não atualiza nem reinicia o app; só a janela principal tem o
    // atualizador e o reinício.
    let mut por_janela: Vec<(String, Vec<String>)> = Vec::new();
    for entrada in std::fs::read_dir(raiz().join("capabilities")).unwrap() {
        let p = entrada.unwrap().path();
        if p.extension().map_or(true, |e| e != "json") {
            continue;
        }
        let texto = ler(&p);
        let cap: serde_json::Value = serde_json::from_str(&texto).unwrap();
        let janelas: Vec<String> = cap["windows"].as_array().unwrap().iter().map(|j| j.as_str().unwrap().to_string()).collect();
        let permissoes: Vec<String> = cap["permissions"]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x.as_str().map(str::to_string).unwrap_or_else(|| x["identifier"].as_str().unwrap_or("").to_string()))
            .collect();
        for j in janelas {
            por_janela.push((j, permissoes.clone()));
        }
    }
    let do_overlay: Vec<&String> = por_janela.iter().filter(|(j, _)| j == "overlay").flat_map(|(_, p)| p).collect();
    assert!(!do_overlay.is_empty(), "o overlay sem capability nenhuma não ouve eventos");
    for p in &do_overlay {
        assert!(!p.starts_with("updater:") && !p.starts_with("process:"), "o overlay tem {p}");
    }
    let da_main: Vec<&String> = por_janela.iter().filter(|(j, _)| j == "main").flat_map(|(_, p)| p).collect();
    assert!(da_main.iter().any(|p| p.starts_with("updater:")), "a janela principal perdeu o atualizador");
}

#[test]
fn o_overlay_arrasta_pela_cabeca_e_so_pega_clique_quando_mostra_algo() {
    // A15: Ajustes promete arrastar o overlay com Ctrl+Shift+O. Para isso a
    // cabeça é região de arrasto, e a capability do overlay deixa arrastar.
    let cap = ler(&raiz().join("capabilities").join("overlay.json"));
    assert!(cap.contains("\"core:window:allow-start-dragging\""), "o overlay não pode arrastar");
    let tela = ler(&raiz().join("..").join("src").join("overlay.tsx"));
    let cabeca = tela.find("className=\"hud-head\"").expect("sem a cabeça do overlay");
    let inicio = tela[..cabeca].rfind('<').unwrap();
    let fim = cabeca + tela[cabeca..].find('>').unwrap();
    assert!(tela[inicio..fim].contains("data-tauri-drag-region"), "a cabeça não é região de arrasto: {}", &tela[inicio..fim]);
    // Clicável, a janela inteira pega clique, desenhe ela algo ou não. Quem
    // decide é a regra do core, que olha a partida e as seções, e não só o atalho.
    let overlay = ler(&raiz().join("src").join("overlay.rs"));
    assert!(overlay.contains("atravessa_o_mouse("), "o clique do overlay não segue OverlayPrefs::atravessa_o_mouse");
    assert_eq!(overlay.matches("set_ignore_cursor_events(").count(), 1, "outro lugar muda o clique sem a regra");
}

/// Os nomes entre aspas de um trecho, na ordem: `"a", "b"` dá `[a, b]`.
fn entre_aspas(trecho: &str) -> Vec<String> {
    trecho.split('"').skip(1).step_by(2).map(str::to_string).collect()
}

/// As permissões `allow-…` da capability de uma janela, sem o prefixo.
fn liberados(janela: &str) -> Vec<String> {
    let arquivo = raiz().join("capabilities").join(format!("{janela}.json"));
    let cap: serde_json::Value = serde_json::from_str(&ler(&arquivo)).unwrap();
    let mut out: Vec<String> = cap["permissions"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|p| p.as_str())
        .filter_map(|p| p.strip_prefix("allow-"))
        .map(|p| p.replace('-', "_"))
        .collect();
    out.sort();
    out
}

#[test]
fn cada_janela_so_alcanca_os_comandos_que_usa() {
    // A52: sem manifesto de comandos, as duas janelas alcançavam os 26. Com ele,
    // cada comando registrado tem a sua permissão, a janela principal libera
    // todos e o overlay só os que ele chama.
    let lib = ler(&raiz().join("src").join("lib.rs"));
    let i = lib.find("generate_handler![").expect("generate_handler");
    let mut registrados: Vec<String> = lib[i + "generate_handler![".len()..lib[i..].find(']').unwrap() + i]
        .split(',')
        .map(|c| c.trim().to_string())
        .filter(|c| !c.is_empty())
        .collect();
    registrados.sort();
    assert!(registrados.len() >= 20, "{registrados:?}");

    let build = ler(&raiz().join("build.rs"));
    let m = build.find("COMANDOS: &[&str] = &[").expect("o build.rs não tem o manifesto de comandos");
    let mut manifesto = entre_aspas(&build[m..m + build[m..].find("];").unwrap()]);
    manifesto.sort();
    assert!(build.contains(".commands(COMANDOS)"), "o manifesto não vai para o tauri-build");
    assert_eq!(manifesto, registrados, "comando registrado sem permissão, ou permissão de comando que não existe");

    assert_eq!(liberados("main"), registrados, "a janela principal não alcança todos os comandos");

    let tela = ler(&raiz().join("..").join("src").join("overlay.tsx"));
    let mut chamados: Vec<String> = tela
        .match_indices("invoke")
        .filter_map(|(i, _)| {
            let resto = &tela[i..];
            let aspas = resto.find("(\"")?;
            (aspas < 80).then(|| resto[aspas + 2..aspas + 2 + resto[aspas + 2..].find('"').unwrap()].to_string())
        })
        .collect();
    chamados.sort();
    chamados.dedup();
    assert!(chamados.len() >= 3, "{chamados:?}");
    assert_eq!(liberados("overlay"), chamados, "o overlay alcança mais, ou menos, do que chama");
}
