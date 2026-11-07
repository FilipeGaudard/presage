# ADR-0024 — A rota marcada à mão vence a inferência, e nunca a posição do cliente

- **Status:** aceito
- **Data:** 2026-11-07 (registra a decisão da lacuna G-1, de 2026-10-28)
- **Decisores:** time do Presage

## Contexto

- Na ranqueada o cliente não informa a rota do inimigo. O Presage a infere pela
  atribuição mais plausível ([ADR-0017](0017-uma-leitura-do-champ-select.md)), e
  sem a base a inferência erra lutador puro e escolha fora de rota
  ([P-13](../produto/perguntas-em-aberto.md)).
- O iTero deixa o jogador corrigir a posição do inimigo (I-17).

## Decisão

**O jogador marca a rota de um inimigo no quadro do draft (`set_enemy_role`).** A
ordem de precedência fica assim:

1. A posição informada pelo cliente nunca é contestada.
2. A marca vence a inferência. Duas marcas na mesma rota ficam com a do primeiro
   a escolher.
3. A inferência distribui só as rotas que sobram.

As marcas morrem com o champ select. Elas vivem na casca, não em disco.

## Consequências

- **Positivas:**
  - o erro da inferência tem conserto sem esperar a base;
  - `roles_assumed` continua dizendo à tela o que é presumido.
- **Negativas:**
  - a marca não sobrevive a reiniciar o app no meio do champ select.
- **O que passa a ser obrigatório:** `a_rota_marcada_vence_a_inferida`,
  `a_rota_marcada_tira_a_rota_dos_outros` e os testes de precedência do catálogo
  ([13](../arquitetura/13-testes.md)).

## Alternativas rejeitadas

| Alternativa | Por que não |
|---|---|
| A marca vence também a posição do cliente | A posição do cliente é dado; a marca é palpite do jogador |
| Guardar as marcas em disco | Valem para um champ select só |

## Referências

- [02 §0](../arquitetura/02-motor-de-draft.md#0-o-contexto-do-draft).
