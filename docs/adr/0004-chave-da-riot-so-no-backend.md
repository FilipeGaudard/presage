# ADR-0004 — A chave da Riot só no backend

- **Status:** aceito
- **Data:** anterior à 0.7.0 (registrada em 2026-10-28)
- **Decisores:** time do Presage

## Contexto

- A Riot Web API exige chave; o limite de taxa é **por chave, não por usuário**.
- Dados de partida são imutáveis: buscar uma vez serve para todos.
- Produto público com chave de desenvolvimento é proibido (C-10).

## Decisão

**A chave é do desenvolvedor, não do usuário, e em produção vive só num backend
do Presage.** O app pede "as estatísticas do puuid X"; o backend chama a Riot.
Não existe, e não vai existir, tela de cadastro de chave no app.

Enquanto é protótipo, a chave vem de `PRESAGE_RIOT_API_KEY`, nunca de argumento
nem de URL.

## Consequências

- **Positivas:** a chave não vaza do binário; o backend enfileira e distribui o
  limite; o mesmo pipeline que serve o usuário preenche winrate base, matchup e
  sinergia.
- **Negativas:** o produto público precisa de um servidor, que ainda não existe.
- **Obrigatório:** o app funciona sem chave (RNF-5): sem ela, `NoStats` e o motor
  segue só com composição.

## Alternativas rejeitadas

| Alternativa | Por que não |
|---|---|
| Chave embutida no instalador | Qualquer um extrai de um `.exe`; a Riot revoga e todo mundo perde acesso de uma vez |
| Cada usuário com a sua chave | O usuário não deveria precisar de conta de desenvolvedor; e chave de desenvolvimento expira em 24h |
| App chamando a Riot direto com a chave do produto | Mil usuários estouram o limite da chave no primeiro minuto |

## Referências

- [00 §4](../arquitetura/00-contexto-e-limites.md#4-riot-web-api); [07 §3](../arquitetura/07-seguranca.md#3-a-chave-da-riot); RNF-6.
