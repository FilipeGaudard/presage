//! Itens e seus atributos.
//!
//! O bloco `stats` do Data Dragon é legado e não traz letalidade, penetração
//! nem haste — o Lord Dominik's aparece lá só com dano de ataque e crítico.
//! O que está completo é a `description`, que carrega o bloco de atributos num
//! formato estável:
//!
//! ```text
//! <stats><attention>35</attention> Attack Damage<br><attention>35%</attention> Armor Penetration</stats>
//! ```
//!
//! Por isso parseamos a descrição, sempre em en_US: os rótulos são a chave, e
//! traduzir quebraria o parser. O nome exibido vem do locale do usuário.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::damage::Penetration;

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemStats {
    pub health: f64,
    pub armor: f64,
    pub magic_resist: f64,
    pub attack_damage: f64,
    pub ability_power: f64,
    pub lethality: f64,
    /// 0 a 1.
    pub armor_pen_percent: f64,
    pub magic_pen_flat: f64,
    /// 0 a 1.
    pub magic_pen_percent: f64,
    /// 0 a 1.
    pub crit_chance: f64,
    /// 0 a 1.
    pub attack_speed: f64,
    pub ability_haste: f64,
}

impl ItemStats {
    pub fn plus(self, other: ItemStats) -> ItemStats {
        ItemStats {
            health: self.health + other.health,
            armor: self.armor + other.armor,
            magic_resist: self.magic_resist + other.magic_resist,
            attack_damage: self.attack_damage + other.attack_damage,
            ability_power: self.ability_power + other.ability_power,
            lethality: self.lethality + other.lethality,
            armor_pen_percent: combine_percent(self.armor_pen_percent, other.armor_pen_percent),
            magic_pen_flat: self.magic_pen_flat + other.magic_pen_flat,
            magic_pen_percent: combine_percent(self.magic_pen_percent, other.magic_pen_percent),
            crit_chance: (self.crit_chance + other.crit_chance).min(1.0),
            attack_speed: self.attack_speed + other.attack_speed,
            ability_haste: self.ability_haste + other.ability_haste,
        }
    }

    pub fn physical_penetration(&self) -> Penetration {
        Penetration {
            flat_reduction: 0.0,
            percent_reduction: 0.0,
            percent: self.armor_pen_percent,
            flat: self.lethality,
        }
    }

    pub fn magic_penetration(&self) -> Penetration {
        Penetration {
            flat_reduction: 0.0,
            percent_reduction: 0.0,
            percent: self.magic_pen_percent,
            flat: self.magic_pen_flat,
        }
    }
}

/// Penetração percentual é multiplicativa no jogo: duas fontes de 35% dão 58%,
/// não 70%.
fn combine_percent(a: f64, b: f64) -> f64 {
    1.0 - (1.0 - a.clamp(0.0, 1.0)) * (1.0 - b.clamp(0.0, 1.0))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub id: i64,
    pub name: String,
    pub cost: f64,
    pub purchasable: bool,
    pub stats: ItemStats,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ItemIndex {
    pub version: String,
    pub by_id: HashMap<i64, Item>,
}

impl ItemIndex {
    pub fn get(&self, id: i64) -> Option<&Item> {
        self.by_id.get(&id)
    }

    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }

    /// Soma dos atributos de uma build. IDs desconhecidos são ignorados.
    pub fn totals(&self, item_ids: &[i64]) -> ItemStats {
        item_ids
            .iter()
            .filter_map(|id| self.get(*id))
            .fold(ItemStats::default(), |acc, item| acc.plus(item.stats))
    }

    pub fn gold_invested(&self, item_ids: &[i64]) -> f64 {
        item_ids
            .iter()
            .filter_map(|id| self.get(*id))
            .map(|item| item.cost)
            .sum()
    }
}

/// Extrai os atributos do bloco `<stats>` de uma descrição em en_US.
pub fn parse_stats(description: &str) -> ItemStats {
    let Some(block) = stats_block(description) else {
        return ItemStats::default();
    };

    let mut stats = ItemStats::default();
    let mut rest = block;

    while let Some(open) = rest.find("<attention>") {
        let after_open = &rest[open + "<attention>".len()..];
        let Some(close) = after_open.find("</attention>") else {
            break;
        };

        let raw_value = after_open[..close].trim();
        let tail = &after_open[close + "</attention>".len()..];

        // O rótulo vai até a próxima tag.
        let label_end = tail.find('<').unwrap_or(tail.len());
        apply(&mut stats, raw_value, tail[..label_end].trim());

        rest = &tail[label_end..];
    }

    stats
}

fn stats_block(description: &str) -> Option<&str> {
    let start = description.find("<stats>")? + "<stats>".len();
    let end = description[start..].find("</stats>")? + start;
    Some(&description[start..end])
}

