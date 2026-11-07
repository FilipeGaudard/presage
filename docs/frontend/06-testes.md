# 06 — Testes do frontend

**O front tem harness desde 2026-11-07:** Vitest, Testing Library e jsdom
(`vitest.config.ts`, `npm test`), e o `docs.yml` roda os testes a cada push. Os
testes ficam ao lado do componente (`GoldTracker.test.tsx`), e o ID da linha vai
no título do teste (`it("TF-PARTIDA-3: …")`): é o marcador que o doccheck
procura em `*.test.ts(x)` de `src/`. O `tsc --noEmit` do `npm run build`
continua provando que o front concorda com `lib/types.ts`
([08 §4](../arquitetura/08-desenvolvimento-e-testes.md#4-os-testes)).

Este documento é o catálogo. Cada linha do §3 é um comportamento que um teste do
front prova, com uma entrada e uma saída concretas. A linha sem teste fica adiada,
uma por uma, em `tools/doccheck/deferred.txt` ([P-16](../produto/perguntas-em-aberto.md)),
e sai de lá quando o teste chega. Linha nova sem teste nem adiamento falha o
doccheck.

Cada linha diz se descreve o comportamento **de hoje** (o teste passaria no
código atual e trava o que já funciona) ou o **desejado** (o teste falharia hoje
e fecha uma lacuna do [07](07-lacunas-da-casca.md) ou um defeito do
[03](03-estado-e-atualizacao.md)).

## 1. Camadas

| Camada | Ferramenta | O que provaria |
|---|---|---|
| Estático | um script Node sobre `src/`, no espírito do doccheck | Só `App.tsx` e `overlay.tsx` importam `@tauri-apps/api`; nenhum HTML cru, `localStorage` ou link |
| Unidade | Vitest | As contas que a tela ainda faz ([03 §9](03-estado-e-atualizacao.md#9-o-que-a-tela-ainda-calcula)): o relógio, a cor do perfil, `draftVazio` |
| Componente | Vitest, Testing Library e jsdom, com os dados de `demo.tsx` | Cada componente com props: o texto de cada estado e o argumento de cada callback |
| Casca do front | Vitest com `mockIPC` de `@tauri-apps/api/mocks` | Que `App.tsx` e `overlay.tsx` chamam os comandos certos, com os argumentos certos, quando o que leem muda, e o que fazem quando um comando rejeita |
| Ponta a ponta | WebdriverIO com `tauri-driver`, no Windows | O que só o app real tem: atalho global, janela atravessável, arrasto do overlay |

A ordem sugerida é a do default do P-16: componentes primeiro, porque já são
puros e o `demo.tsx` já tem dados de tamanho realista; depois a casca do front
com `mockIPC`, que é onde mora o tratamento de erro.

## 2. Dados de teste

- Os objetos de `demo.tsx` (`draft`, `recommendations`, `live`, `laneReport`,
  `review`, `perfilDemo`, `planoDemo`) viram fixtures exportadas, para o teste e a
  demonstração usarem o mesmo dado.
- `StaticData` de teste é montado à mão com meia dúzia de campeões; nenhum teste
  baixa o Data Dragon.
- Nenhum Riot ID, puuid ou partida de pessoa real. O Riot ID de `demo.tsx:85`
  precisa ser conferido como fictício antes de virar fixture.

## 3. Catálogo

| TF-ID | Tipo | O que prova | Regra |
|---|---|---|---|
| TF-CASCA-1 | componente | Hoje. Com `engine.ready`, `version: "16.21.1"` e `__APP_VERSION__ = "0.7.0"`, a barra diz "patch 16.21.1 · v0.7.0"; com `engine: null`, "draft engine · v0.7.0" | L-12 |
| TF-CASCA-2 | componente | Hoje. Com `check()` resolvendo a versão 0.8.0, aparece "Versão 0.8.0 disponível"; `downloadAndInstall` só é chamado no clique de "Atualizar e reiniciar"; "Depois" esconde o aviso; com `check()` rejeitando, nada aparece | RNF-7, R-19 |
| TF-CASCA-3 | componente | Hoje. Na aba Pool, um `presage://review` com uma leitura troca a aba para Pós-jogo e mostra o campeão da leitura | R-16 |
| TF-CASCA-4 | componente | Hoje. Um `rank_draft` que rejeita mostra um aviso que nomeia o ranking; a lista anterior não continua na tela como se fosse atual | GC-1 |
| TF-CASCA-5 | estático | Hoje. Em `src/`, só `App.tsx` e `overlay.tsx` importam `@tauri-apps/api`, e nenhum arquivo usa `dangerouslySetInnerHTML`, `innerHTML`, `eval`, `localStorage` ou `<a href` | 05 §1 |
| TF-CASCA-6 | componente | Hoje. Um motivo `<img src=x onerror=alert(1)>` em `ScoredChampion.reasons` aparece como texto no detalhe, e o DOM não ganha nenhum `<img>` com `onerror` | 05 §1 |
| TF-DRAFT-1 | componente | Hoje. `draft: null` com `connected: false` mostra "Sem draft ativo" e "Abra o League of Legends."; com `connected: true` e `phase: "Lobby"`, "Cliente conectado (Lobby)." | R-18 |
| TF-DRAFT-2 | unidade | Hoje. Um evento com `timeLeftMs: 20000` e 5 s de relógio falso depois mostram 15 no `DraftBoard`; 25 s depois, 0, nunca negativo | RNF-4 |
| TF-DRAFT-3 | componente | Hoje. Inimigo travado sem `position`, com presumida `Top`, mostra "TOP (presumida)"; escolher MID chama `onEnemyRole(id, "Mid")`, e `App` manda `set_enemy_role` com `{ championId: id, role: "middle" }`; a opção vazia manda `role: null` | R-4 |
| TF-DRAFT-4 | componente | Hoje. Com `localPosition: ""`, as cinco abas de rota aparecem e MID manda `set_draft_role` com `{ role: "middle" }`; com `localPosition: "middle"`, não aparecem | R-1 |
| TF-DRAFT-5 | componente | Desejado. Pool só de `Mid` e nenhuma rota escolhida: a tela diz que a lista é de MID, e não "sem rota, a lista inclui todos" | R-1, GC-5 |
| TF-DRAFT-6 | componente | Hoje. Com 3 `ScoredChampion` de `inPool: true` e 20 de `false`, "Sua pool" tem os 3 com ✳ e "Melhores no geral" tem os 15 primeiros dos outros, na ordem recebida | R-1, C-5 |
| TF-DRAFT-7 | componente | Hoje. `hasGlobal: false` mostra a faixa "Sem base estatística"; `globalPatch: "16.20"` com `version: "16.21.1"` mostra "A base é do patch 16.20 e o jogo está no 16.21.1. Os números são de outro patch." | RNF-1 |
| TF-DRAFT-8 | componente | Hoje. `currentAction: "ban"` com `bans: []` mostra "Nada travado do lado deles ainda…"; um ban com `fromPool: true` leva "sua lista" | R-3 |
| TF-DRAFT-9 | componente | Hoje. Clicar um candidato chama `onPreview(id)` uma vez e mostra "Calculando a prévia…" até a promessa resolver | R-6 |
| TF-DRAFT-10 | componente | Hoje. Com um candidato aberto, marcar a rota de um inimigo chama `lane_analysis` com `{ candidate }` de novo (até a 0.7.0 não chamava, A11) | R-6, R-4 |
| TF-SIM-1 | componente | Hoje. Abrir a aba chama `set_simulated_draft` com o draft da tela, e `rank_draft` só é chamado depois que ele resolve | R-8 |
| TF-SIM-2 | componente | Hoje. Trocar para a aba Draft chama `set_simulated_draft` com `{ draft: null }` | R-8 |
| TF-SIM-3 | componente | Hoje. Digitar "Ahri" e sair do campo grava `championId: 103` e `locked: true`; "Ahrii" não muda nada | R-8 |
| TF-ROTAS-1 | componente | Hoje. Rota com `fromData: false` traz "sem base de partidas: só a conta do duelo"; `source: "partida"` troca o subtítulo para "níveis e itens reais da partida" e desenha o ponto de agora | RNF-1, R-11 |
| TF-ROTAS-2 | componente | Hoje. `RunePlan` com `fromData: false` mostra "Isto sai de regras sobre a mecânica do confronto…" e não fala em partidas | R-7, RNF-1 |
| TF-ROTAS-3 | componente | Hoje. Escolher a rota ou mudar a pool chama `lane_analysis` e `rune_plan` de novo; sem draft e sem partida, a runa some junto com as rotas | R-6, R-7 |
| TF-PARTIDA-1 | componente | Hoje. `live: null` com `connected: true` mostra "Entre numa partida e os timers e a build adaptativa ligam sozinhos."; sem cliente, "Abra o cliente do League of Legends." | R-11 |
| TF-PARTIDA-2 | componente | Hoje. Um `Spawn` com `observed: false` leva "estimado"; com `secondsLeft: 0`, "disponível" | R-13 |
| TF-PARTIDA-3 | componente | Hoje. Um inimigo com `unspent: 1200` não ganha `+1.200` na linha dele; o jogador local ainda ganha | C-4, R-12, GC-2 |
| TF-PARTIDA-4 | componente | Hoje. A prioridade sempre traz "Ranking de valor, não de posição.", com ou sem rotas | R-14, C-4 |
| TF-POS-1 | componente | Hoje. `review: null` mostra "Nenhuma partida lida ainda" | R-16 |
| TF-POS-2 | componente | Hoje. `role: "Support"` desenha "Visão, você menos o outro suporte" e não desenha "Tropas, você menos o seu oponente" | R-16, L-27 |
| TF-POS-3 | componente | Hoje. No perfil, `value: 3, sd: 1` ganha `good`, `value: 1, sd: 1` fica `even`, e `deaths` com `value: 3, sd: 1` ganha `bad`; nenhuma linha soma as métricas | R-17, C-7, RNF-11 |
| TF-POOL-1 | componente | Hoje. Pool vazia abre em "Em que rota você joga?"; pool com uma entrada de `Mid` abre em "Sua pool", na aba MID | R-9 |
| TF-POOL-2 | componente | Hoje. No passo 2, com dois core o botão diz "Faltam 1" e está desligado; com três, a grade desliga e o botão diz "Analisar os três" | R-9 |
| TF-POOL-3 | componente | Hoje. No passo 3, aceitar um candidato acrescenta uma entrada `situational` e chama `onSuggest` de novo com a mesma rota | R-9 |
| TF-POOL-4 | componente | Hoje. Um `set_pool` que rejeita mostra o erro e devolve a tela à pool que a casca tem | GC-1 |
| TF-POOL-5 | componente | Hoje. Com cinco bans, o campo "Adicionar ban…" some; "↑" no segundo troca os dois primeiros e grava a pool | R-3 |
| TF-AJUSTES-1 | componente | Hoje. Desmarcar "Ouro em itens" chama `set_overlay_prefs` com `{ prefs: { hidden: ["gold"] } }`, e a caixa muda antes da resposta | R-15 |
| TF-AJUSTES-2 | componente | Hoje. "Iniciar o Presage com o Windows" só muda depois que `set_autostart` resolve; se rejeitar, continua desmarcada | R-19 |
| TF-OVERLAY-1 | componente | Hoje. Sem snapshot, o overlay não desenha nada e não chama `build_suggestions` | R-15 |
| TF-OVERLAY-2 | componente | Hoje. Um `presage://prefs` com `hidden: ["gold"]` tira "Ouro em itens" sem recarregar | R-15 |
| TF-OVERLAY-3 | componente | Hoje. Com snapshot, o overlay chama `build_suggestions` com `{ limit: 2 }` e mostra no máximo dois itens | C-5 |
| TF-OVERLAY-4 | componente | Desejado. Com uma sétima seção em `OVERLAY_SECTIONS`, seis escondidas não apagam o overlay | GC-11 |
| TF-OVERLAY-5 | ponta a ponta | Desejado. Em modo clicável, arrastar a cabeça move a janela, e a posição volta depois de reiniciar | R-15, GC-8 |

## 4. No CI

Hoje o `instalador.yml` roda `cargo test --workspace` no passo "Testes", e o
`tsc --noEmit` só roda dentro do `npm run tauri build`, pelo `beforeBuildCommand`.
Quando o harness existir, `npm test` entra no passo "Testes", antes de
empacotar. Até lá, o doccheck mostra estas linhas como `ui-test` pendentes.
