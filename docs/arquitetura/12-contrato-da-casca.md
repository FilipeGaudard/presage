# 12 — Contrato da casca

`casca`. O que a interface pode pedir à casca (`src-tauri/src`), o que a casca
empurra sozinha, o que ela chama lá fora e o que ela deixa em disco. É o
equivalente de um documento de API REST: cada linha aponta o código, e o
`doccheck` confere comandos, eventos e arquivos contra ele.

Os tipos que atravessam a ponte estão no modelo de dados (10 §32 para os da casca,
os demais na seção do módulo do core). A forma de cada arquivo em disco está em
10 §33.

## 1. Convenções

### Transporte

- **Comando:** a interface chama `invoke(nome, args)` (`@tauri-apps/api/core`).
  Só alcança os 26 nomes de `generate_handler!` (`lib.rs:720`); não há canal
  genérico ([07 §2](07-seguranca.md#a-webview)).
- **Evento:** a casca empurra com `app.emit(nome, carga)`, que chega às duas
  janelas (`main` e `overlay`). A interface ouve com `listen` (`@tauri-apps/api/event`).
- **Quem fala com a casca:** só `src/App.tsx` e `src/overlay.tsx`. Os componentes
  recebem dados por props; a exceção é `Atualizacao.tsx`, que fala com o plugin
  do atualizador, não com um comando nosso (§7).
- **Todos os comandos são síncronos** (`fn`, nenhum `async fn`). O Tauri roda
  comando síncrono na thread principal.

### Nomes no fio

- **camelCase.** Os tipos que cruzam a ponte levam `#[serde(rename_all = "camelCase")]`.
  O Tauri faz o mesmo com os argumentos: `set_enemy_role(champion_id, …)` recebe
  `{ championId }` (`App.tsx:274`).
- **Exceções que não são camelCase:**
  - `Role` sai como a variante: `"Top"`, `"Jungle"`, `"Mid"`, `"Bot"`, `"Support"`, `"Unknown"`;
  - `Tier` sai em minúsculas: `"core"`, `"situational"`, `"extra"`;
  - `Pool` não tem `rename_all`, mas os dois campos (`entries`, `bans`) são uma
    palavra só.
- **`Option`** vira `null`. **Tupla** vira array: `get_snapshot` devolve
  `[status, draft]`, `build_suggestions` devolve `[contexto, itens]`.

### Erros

- **Só quatro comandos podem falhar:** `set_draft_prefs`, `set_pool`,
  `set_overlay_prefs` e `set_autostart`. Os quatro devolvem `Result<(), String>`,
  e o `Err` é o texto do erro de disco ou do registro (`err.to_string()`). O
  `invoke` rejeita com esse texto.
- **Primeiro o disco, depois a memória.** Os três que gravam JSON só trocam o
  estado em memória depois que `storage::write_json` voltou `Ok` (`lib.rs:203`,
  `lib.rs:421`, `lib.rs:441`).
- **Os outros 22 nunca falham.** Sem dado, devolvem lista vazia, `None` ou o
  padrão. Uma rejeição deles só vem do próprio Tauri: argumento que não
  desserializa, ou pânico de um `Mutex` envenenado (todo comando faz
  `.lock().unwrap()`).
- **A interface mostra a falha.** Toda chamada de `App.tsx` passa por
  `chamar`, e o texto do `Err` vira uma faixa
  ([frontend 03 §7](../frontend/03-estado-e-atualizacao.md#7-erros-hoje)). Em
  `set_pool`, `set_overlay_prefs` e `set_draft_prefs` a tela muda antes da
  resposta e volta ao valor de antes se a casca recusar; `set_autostart` espera o
  `Ok` antes de marcar a caixa.

### Rota: dois vocabulários

A rota entra num vocabulário e sai noutro:

| Direção | Vocabulário | Onde |
|---|---|---|
| entra em `set_draft_role`, `set_enemy_role`, `pool_suggestions` | o do LCU: `"top"`, `"jungle"`, `"middle"`, `"bottom"`, `"utility"` | `Role::from_lcu` (`core/src/engine/champion.rs:15`) |
| entra em `set_pool` (`PoolEntry.role`) e sai em tudo que devolve `Role` | o do serde: `"Top"`, `"Jungle"`, `"Mid"`, `"Bot"`, `"Support"` | `#[derive(Serialize, Deserialize)]` de `Role` |
| entra em `set_simulated_draft` (`DraftSlot.position`, `DraftState.localPosition`) | o do LCU, porque `DraftState` é o formato do champ select | `core/src/draft.rs` |

Uma string que `from_lcu` não conhece vira `Role::Unknown`, e os três comandos a
tratam como "sem rota", calados: `set_draft_role("Mid")` apaga a escolha em vez
de escolher o meio. A interface converte com `ROLE_TO_LCU` (`src/lib/types.ts:473`)
antes de chamar.

### Ritmo

A casca não espera a interface pedir: `watcher::run` lê o cliente num laço e
empurra eventos ([ADR-0002](../adr/0002-polling-em-vez-de-websocket.md)). O
intervalo é a pausa depois das chamadas da volta, então o período real é o
intervalo mais o tempo das requisições.

| Fase do fluxo | Pausa | Constante |
|---|---|---|
| `ChampSelect` | 350 ms | `POLL_CHAMP_SELECT` (`watcher.rs:43`) |
| `InProgress` | 1 s | `POLL_LIVE` (`watcher.rs:44`) |
| qualquer outra, com cliente | 1,5 s | `POLL_IDLE` (`watcher.rs:45`) |
| sem cliente | 3 s | `POLL_DISCONNECTED` (`watcher.rs:46`) |

A interface não faz polling. Ela chama os comandos quando um evento muda o estado
dela: o ranking a cada `presage://draft`, a build e o ouro a cada
`presage://live` (`App.tsx:194`, `overlay.tsx:61`). Durante a partida isso dá
quatro `invoke` por segundo na janela principal e três no overlay.

### O estado compartilhado

`Shared` (`watcher.rs:60`, campos em 10 §32.5) é o único estado da casca. Um
`Arc<Shared>` vai para o `app.manage` (`lib.rs:652`) e para as três tarefas de
fundo. Todo comando lê dele; nenhum comando fala com a rede.

| Campo | O quê | Quem escreve | Quem lê |
|---|---|---|---|
| `status` | conexão e fase | watcher (`publish_status`, `watcher.rs:358`) | `get_snapshot` |
| `draft` | o champ select do cliente | watcher a cada 350 ms (`watcher.rs:271`); `None` em `clear_draft` e ao perder o cliente (`watcher.rs:224`) | `get_snapshot`, e o draft do motor quando não há simulador |
| `simulated` | o draft da aba Simulador | `set_simulated_draft` | `current_draft`, que o prefere ao do cliente (`lib.rs:109`) |
| `champions`, `items`, `runes` | os índices do Data Dragon | a tarefa de carga (`lib.rs:673`), uma vez | quase todos os comandos, e o watcher |
| `live` | a última `LiveSnapshot` | watcher a cada 1 s (`watcher.rs:323`); `None` fora da partida | `get_live`, `build_suggestions`, `gold_report`, `skirmish_focus`, `lane_analysis`, `rune_plan` |
| `stats` | a loja de estatísticas (`Layered`) | `rebuild_stats` (`lib.rs:89`): ao subir e quando o histórico pessoal termina | ranking, minimax, análise de rota, pool, runas, `lane_profile`, `engine_status` |
| `dataset` | a base de partidas | setup, de `base-de-partidas.json` (`lib.rs:629`) | `rebuild_stats`, `engine_status` |
| `identity` | puuid, Riot ID e região da conta | watcher, ao conectar (`watcher.rs:188`) | `profile::refresh`, `engine_status` (só o Riot ID) |
| `pool` | a pool | setup (`lib.rs:616`), `set_pool` | `get_pool`, contexto do draft, bans, `pool_suggestions` |
| `draft_prefs` | preferência do draft | setup, `set_draft_prefs` | `get_draft_prefs`, contexto do draft |
| `overlay_prefs` | seções escondidas | setup, `set_overlay_prefs` | `get_overlay_prefs` |
| `draft_role` | a rota escolhida na tela | `set_draft_role`; nunca é limpa | contexto do draft |
| `enemy_roles` | as rotas marcadas nos inimigos | `set_enemy_role`; esvaziada por `clear_draft` (`watcher.rs:151`) | contexto do draft, `get_enemy_roles` |
| `owned` | quem a conta pode escolher | watcher, uma vez por champ select (`watcher.rs:250`); `None` em `clear_draft` | contexto do draft, fora do simulador e sem "mostrar todos" (`lib.rs:132`) |
| `recording` | a partida quadro a quadro | watcher (`recorder::observe`, `watcher.rs:291`) | `close_recording`, que a esvazia |
| `last_review` | a última leitura pós-partida | setup, de `ultima-partida.json`; `close_recording` (`watcher.rs:144`) | `last_review` |

As tarefas de fundo, todas lançadas no `setup` (`lib.rs:612`):

1. `watcher::run` (`lib.rs:669`): o laço do cliente e da partida.
2. A carga do Data Dragon (`lib.rs:673`): campeões, itens e runas em sequência, e
   no fim `presage://engine`.
3. `profile::refresh` (`lib.rs:707`): o histórico pessoal, uma vez por abertura do
   app, e no fim `presage://engine`.

## 2. Comandos

Os 26 de `generate_handler!` (`lib.rs:720`), na ordem em que `lib.rs` os define.
"Args" são os nomes como a interface os manda; os números fixos no Rust estão na
coluna. As linhas de "Quem chama" são de `src/`.

| Comando | Args | Devolve | Erros | Quem chama |
|---|---|---|---|---|
| `get_snapshot` | — | `(Option<ConnStatus>, Option<DraftState>)` → `[ConnStatus \| null, DraftState \| null]`: o último estado que o watcher viu | nenhum | `App.tsx:94`, ao montar |
| `engine_status` | — | `EngineStatus` (10 §32.1) | nenhum | `App.tsx:105`, ao montar; depois o mesmo valor chega por `presage://engine` |
| `rank_draft` | — | `Vec<Scored>` → `ScoredChampion[]`; vazio sem draft | nenhum | `App.tsx:177` (Draft e Simulador), quando muda draft, pool, rota, marca ou preferência |
| `set_draft_role` | `role: Option<String>`, vocabulário do LCU; `null` volta para a rota da pool | `()` | nenhum; string desconhecida apaga a escolha | `App.tsx:266` (seletor de rota do painel de recomendação) |
| `set_enemy_role` | `championId: i64`, `role: Option<String>` (LCU); `null` desfaz a marca | `()` | nenhum; string desconhecida desfaz a marca | `App.tsx:274` (quadro do draft) |
| `get_enemy_roles` | — | `Vec<(i64, Role)>` → `[number, Role][]`, rota no vocabulário de saída | nenhum | **ninguém.** Registrado (`lib.rs:727`), sem `invoke` em `src/`: a interface guarda as marcas no próprio estado (`App.tsx:74`) |
| `set_simulated_draft` | `draft: Option<DraftState>`; `null` volta para o cliente | `()`; o draft passa por `DraftState::normalized` (`lib.rs:193`) | nenhum | `App.tsx:157` (aba Simulador), `App.tsx:153` (`null` ao sair dela) |
| `get_draft_prefs` | — | `DraftPrefs` | nenhum | `App.tsx:111`, ao montar |
| `set_draft_prefs` | `prefs: DraftPrefs` | `Result<(), String>` | `Err` se `draft.json` não gravou; a memória fica como estava | `App.tsx:297` |
| `lane_profile` | — | `Option<LaneProfile>`; `None` sem chave da Riot ou antes do histórico | nenhum | `App.tsx:109`, ao montar; `App.tsx:120`, a cada `presage://engine` |
| `minimax_draft` | — (largura 12, fixa em `lib.rs:226`) | `Vec<MinimaxPick>`; vazio sem draft | nenhum | `App.tsx:178`, junto com `rank_draft` |
| `ban_suggestions` | — (10 sugestões, fixo em `lib.rs:237`) | `Vec<BanSuggestion>`; vazio sem draft | nenhum | `App.tsx:179`, junto com `rank_draft` |
| `get_live` | — | `Option<LiveSnapshot>` (10 §32.3) | nenhum | `App.tsx:107`, `overlay.tsx:52`, ao montar |
| `build_suggestions` | `limit: Option<usize>`, padrão 8 (`lib.rs:268`) | `Option<(BuildContext, Vec<ItemSuggestion>)>` → `[BuildContext, ItemSuggestion[]] \| null`; `None` sem partida ou sem jogador local | nenhum | `App.tsx:204` (`limit: 8`), `overlay.tsx:74` (`limit: 2`) |
| `lane_analysis` | `candidate: Option<i64>`; com id, a prévia dele no seu lugar, e a partida é ignorada (`lib.rs:300`) | `Option<LaneReport>`; a partida ganha do draft (`analysis::report`) | nenhum | `App.tsx:231` (`candidate: null`, abas Rotas e Draft); `App.tsx:258` (a prévia, pedida pelo painel de recomendação) |
| `skirmish_focus` | — | `Option<lanes::Focus>` → `SkirmishFocus`; `None` sem partida, sem jogador local ou sem escaramuça que caiba | nenhum | `overlay.tsx:76` |
| `gold_report` | — | `Option<GoldReport>`; `None` sem partida | nenhum | `App.tsx:205`, `overlay.tsx:75` |
| `last_review` | — | `Option<coach::Report>` → `CoachReport \| null` | nenhum | `App.tsx:108`, ao montar |
| `pool_suggestions` | `role: Option<String>` (LCU; `null` vira `Unknown`), `limit: Option<usize>`, padrão 6 (`lib.rs:372`) | `PoolPlan` (10 §32.2) | nenhum | `App.tsx:248` (`limit: 6`), pedido pelo construtor de pool no último passo |
| `rune_plan` | — | `Option<runes::RunePlan>`; `None` sem análise de rota, sem a sua rota nela ou sem runas carregadas | nenhum | `App.tsx:232`, junto com `lane_analysis` |
| `get_pool` | — | `Pool` | nenhum | `App.tsx:106`, ao montar |
| `set_pool` | `pool: Pool`, rota no vocabulário de saída (`"Mid"`) | `Result<(), String>` | `Err` se `pool.json` não gravou | `App.tsx:314` |
| `get_overlay_prefs` | — | `OverlayPrefs` | nenhum | `App.tsx:110`, `overlay.tsx:43`, ao montar |
| `set_overlay_prefs` | `prefs: OverlayPrefs` | `Result<(), String>`; com `Ok`, emite `presage://prefs` | `Err` se `overlay.json` não gravou, e aí não há evento | `App.tsx:321` (aba Ajustes) |
| `get_autostart` | — | `bool`; erro do plugin vira `false` (`lib.rs:452`) | nenhum | `App.tsx:112`, ao montar |
| `set_autostart` | `enabled: bool` | `Result<(), String>` | `Err` com o texto do plugin, que escreve no registro (§7) | `App.tsx:301` (aba Ajustes) |

Os números fixos e quem os escolhe:

- **`limit` de `build_suggestions`:** o padrão 8 da casca nunca é usado, porque os
  dois chamadores mandam o próprio: 8 na janela e 2 no overlay, que mostra duas
  opções (C-5).
- **`limit` de `pool_suggestions`:** a interface manda 6, o mesmo do padrão da casca.
- **Largura 12 do minimax e 10 bans:** só no Rust, sem argumento
  ([02 §6](02-motor-de-draft.md#6-draft-com-resposta)).

O contexto do draft é montado do mesmo jeito para `rank_draft`, `minimax_draft`,
`lane_analysis` e `rune_plan` (`draft_context`, `lib.rs:119`). Ele junta a rota
resolvida, as marcas nos inimigos e, fora do simulador e sem "mostrar todos", a
lista da conta ([ADR-0017](../adr/0017-uma-leitura-do-champ-select.md)).
`ban_suggestions` não passa por ele: lê o draft e a lista de bans direto
(`lib.rs:237`).

## 3. Eventos

Os nomes são constantes dos dois lados: `watcher.rs:35` a `watcher.rs:41` e
`lib.rs:37` no Rust, `App.tsx:44` e `overlay.tsx:17` no TS.

| Evento | Carga | Quando | Emitido em | Ouvido em |
|---|---|---|---|---|
| `presage://status` | `ConnStatus` | a conexão ou a fase do fluxo mudou. Só emite na mudança (`publish_status`) | `watcher.rs:361` | `App.tsx:115` |
| `presage://draft` | `DraftState`, ou `null` | o champ select mudou, sem contar o relógio (`same_draft`, `watcher.rs:348`); `null` quando ele acaba | `watcher.rs:269`; `null` em `watcher.rs:154` | `App.tsx:129`, que também apaga as marcas da tela no `null` |
| `presage://engine` | `EngineStatus` | a carga do Data Dragon terminou; o histórico pessoal terminou. Nos dois casos, com ou sem sucesso | `lib.rs:702`, `lib.rs:715` | `App.tsx:118`, que relê `lane_profile` |
| `presage://live` | `LiveSnapshot`, ou `null` | a leitura de 1 s mudou alguma coisa (o relógio muda sempre); `null` ao sair de `InProgress` | `watcher.rs:321`; `null` em `watcher.rs:334` | `App.tsx:122`, `overlay.tsx:53` |
| `presage://review` | `coach::Report` → `CoachReport` | a partida acabou com pelo menos 60 quadros gravados | `watcher.rs:143` | `App.tsx:125`, que traz a aba Pós-jogo para a frente |
| `presage://prefs` | `OverlayPrefs` | `set_overlay_prefs` gravou | `lib.rs:443` | `overlay.tsx:45` |

O que não vira evento:

- **Partida carregando.** Em `InProgress`, enquanto a Live Client não responde,
  nada é emitido (`watcher.rs:326`).
- **Perder o cliente no meio do champ select.** A casca zera `draft` e redescobre
  (`watcher.rs:224`) sem emitir `presage://draft` com `null`. A tela fica com o
  último draft até o próximo evento.
- **A lista da conta e as marcas.** `owned` e `enemy_roles` mudam sem evento; a
  interface só vê o efeito no próximo ranking.

## 4. Nomes de um lado e do outro

`src/lib/types.ts` espelha à mão os tipos que a casca serializa. Nada no build
liga os dois lados: o Rust compila e o `tsc` passa com um campo trocado. Quem
confere é o `doccheck`, e só nos pares que o modelo de dados (10) marca com `↔`
no título. Para esses, ele exige que os campos do doc sejam os do Rust e que os
nomes serializados do Rust sejam os da interface TS. Um par sem `↔` não é
conferido.

A maioria dos tipos tem o mesmo nome nos dois lados (`DraftState`, `LiveGame`,
`LaneReport`, `GoldReport`, `Pool`…). Estes não têm:

| TS (`src/lib/types.ts`) | Rust | O que muda |
|---|---|---|
| `ScoredChampion` | `core::engine::Scored` | só o nome do tipo |
| `CoachReport`, `CoachNote` | `core::engine::coach::Report`, `Note` | só o nome; `Report.role` é `#[serde(default)]` e no TS é opcional |
| `SkirmishFocus` | `core::engine::lanes::Focus` | só o nome |
| `WinPoint` | `core::engine::wincon::Point` | o nome; `side` e `source` são uniões inline no TS (`"nosso" \| "deles"`, `"regra" \| "atributos"`) para os enums `Side` e `Source` |
| `LiveScores` | `core::live::RawScores` | o nome: o mesmo tipo serve de JSON cru da Live Client e de saída |
| `PoolAnalysis`, `PoolCandidate` | `core::engine::poolgap::Analysis`, `Candidate` | só o nome |
| `PoolFinding` | `core::engine::poolgap::Finding` | o nome e um campo: `trait` no fio, `kind` no Rust (`#[serde(rename = "trait")]`, `trait` é palavra reservada do Rust) |
| `PoolTrait` | `core::engine::poolgap::Trait` | o nome; união de strings |
| `OverlaySection` | `core::prefs::Section` | o nome; união de strings (`"nextItem"`) |
| `LaneMetric` | `core::history::Metric` | o nome; união de strings |
| `LaneReading` | `core::engine::profile::Reading` | só o nome |
| `Rival.metric`, `Rival.standing` | `core::engine::rival::Metric`, `Standing` | uniões inline no TS, sem nome próprio |
| `CoachNote.verdict` | `core::engine::coach::Verdict` | união inline (`"good" \| "bad"`) |
| `DraftState.currentAction` | `Option<String>` | o TS estreita para `"pick" \| "ban" \| null`; o Rust aceita qualquer texto |
| `EngineStatus`, `PoolPlan`, `LiveSnapshot` | `casca::EngineStatus`, `casca::PoolPlan`, `casca::live::LiveSnapshot` | os únicos tipos da ponte que moram na casca (10 §32) |

O espelho também copia fatos que não são tipos. Nenhum destes é conferido:

- **`POOL_SIZE` e `CORE_SIZE`** (`types.ts:333`) copiam `poolgap::POOL_SIZE` e
  `poolgap::CORE_SIZE`; **`BAN_POOL_MAX`** (`types.ts:455`) copia `pool::BAN_POOL_MAX`.
- **`OVERLAY_SECTIONS`** (`types.ts:411`) repete os rótulos e detalhes de
  `Section::label` e `Section::detail` (`core/src/prefs.rs:35`), texto por texto.
  Ninguém no Rust usa os rótulos fora do teste.
- **`ROLE_TO_LCU`** (`types.ts:473`) é o inverso de `Role::from_lcu`, e
  **`POSITION_LABEL`** (`types.ts:555`) repete `Role::short`, indexado pela string
  do LCU.
- **O overlay some com `prefs.hidden.length >= 6`** (`overlay.tsx:94`): o 6 é
  `Section::ALL.len()`, e a regra é `OverlayPrefs::anything_visible` reescrita.

## 5. Fontes que a casca chama

Tudo que sai do processo. Os comandos não chamam nada disto: só o watcher e as
duas tarefas de carga do `setup`. Os limites de cada fonte estão em
[00](00-contexto-e-limites.md#1-as-fontes).

### Descoberta do cliente (`lcu/credentials.rs`)

`credentials::discover` lê o `lockfile` do League e não toca a linha de comando de
nenhum processo. Procura, do mais barato para o mais caro:

1. as três pastas de `KNOWN_DIRS` (`credentials.rs:36`);
2. todo caminho `X:\…` citado em `LolClientInstalls.json` e no
   `product_settings.yaml` da Riot (`dirs_from_riot_config`), mais a subpasta
   `League of Legends` de cada um;
3. só se nenhum dos anteriores tiver lockfile, a pasta do executável de
   `LeagueClientUx.exe` pelo `sysinfo` (`dir_from_running_client`): o caminho da
   imagem, nunca a memória nem a linha de comando.

O primeiro lockfile que `parse_lockfile` aceita vence. A ordem existe por causa de
um falso positivo de antivírus, contado no cabeçalho do arquivo
(`credentials.rs:6`). Sem lockfile, o erro diz o que fazer, e é ele que vai em
`ConnStatus.detail`.

### LCU (`lcu/client.rs`)

`https://127.0.0.1:<porta>` (`Credentials::base_url`), Basic Auth `riot:<senha>`,
timeout de 5 s, verificação de TLS desligada (`client.rs:34`,
[P-8](../produto/perguntas-em-aberto.md)). Só `GET`
([07 §2](07-seguranca.md#o-cliente-do-league-só-leitura)). Resposta fora de 2xx
vira `Err("LCU respondeu … para …")`.

| Método | Caminho | Quando |
|---|---|---|
| `gameflow_phase` | `/lol-gameflow/v1/gameflow-phase` | toda volta do watcher; o erro derruba a conexão e faz redescobrir |
| `champ_select_session` | `/lol-champ-select/v1/session` | a cada 350 ms no champ select |
| `current_summoner` | `/lol-summoner/v1/current-summoner` | uma vez por conexão |
| `region` | `/riot/v1/region-locale` | uma vez por conexão, quando há puuid; erro vira `"BR"` (`watcher.rs:187`) |
| `grid_champions` | `/lol-champ-select/v1/all-grid-champions` | no champ select, até devolver uma lista não vazia |
| `queue_id` | `/lol-gameflow/v1/session` | junto com a grade |

### Live Client Data API (`live.rs`)

`https://127.0.0.1:2999/liveclientdata`, sem autenticação, timeout de 3 s,
verificação de TLS desligada (`live.rs:47`). Quatro `GET` em sequência a cada
volta de `InProgress` (`LiveClient::snapshot`, `live.rs:67`). Os caminhos estreitos
são escolha: `allgamedata` a Riot marca como só para teste.

| Caminho | Se falhar |
|---|---|
| `/playerlist` | a volta inteira falha, e nada é emitido |
| `/activeplayername` | idem |
| `/gamestats` | `gameTime` vira 0 |
| `/eventdata` | lista de eventos vazia: sem timers nem resultado |

### Riot Web API (`riot.rs`)

Só com `PRESAGE_RIOT_API_KEY` (§7). `https://{routing}.api.riotgames.com`, com o
host regional de `Region::from_lcu` (`americas` para BR). A chave vai no cabeçalho
`X-Riot-Token`, nunca na URL ([ADR-0004](../adr/0004-chave-da-riot-so-no-backend.md),
RNF-6). Timeout de 15 s.

- **429:** espera o `Retry-After` (10 s se não vier) e tenta de novo, até três vezes.
- **401 e 403:** `Err` dizendo que a chave expirou.
- **Outro erro:** `Err` sem a URL, porque o caminho carrega o puuid (`riot.rs:89`).

| Método | Caminho | Quando |
|---|---|---|
| `account_by_riot_id` | `/riot/account/v1/accounts/by-riot-id/{nome}/{tag}` | só com `PRESAGE_RIOT_ID`, depois de 120 s sem cliente (`profile.rs:29`) |
| `ranked_match_ids` | `/lol/match/v5/matches/by-puuid/{puuid}/ids?queue=420&start=0&count=60` | uma vez por abertura do app |
| `match_detail` | `/lol/match/v5/matches/{id}` | cada partida que `seen` ainda não tem, com 1,3 s antes de cada uma (`CALL_SPACING`, `riot.rs:24`) |

O histórico só se atualiza quando o app abre. Uma partida jogada com o app aberto
entra no perfil na abertura seguinte ([04 §7](04-perfil-de-rota.md#7-o-cache)).

### Data Dragon (`ddragon.rs`)

`https://ddragon.leagueoflegends.com`, sem autenticação, timeout de 20 s. Uma vez
por abertura, em sequência: campeões, itens, runas (`lib.rs:674`). Cada uma das
três pede `versions.json` de novo e usa o primeiro da lista. As URLs estão em §6.
Se o download falha, a carga cai para o cache em disco; sem cache, o índice fica
vazio e o motor, inativo (`EngineStatus.ready = false`).

A interface baixa o Data Dragon por conta própria para nomes e ícones
(`src/lib/ddragon.ts:23`): outra chamada a `versions.json` e a `champion.json`,
fora da casca.

## 6. Disco

Tudo em `%LOCALAPPDATA%\Presage` (`storage::cache_dir`, `storage.rs:5`). Sem
`LOCALAPPDATA`, gravar falha e ler devolve nada.

- **`write_json`** cria a pasta, escreve o JSON compacto inteiro em
  `nome.parcial` e só então renomeia para o nome final. Uma escrita que morre no
  meio deixa o arquivo anterior inteiro (A7, T-CASCA-21).
- **`read_json`** devolve `None` quando o arquivo falta, não abre ou não
  desserializa. Os três casos são o mesmo para quem chama, e nenhum deixa log.

O coletor grava do mesmo jeito (`tools/ingest/src/main.rs:189`). A forma de cada JSON e o que acontece quando ele falta ou está
corrompido estão em 10 §33.

| Arquivo | O quê | Escrito por | Lido por | Esquema |
|---|---|---|---|---|
| `pool.json` | a pool e a lista de bans | `set_pool` (`lib.rs:421`) | `setup` (`lib.rs:614`) | 10 §7 `Pool` |
| `overlay.json` | as seções escondidas do overlay | `set_overlay_prefs` (`lib.rs:441`) | `setup` (`lib.rs:644`) | 10 §8 `OverlayPrefs` |
| `draft.json` | "incluir quem a conta não pode escolher" | `set_draft_prefs` (`lib.rs:203`) | `setup` (`lib.rs:640`) | 10 §8 `DraftPrefs` |
| `base-de-partidas.json` | a base de partidas agregada | o coletor (`tools/ingest/src/main.rs:94`), nunca o app | `setup` (`lib.rs:622`), só se `Dataset::is_usable` | 10 §20 `Dataset` |
| `coleta-em-andamento.json` | o progresso do coletor | o coletor (`tools/ingest/src/main.rs:95`) | o coletor, com `--retomar` | `Estado`, privado do coletor ([05 §1](05-coletor.md#1-uso)) |
| `personal-v2.json` | o histórico pessoal | `profile::refresh` (`profile.rs:98`), quando entrou ou foi descartada alguma partida | `profile::refresh` (`profile.rs:47`) | 10 §19 `PersonalStats` |
| `gravacao.json` | a última partida, quadro a quadro | `recorder::finish` (`recorder.rs:53`) | ninguém no app | 10 §31 `Recording` |
| `ultima-partida.json` | a leitura da última partida | `recorder::finish` (`recorder.rs:56`) | `setup` (`lib.rs:636`) | 10 §31 `Report` |
| `champions.json` | o índice de campeões do último download | `load_champions` (`ddragon.rs:140`), a cada abertura com rede | `load_champions` (`ddragon.rs:147`), só quando o download falha | 10 §23 `ChampionIndex` |
| `items.json` | o índice de itens | `load_items` (`ddragon.rs:258`) | `load_items` (`ddragon.rs:265`), só quando o download falha | 10 §24 `ItemIndex` |
| `runas.json` | o índice de runas | `load_runes` (`ddragon.rs:203`) | `load_runes` (`ddragon.rs:210`), só quando o download falha | 10 §18 `RuneIndex` |

O cache do Data Dragon é regravado a cada abertura com rede, e só é lido quando a
rede falha. Nessa hora ele serve o último patch baixado, sem avisar que pode estar
velho ([00 §6](00-contexto-e-limites.md#6-limites-conhecidos)). O
`personal.json` antigo não é lido nem apagado: o app passou a usar
`personal-v2.json` ([L-9](09-licoes-do-prototipo.md)).

### Baixados do Data Dragon

Lidos só por `ddragon.rs`, nunca gravados como estão: a casca traduz e grava o
índice (tabela acima). `{v}` é o primeiro de `versions.json`.

| Arquivo | URL | Locale | Esquema |
|---|---|---|---|
| `versions.json` | `/api/versions.json` | — | `Vec<String>`, o mais novo primeiro |
| `champion.json` | `/cdn/{v}/data/{locale}/champion.json` | `pt_BR`, e `en_US` se falhar | `ChampionPayload` (privado, `ddragon.rs:21`) |
| `item.json` | `/cdn/{v}/data/{locale}/item.json` | `en_US` para os atributos ([ADR-0015](../adr/0015-atributos-de-item-pela-descricao.md)), `pt_BR` só para os nomes | `ItemPayload` (privado, `ddragon.rs:58`) |
| `runesReforged.json` | `/cdn/{v}/data/{locale}/runesReforged.json` | `pt_BR`, e `en_US` se falhar | `Vec<RuneTree>` (privado, `ddragon.rs:78`) |

Nenhum dos três tipos crus tem teste com o JSON de verdade do Data Dragon.

### Lidos da instalação da Riot

Só lidos, para achar o cliente (§5).

| Arquivo | Onde | Lido por |
|---|---|---|
| `lockfile` | na pasta do League | `credentials::discover`, via `parse_lockfile` |
| `LolClientInstalls.json` | `%LOCALAPPDATA%\Riot Games\Riot Client\Config\` | `dirs_from_riot_config` (`credentials.rs:85`), só como texto |
| `league_of_legends.live.product_settings.yaml` | `%ALLUSERSPROFILE%\Riot Games\Metadata\league_of_legends.live\` | `dirs_from_riot_config` (`credentials.rs:89`), só como texto |

## 7. Ambiente, atalhos e bandeja

### Variáveis de ambiente

| Variável | Lida em | Sem ela |
|---|---|---|
| `PRESAGE_RIOT_API_KEY` | `riot.rs:19`; `profile::refresh` e `RiotClient::from_env` | sem histórico pessoal nem perfil de rota; o resto funciona (RNF-5) |
| `PRESAGE_RIOT_ID` | `riot.rs:20`; `identity_from_env` (`profile.rs:123`), formato `nome#tag` | a conta só sai do cliente aberto |
| `PRESAGE_REGION` | `profile.rs:126`, só junto com `PRESAGE_RIOT_ID` | `"BR"` |
| `LOCALAPPDATA` | `storage.rs:5`; `credentials.rs:82` | nada vai para disco; um lugar a menos para achar o cliente |
| `ALLUSERSPROFILE` | `credentials.rs:88` | um lugar a menos para achar o cliente |
| `RUST_LOG` | `lib.rs:499` | filtro `presage_lib=debug,warn` |

A chave é lida do ambiente e de lugar nenhum mais
([00 §4](00-contexto-e-limites.md#enquanto-é-protótipo)).

O log sai na saída padrão (`tracing_subscriber::fmt`). No build de release o
Windows não abre console (`main.rs:2`), então os avisos, como "não consegui gravar
o cache", não aparecem em lugar nenhum.

### Overlay e atalhos

A janela `overlay` é declarada em `tauri.conf.json`: 300 × 340, transparente, sem
borda, sempre no topo, fora da barra de tarefas, sem foco e escondida. Ela carrega
`overlay.html` → `src/overlay.tsx`
([03 §10](03-motor-da-partida.md#10-overlay),
[ADR-0003](../adr/0003-overlay-como-janela-externa.md)).

| Atalho | Faz | Código |
|---|---|---|
| `Ctrl+Shift+P` | mostra ou esconde o overlay | `toggle_shortcut` (`overlay.rs:22`) → `toggle_visible` |
| `Ctrl+Shift+O` | alterna entre atravessável e clicável | `interact_shortcut` (`overlay.rs:26`) → `toggle_interactive` |

- **Atravessável ao nascer:** `overlay::prepare` (`lib.rs:658`) liga
  `set_ignore_cursor_events(true)` antes de qualquer exibição. O modo vive num
  `AtomicBool` (`overlay.rs:20`), que começa em atravessável a cada abertura.
- **Atalho ocupado:** os dois são globais, registrados em `lib.rs:662`. Se outro
  app já os tem, o registro falha com um aviso no log e o overlay fica sem aquele
  atalho.

### Bandeja

`bandeja` (`lib.rs:472`): ícone `presage` com dois itens.

- **"Mostrar o Presage"** (`mostrar`): desminimiza, mostra e foca a janela `main`.
- **"Sair"** (`sair`): `app.exit(0)`.

Se o ícone não sobe, o app segue sem ele.

### Iniciar com o Windows

`tauri-plugin-autostart` (`lib.rs:599`), lido e escrito por `get_autostart` e
`set_autostart`. No Windows a entrada fica na chave `Run` do `HKCU`
([07 §2](07-seguranca.md#iniciar-com-o-windows)). Desligado até o jogador ligar
(R-19).

### Atualizador

`montar(true)` registra `tauri-plugin-updater` e `tauri-plugin-process`
(`lib.rs:591`). O endpoint e a chave pública estão em `tauri.conf.json`, e o
manifesto é o `latest.json` da release
([06 §2](06-distribuicao-e-atualizador.md#2-o-atualizador),
[ADR-0005](../adr/0005-atualizador-assinado-com-endpoint-permanente.md)).

- **Quem chama:** a interface, direto no plugin: `check()` uma vez ao abrir e
  `relaunch()` depois de instalar (`Atualizacao.tsx`). Não há comando nosso.
- **Configuração recusada:** o Tauri derruba a inicialização quando a
  configuração do atualizador é inválida, por exemplo um endpoint `http://`.
  `run` (`lib.rs:515`) tenta de novo com `montar(false)`, sem os dois plugins,
  quando o erro cita `updater` (`erro_do_atualizador`, `lib.rs:539`). Qualquer
  outro erro de inicialização derruba o app com pânico.
- **Sem atualizador:** o `check()` da interface falha e é engolido
  (`Atualizacao.tsx:31`).

### Gravação da partida

`recorder.rs` guarda um quadro por volta do watcher em `InProgress`, perto de um
por segundo. `Recording::push` descarta o quadro cujo relógio não andou.

- **Teto:** 6000 quadros (`MAX_FRAMES`, `recorder.rs:23`), pouco mais de uma
  hora e meia. Depois dele, os quadros novos são descartados, e o resultado ainda
  é lido.
- **Piso:** `finish` só rende leitura quando `coach::worth_reviewing` diz que
  vale: 60 quadros e 5 minutos de partida no relógio. Abaixo disso, nada é gravado
  e nada é emitido: a última partida de verdade continua em `ultima-partida.json`,
  e a aba Pós-jogo não vem para a frente (A6, T-CASCA-19).
- **Resultado:** sai do primeiro evento `GameEnd` (`Win` ou `Lose`) e não é
  reescrito.
- **Fim da gravação:** ao sair de `InProgress`, `close_recording`
  (`watcher.rs:129`) fecha a gravação, grava os dois arquivos e emite
  `presage://review`. A gravação é esvaziada em todos os casos.

Nada disso sai da máquina (RNF-8).
