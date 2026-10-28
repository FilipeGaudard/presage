//! O que o draft sabe, resolvido uma vez para todos que o leem.
//!
//! O ranking, o minimax e a análise de rota faziam cada um a sua leitura do
//! champ select, e discordavam: um contava o hover do aliado, outro não; um
//! achava o oponente de rota pela posição — que o cliente nunca informa do time
//! inimigo na ranqueada —, outro pela ordem de escolha, que não é rota. Aqui a
//! leitura é uma só.
//!
//! A rota do inimigo é inferida: entre todas as formas de distribuir os
//! campeões travados pelas rotas livres (no máximo 120), fica a mais
//! plausível pelo `roles::role_fit` — as partidas da base quando existem, as
//! classes quando não. Empate fica com a ordem de escolha, sempre a mesma
//! resposta para a mesma entrada. Posição informada pelo cliente nunca é
//! contestada.

use std::collections::HashSet;

use crate::draft::{DraftSlot, DraftState};
use crate::pool::Pool;

use super::champion::{ChampionIndex, Role};
use super::lanes::Lineup;
use super::roles::{self, ROLES};
use super::stats::StatsStore;

/// Nível de referência antes da partida começar. É no 6 que as rotas se
/// resolvem, e é igual para os dez, então a comparação continua justa.
pub const PREGAME_LEVEL: u8 = 6;

/// Plausibilidade mínima de uma rota, para uma atribuição improvável ainda
/// poder ganhar quando nenhuma é plausível.
const EPS: f64 = 0.01;

#[derive(Debug, Clone, PartialEq)]
pub struct Placed {
    pub champion_id: i64,
    pub role: Role,
    /// O campeão já está escolhido; falso para hover de aliado.
    pub picked: bool,
    /// A rota veio do cliente; falso quando foi inferida.
    pub informed: bool,
}

#[derive(Debug, Clone)]
pub struct DraftContext {
    pub my_role: Role,
    /// O seu campeão: escolhido, em hover, ou o candidato que se está avaliando.
    pub me: Option<i64>,
    /// Os outros quatro do seu time, com hover.
    pub allies: Vec<Placed>,
    /// Os travados do outro time, com a rota inferida.
    pub enemies: Vec<Placed>,
    /// Alguma rota saiu da inferência, não do cliente. A interface diz isso.
    pub roles_assumed: bool,
    /// Banidos, escolhidos e em hover de aliado. Não inclui a sua própria
    /// escolha ainda não travada: ela é sua.
    pub unavailable: HashSet<i64>,
    pub pool: HashSet<i64>,
    /// Quem você pode escolher: disponível, e joga a sua rota ou está na sua
    /// pool.
    pub candidates: Vec<i64>,
}

/// A sua rota.
///
/// O cliente informa na ranqueada; quando não informa (blind, normal), vale a
/// escolhida na tela; sem escolha, a da pool, se ela for de uma rota só.
pub fn resolve_role(local_position: &str, chosen: Option<Role>, pool: &Pool) -> Role {
    let do_cliente = Role::from_lcu(local_position);
    if do_cliente != Role::Unknown {
        return do_cliente;
    }
    if let Some(role) = chosen.filter(|r| *r != Role::Unknown) {
        return role;
    }
    let mut rotas: Vec<Role> = Vec::new();
    for role in pool.entries.iter().filter_map(|e| e.role) {
        if !rotas.contains(&role) {
            rotas.push(role);
        }
    }
    match rotas.as_slice() {
        [unica] => *unica,
        _ => Role::Unknown,
    }
}

fn slot_champion(slot: &DraftSlot) -> i64 {
    if slot.champion_id > 0 {
        slot.champion_id
    } else {
        slot.intent_id
    }
}

