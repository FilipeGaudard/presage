//! A chance do time: um modelo só, em log-odds, com os dez campeões.
//!
//! Soma, para os dois lados, a força de cada um na rota, os confrontos de rota,
//! o contra de cada par de adversários e a sinergia de cada par de
//! companheiros. É a mesma álgebra do ranking — as mesmas referências e as
//! mesmas forças de prior —, então a chance do time com um candidato e o
//! número dele no ranking contam a mesma história.
//!
//! Sem a base de partidas não sai número: `None`. Cinquenta parcelas pequenas
//! somadas parecem confiantes mesmo quando cada uma é ruído, então o resultado
//! fica preso entre 35% e 65% e leva o `±` da soma das incertezas.

use serde::Serialize;

use super::champion::{ChampionIndex, Role};
use super::context::DraftContext;
use super::stats::{
    expected_against, expected_together, logistic, logit, posterior_sd, shrink, shrunk_delta,
    strength_for_sd, Estimate, StatsStore,
};
use super::{all_roles_rate, lane_share, BASE_STRENGTH, MATCHUP_PRIOR_SD, PRIOR_RATE, SYNERGY_PRIOR_SD, VERSUS_PRIOR_SD};

const FLOOR: f64 = 0.35;
const CEILING: f64 = 0.65;
const MAX_TERMS: usize = 4;

/// Um campeão de um dos lados, com a certeza de que a rota é aquela.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Seat {
    pub champion_id: i64,
    pub role: Role,
    /// 1 com a rota informada pelo cliente; a fração do encaixe dele na rota
    /// quando inferida.
    pub certainty: f64,
}

/// Uma parcela da conta, para a interface dizer de onde vem a vantagem.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamTerm {
    pub label: String,
    /// O quanto ela move a chance, em pontos percentuais, sozinha.
    pub shift: f64,
    pub games: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamEstimate {
    /// A chance do seu time, de 0 a 1.
    pub win: f64,
    /// `1,96 × desvio`, em pontos percentuais.
    pub confidence: f64,
    /// As parcelas que mais pesam, para um lado e para o outro.
    pub terms: Vec<TeamTerm>,
}

fn shift(reference: f64, delta: f64) -> f64 {
    logit(reference + delta) - logit(reference)
}

fn base_in(store: &dyn StatsStore, champion: i64, role: Role) -> Estimate {
    shrink(store.base(champion, role), PRIOR_RATE, BASE_STRENGTH)
}

/// Os dois lados de um contexto de draft: o seu time com você (ou o
/// candidato), e o time deles com as rotas inferidas.
pub fn seats(ctx: &DraftContext, index: &ChampionIndex, store: &dyn StatsStore) -> (Vec<Seat>, Vec<Seat>) {
    let seat = |id: i64, role: Role, informed: bool| Seat {
        champion_id: id,
        role,
        certainty: if informed {
            1.0
        } else {
            index.get(id).map_or(0.0, |c| lane_share(c, role, store))
        },
    };
    let mut ours: Vec<Seat> = ctx
        .allies
        .iter()
        .filter(|p| p.role != Role::Unknown)
        .map(|p| seat(p.champion_id, p.role, p.informed))
        .collect();
    if let Some(me) = ctx.me.filter(|_| ctx.my_role != Role::Unknown) {
        ours.push(seat(me, ctx.my_role, true));
    }
    let theirs = ctx
        .enemies
        .iter()
        .filter(|p| p.role != Role::Unknown)
        .map(|p| seat(p.champion_id, p.role, p.informed))
        .collect();
    (ours, theirs)
}

