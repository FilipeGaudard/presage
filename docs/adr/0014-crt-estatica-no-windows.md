# ADR-0014 — CRT estática no Windows

- **Status:** aceito
- **Data:** anterior à 0.7.0 (registrada em 2026-10-28)
- **Decisores:** time do Presage

## Contexto

- O MSVC portátil montado sem administrador traz só as libs estáticas
  (`libcmt.lib`), não as de importação dinâmica (`msvcrt.lib`): sem ajuste, o link
  falha com `LNK1104`.
- "O app não abre" por falta do Visual C++ Redistributable é um dos motivos
  clássicos de suporte em desktop Windows.

## Decisão

**Linkar a CRT estática no alvo `x86_64-pc-windows-msvc`** (`.cargo/config.toml`,
`-C target-feature=+crt-static`).

## Consequências

- **Positivas:** o MSVC portátil linka; o executável não depende do
  Redistributable na máquina de quem baixa (RNF-10).
- **Negativas:** executável um pouco maior.
- **Obrigatório:** nada de dependência que exija a CRT dinâmica.

## Alternativas rejeitadas

| Alternativa | Por que não |
|---|---|
| CRT dinâmica + Redistributable no instalador | Mais um instalador dentro do instalador, e não linka com o MSVC portátil |

## Referências

- [06 §1](../arquitetura/06-distribuicao-e-atualizador.md#1-gerando-o-instalador).
