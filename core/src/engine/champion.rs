use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Role {
    Top,
    Jungle,
    Mid,
    Bot,
    Support,
    Unknown,
}

impl Role {
    pub fn from_lcu(position: &str) -> Self {
        match position.to_ascii_lowercase().as_str() {
            "top" => Role::Top,
            "jungle" => Role::Jungle,
            "middle" => Role::Mid,
            "bottom" => Role::Bot,
            "utility" => Role::Support,
            _ => Role::Unknown,
        }
    }

    /// Como a rota aparece na interface.
    pub fn short(self) -> &'static str {
        match self {
            Role::Top => "TOP",
            Role::Jungle => "JG",
            Role::Mid => "MID",
            Role::Bot => "ADC",
            Role::Support => "SUP",
            Role::Unknown => "—",
        }
    }
}

/// Crescimento por nível no LoL não é linear:
///
/// ```text
/// valor(n) = base + ganho * n * (0.7025 + 0.0175 * n),  n = nível - 1
/// ```
///
/// No nível 18 o fator vira exatamente 17, então a ponta fecha linear. É a
/// conta que calculadora amadora erra ao somar `ganho * (nível - 1)`.
pub fn grown(base: f64, per_level: f64, level: u8) -> f64 {
    let n = f64::from(level.clamp(1, 18) - 1);
    base + per_level * n * (0.7025 + 0.0175 * n)
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaseStats {
    pub health: f64,
    pub health_per_level: f64,
    pub armor: f64,
    pub armor_per_level: f64,
    pub magic_resist: f64,
    pub magic_resist_per_level: f64,
    pub attack_damage: f64,
    pub attack_damage_per_level: f64,
}

impl BaseStats {
    pub fn health_at(&self, level: u8) -> f64 {
        grown(self.health, self.health_per_level, level)
    }
    pub fn armor_at(&self, level: u8) -> f64 {
        grown(self.armor, self.armor_per_level, level)
    }
    pub fn magic_resist_at(&self, level: u8) -> f64 {
        grown(self.magic_resist, self.magic_resist_per_level, level)
    }
    pub fn attack_damage_at(&self, level: u8) -> f64 {
        grown(self.attack_damage, self.attack_damage_per_level, level)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Champion {
    pub id: i64,
    pub key: String,
    pub name: String,
    pub tags: Vec<String>,
    pub attack: f64,
    pub magic: f64,
    pub defense: f64,
    pub attack_range: f64,
    #[serde(default)]
    pub base: BaseStats,
}

impl Champion {
    pub fn primary_tag(&self) -> &str {
        self.tags.first().map(String::as_str).unwrap_or("")
    }

    /// Fração do dano que sai como físico, de 0 a 1.
    ///
    /// Mistura dois sinais fracos do Data Dragon: a razão attack/magic do campo
    /// `info`, que é grosseira, e um prior pela classe, que acerta melhor os
    /// casos extremos (Jhin pontua attack 10 / magic 6 no `info`, mas o dano
    /// dele é praticamente todo físico).
    ///
    /// Sai de cena quando o pipeline de partidas entregar o split real, que vem
    /// de physicalDamageDealtToChampions vs magicDamageDealtToChampions.
    pub fn physical_share(&self) -> f64 {
        let total = self.attack + self.magic;
        let from_info = if total <= 0.0 { 0.5 } else { self.attack / total };
        let from_class = match self.primary_tag() {
            "Marksman" => 0.90,
            "Mage" => 0.10,
            "Support" => 0.40,
            "Assassin" => 0.55,
            _ => 0.70,
        };
        (0.5 * from_info + 0.5 * from_class).clamp(0.0, 1.0)
    }

    pub fn is_ranged(&self) -> bool {
        self.attack_range >= 500.0
    }

    pub fn is_frontline(&self) -> bool {
        self.tags.iter().any(|tag| tag == "Tank") || self.defense >= 7.0
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChampionIndex {
    pub version: String,
    pub by_id: HashMap<i64, Champion>,
}

impl ChampionIndex {
    pub fn get(&self, id: i64) -> Option<&Champion> {
        self.by_id.get(&id)
    }

    /// Busca pela chave textual (`Annie`, `LeeSin`), que é o que a Live Client
    /// Data API entrega e não muda com o idioma.
    pub fn by_key(&self, key: &str) -> Option<&Champion> {
        self.by_id
            .values()
            .find(|champion| champion.key.eq_ignore_ascii_case(key))
    }

    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }

    pub fn ids(&self) -> Vec<i64> {
        let mut ids: Vec<i64> = self.by_id.keys().copied().collect();
        ids.sort_unstable();
        ids
    }
}

#[cfg(test)]
pub(crate) fn fake(id: i64, name: &str, tag: &str, attack: f64, magic: f64, defense: f64) -> Champion {
    Champion {
        id,
        key: name.to_string(),
        name: name.to_string(),
        tags: vec![tag.to_string()],
        attack,
        magic,
        defense,
        attack_range: if tag == "Marksman" || tag == "Mage" { 550.0 } else { 150.0 },
        base: BaseStats {
            health: 600.0,
            health_per_level: 100.0,
            armor: 30.0,
            armor_per_level: 4.0,
            magic_resist: 32.0,
            magic_resist_per_level: 2.0,
            attack_damage: 60.0,
            attack_damage_per_level: 3.0,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn marksman_pende_para_fisico_mesmo_com_info_ruim() {
        let jhin = fake(202, "Jhin", "Marksman", 10.0, 6.0, 4.0);
        assert!(jhin.physical_share() > 0.75, "{}", jhin.physical_share());
    }

    #[test]
    fn mago_pende_para_magico() {
        let lux = fake(99, "Lux", "Mage", 2.0, 9.0, 4.0);
        assert!(lux.physical_share() < 0.25, "{}", lux.physical_share());
    }

    #[test]
    fn tank_e_frontline_pela_tag() {
        assert!(fake(54, "Malphite", "Tank", 5.0, 7.0, 9.0).is_frontline());
    }

    #[test]
    fn defesa_alta_conta_como_frontline_sem_a_tag() {
        assert!(fake(24, "Jax", "Fighter", 7.0, 3.0, 7.0).is_frontline());
    }

    #[test]
    fn carry_nao_e_frontline() {
        assert!(!fake(202, "Jhin", "Marksman", 10.0, 6.0, 4.0).is_frontline());
    }

    #[test]
    fn champeao_sem_info_nao_divide_por_zero() {
        let vazio = fake(1, "Vazio", "", 0.0, 0.0, 0.0);
        let share = vazio.physical_share();
        assert!((0.0..=1.0).contains(&share));
    }

    #[test]
    fn nivel_um_e_a_base_crua() {
        assert_eq!(grown(600.0, 100.0, 1), 600.0);
    }

    #[test]
    fn nivel_dezoito_fecha_linear() {
        // A propriedade que confirma a fórmula: no 18 o fator é exatamente 17.
        assert!((grown(600.0, 100.0, 18) - (600.0 + 17.0 * 100.0)).abs() < 1e-9);
    }

    #[test]
    fn crescimento_e_sublinear_no_meio() {
        // No nível 10, somar ganho * 9 daria 1500. A fórmula real dá menos.
        let real = grown(600.0, 100.0, 10);
        assert!(real < 1500.0, "{real}");
        assert!(real > 600.0);
    }

    #[test]
    fn nivel_fora_da_faixa_e_limitado() {
        assert_eq!(grown(600.0, 100.0, 0), grown(600.0, 100.0, 1));
        assert_eq!(grown(600.0, 100.0, 99), grown(600.0, 100.0, 18));
    }

    #[test]
    fn stats_base_crescem_juntos() {
        let base = fake(1, "X", "Fighter", 5.0, 5.0, 5.0).base;
        assert!(base.health_at(18) > base.health_at(1));
        assert!(base.armor_at(18) > base.armor_at(1));
        assert!((base.attack_damage_at(18) - (60.0 + 17.0 * 3.0)).abs() < 1e-9);
    }

    #[test]
    fn posicoes_do_lcu() {
        assert_eq!(Role::from_lcu("utility"), Role::Support);
        assert_eq!(Role::from_lcu("MIDDLE"), Role::Mid);
        assert_eq!(Role::from_lcu(""), Role::Unknown);
    }
}
