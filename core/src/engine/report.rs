//! A análise de rota inteira: as rotas, as escaramuças e a condição de vitória.
//!
//! Uma montagem só, para a partida e para o draft — e, no draft, para o
//! candidato que se está avaliando: a prévia de um campeão é a mesma conta de
//! quando ele estiver travado no seu lugar (`DraftContext::with_candidate`).

use serde::Serialize;

use super::champion::ChampionIndex;
use super::context::{DraftContext, PREGAME_LEVEL};
use super::items::ItemIndex;
use super::lanes::{self, LaneOutlook, Lineup, Skirmish};
use super::stats::StatsStore;
use super::wincon::{self, WinCondition};

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LaneReport {
    pub lanes: Vec<LaneOutlook>,
    pub skirmishes: Vec<Skirmish>,
    pub win_condition: WinCondition,
    /// `draft` ou `partida`.
    pub source: &'static str,
    /// Verdadeiro quando alguma rota saiu da inferência em vez de vir do
    /// cliente. A interface precisa dizer isso.
    pub roles_assumed: bool,
}

/// Monta a análise a partir dos dois lados já resolvidos. Sem um dos lados
/// não há o que comparar.
pub fn build(
    ours: &[Lineup],
    theirs: &[Lineup],
    source: &'static str,
    roles_assumed: bool,
    champions: &ChampionIndex,
    items: &ItemIndex,
    store: &dyn StatsStore,
) -> Option<LaneReport> {
    if ours.is_empty() || theirs.is_empty() {
        return None;
    }
    let rotas = lanes::lane_outlooks(ours, theirs, champions, items, store);
    let brigas = lanes::skirmishes(ours, theirs, champions, items);
    let win_condition = wincon::analyze(ours, theirs, &rotas, &brigas, champions);
    Some(LaneReport {
        lanes: rotas,
        skirmishes: brigas,
        win_condition,
        source,
        roles_assumed,
    })
}

/// A análise do champ select, com todos no mesmo nível de referência: sem
/// isto a conta compararia níveis inventados.
pub fn from_context(
    ctx: &DraftContext,
    champions: &ChampionIndex,
    items: &ItemIndex,
    store: &dyn StatsStore,
) -> Option<LaneReport> {
    let (ours, theirs) = ctx.lineups(PREGAME_LEVEL);
    build(&ours, &theirs, "draft", ctx.roles_assumed, champions, items, store)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::draft::{DraftSlot, DraftState};
    use crate::engine::champion::{fake, Role};
    use crate::engine::stats::NoStats;
    use std::collections::HashMap;

    fn indice() -> ChampionIndex {
        let champions = vec![
            fake(1, "Lutador", "Fighter", 8.0, 3.0, 5.0),
            fake(2, "Assassino", "Assassin", 9.0, 2.0, 3.0),
            fake(3, "Mago", "Mage", 2.0, 9.0, 4.0),
            fake(4, "Atirador", "Marksman", 9.0, 2.0, 3.0),
            fake(5, "Suporte", "Support", 3.0, 7.0, 5.0),
            fake(6, "Inimigo", "Mage", 3.0, 8.0, 4.0),
            fake(7, "OutroMago", "Mage", 2.0, 9.0, 4.0),
        ];
        ChampionIndex {
            version: "test".into(),
            by_id: champions.into_iter().map(|c| (c.id, c)).collect::<HashMap<_, _>>(),
        }
    }

    fn slot(cell_id: i64, position: &str, champion_id: i64, is_local: bool) -> DraftSlot {
        DraftSlot {
            cell_id,
            position: position.into(),
            champion_id,
            intent_id: 0,
            spell1: 4,
            spell2: 14,
            is_local,
            is_acting: false,
            locked: champion_id > 0,
        }
    }

    fn draft(local: i64) -> DraftState {
        DraftState {
            phase: "BAN_PICK".into(),
            time_left_ms: 27_000,
            total_time_ms: 30_000,
            local_cell_id: 0,
            local_position: "middle".into(),
            my_team: vec![
                slot(0, "middle", local, true),
                slot(1, "top", 1, false),
                slot(2, "jungle", 2, false),
            ],
            // Sem posição, como na ranqueada.
            their_team: vec![slot(5, "", 6, false), slot(6, "", 4, false)],
            my_bans: vec![],
            their_bans: vec![],
            current_action: Some("pick".into()),
            is_local_turn: true,
            unavailable: vec![],
        }
    }

    fn contexto(d: &DraftState) -> DraftContext {
        DraftContext::new(d, &indice(), &NoStats, &[], Role::Mid)
    }

    #[test]
    fn a_previa_e_a_mesma_conta_de_quando_ele_estiver_travado() {
        let idx = indice();
        let it = ItemIndex::default();
        let previa = from_context(&contexto(&draft(0)).with_candidate(7), &idx, &it, &NoStats);
        let travado = from_context(&contexto(&draft(7)), &idx, &it, &NoStats);
        assert!(previa.is_some());
        assert_eq!(previa, travado);
    }

    #[test]
    fn o_candidato_encontra_o_oponente_inferido() {
        // O mago deles não tem posição; a inferência o põe no meio, contra o
        // candidato.
        let idx = indice();
        let r = from_context(&contexto(&draft(0)).with_candidate(7), &idx, &ItemIndex::default(), &NoStats)
            .unwrap();
        let meio = r.lanes.iter().find(|l| l.ally == 7).expect("o candidato tem rota");
        assert_eq!(meio.enemy, 6);
        assert!(r.roles_assumed);
    }

    #[test]
    fn sem_inimigo_travado_nao_ha_analise() {
        let mut d = draft(0);
        d.their_team.clear();
        let r = from_context(&contexto(&d).with_candidate(7), &indice(), &ItemIndex::default(), &NoStats);
        assert!(r.is_none());
    }
}
