//! Endereçamento da Riot API. Só texto; as chamadas ficam em src-tauri.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Region {
    /// Host por plataforma, p.ex. `br1`. Usado por summoner e mastery.
    pub platform: &'static str,
    /// Host regional, p.ex. `americas`. Usado por account e match.
    pub routing: &'static str,
}

impl Region {
    pub fn from_lcu(region: &str) -> Region {
        match region.to_ascii_uppercase().as_str() {
            "BR" => Region { platform: "br1", routing: "americas" },
            "NA" => Region { platform: "na1", routing: "americas" },
            "LAN" => Region { platform: "la1", routing: "americas" },
            "LAS" => Region { platform: "la2", routing: "americas" },
            "EUW" => Region { platform: "euw1", routing: "europe" },
            "EUNE" => Region { platform: "eun1", routing: "europe" },
            "TR" => Region { platform: "tr1", routing: "europe" },
            "RU" => Region { platform: "ru", routing: "europe" },
            "KR" => Region { platform: "kr", routing: "asia" },
            "JP" => Region { platform: "jp1", routing: "asia" },
            "OCE" | "OC1" => Region { platform: "oc1", routing: "sea" },
            "PH" | "PH2" => Region { platform: "ph2", routing: "sea" },
            "SG" | "SG2" => Region { platform: "sg2", routing: "sea" },
            "TH" | "TH2" => Region { platform: "th2", routing: "sea" },
            "TW" | "TW2" => Region { platform: "tw2", routing: "sea" },
            "VN" | "VN2" => Region { platform: "vn2", routing: "sea" },
            _ => Region { platform: "br1", routing: "americas" },
        }
    }

    /// A partir do host de plataforma (`br1`, `euw1`, `kr`), que é como as
    /// ferramentas de linha de comando recebem a região.
    ///
    /// Devolve `None` em vez de cair num padrão: num coletor, errar a região
    /// em silêncio significa agregar partidas de outro servidor sob o rótulo
    /// errado.
    pub fn from_platform(platform: &str) -> Option<Region> {
        let platform = platform.to_ascii_lowercase();
        PLATFORMS
            .iter()
            .find(|(host, _)| *host == platform)
            .map(|(host, routing)| Region {
                platform: host,
                routing,
            })
    }

    pub fn platforms() -> impl Iterator<Item = &'static str> {
        PLATFORMS.iter().map(|(host, _)| *host)
    }
}

/// Todo host de plataforma e o host regional que o atende.
const PLATFORMS: &[(&str, &str)] = &[
    ("br1", "americas"),
    ("na1", "americas"),
    ("la1", "americas"),
    ("la2", "americas"),
    ("euw1", "europe"),
    ("eun1", "europe"),
    ("tr1", "europe"),
    ("ru", "europe"),
    ("me1", "europe"),
    ("kr", "asia"),
    ("jp1", "asia"),
    ("oc1", "sea"),
    ("ph2", "sea"),
    ("sg2", "sea"),
    ("th2", "sea"),
    ("tw2", "sea"),
    ("vn2", "sea"),
];

/// `nome#tag` -> (`nome`, `tag`)
pub fn split_riot_id(riot_id: &str) -> Option<(String, String)> {
    let (name, tag) = riot_id.rsplit_once('#')?;
    let name = name.trim();
    let tag = tag.trim();
    if name.is_empty() || tag.is_empty() {
        return None;
    }
    Some((name.to_string(), tag.to_string()))
}

/// Riot IDs têm espaço ("crit is back"), então o nome vai percent-encoded.
pub fn urlencode(value: &str) -> String {
    value
        .bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn riot_id_com_espaco() {
        let (name, tag) = split_riot_id("crit is back#split").unwrap();
        assert_eq!(name, "crit is back");
        assert_eq!(tag, "split");
    }

    #[test]
    fn riot_id_com_cerquilha_no_nome_usa_a_ultima() {
        let (name, tag) = split_riot_id("a#b#tag").unwrap();
        assert_eq!(name, "a#b");
        assert_eq!(tag, "tag");
    }

    #[test]
    fn riot_id_invalido() {
        assert!(split_riot_id("semtag").is_none());
        assert!(split_riot_id("#tag").is_none());
        assert!(split_riot_id("nome#").is_none());
    }

    #[test]
    fn espaco_vira_percent_20() {
        assert_eq!(urlencode("crit is back"), "crit%20is%20back");
    }

    #[test]
    fn acento_vira_utf8_percent() {
        assert_eq!(urlencode("ç"), "%C3%A7");
    }

    #[test]
    fn regiao_desconhecida_cai_no_padrao() {
        assert_eq!(Region::from_lcu("XX").routing, "americas");
    }

    #[test]
    fn regioes_conhecidas() {
        assert_eq!(Region::from_lcu("br").platform, "br1");
        assert_eq!(Region::from_lcu("KR").routing, "asia");
        assert_eq!(Region::from_lcu("EUW").routing, "europe");
        assert_eq!(Region::from_lcu("oce").platform, "oc1");
    }

    #[test]
    fn plataforma_resolve_o_host_regional() {
        assert_eq!(Region::from_platform("euw1").unwrap().routing, "europe");
        assert_eq!(Region::from_platform("KR").unwrap().routing, "asia");
        assert_eq!(Region::from_platform("br1").unwrap().platform, "br1");
    }

    #[test]
    fn plataforma_desconhecida_nao_cai_num_padrao() {
        // O contrário do `from_lcu`, e de propósito: num coletor, assumir a
        // região errada agrega partidas de outro servidor sob o rótulo errado.
        assert!(Region::from_platform("xx9").is_none());
        assert!(Region::from_platform("").is_none());
    }

    #[test]
    fn toda_plataforma_listada_resolve() {
        for host in Region::platforms() {
            assert!(Region::from_platform(host).is_some(), "{host}");
        }
    }
}
