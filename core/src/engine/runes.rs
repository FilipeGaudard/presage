//! Runa principal para o confronto da sua rota.
//!
//! # De onde isto sai, e de onde não sai
//!
//! **Não sai de winrate.** Não existe aqui nenhuma estatística de partidas, e
//! enquanto a base de dados não existir não vai existir. O que existe é um
//! conjunto de regras mecânicas sobre o confronto: quem alcança quem, quem
//! aguenta troca longa, quem morre de explosão.
//!
//! Isso é diferente de "a melhor runa segundo um milhão de partidas", e a
//! interface tem de dizer a diferença. Uma regra mecânica acerta o caso comum
//! e erra o campeão excêntrico; uma base de dados acerta os dois. Quando a
//! base existir, ela entra como mais um componente aqui — e aí sim o número
//! vira winrate.
//!
//! As runas são referenciadas pela **chave textual** do Data Dragon
//! (`Conqueror`, `GraspOfTheUndying`), nunca pelo id numérico. Id muda de
//! patch e vira ícone errado em silêncio; a chave é estável, e uma chave que
//! suma simplesmente deixa de ser sugerida.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::champion::{Champion, ChampionIndex, Role};

const MAX_SUGGESTIONS: usize = 3;

/// Uma runa do Data Dragon, achatada para consulta por chave.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Rune {
    pub id: i64,
    /// `Conqueror`, `GraspOfTheUndying`. Estável entre patches.
    pub key: String,
    pub name: String,
    /// Caminho do ícone no CDN.
    pub icon: String,
    /// Árvore a que pertence: `Precision`, `Domination`, ...
    pub tree_key: String,
    pub tree_name: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RuneIndex {
    pub version: String,
    /// Chave da runa -> runa.
    pub by_key: HashMap<String, Rune>,
}

