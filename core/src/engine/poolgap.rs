//! A pool de oito: três que você escolhe, cinco para cobrir o que eles não
//! cobrem.
//!
//! Os três primeiros dizem como você joga a rota. Deles sai o retrato — pontos
//! fortes e fracos — e, um por vez, os cinco que completam a pool: cada um
//! escolhido pelo que os anteriores ainda não cobrem, e recalculado depois de
//! cada escolha. Cinco recomendações calculadas de uma vez sairiam todas para o
//! mesmo buraco.
//!
//! **De onde saem estes números.** Daqui não sai winrate. Sai a composição da
//! pool — tipo de dano, linha de frente, alcance, estilo de jogo — e a
//! matemática de resistências do jogo. O que conta em cada rota, e quanto, são
//! pesos escritos à mão: regra, não estatística, e a interface diz isso. Com a
//! base de partidas entra o que esta conta não vê: contra quem os seus três
//! perdem, e quem ganha desses.

use serde::Serialize;

use super::champion::{Champion, ChampionIndex, Role};
use super::damage::damage_multiplier;
use super::roles::plays;
use super::stats::StatsStore;

/// O tamanho da pool completa, e quantos dela você escolhe.
pub const POOL_SIZE: usize = 8;
pub const CORE_SIZE: usize = 3;

/// Nível de referência para comparar campeões entre si.
const LEVEL: u8 = 11;

/// Acima disto de robustez, as resistências contam como equilibradas.
const ROBUST: f64 = 0.55;

/// Abaixo disto de peso, a característica não vira ponto forte nem fraco na
/// rota — só desempata a escolha.
const RELEVANT: f64 = 0.5;

const MAX_REASONS: usize = 3;

/// Uma característica que um campeão tem ou não tem, e que a pool cobre ou não.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Trait {
    Fisico,
    Magico,
    Frontline,
    Alcance,
    CorpoACorpo,
    /// Vida efetiva parecida contra dano físico e mágico.
    Robusto,
    // Os estilos são as classes do Data Dragon.
    Lutador,
    Tanque,
    Mago,
    Assassino,
    Atirador,
    Suporte,
}

impl Trait {
    pub const ALL: [Trait; 12] = [
        Trait::Fisico,
        Trait::Magico,
        Trait::Frontline,
        Trait::Alcance,
        Trait::CorpoACorpo,
        Trait::Robusto,
        Trait::Lutador,
        Trait::Tanque,
        Trait::Mago,
        Trait::Assassino,
        Trait::Atirador,
        Trait::Suporte,
    ];

