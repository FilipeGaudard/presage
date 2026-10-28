//! Condição de vitória: o que a composição dos dois times diz, e onde a conta
//! de atributos favorece um lado.
//!
//! O que fica de fora importa mais que o que entra. A conta de duelo usa
//! atributos e itens — não vê habilidade, e por isso não vê quem escala: um
//! campeão que cresce por acúmulo ou pela suprema parece fraco no fim por esta
//! conta. Dizer "seu time é de fim de jogo" a partir dela poderia sair ao
//! contrário. A fase do jogo só entra com a base de partidas, medida pela
//! winrate em partidas curtas e longas; até lá, não há linha de fase.
//!
//! Tudo aqui é descrição, nunca ordem: a política da Riot não aceita app que
//! decide pelo jogador (C-5). "Os três de vocês atacam à distância" é fato;
//! o que fazer com ele é do jogador.

use serde::Serialize;

use super::champion::ChampionIndex;
use super::comp;
use super::lanes::{LaneOutlook, Lineup, Skirmish};
use super::stats::{logit, shrink, Length, StatsStore};
use super::BASE_STRENGTH;

/// Acima disto de dano de um tipo, o time é de um tipo só.
const ONE_SIDED: f64 = 0.70;

/// Abaixo disto de distância de 50%, a conta de atributos não diz nada.
const EDGE: f64 = 0.05;

/// Times com menos que isto definidos ainda não têm composição para ler.
const MIN_KNOWN: usize = 3;

const MAX_EDGES: usize = 5;

/// Acima disto de diferença de inclinação, em log-odds (~2 pontos), um time
/// pende para partida curta ou longa.
const LEAN: f64 = 0.08;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Side {
    Nosso,
    Deles,
}

