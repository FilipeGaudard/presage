# ADR-0016 — Pool de oito: três escolhidos, cinco por cobertura

- **Status:** aceito
- **Data:** 2026-10-28
- **Decisores:** time do Presage

## Contexto

- O construtor seguia o desenho do iTero: dois core e três situacionais
  escolhidos pelo jogador, e um sexto sugerido pelo que faltava.
- A sugestão tinha dois defeitos ([L-22](../arquitetura/09-licoes-do-prototipo.md)):
  - vinha de **todos** os campeões do jogo, sem olhar a rota — a pool de atirador
    sem frente recebia tanque;
  - "buraco" eram quatro testes sim/não (tipo de dano, frente, alcance). Uma pool
    equilibrada de três passava nos quatro e **não recebia sugestão nenhuma**.
- O pedido de produto: os três primeiros dizem como o jogador joga a rota; a pool
  se completa a partir deles.
- A política da Riot pede alternativas, não decisão imposta (C-5).
- Não há base de partidas: nada aqui pode ser winrate (RNF-1).

## Decisão

**A pool tem oito por rota: três core escolhidos pelo jogador e cinco
situacionais recomendados, um lugar por vez.**

- **Retrato dos três** (`analyze`): características do Data Dragon — dano físico
  e mágico, linha de frente, alcance, corpo a corpo, resistências equilibradas,
  seis estilos. Forte é o que a rota valoriza e algum tem; fraco é o que a rota
  valoriza e nenhum tem, ou os três com o mesmo estilo.
- **Cobertura com retorno decrescente** (`suggest`): cada característica cobre
  `1 − ½ⁿ`; o candidato vale `Σ peso × ½ⁿ⁺¹`. Recalculada a cada campeão aceito.
- **Quem joga a rota** (`plays`): com a base, quem tem nela 5% das próprias
  partidas; sem ela, as classes do Data Dragon.
- **O jogador escolhe** entre as alternativas de cada lugar. Nada se preenche
  sozinho, e na revisão tudo continua editável.

## Consequências

- **Positivas:**
  - sempre há o que recomendar até oito, e cada lugar responde a um buraco
    diferente;
  - a pool de uma rota só recebe quem joga a rota;
  - o retrato diz em texto o que os três têm e o que falta, e é conferível.
- **Negativas:**
  - os pesos por rota são escritos à mão — **regra, não estatística**, e a tela diz;
  - classe não é rota: sem a base, o topo de alcance nunca é sugerido;
  - a conta não vê confronto: "contra quem os seus três perdem" é tabela de
    matchup, que só a base traz.
- **Obrigatório:** `cada_escolha_muda_a_proxima`, `so_entra_quem_joga_a_rota`,
  `pool_equilibrada_ainda_recebe_sugestao_ate_oito`, `pool_de_oito_esta_completa`.
  Toda sugestão com motivo escrito.

## Alternativas rejeitadas

| Alternativa | Por que não |
|---|---|
| Preencher os cinco de uma vez, sozinho | Tira a decisão do jogador (C-5); e cinco calculados de uma vez saem para o mesmo buraco |
| Manter os buracos sim/não | Coberto uma vez, o buraco some; uma pool equilibrada fica sem sugestão |
| Recomendar por winrate ou popularidade | Não há base, e "o mais forte do patch" não responde ao que falta na pool |
| Cobertura de confronto com regra de classe ("assassino ganha de mago") | Pedra-papel-tesoura escrito à mão, com cara de matchup; o matchup de verdade vem da base |

## Referências

- [02 §7](../arquitetura/02-motor-de-draft.md#7-pool); `core/src/engine/poolgap.rs`.
