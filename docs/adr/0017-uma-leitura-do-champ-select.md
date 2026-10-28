# ADR-0017 — Uma leitura só do champ select, com a rota do inimigo inferida

- **Status:** aceito
- **Data:** 2026-10-28
- **Decisores:** time do Presage

## Contexto

- O draft não recomendava pela rota ([L-23](../arquitetura/09-licoes-do-prototipo.md)):
  `rank()` pontuava todos os campeões; sem base, todo campeão valia 50% e a
  composição ranqueava sozinha.
- Ranking, minimax e análise de rota liam o champ select cada um a seu modo, e
  discordavam:
  - o ranking contava o hover do aliado como aliado, mas não o tirava da lista
    ([L-26](../arquitetura/09-licoes-do-prototipo.md));
  - o ranking achava o oponente de rota pela posição, que a ranqueada não informa
    do inimigo — o matchup nunca disparava;
  - a análise de rota pareava pela ordem de escolha, que não é rota
    ([L-25](../arquitetura/09-licoes-do-prototipo.md)).
- Em blind e normal o cliente também não informa a sua própria rota.

## Decisão

**`DraftContext` (`engine/context.rs`) lê o champ select uma vez, e ranking,
minimax e análise de rota leem dele.**

- **Sua rota** (`resolve_role`): a do cliente; senão a escolhida na tela; senão a
  da pool, se for de uma rota só.
- **Candidatos:** disponíveis, e `roles::plays` na sua rota **ou** na sua pool.
- **Rota do inimigo:** inferida pela atribuição mais plausível das rotas livres,
  por `role_fit` (a base quando existe, as classes quando não), entre no máximo 120
  combinações. Empate fica com a ordem de escolha. Posição informada nunca é
  contestada.
- **Indisponível:** banido, escolhido, hover de aliado. A sua escolha ainda não
  travada continua sua.
- **A prévia do draft** é o mesmo contexto com o candidato no seu lugar
  (`with_candidate`).

## Consequências

- **Positivas:**
  - a lista é da sua rota; a pool continua passando por cima;
  - o matchup de rota passa a valer na ranqueada;
  - a prévia de um candidato e a análise depois de travar são a mesma conta.
- **Negativas:**
  - sem a base, a inferência erra lutador puro (topo ou selva) e escolha fora de
    rota ([P-13](../produto/perguntas-em-aberto.md)); `roles_assumed` avisa;
  - em blind e normal a lista depende do jogador escolher a rota na tela.
- **Obrigatório:** `a_rota_do_inimigo_sai_de_quem_ele_e_nao_da_ordem`,
  `posicao_informada_nunca_e_contestada`, `hover_de_aliado_nao_e_recomendado`,
  `a_sua_escolha_nao_travada_continua_sua`, `quem_nao_joga_a_rota_fica_de_fora`,
  `a_previa_e_a_mesma_conta_de_quando_ele_estiver_travado`.

## Alternativas rejeitadas

| Alternativa | Por que não |
|---|---|
| Filtrar a rota só no `rank()` | Minimax e análise continuariam discordando dele |
| Rota do inimigo pela ordem de escolha | É pick order, não rota — o erro de antes |
| Esconder a análise até o cliente informar a rota | Na ranqueada ele nunca informa a do inimigo |

## Referências

- [02 §0](../arquitetura/02-motor-de-draft.md#0-o-contexto-do-draft).
