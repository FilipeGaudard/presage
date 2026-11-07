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

Nos três otimistas, uma falha deixa a tela, a memória da casca e o disco em
dois estados diferentes, e nada avisa.

## 7. Erros hoje

| Onde | Chamadas com `.catch(() => void 0)` | O que acontece |
|---|---|---|
| `App.tsx` | 22: 9 leituras da carga inicial, `lane_profile` no evento, a limpeza dos ouvintes, 2 `set_simulated_draft`, os 3 efeitos de leitura, `set_draft_role`, `set_enemy_role`, `set_draft_prefs`, `set_autostart`, `set_pool`, `set_overlay_prefs` | nada na tela |
| `overlay.tsx` | 5 | nada; o overlay fica com o dado anterior |
| `Atualizacao.tsx` | 1, mais um `try` | o aviso some (falha calada por decisão, RNF-7) |
| `demo.tsx` | 1 (`loadStaticData`) | a demonstração fica sem retratos |
| `RecommendationPanel.tsx:70` | a prévia vira `null` | aparece "A prévia de rota aparece quando o outro time travar alguém.", que é texto de outro estado |
| `PoolBuilder.tsx:44` | a sugestão vira lista vazia | aparece "Nada a recomendar para este lugar." |
| `App.tsx:88` | o erro do Data Dragon vira `staticError` | a faixa com o texto do erro: o único erro visível |

O que isso faz na tela:

- **Escrita que falha some.** Uma pool, uma preferência do overlay ou do draft
  que não gravou continua na tela como gravada (§6).
- **Leitura que falha deixa o dado velho.** Os três comandos do ranking vão num
  `Promise.all`: se um rejeita, os três resultados se perdem e a tela continua
  com as listas da leitura anterior, sem dizer que envelheceram.
- **Motor que não carrega fica "carregando".** `engine.ready` é "há campeões"
  (`lib.rs:67`). Se a casca não conseguir os campeões nem pelo cache, a tela diz
  "Carregando dados de campeões…" para sempre ([GC-7](07-lacunas-da-casca.md)).

A proposta é um canal de erro ([GC-1](07-lacunas-da-casca.md)).

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
