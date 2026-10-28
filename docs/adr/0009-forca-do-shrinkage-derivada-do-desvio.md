# ADR-0009 — A força do shrinkage sai do desvio

- **Status:** aceito
- **Data:** 2026-10-28
- **Decisores:** time do Presage

## Contexto

- Cada componente tinha duas constantes: a força da Beta (`*_STRENGTH`, que puxa a
  estimativa) e o desvio do prior (`*_PRIOR_SD`, que desenha o `±`). Uma Beta de
  força `s` tem desvio `√(p(1 − p)/(s + 1))`; as duas eram crenças diferentes
  sobre o mesmo efeito:

  | | força ⇒ desvio | desvio do `±` |
  |---|---|---|
  | matchup | 800 ⇒ 1,8pp | 3,0pp |
  | sinergia | 1200 ⇒ 1,4pp | 1,2pp |
  | pessoal | 40 ⇒ 7,8pp | 3,5pp |

- No pessoal, 15V-5D em 20 partidas movia o score em +8,3pp; sob a crença do
  próprio `±`, seriam ~2,2pp ([L-2](../arquitetura/09-licoes-do-prototipo.md)).
- E o `±` de cada componente caía com 1/n, não com 1/√n: com muita amostra,
  prometia certeza demais ([L-3](../arquitetura/09-licoes-do-prototipo.md)).
- `lanes.rs` tinha uma cópia da força de matchup ([L-4](../arquitetura/09-licoes-do-prototipo.md)).

## Decisão

**Uma constante por crença: o espalhamento. A força e o `±` saem dele.**

```text
strength_for_sd(sd)     = 0,25 / sd² − 1
posterior_sd(n, sd)     = √(0,25 / (strength_for_sd(sd) + 1 + n))
```

As forças derivadas: matchup ≈ 277, sinergia ≈ 1735, pessoal ≈ 203. `lanes.rs`
usa o `MATCHUP_PRIOR_SD` do motor. A base fica com `BASE_STRENGTH = 400`, porque o
`±` dela já sai de `Estimate::se`, coerente com a força.

## Consequências

- **Positivas:** a estimativa e o `±` contam a mesma história; sem dado nada muda
  (todo desvio zero, `±` ~11pp); quando a base existir, o espalhamento se mede uma
  vez e as duas coisas acompanham.
- **Negativas:** o matchup passa a pesar mais (força 800 → 277), a sinergia menos,
  o pessoal bem menos; com muita amostra o `±` fica mais largo do que antes — mais
  honesto, não pior.
- **Obrigatório:** `forca_e_desvio_sao_a_mesma_crenca`,
  `desvio_posterior_cai_com_a_raiz_da_amostra`, `historico_pessoal_curto_nao_domina`.
- **Ainda escolhido à mão:** o próprio espalhamento. Com a base, sai de
  `τ² = Var(taxas observadas) − média(p(1 − p)/n)`.

## Alternativas rejeitadas

| Alternativa | Por que não |
|---|---|
| Derivar o desvio da força | Os desvios são a crença interpretável ("o efeito costuma variar 3pp"); a força é um botão em partidas fictícias |
| `const fn` para calcular a força em tempo de compilação | Aritmética de ponto flutuante em `const fn` exige Rust 1.82; o workspace declara 1.77 |

## Referências

- [02 §2](../arquitetura/02-motor-de-draft.md#2-shrinkage); RNF-2.