    /// A classe do Data Dragon, para os estilos.
    fn tag(self) -> Option<&'static str> {
        match self {
            Trait::Lutador => Some("Fighter"),
            Trait::Tanque => Some("Tank"),
            Trait::Mago => Some("Mage"),
            Trait::Assassino => Some("Assassin"),
            Trait::Atirador => Some("Marksman"),
            Trait::Suporte => Some("Support"),
            _ => None,
        }
    }

    fn is_style(self) -> bool {
        self.tag().is_some()
    }

    pub fn has(self, champion: &Champion) -> bool {
        match self {
            Trait::Fisico => champion.physical_share() >= 0.65,
            Trait::Magico => champion.physical_share() <= 0.35,
            Trait::Frontline => champion.is_frontline(),
            Trait::Alcance => champion.is_ranged(),
            Trait::CorpoACorpo => !champion.is_ranged(),
            Trait::Robusto => robustness(champion) >= ROBUST,
            style => champion.tags.iter().any(|t| Some(t.as_str()) == style.tag()),
        }
    }

    fn noun(self) -> &'static str {
        match self {
            Trait::Fisico => "dano físico",
            Trait::Magico => "dano mágico",
            Trait::Frontline => "linha de frente",
            Trait::Alcance => "alcance",
            Trait::CorpoACorpo => "corpo a corpo",
            Trait::Robusto => "resistências equilibradas",
            Trait::Lutador => "lutador",
            Trait::Tanque => "tanque",
            Trait::Mago => "mago",
            Trait::Assassino => "assassino",
            Trait::Atirador => "atirador",
            Trait::Suporte => "suporte",
        }
    }

    fn plural(self) -> &'static str {
        match self {
            Trait::Lutador => "lutadores",
            Trait::Tanque => "tanques",
            Trait::Mago => "magos",
            Trait::Assassino => "assassinos",
            Trait::Atirador => "atiradores",
            Trait::Suporte => "suportes",
            other => other.noun(),
        }
    }

    fn strength(self, count: usize, total: usize) -> String {
        let n = format!("{count} de {total}");
        let um = count == 1;
        match self {
            Trait::Fisico => format!("{n} com dano físico"),
            Trait::Magico => format!("{n} com dano mágico"),
            Trait::Frontline => format!("{n} {} a linha de frente", if um { "segura" } else { "seguram" }),
            Trait::Alcance => format!("{n} {} à distância", if um { "ataca" } else { "atacam" }),
            Trait::CorpoACorpo => format!("{n} corpo a corpo"),
            Trait::Robusto => format!(
                "{n} {} dano físico e mágico parecido",
                if um { "aguenta" } else { "aguentam" }
            ),
            style => format!("{n} {}", if um { style.noun() } else { style.plural() }),
        }
    }

    fn weakness(self) -> String {
        match self {
            Trait::Fisico => "nenhum com dano físico: o inimigo empilha resistência mágica".into(),
            Trait::Magico => "nenhum com dano mágico: o inimigo empilha armadura".into(),
            Trait::Frontline => "nenhum segura a linha de frente".into(),
            Trait::Alcance => "nenhum ataca à distância: quem tem alcance troca de graça na rota".into(),
            Trait::CorpoACorpo => "nenhum corpo a corpo".into(),
            Trait::Robusto => "todos frágeis contra um dos dois tipos de dano".into(),
            style => format!("nenhum {}", style.noun()),
        }
    }

    /// Por que um candidato entra, dado quantos da pool já têm isto.
    fn reason(self, count: usize) -> String {
        match (self, count) {
            (Trait::Fisico, 0) => "traz o dano físico que falta".into(),
            (Trait::Magico, 0) => "traz o dano mágico que falta".into(),
            (Trait::Frontline, 0) => "é a linha de frente que a pool não tem".into(),
            (Trait::Alcance, 0) => "é a opção à distância que falta".into(),
            (Trait::CorpoACorpo, 0) => "é a opção corpo a corpo que falta".into(),
            (Trait::Robusto, 0) => "aguenta dano físico e mágico parecido".into(),
            (style, 0) => format!("traz um estilo que a pool não tem: {}", style.noun()),
            (t, 1) => format!("segunda opção de {}", t.noun()),
            (t, _) => format!("reforça {}", t.noun()),
        }
    }
}

