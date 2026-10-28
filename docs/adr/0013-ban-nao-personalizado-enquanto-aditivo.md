# ADR-0013 — O ban não é personalizado enquanto `team_value` for aditiva

- **Status:** aceito
- **Data:** anterior à 0.7.0 (registrada em 2026-10-28)
- **Decisores:** time do Presage

## Contexto

- A promessa natural de um ban é "bana o que bate no seu pool".
- `team_value` é aditiva: a composição deles e a sua não interagem.

## Decisão

**`suggest_bans` ordena por quanto o campeão completa a composição deles, e não
finge que faz mais.** E devolve vazio enquanto o outro lado não travou nada.

Na álgebra de uma função aditiva, "quanto X bate no seu pool" é idêntico a "quanto
X ajuda eles" — o pool não muda nada na conta. Separar os dois exige o termo de
interação, que é exatamente a tabela de matchup que a base vai trazer.

## Consequências

- **Positivas:** a função não promete o que a conta não sustenta; sem pick e sem
  base, não apresenta ruído como conselho.
- **Negativas:** o ban é igual para qualquer pool.
- **Obrigatório:** um teste trava essa limitação, e **deve falhar** quando o
  matchup entrar — é o aviso de que o ban pode virar pessoal.

## Alternativas rejeitadas

| Alternativa | Por que não |
|---|---|
| Ban pessoal com a conta atual | Mesmo número para qualquer pool, com rótulo de pessoal |
| Ordenar os bans sem nenhum pick inimigo | Todo campeão pontua igual: seria ordenar ruído |

## Referências

- [02 §6](../arquitetura/02-motor-de-draft.md#por-que-o-ban-ainda-não-é-pessoal).
