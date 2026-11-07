# 03 — Estado e atualização

O estado do produto mora na casca (`watcher::Shared`). O front guarda uma
cópia do que recebeu e o estado de interface. Não há cache, biblioteca de
estado nem nova tentativa: cada `useEffect` de `App.tsx` chama comandos quando o
que ele lê muda, e cada evento substitui um pedaço da cópia.

## 1. Onde o estado mora

| Estado | No front | Na casca | Quem escreve |
|---|---|---|---|
| Aba aberta | `view` (`App.tsx:53`) | — | o jogador; `presage://review` |
| Conexão | `status` | `Shared::status` | o watcher |
| Draft do cliente | `draft` e a âncora do relógio | `Shared::draft` | o watcher |
| Draft do simulador | `simDraft` (na tela) e `simAplicado` (o que a casca já guardou) | `Shared::simulated` | o jogador, por `set_simulated_draft` |
| Rotas marcadas nos inimigos | `enemyRoles` | `Shared::enemy_roles` | o jogador, por `set_enemy_role`; a casca limpa no fim do champ select |
| Sua rota escolhida | `draftRole` | `Shared::draft_role` | o jogador, por `set_draft_role` |
| Pool e bans | `pool` | `Shared::pool` e `pool.json` | o jogador, por `set_pool` |
| Preferências do draft | `draftPrefs` | `Shared::draft_prefs` e `draft.json` | o jogador, por `set_draft_prefs` |
| Preferências do overlay | `overlayPrefs` (painel) e `prefs` (overlay) | `Shared::overlay_prefs` e `overlay.json` | o jogador, por `set_overlay_prefs` |
| Iniciar com o Windows | `autostart` | o registro do Windows | o jogador, por `set_autostart` |
| Motor | `engine` | campeões, itens e estatísticas em `Shared` | a casca |
| Partida | `live` | `Shared::live` | o watcher |
| Ranking, resposta, bans | `recommendations`, `minimax`, `bans` | — (calculados a cada pedido) | — |
| Build, ouro | `buildContext`, `suggestions`, `goldReport` | — | — |
| Rotas e runas | `lanes`, `runas` | — | — |
| Leitura pós-partida | `review` | `Shared::last_review` e `ultima-partida.json` | o watcher, no fim da partida |
| Perfil de rota | `laneProfile` | as estatísticas pessoais | a casca, depois do histórico |
| Nomes e ícones | `staticData` | (a casca tem a própria cópia) | `lib/ddragon.ts` |
| Candidato aberto, prévia | `selected`, `previa` em `RecommendationPanel` | — | o jogador |
| Passo, rota e busca da pool | `PoolBuilder` | — | o jogador |

Três estados existem dos dois lados sem forma de reler o da casca: as marcas
(`get_enemy_roles` existe e ninguém chama), a sua rota (não há comando de
leitura) e o draft do simulador. Recarregar a webview zera o lado do front e
mantém o da casca ([GC-4](07-lacunas-da-casca.md)).

## 2. A carga inicial

Um efeito só, na montagem (`App.tsx:91-146`), sem ordem entre as chamadas:

| Comando | Preenche |
|---|---|
| `get_snapshot` | `status` e `draft`, e ancora o relógio |
| `engine_status` | `engine` |
| `get_pool` | `pool` |
| `get_live` | `live` |
| `last_review` | `review` |
| `lane_profile` | `laneProfile` |
| `get_overlay_prefs` | `overlayPrefs` |
| `get_draft_prefs` | `draftPrefs` |
| `get_autostart` | `autostart` |

Em paralelo, `loadStaticData()` busca o Data Dragon (`App.tsx:85-89`). O mesmo
efeito liga os cinco ouvintes do §3 e o relógio do §8.

## 3. Eventos

| Evento | Janela | O que o front faz |
|---|---|---|
| `presage://status` | principal | troca `status` |
| `presage://draft` | principal | troca `draft`, reancora o relógio; com `null`, zera `enemyRoles` e o relógio |
| `presage://engine` | principal | troca `engine` e pede `lane_profile` de novo |
| `presage://live` | principal e overlay | troca `live` |
| `presage://review` | principal | troca `review` e muda para a aba Pós-jogo |
| `presage://prefs` | overlay | troca `prefs` |

