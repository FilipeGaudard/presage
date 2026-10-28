# ADR-0018 — O que a condição de vitória pode afirmar

- **Status:** aceito
- **Data:** 2026-10-28
- **Decisores:** time do Presage

## Contexto

- Pedido de produto: mostrar a condição de vitória do time no draft, como o
  iTero faz.
- A conta de duelo (`skirmish_odds`) usa atributos e itens. **Não vê habilidade**,
  e por isso não vê quem escala: um campeão que cresce pela suprema ou por acúmulo
  parece fraco no fim por esta conta.
- Ainda não há base de partidas.
- A política da Riot não aceita app que decide pelo jogador (C-5).

## Decisão

**A condição de vitória descreve; não prevê fase sem dado, e não ordena.**

- **Fatos de composição, por regra** (`Source::Regra`): dano de um tipo só (≥70%),
  ninguém na linha de frente, time de alcance ou corpo a corpo. Time com menos de
  três definidos fica sem fatos.
- **Onde a conta de atributos pende** (`Source::Atributos`): rotas e escaramuças a
  mais de 5 pontos de 50%.
- **Fase do jogo** (`phase`): `None` sem a base. Com ela, sai da winrate em
  partidas curtas e longas — medida, não deduzida dos atributos.
- **Cada frase leva a fonte ao lado**, e um aviso fixo diz o que a conta não vê.
- **Nenhum verbo de comando:** um teste varre as frases.

## Consequências

- **Positivas:** nada na tela afirma mais do que a conta sustenta; quem lê sabe de
  onde cada frase sai.
- **Negativas:** sem a base, a condição de vitória é mais magra do que a de
  ferramentas que dizem "seu time é de fim de jogo" — porque elas dizem isso de
  dado, e nós ainda não temos.
- **Obrigatório:** `toda_afirmacao_diz_de_onde_sai`, `nada_aqui_e_ordem`,
  `sem_base_nao_ha_fase`, `time_incompleto_nao_vira_composicao`.

## Alternativas rejeitadas

| Alternativa | Por que não |
|---|---|
| Fase pela curva 5v5 de atributos | Não vê habilidade: o time que escala pela suprema sairia como "de começo" |
| Fase por regra de classe ("atirador escala") | Regra com cara de dado, e erra o campeão excêntrico |
| Plano em imperativo ("force dragão", "evite brigar") | Decide pelo jogador (C-5) |

## Referências

- [03 §4](../arquitetura/03-motor-da-partida.md#condição-de-vitória); `core/src/engine/wincon.rs`.
