# ADR-0022 — Os tipos da ponte são conferidos campo a campo

- **Status:** aceito
- **Data:** 2026-11-07
- **Decisores:** time do Presage

## Contexto

- A casca serializa tipos do core e dela mesma em `camelCase`, e o front os
  recebe por `invoke` e por eventos. `src/lib/types.ts` os espelha à mão, com
  nomes às vezes diferentes (`Scored` ↔ `ScoredChampion`).
- Nada conferia os dois lados: um campo renomeado no Rust chegava ao front como
  `undefined`, sem erro de compilação ([01 §4](../arquitetura/01-core-e-casca.md#4-o-front)).

## Decisão

**Todo tipo que cruza a ponte tem, no [10](../arquitetura/10-modelo-de-dados.md),
um título `Rust ↔ TS`.** Debaixo do título vai o bloco com os campos e os
atributos `serde`. O doccheck exige que:

- os campos do doc sejam os campos do Rust, com o mesmo tipo;
- os nomes serializados pelo Rust, aplicando `rename_all`, `rename` e `skip`,
  sejam os campos da interface TS, contando os de `extends`;
- num enum de variantes sem dado, os nomes serializados sejam os valores da união
  de strings do TS.

## Consequências

- **Positivas:**
  - a divergência aparece no commit, com o campo que falta de cada lado;
  - o mapa de nomes TS ↔ Rust fica escrito num lugar só.
- **Negativas:**
  - o tipo do campo do lado TS não é conferido, só o nome;
  - um enum com dado (`Tupla(u8)`) não tem espelho conferido.
- **O que passa a ser obrigatório:** tipo novo na ponte entra no 10 com `↔`, e o
  espelho no TS, no mesmo commit.

## Alternativas rejeitadas

| Alternativa | Por que não |
|---|---|
| Gerar `types.ts` a partir do Rust (ts-rs, specta) | Uma dependência no build da casca e um passo de geração para lembrar. A conferência resolve o mesmo risco sem mudar o build |
| Validar na entrada do front (zod) | Pega o erro em tempo de execução, na máquina do jogador: tarde demais |

## Referências

- [ADR-0020](0020-o-doc-e-o-contrato-e-o-doccheck-prova.md); [12](../arquitetura/12-contrato-da-casca.md).
