//! Draft com resposta do adversário, e bans.
//!
//! O ranking guloso responde "qual o melhor pick agora". Essa não é a pergunta
//! do draft: depois de você travar, eles escolhem contra você. A pergunta certa
//! é "qual pick sobrevive à melhor resposta deles".
//!
//! Então para cada candidato olhamos uma camada à frente e ficamos com o valor
//! **depois** da melhor resposta. A diferença entre os dois números é a
//! exposição do pick — o quanto ele entrega de contra-pick.
//!
//! Os candidatos são os do contexto do draft — disponíveis, e que jogam a sua
//! rota ou estão na sua pool —, podados pelo guloso. As respostas deles são
//! todas as que jogam uma rota que eles ainda não fecharam: varrer tudo é
//! barato, porque `team_value` é aritmética pura. Varrer só os primeiros ids
//! fazia o "melhor contra-pick" sair sempre dos campeões de id baixo (L-24).

use serde::Serialize;
use std::collections::HashSet;

use super::champion::{ChampionIndex, Role};
use super::comp;
use super::context::DraftContext;
use super::roles;
use super::stats::{NoStats, StatsStore};
use crate::draft::DraftState;

/// Peso do desequilíbrio de dano no valor de uma composição.
const W_BALANCE: f64 = 0.06;
/// Peso de não ter frontline nenhuma.
const W_FRONTLINE: f64 = 0.025;
/// A partir de quantos picks travados cobramos frontline.
const FRONTLINE_FROM: usize = 3;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MinimaxPick {
    pub champion_id: i64,
    /// Valor ignorando a resposta deles.
    pub greedy: f64,
    /// Valor depois da melhor resposta deles.
    pub after_response: f64,
    /// Quanto o contra-pick tira. Quanto maior, mais arriscado.
    pub exposure: f64,
    pub best_response: Option<i64>,
    pub in_pool: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BanSuggestion {
    pub champion_id: i64,
    /// Quanto deixar este campeão em jogo custa a você.
    pub threat: f64,
    pub reason: String,
}

/// Valor de uma composição contra a outra, como probabilidade de vitória.
///
/// Simétrica de propósito: o que penaliza um lado premia o outro, então não há
/// viés embutido a favor de quem está sendo avaliado.
pub fn team_value(ours: &[i64], theirs: &[i64], index: &ChampionIndex) -> f64 {
    (0.5 + comp_edge(ours, index) - comp_edge(theirs, index)).clamp(0.05, 0.95)
}

fn comp_edge(team: &[i64], index: &ChampionIndex) -> f64 {
    let profile = comp::profile(team, index);
    if profile.known == 0 {
        return 0.0;
    }

    // Um time de um campeão sempre parece desequilibrado, e isso não quer dizer
    // nada. O sinal só vale o que o draft já revelou.
    let completeness = (profile.known as f64 / 5.0).min(1.0);

    let imbalance = (profile.physical_share - 0.5).abs() * 2.0;
    let mut penalty = imbalance * W_BALANCE;

    if profile.frontline == 0 && profile.known >= FRONTLINE_FROM {
        penalty += W_FRONTLINE;
    }

    -(penalty * completeness)
}

fn locked(slots: &[crate::draft::DraftSlot], skip_local: bool) -> Vec<i64> {
    slots
        .iter()
        .filter(|slot| !(skip_local && slot.is_local))
        .map(|slot| {
            if slot.champion_id > 0 {
                slot.champion_id
            } else {
                slot.intent_id
            }
        })
        .filter(|id| *id > 0)
        .collect()
}

/// O minimax com a rota que o cliente informa e sem base. O app usa
/// `minimax_with`, com o contexto já resolvido.
pub fn minimax(draft: &DraftState, index: &ChampionIndex, pool: &[i64], breadth: usize) -> Vec<MinimaxPick> {
    let role = Role::from_lcu(&draft.local_position);
    let ctx = DraftContext::new(draft, index, &NoStats, pool, role);
    minimax_with(&ctx, index, &NoStats, breadth)
}

/// Ranking com uma camada de resposta adversária.
///
/// `breadth` limita quantos candidatos nossos entram na busca, pelo guloso.
pub fn minimax_with(
    ctx: &DraftContext,
    index: &ChampionIndex,
    store: &dyn StatsStore,
    breadth: usize,
) -> Vec<MinimaxPick> {
    if index.is_empty() {
        return Vec::new();
    }

    let ours = ctx.ally_ids();
    let theirs = ctx.enemy_ids();

    // A resposta deles só pode vir de quem joga uma rota que eles ainda não
    // fecharam. Time inimigo completo não tem resposta.
    let abertas = ctx.open_enemy_roles();
    let respostas: Vec<i64> = index
        .ids()
        .into_iter()
        .filter(|id| !ctx.unavailable.contains(id))
        .filter(|id| {
            index
                .get(*id)
                .is_some_and(|c| abertas.iter().any(|r| roles::plays(c, *r, store)))
        })
        .collect();

    // Poda: ordena pelo guloso e fica com os melhores.
    let mut greedy: Vec<(i64, f64)> = ctx
        .candidates
        .iter()
        .map(|id| {
            let mut mine = ours.clone();
            mine.push(*id);
            (*id, team_value(&mine, &theirs, index))
        })
        .collect();

    greedy.sort_by(|a, b| {
        b.1.partial_cmp(&a.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.0.cmp(&b.0))
    });
    greedy.truncate(breadth.max(1));

    let mut picks: Vec<MinimaxPick> = greedy
        .into_iter()
        .map(|(id, greedy_value)| {
            let mut mine = ours.clone();
            mine.push(id);

            // A melhor resposta deles é a que mais derruba o nosso valor.
            let mut worst = greedy_value;
            let mut best_response = None;

            for response in respostas.iter().filter(|other| **other != id) {
                let mut enemy = theirs.clone();
                enemy.push(*response);
                let value = team_value(&mine, &enemy, index);
                if value < worst {
                    worst = value;
                    best_response = Some(*response);
                }
            }

            MinimaxPick {
                champion_id: id,
                greedy: round3(greedy_value),
                after_response: round3(worst),
                exposure: round3(greedy_value - worst),
                best_response,
                in_pool: ctx.pool.contains(&id),
            }
        })
        .collect();

    picks.sort_by(|a, b| {
        b.after_response
            .partial_cmp(&a.after_response)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.champion_id.cmp(&b.champion_id))
    });

    picks
}

