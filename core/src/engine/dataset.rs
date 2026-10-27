//! Base de partidas agregada: winrate por rota, matchup e sinergia.
//!
//! O arquivo não vem no instalador. Winrate base, matchup e sinergia precisam
//! de milhões de partidas de milhares de contas — uma conta sozinha não produz
//! nenhum dos três, e inventar números daria um app que parece funcionar e
//! recomenda errado.
//!
//! O que existe aqui é o encaixe: um formato, um carregador e o teste de que
//! base ausente devolve `None` em vez de palpite. No dia em que o pipeline de
//! ingestão rodar, basta gravar o arquivo no lugar certo — nada mais muda.
//!
//! As chaves são texto porque JSON só tem chave de texto. `103:Mid`,
//! `103:238:Mid`, `64:103` (o par de sinergia sempre com o menor id antes, para
//! Lee+Ahri e Ahri+Lee caírem na mesma entrada).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::champion::Role;
use super::stats::{PersonalStats, Record, StatsStore};

// `default` no struct inteiro: um arquivo truncado, de uma versão mais velha
// do pipeline ou escrito à mão para teste tem de carregar. Campo que falta vira
// vazio, e `is_usable` decide se aquilo serve — melhor que recusar o arquivo e
// cair sem base nenhuma por causa de uma chave ausente.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Dataset {
    /// Patch de onde as partidas saíram. Base de patch antigo recomenda build
    /// antiga, então a interface mostra isto.
    pub patch: String,
    pub region: String,
    pub tier: String,
    /// ISO 8601.
    pub generated_at: String,
    /// Quantas partidas entraram na agregação.
    pub matches: u64,
    #[serde(default)]
    pub base: HashMap<String, Record>,
    #[serde(default)]
    pub matchup: HashMap<String, Record>,
    #[serde(default)]
    pub synergy: HashMap<String, Record>,
}

pub fn base_key(champion: i64, role: Role) -> String {
    format!("{champion}:{role:?}")
}

pub fn matchup_key(champion: i64, opponent: i64, role: Role) -> String {
    format!("{champion}:{opponent}:{role:?}")
}

/// Sinergia não tem ordem: o par é sempre guardado com o menor id antes.
pub fn synergy_key(a: i64, b: i64) -> String {
    let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
    format!("{lo}:{hi}")
}

impl Dataset {
    pub fn is_usable(&self) -> bool {
        self.matches > 0 && !self.base.is_empty()
    }
}

impl StatsStore for Dataset {
    fn base(&self, champion: i64, role: Role) -> Option<Record> {
        self.base.get(&base_key(champion, role)).copied()
    }
    fn matchup(&self, champion: i64, opponent: i64, role: Role) -> Option<Record> {
        self.matchup
            .get(&matchup_key(champion, opponent, role))
            .copied()
    }
    fn synergy(&self, champion: i64, ally: i64) -> Option<Record> {
        self.synergy.get(&synergy_key(champion, ally)).copied()
    }
    fn personal(&self, _champion: i64) -> Option<Record> {
        None
    }
    fn has_global(&self) -> bool {
        self.is_usable()
    }
}

/// O que o app usa de verdade: a base agregada (quando existe) para os três
/// componentes globais, e o histórico da conta para o componente pessoal.
///
/// Os dois são independentes de propósito. Ter a conta ligada não produz
/// winrate de matchup, e ter a base não diz nada sobre como *você* joga.
#[derive(Debug, Clone, Default)]
pub struct Layered {
    pub global: Option<Dataset>,
    pub personal: PersonalStats,
}

impl Layered {
    pub fn with_global(mut self, dataset: Dataset) -> Self {
        self.global = dataset.is_usable().then_some(dataset);
        self
    }

    pub fn patch(&self) -> Option<&str> {
        self.global.as_ref().map(|d| d.patch.as_str())
    }

    pub fn matches(&self) -> u64 {
        self.global.as_ref().map_or(0, |d| d.matches)
    }
}

