# ADR-0019 — Contra em qualquer rota, e a chance do time num modelo só

- **Status:** aceito
- **Data:** 2026-10-28
- **Decisores:** time do Presage

## Contexto

- Pedido de produto, no modelo do iTero: pesar o candidato pela winrate dele
  contra os campeões inimigos, e com os do próprio time.
- O matchup existia só contra o oponente de rota — e na ranqueada a rota do
  inimigo não é informada ([L-25](../arquitetura/09-licoes-do-prototipo.md)).
- A sinergia com os aliados já existia, medida contra a soma das forças
  ([ADR-0008](0008-matchup-e-sinergia-contra-a-soma-das-forcas.md)).
- O ranking somava em probabilidade (P-10); somar dezenas de efeitos assim sai do
  lugar longe de 50%.

## Decisão

**Uma tabela nova, `versus` — o resultado de A com B em qualquer rota do time
inimigo —, entra no ranking e num modelo de time, tudo em log-odds.**

- **Referência do contra:** a força de cada um somada nas cinco rotas. Usar a rota
  do inimigo exigiria sabê-la; usar 50% para ele traria de volta o vazamento de
  popularidade.
- **Peso de rota:** o inimigo inferido na sua rota entra pelo matchup na proporção
  `w` em que é mesmo da sua rota (1 informado; a fração do encaixe dele quando
  inferido) e pelo contra em `1 − w`. Os outros quatro, só pelo contra. O mesmo
  par nunca conta duas vezes.
- **Log-odds:** cada efeito encolhido entra como `logit(ref + δ) − logit(ref)`.
- **Chance do time** (`engine/team.rs`): os dez campeões, com as mesmas
  referências e forças do ranking. Só com a base, entre 35% e 65%, com `±` e as
  parcelas que mais pesam.
- **Espalhamento do contra:** 1pp até a base existir; o `--verificar` mede o de
  verdade.

## Consequências

- **Positivas:**
  - o candidato é pesado contra o time inimigo inteiro, mesmo sem saber as rotas
    deles;
  - a chance do time com um candidato e o número dele no ranking saem da mesma
    álgebra;
  - sem base nada muda: os testes de antes passaram sem alteração.
- **Negativas:**
  - `versus` é a tabela mais rala: ~170 jogos por par com ~200 mil partidas, ≤0,3pp
    por termo;
  - a chance do time soma ~50 parcelas, e a trava em [0,35; 0,65] é escolhida, não
    medida;
  - o ban continua sem interação: `team_value` do minimax é só composição, e
    [ADR-0013](0013-ban-nao-personalizado-enquanto-aditivo.md) segue valendo. Levar
    o modelo de time ao minimax e ao ban é o passo seguinte.
- **Obrigatório:** `contra_no_esperado_nao_mexe_em_ninguem`,
  `o_oponente_de_rota_nao_conta_duas_vezes`, `contra_de_verdade_vira_peso_e_razao`,
  `a_chance_dos_dois_lados_soma_um`, `sem_base_nao_ha_chance_do_time`,
  `uma_partida_rende_cinquenta_linhas_de_contra`,
  `o_espalhamento_do_contra_e_medido_alem_do_ruido`.

## Alternativas rejeitadas

| Alternativa | Por que não |
|---|---|
| Contra só com o oponente de rota | A rota do inimigo não se sabe na ranqueada |
| Contra com a força do inimigo na rota inferida | Inferência errada viraria referência errada, e o vazamento voltaria |
| Somar matchup e contra do mesmo par por inteiro | O mesmo confronto contado duas vezes |
| Chance do time sem trava nem `±` | Cinquenta parcelas de ruído parecem certeza |
| Prior do contra fixo para sempre | O `--verificar` mede o espalhamento; ele deve substituir o palpite |

## Referências

- [02 §1](../arquitetura/02-motor-de-draft.md#contra-o-time-inimigo-inteiro); [05 §2](../arquitetura/05-coletor.md#2-como-ele-coleta).