/// Bans ordenados pelo quanto o campeão serve à composição deles.
///
/// Duas honestidades embutidas:
///
/// **Devolve vazio enquanto o outro lado não travou nada.** Sem pick deles e
/// sem base estatística não existe sinal: todo campeão pontua igual, e ordenar
/// isso seria apresentar ruído como conselho.
///
/// **Não é ban personalizado.** `team_value` é aditiva — a composição deles e a
/// sua não interagem — então "quanto X bate no seu pool" é, na álgebra,
/// idêntico a "quanto X ajuda eles". Separar os dois exige o termo de
/// interação, que é justamente a tabela de matchup que o pipeline vai trazer.
/// Até lá, o pool não entra nesta conta, e a função não finge que entra.
pub fn suggest_bans(
    draft: &DraftState,
    index: &ChampionIndex,
    limit: usize,
) -> Vec<BanSuggestion> {
    if index.is_empty() {
        return Vec::new();
    }

    let unavailable: HashSet<i64> = draft.unavailable.iter().copied().collect();
    let ours = locked(&draft.my_team, true);
    let theirs = locked(&draft.their_team, false);

    if theirs.is_empty() {
        return Vec::new();
    }

    let mut suggestions: Vec<BanSuggestion> = index
        .ids()
        .into_iter()
        .filter(|id| !unavailable.contains(id))
        .map(|id| {
            let mut enemy = theirs.clone();
            enemy.push(id);
            let threat = team_value(&ours, &theirs, index) - team_value(&ours, &enemy, index);

            BanSuggestion {
                champion_id: id,
                threat: round3(threat),
                reason: "Completa o que falta na composição deles".to_string(),
            }
        })
        .collect();

    suggestions.sort_by(|a, b| {
        b.threat
            .partial_cmp(&a.threat)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.champion_id.cmp(&b.champion_id))
    });
    suggestions.truncate(limit);
    suggestions
}

fn round3(value: f64) -> f64 {
    (value * 1000.0).round() / 1000.0
}

#[cfg(test)]
mod tests {
    use super::super::champion::fake;
    use super::*;
    use crate::draft::DraftSlot;
    use std::collections::HashMap;

    const MARKS: i64 = 1;
    const FIGHTER: i64 = 2;
    const ASSASSIN: i64 = 3;
    const MAGE: i64 = 4;
    const TANK: i64 = 5;