impl RuneIndex {
    pub fn get(&self, key: &str) -> Option<&Rune> {
        self.by_key.get(key)
    }
    pub fn is_empty(&self) -> bool {
        self.by_key.is_empty()
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RuneSuggestion {
    pub key: String,
    pub id: i64,
    pub name: String,
    pub icon: String,
    pub tree_name: String,
    /// Quanto as regras puxam para esta runa. Índice de encaixe, não winrate.
    pub score: f64,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RunePlan {
    pub keystones: Vec<RuneSuggestion>,
    /// Árvore secundária e o motivo dela.
    pub secondary: Option<RuneSuggestion>,
    pub matchup: String,
    /// Sempre falso hoje. Vira verdadeiro quando houver base de partidas.
    pub from_data: bool,
}

// ---- leitura de classe ----

fn tem(champion: &Champion, tag: &str) -> bool {
    champion.tags.iter().any(|t| t == tag)
}

fn assassino(c: &Champion) -> bool {
    tem(c, "Assassin")
}
fn mago(c: &Champion) -> bool {
    tem(c, "Mage")
}
fn atirador(c: &Champion) -> bool {
    tem(c, "Marksman")
}
fn suporte(c: &Champion) -> bool {
    tem(c, "Support")
}
fn tanque(c: &Champion) -> bool {
    tem(c, "Tank")
}
fn lutador(c: &Champion) -> bool {
    tem(c, "Fighter")
}

/// Explode em dano concentrado: quem te mata antes de a troca longa começar.
fn explode(c: &Champion) -> bool {
    assassino(c) || (mago(c) && !suporte(c))
}

// ---- as regras ----

struct Regra {
    runa: &'static str,
    peso: f64,
    motivo: &'static str,
    quando: fn(&Champion, &Champion, Role) -> bool,
}

/// Cada linha é uma afirmação sobre a mecânica do confronto que dá para
/// defender em voz alta. Nenhuma é sobre frequência de vitória.
const REGRAS: &[Regra] = &[
    Regra {
        runa: "GraspOfTheUndying",
        peso: 1.0,
        motivo: "você é corpo a corpo contra alguém que bate de longe: a troca vem por desgaste",
        quando: |eu, ele, _| !eu.is_ranged() && ele.is_ranged() && (tanque(eu) || lutador(eu)),
    },
    Regra {
        runa: "Conqueror",
        peso: 1.0,
        motivo: "os dois ficam na troca longa, e é nela que a runa rende",
        quando: |eu, ele, _| !eu.is_ranged() && lutador(eu) && !ele.is_ranged(),
    },
    Regra {
        runa: "Conqueror",
        peso: 0.8,
        motivo: "contra alguém que aguenta apanhar, o dano cresce com o tempo de luta",
        quando: |eu, ele, _| (lutador(eu) || tem(eu, "Juggernaut")) && ele.is_frontline(),
    },
    Regra {
        runa: "LethalTempo",
        peso: 0.9,
        motivo: "atirador contra frontline: quem vence é quem bate por mais tempo",
        quando: |eu, ele, _| atirador(eu) && ele.is_frontline(),
    },
    Regra {
        runa: "PressTheAttack",
        peso: 0.85,
        motivo: "atirador contra alvo frágil: três tiros já decidem a troca",
        quando: |eu, ele, _| atirador(eu) && !ele.is_frontline(),
    },
    Regra {
        runa: "Electrocute",
        peso: 1.0,
        motivo: "você mata em sequência curta e ele não tem como aguentar",
        quando: |eu, ele, _| assassino(eu) && !ele.is_frontline(),
    },
    Regra {
        runa: "ArcaneComet",
        peso: 0.9,
        motivo: "mago contra corpo a corpo: dá para castigar sem entrar no alcance dele",
        quando: |eu, ele, _| mago(eu) && !suporte(eu) && !ele.is_ranged(),
    },
    Regra {
        runa: "PhaseRush",
        peso: 0.95,
        motivo: "contra quem explode, sair da briga vale mais que somar dano",
        quando: |eu, ele, _| (mago(eu) || lutador(eu)) && explode(ele) && !ele.is_frontline(),
    },
    Regra {
        runa: "SummonAery",
        peso: 0.8,
        motivo: "troca curta e repetida, que é como a rota se desenrola aqui",
        quando: |eu, ele, _| (mago(eu) || suporte(eu)) && ele.is_ranged(),
    },
    Regra {
        runa: "Aftershock",
        peso: 0.95,
        motivo: "você entra primeiro, e a runa paga o preço de entrar",
        quando: |eu, _, role| tanque(eu) && matches!(role, Role::Support | Role::Jungle | Role::Top),
    },
    Regra {
        runa: "Guardian",
        peso: 0.8,
        motivo: "suporte sem iniciação: o valor está em segurar o seu atirador vivo",
        quando: |eu, _, role| suporte(eu) && !tanque(eu) && role == Role::Support,
    },
    Regra {
        runa: "FleetFootwork",
        peso: 0.75,
        motivo: "rota de desgaste: sustentar a onda vale mais que ganhar a troca",
        quando: |eu, ele, _| eu.is_ranged() && ele.is_ranged() && !assassino(eu),
    },
    Regra {
        runa: "DarkHarvest",
        peso: 0.7,
        motivo: "assassino contra alvo que morre: a runa cresce a cada abate",
        quando: |eu, ele, _| assassino(eu) && !ele.is_frontline() && !eu.is_ranged(),
    },
];

/// Árvore secundária: responde à ameaça, não ao seu estilo.
struct RegraSecundaria {
    runa: &'static str,
    motivo: &'static str,
    quando: fn(&Champion, &Champion) -> bool,
}

const SECUNDARIAS: &[RegraSecundaria] = &[
    RegraSecundaria {
        runa: "SecondWind",
        motivo: "contra desgaste de longe, a regeneração entre trocas é o que te mantém na rota",
        quando: |eu, ele| ele.is_ranged() && !eu.is_ranged(),
    },
    RegraSecundaria {
        runa: "BonePlating",
        motivo: "contra sequência curta de dano, ela corta justamente a parte que te mata",
        quando: |_, ele| explode(ele),
    },
    RegraSecundaria {
        runa: "TasteOfBlood",
        motivo: "troca curta e repetida: o retorno de vida paga a troca",
        quando: |eu, ele| eu.is_ranged() && ele.is_ranged(),
    },
];

fn para_sugestao(rune: &Rune, score: f64, reasons: Vec<String>) -> RuneSuggestion {
    RuneSuggestion {
        key: rune.key.clone(),
        id: rune.id,
        name: rune.name.clone(),
        icon: rune.icon.clone(),
        tree_name: rune.tree_name.clone(),
        score,
        reasons,
    }
}

/// O plano de runas para este confronto.
///
/// Devolve `None` quando falta o índice de runas ou um dos dois campeões —
/// sugerir runa sem saber contra quem seria o mesmo que sugerir a do patch.
pub fn plan(
    mine: i64,
    opponent: i64,
    role: Role,
    champions: &ChampionIndex,
    runes: &RuneIndex,
) -> Option<RunePlan> {
    if runes.is_empty() {
        return None;
    }
    let eu = champions.get(mine)?;
    let ele = champions.get(opponent)?;

    let mut pontos: HashMap<&str, (f64, Vec<String>)> = HashMap::new();
    for regra in REGRAS {
        if !(regra.quando)(eu, ele, role) {
            continue;
        }
        // Runa que a Riot removeu ou renomeou deixa de ser sugerida, em vez
        // de virar um ícone quebrado.
        if runes.get(regra.runa).is_none() {
            continue;
        }
        let entrada = pontos.entry(regra.runa).or_insert((0.0, Vec::new()));
        entrada.0 += regra.peso;
        entrada.1.push(regra.motivo.to_string());
    }

    let mut keystones: Vec<RuneSuggestion> = pontos
        .into_iter()
        .filter_map(|(key, (score, reasons))| {
            runes.get(key).map(|r| para_sugestao(r, score, reasons))
        })
        .collect();

    keystones.sort_by(|a, b| b.score.total_cmp(&a.score).then_with(|| a.name.cmp(&b.name)));
    keystones.truncate(MAX_SUGGESTIONS);

    let secondary = SECUNDARIAS
        .iter()
        .find(|regra| (regra.quando)(eu, ele) && runes.get(regra.runa).is_some())
        .and_then(|regra| {
            runes
                .get(regra.runa)
                .map(|r| para_sugestao(r, 1.0, vec![regra.motivo.to_string()]))
        });

    Some(RunePlan {
        keystones,
        secondary,
        matchup: format!("{} contra {}", eu.name, ele.name),
        // Vira verdadeiro no dia em que a base de partidas entrar na conta.
        from_data: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::champion::fake;

    fn runa(id: i64, key: &str, tree: &str) -> Rune {
        Rune {
            id,
            key: key.into(),
            name: key.into(),
            icon: format!("perk-images/Styles/{key}.png"),
            tree_key: tree.into(),
            tree_name: tree.into(),
        }
    }

    fn runas() -> RuneIndex {
        let lista = [
            (8437, "GraspOfTheUndying", "Resolve"),
            (8010, "Conqueror", "Precision"),
            (8008, "LethalTempo", "Precision"),
            (8005, "PressTheAttack", "Precision"),
            (8021, "FleetFootwork", "Precision"),
            (8112, "Electrocute", "Domination"),
            (8128, "DarkHarvest", "Domination"),
            (8229, "ArcaneComet", "Sorcery"),
            (8230, "PhaseRush", "Sorcery"),
            (8214, "SummonAery", "Sorcery"),
            (8439, "Aftershock", "Resolve"),
            (8465, "Guardian", "Resolve"),
            (8473, "BonePlating", "Resolve"),
            (8444, "SecondWind", "Resolve"),
            (8139, "TasteOfBlood", "Domination"),
        ];
        RuneIndex {
            version: "test".into(),
            by_key: lista
                .iter()
                .map(|(id, key, tree)| (key.to_string(), runa(*id, key, tree)))
                .collect(),
        }
    }

    fn indice(campeoes: Vec<Champion>) -> ChampionIndex {
        ChampionIndex {
            version: "test".into(),
            by_id: campeoes.into_iter().map(|c| (c.id, c)).collect(),
        }
    }

    /// Corpo a corpo tanque, à distância mago, etc.
    fn tanque_melee(id: i64) -> Champion {
        fake(id, "Tanque", "Tank", 5.0, 6.0, 9.0)
    }
    fn lutador_melee(id: i64) -> Champion {
        fake(id, "Lutador", "Fighter", 8.0, 3.0, 6.0)
    }
    fn mago_ranged(id: i64) -> Champion {
        fake(id, "Mago", "Mage", 2.0, 9.0, 3.0)
    }
    fn assassino_melee(id: i64) -> Champion {
        let mut c = fake(id, "Assassino", "Assassin", 9.0, 3.0, 4.0);
        c.attack_range = 150.0;
        c
    }
    fn atirador_ranged(id: i64) -> Champion {
        fake(id, "Atirador", "Marksman", 9.0, 2.0, 3.0)
    }

    fn chaves(plano: &RunePlan) -> Vec<String> {
        plano.keystones.iter().map(|k| k.key.clone()).collect()
    }

    #[test]
    fn sem_indice_de_runas_nao_ha_plano() {
        let idx = indice(vec![lutador_melee(1), mago_ranged(2)]);
        assert!(plan(1, 2, Role::Top, &idx, &RuneIndex::default()).is_none());
    }

    #[test]
    fn sem_oponente_conhecido_nao_ha_plano() {
        let idx = indice(vec![lutador_melee(1)]);
        assert!(plan(1, 999, Role::Top, &idx, &runas()).is_none());
    }

    #[test]
    fn corpo_a_corpo_contra_distancia_recebe_sustentacao() {
        let idx = indice(vec![tanque_melee(1), mago_ranged(2)]);
        let plano = plan(1, 2, Role::Top, &idx, &runas()).unwrap();

        assert!(
            chaves(&plano).contains(&"GraspOfTheUndying".to_string()),
            "{:?}",
            chaves(&plano)
        );
    }

    #[test]
    fn atirador_contra_frontline_quer_troca_longa() {
        let idx = indice(vec![atirador_ranged(1), tanque_melee(2)]);
        let plano = plan(1, 2, Role::Bot, &idx, &runas()).unwrap();
        assert_eq!(plano.keystones[0].key, "LethalTempo");
    }

    #[test]
    fn atirador_contra_alvo_fragil_quer_troca_curta() {
        let idx = indice(vec![atirador_ranged(1), mago_ranged(2)]);
        let plano = plan(1, 2, Role::Bot, &idx, &runas()).unwrap();
        assert!(chaves(&plano).contains(&"PressTheAttack".to_string()));
    }

    #[test]
    fn assassino_contra_alvo_fragil_quer_explosao() {
        let idx = indice(vec![assassino_melee(1), mago_ranged(2)]);
        let plano = plan(1, 2, Role::Mid, &idx, &runas()).unwrap();
        assert_eq!(plano.keystones[0].key, "Electrocute");
    }

    #[test]
    fn o_mesmo_campeao_muda_de_runa_conforme_o_oponente() {
        // O ponto inteiro da funcionalidade: a runa responde ao confronto.
        let idx = indice(vec![
            atirador_ranged(1),
            tanque_melee(2),
            mago_ranged(3),
        ]);
        let contra_tanque = plan(1, 2, Role::Bot, &idx, &runas()).unwrap();
        let contra_mago = plan(1, 3, Role::Bot, &idx, &runas()).unwrap();

        assert_ne!(
            contra_tanque.keystones[0].key, contra_mago.keystones[0].key,
            "a runa deveria mudar com o oponente"
        );
    }

    #[test]
    fn contra_quem_explode_a_secundaria_corta_a_explosao() {
        let idx = indice(vec![lutador_melee(1), assassino_melee(2)]);
        let plano = plan(1, 2, Role::Top, &idx, &runas()).unwrap();
        assert_eq!(plano.secondary.unwrap().key, "BonePlating");
    }

    #[test]
    fn contra_desgaste_de_longe_a_secundaria_regenera() {
        let idx = indice(vec![lutador_melee(1), mago_ranged(2)]);
        let plano = plan(1, 2, Role::Top, &idx, &runas()).unwrap();
        assert_eq!(plano.secondary.unwrap().key, "SecondWind");
    }

    #[test]
    fn runa_que_sumiu_do_patch_nao_e_sugerida() {
        // A Riot remove e renomeia runas. Preferimos sugerir menos a sugerir
        // um ícone quebrado.
        let mut parcial = runas();
        parcial.by_key.remove("Electrocute");

        let idx = indice(vec![assassino_melee(1), mago_ranged(2)]);
        let plano = plan(1, 2, Role::Mid, &idx, &parcial).unwrap();

        assert!(!chaves(&plano).contains(&"Electrocute".to_string()));
        for k in &plano.keystones {
            assert!(parcial.get(&k.key).is_some());
        }
    }

    #[test]
    fn nunca_sai_mais_que_o_teto_de_sugestoes() {
        let idx = indice(vec![lutador_melee(1), tanque_melee(2)]);
        let plano = plan(1, 2, Role::Top, &idx, &runas()).unwrap();
        assert!(plano.keystones.len() <= MAX_SUGGESTIONS);
    }

    #[test]
    fn toda_runa_sugerida_vem_com_motivo() {
        let idx = indice(vec![assassino_melee(1), atirador_ranged(2)]);
        let plano = plan(1, 2, Role::Mid, &idx, &runas()).unwrap();

        for k in &plano.keystones {
            assert!(!k.reasons.is_empty(), "{} veio sem motivo", k.name);
        }
    }

    #[test]
    fn a_lista_sai_ordenada() {
        let idx = indice(vec![atirador_ranged(1), tanque_melee(2)]);
        let plano = plan(1, 2, Role::Bot, &idx, &runas()).unwrap();
        for par in plano.keystones.windows(2) {
            assert!(par[0].score >= par[1].score);
        }
    }

    #[test]
    fn o_plano_nunca_se_apresenta_como_estatistica() {
        // Enquanto não houver base de partidas, `from_data` é falso — e é por
        // ele que a interface decide se escreve "segundo N partidas".
        let idx = indice(vec![lutador_melee(1), mago_ranged(2)]);
        let plano = plan(1, 2, Role::Top, &idx, &runas()).unwrap();
        assert!(!plano.from_data);
    }

    #[test]
    fn o_confronto_aparece_escrito() {
        let idx = indice(vec![lutador_melee(1), mago_ranged(2)]);
        let plano = plan(1, 2, Role::Top, &idx, &runas()).unwrap();
        assert_eq!(plano.matchup, "Lutador contra Mago");
    }
}
