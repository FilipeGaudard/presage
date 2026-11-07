# 00 — Frontend: visão e escopo

O front é a parte do Presage que o jogador vê: a janela principal, com sete
abas, e o overlay por cima do jogo. Ele não calcula o que mostra. Cada número
vem pronto de um comando da casca ([01 §3](../arquitetura/01-core-e-casca.md#3-a-casca)),
e o contrato entre os dois lados está em
[12](../arquitetura/12-contrato-da-casca.md).

> **Status (2026-11-07):**
> - **Construído** em `src/`, cerca de 7.800 linhas: `App.tsx`, `overlay.tsx`,
>   `demo.tsx`, 18 componentes, o espelho de tipos `lib/types.ts` e dois CSS.
> - **Sem teste automatizado.** O único portão é o `tsc --noEmit` do
>   `npm run build`. O catálogo `TF-*` está escrito e todo pendente
>   ([06](06-testes.md), [P-16](../produto/perguntas-em-aberto.md)).
> - **Erro de comando não aparece na tela.** As 22 chamadas de `App.tsx` e as
>   5 de `overlay.tsx` engolem a falha (`.catch(() => void 0)`). Uma pool que
>   não gravou continua na tela como se tivesse gravado ([GC-1](07-lacunas-da-casca.md)).
> - **Uma tela contraria uma regra da Riot.** O rastreador de ouro mostra o ouro
>   em mão estimado dos inimigos, que C-4 e R-12 proíbem ([GC-2](07-lacunas-da-casca.md)).

## 1. Decisões já tomadas

| Tema | Decisão | ADR |
|---|---|---|
| Divisão | O motor é puro e a casca faz o I/O; o front só fala com a casca, por `invoke` e eventos | [ADR-0001](../adr/0001-core-puro-separado-da-casca.md) |
| Atualização da tela | A casca lê o cliente por polling e emite evento quando algo muda; o front não consulta em laço | [ADR-0002](../adr/0002-polling-em-vez-de-websocket.md) |
| Overlay | Janela do próprio app, transparente, atravessável pelo mouse até `Ctrl+Shift+O` | [ADR-0003](../adr/0003-overlay-como-janela-externa.md) |
| Versão nova | Avisa e espera o clique; falha de rede some calada | [ADR-0005](../adr/0005-atualizador-assinado-com-endpoint-permanente.md) |
| Sem dado | Componente sem amostra vale zero, e a tela diz que o número não sai de partidas (`fromData`) | [ADR-0007](../adr/0007-sem-dado-o-componente-vale-zero.md) |
| Perfil de rota | Uma tabela de diferenças com `±`, sem nota única | [ADR-0011](../adr/0011-perfil-de-rota-e-razao-nao-peso.md) |
| Ban vazio | Sem pick deles e sem base, a lista de bans fica vazia e a tela explica por quê | [ADR-0013](../adr/0013-ban-nao-personalizado-enquanto-aditivo.md) |
| Pool | Assistente de três passos: a rota, três escolhidos, cinco sugeridos um por vez | [ADR-0016](../adr/0016-pool-de-oito-tres-escolhidos-cinco-por-cobertura.md) |
| Prévia | A prévia de um candidato é a mesma conta de quando ele travar (`lane_analysis` com `candidate`) | [ADR-0017](../adr/0017-uma-leitura-do-champ-select.md) |
| Condição de vitória | Cada frase leva a fonte ao lado (`regra`, `atributos`, `dados`) | [ADR-0018](../adr/0018-o-que-a-condicao-de-vitoria-pode-afirmar.md) |
| Chance do time | Só aparece com a base de partidas | [ADR-0019](../adr/0019-contra-em-qualquer-rota-e-chance-do-time.md) |
| Espelho de tipos | `lib/types.ts` é escrito à mão e conferido campo a campo pelo doccheck | [ADR-0022](../adr/0022-tipos-da-ponte-conferidos-campo-a-campo.md) |
| Simulador | A aba manda o draft montado para a casca, que o lê no lugar do cliente | [ADR-0023](../adr/0023-o-simulador-passa-por-cima-do-cliente.md) |
| Rota do inimigo | O seletor mostra a presumida; a marca à mão vence a inferência | [ADR-0024](../adr/0024-a-marca-de-rota-vence-a-inferencia.md) |
| Quem a conta tem | Desligado por padrão: "incluir quem você não tem" | [ADR-0025](../adr/0025-so-quem-a-conta-pode-escolher-por-padrao.md) |
| Iniciar com o Windows | Desligado até o jogador marcar, em Ajustes | [ADR-0026](../adr/0026-lcu-so-leitura-e-autostart-desligado.md) |

## 2. As áreas

| Área | O que o jogador faz | Requisitos |
|---|---|---|
| Draft | Vê o ranking da sua rota, os menos expostos ao contra-pick e os bans; corrige a rota presumida de um inimigo; escolhe a própria rota em blind; abre a prévia de um candidato | R-1, R-2, R-3, R-4, R-5, R-6 |
| Simulador | Monta os dois times à mão e vê a mesma recomendação do champ select | R-8, R-6 |
| Rotas | Lê a chance de cada rota, as escaramuças, a condição de vitória e a runa do confronto | R-6, R-7, R-11 |
| Partida | Vê timers, prioridade, a sua rota ao vivo, o ouro dos dez e a build adaptativa | R-10, R-11, R-12, R-13, R-14 |
| Pós-jogo | Lê o que foi bem e o que foi mal na última partida e o perfil de rota | R-16, R-17 |
| Pool | Monta a pool por rota e a lista de bans | R-9, R-3 |
| Ajustes | Escolhe as seções do overlay e liga o início com o Windows | R-15, R-19 |
| Overlay | Olha ouro, dano, objetivos, prioridade, escaramuça e próximo item sem sair do jogo | R-15 |
| Cabeçalho | Vê a versão, o patch, a conexão com o cliente e o aviso de versão nova | R-18, R-19 |

## 3. O que fica fora

| Tema | Por quê | Onde vive |
|---|---|---|
| Mandar runa, item ou feitiço para o cliente; hover, travar, banir | Tirar a decisão do jogador viola C-5; o LCU é só lido | [P-15](../produto/perguntas-em-aberto.md), [ADR-0026](../adr/0026-lcu-so-leitura-e-autostart-desligado.md) |
| Timer de habilidade inimiga | Proibido, inclusive o manual (C-1) | — |
| Onde o inimigo está no mapa | Informação que o jogador não teria (C-4). A prioridade diz valor, não posição (R-14) | `priority::report` |
| Nota, rating ou escada | C-7, RNF-11 | — |
| Anúncio no overlay | C-9 | Painel ou site, se um dia houver |
| Telemetria e métrica de uso | Nada sai da máquina (RNF-8) | — |
| Outros idiomas | Público brasileiro; o Data Dragon é pedido em `pt_BR` (`ddragon.ts:22`) | — |
| macOS e Linux | O app é só Windows | [08 §3](../arquitetura/08-desenvolvimento-e-testes.md#3-o-que-roda-em-cada-máquina) |
| Teste automatizado do front | Ainda sem harness | [06](06-testes.md), [P-16](../produto/perguntas-em-aberto.md) |

## 4. Princípios de interface

Os do produto, traduzidos em regras de tela. Cada um diz onde o código já
cumpre e onde ainda não.

1. **A casca calcula, a tela mostra.** Chance, ouro, dano, score e valor de item
   chegam prontos. O front formata, escala para porcentagem, ordena para exibir
   e escolhe a cor. As exceções de hoje, como o `±` do perfil calculado na tela,
   estão em [03 §9](03-estado-e-atualizacao.md#9-o-que-a-tela-ainda-calcula).
2. **Todo número diz de onde veio** ([confiança do número](../README.md#confiança-do-número)).
   Medição sai sem ressalva; matemática do jogo diz que é índice; peso à mão diz
   "regra, não estatística"; estimativa diz "estimado". O §5 lista cada número e
   o que a tela diz dele hoje.
3. **Sem dado, sem número** (RNF-1, [ADR-0007](../adr/0007-sem-dado-o-componente-vale-zero.md)).
   Sem base, a chance do time não aparece, a rota diz "sem base de partidas: só
   a conta do duelo" e o draft abre a faixa "Sem base estatística".
4. **Alternativas, não ordens** (C-5). O ranking mostra a pool e os melhores no
   geral lado a lado, com motivo em texto; o overlay mostra dois itens, não um;
   a condição de vitória descreve, não manda.
5. **Nada que o jogador não veria** (C-4). Prioridade é valor, nunca posição; o
   texto da tela diz isso. Hoje uma tela fura a regra: o ouro em mão dos
   inimigos ([GC-2](07-lacunas-da-casca.md)).
6. **Aponta, não dá nota** (C-7, RNF-11). O pós-jogo lista observações; o perfil
   é uma tabela de diferenças por métrica, sem total.
7. **A cor da marca nunca carrega dado** (RNF-12, [identidade](../produto/identidade.md#paleta)).
   Número bom, ruim e neutro usam `good`, `bad`, `even`; o score do draft usa
   `delta-strong`, `delta-ok` e `delta-weak`, com menta, âmbar e coral. O violeta `--halo` marca identidade, seleção e a sua
   vez. Uma exceção hoje: a barra de prioridade é pintada de `--halo`
   (`styles.css:1073`).
8. **O jogo primeiro.** Nada instala sozinho, o overlay nasce atravessável, e
   nada pisca vazio por cima da partida: sem snapshot, o overlay não desenha
   nada (`overlay.tsx:94`).
9. **Falha calada só por escolha.** A checagem de versão falha calada de
   propósito (RNF-7). O resto deveria dizer que falhou; hoje não diz
   ([GC-1](07-lacunas-da-casca.md)).

## 5. Cada número e a etiqueta dele

A etiqueta é a da [visão geral §2](../produto/visao-geral.md#2-de-onde-vem-cada-número).
A última coluna é o que a tela escreve hoje perto do número.

| Número | Onde | Etiqueta | O que a tela diz |
|---|---|---|---|
| Ouro em itens, do time e de cada um | Partida, overlay, pós-jogo | exata | "A coluna em negrito é exata" (`GoldTracker.tsx:93`); "O ouro em itens é soma exata dos preços" (`GraficoPartida.tsx:175`). O overlay não diz nada |
| Ouro em mão estimado (`+N`) | Partida | estimativa | `title` "Estimativa do ouro em mão". Não deveria existir para os outros nove (GC-2) |
| Timer de objetivo | Partida, overlay | exata quando observado; estimada no primeiro spawn | "estimado" (`LiveOverview.tsx:54`), "est" no overlay |
| Tropas e visão por minuto contra o oponente; à frente, igual ou atrás | Partida | exata; o corte é regra | "Ouro em itens é exato", e o corte de 300 ou 10% é "regra, não estatística" (`LiveOverview.tsx:175`) |
| Valor por 1000 de ouro | Partida | alta na ordem, índice no valor | "poder por 1000g". Não diz que é índice |
| Divisão do dano (`% físico`) | Partida, overlay | alta depois dos primeiros itens; fraca antes | nada |
| Vida efetiva | Partida, overlay | alta | nada |
| Chance por rota e de escaramuça | Rotas, draft, overlay | fraca como probabilidade | "sem base de partidas: só a conta do duelo" quando `fromData` é falso. Não diz "fraca" |
| Curva por nível e vantagem de nível | Rotas, prévia | não vê habilidades | "atributos, sem a suprema" e a nota sobre a suprema no 6 (`LaneAnalysis.tsx:179`) |
| Condição de vitória | Rotas, prévia | regra | a etiqueta `regra`, `atributos` ou `dados` em cada frase |
| Runa do confronto | Rotas | regra | "Isto sai de regras sobre a mecânica do confronto … Não sai de winrate" |
| Sugestão dos lugares 4 a 8 da pool | Pool | regra | "regra, não estatística" (`PoolBuilder.tsx:347`) |
| Score e delta do draft | Draft, simulador | fraca | sem base, a faixa "Sem base estatística"; com base, "Base do patch …" |
| Exposição ao contra-pick | Draft, simulador | fraca | `−x.x` sem unidade na grade; "custa x pp" no detalhe |
| Prioridade de rota (0 a 100) | Partida, overlay | não está na visão geral §2 | "Ranking de valor, não de posição" |
| Perfil de rota | Pós-jogo | medição, com `±` | "Só ganha cor o que sai do ruído (intervalo de 95%)" |
| Chance do time | Prévia | só com a base | "Chance do time com ele, pela base de partidas" |
| Relógio do draft | Draft | exata, interpolada na tela | nada |

O alvo da [visão geral §7](../produto/visao-geral.md#7-como-medimos) é nenhum
número sem etiqueta. Hoje faltam cinco: valor por 1000 de ouro, divisão do
dano antes dos itens, vida efetiva, exposição sem unidade e chance por rota
sem a palavra "fraca".

## 6. Mapa dos documentos

| Documento | Conteúdo |
|---|---|
| [01-arquitetura](01-arquitetura.md) | As três páginas, quem chama a casca, o espelho de tipos, Data Dragon, build e versão |
| [02-telas-e-fluxos](02-telas-e-fluxos.md) | Cada aba: o que mostra, de qual comando, as interações e os estados, com o texto exato |
| [03-estado-e-atualizacao](03-estado-e-atualizacao.md) | Onde o estado mora, o que refaz cada comando, eventos, simulador, erros, o que nunca acontece |
| [04-overlay](04-overlay.md) | A janela por cima do jogo: seções, atalhos, preferências, regras da Riot |
| [05-seguranca](05-seguranca.md) | Texto que não controlamos, permissões da webview, CSP, checklist de PR |
| [06-testes](06-testes.md) | O plano de teste do front e o catálogo `TF-*` |
| [07-lacunas-da-casca](07-lacunas-da-casca.md) | O que a casca precisa ganhar para as telas dizerem a verdade (`GC-*`) |