    fn index() -> ChampionIndex {
        let all = vec![
            fake(MARKS, "Marks", "Marksman", 9.0, 2.0, 3.0),
            fake(FIGHTER, "Lutador", "Fighter", 8.0, 3.0, 5.0),
            fake(ASSASSIN, "Assassino", "Assassin", 9.0, 3.0, 3.0),
            fake(MAGE, "Mago", "Mage", 2.0, 9.0, 4.0),
            fake(TANK, "Tanque", "Tank", 5.0, 6.0, 9.0),
        ];
        ChampionIndex {
            version: "test".into(),
            by_id: all.into_iter().map(|c| (c.id, c)).collect::<HashMap<_, _>>(),
        }
    }

    fn slot(cell_id: i64, champion_id: i64, is_local: bool) -> DraftSlot {
        DraftSlot {
            cell_id,
            position: "middle".into(),
            champion_id,
            intent_id: 0,
            spell1: 4,
            spell2: 14,
            is_local,
            is_acting: false,
            locked: champion_id > 0,
        }
    }

    fn draft(mine: Vec<i64>, theirs: Vec<i64>) -> DraftState {
        let mut my_team = vec![slot(0, 0, true)];
        for (i, id) in mine.iter().enumerate() {
            my_team.push(slot(i as i64 + 1, *id, false));
        }
        let their_team: Vec<DraftSlot> = theirs
            .iter()
            .enumerate()
            .map(|(i, id)| slot(i as i64 + 5, *id, false))
            .collect();

        let mut unavailable: Vec<i64> = mine.iter().chain(theirs.iter()).copied().collect();
        unavailable.sort_unstable();
        unavailable.dedup();

        DraftState {
            phase: "BAN_PICK".into(),
            time_left_ms: 27_000,
            total_time_ms: 30_000,
            local_cell_id: 0,
            local_position: "middle".into(),
            my_team,
            their_team,
            my_bans: vec![],
            their_bans: vec![],
            current_action: Some("pick".into()),
            is_local_turn: true,
            unavailable,
        }
    }

    #[test]
    fn indice_vazio_devolve_vazio() {
        let picks = minimax(&draft(vec![], vec![]), &ChampionIndex::default(), &[], 5);
        assert!(picks.is_empty());
    }

    #[test]
    fn composicao_equilibrada_vale_mais_que_unilateral() {
        let index = index();
        let equilibrada = team_value(&[MARKS, MAGE], &[], &index);
        let unilateral = team_value(&[MARKS, ASSASSIN], &[], &index);
        assert!(equilibrada > unilateral);
    }

    #[test]
    fn a_funcao_de_valor_e_simetrica() {
        let index = index();
        let a = team_value(&[MARKS, MAGE], &[MARKS, ASSASSIN], &index);
        let b = team_value(&[MARKS, ASSASSIN], &[MARKS, MAGE], &index);
        assert!((a + b - 1.0).abs() < 1e-9, "{a} + {b}");
    }

    #[test]
    fn exposicao_nunca_e_negativa() {
        let picks = minimax(&draft(vec![FIGHTER], vec![MARKS]), &index(), &[], 5);
        assert!(picks.iter().all(|pick| pick.exposure >= 0.0), "{picks:?}");
    }

    #[test]
    fn ordena_pelo_valor_depois_da_resposta() {
        let picks = minimax(&draft(vec![FIGHTER], vec![MARKS]), &index(), &[], 5);
        for pair in picks.windows(2) {
            assert!(pair[0].after_response >= pair[1].after_response);
        }
    }

    #[test]
    fn indisponiveis_nao_entram_nem_como_pick_nem_como_resposta() {
        let picks = minimax(&draft(vec![FIGHTER], vec![MARKS]), &index(), &[], 10);
        assert!(picks.iter().all(|pick| pick.champion_id != FIGHTER));
        assert!(picks.iter().all(|pick| pick.champion_id != MARKS));
        assert!(picks
            .iter()
            .all(|pick| pick.best_response != Some(FIGHTER)));
    }

    #[test]
    fn breadth_limita_quantos_candidatos_voltam() {
        let picks = minimax(&draft(vec![], vec![]), &index(), &[], 2);
        assert_eq!(picks.len(), 2);
    }