/// Quanto cada característica conta na rota. Escritos à mão.
///
/// O que não existe na rota pesa zero: todo atirador tem alcance e nenhum
/// segura a frente, então cobrar isso da pool de atirador seria ruído.
fn weight(role: Role, t: Trait) -> f64 {
    use Trait::*;
    match (role, t) {
        (Role::Bot, Frontline | Alcance | CorpoACorpo) => 0.0,
        (Role::Bot, Fisico | Magico) => 0.6,
        (Role::Mid, Frontline) => 0.4,
        (Role::Jungle, Alcance) => 0.3,
        (Role::Support, Fisico | Magico) => 0.5,
        (_, Fisico | Magico | Frontline) => 1.0,
        (_, Alcance) => 0.6,
        (_, CorpoACorpo) => 0.4,
        (_, Robusto) => 0.5,
        // Estilo diversifica a pool, mas falta de estilo não é fraqueza.
        (_, _) => 0.35,
    }
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

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Finding {
    #[serde(rename = "trait")]
    pub kind: Trait,
    pub text: String,
}

/// O retrato dos três primeiros.
#[derive(Debug, Clone, Default, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Analysis {
    pub strengths: Vec<Finding>,
    pub weaknesses: Vec<Finding>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Candidate {
    pub champion_id: i64,
    pub name: String,
    /// Quanto ele cobre do que falta, de 0 a 1. É um índice de encaixe, não uma
    /// winrate.
    pub score: f64,
    /// O que ninguém da pool tinha, e ele traz.
    pub fills: Vec<Trait>,
    pub reasons: Vec<String>,
}

/// Pontos fortes e fracos dos campeões que você escolheu.
///
/// Forte é o que a rota valoriza e ao menos um deles tem; fraco é o que a rota
/// valoriza e nenhum tem, ou os três terem o mesmo estilo — a mesma resposta
/// serve contra todos. Sem campeão, sem retrato.
pub fn analyze(core: &[i64], champions: &ChampionIndex, role: Role) -> Analysis {
    let membros: Vec<&Champion> = core.iter().filter_map(|id| champions.get(*id)).collect();
    let total = membros.len();
    if total == 0 {
        return Analysis::default();
    }

    let mut relevantes: Vec<(Trait, usize)> = Trait::ALL
        .iter()
        .filter(|t| !t.is_style() && weight(role, **t) >= RELEVANT)
        .map(|&t| (t, membros.iter().filter(|c| t.has(c)).count()))
        .collect();
    relevantes.sort_by(|a, b| {
        weight(role, b.0)
            .total_cmp(&weight(role, a.0))
            .then(b.1.cmp(&a.1))
    });

    let strengths = relevantes
        .iter()
        .filter(|(_, n)| *n > 0)
        .map(|&(kind, n)| Finding {
            kind,
            text: kind.strength(n, total),
        })
        .collect();

    let mut weaknesses: Vec<Finding> = relevantes
        .iter()
        .filter(|(_, n)| *n == 0)
        .map(|&(kind, _)| Finding {
            kind,
            text: kind.weakness(),
        })
        .collect();

    if total >= 2 {
        let estilo = membros[0].primary_tag();
        let mesmo = membros.iter().all(|c| c.primary_tag() == estilo);
        if let Some(kind) = Trait::ALL.iter().find(|t| t.tag() == Some(estilo)) {
            if mesmo {
                weaknesses.push(Finding {
                    kind: *kind,
                    text: format!(
                        "os {total} são {}: um estilo só, e a mesma resposta serve contra todos",
                        kind.plural()
                    ),
                });
            }
        }
    }

    Analysis {
        strengths,
        weaknesses,
    }
}

/// Candidatos para o próximo lugar da pool.
///
/// A cobertura de cada característica é `1 − ½ⁿ`, com n quantos da pool a têm:
/// a primeira vale metade, a segunda um quarto, e assim por diante. O candidato
/// vale o quanto sobe essa cobertura, `Σ peso × ½ⁿ⁺¹` sobre o que ele tem — o
/// buraco maior primeiro, o segundo exemplar de algo raro depois, o reforço do
/// que já sobra por último.
///
/// Pool vazia não tem buraco, e pool cheia não tem lugar: nos dois casos não há
/// recomendação.
pub fn suggest(
    pool: &[i64],
    champions: &ChampionIndex,
    role: Role,
    store: &dyn StatsStore,
    limit: usize,
) -> Vec<Candidate> {
    let membros: Vec<&Champion> = pool.iter().filter_map(|id| champions.get(*id)).collect();
    if membros.is_empty() || pool.len() >= POOL_SIZE {
        return Vec::new();
    }

    let conta = |t: Trait| membros.iter().filter(|c| t.has(c)).count();
    let contagens: Vec<(Trait, usize)> = Trait::ALL.iter().map(|&t| (t, conta(t))).collect();
    let marginal = |t: Trait, n: usize| weight(role, t) * 0.5f64.powi(n as i32 + 1);
    let teto: f64 = contagens.iter().map(|&(t, n)| marginal(t, n)).sum();

    let mut candidatos: Vec<Candidate> = champions
        .by_id
        .values()
        .filter(|c| !pool.contains(&c.id) && plays(c, role, store))
        .filter_map(|champion| {
            let mut ganhos: Vec<(Trait, usize, f64)> = contagens
                .iter()
                .filter(|(t, _)| t.has(champion))
                .map(|&(t, n)| (t, n, marginal(t, n)))
                .filter(|(_, _, g)| *g > 0.0)
                .collect();
            let ganho: f64 = ganhos.iter().map(|(_, _, g)| g).sum();
            if ganho <= 0.0 {
                return None;
            }
            ganhos.sort_by(|a, b| b.2.total_cmp(&a.2).then(a.0.cmp(&b.0)));

            let reasons = ganhos
                .iter()
                .take(MAX_REASONS)
                .map(|&(t, n, _)| t.reason(n))
                .collect();

            Some(Candidate {
                champion_id: champion.id,
                name: champion.name.clone(),
                score: (ganho / teto).clamp(0.0, 1.0),
                fills: ganhos
                    .iter()
                    .filter(|(_, n, _)| *n == 0)
                    .map(|(t, _, _)| *t)
                    .collect(),
                reasons,
            })
        })
        .collect();

    candidatos.sort_by(|a, b| b.score.total_cmp(&a.score).then_with(|| a.name.cmp(&b.name)));
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
    use crate::engine::stats::NoStats;
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

    fn com_tags(mut c: Champion, tags: &[&str]) -> Champion {
        c.tags = tags.iter().map(|t| t.to_string()).collect();
        c
    }

    fn indice() -> ChampionIndex {
        let mut by_id = HashMap::new();
        // 1-3: magos à distância, frágeis.
        for id in 1..=3 {
            by_id.insert(id, fake(id, &format!("Mago{id}"), "Mage", 2.0, 9.0, 3.0));
        }
        // 4: assassino mágico corpo a corpo.
        by_id.insert(
            4,
            com_tags(fake(4, "AssassinoAP", "Assassin", 3.0, 8.0, 3.0), &["Assassin", "Mage"]),
        );
        // 5-6: assassinos físicos corpo a corpo, de meio.
        for id in 5..=6 {
            by_id.insert(id, fake(id, &format!("Assassino{id}"), "Assassin", 9.0, 2.0, 3.0));
        }
        // 11-13: atiradores.
        for id in 11..=13 {
            by_id.insert(id, fake(id, &format!("Atirador{id}"), "Marksman", 9.0, 2.0, 3.0));
        }
        // 14: atirador de dano mágico. A classe principal é Mage, como nos
        // campeões reais desse tipo — com Marksman na frente, o prior de classe
        // puxaria o dano para físico.
        by_id.insert(
            14,
            com_tags(fake(14, "AtiradorAP", "Mage", 2.0, 9.0, 3.0), &["Mage", "Marksman"]),
        );
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
        // 33: tanque de dano mágico que também é mago.
        by_id.insert(
            33,
            com_tags(
                com_base(fake(33, "TanqueAP", "Tank", 2.0, 9.0, 9.0), 650.0, 40.0, 40.0),
                &["Tank", "Mage"],
            ),
        );
        // 41: suporte.
        by_id.insert(41, fake(41, "Suporte", "Support", 3.0, 7.0, 5.0));

        ChampionIndex {
            version: "test".into(),
            by_id,
        }
    }

    fn ids(c: &[Candidate]) -> Vec<i64> {
        c.iter().map(|c| c.champion_id).collect()
    }

    #[test]
    fn pool_vazia_nao_tem_retrato_nem_sugestao() {
        let a = analyze(&[], &indice(), Role::Mid);
        assert!(a.strengths.is_empty() && a.weaknesses.is_empty());
        assert!(suggest(&[], &indice(), Role::Mid, &NoStats, 10).is_empty());
    }

    #[test]
    fn tres_magos_mostram_o_que_tem_e_o_que_falta() {
        let a = analyze(&[1, 2, 3], &indice(), Role::Top);
        let forte: Vec<Trait> = a.strengths.iter().map(|f| f.kind).collect();
        let fraco: Vec<Trait> = a.weaknesses.iter().map(|f| f.kind).collect();

        assert!(forte.contains(&Trait::Magico), "{a:?}");
        assert!(forte.contains(&Trait::Alcance), "{a:?}");
        assert!(fraco.contains(&Trait::Fisico), "{a:?}");
        assert!(fraco.contains(&Trait::Frontline), "{a:?}");
        assert!(
            a.weaknesses.iter().any(|f| f.text.contains("um estilo só")),
            "três magos é um estilo só: {a:?}"
        );
    }

    #[test]
    fn o_que_a_rota_nao_cobra_nao_vira_fraqueza() {
        // Três atiradores: nenhum segura a frente, e isso é a rota, não defeito.
        let a = analyze(&[11, 12, 13], &indice(), Role::Bot);
        assert!(!a.weaknesses.iter().any(|f| f.kind == Trait::Frontline), "{a:?}");
        assert!(!a.strengths.iter().any(|f| f.kind == Trait::Alcance), "{a:?}");
    }

    #[test]
    fn so_entra_quem_joga_a_rota() {
        // A pool de atirador nunca recebe tanque, por mais que falte frente.
        for c in suggest(&[11, 12, 13], &indice(), Role::Bot, &NoStats, 50) {
            let campeao = indice().get(c.champion_id).unwrap().clone();
            assert!(
                campeao.tags.iter().any(|t| t == "Marksman"),
                "{} não joga de atirador",
                c.name
            );
        }
    }

    #[test]
    fn o_primeiro_tapa_o_buraco_maior() {
        // Três magos no topo: faltam dano físico e frente. Quem cobre os dois
        // vem antes de quem cobre um.
        let sugestoes = suggest(&[1, 2, 3], &indice(), Role::Top, &NoStats, 20);
        let primeiro = &sugestoes[0];
        assert!(primeiro.fills.contains(&Trait::Frontline), "{primeiro:?}");
        assert!(primeiro.fills.len() >= 2, "{primeiro:?}");
    }

    #[test]
    fn cada_escolha_muda_a_proxima() {
        // Depois que um tanque entra, a frente deixa de ser buraco e o próximo
        // sai para outra coisa. É isso que impede cinco respostas iguais.
        let antes = suggest(&[1, 2, 3], &indice(), Role::Top, &NoStats, 20);
        let depois = suggest(&[1, 2, 3, 31], &indice(), Role::Top, &NoStats, 20);

        let frente = |c: &Candidate| c.fills.contains(&Trait::Frontline);
        assert!(frente(&antes[0]));
        assert!(!depois.iter().any(frente), "a frente já está coberta: {depois:?}");
        assert_ne!(antes[0].champion_id, depois[0].champion_id);
    }

    #[test]
    fn pool_equilibrada_ainda_recebe_sugestao_ate_oito() {
        // Mago, lutador e tanque cobrem dano, frente e corpo a corpo — mas a
        // pool tem três, e o passo é chegar a oito.
        let sugestoes = suggest(&[1, 21, 31], &indice(), Role::Top, &NoStats, 10);
        assert!(!sugestoes.is_empty());
    }

    #[test]
    fn pool_de_oito_esta_completa() {
        let oito = [1, 2, 3, 21, 22, 23, 31, 32];
        assert!(suggest(&oito, &indice(), Role::Top, &NoStats, 10).is_empty());
    }

    #[test]
    fn quem_ja_esta_na_pool_nao_e_sugerido() {
        let pool = [21, 22, 31];
        for c in suggest(&pool, &indice(), Role::Top, &NoStats, 50) {
            assert!(!pool.contains(&c.champion_id), "{} já está na pool", c.name);
        }
    }

    #[test]
    fn segunda_copia_vale_menos_que_a_primeira() {
        // Pool de atirador no bot: o atirador de dano mágico traz o que falta e
        // vem antes dos atiradores físicos, que só reforçam.
        let sugestoes = suggest(&[11, 12], &indice(), Role::Bot, &NoStats, 10);
        assert_eq!(sugestoes[0].champion_id, 14, "{:?}", ids(&sugestoes));
        assert!(sugestoes[0].fills.contains(&Trait::Magico));
    }

    #[test]
    fn a_robustez_premia_resistencias_equilibradas() {
        let idx = indice();
        let equilibrado = robustness(idx.get(31).unwrap());
        let so_armadura = robustness(idx.get(32).unwrap());
        assert!(equilibrado > so_armadura, "{equilibrado} vs {so_armadura}");
        assert!(equilibrado <= 1.0 && so_armadura >= 0.0);
    }

    #[test]
    fn a_lista_sai_ordenada_limitada_e_com_motivo() {
        let sugestoes = suggest(&[1, 2, 3], &indice(), Role::Top, &NoStats, 4);
        assert!(sugestoes.len() <= 4);
        for par in sugestoes.windows(2) {
            assert!(par[0].score >= par[1].score);
        }
        for c in &sugestoes {
            assert!((0.0..=1.0).contains(&c.score), "{} saiu com {}", c.name, c.score);
            // Número sem frase é número que ninguém confere.
            assert!(!c.reasons.is_empty(), "{} veio sem motivo", c.name);
        }
    }

    #[test]
    fn campeao_fora_do_indice_nao_derruba_a_conta() {
        assert!(!analyze(&[1, 999], &indice(), Role::Mid).strengths.is_empty());
        let _ = suggest(&[1, 999], &indice(), Role::Mid, &NoStats, 5);
    }
}
