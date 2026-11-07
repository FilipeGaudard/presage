# ADR-0025 — Por padrão, só quem a conta pode escolher

- **Status:** aceito
- **Data:** 2026-11-07 (registra a decisão da lacuna G-6, de 2026-10-28)
- **Decisores:** time do Presage

## Contexto

- Recomendar um campeão que o jogador não tem é ruído que parece conselho. O
  iTero filtra pelos campeões da conta (I-12).
- O champ select já sabe quem a conta pode escolher
  (`/lol-champ-select/v1/all-grid-champions`): os próprios, os do Game Pass e a
  rotação grátis. A rotação não vale na ranqueada.

## Decisão

**Fora do simulador, os candidatos se limitam a `draft::selectable(grade, fila)`.**

- Entram os próprios, os do Game Pass, e a rotação grátis em fila conhecida que
  não seja ranqueada (`RANKED_QUEUES`).
- Fila desconhecida fica do lado seguro, sem a rotação: recomendar quem não dá
  para pegar é pior do que esconder uma opção.
- Grade vazia ou indisponível não tira ninguém.
- A preferência "incluir quem você não tem" (`DraftPrefs::show_all_champions`)
  desliga o filtro.

A leitura da grade é GET; o LCU continua só lido
([ADR-0026](0026-lcu-so-leitura-e-autostart-desligado.md)).

## Consequências

- **Positivas:** a lista é do que o jogador pode de fato travar.
- **Negativas:** uma conta nova vê poucos candidatos; a preferência existe por
  isso.
- **O que passa a ser obrigatório:** os testes de `selectable` e de "grade vazia
  não filtra" no catálogo ([13](../arquitetura/13-testes.md)).

## Alternativas rejeitadas

| Alternativa | Por que não |
|---|---|
| Rotação sempre | Na ranqueada ela não vale |
| Fila desconhecida com rotação | Recomendaria quem não dá para pegar |

## Referências

- [02 §0](../arquitetura/02-motor-de-draft.md#0-o-contexto-do-draft).