/// Distribui as rotas livres entre quem está sem rota, pela atribuição mais
/// plausível. Devolve verdadeiro se alguma rota foi inferida.
fn infer(team: &mut [Placed], reserved: &[Role], index: &ChampionIndex, store: &dyn StatsStore) -> bool {
    let sem_rota: Vec<usize> = (0..team.len())
        .filter(|&i| team[i].role == Role::Unknown)
        .collect();
    if sem_rota.is_empty() {
        return false;
    }
    let ocupadas: Vec<Role> = team
        .iter()
        .map(|p| p.role)
        .chain(reserved.iter().copied())
        .filter(|r| *r != Role::Unknown)
        .collect();
    let livres: Vec<Role> = ROLES.iter().copied().filter(|r| !ocupadas.contains(r)).collect();

    let fit = |i: usize, role: Role| -> f64 {
        index
            .get(team[i].champion_id)
            .map_or(0.0, |c| roles::role_fit(c, role, store))
    };

    // Busca exaustiva em ordem lexicográfica: a primeira melhor fica, e é isso
    // que torna o empate determinístico.
    let mut melhor: Option<(f64, Vec<Role>)> = None;
    let mut atual: Vec<Role> = Vec::new();
    fn busca(
        k: usize,
        sem_rota: &[usize],
        livres: &[Role],
        atual: &mut Vec<Role>,
        melhor: &mut Option<(f64, Vec<Role>)>,
        fit: &dyn Fn(usize, Role) -> f64,
    ) {
        if k == sem_rota.len() || atual.len() == livres.len() {
            let score: f64 = atual
                .iter()
                .enumerate()
                .map(|(j, role)| (fit(sem_rota[j], *role) + EPS).ln())
                .sum();
            if melhor.as_ref().map_or(true, |(s, _)| score > *s + 1e-12) {
                *melhor = Some((score, atual.clone()));
            }
            return;
        }
        for role in livres {
            if !atual.contains(role) {
                atual.push(*role);
                busca(k + 1, sem_rota, livres, atual, melhor, fit);
                atual.pop();
            }
        }
    }
    busca(0, &sem_rota, &livres, &mut atual, &mut melhor, &fit);

    if let Some((_, atribuicao)) = melhor {
        for (j, role) in atribuicao.into_iter().enumerate() {
            team[sem_rota[j]].role = role;
        }
    }
    true
}

impl DraftContext {
    pub fn new(
        draft: &DraftState,
        index: &ChampionIndex,
        store: &dyn StatsStore,
        pool: &[i64],
        my_role: Role,
    ) -> Self {
        let local = draft.my_team.iter().find(|slot| slot.is_local);
        let me = local.map(slot_champion).filter(|id| *id > 0);

        let mut allies: Vec<Placed> = draft
            .my_team
            .iter()
            .filter(|slot| !slot.is_local)
            .filter_map(|slot| {
                let id = slot_champion(slot);
                let role = Role::from_lcu(&slot.position);
                (id > 0).then(|| Placed {
                    champion_id: id,
                    role,
                    picked: slot.champion_id > 0,
                    informed: role != Role::Unknown,
                })
            })
            .collect();

        let mut enemies: Vec<Placed> = draft
            .their_team
            .iter()
            .filter(|slot| slot.champion_id > 0)
            .map(|slot| {
                let role = Role::from_lcu(&slot.position);
                Placed {
                    champion_id: slot.champion_id,
                    role,
                    picked: true,
                    informed: role != Role::Unknown,
                }
            })
            .collect();

        let assumed_allies = infer(&mut allies, &[my_role], index, store);
        let assumed_enemies = infer(&mut enemies, &[], index, store);

        let mut unavailable: HashSet<i64> = draft
            .my_bans
            .iter()
            .chain(draft.their_bans.iter())
            .copied()
            .chain(allies.iter().map(|p| p.champion_id))
            .chain(enemies.iter().map(|p| p.champion_id))
            .filter(|id| *id > 0)
            .collect();
        if let Some(slot) = local.filter(|slot| slot.locked) {
            unavailable.insert(slot.champion_id);
        }

        let pool: HashSet<i64> = pool.iter().copied().collect();
        let candidates = index
            .ids()
            .into_iter()
            .filter(|id| !unavailable.contains(id))
            .filter(|id| {
                pool.contains(id)
                    || index
                        .get(*id)
                        .is_some_and(|c| roles::plays(c, my_role, store))
            })
            .collect();

        DraftContext {
            my_role,
            me,
            allies,
            enemies,
            roles_assumed: assumed_allies || assumed_enemies,
            unavailable,
            pool,
            candidates,
        }
    }

    /// O mesmo draft com o candidato no seu lugar. É o que a prévia mostra: a
    /// mesma conta de quando ele estiver travado.
    pub fn with_candidate(&self, champion_id: i64) -> Self {
        DraftContext {
            me: Some(champion_id),
            ..self.clone()
        }
    }