/// De onde a afirmação sai, para a interface mostrar ao lado dela.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Source {
    /// Regra escrita à mão sobre a composição.
    Regra,
    /// A conta de duelo: atributos e itens, sem habilidades.
    Atributos,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Point {
    pub side: Side,
    pub source: Source,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WinCondition {
    /// O que a composição de cada time é.
    pub facts: Vec<Point>,
    /// Onde a conta de atributos pende para um lado.
    pub edges: Vec<Point>,
    /// A fase do jogo em que o time rende mais. Só com a base de partidas.
    pub phase: Option<String>,
    /// O que nada disto vê. Sempre presente.
    pub caveat: &'static str,
}

const CAVEAT: &str = "Composição por regra e duelos pela conta de atributos e itens, \
sem habilidades: quem escala pela suprema ou por acúmulo não aparece aqui. A fase do \
jogo só entra com a base de partidas.";

fn facts_of(ids: &[i64], side: Side, index: &ChampionIndex) -> Vec<Point> {
    let perfil = comp::profile(ids, index);
    if perfil.known < MIN_KNOWN {
        return Vec::new();
    }
    let quem = match side {
        Side::Nosso => "Vocês",
        Side::Deles => "Eles",
    };
    let mut out = Vec::new();
    let ponto = |text: String| Point {
        side,
        source: Source::Regra,
        text,
    };

    let fisico = (perfil.physical_share * 100.0).round();
    if perfil.physical_share >= ONE_SIDED {
        out.push(ponto(format!(
            "{quem}: {fisico:.0}% do dano é físico — armadura responde a quase tudo"
        )));
    } else if perfil.physical_share <= 1.0 - ONE_SIDED {
        out.push(ponto(format!(
            "{quem}: {:.0}% do dano é mágico — resistência mágica responde a quase tudo",
            100.0 - fisico
        )));
    }

    if perfil.frontline == 0 {
        out.push(ponto(format!("{quem}: ninguém segura a linha de frente")));
    } else if perfil.frontline >= 3 {
        out.push(ponto(format!(
            "{quem}: {} de {} seguram a linha de frente",
            perfil.frontline, perfil.known
        )));
    }

    let corpo = perfil.known - perfil.ranged;
    if perfil.ranged >= 4 {
        out.push(ponto(format!(
            "{quem}: {} de {} atacam à distância",
            perfil.ranged, perfil.known
        )));
    } else if corpo >= 4 {
        out.push(ponto(format!(
            "{quem}: {} de {} são corpo a corpo e precisam chegar perto",
            corpo, perfil.known
        )));
    }

    out
}

/// A fase pelos dados: a winrate de cada um, na rota dele, em partidas longas
/// contra curtas — encolhida para a força dele na rota, para pouca amostra não
/// virar inclinação. A diferença entre os dois times é a fase.
fn phase_from_data(ours: &[Lineup], theirs: &[Lineup], store: &dyn StatsStore) -> Option<String> {
    if !store.has_global() {
        return None;
    }
    let inclinacao = |lado: &[Lineup]| -> Option<(f64, u32)> {
        let (mut soma, mut quantos, mut jogos) = (0.0, 0, 0);
        for l in lado {
            let (id, rota) = (l.combatant.champion_id, l.role);
            let curta = store.length(id, rota, Length::Curta);
            let longa = store.length(id, rota, Length::Longa);
            if curta.is_none() && longa.is_none() {
                continue;
            }
            let base = shrink(store.base(id, rota), 0.5, BASE_STRENGTH).rate;
            let pc = shrink(curta, base, BASE_STRENGTH).rate;
            let pl = shrink(longa, base, BASE_STRENGTH).rate;
            soma += logit(pl) - logit(pc);
            quantos += 1;
            jogos += curta.map_or(0, |r| r.games) + longa.map_or(0, |r| r.games);
        }
        (quantos > 0).then(|| (soma / quantos as f64, jogos))
    };
    let (nossa, n_nossa) = inclinacao(ours)?;
    let (deles, n_deles) = inclinacao(theirs).unwrap_or((0.0, 0));
    let n = n_nossa + n_deles;
    let diferenca = nossa - deles;
    Some(if diferenca > LEAN {
        format!("Pelos dados, o seu time rende mais em partidas longas que o deles (n={n})")
    } else if diferenca < -LEAN {
        format!("Pelos dados, o seu time rende mais em partidas curtas que o deles (n={n})")
    } else {
        format!("Pelos dados, nenhum dos dois times pende para partida curta ou longa (n={n})")
    })
}

fn pct(odds: f64) -> f64 {
    (odds * 100.0).round()
}

/// Lê a composição dos dois times e os duelos que a análise de rota já fez.
///
/// Time com menos de três definidos fica sem fatos: ler a composição de dois
/// campeões seria inventar o resto.
pub fn analyze(
    ours: &[Lineup],
    theirs: &[Lineup],
    lanes: &[LaneOutlook],
    skirmishes: &[Skirmish],
    index: &ChampionIndex,
    store: &dyn StatsStore,
) -> WinCondition {
    let nossos: Vec<i64> = ours.iter().map(|l| l.combatant.champion_id).collect();
    let deles: Vec<i64> = theirs.iter().map(|l| l.combatant.champion_id).collect();

    let mut facts = facts_of(&nossos, Side::Nosso, index);
    facts.extend(facts_of(&deles, Side::Deles, index));

    let mut edges: Vec<(f64, Point)> = Vec::new();
    for lane in lanes {
        let distancia = (lane.duel - 0.5).abs();
        if distancia < EDGE {
            continue;
        }
        let side = if lane.duel > 0.5 { Side::Nosso } else { Side::Deles };
        let (vence, perde) = match side {
            Side::Nosso => (&lane.ally_name, &lane.enemy_name),
            Side::Deles => (&lane.enemy_name, &lane.ally_name),
        };
        edges.push((
            distancia,
            Point {
                side,
                source: Source::Atributos,
                text: format!(
                    "{}: {vence} ganha o 1v1 de {perde} na conta de atributos ({:.0}%)",
                    lane.lane,
                    pct(lane.duel.max(1.0 - lane.duel))
                ),
            },
        ));
    }
    for briga in skirmishes {
        let distancia = (briga.odds - 0.5).abs();
        if distancia < EDGE {
            continue;
        }
        let side = if briga.odds > 0.5 { Side::Nosso } else { Side::Deles };
        let lado = match side {
            Side::Nosso => "vocês",
            Side::Deles => "eles",
        };
        edges.push((
            distancia,
            Point {
                side,
                source: Source::Atributos,
                text: format!(
                    "{} ({}): {lado} levam na conta de atributos ({:.0}%)",
                    briga.label,
                    briga.detail,
                    pct(briga.odds.max(1.0 - briga.odds))
                ),
            },
        ));
    }
    edges.sort_by(|a, b| b.0.total_cmp(&a.0));
    let edges = edges.into_iter().take(MAX_EDGES).map(|(_, p)| p).collect();

    WinCondition {
        facts,
        edges,
        phase: phase_from_data(ours, theirs, store),
        caveat: CAVEAT,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::champion::{fake, Champion, Role};
    use crate::engine::items::ItemIndex;
    use crate::engine::lanes;
    use crate::engine::stats::NoStats;
    use std::collections::HashMap;

    fn alcance(mut c: Champion, range: f64) -> Champion {
        c.attack_range = range;
        c
    }

    fn indice() -> ChampionIndex {
        let champions = vec![
            alcance(fake(1, "Atirador", "Marksman", 9.0, 2.0, 3.0), 550.0),
            alcance(fake(2, "Mago", "Mage", 2.0, 9.0, 3.0), 550.0),
            alcance(fake(3, "Suporte", "Support", 3.0, 7.0, 5.0), 525.0),
            alcance(fake(4, "Lutador", "Fighter", 8.0, 3.0, 5.0), 125.0),
            alcance(fake(5, "Tanque", "Tank", 5.0, 6.0, 9.0), 125.0),
            alcance(fake(6, "Assassino", "Assassin", 9.0, 2.0, 3.0), 125.0),
            alcance(fake(7, "Lutador2", "Fighter", 8.0, 3.0, 5.0), 125.0),
            alcance(fake(8, "Atirador2", "Marksman", 9.0, 2.0, 3.0), 550.0),
            // Atributos bem acima dos outros: é quem faz a conta de duelo
            // pender. Sem ele, todos têm os mesmos atributos e todo duelo dá 50%.
            {
                let mut gigante = fake(9, "Gigante", "Fighter", 8.0, 3.0, 6.0);
                gigante.base.health = 900.0;
                gigante.base.attack_damage = 85.0;
                gigante
            },
        ];
        ChampionIndex {
            version: "test".into(),
            by_id: champions.into_iter().map(|c| (c.id, c)).collect::<HashMap<_, _>>(),
        }
    }

    fn time(ids: &[(Role, i64)]) -> Vec<Lineup> {
        ids.iter().map(|(r, id)| Lineup::new(*r, *id, 6, vec![])).collect()
    }

    fn ler(ours: &[Lineup], theirs: &[Lineup]) -> WinCondition {
        let idx = indice();
        let it = ItemIndex::default();
        let lanes = lanes::lane_outlooks(ours, theirs, &idx, &it, &NoStats);
        let brigas = lanes::skirmishes(ours, theirs, &idx, &it);
        analyze(ours, theirs, &lanes, &brigas, &idx, &NoStats)
    }

    #[test]
    fn time_todo_fisico_e_sem_frente_e_descrito() {
        let nos = time(&[(Role::Top, 4), (Role::Jungle, 6), (Role::Mid, 7), (Role::Bot, 1)]);
        let w = ler(&nos, &[]);
        let textos: Vec<&str> = w.facts.iter().map(|p| p.text.as_str()).collect();
        assert!(textos.iter().any(|t| t.contains("físico")), "{textos:?}");
        assert!(textos.iter().any(|t| t.contains("ninguém segura")), "{textos:?}");
        assert!(w.facts.iter().all(|p| p.side == Side::Nosso));
    }

    #[test]
    fn time_de_alcance_e_descrito() {
        let nos = time(&[(Role::Mid, 2), (Role::Bot, 1), (Role::Support, 3), (Role::Top, 8)]);
        let w = ler(&nos, &[]);
        assert!(w.facts.iter().any(|p| p.text.contains("atacam à distância")), "{:?}", w.facts);
    }

    #[test]
    fn time_incompleto_nao_vira_composicao() {
        // Dois inimigos travados não dizem nada sobre a composição deles.
        let nos = time(&[(Role::Top, 4), (Role::Jungle, 6), (Role::Mid, 7)]);
        let eles = time(&[(Role::Top, 5), (Role::Mid, 2)]);
        let w = ler(&nos, &eles);
        assert!(!w.facts.iter().any(|p| p.side == Side::Deles), "{:?}", w.facts);
    }

    #[test]
    fn sem_base_nao_ha_fase() {
        let nos = time(&[(Role::Top, 4), (Role::Jungle, 6), (Role::Mid, 7), (Role::Bot, 1)]);
        assert!(ler(&nos, &[]).phase.is_none());
    }

    #[test]
    fn com_base_a_fase_sai_da_duracao_das_partidas() {
        // Os seus ganham muito mais em partida longa; os deles, igual.
        struct Base;
        impl StatsStore for Base {
            fn base(&self, _: i64, _: Role) -> Option<crate::engine::stats::Record> {
                Some(crate::engine::stats::Record::new(20_000, 10_000))
            }
            fn matchup(&self, _: i64, _: i64, _: Role) -> Option<crate::engine::stats::Record> {
                None
            }
            fn synergy(&self, _: i64, _: i64) -> Option<crate::engine::stats::Record> {
                None
            }
            fn personal(&self, _: i64) -> Option<crate::engine::stats::Record> {
                None
            }
            fn length(&self, c: i64, _: Role, faixa: Length) -> Option<crate::engine::stats::Record> {
                use crate::engine::stats::Record;
                let nosso = c <= 4;
                Some(match (nosso, faixa) {
                    (true, Length::Longa) => Record::new(5_000, 2_800),
                    (true, Length::Curta) => Record::new(5_000, 2_200),
                    _ => Record::new(5_000, 2_500),
                })
            }
            fn has_global(&self) -> bool {
                true
            }
        }
        let nos = time(&[(Role::Top, 4), (Role::Mid, 2), (Role::Bot, 1)]);
        let eles = time(&[(Role::Top, 7), (Role::Mid, 6), (Role::Bot, 8)]);
        let idx = indice();
        let w = analyze(&nos, &eles, &[], &[], &idx, &Base);
        let fase = w.phase.expect("com base há fase");
        assert!(fase.contains("longas") && fase.starts_with("Pelos dados"), "{fase}");
    }

    #[test]
    fn toda_afirmacao_diz_de_onde_sai() {
        let nos = time(&[(Role::Top, 9), (Role::Jungle, 5), (Role::Mid, 2), (Role::Bot, 1), (Role::Support, 3)]);
        let eles = time(&[(Role::Top, 7), (Role::Jungle, 6), (Role::Mid, 4), (Role::Bot, 8), (Role::Support, 5)]);
        let w = ler(&nos, &eles);
        assert!(
            w.edges.iter().any(|p| p.side == Side::Nosso && p.text.starts_with("TOP")),
            "o Gigante tem de pesar no topo: {:?}",
            w.edges
        );
        assert!(w.facts.iter().all(|p| p.source == Source::Regra));
        assert!(w.edges.iter().all(|p| p.source == Source::Atributos));
        assert!(w.caveat.contains("sem habilidades"));
    }

    #[test]
    fn nada_aqui_e_ordem() {
        // C-5: descrição, nunca imperativo. Os verbos de ordem mais comuns em
        // app de draft não podem abrir nenhuma frase.
        let nos = time(&[(Role::Top, 9), (Role::Jungle, 5), (Role::Mid, 2), (Role::Bot, 1), (Role::Support, 3)]);
        let eles = time(&[(Role::Top, 7), (Role::Jungle, 6), (Role::Mid, 4), (Role::Bot, 8), (Role::Support, 5)]);
        let w = ler(&nos, &eles);
        assert!(!w.edges.is_empty(), "sem vantagem não há frase para conferir");
        let ordens = ["jogue", "force", "evite", "foque", "priorize", "invada", "escolha", "pegue"];
        for p in w.facts.iter().chain(w.edges.iter()) {
            let texto = p.text.to_lowercase();
            for o in ordens {
                assert!(!texto.contains(o), "\"{}\" soa como ordem", p.text);
            }
        }
    }
}
