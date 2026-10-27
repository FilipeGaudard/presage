//! O que falta na sua pool, e quem preenche.
//!
//! O terceiro passo do Construtor de Pool: depois dos dois core e dos três
//! situacionais, o sexto pick não é escolhido por gosto — é escolhido para
//! cobrir o buraco que os cinco primeiros deixaram.
//!
//! **De onde saem estes números.** Daqui não sai winrate. Sai a composição da
//! sua pool (tipo de dano, frontline, alcance) e a matemática de resistências
//! do jogo. Um campeão "robusto" aqui é um campeão cuja vida efetiva varia
//! pouco conforme o dano que vem contra ele — isso é aritmética sobre armadura
//! e resistência mágica, não estatística de partidas. A diferença importa, e a
//! interface tem de dizer isso.

use serde::Serialize;

use super::champion::{Champion, ChampionIndex, Role};
use super::damage::damage_multiplier;

/// Nível de referência para comparar campeões entre si.
const LEVEL: u8 = 11;

/// Acima disto a pool é considerada de um tipo de dano só.
const SKEW: f64 = 0.72;

/// Peso de cada buraco preenchido contra a robustez.
const W_GAP: f64 = 1.0;
const W_ROBUST: f64 = 0.45;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Gap {
    /// A pool é quase toda física; falta uma opção mágica.
    Magico,
    /// E o contrário.
    Fisico,
    /// Nenhum dos seus aguenta a linha de frente.
    Frontline,
    /// Todos corpo a corpo.
    Alcance,
}

