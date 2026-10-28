//! O que o jogador quer ver por cima do jogo.
//!
//! O overlay não tem uma resposta certa. Quem joga de suporte olha objetivo e
//! ignora build; quem joga de ADC quer o próximo item e acha o ouro do time
//! ruído. Decidir por todo mundo seria errar com a maioria.
//!
//! Então cada seção liga e desliga, e a escolha mora aqui — num arquivo, para
//! sobreviver a fechar o app.

use serde::{Deserialize, Serialize};

/// Uma seção do overlay.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Section {
    Gold,
    Damage,
    Objectives,
    Priority,
    Skirmish,
    NextItem,
}

impl Section {
    /// Todas, na ordem em que aparecem no overlay.
    pub const ALL: [Section; 6] = [
        Section::Gold,
        Section::Damage,
        Section::Objectives,
        Section::Priority,
        Section::Skirmish,
        Section::NextItem,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Section::Gold => "Ouro em itens",
            Section::Damage => "Dano que vem contra você",
            Section::Objectives => "Objetivos",
            Section::Priority => "Prioridade de rota",
            Section::Skirmish => "Escaramuça",
            Section::NextItem => "Próximo item",
        }
    }

    pub fn detail(self) -> &'static str {
        match self {
            Section::Gold => "a diferença entre os times, soma exata dos inventários",
            Section::Damage => "quanto é físico, sua vida efetiva e a penetração deles",
            Section::Objectives => "dragão, barão, arauto e grubs",
            Section::Priority => "qual rota vale mais agora, com o motivo",
            Section::Skirmish => "a briga que importa neste minuto",
            Section::NextItem => "duas opções, contra o que eles têm agora",
        }
    }
}

/// Seção escondida pelo jogador.
///
/// Guardamos o que está **desligado**, não o que está ligado. A diferença
/// aparece quando uma seção nova chega: com a lista de ligados, ela nasceria
/// invisível para todo mundo que já tem configuração salva, e ninguém
/// descobriria que ela existe. Guardando os desligados, o que é novo aparece.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct OverlayPrefs {
    pub hidden: Vec<Section>,
}

impl OverlayPrefs {
    pub fn shows(&self, section: Section) -> bool {
        !self.hidden.contains(&section)
    }

    pub fn set(&mut self, section: Section, visible: bool) {
        self.hidden.retain(|s| *s != section);
        if !visible {
            self.hidden.push(section);
        }
    }

    /// Tudo desligado quer dizer overlay vazio, e overlay vazio é uma janela
    /// invisível ocupando memória. Quem chama decide o que fazer.
    pub fn anything_visible(&self) -> bool {
        Section::ALL.iter().any(|s| self.shows(*s))
    }

    pub fn visible(&self) -> Vec<Section> {
        Section::ALL.iter().copied().filter(|s| self.shows(*s)).collect()
    }
}

/// Preferências do draft.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct DraftPrefs {
    /// Recomendar também quem a conta não pode escolher. Desligado: a lista da
    /// conta filtra, quando o cliente a entrega.
    pub show_all_champions: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn por_padrao_tudo_aparece() {
        let p = OverlayPrefs::default();
        for s in Section::ALL {
            assert!(p.shows(s), "{s:?} deveria aparecer");
        }
        assert_eq!(p.visible().len(), Section::ALL.len());
    }

    #[test]
    fn desligar_e_religar_volta_ao_estado_anterior() {
        let mut p = OverlayPrefs::default();
        p.set(Section::Gold, false);
        assert!(!p.shows(Section::Gold));
        assert!(p.shows(Section::Damage), "não pode mexer nas outras");

        p.set(Section::Gold, true);
        assert_eq!(p, OverlayPrefs::default());
    }

    #[test]
    fn desligar_duas_vezes_nao_duplica() {
        let mut p = OverlayPrefs::default();
        p.set(Section::Skirmish, false);
        p.set(Section::Skirmish, false);
        assert_eq!(p.hidden.len(), 1);
    }

    #[test]
    fn tudo_desligado_e_detectavel() {
        let mut p = OverlayPrefs::default();
        assert!(p.anything_visible());
        for s in Section::ALL {
            p.set(s, false);
        }
        assert!(!p.anything_visible());
        assert!(p.visible().is_empty());
    }

    #[test]
    fn a_ordem_das_visiveis_e_a_do_overlay() {
        let mut p = OverlayPrefs::default();
        p.set(Section::Objectives, false);
        assert_eq!(
            p.visible(),
            vec![
                Section::Gold,
                Section::Damage,
                Section::Priority,
                Section::Skirmish,
                Section::NextItem,
            ]
        );
    }

    #[test]
    fn secao_nova_nasce_visivel_para_quem_ja_tinha_configuracao() {
        // O motivo de guardar os desligados. Este arquivo foi salvo antes de
        // `skirmish` existir, e ela tem de aparecer mesmo assim.
        let antigo = r#"{"hidden":["gold"]}"#;
        let p: OverlayPrefs = serde_json::from_str(antigo).unwrap();

        assert!(!p.shows(Section::Gold));
        assert!(p.shows(Section::Skirmish), "o que é novo tem de aparecer");
    }

    #[test]
    fn arquivo_vazio_ou_corrompido_nao_esconde_nada() {
        let vazio: OverlayPrefs = serde_json::from_str("{}").unwrap();
        assert!(vazio.anything_visible());
        assert_eq!(vazio, OverlayPrefs::default());
    }

    #[test]
    fn sobrevive_a_ida_e_volta_em_json() {
        let mut p = OverlayPrefs::default();
        p.set(Section::Gold, false);
        p.set(Section::NextItem, false);

        let texto = serde_json::to_string(&p).unwrap();
        let voltou: OverlayPrefs = serde_json::from_str(&texto).unwrap();

        assert_eq!(voltou, p);
        assert!(!voltou.shows(Section::NextItem));
        assert!(voltou.shows(Section::Damage));
    }

    #[test]
    fn toda_secao_tem_rotulo_e_explicacao() {
        for s in Section::ALL {
            assert!(!s.label().is_empty(), "{s:?}");
            assert!(!s.detail().is_empty(), "{s:?}");
        }
    }
}
