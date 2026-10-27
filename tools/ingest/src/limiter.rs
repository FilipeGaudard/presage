//! Limite de taxa por janela deslizante.
//!
//! A Riot aplica duas janelas ao mesmo tempo — uma curta e uma longa — e
//! declara as duas num cabeçalho: `X-App-Rate-Limit: 20:1,100:120` quer dizer
//! 20 chamadas por segundo *e* 100 a cada 120 segundos. Estourar qualquer uma
//! rende 429, e 429 demais rende chave suspensa.
//!
//! Por isso o limite não vem de constante nossa: vem do cabeçalho da resposta,
//! que é a declaração do próprio servidor. Enquanto ele não chega, vale o que
//! a linha de comando disser.
//!
//! O tempo entra como `f64` de segundos em vez de `Instant` porque `Instant`
//! não se constrói à mão — e um limitador que não dá para testar com relógio
//! falso não dá para testar.

use std::collections::VecDeque;

#[derive(Debug, Clone)]
struct Bucket {
    max: usize,
    window: f64,
    hits: VecDeque<f64>,
}

impl Bucket {
    fn prune(&mut self, now: f64) {
        while let Some(&oldest) = self.hits.front() {
            if now - oldest >= self.window {
                self.hits.pop_front();
            } else {
                break;
            }
        }
    }

    /// Quanto falta esperar para caber mais uma chamada.
    fn wait(&mut self, now: f64) -> f64 {
        self.prune(now);
        if self.hits.len() < self.max {
            return 0.0;
        }
        // A vaga abre quando a chamada mais antiga sai da janela.
        let oldest = *self.hits.front().expect("cheio, então tem pelo menos uma");
        (oldest + self.window - now).max(0.0)
    }

    fn record(&mut self, now: f64) {
        self.hits.push_back(now);
    }
}

#[derive(Debug, Clone, Default)]
pub struct Limiter {
    buckets: Vec<Bucket>,
    /// Até quando tudo está parado por causa de um 429.
    penalty_until: f64,
}

impl Limiter {
    /// Lê o formato do cabeçalho da Riot: `20:1,100:120`.
    ///
    /// Devolve `None` quando não dá para entender nada — aí quem chama mantém
    /// o limite que já tinha, em vez de ficar sem limite nenhum.
    pub fn parse(spec: &str) -> Option<Limiter> {
        let buckets: Vec<Bucket> = spec
            .split(',')
            .filter_map(|par| {
                let (max, window) = par.trim().split_once(':')?;
                let max: usize = max.trim().parse().ok()?;
                let window: f64 = window.trim().parse().ok()?;
                (max > 0 && window > 0.0).then_some(Bucket {
                    max,
                    window,
                    hits: VecDeque::new(),
                })
            })
            .collect();

        (!buckets.is_empty()).then_some(Limiter {
            buckets,
            penalty_until: 0.0,
        })
    }

    /// Troca as janelas mantendo o histórico de chamadas já feitas.
    ///
    /// É o que acontece quando a primeira resposta chega e revela o limite
    /// real: as chamadas que já saíram continuam valendo contra a janela nova.
    pub fn adopt(&mut self, other: Limiter) {
        if other.buckets.is_empty() || self.same_shape(&other) {
            return;
        }
        let historico: Vec<f64> = self
            .buckets
            .iter()
            .max_by_key(|b| b.hits.len())
            .map(|b| b.hits.iter().copied().collect())
            .unwrap_or_default();

        self.buckets = other.buckets;
        for bucket in &mut self.buckets {
            bucket.hits = historico.iter().copied().collect();
        }
    }

    fn same_shape(&self, other: &Limiter) -> bool {
        self.buckets.len() == other.buckets.len()
            && self
                .buckets
                .iter()
                .zip(&other.buckets)
                .all(|(a, b)| a.max == b.max && (a.window - b.window).abs() < f64::EPSILON)
    }

    /// Quanto esperar antes da próxima chamada.
    pub fn wait(&mut self, now: f64) -> f64 {
        let pela_penalidade = (self.penalty_until - now).max(0.0);
        let pelas_janelas = self
            .buckets
            .iter_mut()
            .map(|b| b.wait(now))
            .fold(0.0, f64::max);
        pela_penalidade.max(pelas_janelas)
    }

    pub fn record(&mut self, now: f64) {
        for bucket in &mut self.buckets {
            bucket.record(now);
        }
    }

    /// Veio 429. Para tudo pelo tempo que o servidor pediu.
    ///
    /// Vale para o limitador inteiro, não só para o balde estourado: quando o
    /// servidor diz para esperar, insistir em outra janela é o caminho curto
    /// para a chave ser suspensa.
    pub fn penalize(&mut self, now: f64, seconds: f64) {
        self.penalty_until = self.penalty_until.max(now + seconds.max(0.0));
    }