impl Gap {
    pub fn texto(self) -> &'static str {
        match self {
            Gap::Magico => "sua pool é quase toda de dano físico",
            Gap::Fisico => "sua pool é quase toda de dano mágico",
            Gap::Frontline => "nenhum dos seus segura a linha de frente",
            Gap::Alcance => "todos os seus são corpo a corpo",
        }
    }

    /// A frase que explica por que *este* campeão entra.
    pub fn preenche(self) -> &'static str {
        match self {
            Gap::Magico => "traz o dano mágico que falta",
            Gap::Fisico => "traz o dano físico que falta",
            Gap::Frontline => "é o frontline que a pool não tem",
            Gap::Alcance => "é a opção à distância que falta",
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Candidate {
    pub champion_id: i64,
    pub name: String,
    /// Quanto ele resolve, de 0 a 1. É um índice de encaixe, não uma winrate.
    pub score: f64,
    pub fills: Vec<Gap>,
    pub reasons: Vec<String>,
}

/// Vida efetiva contra uma mistura de dano, sem itens.
///
/// É a conta do jogo: dano que passa é `100 / (100 + resistência)`.
fn ehp_against(champion: &Champion, physical_share: f64) -> f64 {
    let hp = champion.base.health_at(LEVEL);
    let armor = champion.base.armor_at(LEVEL);
    let mr = champion.base.magic_resist_at(LEVEL);

    let taken =
        physical_share * damage_multiplier(armor) + (1.0 - physical_share) * damage_multiplier(mr);
    if taken <= 0.0 {
        return f64::INFINITY;
    }
    hp / taken
}

/// Quanto a vida efetiva dele varia conforme o dano que vem, de 0 a 1.
///
/// 1 significa que tanto faz enfrentar um time AD ou AP — as resistências dele
/// são equilibradas. Perto de 0 significa que ele é ótimo contra um tipo e
/// frágil contra o outro, e portanto fácil de anular escolhendo o outro.
///
/// Isto **não** é "quantas vezes ele perde para o counter". É a sensibilidade
/// dele ao tipo de dano, calculada com a fórmula de resistência do jogo.
pub fn robustness(champion: &Champion) -> f64 {
    let contra_ad = ehp_against(champion, 1.0);
    let contra_ap = ehp_against(champion, 0.0);
    if !contra_ad.is_finite() || !contra_ap.is_finite() {
        return 0.0;
    }
    let (menor, maior) = if contra_ad < contra_ap {
        (contra_ad, contra_ap)
    } else {
        (contra_ap, contra_ad)
    };
    if maior <= 0.0 {
        return 0.0;
    }
    menor / maior
}

/// Os buracos da pool.
///
/// Pool vazia não tem buraco: sem nada escolhido não há o que faltar, e
/// apontar "falta dano mágico" para quem não escolheu ninguém seria ruído.
pub fn gaps(pool: &[i64], champions: &ChampionIndex) -> Vec<Gap> {
    let membros: Vec<&Champion> = pool.iter().filter_map(|id| champions.get(*id)).collect();
    if membros.is_empty() {
        return Vec::new();
    }

    let mut out = Vec::new();

    let fisico: f64 =
        membros.iter().map(|c| c.physical_share()).sum::<f64>() / membros.len() as f64;
    if fisico >= SKEW {
        out.push(Gap::Magico);
    } else if fisico <= 1.0 - SKEW {
        out.push(Gap::Fisico);
    }

    if !membros.iter().any(|c| c.is_frontline()) {
        out.push(Gap::Frontline);
    }
    if !membros.iter().any(|c| c.is_ranged()) {
        out.push(Gap::Alcance);
    }

    out
}

fn fills(champion: &Champion, gaps: &[Gap]) -> Vec<Gap> {
    gaps.iter()
        .copied()
        .filter(|gap| match gap {
            Gap::Magico => champion.physical_share() <= 0.35,
            Gap::Fisico => champion.physical_share() >= 0.65,
            Gap::Frontline => champion.is_frontline(),
            Gap::Alcance => champion.is_ranged(),
        })
        .collect()
}

/// Candidatos para o último lugar da pool.
///
/// Ordenados por quanto resolvem. Quem já está na pool fica de fora, e com a
/// pool vazia não há recomendação — não há buraco para preencher, e sugerir
/// "os melhores" sem dado de partidas seria inventar.
pub fn suggest(
    pool: &[i64],
    champions: &ChampionIndex,
    limit: usize,
) -> Vec<Candidate> {
    let buracos = gaps(pool, champions);
    if buracos.is_empty() {
        return Vec::new();
    }

    let mut candidatos: Vec<Candidate> = champions
        .by_id
        .values()
        .filter(|c| !pool.contains(&c.id))
        .map(|champion| {
            let preenchidos = fills(champion, &buracos);
            let robusto = robustness(champion);

            let cobertura = preenchidos.len() as f64 / buracos.len() as f64;
            let score = (W_GAP * cobertura + W_ROBUST * robusto) / (W_GAP + W_ROBUST);

            let mut reasons: Vec<String> = preenchidos
                .iter()
                .map(|g| g.preenche().to_string())
                .collect();
            if robusto >= 0.55 {
                reasons.push("aguenta dano físico e mágico parecido".into());
            }

            Candidate {
                champion_id: champion.id,
                name: champion.name.clone(),
                score,
                fills: preenchidos,
                reasons,
            }
        })
        // Quem não preenche nada não é candidato: a pergunta é o que falta.
        .filter(|c| !c.fills.is_empty())
        .collect();

    candidatos.sort_by(|a, b| {
        b.score
            .total_cmp(&a.score)
            .then_with(|| a.name.cmp(&b.name))
    });
    candidatos.truncate(limit);
    candidatos
}

/// Rótulo da rota, para a interface.
pub fn role_label(role: Role) -> &'static str {
    role.short()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::champion::{fake, BaseStats};
    use std::collections::HashMap;

    fn com_base(mut c: Champion, hp: f64, armor: f64, mr: f64) -> Champion {
        c.base = BaseStats {
            health: hp,
            health_per_level: 100.0,
            armor,
            armor_per_level: 4.0,
            magic_resist: mr,
            magic_resist_per_level: 2.0,
            attack_damage: 60.0,
            attack_damage_per_level: 3.0,
        };
        c
    }

    fn indice() -> ChampionIndex {
        let mut by_id = HashMap::new();
        // 1-3: magos à distância, frágeis.
        for id in 1..=3 {
            by_id.insert(id, fake(id, &format!("Mago{id}"), "Mage", 2.0, 9.0, 3.0));
        }
        // 11-13: marksmen à distância.
        for id in 11..=13 {
            by_id.insert(id, fake(id, &format!("Atirador{id}"), "Marksman", 9.0, 2.0, 3.0));
        }
        // 21-23: lutadores corpo a corpo.
        for id in 21..=23 {
            by_id.insert(id, fake(id, &format!("Lutador{id}"), "Fighter", 8.0, 3.0, 5.0));
        }
        // 31: tanque corpo a corpo, resistências equilibradas.
        by_id.insert(
            31,
            com_base(fake(31, "Tanque", "Tank", 5.0, 6.0, 9.0), 650.0, 40.0, 40.0),
        );
        // 32: tanque só com armadura — forte contra AD, frágil contra AP.
        by_id.insert(
            32,
            com_base(fake(32, "SoArmadura", "Tank", 5.0, 6.0, 9.0), 650.0, 80.0, 20.0),
        );
        // 33: tanque de dano físico — cobre frontline e dano físico de uma vez.
        by_id.insert(
            33,
            com_base(fake(33, "TanqueAD", "Tank", 9.0, 2.0, 9.0), 650.0, 40.0, 40.0),
        );

        ChampionIndex {
            version: "test".into(),
            by_id,
        }
    }

    #[test]
    fn pool_vazia_nao_tem_buraco() {
        assert!(gaps(&[], &indice()).is_empty());
        assert!(suggest(&[], &indice(), 10).is_empty());
    }

    #[test]
    fn pool_toda_magica_pede_dano_fisico() {
        let buracos = gaps(&[1, 2, 3], &indice());
        assert!(buracos.contains(&Gap::Fisico), "{buracos:?}");
        assert!(!buracos.contains(&Gap::Magico));
    }

    #[test]
    fn pool_toda_fisica_pede_dano_magico() {
        let buracos = gaps(&[11, 12, 13], &indice());
        assert!(buracos.contains(&Gap::Magico), "{buracos:?}");
    }

    #[test]
    fn pool_misturada_nao_reclama_de_dano() {
        let buracos = gaps(&[1, 11, 21], &indice());
        assert!(!buracos.contains(&Gap::Magico));
        assert!(!buracos.contains(&Gap::Fisico));
    }

    #[test]
    fn pool_sem_tanque_pede_frontline() {
        assert!(gaps(&[1, 11], &indice()).contains(&Gap::Frontline));
        assert!(!gaps(&[1, 31], &indice()).contains(&Gap::Frontline));
    }

    #[test]
    fn pool_toda_corpo_a_corpo_pede_alcance() {
        assert!(gaps(&[21, 22, 31], &indice()).contains(&Gap::Alcance));
        assert!(!gaps(&[21, 11], &indice()).contains(&Gap::Alcance));
    }

    #[test]
    fn a_robustez_premia_resistencias_equilibradas() {
        let idx = indice();
        let equilibrado = robustness(idx.get(31).unwrap());
        let so_armadura = robustness(idx.get(32).unwrap());

        assert!(
            equilibrado > so_armadura,
            "equilibrado {equilibrado} deveria passar de {so_armadura}"
        );
        assert!(equilibrado <= 1.0 && so_armadura >= 0.0);
    }

    #[test]
    fn quem_nao_preenche_nada_fica_de_fora() {
        // Pool toda mágica e corpo a corpo: o buraco é dano físico.
        let sugestoes = suggest(&[1, 2, 3], &indice(), 10);

        assert!(!sugestoes.is_empty());
        for c in &sugestoes {
            assert!(!c.fills.is_empty(), "{} não preenche nada", c.name);
        }
    }

    #[test]
    fn quem_ja_esta_na_pool_nao_e_sugerido() {
        let pool = [1, 2, 3];
        for c in suggest(&pool, &indice(), 10) {
            assert!(!pool.contains(&c.champion_id), "{} já está na pool", c.name);
        }
    }

    #[test]
    fn quem_preenche_mais_buracos_vem_antes() {
        // Pool de magos à distância: faltam dano físico e frontline. O tanque
        // de dano físico cobre os dois, e tem de vir na frente de quem cobre
        // só um.
        let buracos = gaps(&[1, 2, 3], &indice());
        assert_eq!(buracos.len(), 2, "{buracos:?}");

        let sugestoes = suggest(&[1, 2, 3], &indice(), 10);
        let primeiro = &sugestoes[0];

        assert_eq!(primeiro.champion_id, 33, "{primeiro:?}");
        assert_eq!(primeiro.fills.len(), 2);

        let so_um = sugestoes.iter().find(|c| c.fills.len() == 1).unwrap();
        assert!(primeiro.score > so_um.score);
    }

    #[test]
    fn cobrir_dois_buracos_vale_mais_que_ser_robusto() {
        // O risco do jeito que a conta é montada: um campeão muito equilibrado
        // nas resistências passar na frente de quem realmente tapa o buraco.
        let sugestoes = suggest(&[1, 2, 3], &indice(), 20);
        let dois = sugestoes.iter().find(|c| c.fills.len() == 2).unwrap();
        let mais_robusto_de_um = sugestoes
            .iter()
            .filter(|c| c.fills.len() == 1)
            .max_by(|a, b| a.score.total_cmp(&b.score))
            .unwrap();

        assert!(
            dois.score > mais_robusto_de_um.score,
            "{} ({}) deveria passar de {} ({})",
            dois.name,
            dois.score,
            mais_robusto_de_um.name,
            mais_robusto_de_um.score
        );
    }

    #[test]
    fn a_lista_sai_ordenada_do_maior_para_o_menor() {
        let sugestoes = suggest(&[1, 2, 3], &indice(), 10);
        for par in sugestoes.windows(2) {
            assert!(par[0].score >= par[1].score);
        }
    }

    #[test]
    fn o_limite_e_respeitado() {
        assert!(suggest(&[1, 2, 3], &indice(), 2).len() <= 2);
        assert!(suggest(&[1, 2, 3], &indice(), 100).len() <= 100);
    }

    #[test]
    fn o_score_fica_entre_zero_e_um() {
        for c in suggest(&[1, 2, 3], &indice(), 20) {
            assert!(
                (0.0..=1.0).contains(&c.score),
                "{} saiu com {}",
                c.name,
                c.score
            );
        }
    }

    #[test]
    fn toda_sugestao_vem_com_motivo_escrito() {
        // Número sem frase é número que ninguém confere.
        for c in suggest(&[1, 2, 3], &indice(), 10) {
            assert!(!c.reasons.is_empty(), "{} veio sem motivo", c.name);
        }
    }

    #[test]
    fn campeao_fora_do_indice_nao_derruba_a_conta() {
        let buracos = gaps(&[1, 999], &indice());
        assert!(!buracos.is_empty());
    }

    #[test]
    fn pool_completa_e_equilibrada_nao_recomenda_nada() {
        // Mago, atirador, tanque: dano misto, frontline e alcance cobertos.
        assert!(gaps(&[1, 11, 31], &indice()).is_empty());
        assert!(suggest(&[1, 11, 31], &indice(), 10).is_empty());
    }
}