impl StatsStore for Layered {
    fn base(&self, champion: i64, role: Role) -> Option<Record> {
        self.global.as_ref()?.base(champion, role)
    }
    fn matchup(&self, champion: i64, opponent: i64, role: Role) -> Option<Record> {
        self.global.as_ref()?.matchup(champion, opponent, role)
    }
    fn synergy(&self, champion: i64, ally: i64) -> Option<Record> {
        self.global.as_ref()?.synergy(champion, ally)
    }
    fn personal(&self, champion: i64) -> Option<Record> {
        self.personal.personal(champion)
    }
    fn has_global(&self) -> bool {
        self.global.is_some()
    }
    fn has_personal(&self) -> bool {
        self.personal.has_personal()
    }
    fn personal_games(&self) -> u32 {
        self.personal.personal_games()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cheio() -> Dataset {
        Dataset {
            patch: "16.21".into(),
            region: "BR1".into(),
            tier: "EMERALD+".into(),
            generated_at: "2026-10-27T00:00:00Z".into(),
            matches: 1_200_000,
            base: HashMap::from([(base_key(103, Role::Mid), Record::new(40_000, 21_000))]),
            matchup: HashMap::from([(
                matchup_key(103, 238, Role::Mid),
                Record::new(1840, 1050),
            )]),
            synergy: HashMap::from([(synergy_key(103, 64), Record::new(900, 480))]),
        }
    }

    #[test]
    fn base_vazia_nao_responde_nada() {
        let vazia = Dataset::default();
        assert!(!vazia.has_global());
        assert!(vazia.base(103, Role::Mid).is_none());
        assert!(vazia.matchup(103, 238, Role::Mid).is_none());
        assert!(vazia.synergy(103, 64).is_none());
    }

    #[test]
    fn base_com_partidas_responde() {
        let d = cheio();
        assert!(d.has_global());
        assert_eq!(d.matchup(103, 238, Role::Mid), Some(Record::new(1840, 1050)));
    }

    #[test]
    fn a_sinergia_nao_depende_da_ordem_do_par() {
        let d = cheio();
        assert_eq!(d.synergy(103, 64), d.synergy(64, 103));
        assert!(d.synergy(103, 64).is_some());
    }

    #[test]
    fn matchup_e_direcional_ao_contrario_da_sinergia() {
        let d = cheio();
        assert!(d.matchup(103, 238, Role::Mid).is_some());
        assert!(d.matchup(238, 103, Role::Mid).is_none());
    }

    #[test]
    fn a_rota_faz_parte_da_chave() {
        let d = cheio();
        assert!(d.base(103, Role::Mid).is_some());
        assert!(d.base(103, Role::Top).is_none());
    }

    #[test]
    fn base_que_diz_ter_partidas_mas_esta_vazia_nao_conta() {
        // Arquivo truncado ou pipeline que falhou no meio. Melhor tratar como
        // ausente do que recomendar a partir de quatro linhas.
        let mentirosa = Dataset {
            matches: 9_000_000,
            ..Default::default()
        };
        assert!(!mentirosa.is_usable());
        assert!(!mentirosa.has_global());
    }

    #[test]
    fn as_camadas_sao_independentes() {
        let mut pessoal = PersonalStats::default();
        pessoal.add_match("BR1_1", 103, true);

        let so_pessoal = Layered {
            global: None,
            personal: pessoal.clone(),
        };
        assert!(so_pessoal.has_personal());
        assert!(!so_pessoal.has_global());
        // Ter a conta ligada não produz matchup.
        assert!(so_pessoal.matchup(103, 238, Role::Mid).is_none());

        let so_global = Layered::default().with_global(cheio());
        assert!(so_global.has_global());
        assert!(!so_global.has_personal());
        // E ter a base não diz nada sobre como você joga.
        assert!(so_global.personal(103).is_none());
    }

    #[test]
    fn as_duas_camadas_juntas_respondem_as_duas_perguntas() {
        let mut pessoal = PersonalStats::default();
        pessoal.add_match("BR1_1", 103, true);

        let tudo = Layered {
            global: None,
            personal: pessoal,
        }
        .with_global(cheio());

        assert!(tudo.has_global() && tudo.has_personal());
        assert!(tudo.matchup(103, 238, Role::Mid).is_some());
        assert_eq!(tudo.personal(103), Some(Record::new(1, 1)));
        assert_eq!(tudo.patch(), Some("16.21"));
        assert_eq!(tudo.matches(), 1_200_000);
    }

    #[test]
    fn base_inutilizavel_nao_e_aceita_como_global() {
        let camada = Layered::default().with_global(Dataset {
            matches: 10,
            ..Default::default()
        });
        assert!(!camada.has_global());
        assert_eq!(camada.patch(), None);
    }

    #[test]
    fn o_formato_sobrevive_a_ida_e_volta_em_json() {
        let original = cheio();
        let texto = serde_json::to_string(&original).unwrap();
        let voltou: Dataset = serde_json::from_str(&texto).unwrap();

        assert_eq!(voltou.matches, original.matches);
        assert_eq!(voltou.matchup(103, 238, Role::Mid), Some(Record::new(1840, 1050)));
    }

    #[test]
    fn arquivo_so_com_o_cabecalho_carrega_sem_quebrar() {
        let voltou: Dataset =
            serde_json::from_str(r#"{"patch":"16.21","matches":0}"#).unwrap();
        assert!(!voltou.is_usable());
        assert!(voltou.base.is_empty());
    }
}