fn apply(stats: &mut ItemStats, raw: &str, label: &str) {
    let percent = raw.ends_with('%');
    let Ok(value) = raw.trim_end_matches('%').trim().parse::<f64>() else {
        return;
    };
    let fraction = value / 100.0;

    match label {
        "Attack Damage" => stats.attack_damage += value,
        "Ability Power" => stats.ability_power += value,
        "Armor" => stats.armor += value,
        "Magic Resist" | "Magic Resistance" => stats.magic_resist += value,
        "Health" => stats.health += value,
        "Lethality" => stats.lethality += value,
        "Armor Penetration" => stats.armor_pen_percent = combine_percent(stats.armor_pen_percent, fraction),
        "Magic Penetration" => {
            if percent {
                stats.magic_pen_percent = combine_percent(stats.magic_pen_percent, fraction);
            } else {
                stats.magic_pen_flat += value;
            }
        }
        "Critical Strike Chance" => stats.crit_chance += fraction,
        "Attack Speed" => stats.attack_speed += fraction,
        "Ability Haste" => stats.ability_haste += value,
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Descrições reais do patch 16.21.1.
    const LORD_DOMINIKS: &str = "<mainText><stats><attention>35</attention> Attack Damage<br><attention>35%</attention> Armor Penetration<br><attention>25%</attention> Critical Strike Chance</stats><br><br><passive>Giant Slayer</passive><br>Deal up to 15% bonus damage against champions based on their bonus Health.</mainText>";

    const YOUMUUS: &str = "<mainText><stats><attention>55</attention> Attack Damage<br><attention>18</attention> Lethality<br><attention>4%</attention> Move Speed</stats><br><br><passive>Haunt</passive> <br>Gain <speed> Move Speed</speed> while out of combat.</mainText>";

    const VOID_STAFF: &str = "<mainText><stats><attention>95</attention> Ability Power<br><attention>40%</attention> Magic Penetration</stats></mainText>";

    const SORCERERS: &str = "<mainText><stats><attention>18</attention> Magic Penetration</stats></mainText>";

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn le_penetracao_percentual_que_o_bloco_stats_nao_tem() {
        let stats = parse_stats(LORD_DOMINIKS);
        assert!(close(stats.attack_damage, 35.0));
        assert!(close(stats.armor_pen_percent, 0.35));
        assert!(close(stats.crit_chance, 0.25));
    }

    #[test]
    fn le_letalidade() {
        let stats = parse_stats(YOUMUUS);
        assert!(close(stats.attack_damage, 55.0));
        assert!(close(stats.lethality, 18.0));
    }

    #[test]
    fn distingue_pen_magica_plana_de_percentual() {
        assert!(close(parse_stats(VOID_STAFF).magic_pen_percent, 0.40));
        assert!(close(parse_stats(VOID_STAFF).magic_pen_flat, 0.0));
        assert!(close(parse_stats(SORCERERS).magic_pen_flat, 18.0));
        assert!(close(parse_stats(SORCERERS).magic_pen_percent, 0.0));
    }

    #[test]
    fn descricao_sem_bloco_de_stats_nao_entra_em_panico() {
        assert_eq!(parse_stats("<mainText>sem stats</mainText>"), ItemStats::default());
        assert_eq!(parse_stats(""), ItemStats::default());
    }

    #[test]
    fn bloco_truncado_nao_entra_em_panico() {
        assert_eq!(
            parse_stats("<stats><attention>35</attention"),
            ItemStats::default()
        );
    }

    #[test]
    fn rotulo_desconhecido_e_ignorado_sem_perder_o_resto() {
        let raw = "<stats><attention>10</attention> Omnivamp<br><attention>40</attention> Armor</stats>";
        let stats = parse_stats(raw);
        assert!(close(stats.armor, 40.0));
    }

    #[test]
    fn penetracao_percentual_combina_multiplicativo() {
        // Duas fontes de 35% dão 57,75%, não 70%.
        let a = ItemStats {
            armor_pen_percent: 0.35,
            ..Default::default()
        };
        let total = a.plus(a);
        assert!(close(total.armor_pen_percent, 0.5775));
    }

    #[test]
    fn critico_nao_passa_de_cem_por_cento() {
        let a = ItemStats {
            crit_chance: 0.75,
            ..Default::default()
        };
        assert!(close(a.plus(a).crit_chance, 1.0));
    }

    #[test]
    fn letalidade_vira_penetracao_plana() {
        let stats = parse_stats(YOUMUUS);
        let pen = stats.physical_penetration();
        assert!(close(pen.flat, 18.0));
        assert!(close(pen.percent, 0.0));
    }

    #[test]
    fn totais_ignoram_item_desconhecido() {
        let index = ItemIndex {
            version: "test".into(),
            by_id: HashMap::from([(
                3036,
                Item {
                    id: 3036,
                    name: "Lord Dominik's".into(),
                    cost: 3300.0,
                    purchasable: true,
                    stats: parse_stats(LORD_DOMINIKS),
                },
            )]),
        };

        let totals = index.totals(&[3036, 99999]);
        assert!(close(totals.attack_damage, 35.0));
        assert!(close(index.gold_invested(&[3036, 99999]), 3300.0));
    }
}
