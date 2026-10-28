# ADR-0010 — O oponente direto como grupo de controle

- **Status:** aceito
- **Data:** 2026-10-28
- **Decisores:** time do Presage

## Contexto

- O componente pessoal só usava vitória e derrota. Cada partida é uma moeda de
  variância 0,25: ver 5pp entre dois campeões seus pede ~400 partidas em cada. E o
  matchmaking puxa toda conta para 50% ([L-6](../arquitetura/09-licoes-do-prototipo.md)).
- A busca já baixava a partida inteira da match-v5 e jogava fora quase tudo.
- Comparar o jogador com "a média" exige uma referência por campeão, rota e elo —
  a base de partidas, que não existe.

## Decisão

**Cada partida é comparada com o oponente da mesma rota no outro time, métrica a
métrica** (`history::played`): tropas aos 10 (selva para o caçador), participação
em abates, fatia de dano, visão por minuto, mortes.

O oponente foi escolhido pelo matchmaking para ter o seu nível, jogou a mesma
partida, a mesma rota, pelo mesmo tempo. E a diferença soma exatamente zero na
população, porque cada confronto entra uma vez com +d e outra com −d: a
referência existe sem base nenhuma.

Os cortes do coletor valem aqui, para vitória e derrota também
([ADR-0006](0006-coletor-sem-bola-de-neve-um-patch-partida-inteira.md)).

## Consequências

- **Positivas:** métricas contínuas que convergem em dezenas de partidas, não
  centenas; dizem *por que* se perde, não só que se perde; nenhuma chamada a mais.
- **Negativas:**
  - o número mistura você e o campeão — quem perde tropa por desenho aparece como
    seu ([04 §6](../arquitetura/04-perfil-de-rota.md#6-o-que-ele-não-separa));
  - suporte não compara tropa; sem `challenges`, sobram as mortes;
  - os nomes de campo ainda não foram conferidos contra uma partida real
    ([P-5](../produto/perguntas-em-aberto.md)).
- **Obrigatório:** `le_os_nomes_de_campo_da_riot` trava os nomes;
  `remake_nao_conta_nem_para_vitoria`.

## Alternativas rejeitadas

| Alternativa | Por que não |
|---|---|
| Winrate por campeão | ~400 partidas por campeão para ver 5pp |
| Ouro e dano por minuto do fim da partida | Medem quem ganhou: quem ganha farma o mapa |
| Timeline para ouro aos 10 e aos 15 | Uma chamada a mais por partida, metade da vazão ([P-3](../produto/perguntas-em-aberto.md)) |
| Comparar com a média da população | Exige a base de partidas |

## Referências

- [04 — Perfil de rota](../arquitetura/04-perfil-de-rota.md).
