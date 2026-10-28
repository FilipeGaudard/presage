# ADR-0002 — Polling em vez de WebSocket no champ select

- **Status:** aceito
- **Data:** anterior à 0.7.0 (registrada em 2026-10-28)
- **Decisores:** time do Presage

## Contexto

- O champ select dura 30 segundos por turno; a recomendação precisa acompanhar
  (RNF-4).
- O LCU oferece WebSocket (`wss://127.0.0.1:<porta>/`, mesma Basic Auth, evento
  `OnJsonApiEvent_lol-champ-select_v1_session`) e também o mesmo estado por GET.
- As credenciais mudam a cada reinício do cliente: reconectar é o caminho normal.

## Decisão

**Polling adaptativo**, com o intervalo pela fase: 350 ms no champ select, 1 s na
partida, 1,5 s com o cliente parado, 3 s sem cliente (`watcher.rs`).

## Consequências

- **Positivas:** 350 ms é tempo real para uma fase de 30 segundos, e é bem mais
  simples de acertar — reconectar é só a próxima requisição.
- **Negativas:** requisições à toa quando nada muda; latência de até 350 ms.
- **Obrigatório:** `rank()` e `minimax` cabem com folga no intervalo, e por isso o
  minimax é podado.

## Alternativas rejeitadas

| Alternativa | Por que não, por enquanto |
|---|---|
| WebSocket do LCU | Vira ganho real quando formos reagir a hover de aliado; até lá, mais estado de conexão para o mesmo resultado |

## Referências

- [01 §3](../arquitetura/01-core-e-casca.md#polling).
