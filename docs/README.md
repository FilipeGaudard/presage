# Documentação — Presage

Assistente de draft, partida e pós-jogo para League of Legends. App de desktop
(Tauri + Rust + React), só Windows.

> **Status (2026-11-07):**
> - **0.7.0 é a última versão publicada.** Depois dela vieram, sem versão nova:
>   o draft pela sua rota, a pool por cobertura, a prévia do candidato, o ranking
>   em log-odds contra o time inteiro e as lacunas G-1 a G-8 frente ao iTero
>   ([roteiro](produto/visao-geral.md#4-roteiro)).
> - **Os docs são contrato** ([ADR-0020](adr/0020-o-doc-e-o-contrato-e-o-doccheck-prova.md)).
>   `npm run doccheck -- -strict` prova, a cada push:
>   - que todo tipo, função, constante, comando, evento, arquivo, opção do coletor
>     e teste citado aqui existe no código;
>   - que todo item público do código está aqui;
>   - que o espelho TS dos tipos da ponte bate com o Rust campo a campo.
>
>   O resultado fica em `build/ledger.md`.
> - **Camada estatística vazia:** winrate base, matchup, contra e sinergia
>   esperam a base de partidas, que espera uma chave da Riot que dure
>   ([P-1](produto/perguntas-em-aberto.md), [P-6](produto/perguntas-em-aberto.md)).
> - **Defeitos conhecidos** no que já está construído ficam em `build/audit.md`,
>   abertos até o teste que falha antes da correção. As linhas graves (A1, A3 a
>   A13) são a próxima etapa; a A3, ouro em mão dos inimigos na tela contra o
>   C-4, já está fechada.

## Por onde começar

| Ordem | Documento | Para quê |
|---|---|---|
| 1 | [produto/visao-geral.md](produto/visao-geral.md) | O que é, o diferencial, de onde vem cada número, quem usa, o roteiro |
| 2 | [produto/glossario.md](produto/glossario.md) | Vocabulário PT↔código. **Leia antes de qualquer doc de arquitetura** |
| 3 | [produto/requisitos.md](produto/requisitos.md) | As regras da Riot (C-\*), os requisitos não funcionais (RNF-\*), os funcionais (R-\*) e a matriz que os liga a funções e testes |
| 3a | [produto/regras-e-pesos.md](produto/regras-e-pesos.md) | Cada regra e peso escrito à mão que decide um resultado, a confiança dele e o que o substitui quando a base existir |
| 4 | [arquitetura/00-contexto-e-limites.md](arquitetura/00-contexto-e-limites.md) | As fontes de dado e o que cada uma pode e não pode dar |
| 5 | [arquitetura/01-core-e-casca.md](arquitetura/01-core-e-casca.md) | A divisão do workspace e as fronteiras entre JSON cru e domínio |
| 5a | [arquitetura/14-uma-partida-passo-a-passo.md](arquitetura/14-uma-partida-passo-a-passo.md) | ★ Uma partida inteira, do cliente aberto ao perfil de rota: o que acontece em cada passo, por quê, onde, e o que pode dar errado |
| 6 | [arquitetura/10-modelo-de-dados.md](arquitetura/10-modelo-de-dados.md) | ★ Todos os tipos, campos e unidades |
| 7 | [arquitetura/11-catalogo-de-funcoes.md](arquitetura/11-catalogo-de-funcoes.md) | ★ Cada função: assinatura, o que faz, erros, confiança, requisito |
| 8 | [arquitetura/12-contrato-da-casca.md](arquitetura/12-contrato-da-casca.md) | ★ Os comandos, eventos e arquivos entre a casca e o front |
| 9 | demais docs de `arquitetura/` | Aprofundamento por tema: draft (02), partida (03), perfil (04), coletor (05), distribuição (06), segurança (07), desenvolvimento (08) |
| 10 | [frontend/00-visao-e-escopo.md](frontend/00-visao-e-escopo.md) | A interface: telas, estado, overlay, segurança, testes e as lacunas da casca |
| 11 | [adr/](adr/) | Decisões, com contexto e alternativas rejeitadas |
| — | [arquitetura/13-testes.md](arquitetura/13-testes.md) | O catálogo de todo teste, por T-ID |
| — | [produto/perguntas-em-aberto.md](produto/perguntas-em-aberto.md) | O que depende da Riot, de dado ou do dono do produto, com o default adotado enquanto isso |
| — | [arquitetura/09-licoes-do-prototipo.md](arquitetura/09-licoes-do-prototipo.md) | Cada defeito encontrado → o que o criou → a prevenção |
| — | [referencia/itero/README.md](referencia/itero/README.md) | O iTero por dentro, e a decisão do Presage para cada funcionalidade dele |

## Mapa completo

```
docs/
  README.md                              este índice
  produto/
    visao-geral.md                       o que é, de onde vem cada número, quem usa, roteiro
    glossario.md                         vocabulário PT↔código
    requisitos.md                        C-*, RNF-*, R-* e a matriz de rastreabilidade
    regras-e-pesos.md                    RN-*: cada peso escrito à mão
    perguntas-em-aberto.md               P-*, cada uma com default e dono
    identidade.md                        marca e paleta
  arquitetura/
    00-contexto-e-limites.md             LCU, Live Client, Riot Web API, Data Dragon
    01-core-e-casca.md                   workspace, fronteiras
    02-motor-de-draft.md                 rank(), shrinkage, minimax, ban, pool
    03-motor-da-partida.md               build, dano, rotas, ouro, timers, prioridade, pós-jogo, overlay
    04-perfil-de-rota.md                 você contra o oponente direto
    05-coletor.md                        tools/ingest e a base de partidas
    06-distribuicao-e-atualizador.md     instalador, atualizador, versão
    07-seguranca.md                      ativos, superfície, entradas não confiáveis
    08-desenvolvimento-e-testes.md       rodar, toolchain, como os testes são escritos, demo
    09-licoes-do-prototipo.md            defeito → causa → prevenção
    10-modelo-de-dados.md                ★ tipos, campos, unidades, constantes (conferido)
    11-catalogo-de-funcoes.md            ★ assinaturas, erros, confiança, requisito (conferido)
    12-contrato-da-casca.md              ★ comandos, eventos, arquivos, fontes (conferido)
    13-testes.md                         ★ catálogo T-* de todo teste (conferido)
    14-uma-partida-passo-a-passo.md      ★ a história inteira, passo a passo, e por quê
  frontend/
    00-visao-e-escopo.md … 07-lacunas-da-casca.md
  adr/
    0000-template.md … 0026-lcu-so-leitura-e-autostart-desligado.md
  referencia/
    itero/                               o iTero por dentro, cruzado com o Presage (I-*, G-*)

build/                                   recibos, fora de docs/
  ledger.md                              gerado pelo doccheck: o que está feito, adiado e pendente
  audit.md                               A-*, B-*, D-*, P-*: defeitos e requisitos sem prova
  gaps.md                                recibos das lacunas G-1 a G-8
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
| `R-n` | Requisito funcional | [requisitos](produto/requisitos.md#3-requisitos-funcionais-r) |
| `RN-n` | Regra ou peso escrito à mão | [regras e pesos](produto/regras-e-pesos.md) |
| `T-ÁREA-n` | Linha do catálogo de testes: nomeia as funções de teste | [13](arquitetura/13-testes.md) |
| `TF-ÁREA-n` | Teste do front (plano; sem harness ainda, P-16) | [frontend 06](frontend/06-testes.md) |
| `GC-n` | Lacuna que as telas expõem no contrato da casca | [frontend 07](frontend/07-lacunas-da-casca.md) |
| `I-n` | Funcionalidade do iTero | [referência](referencia/itero/05-funcionalidades.md) |
| `G-n` | Lacuna do Presage frente ao iTero | [cruzamento](referencia/itero/07-cruzamento.md#6-as-lacunas-que-cabem-por-valor) |
| `A`n, `B`n, `D`n, `P`n | Linha da auditoria: defeito no que está construído (`A`), requisito sem prova (`B`), doc que o código desmentia (`D`), item prático (`P`). `P1` ali não é `P-1` | `build/audit.md` |

Testes são citados pelo nome da função (`popularidade_nao_vira_matchup`): o nome
é a regra, e `cargo test <nome>` roda só ele (salvo dois nomes repetidos entre módulos, P1 do `build/audit.md`). O T-ID do catálogo agrupa os nomes
e é o que a matriz cita ([ADR-0021](adr/0021-o-id-do-teste-nomeia-a-funcao.md)).

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

Quando a cópia é inevitável (a assinatura no 11, o campo no 10, o valor de uma
constante), ela vai num bloco `rust`, e o `doccheck` a compara com o código. O
comportamento novo começa pela linha do teste no 13; o item público novo entra no
10, 11 ou 12 no mesmo commit. Sem isso, `npm run doccheck -- -strict` falha.
Adiar um item pede motivo em `tools/doccheck/deferred.txt`.

### Dados pessoais

Nenhum documento traz Riot ID, puuid ou histórico de jogador real. Os exemplos
usam números de tamanho realista, não partidas de alguém.
