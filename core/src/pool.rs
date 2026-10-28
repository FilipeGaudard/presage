//! O pool do jogador. Ele escolhe o que quiser: nada é obrigatório e não há
//! cota por tier. Os tiers servem para o jogador se organizar e para o draft
//! desempatar, não para travar a tela.
//!
//! O construtor monta a pool em volta dos core: três escolhidos, e os
//! situacionais recomendados para cobrir o que eles não cobrem
//! (`engine::poolgap`).

use serde::{Deserialize, Serialize};

use crate::engine::champion::Role;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Tier {
    Core,
    Situational,
    Extra,
}

impl Tier {
    /// Ordem de exibição dentro do pool, antes do score desempatar.
    pub fn weight(self) -> u8 {
        match self {
            Tier::Core => 0,
            Tier::Situational => 1,
            Tier::Extra => 2,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PoolEntry {
    pub champion_id: i64,
    /// `None` significa "vale em qualquer rota".
    pub role: Option<Role>,
    pub tier: Tier,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pool {
    pub entries: Vec<PoolEntry>,
}

impl Pool {
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn ids(&self) -> Vec<i64> {
        let mut ids: Vec<i64> = self.entries.iter().map(|entry| entry.champion_id).collect();
        ids.sort_unstable();
        ids.dedup();
        ids
    }

    /// Campeões válidos para uma rota: os marcados nela, mais os sem rota.
    pub fn ids_for_role(&self, role: Role) -> Vec<i64> {
        let mut ids: Vec<i64> = self
            .entries
            .iter()
            .filter(|entry| entry.role.is_none() || entry.role == Some(role))
            .map(|entry| entry.champion_id)
            .collect();
        ids.sort_unstable();
        ids.dedup();
        ids
    }

    /// Insere, ou atualiza o tier se o campeão já estiver naquela rota.
    pub fn upsert(&mut self, entry: PoolEntry) {
        match self
            .entries
            .iter_mut()
            .find(|e| e.champion_id == entry.champion_id && e.role == entry.role)
        {
            Some(existing) => existing.tier = entry.tier,
            None => self.entries.push(entry),
        }
    }

    pub fn remove(&mut self, champion_id: i64, role: Option<Role>) {
        self.entries
            .retain(|entry| !(entry.champion_id == champion_id && entry.role == role));
    }

    /// Os de um tier numa rota, sem os que valem em qualquer rota: os core de
    /// uma rota são escolha para ela.
    pub fn ids_for_role_tier(&self, role: Role, tier: Tier) -> Vec<i64> {
        self.entries
            .iter()
            .filter(|entry| entry.role == Some(role) && entry.tier == tier)
            .map(|entry| entry.champion_id)
            .collect()
    }

    pub fn tier_of(&self, champion_id: i64, role: Role) -> Option<Tier> {
        self.entries
            .iter()
            .filter(|entry| {
                entry.champion_id == champion_id
                    && (entry.role.is_none() || entry.role == Some(role))
            })
            .map(|entry| entry.tier)
            .min_by_key(|tier| tier.weight())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(champion_id: i64, role: Option<Role>, tier: Tier) -> PoolEntry {
        PoolEntry {
            champion_id,
            role,
            tier,
        }
    }

    #[test]
    fn pool_vazio() {
        let pool = Pool::default();
        assert!(pool.is_empty());
        assert!(pool.ids().is_empty());
        assert!(pool.ids_for_role(Role::Mid).is_empty());
    }

    #[test]
    fn os_core_de_uma_rota_sao_so_dela() {
        let mut pool = Pool::default();
        pool.upsert(entry(103, Some(Role::Mid), Tier::Core));
        pool.upsert(entry(7, Some(Role::Mid), Tier::Situational));
        pool.upsert(entry(64, Some(Role::Jungle), Tier::Core));
        pool.upsert(entry(1, None, Tier::Core));

        assert_eq!(pool.ids_for_role_tier(Role::Mid, Tier::Core), vec![103]);
        assert_eq!(pool.ids_for_role_tier(Role::Mid, Tier::Situational), vec![7]);
    }

    #[test]
    fn sem_rota_vale_em_qualquer_uma() {
        let mut pool = Pool::default();
        pool.upsert(entry(103, None, Tier::Core));
        assert_eq!(pool.ids_for_role(Role::Mid), vec![103]);
        assert_eq!(pool.ids_for_role(Role::Top), vec![103]);
    }

    #[test]
    fn rota_marcada_nao_vaza_para_outra() {
        let mut pool = Pool::default();
        pool.upsert(entry(64, Some(Role::Jungle), Tier::Core));
        assert_eq!(pool.ids_for_role(Role::Jungle), vec![64]);
        assert!(pool.ids_for_role(Role::Mid).is_empty());
    }

    #[test]
    fn mesmo_campeao_em_duas_rotas_convive() {
        let mut pool = Pool::default();
        pool.upsert(entry(157, Some(Role::Mid), Tier::Core));
        pool.upsert(entry(157, Some(Role::Top), Tier::Situational));

        assert_eq!(pool.entries.len(), 2);
        assert_eq!(pool.ids(), vec![157]);
        assert_eq!(pool.tier_of(157, Role::Mid), Some(Tier::Core));
        assert_eq!(pool.tier_of(157, Role::Top), Some(Tier::Situational));
    }

    #[test]
    fn upsert_atualiza_o_tier_sem_duplicar() {
        let mut pool = Pool::default();
        pool.upsert(entry(103, Some(Role::Mid), Tier::Extra));
        pool.upsert(entry(103, Some(Role::Mid), Tier::Core));

        assert_eq!(pool.entries.len(), 1);
        assert_eq!(pool.tier_of(103, Role::Mid), Some(Tier::Core));
    }

    #[test]
    fn remove_so_a_entrada_daquela_rota() {
        let mut pool = Pool::default();
        pool.upsert(entry(157, Some(Role::Mid), Tier::Core));
        pool.upsert(entry(157, Some(Role::Top), Tier::Core));
        pool.remove(157, Some(Role::Top));

        assert_eq!(pool.entries.len(), 1);
        assert_eq!(pool.entries[0].role, Some(Role::Mid));
    }

    #[test]
    fn sem_cota_por_tier() {
        // Nada obrigatório e nenhum teto: dez core numa rota é válido.
        let mut pool = Pool::default();
        for id in 0..10 {
            pool.upsert(entry(id, Some(Role::Mid), Tier::Core));
        }
        assert_eq!(pool.ids_for_role(Role::Mid).len(), 10);
    }

    #[test]
    fn tier_mais_forte_ganha_quando_ha_rota_e_curinga() {
        let mut pool = Pool::default();
        pool.upsert(entry(103, None, Tier::Extra));
        pool.upsert(entry(103, Some(Role::Mid), Tier::Core));
        assert_eq!(pool.tier_of(103, Role::Mid), Some(Tier::Core));
    }

    #[test]
    fn campeao_fora_do_pool_nao_tem_tier() {
        assert_eq!(Pool::default().tier_of(999, Role::Mid), None);
    }
}