A janela principal não ouve `presage://prefs`: ela é quem grava, e já tem o
valor. A casca só emite quando algo mudou; o draft ignora o relógio na
comparação (`watcher.rs:261-269`), e por isso o front interpola o tempo.

## 4. O que refaz cada comando

| Efeito | Roda quando muda | Chama | Com `null` ou sem motor |
|---|---|---|---|
| Ranking (`App.tsx:167-192`) | `activeDraft`, `engine.ready`, `pool`, `draftRole`, `enemyRoles`, `draftPrefs` | `rank_draft`, `minimax_draft`, `ban_suggestions`, juntos | zera as três listas |
| Partida (`App.tsx:194-220`) | `live` | `build_suggestions` com `{ limit: 8 }`, `gold_report` | zera build e ouro |
| Rotas (`App.tsx:224-243`) | `activeDraft`, `live`, `engine.ready`, `enemyRoles` | `lane_analysis` com `{ candidate: null }`, `rune_plan` | zera `lanes` (as runas ficam) |
| Simulador (`App.tsx:150-163`) | `view`, `simDraft` | `set_simulated_draft` | fora da aba, manda `null` |
| Prévia (`RecommendationPanel.tsx:61-75`) | candidato, `draft`, `onPreview` | `lane_analysis` com `{ candidate }` | zera a prévia |
| Pool, passo 3 (`PoolBuilder.tsx:38-48`) | passo, rota, `pool` | `pool_suggestions` | — |
| Overlay (`overlay.tsx:61-90`) | `snapshot` | `build_suggestions` com `{ limit: 2 }`, `gold_report`, `skirmish_focus` | zera tudo |

`activeDraft` é `simAplicado` na aba Simulador e `draft` nas outras
(`App.tsx:165`).

Dependências que faltam, e o efeito na tela:

- **Rotas não dependem de `draftRole` nem de `pool`.** A casca resolve a sua
  rota pela escolha e pela pool (`context::resolve_role`). Escolher "MID" em
  blind refaz o ranking, mas a análise de rota, a runa e as rotas presumidas no
  `DraftBoard` continuam as da rota anterior até o draft mudar.
- **A prévia não depende de `enemyRoles`.** O comentário de `App.tsx:271` diz
  que corrigir a rota de um inimigo refaz a prévia; não refaz, porque o efeito
  da prévia só olha o candidato e o `draft`, e marcar não muda o `draft`.
- **As runas não zeram.** Com `activeDraft` e `live` nulos, `lanes` vira `null` e
  `runas` fica com o plano anterior. Na aba Rotas, depois da partida, aparece
  "Sem rotas para comparar" e, embaixo, a runa do confronto que já acabou.

Cada `.then` confere um `cancelled` ou `vivo` antes de escrever: uma resposta
atrasada de um draft antigo não sobrescreve a do novo.

## 5. O simulador

O simulador passa por cima do cliente
([ADR-0023](../adr/0023-o-simulador-passa-por-cima-do-cliente.md)):

1. Abrir a aba manda o `simDraft` atual para a casca (`set_simulated_draft`).
2. Só quando a casca responde, `simAplicado` recebe o mesmo draft, e o efeito do
   ranking roda. Sem isso, o ranking poderia ser pedido antes de a casca trocar o
   draft e mostraria o do cliente.
3. Cada edição repete os passos 1 e 2.
4. Sair da aba manda `{ draft: null }` e zera `simAplicado`. A casca volta a ler o
   champ select.

Enquanto a aba está aberta, **todo** comando de draft lê o simulado, inclusive o
`lane_analysis` sem candidato. A aba Draft não aparece ao mesmo tempo, então a
tela nunca mistura os dois.

## 6. Escritas: otimista ou confirmada

| Comando | A tela muda | A casca grava em disco antes? | Falha |
|---|---|---|---|
| `set_pool` | antes da resposta (`App.tsx:312-315`) | sim (`lib.rs:420-424`): falhou o disco, a memória fica com a pool velha | a tela mostra a pool nova; o ranking usa a velha; o disco tem a velha |
| `set_overlay_prefs` | antes (`App.tsx:317-322`) | sim | o painel mostra a escolha nova; o overlay e o disco ficam na velha |
| `set_draft_prefs` | antes (`App.tsx:295-298`) | sim | a caixa marcada; o ranking sem a mudança |
| `set_autostart` | depois (`App.tsx:300-304`) | — (registro do Windows) | a caixa não muda, sem explicação |
| `set_draft_role` | depois | não grava | a aba de rota não acende |
| `set_enemy_role` | depois | não grava | o seletor volta para "(presumida)" |
| `set_simulated_draft` | depois (`simAplicado`) | não grava | o ranking fica no draft anterior |

