//! Modelo de dano.
//!
//! As fórmulas abaixo são regras do jogo, não estatística: dado o item que o
//! inimigo comprou, o resultado é exato. É isso que separa a build adaptativa
//! do resto do motor — aqui não há amostra, prior nem intervalo de confiança.
//!
//! A ordem de penetração segue o que o jogo faz hoje, nesta sequência:
//! redução plana, redução percentual, penetração percentual, penetração plana.
//! Redução pode levar a resistência abaixo de zero; penetração não.

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Penetration {
    /// Redução plana, tipo Cutelo Negro. Aplica primeiro e pode negativar.
    pub flat_reduction: f64,
    /// Redução percentual, 0 a 1.
    pub percent_reduction: f64,
    /// Penetração percentual, 0 a 1.
    pub percent: f64,
    /// Penetração plana. Letalidade entra aqui.
    pub flat: f64,
}

/// Resistência que sobra depois de toda a penetração.
pub fn effective_resist(base: f64, pen: &Penetration) -> f64 {
    let mut resist = base - pen.flat_reduction;

    // Efeitos percentuais não mexem em resistência já negativa.
    if resist > 0.0 {
        resist *= 1.0 - pen.percent_reduction.clamp(0.0, 1.0);
    }
    if resist > 0.0 {
        resist *= 1.0 - pen.percent.clamp(0.0, 1.0);
    }

    if resist <= 0.0 {
        return resist;
    }

    // Penetração plana não leva abaixo de zero; só redução faz isso.
    (resist - pen.flat).max(0.0)
}

/// Fração do dano que passa. Resistência negativa amplifica, com retorno
/// decrescente.
pub fn damage_multiplier(resist: f64) -> f64 {
    if resist >= 0.0 {
        100.0 / (100.0 + resist)
    } else {
        2.0 - 100.0 / (100.0 - resist)
    }
}

/// Vida efetiva: quanto de dano bruto o alvo aguenta.
pub fn effective_hp(health: f64, resist: f64) -> f64 {
    health / damage_multiplier(resist)
}

/// Dano que de fato chega, dada a resistência e a penetração do atacante.
pub fn damage_after_resist(raw: f64, base_resist: f64, pen: &Penetration) -> f64 {
    raw * damage_multiplier(effective_resist(base_resist, pen))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn sem_resistencia_o_dano_passa_inteiro() {
        assert!(close(damage_multiplier(0.0), 1.0));
    }

    #[test]
    fn cem_de_resistencia_corta_metade() {
        assert!(close(damage_multiplier(100.0), 0.5));
    }

    #[test]
    fn resistencia_negativa_amplifica_com_retorno_decrescente() {
        // -100 amplifica 50%, não 100%. É o que impede dano infinito.
        assert!(close(damage_multiplier(-100.0), 1.5));
        assert!(damage_multiplier(-1000.0) < 2.0);
    }

    #[test]
    fn vida_efetiva_dobra_com_cem_de_resistencia() {
        assert!(close(effective_hp(1000.0, 100.0), 2000.0));
    }

    #[test]
    fn penetracao_plana_nao_negativa_a_resistencia() {
        let pen = Penetration {
            flat: 50.0,
            ..Default::default()
        };
        assert!(close(effective_resist(30.0, &pen), 0.0));
    }

    #[test]
    fn reducao_plana_negativa_a_resistencia() {
        let pen = Penetration {
            flat_reduction: 50.0,
            ..Default::default()
        };
        assert!(close(effective_resist(30.0, &pen), -20.0));
    }

    #[test]
    fn percentual_nao_mexe_em_resistencia_negativa() {
        // Já negativa pela redução: o percentual não pode "desnegativar".
        let pen = Penetration {
            flat_reduction: 50.0,
            percent: 0.35,
            ..Default::default()
        };
        assert!(close(effective_resist(30.0, &pen), -20.0));
    }

    #[test]
    fn ordem_importa_percentual_antes_da_plana() {
        // 100 de armadura, 35% de pen e 18 de letalidade.
        // Correto: 100 * 0.65 = 65, menos 18 = 47.
        // Se a plana viesse antes: (100 - 18) * 0.65 = 53,3. Não é o que o jogo faz.
        let pen = Penetration {
            percent: 0.35,
            flat: 18.0,
            ..Default::default()
        };
        assert!(close(effective_resist(100.0, &pen), 47.0));
    }

    #[test]
    fn reducao_vem_antes_da_penetracao_percentual() {
        // Cutelo (-30) e depois 35%: (100 - 30) * 0.65 = 45,5.
        let pen = Penetration {
            flat_reduction: 30.0,
            percent: 0.35,
            ..Default::default()
        };
        assert!(close(effective_resist(100.0, &pen), 45.5));
    }

    #[test]
    fn penetracao_percentual_acima_de_um_nao_inverte() {
        let pen = Penetration {
            percent: 5.0,
            ..Default::default()
        };
        assert!(effective_resist(100.0, &pen) >= 0.0);
    }

    #[test]
    fn dano_apos_resistencia_bate_com_a_conta_manual() {
        let pen = Penetration {
            percent: 0.35,
            flat: 18.0,
            ..Default::default()
        };
        // Resistência efetiva 47, multiplicador 100/147.
        let esperado = 1000.0 * (100.0 / 147.0);
        assert!(close(damage_after_resist(1000.0, 100.0, &pen), esperado));
    }

    #[test]
    fn alvo_sem_vida_tem_vida_efetiva_zero() {
        assert!(close(effective_hp(0.0, 200.0), 0.0));
    }
}