    /// O inimigo da sua rota, quando há um.
    pub fn lane_opponent(&self) -> Option<&Placed> {
        if self.my_role == Role::Unknown {
            return None;
        }
        self.enemies.iter().find(|p| p.role == self.my_role)
    }

    /// As rotas do inimigo que ainda não têm campeão.
    pub fn open_enemy_roles(&self) -> Vec<Role> {
        ROLES
            .iter()
            .copied()
            .filter(|r| !self.enemies.iter().any(|p| p.role == *r))
            .collect()
    }

    pub fn ally_ids(&self) -> Vec<i64> {
        self.allies.iter().map(|p| p.champion_id).collect()
    }

    pub fn enemy_ids(&self) -> Vec<i64> {
        self.enemies.iter().map(|p| p.champion_id).collect()
    }

    /// Os dois lados como a análise de rota lê: o seu time com você, cada um na
    /// sua rota, no nível dado.
    pub fn lineups(&self, level: u8) -> (Vec<Lineup>, Vec<Lineup>) {
        let mut ours: Vec<Lineup> = self
            .allies
            .iter()
            .filter(|p| p.role != Role::Unknown)
            .map(|p| Lineup::new(p.role, p.champion_id, level, Vec::new()))
            .collect();
        if let Some(me) = self.me.filter(|_| self.my_role != Role::Unknown) {
            ours.push(Lineup::new(self.my_role, me, level, Vec::new()));
        }
        let theirs = self
            .enemies
            .iter()
            .filter(|p| p.role != Role::Unknown)
            .map(|p| Lineup::new(p.role, p.champion_id, level, Vec::new()))
            .collect();
        (ours, theirs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::champion::{fake, Champion};
    use crate::engine::stats::NoStats;
    use crate::pool::{PoolEntry, Tier};
    use std::collections::HashMap;

    fn com_tags(mut c: Champion, tags: &[&str]) -> Champion {
        c.tags = tags.iter().map(|t| t.to_string()).collect();
        c
    }

    fn indice() -> ChampionIndex {
        let champions = vec![
            fake(1, "Atirador", "Marksman", 9.0, 2.0, 3.0),
            fake(2, "Assassino", "Assassin", 9.0, 3.0, 3.0),
            fake(3, "Suporte", "Support", 3.0, 7.0, 5.0),
            fake(4, "Mago", "Mage", 2.0, 9.0, 4.0),
            com_tags(fake(5, "Tanque", "Tank", 5.0, 6.0, 9.0), &["Tank", "Fighter"]),
            fake(6, "Lutador", "Fighter", 8.0, 3.0, 5.0),
            fake(7, "OutroMago", "Mage", 2.0, 9.0, 4.0),
        ];
        ChampionIndex {
            version: "test".into(),
            by_id: champions.into_iter().map(|c| (c.id, c)).collect::<HashMap<_, _>>(),
        }
    }

    fn slot(cell_id: i64, position: &str, champion_id: i64, intent_id: i64, is_local: bool, locked: bool) -> DraftSlot {
        DraftSlot {
            cell_id,
            position: position.into(),
            champion_id,
            intent_id,
            spell1: 4,
            spell2: 14,
            is_local,
            is_acting: false,
            locked,
        }
    }

    fn draft(my_team: Vec<DraftSlot>, their_team: Vec<DraftSlot>) -> DraftState {
        DraftState {
            phase: "BAN_PICK".into(),
            time_left_ms: 27_000,
            total_time_ms: 30_000,
            local_cell_id: 0,
            local_position: my_team
                .iter()
                .find(|s| s.is_local)
                .map(|s| s.position.clone())
                .unwrap_or_default(),
            my_team,
            their_team,
            my_bans: vec![],
            their_bans: vec![],
            current_action: Some("pick".into()),
            is_local_turn: true,
            unavailable: vec![],
        }
    }

    fn ctx(d: &DraftState, pool: &[i64]) -> DraftContext {
        DraftContext::new(d, &indice(), &NoStats, pool, Role::from_lcu(&d.local_position))
    }

    #[test]
    fn a_rota_do_inimigo_sai_de_quem_ele_e_nao_da_ordem() {
        // Atirador, assassino e suporte escolhidos nessa ordem, sem posição.
        // Pela ordem do draft seriam topo, selva e meio.
        let d = draft(
            vec![slot(0, "middle", 0, 0, true, false)],
            vec![
                slot(5, "", 1, 0, false, true),
                slot(6, "", 2, 0, false, true),
                slot(7, "", 3, 0, false, true),
            ],
        );
        let c = ctx(&d, &[]);
        let rota = |id: i64| c.enemies.iter().find(|p| p.champion_id == id).unwrap().role;
        assert_eq!(rota(1), Role::Bot);
        assert_eq!(rota(3), Role::Support);
        assert!(matches!(rota(2), Role::Jungle | Role::Mid));
        assert!(c.roles_assumed);
    }

    #[test]
    fn posicao_informada_nunca_e_contestada() {
        // O cliente diz que o atirador está no meio: fica no meio.
        let d = draft(
            vec![slot(0, "middle", 0, 0, true, false)],
            vec![slot(5, "middle", 1, 0, false, true)],
        );
        let c = ctx(&d, &[]);
        assert_eq!(c.enemies[0].role, Role::Mid);
        assert!(!c.roles_assumed);
        assert_eq!(c.lane_opponent().map(|p| p.champion_id), Some(1));
    }

    #[test]
    fn o_empate_da_sempre_a_mesma_resposta() {
        let d = draft(
            vec![slot(0, "bottom", 0, 0, true, false)],
            vec![slot(5, "", 4, 0, false, true), slot(6, "", 7, 0, false, true)],
        );
        let a = ctx(&d, &[]);
        let b = ctx(&d, &[]);
        assert_eq!(a.enemies, b.enemies);
    }

    #[test]
    fn hover_de_aliado_nao_e_recomendado() {
        // O aliado está em hover no Mago: não pode aparecer para você.
        let d = draft(
            vec![
                slot(0, "middle", 0, 0, true, false),
                slot(1, "top", 0, 4, false, false),
            ],
            vec![],
        );
        let c = ctx(&d, &[]);
        assert!(c.unavailable.contains(&4));
        assert!(!c.candidates.contains(&4));
    }

    #[test]
    fn a_sua_escolha_nao_travada_continua_sua() {
        // O cliente preenche o seu championId antes de travar. Ele não pode
        // sumir da sua própria lista.
        let d = draft(vec![slot(0, "middle", 4, 0, true, false)], vec![]);
        let c = ctx(&d, &[]);
        assert!(!c.unavailable.contains(&4));
        assert!(c.candidates.contains(&4));

        let travado = draft(vec![slot(0, "middle", 4, 0, true, true)], vec![]);
        assert!(ctx(&travado, &[]).unavailable.contains(&4));
    }

    #[test]
    fn so_entra_quem_joga_a_rota_ou_esta_na_pool() {
        let d = draft(vec![slot(0, "middle", 0, 0, true, false)], vec![]);
        let sem_pool = ctx(&d, &[]);
        assert!(sem_pool.candidates.contains(&4), "mago joga o meio");
        assert!(!sem_pool.candidates.contains(&1), "atirador não joga o meio");

        let com_pool = ctx(&d, &[1]);
        assert!(com_pool.candidates.contains(&1), "a pool é escolha do jogador");
    }

    #[test]
    fn rota_desconhecida_nao_filtra() {
        let d = draft(vec![slot(0, "", 0, 0, true, false)], vec![]);
        assert_eq!(ctx(&d, &[]).candidates.len(), indice().by_id.len());
    }

    #[test]
    fn a_rota_vem_do_cliente_da_escolha_ou_da_pool() {
        let pool_meio = Pool {
            entries: vec![PoolEntry { champion_id: 4, role: Some(Role::Mid), tier: Tier::Core }],
        };
        assert_eq!(resolve_role("top", Some(Role::Mid), &pool_meio), Role::Top);
        assert_eq!(resolve_role("", Some(Role::Bot), &pool_meio), Role::Bot);
        assert_eq!(resolve_role("", None, &pool_meio), Role::Mid);
        assert_eq!(resolve_role("", None, &Pool::default()), Role::Unknown);
    }

    #[test]
    fn o_candidato_entra_no_seu_lugar() {
        let d = draft(
            vec![slot(0, "middle", 0, 0, true, false), slot(1, "top", 6, 0, false, true)],
            vec![slot(5, "", 7, 0, false, true)],
        );
        let c = ctx(&d, &[]).with_candidate(4);
        let (nos, eles) = c.lineups(PREGAME_LEVEL);
        assert!(nos.iter().any(|l| l.role == Role::Mid && l.combatant.champion_id == 4));
        assert_eq!(eles.len(), 1);
    }
}