    #[test]
    fn pool_e_marcado_sem_mudar_a_ordem() {
        // O mago já joga o meio: a pool só o marca.
        let com = minimax(&draft(vec![], vec![]), &index(), &[MAGE], 5);
        let sem = minimax(&draft(vec![], vec![]), &index(), &[], 5);

        assert!(com.iter().find(|p| p.champion_id == MAGE).unwrap().in_pool);
        let a: Vec<i64> = com.iter().map(|p| p.champion_id).collect();
        let b: Vec<i64> = sem.iter().map(|p| p.champion_id).collect();
        assert_eq!(a, b);
    }

    #[test]
    fn so_entra_quem_joga_a_rota() {
        // O lugar é o meio: lutador, tanque e atirador ficam de fora.
        let picks = minimax(&draft(vec![], vec![]), &index(), &[], 10);
        for p in &picks {
            assert!(matches!(p.champion_id, MAGE | ASSASSIN), "{p:?}");
        }
    }

    #[test]
    fn a_resposta_varre_todos_os_campeoes() {
        // Quarenta lutadores de id baixo e um mago-suporte de id alto. Com o
        // atirador deles travado, quem equilibra o dano deles é o mago — e
        // varrer só os primeiros ids nunca chegaria nele.
        let mut idx = index();
        for id in 10..50 {
            idx.by_id.insert(id, fake(id, &format!("Lutador{id}"), "Fighter", 8.0, 3.0, 5.0));
        }
        let mut alto = fake(999, "MagoSuporte", "Mage", 2.0, 9.0, 4.0);
        alto.tags.push("Support".into());
        idx.by_id.insert(999, alto);

        let picks = minimax(&draft(vec![], vec![MARKS]), &idx, &[], 3);
        assert!(
            picks.iter().any(|p| p.best_response == Some(999)),
            "{picks:?}"
        );
    }

    #[test]
    fn resultado_e_deterministico() {
        let a = minimax(&draft(vec![FIGHTER], vec![MARKS]), &index(), &[], 5);
        let b = minimax(&draft(vec![FIGHTER], vec![MARKS]), &index(), &[], 5);
        assert_eq!(a, b);
    }

    #[test]
    fn sem_pick_deles_nao_ha_ban_a_sugerir() {
        // Honestidade antes de utilidade: sem sinal, nada é melhor que ruído.
        assert!(suggest_bans(&draft(vec![], vec![]), &index(), 5).is_empty());
    }

    #[test]
    fn bans_saem_ordenados_pela_ameaca() {
        let bans = suggest_bans(&draft(vec![], vec![MARKS]), &index(), 5);
        assert!(!bans.is_empty());
        for pair in bans.windows(2) {
            assert!(pair[0].threat >= pair[1].threat);
        }
    }

    #[test]
    fn ban_prioriza_quem_completa_o_que_falta_neles() {
        // Com um atirador travado, a comp deles é física demais; um mago
        // resolve isso e por isso ameaça mais que outro físico.
        let bans = suggest_bans(&draft(vec![], vec![MARKS]), &index(), 10);
        let posicao = |id: i64| bans.iter().position(|b| b.champion_id == id).unwrap();
        assert!(posicao(MAGE) < posicao(ASSASSIN));
    }

    #[test]
    fn ban_nao_sugere_campeao_ja_fora() {
        let bans = suggest_bans(&draft(vec![FIGHTER], vec![MARKS]), &index(), 10);
        assert!(bans.iter().all(|ban| ban.champion_id != FIGHTER));
        assert!(bans.iter().all(|ban| ban.champion_id != MARKS));
    }

    #[test]
    fn o_valor_e_aditivo_e_e_por_isso_que_nao_ha_ban_pessoal() {
        // Este teste existe para travar a limitação, não para celebrá-la:
        // enquanto team_value não tiver termo de interação, o quanto um pick
        // nosso sofre com X é igual para todo pick nosso. Quando a tabela de
        // matchup entrar, este teste deve falhar — e aí o ban vira pessoal.
        let index = index();
        let theirs = vec![MARKS];
        let mut com_x = theirs.clone();
        com_x.push(MAGE);

        let sofrido_por_tank =
            team_value(&[TANK], &theirs, &index) - team_value(&[TANK], &com_x, &index);
        let sofrido_por_assassino =
            team_value(&[ASSASSIN], &theirs, &index) - team_value(&[ASSASSIN], &com_x, &index);

        assert!((sofrido_por_tank - sofrido_por_assassino).abs() < 1e-9);
    }

    #[test]
    fn limite_de_bans_e_respeitado() {
        assert_eq!(suggest_bans(&draft(vec![], vec![MARKS]), &index(), 3).len(), 3);
    }
}
