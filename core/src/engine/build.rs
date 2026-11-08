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

use super::champion::{Champion, ChampionIndex};
use super::damage::{damage_multiplier, effective_resist, Penetration};
use super::items::{ItemIndex, ItemStats};
use super::champion::Role;
use super::dataset::{base_key, Dataset};
use super::stats::{shrink, strength_for_sd};

/// Abaixo disto são consumíveis, totens e componentes baratos.
const MIN_ITEM_COST: f64 = 1000.0;

/// Partidas do campeão na rota abaixo das quais a base não diz o que ele compra.
pub const MIN_BOUGHT_GAMES: u32 = 100;
/// Desvio padrão do efeito de um item na vitória, em fração: a crença antes da
/// amostra. Escolha à mão, como `MATCHUP_PRIOR_SD` (RN-34).
pub const ITEM_PRIOR_SD: f64 = 0.03;

/// O que os jogadores do campeão, na rota, terminam a partida tendo: um item.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BoughtItem {
    pub item_id: i64,
    pub name: String,
    pub cost: f64,
    /// Partidas em que ele estava no inventário final.
    pub games: u32,
    /// Fração das partidas do campeão na rota que terminaram com ele.
    pub share: f64,
    /// Vitória com ele, encolhida para a do campeão na rota (`ITEM_PRIOR_SD`).
    pub winrate: f64,
}

/// A build da base para um campeão numa rota.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BoughtReport {
    pub champion_id: i64,
    pub role: Role,
    /// Partidas do campeão na rota, o denominador de `share`.
    pub games: u32,
    /// Vitória do campeão na rota, a referência de `winrate`.
    pub winrate: f64,
    pub items: Vec<BoughtItem>,
}

/// O que se compra com o campeão na rota, segundo a base: os itens que a loja da
/// Fenda vende (`Item::purchasable`) e custam ao menos `MIN_ITEM_COST`, do mais
/// comprado ao menos, até `limit`. Com `Role::Unknown` (o treino não informa a
/// rota) usa a rota em que o campeão mais joga na base. `None` sem base para o
/// campeão ou com menos de `MIN_BOUGHT_GAMES` partidas: aí a tela fica com a
/// conta de atributos, e diz isso.
///
/// A vitória com um item vem do inventário final, e quem ganha termina com mais
/// itens: todo item sai acima da média do campeão. Serve para comparar um item
/// com outro, não como chance (RN-34).
pub fn bought_by(
    champion_id: i64,
    role: Role,
    dataset: &Dataset,
    items: &ItemIndex,
    limit: usize,
) -> Option<BoughtReport> {
    let role = if role == Role::Unknown {
        super::roles::ROLES
            .iter()
            .copied()
            .filter_map(|r| dataset.base.get(&base_key(champion_id, r)).map(|rec| (r, rec.games)))
            .max_by_key(|(_, g)| *g)?
            .0
    } else {
        role
    };
    let base = *dataset.base.get(&base_key(champion_id, role))?;
    if base.games < MIN_BOUGHT_GAMES {
        return None;
    }
    let referencia = base.wins as f64 / base.games as f64;
    let prefixo = format!("{}:", base_key(champion_id, role));
    let forca = strength_for_sd(ITEM_PRIOR_SD);

    let mut lista: Vec<BoughtItem> = dataset
        .items
        .iter()
        .filter_map(|(chave, rec)| {
            let id: i64 = chave.strip_prefix(&prefixo)?.parse().ok()?;
            let item = items.get(id)?;
            if !item.purchasable || item.cost < MIN_ITEM_COST {
                return None;
            }
            Some(BoughtItem {
                item_id: id,
                name: item.name.clone(),
                cost: item.cost,
                games: rec.games,
                share: rec.games as f64 / base.games as f64,
                winrate: shrink(Some(*rec), referencia, forca).rate,
            })
        })
        .collect();
    lista.sort_by(|a, b| b.games.cmp(&a.games).then(a.item_id.cmp(&b.item_id)));
    lista.truncate(limit);

    Some(BoughtReport {
        champion_id,
        role,
        games: base.games,
        winrate: referencia,
        items: lista,
    })
}

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
    offense_weighted(me, targets, (1.0, 1.0))
}

