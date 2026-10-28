//! Quem joga cada rota.
//!
//! Uma regra só, para o draft, a pool e a resposta do minimax não
//! discordarem sobre o mesmo campeão (L-4).
//!
//! Com a base de partidas, a resposta sai das partidas: joga a rota quem tem
//! nela ao menos 5% das próprias. Sem a base, das classes do Data Dragon —
//! grosseiro, porque classe não é rota, e o topo de alcance fica de fora. Por
//! isso o histórico do próprio jogador também conta: o campeão que você já
//! jogou na rota é da rota para você, diga a classe o que disser.

use super::champion::{Champion, Role};
use super::stats::StatsStore;

pub const ROLES: [Role; 5] = [Role::Top, Role::Jungle, Role::Mid, Role::Bot, Role::Support];

/// Com a base, joga a rota quem tem nela ao menos esta fração das partidas.
const ROLE_SHARE: f64 = 0.05;

fn index(role: Role) -> Option<usize> {
    ROLES.iter().position(|r| *r == role)
}

/// As classes que contam para cada rota quando não há base.
///
/// Suporte exige a classe Suporte: aceitar todo tanque punha Malphite e Sion
/// no topo da lista de suporte, levados pelo peso de frontline.
fn tags(role: Role) -> &'static [&'static str] {
    match role {
        Role::Top => &["Fighter", "Tank"],
        Role::Jungle => &["Fighter", "Tank", "Assassin"],
        Role::Mid => &["Mage", "Assassin"],
        Role::Bot => &["Marksman"],
        Role::Support => &["Support"],
        Role::Unknown => &[],
    }
}

/// A fração das partidas do campeão em cada rota, na ordem de `ROLES`.
/// `None` sem base ou sem partida dele.
pub fn role_share(champion: i64, store: &dyn StatsStore) -> Option<[f64; 5]> {
    if !store.has_global() {
        return None;
    }
    let games = ROLES.map(|r| store.base(champion, r).map_or(0, |rec| rec.games) as f64);
    let total: f64 = games.iter().sum();
    (total > 0.0).then(|| games.map(|g| g / total))
}

/// Quanto o campeão é da rota, de 0 a 1.
///
/// Com a base, a fração das partidas dele ali. Sem ela, pela classe: principal
/// na rota vale 1, secundária 0,5, nenhuma 0. É o que a inferência da rota do
/// inimigo compara entre campeões.
pub fn role_fit(champion: &Champion, role: Role, store: &dyn StatsStore) -> f64 {
    let Some(i) = index(role) else {
        return 0.0;
    };
    if let Some(share) = role_share(champion.id, store) {
        return share[i];
    }
    let na_rota = |tag: &String| tags(role).contains(&tag.as_str());
    match champion.tags.iter().position(na_rota) {
        Some(0) => 1.0,
        Some(_) => 0.5,
        None => 0.0,
    }
}

/// O campeão joga a rota?
///
/// Rota desconhecida (blind, normal sem posição) não filtra ninguém. O
/// histórico do jogador vale antes de tudo; depois a base, se houver partida
/// do campeão nela; por fim a classe.
pub fn plays(champion: &Champion, role: Role, store: &dyn StatsStore) -> bool {
    if role == Role::Unknown {
        return true;
    }
    if store.played_in(champion.id, role) {
        return true;
    }
    match (role_share(champion.id, store), index(role)) {
        (Some(share), Some(i)) => share[i] >= ROLE_SHARE,
        _ => role_fit(champion, role, store) > 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::champion::fake;
    use crate::engine::stats::{NoStats, Record};

    fn com_tags(mut c: Champion, tags: &[&str]) -> Champion {
        c.tags = tags.iter().map(|t| t.to_string()).collect();
        c
    }

    /// Base com partidas só de quem está na tabela.
    struct Base(Vec<(i64, Role, u32)>);

    impl StatsStore for Base {
        fn base(&self, champion: i64, role: Role) -> Option<Record> {
            self.0
                .iter()
                .find(|(c, r, _)| *c == champion && *r == role)
                .map(|(_, _, g)| Record::new(*g, *g / 2))
        }
        fn matchup(&self, _: i64, _: i64, _: Role) -> Option<Record> {
            None
        }
        fn synergy(&self, _: i64, _: i64) -> Option<Record> {
            None
        }
        fn personal(&self, _: i64) -> Option<Record> {
            None
        }
        fn has_global(&self) -> bool {
            true
        }
    }

    #[test]
    fn rota_desconhecida_nao_filtra_nem_com_base() {
        let lutador = fake(1, "Lutador", "Fighter", 8.0, 3.0, 5.0);
        assert!(plays(&lutador, Role::Unknown, &NoStats));
        assert!(plays(&lutador, Role::Unknown, &Base(vec![])));
    }

    #[test]
    fn tanque_sem_classe_de_suporte_nao_vira_suporte() {
        let tanque = com_tags(fake(54, "Malphite", "Tank", 5.0, 7.0, 9.0), &["Tank", "Fighter"]);
        let suporte = com_tags(fake(89, "Leona", "Tank", 4.0, 3.0, 8.0), &["Tank", "Support"]);
        assert!(!plays(&tanque, Role::Support, &NoStats));
        assert!(plays(&suporte, Role::Support, &NoStats));
        assert!(plays(&tanque, Role::Top, &NoStats));
    }

    #[test]
    fn classe_principal_pesa_mais_que_a_secundaria() {
        let principal = com_tags(fake(1, "A", "Mage", 2.0, 9.0, 3.0), &["Mage", "Support"]);
        let secundaria = com_tags(fake(2, "B", "Support", 2.0, 8.0, 3.0), &["Support", "Mage"]);
        assert_eq!(role_fit(&principal, Role::Mid, &NoStats), 1.0);
        assert_eq!(role_fit(&secundaria, Role::Mid, &NoStats), 0.5);
        assert_eq!(role_fit(&principal, Role::Bot, &NoStats), 0.0);
    }

    #[test]
    fn com_a_base_a_rota_sai_das_partidas() {
        // 30% das partidas do lutador no meio: pela classe não entraria.
        let lutador = fake(21, "Lutador", "Fighter", 8.0, 3.0, 5.0);
        let base = Base(vec![(21, Role::Mid, 300), (21, Role::Top, 700)]);
        assert!(plays(&lutador, Role::Mid, &base));
        assert!(!plays(&lutador, Role::Mid, &NoStats));
        assert!((role_fit(&lutador, Role::Mid, &base) - 0.3).abs() < 1e-9);
    }

    #[test]
    fn campeao_sem_partida_na_base_cai_na_classe() {
        // Campeão novo, que a base ainda não viu, não some da lista.
        let novo = fake(999, "Novo", "Mage", 2.0, 9.0, 3.0);
        assert!(plays(&novo, Role::Mid, &Base(vec![])));
    }

    #[test]
    fn o_que_voce_ja_jogou_na_rota_e_da_rota() {
        use crate::history::{LaneGame, Metric};
        use std::collections::BTreeMap;

        let mut pessoal = crate::engine::stats::PersonalStats::default();
        pessoal.lanes.push(LaneGame {
            champion: 17,
            opponent: 86,
            role: Role::Top,
            won: true,
            diffs: BTreeMap::from([(Metric::Deaths, 0.0)]),
        });
        // Atirador de alcance no topo: a classe diria que não.
        let atirador = fake(17, "Teemo", "Marksman", 5.0, 7.0, 3.0);
        assert!(plays(&atirador, Role::Top, &pessoal));
        assert!(!plays(&atirador, Role::Top, &NoStats));
    }
}
