# ADR-0021 — O ID do teste nomeia a função, e o nome continua sendo a regra

- **Status:** aceito
- **Data:** 2026-11-07
- **Decisores:** time do Presage

## Contexto

- Os testes do Presage são nomeados como frases em português, e o nome é a regra
  que o teste prova (`popularidade_nao_vira_matchup`). `cargo test <nome>` roda
  só ele ([08 §4](../arquitetura/08-desenvolvimento-e-testes.md#como-os-testes-são-escritos)).
- Faltava o que liga um teste a um requisito. Os docs citavam 55 dos 463 testes,
  e a regra C-4 tinha um teste citado como "um teste".
- O padrão de outros projetos é um ID (`T-…`) num comentário marcador dentro do
  teste. Aqui isso pediria tocar os 463 testes, e o marcador seria uma segunda
  cópia do vínculo.

## Decisão

**Cada linha do catálogo ([13](../arquitetura/13-testes.md)) tem um `T-ÁREA-n` e
nomeia as funções de teste que a provam.** O código não leva marcador: o nome
da função é a chave. O doccheck exige que:

- todo teste nomeado exista;
- todo `#[test]` esteja em exatamente uma linha.

A matriz dos [requisitos](../produto/requisitos.md#4-matriz-de-rastreabilidade)
cita T-IDs; o catálogo diz quais funções cada T-ID é.

## Consequências

- **Positivas:**
  - nenhum teste muda;
  - o nome continua sendo a regra, e `cargo test <nome>` continua rodando um só;
  - teste novo sem linha no catálogo falha o doccheck. O caminho é a linha
    primeiro, depois o teste.
- **Negativas:**
  - renomear um teste pede mudar o 13 no mesmo commit (o doccheck avisa);
  - dois testes com o mesmo nome em módulos diferentes ficam indistinguíveis
    para o catálogo.
- **O que passa a ser obrigatório:** comportamento novo começa por uma linha no
  13, com "O que prova" falsificável e "Atende" apontando o requisito.

## Alternativas rejeitadas

| Alternativa | Por que não |
|---|---|
| Marcador `// T-…` em cada teste | 463 edições, e o vínculo passa a morar em dois lugares |
| Catalogar só os testes que provam requisito | Teste fora do catálogo vira teste que ninguém sabe por que existe |

## Referências

- [ADR-0020](0020-o-doc-e-o-contrato-e-o-doccheck-prova.md).
