//! Build adaptativa.
//!
//! Para cada item candidato perguntamos a mesma coisa: quanto este item sobe o
//! seu poder de combate, por ouro gasto, **contra os itens que os inimigos têm
//! agora**. Se o ADC deles comprou penetração de armadura, a armadura cai no
//! ranking sozinha. Se o mago fechou Cajado do Vazio, resistência mágica sobe.
//!
//! Poder é vida efetiva vezes dano efetivo. O produto é o que importa: dobrar
//! o dano e dobrar a vida valem o mesmo numa luta, e um item que só sobe um dos
//! dois compete de igual para igual com um que sobe o outro.
//!
//! O lado ofensivo é um proxy, não DPS: soma dano de ataque e poder de
//! habilidade passados pelas resistências reais do alvo, sem razões de
//! habilidade, recarga ou crítico. Serve para ordenar itens, não para prever o
//! dano de um combo.

use serde::Serialize;

use super::champion::ChampionIndex;
use super::damage::{damage_multiplier, effective_resist, Penetration};
use super::items::{ItemIndex, ItemStats};

/// Abaixo disto são consumíveis, totens e componentes baratos.
const MIN_ITEM_COST: f64 = 1000.0;

#[derive(Debug, Clone, PartialEq)]
pub struct Combatant {
    pub champion_id: i64,
    pub level: u8,
    pub items: Vec<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Loadout {
    pub health: f64,
    pub armor: f64,
    pub magic_resist: f64,
    pub attack_damage: f64,
    pub ability_power: f64,
    pub bonus_attack_damage: f64,
    pub physical_pen: Penetration,
    pub magic_pen: Penetration,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BuildContext {
    /// Fração do dano inimigo que chega como físico, 0 a 1.
    pub incoming_physical_share: f64,
    pub enemy_armor_pen_percent: f64,
    pub enemy_lethality: f64,
    pub enemy_magic_pen_percent: f64,
    pub your_effective_hp: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ItemSuggestion {
    pub item_id: i64,
    pub name: String,
    pub cost: f64,
    /// Ganho de poder de combate por 1000 de ouro, em percentual.
    pub value_per_1000g: f64,
    pub ehp_gain: f64,
    pub offense_gain_percent: f64,
    pub reasons: Vec<String>,
}

pub fn loadout(
    combatant: &Combatant,
    champions: &ChampionIndex,
    items: &ItemIndex,
) -> Option<Loadout> {
    let champion = champions.get(combatant.champion_id)?;
    let gear = items.totals(&combatant.items);
    Some(loadout_from(champion.base.health_at(combatant.level), champion.base.armor_at(combatant.level), champion.base.magic_resist_at(combatant.level), champion.base.attack_damage_at(combatant.level), gear))
}

fn loadout_from(health: f64, armor: f64, magic_resist: f64, attack_damage: f64, gear: ItemStats) -> Loadout {
    Loadout {
        health: health + gear.health,
        armor: armor + gear.armor,
        magic_resist: magic_resist + gear.magic_resist,
        attack_damage: attack_damage + gear.attack_damage,
        ability_power: gear.ability_power,
        bonus_attack_damage: gear.attack_damage,
        physical_pen: gear.physical_penetration(),
        magic_pen: gear.magic_penetration(),
    }
}

/// Vida efetiva contra a mistura de dano que este time inimigo entrega, já
/// descontada a penetração que eles carregam.
pub fn effective_hp_against(
    me: &Loadout,
    physical_share: f64,
    enemy_physical_pen: &Penetration,
    enemy_magic_pen: &Penetration,
) -> f64 {
    let share = physical_share.clamp(0.0, 1.0);
    let through_armor = damage_multiplier(effective_resist(me.armor, enemy_physical_pen));
    let through_mr = damage_multiplier(effective_resist(me.magic_resist, enemy_magic_pen));

    let taken = share * through_armor + (1.0 - share) * through_mr;
    if taken <= 0.0 {
        return f64::INFINITY;
    }
    me.health / taken
}

/// Proxy de saída de dano contra as resistências reais dos alvos.
pub fn offense_against(me: &Loadout, targets: &[Loadout]) -> f64 {
    if targets.is_empty() {
        return me.attack_damage + me.ability_power;
    }

    let sum: f64 = targets
        .iter()
        .map(|target| {
            me.attack_damage * damage_multiplier(effective_resist(target.armor, &me.physical_pen))
                + me.ability_power
                    * damage_multiplier(effective_resist(target.magic_resist, &me.magic_pen))
        })
        .sum();

    sum / targets.len() as f64
}

/// Quanto do dano que vem deste time é físico.
///
/// Os itens mandam quando existem, porque são evidência; sem itens caímos no
/// prior da classe. É isso que faz o número andar durante a partida.
pub fn incoming_physical_share(
    enemies: &[Combatant],
    champions: &ChampionIndex,
    items: &ItemIndex,
) -> f64 {
    let mut weighted = 0.0;
    let mut total_weight = 0.0;

    for enemy in enemies {
        let Some(champion) = champions.get(enemy.champion_id) else {
            continue;
        };
        let gear = items.totals(&enemy.items);
        let from_items = gear.attack_damage + gear.ability_power;

        let share = if from_items > 0.0 {
            gear.attack_damage / from_items
        } else {
            champion.physical_share()
        };

        let weight = champion.base.attack_damage_at(enemy.level) + from_items;
        weighted += share * weight;
        total_weight += weight;
    }

    if total_weight <= 0.0 {
        return 0.5;
    }
    weighted / total_weight
}

/// Poder de combate de um lado contra o outro: soma de vida efetiva vezes dano
/// efetivo, com as resistências e a penetração reais dos dois lados.
pub fn side_power(
    side: &[Combatant],
    foes: &[Combatant],
    champions: &ChampionIndex,
    items: &ItemIndex,
) -> f64 {
    let share = incoming_physical_share(foes, champions, items);
    let (physical_pen, magic_pen) = team_penetration(foes, items);
    let foe_loadouts: Vec<Loadout> = foes
        .iter()
        .filter_map(|foe| loadout(foe, champions, items))
        .collect();

    side.iter()
        .filter_map(|member| loadout(member, champions, items))
        .map(|member| {
            let ehp = effective_hp_against(&member, share, &physical_pen, &magic_pen);
            if !ehp.is_finite() {
                return 0.0;
            }
            ehp * offense_against(&member, &foe_loadouts)
        })
        .sum()
}

/// Chance de o primeiro lado ganhar a escaramuça. Estimativa de primeira ordem:
/// quem tem mais poder vence, e a margem dá a probabilidade.
pub fn skirmish_odds(
    side: &[Combatant],
    foes: &[Combatant],
    champions: &ChampionIndex,
    items: &ItemIndex,
) -> f64 {
    let ours = side_power(side, foes, champions, items);
    let theirs = side_power(foes, side, champions, items);

    if ours + theirs <= 0.0 {
        return 0.5;
    }
    ours / (ours + theirs)
}

pub fn team_penetration(enemies: &[Combatant], items: &ItemIndex) -> (Penetration, Penetration) {
    // O pior caso é o que importa para decidir defesa: basta um deles ter
    // penetração para a sua armadura render menos naquele duelo.
    let mut physical = Penetration::default();
    let mut magic = Penetration::default();

    for enemy in enemies {
        let gear = items.totals(&enemy.items);
        let phys = gear.physical_penetration();
        let mag = gear.magic_penetration();
        physical.percent = physical.percent.max(phys.percent);
        physical.flat = physical.flat.max(phys.flat);
        magic.percent = magic.percent.max(mag.percent);
        magic.flat = magic.flat.max(mag.flat);
    }

    (physical, magic)
}

pub fn suggest(
    you: &Combatant,
    enemies: &[Combatant],
    champions: &ChampionIndex,
    items: &ItemIndex,
    limit: usize,
) -> (BuildContext, Vec<ItemSuggestion>) {
    let Some(mine) = loadout(you, champions, items) else {
        return (empty_context(), Vec::new());
    };

    let share = incoming_physical_share(enemies, champions, items);
    let (enemy_physical_pen, enemy_magic_pen) = team_penetration(enemies, items);
    let targets: Vec<Loadout> = enemies
        .iter()
        .filter_map(|enemy| loadout(enemy, champions, items))
        .collect();

    let base_ehp = effective_hp_against(&mine, share, &enemy_physical_pen, &enemy_magic_pen);
    let base_offense = offense_against(&mine, &targets);
    let base_power = base_ehp * base_offense;

    let context = BuildContext {
        incoming_physical_share: share,
        enemy_armor_pen_percent: enemy_physical_pen.percent,
        enemy_lethality: enemy_physical_pen.flat,
        enemy_magic_pen_percent: enemy_magic_pen.percent,
        your_effective_hp: base_ehp,
    };

    if base_power <= 0.0 || !base_power.is_finite() {
        return (context, Vec::new());
    }

    let owned = &you.items;
    let mut suggestions: Vec<ItemSuggestion> = items
        .by_id
        .values()
        .filter(|item| item.purchasable && item.cost >= MIN_ITEM_COST)
        .filter(|item| !owned.contains(&item.id))
        .map(|item| {
            let champion = champions.get(you.champion_id).expect("checado acima");
            let with_item = loadout_from(
                champion.base.health_at(you.level),
                champion.base.armor_at(you.level),
                champion.base.magic_resist_at(you.level),
                champion.base.attack_damage_at(you.level),
                items.totals(owned).plus(item.stats),
            );

            let ehp = effective_hp_against(&with_item, share, &enemy_physical_pen, &enemy_magic_pen);
            let offense = offense_against(&with_item, &targets);
            let power = ehp * offense;

            let gain = (power / base_power - 1.0) * 100.0;

            ItemSuggestion {
                item_id: item.id,
                name: item.name.clone(),
                cost: item.cost,
                value_per_1000g: round1(gain * 1000.0 / item.cost),
                ehp_gain: round1(ehp - base_ehp),
                offense_gain_percent: round1((offense / base_offense - 1.0) * 100.0),
                reasons: reasons_for(item.stats, &context),
            }
        })
        .filter(|suggestion| suggestion.value_per_1000g.is_finite())
        .collect();

    suggestions.sort_by(|a, b| {
        b.value_per_1000g
            .partial_cmp(&a.value_per_1000g)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.item_id.cmp(&b.item_id))
    });
    suggestions.truncate(limit);

    (context, suggestions)
}

fn reasons_for(stats: ItemStats, context: &BuildContext) -> Vec<String> {
    let mut reasons = Vec::new();
    let physical = context.incoming_physical_share * 100.0;

    if stats.armor > 0.0 && context.incoming_physical_share > 0.6 {
        reasons.push(format!("{physical:.0}% do dano deles é físico"));
    }
    if stats.magic_resist > 0.0 && context.incoming_physical_share < 0.4 {
        reasons.push(format!("{:.0}% do dano deles é mágico", 100.0 - physical));
    }
    if stats.armor > 0.0 && context.enemy_armor_pen_percent > 0.0 {
        reasons.push(format!(
            "Eles têm {:.0}% de penetração de armadura — armadura rende menos",
            context.enemy_armor_pen_percent * 100.0
        ));
    }
    if stats.health > 0.0 && context.enemy_lethality > 0.0 {
        reasons.push(format!(
            "Contra {:.0} de letalidade, vida rende mais que armadura",
            context.enemy_lethality
        ));
    }
    if stats.armor_pen_percent > 0.0 || stats.lethality > 0.0 {
        reasons.push("Fura a resistência que eles já compraram".to_string());
    }
    if stats.magic_pen_percent > 0.0 || stats.magic_pen_flat > 0.0 {
        reasons.push("Fura a resistência mágica deles".to_string());
    }

    reasons.truncate(2);
    reasons
}

fn empty_context() -> BuildContext {
    BuildContext {
        incoming_physical_share: 0.5,
        enemy_armor_pen_percent: 0.0,
        enemy_lethality: 0.0,
        enemy_magic_pen_percent: 0.0,
        your_effective_hp: 0.0,
    }
}

fn round1(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}

#[cfg(test)]
mod tests {
    use super::super::champion::fake;
    use super::super::items::{parse_stats, Item};
    use super::*;
    use std::collections::HashMap;

    const ARMADURA: i64 = 1;
    const RESIST_MAGICA: i64 = 2;
    const PEN_ARMADURA: i64 = 3;
    const VIDA: i64 = 4;
    const DANO: i64 = 5;
    const PODER: i64 = 6;

    fn item(id: i64, name: &str, cost: f64, description: &str) -> Item {
        Item {
            id,
            name: name.to_string(),
            cost,
            purchasable: true,
            stats: parse_stats(description),
        }
    }

    fn items() -> ItemIndex {
        let all = vec![
            item(ARMADURA, "Placas", 2800.0, "<stats><attention>80</attention> Armor</stats>"),
            item(RESIST_MAGICA, "Manto", 2800.0, "<stats><attention>80</attention> Magic Resist</stats>"),
            item(PEN_ARMADURA, "Perfurante", 3000.0, "<stats><attention>40</attention> Attack Damage<br><attention>35%</attention> Armor Penetration</stats>"),
            item(VIDA, "Vitalidade", 2800.0, "<stats><attention>800</attention> Health</stats>"),
            item(DANO, "Lâmina", 3000.0, "<stats><attention>70</attention> Attack Damage</stats>"),
            item(PODER, "Bastão", 3000.0, "<stats><attention>120</attention> Ability Power</stats>"),
        ];
        ItemIndex {
            version: "test".into(),
            by_id: all.into_iter().map(|i| (i.id, i)).collect::<HashMap<_, _>>(),
        }
    }

    fn champions() -> ChampionIndex {
        let all = vec![
            fake(10, "Eu", "Fighter", 7.0, 3.0, 6.0),
            fake(20, "AtiradorAD", "Marksman", 9.0, 2.0, 3.0),
            fake(21, "MagoAP", "Mage", 2.0, 9.0, 4.0),
        ];
        ChampionIndex {
            version: "test".into(),
            by_id: all.into_iter().map(|c| (c.id, c)).collect::<HashMap<_, _>>(),
        }
    }

    fn me() -> Combatant {
        Combatant {
            champion_id: 10,
            level: 11,
            items: vec![],
        }
    }

    fn enemy(champion_id: i64, gear: Vec<i64>) -> Combatant {
        Combatant {
            champion_id,
            level: 11,
            items: gear,
        }
    }

    fn rank_of(suggestions: &[ItemSuggestion], id: i64) -> usize {
        suggestions.iter().position(|s| s.item_id == id).expect("item no ranking")
    }

    #[test]
    fn time_inimigo_ad_puxa_armadura_acima_de_resistencia_magica() {
        let inimigos = vec![enemy(20, vec![DANO]), enemy(20, vec![DANO])];
        let (_, sugestoes) = suggest(&me(), &inimigos, &champions(), &items(), 10);
        assert!(rank_of(&sugestoes, ARMADURA) < rank_of(&sugestoes, RESIST_MAGICA));
    }

    #[test]
    fn inimigo_comprando_ap_inverte_a_prioridade() {
        let inimigos = vec![enemy(21, vec![PODER]), enemy(21, vec![PODER])];
        let (_, sugestoes) = suggest(&me(), &inimigos, &champions(), &items(), 10);
        assert!(rank_of(&sugestoes, RESIST_MAGICA) < rank_of(&sugestoes, ARMADURA));
    }

    #[test]
    fn penetracao_inimiga_derruba_o_valor_da_armadura() {
        // Este é o caso que define a feature: mesmo time, mesma composição de
        // dano, só muda o item que eles compraram.
        let sem_pen = vec![enemy(20, vec![DANO]), enemy(20, vec![DANO])];
        let com_pen = vec![enemy(20, vec![PEN_ARMADURA]), enemy(20, vec![PEN_ARMADURA])];

        let (_, a) = suggest(&me(), &sem_pen, &champions(), &items(), 10);
        let (_, b) = suggest(&me(), &com_pen, &champions(), &items(), 10);

        let antes = a.iter().find(|s| s.item_id == ARMADURA).unwrap().value_per_1000g;
        let depois = b.iter().find(|s| s.item_id == ARMADURA).unwrap().value_per_1000g;
        assert!(depois < antes, "antes {antes}, depois {depois}");
    }

    #[test]
    fn contra_letalidade_vida_sobe_em_relacao_a_armadura() {
        let items = items();
        let inimigos = vec![enemy(20, vec![PEN_ARMADURA]), enemy(20, vec![PEN_ARMADURA])];
        let (contexto, sugestoes) = suggest(&me(), &inimigos, &champions(), &items, 10);

        assert!(contexto.enemy_armor_pen_percent > 0.0);
        assert!(rank_of(&sugestoes, VIDA) < rank_of(&sugestoes, ARMADURA));
    }

    #[test]
    fn inimigo_com_armadura_valoriza_penetracao_para_mim() {
        let items = items();
        let sem_armadura = vec![enemy(20, vec![DANO])];
        let com_armadura = vec![enemy(20, vec![ARMADURA])];

        let (_, a) = suggest(&me(), &sem_armadura, &champions(), &items, 10);
        let (_, b) = suggest(&me(), &com_armadura, &champions(), &items, 10);

        let antes = a.iter().find(|s| s.item_id == PEN_ARMADURA).unwrap().value_per_1000g;
        let depois = b.iter().find(|s| s.item_id == PEN_ARMADURA).unwrap().value_per_1000g;
        assert!(depois > antes, "antes {antes}, depois {depois}");
    }

    #[test]
    fn item_que_ja_tenho_sai_da_lista() {
        let meu = Combatant {
            items: vec![ARMADURA],
            ..me()
        };
        let (_, sugestoes) = suggest(&meu, &[enemy(20, vec![DANO])], &champions(), &items(), 10);
        assert!(sugestoes.iter().all(|s| s.item_id != ARMADURA));
    }

    #[test]
    fn sem_itens_inimigos_cai_no_prior_da_classe() {
        let champions = champions();
        let items = items();
        let so_ad = incoming_physical_share(&[enemy(20, vec![])], &champions, &items);
        let so_ap = incoming_physical_share(&[enemy(21, vec![])], &champions, &items);

        assert!(so_ad > 0.7, "{so_ad}");
        assert!(so_ap < 0.5, "{so_ap}");
    }

    #[test]
    fn share_anda_quando_o_inimigo_compra() {
        let champions = champions();
        let items = items();
        let antes = incoming_physical_share(&[enemy(20, vec![])], &champions, &items);
        let depois = incoming_physical_share(&[enemy(20, vec![PODER])], &champions, &items);
        assert!(depois < antes, "antes {antes}, depois {depois}");
    }

    #[test]
    fn campeao_desconhecido_devolve_vazio_sem_panico() {
        let desconhecido = Combatant {
            champion_id: 9999,
            level: 11,
            items: vec![],
        };
        let (_, sugestoes) = suggest(&desconhecido, &[], &champions(), &items(), 10);
        assert!(sugestoes.is_empty());
    }

    #[test]
    fn sem_inimigos_ainda_ranqueia() {
        let (_, sugestoes) = suggest(&me(), &[], &champions(), &items(), 10);
        assert!(!sugestoes.is_empty());
    }

    #[test]
    fn limite_e_respeitado_e_a_ordem_e_estavel() {
        let inimigos = vec![enemy(20, vec![DANO])];
        let (_, a) = suggest(&me(), &inimigos, &champions(), &items(), 3);
        let (_, b) = suggest(&me(), &inimigos, &champions(), &items(), 3);
        assert_eq!(a.len(), 3);
        assert_eq!(a, b);
    }

    #[test]
    fn a_razao_cita_o_numero_real() {
        let inimigos = vec![enemy(20, vec![DANO]), enemy(20, vec![DANO])];
        let (_, sugestoes) = suggest(&me(), &inimigos, &champions(), &items(), 10);
        let armadura = sugestoes.iter().find(|s| s.item_id == ARMADURA).unwrap();
        assert!(
            armadura.reasons.iter().any(|r| r.contains("físico")),
            "{:?}",
            armadura.reasons
        );
    }
}