Nos três otimistas, a tela muda na hora. Se a casca recusa, a tela volta ao
valor de antes, que é o que a casca e o disco ainda têm, e a faixa de falha diz o
que não gravou (§7). Ela só volta se ainda mostra o valor daquela escrita: uma
escrita mais nova, que deu certo, não é desfeita. Nos confirmados, a tela só muda
com o `Ok`, e a falha também vira faixa.

## 7. Erros hoje

Toda chamada de `App.tsx` à casca passa por `chamar(rótulo, comando, args)`
(A1, [GC-1](07-lacunas-da-casca.md)):

- **Falhou:** a falha entra na lista de faixas, uma por rótulo ("Não consegui
  gravar a pool: {o texto da casca}"), e a chamada devolve `{ ok: false }`.
- **Deu certo:** a faixa daquele rótulo some.
- Quem chamou decide o que mostrar no lugar, e a regra é uma só: **dado que não
  veio não fica na tela como se fosse o de agora.**

| Onde | Falhou | A tela |
|---|---|---|
| Carga inicial (9 leituras) e `lane_profile` no evento | a leitura | fica o valor inicial, com a faixa |
| Ranking, resposta, bans | cada um por si | a lista que falhou fica vazia; as outras duas valem |
| Build e ouro | cada um por si | build vazia; ouro sem relatório |
| Rotas e runas | cada um por si | sem análise; sem runa |
| Prévia (`lane_analysis` com candidato) | a promessa rejeita | `RecommendationPanel` mostra o estado sem prévia, e a faixa diz por quê |
| Sugestão da pool | a promessa rejeita | "Não consegui calcular a sugestão agora. O aviso no topo diz por quê.", nunca "Nada a recomendar" |
| Escritas otimistas | a casca recusa | a tela volta ao valor de antes (§6) |
| Escritas confirmadas | a casca recusa | a tela não muda |
| `overlay.tsx` | cada seção por si (`Promise.allSettled`) | a seção que falhou some; o overlay não mostra faixa por cima do jogo, porque a janela principal pede os mesmos comandos e mostra a dela |
| `Atualizacao.tsx` | a checagem | o aviso some: falha calada por decisão (RNF-7) |
| `App.tsx:88` | o Data Dragon | a faixa com o texto do erro |

A única chamada que ainda ignora a falha é desligar um ouvinte que nunca ligou,
na limpeza dos efeitos: não há o que avisar.

O que a faixa não cobre:

- **Erro de fundo:** Data Dragon da casca, histórico pessoal, registro de atalho.
  Ele não volta por comando nenhum, então só vai para o log, e a versão de
  lançamento não tem console (A14).
- **A forma do erro:** ele chega como texto livre (`Result<(), String>`, 12 §1),
  sem código para a tela escolher o que dizer.
- **Motor que não carrega fica "carregando".** `engine.ready` é "há campeões"
  (`lib.rs:67`). Se a casca não conseguir os campeões nem pelo cache, a tela diz
  "Carregando dados de campeões…" para sempre ([GC-7](07-lacunas-da-casca.md)).

Provas: TF-POOL-4 (escrita recusada volta e avisa) e TF-CASCA-4 (ranking que
falha avisa e sai da tela).

## 8. O relógio

A casca manda `timeLeftMs` só quando o draft muda. `App.tsx` guarda
`{ at: Date.now(), leftMs }` a cada evento (`App.tsx:100`, `:133-135`), e um
`setInterval` de 100 ms (`App.tsx:139`) força o cálculo
`max(0, leftMs − (agora − at)) / 1000` (`App.tsx:306-310`). O `DraftBoard`
arredonda para cima. O intervalo nunca para e re-renderiza a janela inteira, em
qualquer aba ([01 §8](01-arquitetura.md#8-desempenho)).

Quando a casca perde o cliente no meio do champ select, ela esquece o draft sem
emitir `presage://draft` (`watcher.rs:218-225`). A tela continua com o draft e o
relógio desce até zero ([GC-3](07-lacunas-da-casca.md)).

## 9. O que a tela ainda calcula

Nenhuma chance, ouro, dano, score ou valor de item nasce no front. Mas a tela
faz contas além de formatar:

| Conta | Onde | Por que importa |
|---|---|---|
| `±1,96·sd` e "clara" (`abs(value) > 1,96·sd`) do perfil de rota | `PerfilDeRota.tsx:19`, `:55` | repete `Reading::is_clear` (`profile.rs:52-53`); o `±` mostrado nasce na tela ([GC-9](07-lacunas-da-casca.md)) |
| O destaque de cada confronto, pelo maior `abs(value) / sd` | `PerfilDeRota.tsx:62-63` | regra de exibição na tela |
| Confronto só com 2 partidas ou mais, até 4 | `PerfilDeRota.tsx:85` | corte escolhido na tela |
| `Eles = 100 − Vocês`, `mágico = 100 − físico` | `LaneAnalysis.tsx:122`, `LiveBuild.tsx:42` | complemento; certo enquanto a casca mandar uma fração do seu lado |
| Largura da barra de ouro: `nosso / (nosso + deles)` | `GoldTracker.tsx:40` | razão de dois números exatos |
| Ouro em mão só a partir de 800 | `GoldTracker.tsx:70` | o core avisa a partir de 1.500 (`gold.rs:39`); dois cortes |
| A frase do gráfico: pico ≥ 1.000 e queda > 1.500 | `GraficoPartida.tsx:133` | regra de texto na tela |
| Menos expostos: ordena `minimax` por `exposure` e corta em 5 | `RecommendationPanel.tsx:102` | ordem de exibição |
| Melhores no geral: os 15 primeiros fora da pool; bans: os 10 primeiros | `RecommendationPanel.tsx:99`, `:191` | corte de exibição |
| Limiares de cor (55/45, 53/47, 52/48, delta 2/1) | [01 §7](01-arquitetura.md#7-cor-e-estilo) | três limiares diferentes para a mesma pergunta |
| O relógio do draft | §8 | interpolação |
| Tudo desligado: `hidden.length >= 6` | `overlay.tsx:94` | contagem escrita à mão ([GC-11](07-lacunas-da-casca.md)) |

## 10. O que nunca acontece

Cada item foi conferido no código de `src/` nesta data.

- **Um componente chamar a casca.** Só `App.tsx` e `overlay.tsx` importam
  `@tauri-apps/api`. `Atualizacao.tsx` fala com os plugins updater e process, e
  com nada mais.
- **O front falar com a Riot.** Nenhuma chamada para o LCU, para a porta 2999 nem
  para a Riot Web API. O único `fetch` é `lib/ddragon.ts`, para o CDN público, e
  a CSP só deixa `connect-src` para a própria origem, o IPC e o Data Dragon
  ([05](05-seguranca.md)).
- **Escrever no cliente do League.** Nenhum controle manda hover, pick, ban,
  runa ou item; nenhum comando da casca faz isso.
- **Guardar dado no navegador.** Nenhum `localStorage`, `sessionStorage` ou
  IndexedDB. O que persiste passa por um `set_*` da casca.
- **Consultar a casca em laço.** Nenhum `setInterval` chama comando. Os comandos
  rodam na montagem, num evento ou numa ação do jogador.
- **Tentar de novo.** Nenhuma chamada que falha é repetida.
- **Instalar uma versão sozinho.** `downloadAndInstall` só roda no clique de
  "Atualizar e reiniciar".
- **Deixar o simulador ligado fora da aba.** Trocar de aba manda
  `set_simulated_draft` com `null`.
- **Mostrar o Riot ID de outro jogador.** `displayName` dos outros nove só vira
  `key` do React (`GoldTracker.tsx:60`, `LiveBuild.tsx:103`). O único Riot ID
  na tela é o seu, na barra e na faixa "Sem base estatística".
- **Desenhar HTML vindo de fora.** Nenhum `dangerouslySetInnerHTML`, `innerHTML`
  ou `eval` em `src/`.
- **Abrir um link.** Nenhum `<a>`, `window.open` ou plugin de shell.
