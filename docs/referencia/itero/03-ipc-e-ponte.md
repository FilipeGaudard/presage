# 03 — IPC e a ponte do preload

## 1. O que o preload expõe

`dist-electron/preload.mjs` usa `contextBridge.exposeInMainWorld` e põe nove
objetos em `window` de **toda** janela (todas carregam o mesmo preload):

| Objeto | Métodos → canal |
|---|---|
| `ipcRenderer` | `on`, `off`, `send`, `invoke` — **genéricos**: aceitam qualquer nome de canal |
| `logger` | `debug`, `info`, `warn`, `error` → `logger:debug`, `logger:info`, `logger:warn`, `logger:error` |
| `errorReporter` | `report` → `report:send` |
| `network` | `online`, `offline` → `network:online`, `network:offline` |
| `electronAPI` | `openExternal` → `open-external`; `restartApp` → `restart-app`; `openLogFolder` → `open-log-folder`; `openOWLogFolder` → `open-ow-log-folder`; `checkForUpdates` → `check-for-updates`; `uninstall` → `uninstall`; `quit` → `quit`; `resetCache` → `reset-cache`; `restartAppAsAdmin` → `restart-app-as-admin` |
| `api` | `benchSwap`, `createItemSet`, `createPerkPage`, `editItemSets`, `editPerkPage`, `editSessionSelection`, `getCurrentPerkPage`, `getAramChampionBuilds`, `getAramChampionAugments`, `getItemSets`, `getPerkPages`, `patchSessionAction`, `fetchDraftData`, `updateGameSettings`, `setHashedEmail`, `clearHashedEmails`, `openElectronCMP` → `api:<kebab>` (o último é `api:open-ow-cmp`) |
| `overwolfPackages` | `getAvailableChannels`, `getChannel`, `setChannel` → `overwolf-packages:get-available-channels`, `overwolf-packages:get-channel`, `overwolf-packages:set-channel` |
| `windows` | `open`, `close`, `toggle`, `center`, `openDevtools`, `minimize`, `maximize`, `isMaximized`, `resize`, `reposition` → `windows:*` |

Os objetos nomeados são açúcar: como `ipcRenderer.invoke` aceita qualquer canal,
qualquer janela alcança qualquer `ipcMain.handle`, inclusive os das stores
(abaixo), que não têm método nomeado.

## 2. Canais tratados no processo principal

### Literais (`ipcMain.handle`)

| Canal | Efeito | Onde |
|---|---|---|
| `logger:debug`, `logger:info`, `logger:warn`, `logger:error` | escreve no log do processo principal | `main.js:35937` |
| `report:send` | encaminha relatório de erro à janela `background` (função `reporterror`) | — |
| `network:online`, `network:offline` | `[NetworkStatus]`: refaz configuração e draft ao reconectar | `main.js:36508` |
| `open-external` | `shell.openExternal(url)` com a URL recebida | `main.js:35922` |
| `restart-app` | relança | idem |
| `open-log-folder`, `open-ow-log-folder` | abre a pasta de log do app / do Overwolf | idem |
| `check-for-updates` | `[AutoUpdater]` | idem |
| `uninstall` | roda o desinstalador e sai | idem |
| `quit` | sai | idem |
| `reset-cache` | zera `config`, `settings` e `window-state` e relança | idem |
| `restart-app-as-admin` | diálogo e relançamento elevado | idem |
| `windows:open`, `windows:toggle` | cria / alterna a janela pelo nome | `main.js:36588` |
| `windows:close`, `windows:center`, `windows:minimize`, `windows:maximize`, `windows:is-maximized`, `windows:resize`, `windows:reposition`, `windows:open-devtools` | operam na janela pelo nome ou na que chamou; devtools só abre em build de desenvolvimento (`devTools: false` em produção) | idem |
| `overwolf-packages:get-available-channels`, `overwolf-packages:get-channel`, `overwolf-packages:set-channel` | canais Overwolf | `main.js:36531` |
| `api:patch-session-action` | `PATCH` na ação do champ select (hover/lock pela UI) e anota `lastAppHover` | `main.js:26233` |

