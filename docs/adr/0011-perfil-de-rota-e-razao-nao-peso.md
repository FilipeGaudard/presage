# ADR-0011 — O perfil de rota é razão, não peso

- **Status:** aceito
- **Data:** 2026-10-28
- **Decisores:** time do Presage

## Contexto

- O perfil de rota mede tropas, participação, dano, visão e mortes contra o
  oponente direto ([ADR-0010](0010-oponente-direto-como-grupo-de-controle.md)).
- O ranking de draft é uma probabilidade de vitória.
- Converter "+6 tropas aos 10" em pontos de vitória exige um coeficiente, e o
  único jeito honesto de tê-lo é ajustá-lo contra muitas partidas.

## Decisão

**O perfil entra no draft como texto, nunca no número.** A leitura mais clara do
candidato vira razão — "Sua rota com ele: +6 tropas aos 10 contra o oponente
(n=22)" —, e só se o intervalo de 95% dela não cruza o zero.

Na tela do pós-jogo, a mesma trava decide a cor.

## Consequências

- **Positivas:** nenhum coeficiente inventado (RNF-1); o jogador vê o que o
  histórico dele diz sem o ranking fingir que sabe converter.
- **Negativas:** o perfil não muda a ordem; um campeão em que você domina a rota
  não sobe por isso até a base existir.
- **Obrigatório:** `rota_clara_vira_razao_sem_mexer_na_ordem` (o ranking é idêntico
  com e sem perfil), `rota_dentro_do_ruido_nao_vira_razao`. Nada de nota única que
  some as métricas (C-7).

## Alternativas rejeitadas

| Alternativa | Por que não |
|---|---|
| Somar o perfil ao winrate com um peso escolhido | É o número inventado que o produto inteiro evita |
| Uma nota de rota que combina as cinco métricas | Substituto de ranking, proibido pela Riot |
| Mostrar toda leitura, clara ou não | Com 6 partidas, ruído com cara de diagnóstico |

## Referências

- [04 §5](../arquitetura/04-perfil-de-rota.md#5-na-tela-e-no-draft); [02 §5](../arquitetura/02-motor-de-draft.md#5-componente-pessoal-e-razões).
