# Identidade

## A marca

A marca é um **caminho que se ramifica**, com a ponta escolhida preenchida e
acima das alternativas, que ficam esmaecidas. É o minimax do draft desenhado:
prever qual galho sobrevive à resposta do adversário. Lê como sigilo a 16px e
como árvore de decisão a 128px.

Vive em `src/components/Logo.tsx` (usa `currentColor`, então herda a cor do
contexto) e em `public/favicon.svg`. Os ícones do executável saem da mesma
geometria (`scripts/icone.ps1`).

## Paleta

Uma regra organiza tudo: **a cor da marca nunca carrega significado de dado**
(RNF-12).

| Token | Valor | Papel |
|---|---|---|
| `--ink` | `#06080f` | fundo |
| `--veil` | `#0c1020` | painéis |
| `--rise` | `#141a30` | superfícies elevadas |
| `--line` | `#222b47` | divisórias |
| `--bone` | `#e6e9f2` | texto |
| `--dim` | `#7c86a6` | texto secundário |
| `--halo` | `#7b6cff` | **marca**: seleção, foco, identidade |
| `--mint` | `#3ddc97` | número bom |
| `--coral` | `#ff5c7a` | número ruim |
| `--ember` | `#ffb454` | atenção, urgência |

Não há dourado aqui, e isso é deliberado: o dourado é a cor do próprio League, e
um app que copia a paleta do jogo não tem cara de nada. O violeta também sai da
rota de Blitz e Porofessor, que são azuis.

As classes `good`, `bad` e `even` em `src/styles.css` são o único caminho de cor
para número. O perfil de rota usa a mesma regra com uma condição a mais: só
ganha cor o que sai do intervalo de 95%; o resto fica em `even`.