/// Quanto do dano do campeão é AD e quanto é AP, pelas notas de ataque e magia
/// do Data Dragon (`info.attack`, `info.magic`, de 0 a 10). Sem nota nenhuma,
/// metade de cada, que é a soma um para um de antes.
///
/// Serve à build: um ponto de AP não vira dano para quem bate com AD, e o
/// Bastão, que dá mais pontos por ouro, não pode passar à frente da Lâmina de um
/// lutador (A4). É regra escrita à mão sobre uma nota da Riot, não medição.
fn damage_weights(champion: &Champion) -> (f64, f64) {
    let total = champion.attack + champion.magic;
    if total <= 0.0 {
        return (0.5, 0.5);
    }
    (champion.attack / total, champion.magic / total)
}

fn offense_weighted(me: &Loadout, targets: &[Loadout], (ad, ap): (f64, f64)) -> f64 {
    if targets.is_empty() {
        return me.attack_damage * ad + me.ability_power * ap;
    }

    let sum: f64 = targets
        .iter()
        .map(|target| {
            ad * me.attack_damage * damage_multiplier(effective_resist(target.armor, &me.physical_pen))
                + ap * me.ability_power
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

    // O dano que conta é o do jeito que você bate (A4).
    let weights = champions
        .get(you.champion_id)
        .map(damage_weights)
        .unwrap_or((0.5, 0.5));

    let base_ehp = effective_hp_against(&mine, share, &enemy_physical_pen, &enemy_magic_pen);
    let base_offense = offense_weighted(&mine, &targets, weights);
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
            let offense = offense_weighted(&with_item, &targets, weights);
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
    use super::super::dataset::item_key;
    use super::super::stats::Record;
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

    /// Uma base com o campeão 42 no meio (1000 partidas, 500 vitórias) e no topo
    /// (150), e o que ele terminou tendo no meio.
    fn base_de_itens() -> Dataset {
        let mut d = Dataset { patch: "16.22".into(), matches: 5000, ..Default::default() };
        d.base.insert(base_key(42, Role::Mid), Record::new(1000, 500));
        d.base.insert(base_key(42, Role::Top), Record::new(150, 70));
        for (id, games, wins) in [(PODER, 700, 380), (VIDA, 300, 140), (DANO, 40, 30), (ARMADURA, 900, 470)] {
            d.items.insert(item_key(42, Role::Mid, id), Record::new(games, wins));
        }
        d.items.insert(item_key(42, Role::Top, DANO), Record::new(120, 60));
        d
    }

    #[test]
    fn a_build_da_base_e_o_que_se_compra_na_fenda_do_mais_comprado_ao_menos() {
        let mut idx = items();
        // As placas não se vendem na Fenda (uma cópia da Arena, A54); um componente
        // barato também fica de fora.
        idx.by_id.get_mut(&ARMADURA).unwrap().purchasable = false;
        idx.by_id.insert(77, item(77, "Espada longa", 350.0, ""));
        let mut d = base_de_itens();
        d.items.insert(item_key(42, Role::Mid, 77), Record::new(800, 400));

        let r = bought_by(42, Role::Mid, &d, &idx, 8).unwrap();
        assert_eq!((r.role, r.games), (Role::Mid, 1000));
        assert!((r.winrate - 0.5).abs() < 1e-9);
        let ids: Vec<i64> = r.items.iter().map(|i| i.item_id).collect();
        assert_eq!(ids, vec![PODER, VIDA, DANO], "mais comprado primeiro, só o da Fenda e fechado");
        assert!((r.items[0].share - 0.7).abs() < 1e-9);

        // 380 de 700 é 54,3%; encolhido para os 50% do campeão, fica entre os dois.
        let poder = r.items[0].winrate;
        assert!(poder > 0.5 && poder < 380.0 / 700.0, "{poder}");
        // 30 de 40 é 75%, mas com 40 partidas fica perto dos 50% (53,2%).
        assert!(r.items[2].winrate < 0.55, "{}", r.items[2].winrate);

        assert_eq!(bought_by(42, Role::Mid, &d, &idx, 1).unwrap().items.len(), 1);
    }

    #[test]
    fn sem_amostra_a_base_nao_diz_a_build() {
        let d = base_de_itens();
        let idx = items();
        // O topo tem 150 partidas, acima do mínimo; com 99 não diz nada.
        assert!(bought_by(42, Role::Top, &d, &idx, 8).is_some());
        let mut pouca = d.clone();
        pouca.base.insert(base_key(42, Role::Top), Record::new(MIN_BOUGHT_GAMES - 1, 40));
        assert!(bought_by(42, Role::Top, &pouca, &idx, 8).is_none());
        assert!(bought_by(99, Role::Mid, &d, &idx, 8).is_none(), "campeão fora da base");
        // Sem rota (o treino), vale a rota em que ele mais joga.
        assert_eq!(bought_by(42, Role::Unknown, &d, &idx, 8).unwrap().role, Role::Mid);
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
    fn para_um_campeao_de_ad_a_lamina_fica_acima_do_bastao() {
        // Eu: lutador com ataque 7 e magia 3 no Data Dragon. O Bastão dá mais
        // pontos por ouro que a Lâmina, mas ponto de AP não é dano para quem bate
        // com AD (A4).
        let inimigos = vec![enemy(20, vec![DANO]), enemy(20, vec![DANO])];
        let (_, sugestoes) = suggest(&me(), &inimigos, &champions(), &items(), 10);
        let lamina = sugestoes.iter().find(|s| s.item_id == DANO).unwrap().value_per_1000g;
        let bastao = sugestoes.iter().find(|s| s.item_id == PODER).unwrap().value_per_1000g;
        assert!(lamina > bastao, "Lâmina {lamina} por 1000, Bastão {bastao}");
    }

    #[test]
    fn para_um_mago_o_bastao_fica_acima_da_lamina() {
        let mago = Combatant { champion_id: 21, level: 11, items: vec![] };
        let inimigos = vec![enemy(20, vec![DANO]), enemy(20, vec![DANO])];
        let (_, sugestoes) = suggest(&mago, &inimigos, &champions(), &items(), 10);
        let lamina = sugestoes.iter().find(|s| s.item_id == DANO).unwrap().value_per_1000g;
        let bastao = sugestoes.iter().find(|s| s.item_id == PODER).unwrap().value_per_1000g;
        assert!(bastao > lamina, "Bastão {bastao} por 1000, Lâmina {lamina}");
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
    fn contra_penetracao_percentual_vida_ganha_da_armadura() {
        // A mesma dupla de atiradores, sem e com 35% de penetração de armadura.
        // A penetração percentual corta cada ponto de armadura, e não a vida:
        // contra ela, a vida ganha terreno sobre a armadura.
        //
        // A20: este teste se chamava "contra letalidade", e o fixture não tinha
        // letalidade nenhuma; a vida já ficava acima sem penetração. Letalidade é
        // plana e, nesta conta, faz o contrário: tira o mesmo de toda armadura, e
        // a vida passa a valer menos por ponto. Só deixa de ser assim quando ela
        // passa da sua armadura e o piso em zero desperdiça pontos.
        let items = items();
        let razao = |inimigos: &[Combatant]| {
            let (contexto, s) = suggest(&me(), inimigos, &champions(), &items, 10);
            let v = |id| s.iter().find(|x| x.item_id == id).unwrap().value_per_1000g;
            (contexto.enemy_armor_pen_percent, v(VIDA) / v(ARMADURA))
        };
        let (pen_sem, sem) = razao(&[enemy(20, vec![DANO]), enemy(20, vec![DANO])]);
        let (pen_com, com) = razao(&[enemy(20, vec![PEN_ARMADURA]), enemy(20, vec![PEN_ARMADURA])]);

        assert_eq!(pen_sem, 0.0);
        assert!(pen_com > 0.0);
        assert!(com > sem, "vida/armadura sem penetração {sem}, com {com}");
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
