# Documentação — Presage

Assistente de draft, partida e pós-jogo para League of Legends. App de desktop
(Tauri + Rust + React), só Windows.

> **Status (2026-10-28):**
> - **0.7.0 no ar:** ranking de draft com resposta adversária, build adaptativa,
>   timers, prioridade, overlay, leitura pós-partida e construtor de pool.
> - **Camada estatística vazia:** winrate base, matchup e sinergia esperam a base
>   de partidas, que espera uma chave da Riot que dure ([P-1](produto/perguntas-em-aberto.md), [P-6](produto/perguntas-em-aberto.md)).
> - **Perfil de rota escrito e compilando no Windows** (CI, `cargo test
>   --workspace`). Os nomes de campo da Riot ainda não foram conferidos contra
>   uma partida real ([P-5](produto/perguntas-em-aberto.md)).

## Por onde começar

| Ordem | Documento | Para quê |
|---|---|---|
| 1 | [produto/visao-geral.md](produto/visao-geral.md) | O que é, o diferencial, de onde vem cada número, o roteiro |
| 2 | [produto/glossario.md](produto/glossario.md) | Vocabulário PT↔código. **Leia antes de qualquer doc de arquitetura** |
| 3 | [produto/requisitos.md](produto/requisitos.md) | As regras da Riot (C-\*) e os requisitos não funcionais (RNF-\*): o que o produto não pode ser |
| 4 | [arquitetura/00-contexto-e-limites.md](arquitetura/00-contexto-e-limites.md) | As fontes de dado e o que cada uma pode e não pode dar |
| 5 | [arquitetura/01-core-e-casca.md](arquitetura/01-core-e-casca.md) | A divisão do workspace e as fronteiras entre JSON cru e domínio |
| 6 | demais docs de `arquitetura/` | Aprofundamento por tema |
| 7 | [adr/](adr/) | Decisões, com contexto e alternativas rejeitadas |
| — | [produto/perguntas-em-aberto.md](produto/perguntas-em-aberto.md) | O que depende da Riot ou de dado, com o default adotado enquanto isso |
| — | [arquitetura/09-licoes-do-prototipo.md](arquitetura/09-licoes-do-prototipo.md) | Cada defeito encontrado → o que o criou → a prevenção |

## Mapa completo

```
docs/
  README.md                              este índice
  produto/
    visao-geral.md                       o que é, de onde vem cada número, roteiro
    glossario.md                         vocabulário PT↔código
    requisitos.md                        C-* (Riot) e RNF-*
    perguntas-em-aberto.md               P-*, cada uma com default
    identidade.md                        marca e paleta
  arquitetura/
    00-contexto-e-limites.md             LCU, Live Client, Riot Web API, Data Dragon
    01-core-e-casca.md                   workspace, fronteiras, comandos e eventos
    02-motor-de-draft.md                 rank(), shrinkage, minimax, ban, pool
    03-motor-da-partida.md               build, dano, rotas, ouro, timers, prioridade, pós-jogo, overlay
    04-perfil-de-rota.md                 você contra o oponente direto
    05-coletor.md                        tools/ingest e a base de partidas
    06-distribuicao-e-atualizador.md     instalador, atualizador, versão
    07-seguranca.md                      ativos, superfície, entradas não confiáveis
    08-desenvolvimento-e-testes.md       rodar, toolchain, testes, demo
    09-licoes-do-prototipo.md            defeito → causa → prevenção
  adr/
    0000-template.md … 0018-o-que-a-condicao-de-vitoria-pode-afirmar.md
```

## Convenções

### Idioma

- A prosa é em **pt-BR**.
- Identificadores de código ficam **como estão no código**, sempre em
  `monoespaçado`. O código mistura: tipos e funções em inglês, testes e parte do
  coletor em português (`Opcoes`, `--retomar`). O [glossário](produto/glossario.md)
  faz a ponte.

### Identificadores de rastreabilidade

| Prefixo | Significado | Definido em |
|---|---|---|
| `C-n` | Regra da Riot que não se negocia | [requisitos](produto/requisitos.md#1-regras-da-riot-c) |
| `RNF-n` | Requisito não funcional | [requisitos](produto/requisitos.md#2-requisitos-não-funcionais-rnf) |
| `ADR-00NN` | Decisão de arquitetura | [adr/](adr/) |
| `P-n` | Pergunta em aberto | [perguntas](produto/perguntas-em-aberto.md) |
| `L-n` | Lição do protótipo | [09](arquitetura/09-licoes-do-prototipo.md) |

Testes são citados pelo nome da função (`popularidade_nao_vira_matchup`): o nome
é o identificador, e `cargo test <nome>` roda só ele.

### Confiança do número

Todo número que aparece na tela tem uma de quatro etiquetas
([visão geral §2](produto/visao-geral.md#2-de-onde-vem-cada-número)):

| Etiqueta | Significa |
|---|---|
| **exata** | medição direta da partida |
| **alta** | matemática do próprio jogo: a ordem é confiável, o valor é índice |
| **fraca** | peso escolhido à mão ou prior escrito à mão |
| **regra** | regra mecânica escrita à mão, não estatística |

### Pureza

| Marca | Onde | Pode |
|---|---|---|
| `core` | `presage-core` | nada de Tauri, rede, disco ou relógio. Testável em qualquer máquina |
| `casca` | `src-tauri` | I/O: LCU, Live Client, Riot Web API, Data Dragon, disco |
| `coletor` | `tools/ingest` | binário à parte, fora do instalador |

### Quando o código muda

Doc e código andam no mesmo commit, e um fato vive num lugar só: o README da
raiz aponta para cá em vez de repetir. [L-7](arquitetura/09-licoes-do-prototipo.md)
é o que acontece com a segunda cópia.

### Dados pessoais

Nenhum documento traz Riot ID, puuid ou histórico de jogador real. Os exemplos
usam números de tamanho realista, não partidas de alguém.
