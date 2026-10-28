//! Dois conjuntos de tipos vivem aqui, de propósito.
//!
//! `Lcu*` espelha o JSON cru do cliente da Riot — formato não documentado, que
//! muda sem aviso entre patches. `Draft*` é o modelo de domínio do Presage, que
//! é o que o front consome. A função `to_draft_state` é a única fronteira entre
//! os dois: quando a Riot mexer no formato, só ela quebra.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

// ---------------------------------------------------------------------------
// Formato cru do LCU
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct LcuSession {
    /// Agrupadas por turno: cada turno é uma lista de ações simultâneas.
    pub actions: Vec<Vec<LcuAction>>,
    pub local_player_cell_id: i64,
    pub my_team: Vec<LcuPlayer>,
    pub their_team: Vec<LcuPlayer>,
    pub timer: LcuTimer,
    pub is_spectating: bool,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct LcuAction {
    pub actor_cell_id: i64,
    pub champion_id: i64,
    pub completed: bool,
    pub is_ally_action: bool,
    pub is_in_progress: bool,
    pub pick_turn: i64,
    /// `ban`, `pick` ou `ten_bans_reveal`.
    #[serde(rename = "type")]
    pub action_type: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct LcuPlayer {
    pub cell_id: i64,
    pub champion_id: i64,
    /// O campeão que o jogador está passando o mouse (só visível para aliados).
    pub champion_pick_intent: i64,
    /// `top`, `jungle`, `middle`, `bottom`, `utility` ou vazio em filas sem rota.
    pub assigned_position: String,
    pub spell1_id: i64,
    pub spell2_id: i64,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct LcuTimer {
    pub adjusted_time_left_in_phase: i64,
    pub total_time_in_phase: i64,
    /// `PLANNING`, `BAN_PICK`, `FINALIZATION`.
    pub phase: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct LcuSummoner {
    pub game_name: String,
    pub tag_line: String,
    pub summoner_level: i64,
    pub puuid: String,
}

// ---------------------------------------------------------------------------
// Modelo de domínio do Presage
// ---------------------------------------------------------------------------

// O domínio também desserializa: o simulador de draft monta um `DraftState` na
// interface e manda para o motor, e campo omitido vale o padrão.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct DraftSlot {
    pub cell_id: i64,
    pub position: String,
    /// Campeão travado. 0 = ainda não escolheu.
    pub champion_id: i64,
    /// Campeão em hover. Só aparece para o seu time.
    pub intent_id: i64,
    pub spell1: i64,
    pub spell2: i64,
    pub is_local: bool,
    /// Está agindo neste exato momento (escolhendo ou banindo).
    pub is_acting: bool,
    pub locked: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct DraftState {
    pub phase: String,
    pub time_left_ms: i64,
    pub total_time_ms: i64,
    pub local_cell_id: i64,
    pub local_position: String,
    pub my_team: Vec<DraftSlot>,
    pub their_team: Vec<DraftSlot>,
    pub my_bans: Vec<i64>,
    pub their_bans: Vec<i64>,
    /// `pick` ou `ban` — o que está acontecendo agora.
    pub current_action: Option<String>,
    /// É a sua vez de agir.
    pub is_local_turn: bool,
    /// Todos os campeões indisponíveis (banidos ou já escolhidos). O motor de
    /// recomendação precisa disso para filtrar candidatos.
    pub unavailable: Vec<i64>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ConnStatus {
    pub connected: bool,
    pub phase: String,
    pub summoner: Option<String>,
    pub detail: Option<String>,
}

// ---------------------------------------------------------------------------
// Tradução
// ---------------------------------------------------------------------------

pub fn to_draft_state(session: &LcuSession) -> DraftState {
    let actions: Vec<&LcuAction> = session.actions.iter().flatten().collect();

    let mut my_bans = Vec::new();
    let mut their_bans = Vec::new();
    // Picks concluídos indexados por célula. Necessário porque o time inimigo
    // só expõe o campeão através da ação, não no objeto do jogador.
    let mut locked_picks: HashMap<i64, i64> = HashMap::new();
    let mut acting: Vec<i64> = Vec::new();
    let mut current_action: Option<String> = None;

    for action in &actions {
        match (action.action_type.as_str(), action.completed) {
            ("ban", true) if action.champion_id > 0 => {
                if action.is_ally_action {
                    my_bans.push(action.champion_id);
                } else {
                    their_bans.push(action.champion_id);
                }
            }
            ("pick", true) if action.champion_id > 0 => {
                locked_picks.insert(action.actor_cell_id, action.champion_id);
            }
            _ => {}
        }

        if action.is_in_progress && !action.completed {
            acting.push(action.actor_cell_id);
            // Num mesmo turno todas as ações simultâneas são do mesmo tipo.
            current_action.get_or_insert_with(|| action.action_type.clone());
        }
    }

    let build_slots = |players: &[LcuPlayer]| -> Vec<DraftSlot> {
        players
            .iter()
            .map(|p| {
                let locked_champion = locked_picks.get(&p.cell_id).copied().unwrap_or(0);
                DraftSlot {
                    cell_id: p.cell_id,
                    position: p.assigned_position.clone(),
                    champion_id: if p.champion_id > 0 {
                        p.champion_id
                    } else {
                        locked_champion
                    },
                    intent_id: p.champion_pick_intent,
                    spell1: p.spell1_id,
                    spell2: p.spell2_id,
                    is_local: p.cell_id == session.local_player_cell_id,
                    is_acting: acting.contains(&p.cell_id),
                    locked: locked_champion > 0,
                }
            })
            .collect()
    };

    let my_team = build_slots(&session.my_team);
    let their_team = build_slots(&session.their_team);

    let local_position = my_team
        .iter()
        .find(|s| s.is_local)
        .map(|s| s.position.clone())
        .unwrap_or_default();

    let unavailable = unavailable_of(&my_bans, &their_bans, &my_team, &their_team);

    DraftState {
        phase: session.timer.phase.clone(),
        time_left_ms: session.timer.adjusted_time_left_in_phase,
        total_time_ms: session.timer.total_time_in_phase,
        local_cell_id: session.local_player_cell_id,
        local_position,
        is_local_turn: acting.contains(&session.local_player_cell_id),
        my_team,
        their_team,
        my_bans,
        their_bans,
        current_action,
        unavailable,
    }
}

/// Banidos e escolhidos, sem repetição. A mesma conta para o draft do cliente e
/// para o montado à mão no simulador.
fn unavailable_of(my_bans: &[i64], their_bans: &[i64], my_team: &[DraftSlot], their_team: &[DraftSlot]) -> Vec<i64> {
    let mut unavailable: Vec<i64> = my_bans
        .iter()
        .chain(their_bans.iter())
        .copied()
        .chain(my_team.iter().chain(their_team.iter()).filter_map(|s| {
            (s.champion_id > 0).then_some(s.champion_id)
        }))
        .filter(|id| *id > 0)
        .collect();
    unavailable.sort_unstable();
    unavailable.dedup();
    unavailable
}

impl DraftState {
    /// Refaz `unavailable` e `local_position` a partir dos times e bans. O
    /// simulador chama isso no que a interface mandou, para o motor não
    /// depender de a interface acertar os dois.
    pub fn normalized(mut self) -> Self {
        self.unavailable = unavailable_of(&self.my_bans, &self.their_bans, &self.my_team, &self.their_team);
        if let Some(local) = self.my_team.iter().find(|s| s.is_local) {
            self.local_cell_id = local.cell_id;
            self.local_position = local.position.clone();
        }
        self
    }
}

/// Um campeão da grade do champ select (`/lol-champ-select/v1/all-grid-champions`).
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct LcuGridChampion {
    pub id: i64,
    pub owned: bool,
    pub free_to_play: bool,
    #[serde(rename = "xboxGPReward")]
    pub xbox_gp_reward: bool,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct LcuGameflowSession {
    pub game_data: LcuGameData,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct LcuGameData {
    pub queue: LcuQueue,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct LcuQueue {
    pub id: i64,
}

/// Filas ranqueadas: solo/duo e flex. Nelas a rotação grátis não vale.
pub const RANKED_QUEUES: [i64; 2] = [420, 440];

/// Quem a conta pode escolher nesta fila: os próprios e os do Game Pass; fora
/// da ranqueada, também os da rotação grátis. Fila desconhecida fica do lado
/// seguro — sem a rotação —, porque recomendar quem não dá para pegar é pior do
/// que esconder uma opção.
pub fn selectable(grid: &[LcuGridChampion], queue_id: Option<i64>) -> HashSet<i64> {
    let gratis_vale = queue_id.is_some_and(|q| q > 0 && !RANKED_QUEUES.contains(&q));
    grid.iter()
        .filter(|c| c.id > 0 && (c.owned || c.xbox_gp_reward || (gratis_vale && c.free_to_play)))
        .map(|c| c.id)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn action(cell: i64, champ: i64, kind: &str, completed: bool, ally: bool) -> LcuAction {
        LcuAction {
            actor_cell_id: cell,
            champion_id: champ,
            completed,
            is_ally_action: ally,
            is_in_progress: !completed,
            pick_turn: 1,
            action_type: kind.to_string(),
        }
    }

    fn player(cell: i64, position: &str) -> LcuPlayer {
        LcuPlayer {
            cell_id: cell,
            assigned_position: position.to_string(),
            ..Default::default()
        }
    }

    fn session() -> LcuSession {
        LcuSession {
            local_player_cell_id: 0,
            my_team: vec![player(0, "middle"), player(1, "jungle")],
            their_team: vec![player(5, "middle"), player(6, "jungle")],
            actions: vec![
                vec![
                    action(0, 266, "ban", true, true),
                    action(5, 103, "ban", true, false),
                ],
                vec![action(5, 64, "pick", true, false)],
                vec![action(0, 0, "pick", false, true)],
            ],
            timer: LcuTimer {
                adjusted_time_left_in_phase: 27_000,
                total_time_in_phase: 30_000,
                phase: "BAN_PICK".to_string(),
            },
            is_spectating: false,
        }
    }

    #[test]
    fn separa_bans_por_time() {
        let state = to_draft_state(&session());
        assert_eq!(state.my_bans, vec![266]);
        assert_eq!(state.their_bans, vec![103]);
    }

    #[test]
    fn pick_inimigo_vem_da_acao_nao_do_jogador() {
        // O LCU deixa `championId` zerado em theirTeam; o campeão só aparece na
        // ação concluída. Esta é a regressão mais provável do módulo.
        let state = to_draft_state(&session());
        let lee_sin = state.their_team.iter().find(|s| s.cell_id == 5).unwrap();
        assert_eq!(lee_sin.champion_id, 64);
        assert!(lee_sin.locked);
    }

    #[test]
    fn detecta_a_vez_do_jogador_local() {
        let state = to_draft_state(&session());
        assert!(state.is_local_turn);
        assert_eq!(state.current_action.as_deref(), Some("pick"));
    }

    #[test]
    fn indisponiveis_sem_duplicatas_e_sem_zeros() {
        let state = to_draft_state(&session());
        assert_eq!(state.unavailable, vec![64, 103, 266]);
    }

    #[test]
    fn sessao_vazia_nao_entra_em_panico() {
        let state = to_draft_state(&LcuSession::default());
        assert!(state.my_team.is_empty());
        assert!(!state.is_local_turn);
        assert_eq!(state.current_action, None);
    }

    fn grade() -> Vec<LcuGridChampion> {
        serde_json::from_str(
            r#"[
                {"id": 1, "name": "Próprio", "owned": true, "freeToPlay": false, "xboxGPReward": false, "rented": false},
                {"id": 2, "name": "Grátis", "owned": false, "freeToPlay": true, "xboxGPReward": false, "rented": false},
                {"id": 3, "name": "Game Pass", "owned": false, "freeToPlay": false, "xboxGPReward": true, "rented": false},
                {"id": 4, "name": "Não tem", "owned": false, "freeToPlay": false, "xboxGPReward": false, "rented": false}
            ]"#,
        )
        .unwrap()
    }

    #[test]
    fn na_ranqueada_a_rotacao_gratis_nao_vale() {
        let ids = selectable(&grade(), Some(420));
        assert_eq!(ids, [1, 3].into_iter().collect());
    }

    #[test]
    fn fora_da_ranqueada_a_rotacao_vale() {
        let ids = selectable(&grade(), Some(400));
        assert_eq!(ids, [1, 2, 3].into_iter().collect());
    }

    #[test]
    fn fila_desconhecida_fica_do_lado_seguro() {
        assert_eq!(selectable(&grade(), None), [1, 3].into_iter().collect());
    }

    #[test]
    fn a_fila_sai_da_sessao_do_fluxo_de_jogo() {
        let sessao: LcuGameflowSession =
            serde_json::from_str(r#"{"phase": "ChampSelect", "gameData": {"queue": {"id": 420, "type": "RANKED_SOLO_5x5"}}}"#)
                .unwrap();
        assert_eq!(sessao.game_data.queue.id, 420);
    }

    #[test]
    fn draft_montado_a_mao_ganha_indisponiveis_e_rota() {
        // O simulador manda só o essencial; o resto vem do padrão e da conta.
        let d: DraftState = serde_json::from_str(
            r#"{
                "myTeam": [{"cellId": 0, "position": "middle", "isLocal": true},
                           {"cellId": 1, "position": "top", "championId": 86, "locked": true}],
                "theirTeam": [{"cellId": 5, "championId": 103, "locked": true}],
                "myBans": [64], "theirBans": [0]
            }"#,
        )
        .unwrap();
        let d = d.normalized();
        assert_eq!(d.unavailable, vec![64, 86, 103]);
        assert_eq!(d.local_position, "middle");
        assert_eq!(d.local_cell_id, 0);
    }
}