### Registrados pelo helper (`api:*`, `main.js:26233-26354`)

O helper embrulha cada chamada com rótulo de log, valor de reserva em erro e um
objeto de depuração.

| Canal | Chamada | Reserva |
|---|---|---|
| `api:bench-swap` | LCU `POST /lol-champ-select/v1/session/bench/swap/{id}` | `false` |
| `api:create-item-set` | LCU `POST /lol-item-sets/v1/item-sets/{summonerId}/sets` | `false` |
| `api:edit-item-sets` | LCU `PUT` idem | `false` |
| `api:get-item-sets` | LCU `GET` idem | `null` |
| `api:create-perk-page` | LCU `POST /lol-perks/v1/pages` | `null` |
| `api:edit-perk-page` | LCU `PUT /lol-perks/v1/pages/{id}` | `null` |
| `api:get-perk-pages` | LCU `GET /lol-perks/v1/pages` | `null` |
| `api:get-current-perk-page` | LCU `GET /lol-perks/v1/currentpage` | `null` |
| `api:edit-session-selection` | LCU `PATCH /lol-champ-select/v1/session/my-selection` (feitiços, skin) | `false` |
| `api:get-aram-champion-builds` | backend: builds de ARAM do campeão | `null` |
| `api:get-aram-champion-augments` | backend: tiers de aprimoramento do campeão | `null` |
| `api:fetch-draft-data` | `[DraftData]` para a fase atual | `null` |
| `api:update-game-settings` | LCU `GET /lol-game-settings/v1/game-settings` → `config` | `null` |
| `api:set-hashed-email` | Overwolf: gera e registra hashes do e-mail para anúncio personalizado | `false` |
| `api:clear-hashed-emails` | Overwolf: limpa os hashes | `false` |
| `api:open-ow-cmp` | Overwolf: janela de privacidade de anúncios (CMP) | — |

### Stores (`store:<nome>:<op>`)

Cada store do processo principal ([01 §3](01-processo-principal.md#3-estado-as-stores))
registra `subscribe` (a janela passa a receber as mudanças das chaves pedidas),
`get` e `set`; `settings` também tem `update` (por chave), `reset` e `enable`.
Nomes: `settings`, `config`, `launcher`, `champ-select`, `q7z2k`,
`game-clock`, `damage`, `ult-cooldowns`, `live-players`, `gold`,
`player-stats`, `objectives`, `overlay-status`, `aram-augments`, `scouting`.

`store:<nome>:set` aplica o objeto recebido direto no estado do processo
principal, que o repassa a todas as janelas inscritas.

## 3. Do principal para as janelas (`webContents.send`)

| Canal | Para | Quando |
|---|---|---|
| `log` | qualquer janela | depuração |
| `window:maximize-change` | a janela | maximizou/restaurou |
| `post-match-analysis:start`, `:end`, `:error`, `:cancel` | `desktop` | análise pós-partida ([01 §6](01-processo-principal.md#6-pós-partida)) |
| `overwolf-packages:channel-ready` | `desktop` | troca de canal baixada |
| `save-settings-to-db` | `background` | configurações mudaram (debounce 5 s) |
| `before-quit-cleanup` | `background` | antes de sair |
| `send-error-report` | `background` | erro a reportar |

## 4. Contraste com o Presage

O Presage expõe à interface uma lista fechada de comandos Tauri, cada um com
tipo de argumento, e a interface não tem um canal genérico para o processo
nativo. O cliente do LCU do Presage só tem `GET` (`src-tauri/src/lcu/client.rs`).
O iTero escreve no LCU (hover, lock, runas, itens), e é isso que dá à ponte dele
o tamanho que ela tem.
