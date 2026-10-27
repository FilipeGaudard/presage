//! Parsing das credenciais do cliente. Só texto — a parte que vai ao disco e
//! aos processos fica em src-tauri.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Credentials {
    pub port: u16,
    pub token: String,
}

impl Credentials {
    pub fn base_url(&self) -> String {
        format!("https://127.0.0.1:{}", self.port)
    }
}

/// `LeagueClient:<pid>:<porta>:<senha>:https`
pub fn parse_lockfile(contents: &str) -> Option<Credentials> {
    let parts: Vec<&str> = contents.trim().split(':').collect();
    if parts.len() < 4 {
        return None;
    }
    Some(Credentials {
        port: parts[2].parse().ok()?,
        token: parts[3].to_string(),
    })
}

pub fn parse_cmdline(cmdline: &str) -> Option<Credentials> {
    Some(Credentials {
        port: extract_arg(cmdline, "--app-port=")?.parse().ok()?,
        token: extract_arg(cmdline, "--remoting-auth-token=")?,
    })
}

/// Extrai o valor de `--chave=valor`. A linha de comando do cliente mistura
/// argumentos com e sem aspas.
pub fn extract_arg(haystack: &str, key: &str) -> Option<String> {
    let start = haystack.find(key)? + key.len();
    let value: String = haystack[start..]
        .chars()
        .take_while(|c| *c != '"' && !c.is_whitespace())
        .collect();
    (!value.is_empty()).then_some(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lockfile_valido() {
        let creds = parse_lockfile("LeagueClient:21360:54321:aBcD_eFg123:https").unwrap();
        assert_eq!(creds.port, 54321);
        assert_eq!(creds.token, "aBcD_eFg123");
    }

    #[test]
    fn lockfile_truncado_nao_entra_em_panico() {
        assert!(parse_lockfile("LeagueClient:21360").is_none());
    }

    #[test]
    fn lockfile_com_porta_invalida() {
        assert!(parse_lockfile("LeagueClient:21360:abc:senha:https").is_none());
    }

    #[test]
    fn cmdline_com_aspas() {
        let line = r#"LeagueClientUx.exe "--app-port=50123" "--remoting-auth-token=AbC_dEf-123""#;
        let creds = parse_cmdline(line).unwrap();
        assert_eq!(creds.port, 50123);
        assert_eq!(creds.token, "AbC_dEf-123");
    }

    #[test]
    fn cmdline_sem_aspas() {
        let line = "LeagueClientUx.exe --app-port=50123 --remoting-auth-token=AbC_dEf --locale=pt_BR";
        let creds = parse_cmdline(line).unwrap();
        assert_eq!(creds.port, 50123);
        assert_eq!(creds.token, "AbC_dEf");
    }

    #[test]
    fn cmdline_sem_token_falha() {
        assert!(parse_cmdline("LeagueClientUx.exe --app-port=50123").is_none());
    }

    #[test]
    fn base_url_usa_loopback() {
        let creds = Credentials {
            port: 1234,
            token: "x".into(),
        };
        assert_eq!(creds.base_url(), "https://127.0.0.1:1234");
    }
}
