# ADR-0007 — Sem dado, o componente vale exatamente zero

- **Status:** aceito
- **Data:** anterior à 0.7.0 (registrada em 2026-10-28)
- **Decisores:** time do Presage

## Contexto

- Três dos cinco componentes do ranking dependem de uma base que ainda não
  existe.
- Ferramentas do gênero mostram número sem dizer de onde. Um número inventado com
  cara de medida é pior que silêncio.

## Decisão

**Todo componente estatístico passa por shrinkage, e sem amostra devolve
exatamente o prior** — o desvio (`shrunk_delta`) é zero por construção. A
interface diz de onde cada número veio.

- A incerteza de um componente sem dado **não** é zero: é o espalhamento típico do
  efeito na população. Por isso o `±` fica em ~11pp sem base.
- Resultados que saem de regra levam `from_data = false`, e a tela não escreve
  "segundo N partidas".
- Rota sem oponente identificado fica de fora em vez de virar 50%.
- O ban devolve vazio enquanto o outro lado não travou nada.

## Consequências

- **Positivas:** o motor é honesto enquanto a base não existe — hoje a ordenação
  vem inteira da composição e o intervalo é largo porque de fato não sabemos; uma
  base parcial recomenda tão bem quanto a parte dela que tem amostra.
- **Negativas:** sem base, o ranking tem pouco a dizer, e diz isso.
- **Obrigatório:** `sem_amostra_o_desvio_e_zero`, `sem_amostra_devolve_o_prior`,
  `sem_dados_o_intervalo_e_largo`, `sem_base_de_partidas_o_numero_vem_so_do_duelo`.

## Alternativas rejeitadas

| Alternativa | Por que não |
|---|---|
| Winrate de site de estatística colada à mão | Número sem amostra conhecida, de outro patch e outro elo |
| Esconder o `±` enquanto não há base | Esconderia justamente que não sabemos |

## Referências

- [02 §2](../arquitetura/02-motor-de-draft.md#2-shrinkage); RNF-1.
