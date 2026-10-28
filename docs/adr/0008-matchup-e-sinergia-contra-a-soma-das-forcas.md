# ADR-0008 — Matchup e sinergia medidos contra a soma das forças

- **Status:** aceito
- **Data:** 2026-10-28
- **Decisores:** time do Presage

## Contexto

- `rank()` encolhia matchup e sinergia para a **base do candidato**. Mas a taxa
  crua de A contra B já carrega a fraqueza geral de B; a de A com C, a força de C.
- O efeito disso cresce com a amostra. Com o rival em 47%, dois candidatos de 50%
  sem nada de especial no confronto: 5000 partidas valiam +2,6pp, 100 partidas
  +0,3pp. O pick popular subia por ser popular, e a diferença tinha o tamanho do
  próprio efeito de matchup (`MATCHUP_PRIOR_SD = 3pp`). Na sinergia, a força do
  aliado vazava para cada candidato, somada sobre quatro aliados ([L-1](../arquitetura/09-licoes-do-prototipo.md)).
- Invisível enquanto a base não existe: todo componente sai multiplicado por zero
  ([L-8](../arquitetura/09-licoes-do-prototipo.md)).

## Decisão

**A referência de um efeito é o que as duas forças já explicam, somadas em
log-odds:**

```text
expected_against(você, rival)   = σ(logit p_você − logit p_rival)
expected_together(você, aliado) = σ(logit p_você + logit p_aliado)
```

O matchup encolhe para `expected_against` com a base do rival na rota; cada
sinergia, para `expected_together` com a base do aliado na rota dele. Rota
desconhecida cai em 50%, e a referência volta a ser a base do candidato.

## Consequências

- **Positivas:** sem efeito de verdade, acrescentar amostra não mexe em ninguém;
  com efeito, ele aparece no tamanho certo.
- **Negativas:** `rank()` passa a ler a base de cada aliado e do rival.
- **Obrigatório:** `popularidade_nao_vira_matchup` e
  `sinergia_nao_herda_a_forca_do_aliado` — os dois falharam no código antigo com
  +2,59pp e +0,86pp antes da correção.

## Alternativas rejeitadas

| Alternativa | Por que não |
|---|---|
| Manter a base do candidato | É o defeito |
| Soma em probabilidade (`p_você + 0,5 − p_rival`) | Aproximação que erra longe de 50%; log-odds é onde forças independentes se somam |
| Corrigir também `lanes.rs` agora | Lá não há ranking entre candidatos; o ajuste mexe no número só com a base e quebra o `from_data` da tela ([P-7](../produto/perguntas-em-aberto.md)) |

## Referências

- [02 §3](../arquitetura/02-motor-de-draft.md#3-a-referência); RNF-3. O perfil de rota tem o mesmo problema um nível abaixo ([04 §6](../arquitetura/04-perfil-de-rota.md#6-o-que-ele-não-separa)).