/// A chance de `ours` contra `theirs`. `None` sem a base de partidas ou sem
/// ninguém de um dos lados.
pub fn estimate(
    ours: &[Seat],
    theirs: &[Seat],
    index: &ChampionIndex,
    store: &dyn StatsStore,
) -> Option<TeamEstimate> {
    if !store.has_global() || ours.is_empty() || theirs.is_empty() {
        return None;
    }
    let nome = |id: i64| index.get(id).map_or_else(|| format!("#{id}"), |c| c.name.clone());

    let mut z = 0.0;
    let mut variance = 0.0;
    let mut terms: Vec<TeamTerm> = Vec::new();
    let mut parcela = |label: String, dz: f64, var_p: f64, games: u32, z: &mut f64, variance: &mut f64| {
        *z += dz;
        *variance += var_p;
        if games > 0 && dz.abs() > 1e-9 {
            terms.push(TeamTerm {
                label,
                shift: ((logistic(dz) - 0.5) * 1000.0).round() / 10.0,
                games,
            });
        }
    };

    // A força de cada um na rota.
    for (lado, sinal) in [(ours, 1.0), (theirs, -1.0)] {
        for s in lado {
            let b = base_in(store, s.champion_id, s.role);
            parcela(
                format!("{} na rota", nome(s.champion_id)),
                sinal * logit(b.rate),
                b.se().powi(2),
                b.games,
                &mut z,
                &mut variance,
            );
        }
    }

    // Cada par de adversários: confronto de rota na proporção em que os dois
    // são mesmo da mesma rota, contra no resto.
    for o in ours {
        for t in theirs {
            let w = if o.role == t.role { o.certainty * t.certainty } else { 0.0 };
            if w > 0.0 {
                let (bo, bt) = (base_in(store, o.champion_id, o.role).rate, base_in(store, t.champion_id, t.role).rate);
                let referencia = expected_against(bo, bt);
                let m = shrunk_delta(
                    store.matchup(o.champion_id, t.champion_id, o.role),
                    referencia,
                    strength_for_sd(MATCHUP_PRIOR_SD),
                );
                parcela(
                    format!("{} × {} na rota", nome(o.champion_id), nome(t.champion_id)),
                    w * shift(referencia, m.rate),
                    (w * posterior_sd(m.games, MATCHUP_PRIOR_SD)).powi(2),
                    m.games,
                    &mut z,
                    &mut variance,
                );
            }
            if w < 1.0 {
                let referencia = expected_against(
                    all_roles_rate(store, o.champion_id),
                    all_roles_rate(store, t.champion_id),
                );
                let v = shrunk_delta(
                    store.versus(o.champion_id, t.champion_id),
                    referencia,
                    strength_for_sd(VERSUS_PRIOR_SD),
                );
                parcela(
                    format!("{} contra {}", nome(o.champion_id), nome(t.champion_id)),
                    (1.0 - w) * shift(referencia, v.rate),
                    ((1.0 - w) * posterior_sd(v.games, VERSUS_PRIOR_SD)).powi(2),
                    v.games,
                    &mut z,
                    &mut variance,
                );
            }
        }
    }

    // A sinergia de cada par de companheiros, a favor de quem a tem.
    for (lado, sinal) in [(ours, 1.0), (theirs, -1.0)] {
        for (i, a) in lado.iter().enumerate() {
            for b in &lado[i + 1..] {
                let referencia = expected_together(
                    base_in(store, a.champion_id, a.role).rate,
                    base_in(store, b.champion_id, b.role).rate,
                );
                let s = shrunk_delta(
                    store.synergy(a.champion_id, b.champion_id),
                    referencia,
                    strength_for_sd(SYNERGY_PRIOR_SD),
                );
                parcela(
                    format!("{} com {}", nome(a.champion_id), nome(b.champion_id)),
                    sinal * shift(referencia, s.rate),
                    posterior_sd(s.games, SYNERGY_PRIOR_SD).powi(2),
                    s.games,
                    &mut z,
                    &mut variance,
                );
            }
        }
    }

    let win = logistic(z).clamp(FLOOR, CEILING);
    terms.sort_by(|a, b| b.shift.abs().total_cmp(&a.shift.abs()));
    terms.truncate(MAX_TERMS);

    Some(TeamEstimate {
        win,
        confidence: (1.96 * variance.sqrt() * 1000.0).round() / 10.0,
        terms,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::champion::fake;
    use crate::engine::stats::{NoStats, Record};
    use std::collections::HashMap;

    fn indice() -> ChampionIndex {
        let c = vec![
            fake(1, "A", "Fighter", 8.0, 3.0, 5.0),
            fake(2, "B", "Mage", 2.0, 9.0, 4.0),
            fake(11, "X", "Fighter", 8.0, 3.0, 5.0),
            fake(12, "Y", "Mage", 2.0, 9.0, 4.0),
        ];
        ChampionIndex {
            version: "test".into(),
            by_id: c.into_iter().map(|c| (c.id, c)).collect::<HashMap<_, _>>(),
        }
    }

    /// Base coerente: cada registro direcional tem o seu complemento.
    struct Base;
    impl StatsStore for Base {
        fn base(&self, c: i64, _r: Role) -> Option<Record> {
            Some(match c {
                1 => Record::new(20_000, 10_600),
                2 => Record::new(20_000, 9_800),
                11 => Record::new(20_000, 9_900),
                _ => Record::new(20_000, 10_100),
            })
        }
        fn matchup(&self, c: i64, o: i64, _r: Role) -> Option<Record> {
            match (c, o) {
                (1, 11) => Some(Record::new(3_000, 1_800)),
                (11, 1) => Some(Record::new(3_000, 1_200)),
                _ => None,
            }
        }
        fn synergy(&self, a: i64, b: i64) -> Option<Record> {
            (a.min(b) == 1 && a.max(b) == 2).then(|| Record::new(2_000, 1_080))
        }
        fn versus(&self, c: i64, o: i64) -> Option<Record> {
            match (c, o) {
                (2, 11) => Some(Record::new(2_500, 1_400)),
                (11, 2) => Some(Record::new(2_500, 1_100)),
                _ => None,
            }
        }
        fn personal(&self, _: i64) -> Option<Record> {
            None
        }
        fn has_global(&self) -> bool {
            true
        }
    }

    fn lado(ids: &[(i64, Role)]) -> Vec<Seat> {
        ids.iter()
            .map(|&(champion_id, role)| Seat {
                champion_id,
                role,
                certainty: 1.0,
            })
            .collect()
    }

    #[test]
    fn sem_base_nao_ha_chance_do_time() {
        let a = lado(&[(1, Role::Top)]);
        let b = lado(&[(11, Role::Top)]);
        assert!(estimate(&a, &b, &indice(), &NoStats).is_none());
    }

    #[test]
    fn a_chance_dos_dois_lados_soma_um() {
        let a = lado(&[(1, Role::Top), (2, Role::Mid)]);
        let b = lado(&[(11, Role::Top), (12, Role::Mid)]);
        let ab = estimate(&a, &b, &indice(), &Base).unwrap().win;
        let ba = estimate(&b, &a, &indice(), &Base).unwrap().win;
        assert!((ab + ba - 1.0).abs() < 1e-9, "{ab} + {ba}");
        assert!(ab > 0.5, "matchup, contra e sinergia são todos do lado A");
    }

    #[test]
    fn fica_entre_35_e_65() {
        let a = lado(&[(1, Role::Top), (2, Role::Mid)]);
        let b = lado(&[(11, Role::Top), (12, Role::Mid)]);
        let r = estimate(&a, &b, &indice(), &Base).unwrap();
        assert!((FLOOR..=CEILING).contains(&r.win));
        assert!(r.confidence > 0.0);
    }

    #[test]
    fn diz_de_onde_vem_a_vantagem() {
        let a = lado(&[(1, Role::Top), (2, Role::Mid)]);
        let b = lado(&[(11, Role::Top), (12, Role::Mid)]);
        let r = estimate(&a, &b, &indice(), &Base).unwrap();
        assert!(r.terms.iter().any(|t| t.label == "A × X na rota" && t.shift > 0.0), "{:?}", r.terms);
        assert!(r.terms.iter().all(|t| t.games > 0));
        assert!(r.terms.len() <= MAX_TERMS);
    }

    #[test]
    fn rota_incerta_divide_entre_matchup_e_contra() {
        // Com certeza zero de que os dois são da mesma rota, o par entra só
        // pelo contra: o matchup de rota some da lista.
        let a = vec![Seat { champion_id: 1, role: Role::Top, certainty: 1.0 }];
        let b = vec![Seat { champion_id: 11, role: Role::Top, certainty: 0.0 }];
        let r = estimate(&a, &b, &indice(), &Base).unwrap();
        assert!(!r.terms.iter().any(|t| t.label.contains("na rota") && t.label.contains('×')));
    }
}