    /// Chamadas por segundo que dá para sustentar, pela janela mais apertada.
    ///
    /// `20:1,100:120` sustenta 0,83/s, não 20/s: a janela longa é quem manda
    /// no longo prazo, e é ela que decide quanto tempo um crawl vai levar.
    pub fn sustained_rate(&self) -> f64 {
        self.buckets
            .iter()
            .map(|b| b.max as f64 / b.window)
            .fold(f64::INFINITY, f64::min)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn limiter(spec: &str) -> Limiter {
        Limiter::parse(spec).expect("spec válida")
    }

    #[test]
    fn le_o_formato_do_cabecalho_da_riot() {
        let l = limiter("20:1,100:120");
        assert_eq!(l.buckets.len(), 2);
        assert_eq!(l.buckets[0].max, 20);
        assert_eq!(l.buckets[1].window, 120.0);
    }

    #[test]
    fn cabecalho_ilegivel_nao_vira_limitador_sem_limite() {
        assert!(Limiter::parse("").is_none());
        assert!(Limiter::parse("lixo").is_none());
        assert!(Limiter::parse("0:1").is_none(), "zero chamadas não é limite");
        assert!(Limiter::parse("20:0").is_none(), "janela zero não é janela");
    }

    #[test]
    fn cabecalho_meio_quebrado_aproveita_o_que_da() {
        let l = limiter("20:1,lixo,100:120");
        assert_eq!(l.buckets.len(), 2);
    }

    #[test]
    fn dentro_do_limite_nao_espera() {
        let mut l = limiter("3:1");
        for i in 0..3 {
            assert_eq!(l.wait(0.0), 0.0, "chamada {i}");
            l.record(0.0);
        }
    }

    #[test]
    fn ao_encher_espera_a_janela_da_mais_antiga() {
        let mut l = limiter("3:1");
        for _ in 0..3 {
            l.record(0.0);
        }
        assert!((l.wait(0.0) - 1.0).abs() < 1e-9);
        assert!((l.wait(0.4) - 0.6).abs() < 1e-9);
        assert_eq!(l.wait(1.0), 0.0, "a mais antiga saiu da janela");
    }

    #[test]
    fn a_janela_desliza_de_verdade() {
        let mut l = limiter("2:10");
        l.record(0.0);
        l.record(5.0);

        assert!((l.wait(5.0) - 5.0).abs() < 1e-9, "espera a de t=0 sair");
        assert_eq!(l.wait(10.0), 0.0);
        l.record(10.0);
        // Agora as de t=5 e t=10 ocupam o balde.
        assert!((l.wait(10.0) - 5.0).abs() < 1e-9);
    }

    #[test]
    fn manda_a_janela_mais_apertada_das_duas() {
        // Longa cheia, curta vazia: quem decide é a longa.
        let mut l = limiter("10:1,4:60");
        for t in [0.0, 0.1, 0.2, 0.3] {
            l.record(t);
        }
        assert!((l.wait(1.0) - 59.0).abs() < 1e-9);
    }

    #[test]
    fn o_429_para_tudo_pelo_tempo_pedido() {
        let mut l = limiter("100:1");
        l.penalize(10.0, 5.0);

        assert!((l.wait(10.0) - 5.0).abs() < 1e-9);
        assert!((l.wait(13.0) - 2.0).abs() < 1e-9);
        assert_eq!(l.wait(15.0), 0.0);
    }

    #[test]
    fn dois_429_seguidos_nao_encurtam_a_pausa() {
        let mut l = limiter("100:1");
        l.penalize(0.0, 30.0);
        l.penalize(1.0, 2.0);
        assert!((l.wait(1.0) - 29.0).abs() < 1e-9, "a pausa longa manda");
    }

    #[test]
    fn adotar_o_limite_do_servidor_nao_esquece_o_que_ja_saiu() {
        // Começamos achando que cabiam 100 por segundo e disparamos 3. O
        // servidor responde que cabem 2. As 3 já feitas têm de contar.
        let mut l = limiter("100:1");
        for t in [0.0, 0.1, 0.2] {
            l.record(t);
        }
        l.adopt(limiter("2:10"));

        assert!(l.wait(0.3) > 0.0, "o histórico tem de valer contra a janela nova");
        assert!((l.wait(0.3) - 9.7).abs() < 1e-9);
    }

    #[test]
    fn adotar_o_mesmo_limite_nao_mexe_em_nada() {
        let mut l = limiter("20:1,100:120");
        l.record(0.0);
        l.adopt(limiter("20:1,100:120"));
        assert_eq!(l.buckets[0].hits.len(), 1);
    }

    #[test]
    fn adotar_lixo_mantem_o_limite_atual() {
        let mut l = limiter("5:1");
        l.adopt(Limiter::default());
        assert_eq!(l.buckets.len(), 1);
        assert_eq!(l.buckets[0].max, 5);
    }

    #[test]
    fn a_taxa_sustentavel_sai_da_janela_mais_apertada() {
        // A chave de desenvolvedor parece permitir 20/s, mas a janela longa
        // derruba isso para menos de uma por segundo.
        let dev = limiter("20:1,100:120");
        assert!((dev.sustained_rate() - 100.0 / 120.0).abs() < 1e-9);

        let producao = limiter("500:10,30000:600");
        assert!((producao.sustained_rate() - 50.0).abs() < 1e-9);
    }
}
