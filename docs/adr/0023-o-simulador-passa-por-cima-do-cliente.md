# ADR-0023 — O draft do simulador passa por cima do draft do cliente

- **Status:** aceito
- **Data:** 2026-11-07 (registra a decisão da lacuna G-7, de 2026-10-28)
- **Decisores:** time do Presage

## Contexto

- O iTero tem um simulador de draft (I-27). O Presage precisava do mesmo para
  treinar sem estar num champ select e para mostrar o app sem o cliente.
- Ranking, resposta adversária, bans e prévia já leem o draft de um lugar só
  ([ADR-0017](0017-uma-leitura-do-champ-select.md)). Um simulador com motor
  próprio seria uma segunda leitura, e voltaria o defeito de duas contas
  discordando.

## Decisão

**O simulador manda um `DraftState` montado à mão para a casca
(`set_simulated_draft`), e enquanto ele existe todo comando de draft lê esse
draft no lugar do champ select (`current_draft`).** `None` volta para o cliente.

A casca normaliza o draft montado (`DraftState::normalized`): refaz os
indisponíveis e a sua rota a partir dos slots, para o motor não depender de a
interface acertar os dois. No simulador, o filtro da conta não vale
([ADR-0025](0025-so-quem-a-conta-pode-escolher-por-padrao.md)). Rota de inimigo
em branco é presumida como na ranqueada.

## Consequências

- **Positivas:**
  - o simulador e o champ select são a mesma conta;
  - a demonstração funciona sem cliente.
- **Negativas:**
  - enquanto a aba do simulador está aberta, um champ select de verdade fica
    escondido atrás dele; fechar a aba volta.
- **O que passa a ser obrigatório:** comando de draft novo lê `current_draft`,
  nunca o draft do cliente direto.

## Alternativas rejeitadas

| Alternativa | Por que não |
|---|---|
| Motor separado só no front | Segunda leitura do draft, a causa do [L-26](../arquitetura/09-licoes-do-prototipo.md) |
| Confiar nos indisponíveis montados pela interface | O motor passaria a depender de a tela acertar a regra |

## Referências

- [02 §6](../arquitetura/02-motor-de-draft.md#o-simulador);
  [cruzamento](../referencia/itero/07-cruzamento.md).
