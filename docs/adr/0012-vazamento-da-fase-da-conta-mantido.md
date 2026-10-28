# ADR-0012 — O vazamento da fase da conta fica, por enquanto

- **Status:** aceito
- **Data:** 2026-10-28
- **Decisores:** time do Presage

## Contexto

- O componente pessoal encolhe o seu registro num campeão para a base dele. Esse
  registro carrega a **fase da conta**: numa sequência boa, cada campeão ganha um
  pedaço dela, na proporção do quanto foi jogado — o mesmo erro de referência do
  [ADR-0008](0008-matchup-e-sinergia-contra-a-soma-das-forcas.md), um nível acima.
- A correção óbvia mede cada campeão contra a fase: `σ(logit p_campeão + logit
  p_fase)`.
- Para quem só joga um campeão, a fase **é** o principal. Medido contra ela, o
  principal vai a zero — empatado com campeões nunca jogados, que também valem
  zero.

## Decisão

**Manter a referência atual (a base do campeão) e documentar o vazamento no
código**, até existir um termo de experiência.

O modelo certo separa três coisas: a fase (comum a todo candidato, não ordena
nada), a força do campeão e a sua maestria nele — com campeão nunca jogado
**abaixo** da sua média. Sem esse último termo, os dois erros disponíveis são:

| Referência | Vazamento da fase | Principal × nunca jogado |
|---|---|---|
| base do campeão (mantida) | sim, proporcional à amostra | principal acima — certo |
| fase da conta | não | empatados — errado |

O primeiro ordena melhor.

## Consequências

- **Positivas:** o principal de quem só joga um campeão continua acima do que ele
  nunca jogou.
- **Negativas:** numa sequência boa, os campeões mais jogados sobem um pouco além
  da conta.
- **Obrigatório:** o comentário em `engine/mod.rs`, no ponto do vazamento, diz
  isso. Quando o termo de experiência existir, esta ADR é substituída.

## Alternativas rejeitadas

| Alternativa | Por que não |
|---|---|
| Medir contra a fase da conta | Empata o principal com o nunca jogado |
| Termo de experiência com valor escolhido à mão | Número inventado; nenhum dado o calibra ainda ([P-4](../produto/perguntas-em-aberto.md)) |

## Referências

- [02 §5](../arquitetura/02-motor-de-draft.md#5-componente-pessoal-e-razões); [P-4](../produto/perguntas-em-aberto.md).
