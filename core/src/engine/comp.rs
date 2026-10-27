use super::champion::{Champion, ChampionIndex};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TeamProfile {
    pub physical_share: f64,
    pub frontline: usize,
    pub ranged: usize,
    pub known: usize,
}

impl TeamProfile {
    /// Quanto do time já está definido. Um sinal lido com um pick travado vale
    /// menos que o mesmo sinal com quatro.
    fn completeness(&self) -> f64 {
        (self.known as f64 / 4.0).min(1.0)
    }
}

pub fn profile(ids: &[i64], index: &ChampionIndex) -> TeamProfile {
    let champions: Vec<&Champion> = ids.iter().filter_map(|id| index.get(*id)).collect();
    let known = champions.len();

    if known == 0 {
        return TeamProfile {
            physical_share: 0.5,
            frontline: 0,
            ranged: 0,
            known: 0,
        };
    }

    TeamProfile {
        physical_share: champions.iter().map(|c| c.physical_share()).sum::<f64>() / known as f64,
        frontline: champions.iter().filter(|c| c.is_frontline()).count(),
        ranged: champions.iter().filter(|c| c.is_ranged()).count(),
        known,
    }
}

/// Quanto o candidato aproxima o dano do time de um split equilibrado.
/// Positivo quando corrige um time unilateral, negativo quando piora.
pub fn damage_balance_gain(team: &TeamProfile, candidate: &Champion) -> f64 {
    if team.known == 0 {
        return 0.0;
    }
    let before = team.physical_share;
    let after = (before * team.known as f64 + candidate.physical_share()) / (team.known as f64 + 1.0);
    let gain = (before - 0.5).abs() - (after - 0.5).abs();
    gain * team.completeness()
}

/// Quanto o time ainda precisa de frontline, de 0 a 1.
pub fn frontline_need(team: &TeamProfile) -> f64 {
    if team.known == 0 {
        return 0.0;
    }
    (1.0 - team.frontline as f64).max(0.0) * team.completeness()
}

pub fn share_after(team: &TeamProfile, candidate: &Champion) -> f64 {
    if team.known == 0 {
        return candidate.physical_share();
    }
    (team.physical_share * team.known as f64 + candidate.physical_share())
        / (team.known as f64 + 1.0)
}

#[cfg(test)]
mod tests {
    use super::super::champion::fake;
    use super::*;
    use std::collections::HashMap;

    fn index(champions: Vec<Champion>) -> ChampionIndex {
        ChampionIndex {
            version: "test".into(),
            by_id: champions.into_iter().map(|c| (c.id, c)).collect::<HashMap<_, _>>(),
        }
    }

    fn ad_squad() -> ChampionIndex {
        index(vec![
            fake(1, "Marks", "Marksman", 9.0, 2.0, 3.0),
            fake(2, "Lutador", "Fighter", 8.0, 3.0, 5.0),
            fake(3, "Assassino", "Assassin", 9.0, 3.0, 3.0),
            fake(4, "Mago", "Mage", 2.0, 9.0, 4.0),
            fake(5, "Tanque", "Tank", 5.0, 6.0, 9.0),
        ])
    }

    #[test]
    fn time_vazio_e_neutro() {
        let p = profile(&[], &ad_squad());
        assert_eq!(p.known, 0);
        assert!((p.physical_share - 0.5).abs() < 1e-12);
    }

    #[test]
    fn ids_desconhecidos_sao_ignorados() {
        let p = profile(&[1, 999], &ad_squad());
        assert_eq!(p.known, 1);
    }

    #[test]
    fn mago_equilibra_time_todo_ad() {
        let idx = ad_squad();
        let team = profile(&[1, 2, 3], &idx);
        let mago = idx.get(4).unwrap();
        assert!(damage_balance_gain(&team, mago) > 0.0);
    }

    #[test]
    fn mais_um_ad_em_time_ad_nao_ajuda() {
        let idx = ad_squad();
        let team = profile(&[1, 2, 3], &idx);
        let outro_ad = idx.get(1).unwrap();
        assert!(damage_balance_gain(&team, outro_ad) <= 0.0);
    }

    #[test]
    fn time_sem_tanque_precisa_de_frontline() {
        let idx = ad_squad();
        assert!(frontline_need(&profile(&[1, 3, 4], &idx)) > 0.0);
    }

    #[test]
    fn time_com_tanque_nao_precisa() {
        let idx = ad_squad();
        assert_eq!(frontline_need(&profile(&[1, 3, 5], &idx)), 0.0);
    }

    #[test]
    fn sinal_vale_menos_no_comeco_do_draft() {
        let idx = ad_squad();
        let cedo = profile(&[1], &idx);
        let tarde = profile(&[1, 2, 3], &idx);
        let mago = idx.get(4).unwrap();
        assert!(damage_balance_gain(&cedo, mago) < damage_balance_gain(&tarde, mago));
    }
}
