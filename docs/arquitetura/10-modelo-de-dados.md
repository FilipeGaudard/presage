# 10 — Modelo de dados

Cada tipo público do `presage-core` e cada tipo que a casca serializa para o
front: campos, unidades, origem e invariantes. Este doc é contrato
([ADR-0020](../adr/0020-o-doc-e-o-contrato-e-o-doccheck-prova.md)). O `doccheck`
compara cada bloco `rust` daqui com o código:

- campos `nome:tipo` de cada struct;
- variantes de cada enum;
- valor de cada constante;
- nos títulos com `↔`, os campos do doc, do Rust e da interface TS
  ([ADR-0022](../adr/0022-tipos-da-ponte-conferidos-campo-a-campo.md)).

Um tipo que muda no código e não muda aqui falha o commit.

O porquê de cada cálculo continua nos docs de tema: [02](02-motor-de-draft.md)
(draft), [03](03-motor-da-partida.md) (partida), [04](04-perfil-de-rota.md)
(perfil), [05](05-coletor.md) (coletor). Aqui fica o que existe e em que
unidade. As funções que leem e escrevem esses tipos estão no 11, com a mesma
numeração de seções.

## 1. Convenções

### Como ler um bloco

- **Módulo:** o título `##` diz o módulo, com o caminho entre crases. É de lá que
  o `doccheck` tira onde procurar.
- **Bloco:** traz os atributos `serde` como estão no código. O comentário de cada
  campo diz a **unidade** e a **origem**: de que API vem, quem preenche.
- **Ponte:** `Rust ↔ TS` no título quer dizer que o tipo cruza a ponte. O nome da
  direita é o da interface em `src/lib/types.ts`. A casca serializa em
  `camelCase` quase sempre; quando não, o atributo está no bloco.
- **Invariantes:** só o que o código garante, e onde.

### Unidades

A unidade vai no
comentário do campo, porque quase tudo é `f64` e o tipo não diz a escala.

| Grandeza | Escrita | Cuidado |
|---|---|---|
| Probabilidade | fração `0–1` | `winrate`, chance por rota |
| Desvio, delta, `±` | **pp** (pontos percentuais): `1.0` = 1 ponto | `delta` e `confidence` de `Scored` são pp; `winrate` é fração |
| Efeito em log-odds | log-odds | soma de componentes do ranking ([02 §1](02-motor-de-draft.md#1-rank)) |
| Tempo de jogo | segundos (`f64`), desde o início da partida | a Live Client manda `gameTime` em segundos |
| Ritmo de leitura e prazos da casca | ms | só na casca |
| Ouro | ouro do jogo | investido é soma de preço de item: exato |
| Campeão | `i64`, o `key` numérico do Data Dragon | o LCU usa o mesmo número |
| Item | `i64`, o id do Data Dragon | |
| Runa | chave textual do Data Dragon (`Conqueror`) | nunca o id numérico ([L-17](09-licoes-do-prototipo.md)) |
| Nível | `1–18` | a fórmula usa `n = nível − 1` |

### Confiança e pureza

Os números que chegam à tela carregam uma das quatro etiquetas do
[README](../README.md#confiança-do-número): exata, alta, fraca, regra. O
comentário do campo diz qual quando o campo vai para a tela. Todo tipo deste doc
fora do §32 é `core`: puro, sem I/O, sem relógio
([ADR-0001](../adr/0001-core-puro-separado-da-casca.md)).

### Mapa

| § | Módulo | Do que trata |
|---|---|---|
| 2–8 | `core::lockfile`, `draft`, `live`, `history`, `riot`, `pool`, `prefs` | as fronteiras com o LCU, a Live Client e a Riot, e o que o jogador guarda |
| 9–13 | `core::engine`, `context`, `roles`, `minimax`, `comp` | o draft |
| 14–18 | `team`, `wincon`, `report`, `poolgap`, `runes` | a prévia, a pool e as runas |
| 19–25 | `stats`, `dataset`, `profile`, `rival`, `champion`, `items`, `damage` | a estatística e a matemática do jogo |
| 26–31 | `build`, `lanes`, `gold`, `timeline`, `priority`, `coach` | a partida e o pós-jogo |
| 32 | `casca` | os tipos da casca que o front recebe |
| 33 | — | os arquivos em disco e o esquema de cada um |

## 2. `core::lockfile` — Credenciais do cliente

A porta e a senha do LCU, tiradas do texto do `lockfile` ou da linha de comando
do `LeagueClientUx.exe`. Só texto: achar o arquivo é da casca
(`src-tauri/src/lcu/credentials.rs`), que hoje só lê o `lockfile`.

### 2.1 `Credentials`

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Credentials {
    pub port: u16,     // porta HTTPS do LCU em 127.0.0.1, nova a cada execução do cliente. 3º campo do lockfile, ou `--app-port=`
    pub token: String, // senha do usuário `riot` na Basic Auth do LCU (`lcu/client.rs`). 4º campo do lockfile, ou `--remoting-auth-token=`
}
```

**Invariantes:**
- `port` é sempre um número que coube em `u16`: porta não numérica ou fora da
  faixa não vira `Credentials` (`parse_lockfile`, `parse_cmdline` devolvem
  `None`).
- `token` vindo de `parse_cmdline` nunca é vazio (`extract_arg` recusa valor
  vazio). Vindo de `parse_lockfile`, pode ser: `LeagueClient:1:2::https` passa.

## 3. `core::draft` — Champ select: o JSON do LCU e o draft do domínio

`Lcu*` espelha o JSON do cliente, que não é documentado e muda entre patches.
`DraftState` e `DraftSlot` são o que o motor e o front leem. `to_draft_state` é
a única fronteira entre os dois (`01 §2`). Todo tipo cru é `default`: campo que
falta vira zero, não erro.

### 3.1 `LcuSession`

```rust
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct LcuSession {
    pub actions: Vec<Vec<LcuAction>>, // `actions`: um vetor por turno, com as ações simultâneas do turno
    pub local_player_cell_id: i64,    // `localPlayerCellId`: a célula do jogador local
    pub my_team: Vec<LcuPlayer>,      // `myTeam`: o seu time, com hover
    pub their_team: Vec<LcuPlayer>,   // `theirTeam`: o time deles, com `championId` zerado (L-10)
    pub timer: LcuTimer,              // `timer`
    pub is_spectating: bool,          // `isSpectating`: lido e não usado
}
```

Vem de `GET /lol-champ-select/v1/session` (`src-tauri/src/lcu/client.rs:73`),
a 350 ms no champ select.

**Nomes da Riot:** nenhum teste os trava. Os testes de `draft.rs` montam a
sessão em Rust (`session()`), nunca de JSON: renomear `localPlayerCellId`,
`myTeam`, `theirTeam`, `actions` ou `timer` não quebra nada e esvazia o draft em
silêncio.

### 3.2 `LcuAction`

```rust
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct LcuAction {
    pub actor_cell_id: i64,   // `actorCellId`: a célula de quem age
    pub champion_id: i64,     // `championId`: id numérico do Data Dragon (`key`); 0 = nenhum
    pub completed: bool,      // `completed`: ação travada
    pub is_ally_action: bool, // `isAllyAction`: do seu time. Separa `my_bans` de `their_bans`
    pub is_in_progress: bool, // `isInProgress`: a vez é agora
    pub pick_turn: i64,       // `pickTurn`: lido e não usado
    #[serde(rename = "type")]
    pub action_type: String,  // `type`: `ban`, `pick` ou `ten_bans_reveal`
}
```

**Nomes da Riot:** nenhum teste os trava (o mesmo de `LcuSession`).

### 3.3 `LcuPlayer`

```rust
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct LcuPlayer {
    pub cell_id: i64,              // `cellId`: a célula no champ select
    pub champion_id: i64,          // `championId`: id do Data Dragon. Zerado no time deles; no seu, pode vir preenchido antes de travar (P-12)
    pub champion_pick_intent: i64, // `championPickIntent`: o hover. Só o do seu time chega
    pub assigned_position: String, // `assignedPosition`: `top`, `jungle`, `middle`, `bottom`, `utility`, ou vazio em fila sem rota
    pub spell1_id: i64,            // `spell1Id`: id do feitiço de invocador
    pub spell2_id: i64,            // `spell2Id`: id do feitiço de invocador
}
```

**Nomes da Riot:** nenhum teste os trava. `pick_inimigo_vem_da_acao_nao_do_jogador`
prova a regra do `championId` zerado, mas sobre struct montada à mão.

### 3.4 `LcuTimer`

```rust
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct LcuTimer {
    pub adjusted_time_left_in_phase: i64, // `adjustedTimeLeftInPhase`: ms que restam na fase
    pub total_time_in_phase: i64,         // `totalTimeInPhase`: ms da fase inteira
    pub phase: String,                    // `phase`: `PLANNING`, `BAN_PICK`, `FINALIZATION`
}
```

**Nomes da Riot:** nenhum teste os trava.

### 3.5 `LcuSummoner`

```rust
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct LcuSummoner {
    pub game_name: String,   // `gameName`: o nome do Riot ID
    pub tag_line: String,    // `tagLine`: a tag do Riot ID. Vazia, a casca usa só o nome (`watcher.rs:179`)
    pub summoner_level: i64, // `summonerLevel`: nível da conta. Lido e não usado
    pub puuid: String,       // `puuid`: o identificador permanente. Vazio, a casca não monta a identidade e o histórico pessoal não roda
}
```

Vem de `GET /lol-summoner/v1/current-summoner` (`lcu/client.rs:77`), uma vez por
conexão.

**Nomes da Riot:** nenhum teste os trava.

### 3.6 `DraftSlot` ↔ `DraftSlot`

```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct DraftSlot {
    pub cell_id: i64,     // `LcuPlayer::cell_id`
    pub position: String, // `assigned_position` como veio: vocabulário do LCU, minúsculo; vazio sem rota
    pub champion_id: i64, // id do Data Dragon: o do jogador se > 0, senão o da ação `pick` concluída; 0 = nada
    pub intent_id: i64,   // hover (`champion_pick_intent`); 0 no time deles
    pub spell1: i64,      // `spell1_id`
    pub spell2: i64,      // `spell2_id`
    pub is_local: bool,   // `cell_id == local_player_cell_id`
    pub is_acting: bool,  // a célula tem ação em andamento e não concluída
    pub locked: bool,     // há ação `pick` concluída com campeão > 0 para a célula
}
```

Preenche: `to_draft_state`; no simulador, o front, e `DraftState::normalized`
refaz o que depende dos times.

**Invariantes:**
- `locked` implica `champion_id > 0` (`to_draft_state`). O contrário não vale:
  se o cliente preenche o seu `championId` antes de travar
  ([P-12](../produto/perguntas-em-aberto.md)), sai `champion_id > 0` com
  `locked == false`. O comentário do código ("Campeão travado") diz menos do que
  o campo carrega.

### 3.7 `DraftState` ↔ `DraftState`

```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct DraftState {
    pub phase: String,                  // `LcuTimer::phase`: `PLANNING`, `BAN_PICK`, `FINALIZATION`
    pub time_left_ms: i64,              // ms, de `adjusted_time_left_in_phase`
    pub total_time_ms: i64,             // ms, de `total_time_in_phase`
    pub local_cell_id: i64,             // a célula do jogador local
    pub local_position: String,         // `position` do slot local; vazio sem slot local ou sem rota
    pub my_team: Vec<DraftSlot>,        // na ordem de `myTeam`
    pub their_team: Vec<DraftSlot>,     // na ordem de `theirTeam`
    pub my_bans: Vec<i64>,              // ids do Data Dragon, na ordem das ações
    pub their_bans: Vec<i64>,           // ids do Data Dragon, na ordem das ações
    pub current_action: Option<String>, // `type` da primeira ação em andamento: `pick`, `ban` (ou `ten_bans_reveal`); `None` fora de turno
    pub is_local_turn: bool,            // a célula local está agindo
    pub unavailable: Vec<i64>,          // banidos e escolhidos: ordenado, sem repetição, sem 0
}
```

Preenche: `to_draft_state`, a partir do LCU; o simulador manda um montado à mão,
que `normalized` refaz (R-8).

**Invariantes:**
- `unavailable` é ordenado, sem repetição e sem zero quando sai de
  `to_draft_state` ou de `normalized` (`unavailable_of`). Um `DraftState`
  desserializado e não normalizado não tem essa garantia.
- Hover de aliado **não** entra em `unavailable`. Já a sua escolha não travada
  entra, se o cliente a preencher (P-12). As duas correções são do
  `DraftContext` (`02 §0`).
- O TS tipa `currentAction` como `"pick" | "ban" | null`. O Rust passa adiante o
  `type` cru, e `ten_bans_reveal` chegaria ao front fora da união.

### 3.8 `ConnStatus` ↔ `ConnStatus`

```rust
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ConnStatus {
    pub connected: bool,          // a casca tem credenciais e o LCU respondeu à fase
    pub phase: String,            // `/lol-gameflow/v1/gameflow-phase` (`ChampSelect`, `InProgress`…), ou `Disconnected`
    pub summoner: Option<String>, // `nome#tag` da conta logada, de `LcuSummoner`
    pub detail: Option<String>,   // a mensagem de erro quando desconectado
}
```

Preenche: a casca (`src-tauri/src/watcher.rs`), a cada volta do polling
(`presage://status`).

### 3.9 `LcuGridChampion`

```rust
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct LcuGridChampion {
    pub id: i64,              // `id`: id do Data Dragon
    pub owned: bool,          // `owned`: a conta tem
    pub free_to_play: bool,   // `freeToPlay`: está na rotação grátis
    #[serde(rename = "xboxGPReward")]
    pub xbox_gp_reward: bool, // `xboxGPReward`: liberado pelo Game Pass
}
```

Vem de `GET /lol-champ-select/v1/all-grid-champions` (`lcu/client.rs:83`), que
só responde durante o champ select.

**Nomes da Riot:** `na_ranqueada_a_rotacao_gratis_nao_vale`,
`fora_da_ranqueada_a_rotacao_vale` e `fila_desconhecida_fica_do_lado_seguro`
leem a grade de JSON (`grade()`) com `id`, `owned`, `freeToPlay` e
`xboxGPReward`. O JSON foi escrito à mão: trava o nome que o código lê, não o
que o cliente manda.

### 3.10 `LcuGameflowSession`

```rust
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct LcuGameflowSession {
    pub game_data: LcuGameData, // `gameData`
}
```

Vem de `GET /lol-gameflow/v1/session` (`lcu/client.rs:89`), só para a fila.

**Nomes da Riot:** `a_fila_sai_da_sessao_do_fluxo_de_jogo` lê
`{"gameData": {"queue": {"id": 420}}}`, com campos a mais que precisam ser
ignorados.

### 3.11 `LcuGameData`

```rust
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct LcuGameData {
    pub queue: LcuQueue, // `queue`
}
```

**Nomes da Riot:** `a_fila_sai_da_sessao_do_fluxo_de_jogo`.

### 3.12 `LcuQueue`

```rust
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct LcuQueue {
    pub id: i64, // `id`: id de fila da Riot (420 solo/duo, 440 flex, 400 normal com escolha)
}
```

**Nomes da Riot:** `a_fila_sai_da_sessao_do_fluxo_de_jogo`.

### 3.13 `RANKED_QUEUES`

```rust
pub const RANKED_QUEUES: [i64; 2] = [420, 440]; // ids de fila da Riot: 420 solo/duo, 440 flex. Mecânica do jogo: nelas a rotação grátis não vale
```

## 4. `core::live` — A partida ao vivo: o JSON da Live Client e o jogo do domínio

`Raw*` espelha `/liveclientdata/playerlist` (`127.0.0.1:2999`, sem
autenticação). `LiveGame` e `LivePlayer` são o que o motor da partida, a gravação
e o front leem. `to_live_game` é a fronteira (`01 §2`).

### 4.1 `RawPlayer`

```rust
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct RawPlayer {
    pub champion_name: String,     // `championName`: nome traduzido. Lido e não usado
    pub raw_champion_name: String, // `rawChampionName`: `game_character_displayname_<chave>`, o mesmo em todo idioma
    pub summoner_name: String,     // `summonerName`: nome de invocador; casa com `/activeplayername` em cliente antigo
    pub riot_id: String,           // `riotId`: `nome#tag`
    pub team: String,              // `team`: `ORDER` ou `CHAOS`
    pub position: String,          // `position`: `TOP`, `JUNGLE`, `MIDDLE`, `BOTTOM`, `UTILITY`, ou vazio
    pub level: u8,                 // `level`: nível 1–18
    pub is_dead: bool,             // `isDead`
    pub is_bot: bool,              // `isBot`: bot sai em `to_live_game`
    pub respawn_timer: f64,        // `respawnTimer`: segundos até renascer
    pub items: Vec<RawItem>,       // `items`: o inventário
    pub scores: RawScores,         // `scores`
}
```

Vem de `GET /playerlist` (`src-tauri/src/live.rs:68`), a cada segundo de partida.

**Nomes da Riot:** nenhum teste os trava. Os testes de `live.rs` montam
`RawPlayer` em Rust (`player()`), nunca de JSON.

### 4.2 `RawItem`

```rust
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct RawItem {
    #[serde(rename = "itemID")]
    pub item_id: i64, // `itemID`: id do item no Data Dragon
    pub price: f64,   // `price`: ouro, o preço do item. Somado vira `gold_invested`
    pub slot: i64,    // `slot`: posição no inventário. Lido e não usado
}
```

**Nomes da Riot:** nenhum teste os trava. `itemID` com `ID` maiúsculo é o caso
em que o `camelCase` não basta, e ele também não tem teste.

### 4.3 `RawScores` ↔ `LiveScores`

```rust
#[derive(Debug, Clone, Deserialize, Default, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct RawScores {
    pub kills: i64,       // `kills`: contagem
    pub deaths: i64,      // `deaths`: contagem
    pub assists: i64,     // `assists`: contagem
    pub creep_score: i64, // `creepScore`: tropas e monstros abatidos, contagem
    pub ward_score: f64,  // `wardScore`: placar de visão
}
```

É cru e domínio ao mesmo tempo: `LivePlayer` carrega o bloco como veio, e o
front lê os mesmos nomes da Riot.

**Nomes da Riot:** nenhum teste os trava. Um nome que a Riot mudar zera o campo
no motor e no front de uma vez.

### 4.4 `LivePlayer` ↔ `LivePlayer`

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LivePlayer {
    pub champion_key: String, // chave textual do Data Dragon (`Annie`, `LeeSin`), de `raw_champion_name` sem o prefixo
    pub champion_id: i64,     // id numérico do Data Dragon; 0 quando a chave não está no `ChampionIndex`
    pub display_name: String, // `riot_id`; `summoner_name` quando ele vem vazio
    pub team: String,         // `ORDER` ou `CHAOS`
    pub position: String,     // como veio da Live Client
    pub level: u8,            // nível 1–18
    pub items: Vec<i64>,      // ids de item, na ordem da resposta
    pub gold_invested: f64,   // ouro: soma de `RawItem::price` do inventário. Exata
    pub is_dead: bool,        // `RawPlayer::is_dead`
    pub respawn_timer: f64,   // segundos
    pub is_local: bool,       // `riot_id` ou `summoner_name` igual a `/activeplayername`
    pub scores: RawScores,    // o bloco cru, copiado
}
```

`Deserialize` porque a gravação (`gravacao.json`) guarda quadros de `LiveGame`
(`engine::coach::Recording`).

### 4.5 `LiveGame` ↔ `LiveGame`

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LiveGame {
    pub game_time: f64,           // segundos de partida, de `/gamestats` (`gameTime`)
    pub players: Vec<LivePlayer>, // os humanos, na ordem de `/playerlist`; bots fora
}
```

Preenche: `to_live_game`; a casca o publica em `presage://live`.

**Invariantes:**
- Nenhum bot em `players` (`to_live_game`).
- No máximo um jogador local não é garantido: `me()` pega o primeiro
  `is_local`. Com `active` vazio, nenhum é local.

## 5. `core::history` — A partida da match-v5 e o oponente direto

`Match*` e `Challenges` espelham o JSON da match-v5. `PlayedGame` e `LaneGame`
são o que o histórico pessoal guarda. `played` é a fronteira (`01 §2`). Todo
tipo cru é `default`: campo que falta derruba a métrica, não a partida.

### 5.1 `MatchDto`

```rust
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MatchDto {
    pub info: MatchInfo, // `info`; `metadata` é ignorado
}
```

Vem de `GET /lol/match/v5/matches/{id}` (`src-tauri/src/riot.rs:115`), pela
chave da Riot.

**Nomes da Riot:** `le_os_nomes_de_campo_da_riot`, com JSON escrito à mão
([P-5](../produto/perguntas-em-aberto.md)).

### 5.2 `MatchInfo`

```rust
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MatchInfo {
    pub game_duration: i64,                  // `gameDuration`: segundos. Abaixo de 300 é `Skip::Curta`
    pub game_mode: String,                   // `gameMode`: `CLASSIC` na Fenda
    pub end_of_game_result: String,          // `endOfGameResult`: `GameComplete` numa partida normal; vazio em resposta antiga
    pub participants: Vec<MatchParticipant>, // `participants`: os dez
}
```

**Nomes da Riot:** `le_os_nomes_de_campo_da_riot`.

### 5.3 `MatchParticipant`

```rust
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MatchParticipant {
    pub puuid: String,                       // `puuid`
    pub champion_id: i64,                    // `championId`: id do Data Dragon
    pub team_id: i64,                        // `teamId`: 100 ou 200
    pub team_position: String,               // `teamPosition`: `TOP`, `JUNGLE`, `MIDDLE`, `BOTTOM`, `UTILITY`, ou vazio
    pub win: bool,                           // `win`
    pub game_ended_in_early_surrender: bool, // `gameEndedInEarlySurrender`: remake
    pub deaths: u32,                         // `deaths`: contagem
    pub challenges: Option<Challenges>,      // `challenges`: ausente em partida antiga e em alguns modos
}
```

**Nomes da Riot:** `le_os_nomes_de_campo_da_riot`.

### 5.4 `Challenges`

```rust
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Challenges {
    #[serde(rename = "laneMinionsFirst10Minutes")]
    pub lane_minions_first_10: Option<f64>, // tropas de rota nos 10 primeiros minutos
    #[serde(rename = "jungleCsBefore10Minutes")]
    pub jungle_cs_before_10: Option<f64>,   // campos da selva antes dos 10 minutos
    #[serde(rename = "killParticipation")]
    pub kill_participation: Option<f64>,    // fração 0–1 dos abates do time
    #[serde(rename = "teamDamagePercentage")]
    pub team_damage_share: Option<f64>,     // fração 0–1 do dano do time em campeões
    #[serde(rename = "visionScorePerMinute")]
    pub vision_per_minute: Option<f64>,     // placar de visão por minuto
}
```

**Nomes da Riot:** `le_os_nomes_de_campo_da_riot` trava quatro dos cinco.
`jungleCsBefore10Minutes` não está no JSON do teste: `cacador_compara_a_selva`
monta o valor em Rust, e um nome errado zeraria o `Cs10` do caçador sem nenhum
teste falhar.

### 5.5 `Metric` ↔ `LaneMetric`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Metric {
    Cs10,              // tropas aos 10 (`laneMinionsFirst10Minutes`); campos da selva para o caçador; nenhum valor para suporte
    KillParticipation, // pp: `killParticipation` × 100
    DamageShare,       // pp: `teamDamagePercentage` × 100
    VisionPerMinute,   // placar de visão por minuto (`visionScorePerMinute`)
    Deaths,            // mortes na partida, contagem. A única em que menos é melhor
}

impl Metric {
    pub const ALL: [Metric; 5] = [
        Metric::Cs10,
        Metric::KillParticipation,
        Metric::DamageShare,
        Metric::VisionPerMinute,
        Metric::Deaths,
    ]; // as cinco, na ordem em que `played` mede. Escolhido à mão
}
```

O front recebe a métrica dentro do perfil de rota (`LaneReading::metric`,
`engine::profile`).

### 5.6 `PlayedGame`

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct PlayedGame {
    pub champion: i64,          // id do Data Dragon do campeão do jogador
    pub won: bool,              // `MatchParticipant::win` do jogador
    pub lane: Option<LaneGame>, // `None`: posição em branco ou dois na mesma rota do outro lado
}
```

Preenche: `played`. Consome: `PersonalStats::add_played`
(`core/src/engine/stats.rs:220`).

### 5.7 `LaneGame`

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LaneGame {
    pub champion: i64,                // id do Data Dragon, o seu
    pub opponent: i64,                // id do Data Dragon do oponente direto
    pub role: Role,                   // de `team_position` por `Role::from_lcu`
    pub won: bool,                    // o seu resultado
    pub diffs: BTreeMap<Metric, f64>, // você menos ele, na unidade da métrica; só as que os dois lados têm
}
```

Vai para o disco dentro do histórico pessoal (`PersonalStats::lanes`,
`personal-v2.json`), não para o front.

**Invariantes:**
- `role` nunca é `Unknown` (`lane_game` devolve `None` antes).
- `diffs` sempre tem `Deaths`: `deaths` não é opcional na match-v5
  (`sem_challenges_sobra_so_mortes`).
- `diffs` nunca tem `Cs10` com `role == Support`.

### 5.8 `Skip`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Skip {
    Ausente,   // o puuid não está entre os participantes
    Modo,      // `gameMode` não é `CLASSIC`
    Terminada, // `endOfGameResult` não vazio e diferente de `GameComplete`
    Remake,    // algum participante com `gameEndedInEarlySurrender`
    Curta,     // `gameDuration` abaixo de 300 s (`MIN_DURATION`, privada)
}
```

O coletor tem o próprio `Skip`, com nove motivos
(`tools/ingest/src/aggregate.rs:82`). `Modo`, `Terminada`, `Remake` e `Curta`
são a mesma regra escrita duas vezes, com o mesmo `300` em cada arquivo
([05 §5](05-coletor.md#5-a-duplicação-com-historyrs)).

## 6. `core::riot` — Endereçamento da Riot Web API

Que host atende cada conta, e o Riot ID em texto de URL. As chamadas ficam na
casca (`src-tauri/src/riot.rs`) e no coletor.

### 6.1 `Region`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Region {
    pub platform: &'static str, // host de plataforma (`br1`, `euw1`, `kr`): `league-v4`, summoner
    pub routing: &'static str,  // host regional (`americas`, `europe`, `asia`, `sea`): `account-v1`, `match-v5`
}
```

**Invariantes:**
- `from_platform` só monta pares da tabela `PLATFORMS` (privada, 17 hosts).
- `from_lcu` monta os seus de uma tabela própria, que não tem `me1`. São duas
  listas do mesmo fato.

## 7. `core::pool` — A pool do jogador

Os campeões do jogador por rota e a sua lista de bans. A casca guarda em
`pool.json` o que o front manda (`set_pool`).

### 7.1 `Tier` ↔ `Tier`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Tier {
    Core,        // os escolhidos pelo jogador, três por rota (ADR-0016). `weight` 0
    Situational, // os que cobrem o que os core não cobrem. `weight` 1
    Extra,       // qualquer outro que o jogador queira na pool. `weight` 2
}
```

### 7.2 `PoolEntry` ↔ `PoolEntry`

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PoolEntry {
    pub champion_id: i64,   // id do Data Dragon
    pub role: Option<Role>, // `None` vale em qualquer rota
    pub tier: Tier,         // escolhido pelo jogador
}
```

### 7.3 `BAN_POOL_MAX`

```rust
pub const BAN_POOL_MAX: usize = 5; // contagem: quantos bans cabem na sua lista, um por jogador do time. Mecânica do jogo
```

Nenhum código Rust lê esta constante. O teto vale só no front, que tem a própria
cópia (`BAN_POOL_MAX` em `src/lib/types.ts`).

### 7.4 `Pool` ↔ `Pool`

```rust
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pool {
    pub entries: Vec<PoolEntry>, // em ordem de inserção; um campeão pode ter uma entrada por rota
    #[serde(default)]
    pub bans: Vec<i64>,          // ids do Data Dragon, em ordem de preferência; arquivo antigo carrega vazio
}
```

**Invariantes:**
- `upsert` mantém o par (`champion_id`, `role`) único. A pool que chega do front
  não passa por `upsert`: `set_pool` grava como veio.
- O tamanho de `bans` não tem teto no Rust.

## 8. `core::prefs` — O que o jogador quer ver

As escolhas do overlay e do draft, guardadas pela casca em `overlay.json` e
`draft.json`.

### 8.1 `Section` ↔ `OverlaySection`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Section {
    Gold,       // ouro em itens dos dois times
    Damage,     // o dano que vem contra você
    Objectives, // dragão, barão, arauto e grubs
    Priority,   // prioridade de rota
    Skirmish,   // a escaramuça que importa no minuto
    NextItem,   // as duas opções de próximo item
}

impl Section {
    pub const ALL: [Section; 6] = [
        Section::Gold,
        Section::Damage,
        Section::Objectives,
        Section::Priority,
        Section::Skirmish,
        Section::NextItem,
    ]; // todas, na ordem em que aparecem no overlay. Escolhido à mão
}
```

O rótulo e a explicação de cada seção existem duas vezes: `Section::label` e
`Section::detail` no Rust, `OVERLAY_SECTIONS` em `src/lib/types.ts`. A tela usa
a do TS.

### 8.2 `OverlayPrefs` ↔ `OverlayPrefs`

```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct OverlayPrefs {
    pub hidden: Vec<Section>, // as seções desligadas, não as ligadas: seção nova nasce visível
}
```

**Invariantes:**
- `set` nunca deixa a mesma seção duas vezes em `hidden`.
- Vazio quer dizer tudo visível. Um nome de seção que esta versão não conhece
  faz o arquivo inteiro falhar na leitura, e a casca volta ao padrão.

### 8.3 `DraftPrefs` ↔ `DraftPrefs`

```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct DraftPrefs {
    pub show_all_champions: bool, // ligado: recomenda também quem a conta não pode escolher. Padrão desligado (R-5)
}
```

## 9. `core::engine` — Ranking do draft

`engine/mod.rs` transforma cada candidato do `DraftContext` numa probabilidade
com `±` e motivos ([02 §1](02-motor-de-draft.md#1-rank)). O único tipo público é
a saída. Os pesos e espalhamentos que a produzem são constantes privadas do
módulo, e os valores estão em [02 §2](02-motor-de-draft.md#força-e-desvio) e
[02 §4](02-motor-de-draft.md#4-fit-de-composição).

### 9.1 `Scored` ↔ `ScoredChampion`

```rust
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Scored {
    pub champion_id: i64,     // id numérico do Data Dragon (`key`), um de `ctx.candidates`
    pub score: f64,           // `winrate` em porcentagem (0–100), arredondada a 0,1
    pub delta: f64,           // pp contra uma partida neutra: `(winrate − 0,5) × 100`, a 0,1. É o número embaixo do retrato
    pub winrate: f64,         // fração 0–1: σ(z) da soma em log-odds, cortada em [0,25; 0,75], sem arredondar. A ordem usa este
    pub confidence: f64,      // pp: meia largura do intervalo de 95%, `1,96 × √variância × 100`, a 0,1. É o `±`
    pub sample: u32,          // contagem: partidas do candidato na sua rota segundo `StatsStore::base`; 0 sem a base
    pub in_pool: bool,        // o id está em `ctx.pool`, a pool da rota que a casca passou
    pub reasons: Vec<String>, // até três motivos em pt-BR, na ordem de `build_reasons` (02 §5)
}
```

O front recebe a lista de `rank_draft` e mostra `score`, `confidence`, `sample`,
`delta` e `reasons` (`src/components/RecommendationPanel.tsx`).

**Invariantes:**
- `winrate` está em [0,25; 0,75] (`engine/mod.rs:262`); daí `score` em [25; 75]
  e `delta` em [−25; +25].
- Sem amostra, cada componente estatístico entra com exatamente zero, e `winrate`
  sai só da composição ([ADR-0007](../adr/0007-sem-dado-o-componente-vale-zero.md)).
- `confidence > 0` sempre: a variância soma a da base e a da composição mesmo sem
  amostra. No draft dos testes, sem base, dá 11,2pp ([02 §2](02-motor-de-draft.md#incerteza)).
- `sample` só cresce com a base global. O histórico pessoal não o preenche
  (`historico_pessoal_nao_preenche_matchup`).
- `reasons` tem no máximo três itens (`MAX_REASONS`, `engine/mod.rs:377`). Fica
  vazio só com a base ligada e nenhum motivo; sem a base, o motivo mínimo é
  "Sem base estatística ainda — ordenado só por composição".
- `rank_with` devolve a lista ordenada por `winrate` decrescente, depois
  `in_pool` primeiro, depois `champion_id` crescente (`engine/mod.rs:148-154`).

## 10. `core::engine::context` — Contexto do draft

A leitura única do champ select, que ranking, minimax e análise de rota
compartilham ([ADR-0017](../adr/0017-uma-leitura-do-champ-select.md),
[02 §0](02-motor-de-draft.md#0-o-contexto-do-draft)). Ela sai do `DraftState` (o do
LCU ou o do simulador), do `ChampionIndex`, da loja, da pool da rota e da
`Leitura`. A casca monta um contexto novo a cada comando do draft
(`draft_context`, `src-tauri/src/lib.rs:119`).

### 10.1 `PREGAME_LEVEL`

```rust
pub const PREGAME_LEVEL: u8 = 6; // escolhido à mão: nível 1–18 em que a prévia compara as rotas antes da partida
```

Quem lê é `report::from_context`, pelo `DraftContext::lineups`.

### 10.2 `Placed`

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Placed {
    pub champion_id: i64, // id do Data Dragon: o travado (`champion_id` do slot) ou, para aliado, o hover (`intent_id`)
    pub role: Role,       // a do cliente (`position`), a marcada, a inferida, ou `Unknown` quando sobrou sem rota livre
    pub picked: bool,     // o slot tem `champion_id > 0`; falso para hover de aliado. Inimigo é sempre verdadeiro
    pub informed: bool,   // a rota veio do cliente ou da sua marca; falso quando saiu da inferência
}
```

**Invariantes:**
- `champion_id > 0`: slot sem campeão nem hover não vira `Placed`.
- `role == Unknown` implica `informed == false`. Uma rota inferida tem
  `role != Unknown` e `informed == false`.

### 10.3 `Leitura`

```rust
#[derive(Debug, Clone, Default)]
pub struct Leitura {
    pub enemy_roles: HashMap<i64, Role>, // id do Data Dragon → rota marcada à mão (`set_enemy_role`); a casca apaga ao fim do champ select
    pub owned: Option<HashSet<i64>>,     // ids que a conta pode escolher nesta fila (`draft::selectable`, grade do LCU); `None` no simulador e com "incluir quem você não tem"
}
```

**Invariantes:**
- `owned` vazio vale como `None`, e então ninguém sai (`context.rs:257`).
- Uma marca não tem efeito quando é `Role::Unknown`, quando o inimigo tem posição
  do cliente, ou quando a rota já está tomada. A rota tomada conta as posições do
  cliente e as marcas anteriores, na ordem dos slots (`context.rs:229-238`).

### 10.4 `DraftContext`

```rust
#[derive(Debug, Clone)]
pub struct DraftContext {
    pub my_role: Role,             // a sua rota, já resolvida (`resolve_role`); `Unknown` não filtra candidatos
    pub me: Option<i64>,           // id do Data Dragon: o seu travado, o seu hover, ou o candidato de `with_candidate`; `None` sem nenhum
    pub allies: Vec<Placed>,       // os outros do seu time com campeão ou hover, na ordem dos slots
    pub enemies: Vec<Placed>,      // os travados do outro time, na ordem dos slots, com rota informada, marcada ou inferida
    pub roles_assumed: bool,       // alguma rota, aliada ou inimiga, saiu da inferência; a interface avisa
    pub unavailable: HashSet<i64>, // ids: bans dos dois lados, aliados (hover inclusive), inimigos, e o seu só depois de travar
    pub pool: HashSet<i64>,        // ids da pool que a casca passou (`Pool::ids_for_role`)
    pub candidates: Vec<i64>,      // ids em ordem crescente: fora de `unavailable`, dentro de `owned`, e que jogam `my_role` ou estão na pool
}
```

Não cruza a ponte: a casca o monta e o consome no mesmo comando.

**Invariantes:**
- `candidates` e `unavailable` não se cruzam, e `candidates` vem em ordem crescente
  (`ChampionIndex::ids` ordena, `context.rs:258-269`).
- Com `owned` não vazio, todo candidato está em `owned`: a pool não passa por cima
  da conta (`so_entra_quem_a_sua_conta_pode_escolher`).
- `me` só entra em `unavailable` com o slot local `locked`
  ([P-12](../produto/perguntas-em-aberto.md)).
- Entre os inimigos, uma rota marcada ou inferida nunca repete uma rota já
  ocupada. A posição do cliente é aceita como vem.
- `Role::Unknown` sobra em `allies` ou `enemies` só quando há mais gente sem rota
  que rotas livres.
- `with_candidate` muda só `me`.

## 11. `core::engine::roles` — Quem joga a rota

Uma regra só, para o draft, a pool e a resposta do minimax não discordarem sobre o
mesmo campeão ([L-4](09-licoes-do-prototipo.md),
[02 §7](02-motor-de-draft.md#quem-joga-a-rota)). O dado vem de três lugares: a
base de partidas (`StatsStore::base`), as classes do Data Dragon
(`Champion::tags`) e o histórico do próprio jogador (`StatsStore::played_in`).

### 11.1 `ROLES`

```rust
pub const ROLES: [Role; 5] = [Role::Top, Role::Jungle, Role::Mid, Role::Bot, Role::Support]; // mecânica do jogo: as cinco rotas, sem `Unknown`. A ordem é escolhida à mão
```

**Invariantes:**
- `role_share` devolve as frações nesta ordem.
- A ordem também desempata a inferência de rota: a busca percorre as rotas livres
  nesta ordem e fica com a primeira atribuição melhor (`context.rs:158`).

## 12. `core::engine::minimax` — Draft com resposta e bans

A saída do draft com resposta e da sugestão de ban
([02 §6](02-motor-de-draft.md#6-draft-com-resposta)). O valor de cada número é
`team_value`, só composição do Data Dragon, com pesos próprios do módulo
([P-11](../produto/perguntas-em-aberto.md)). Com ou sem base, a loja não entra no
valor.

### 12.1 `MinimaxPick` ↔ `MinimaxPick`

```rust
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MinimaxPick {
    pub champion_id: i64,           // id do Data Dragon, um de `ctx.candidates`
    pub greedy: f64,                // fração 0–1: `team_value` do seu time com ele contra o deles de agora, a 3 casas
    pub after_response: f64,        // fração 0–1: o mesmo depois da resposta deles que mais o derruba, a 3 casas
    pub exposure: f64,              // fração 0–1: `greedy − after_response` antes de arredondar, a 3 casas. A tela mostra ×100 como pp
    pub best_response: Option<i64>, // id do Data Dragon da resposta que mais derruba; `None` quando nenhuma derruba (time deles fechado ou nada piora)
    pub in_pool: bool,              // o id está em `ctx.pool`
}
```

**Invariantes:**
- `exposure ≥ 0`: o pior valor começa no guloso e só desce (`minimax.rs:166-177`).
- `best_response` nunca é o próprio candidato nem um id de `ctx.unavailable`.
- `greedy` e `after_response` ficam perto de 0,5. Cada lado perde no máximo
  `W_BALANCE + W_FRONTLINE`, e o corte em [0,05; 0,95] de `team_value` não chega a
  agir.

### 12.2 `BanSuggestion` ↔ `BanSuggestion`

```rust
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BanSuggestion {
    pub champion_id: i64, // id do Data Dragon
    pub threat: f64,      // fração 0–1: quanto o seu `team_value` cai se eles o pegarem, a 3 casas; 0 quando vem da sua lista
    pub reason: String,   // texto fixo: "Completa o que falta na composição deles" ou "Da sua lista de bans"
    #[serde(default)]
    pub from_pool: bool,  // veio de `Pool::bans`, não da conta da composição
}
```

**Invariantes:**
- Em `bans_with_pool`, os itens com `from_pool` vêm antes dos outros e na ordem da
  sua lista, e nenhum id se repete.
- Sem campeão do outro lado, nenhum item sai da conta da composição
  ([ADR-0013](../adr/0013-ban-nao-personalizado-enquanto-aditivo.md)).
- O Rust sempre serializa `fromPool`. No TS o campo é opcional (`fromPool?`).

## 13. `core::engine::comp` — Perfil de composição

O retrato de um time em dano físico, frontline e alcance, lido das classes e do
bloco `info` do Data Dragon ([02 §4](02-motor-de-draft.md#4-fit-de-composição)).
Quem lê: `rank()`, `team_value` e a condição de vitória (`wincon.rs:81`).

### 13.1 `TeamProfile`

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TeamProfile {
    pub physical_share: f64, // fração 0–1: média de `Champion::physical_share` (razão `attack`/`magic` do `info` e prior de classe); 0,5 com time vazio
    pub frontline: usize,    // contagem: quantos são `is_frontline` (classe Tank ou `info.defense ≥ 7`)
    pub ranged: usize,       // contagem: quantos são `is_ranged` (`attackrange ≥ 500`)
    pub known: usize,        // contagem: ids que o `ChampionIndex` conhece; os outros ficam de fora
}
```

**Invariantes:**
- `frontline ≤ known` e `ranged ≤ known`.
- Com `known == 0`, `physical_share == 0,5` e as contagens são zero
  (`time_vazio_e_neutro`).
- A completude que pondera os sinais é `min(known / 4, 1)`, método privado
  `completeness` (`comp.rs:14-16`). `team_value` usa outra, `min(known / 5, 1)`
  (`minimax.rs:76`).

## 14. `core::engine::team` — A chance do time

Os dois lados do draft como assentos (`Seat`) e a chance do seu time contra o
deles, somada em log-odds sobre os dez campeões. Todo número sai da base de
partidas pelo `StatsStore`; sem ela não há `TeamEstimate`. O desenho está em
[02 §1](02-motor-de-draft.md#a-chance-do-time) e no
[ADR-0019](../adr/0019-contra-em-qualquer-rota-e-chance-do-time.md).

### 14.1 `Seat`

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Seat {
    pub champion_id: i64, // id numérico do Data Dragon (`key`), lido do `DraftContext`
    pub role: Role,       // rota do assento: do cliente, marcada à mão ou inferida no `DraftContext`
    pub certainty: f64,   // fração 0–1: 1 com a rota informada; `lane_share` (fração do encaixe) quando inferida
}
```

Não cruza a ponte. `team::seats` monta, `team::estimate` consome.

**Invariantes:**
- Em `seats`, `certainty` é `1.0` ou `lane_share`, que divide `role_fit` (≥ 0)
  pela soma das cinco rotas: fica em [0, 1] (`mod.rs`, `lane_share`).
- `role` nunca é `Role::Unknown`: `seats` descarta quem não tem rota
  (`team.rs:81`, `team.rs:90`).
- O seu assento (`ctx.me`) entra sempre com `certainty = 1`, venha a sua rota do
  cliente, da tela ou da pool (`team.rs:85`).

### 14.2 `TeamTerm` ↔ `TeamTerm`

```rust
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamTerm {
    pub label: String, // texto pronto: "A na rota", "A × X na rota", "A contra X", "A com B"; nome do Data Dragon ou "#id"
    pub shift: f64,    // pp, uma casa: (σ(dz) − 0,5) × 100 da parcela sozinha; positivo a favor de `ours`
    pub games: u32,    // contagem de partidas da tabela da parcela: `base`, `matchup`, `versus` ou `synergy`
}
```

**Invariantes:**
- `games > 0`: parcela sem amostra soma na conta mas não vira `TeamTerm`
  (`team.rs:115`).
- O filtro é `|dz| > 1e-9` e o arredondamento a 0,1pp vem depois: `shift` pode
  sair `0.0`.

### 14.3 `TeamEstimate` ↔ `TeamEstimate`

```rust
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamEstimate {
    pub win: f64,             // fração 0–1: σ(z) do seu time, presa em [0,35; 0,65] (`FLOOR`, `CEILING`, escolhidos à mão)
    pub confidence: f64,      // pp, uma casa: 1,96 × √(soma das variâncias das parcelas) × 100 — o `±`
    pub terms: Vec<TeamTerm>, // até 4 (`MAX_TERMS`), do maior |shift| para o menor
}
```

**Invariantes:**
- `win` ∈ [0,35; 0,65] (`team.rs:208`). O piso e o teto são simétricos em torno
  de 0,5, então numa base coerente (cada registro direcional com o seu
  complemento) `estimate(a, b).win + estimate(b, a).win = 1`.
- `terms.len() ≤ 4` (`team.rs:210`).
- Só existe com `store.has_global()` e com alguém dos dois lados
  (`team.rs:104`).

## 15. `core::engine::wincon` — Condição de vitória

O que a composição de cada time é, onde a conta de duelo pende para um lado e,
com a base, a fase do jogo. Os fatos saem do Data Dragon (`comp::profile`); as
vantagens, dos `LaneOutlook` e `Skirmish` que `lanes` já calculou; a fase, da
tabela `length` da base. O desenho está em
[03 §4](03-motor-da-partida.md#condição-de-vitória) e no
[ADR-0018](../adr/0018-o-que-a-condicao-de-vitoria-pode-afirmar.md).

### 15.1 `Side`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Side {
    Nosso, // "nosso": o time do jogador (`ours`)
    Deles, // "deles": o time inimigo (`theirs`)
}
```

Sem `↔`: o front não tem união com nome. O espelho está escrito por dentro de
`WinPoint.side`, `"nosso" | "deles"` (`src/lib/types.ts`).

### 15.2 `Source`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Source {
    Regra,     // "regra": regra escrita à mão sobre a composição; só em `facts`
    Atributos, // "atributos": a conta de duelo, atributos e itens sem habilidades; só em `edges`
}
```

Sem `↔`, pelo mesmo motivo: o espelho é `WinPoint.source`,
`"regra" | "atributos"`.

### 15.3 `Point` ↔ `WinPoint`

```rust
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Point {
    pub side: Side,     // a favor de quem a frase fala
    pub source: Source, // de onde ela sai; a interface mostra ao lado (`CondicaoDeVitoria.tsx`)
    pub text: String,   // frase pronta em pt-BR: descrição, nunca ordem (C-5)
}
```

### 15.4 `WinCondition` ↔ `WinCondition`

```rust
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WinCondition {
    pub facts: Vec<Point>,     // fatos de composição, `Source::Regra`: primeiro os nossos, depois os deles
    pub edges: Vec<Point>,     // até 5 (`MAX_EDGES`) rotas e escaramuças a 5 pontos ou mais de 50%, `Source::Atributos`, da maior distância para a menor
    pub phase: Option<String>, // frase da fase pela tabela `length` da base, com o n somado dos dois times; `None` sem a base
    pub caveat: &'static str,  // aviso fixo (`CAVEAT`): o que nada disto vê. Sempre presente
}
```

**Invariantes:**
- Time com menos de 3 campeões conhecidos (`MIN_KNOWN`) não tem fato
  (`wincon.rs:82`).
- Todo fato é `Source::Regra` e toda vantagem é `Source::Atributos`
  (`wincon.rs:92`, `wincon.rs:208`, `wincon.rs:231`).
- `edges.len() ≤ 5` (`wincon.rs:242`).
- `phase` é `None` sem `has_global()` e quando nenhum dos seus tem partida na
  tabela `length` (`wincon.rs:137`, `wincon.rs:158`).

## 16. `core::engine::report` — A análise de rota inteira

Rotas, escaramuças, condição de vitória e, no draft, a chance do time, numa
montagem só. A partida chama com os níveis e itens reais (casca,
`analysis.rs`); o draft, com todos no mesmo nível de referência. A prévia de um
candidato é a mesma conta com ele no seu lugar
([03 §4](03-motor-da-partida.md#a-prévia-no-draft)).

### 16.1 `LaneReport` ↔ `LaneReport`

```rust
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LaneReport {
    pub lanes: Vec<LaneOutlook>,     // uma por rota com oponente identificado (`lanes::lane_outlooks`)
    pub skirmishes: Vec<Skirmish>,   // 2v2 e 3v3 (`lanes::skirmishes`)
    pub win_condition: WinCondition, // `wincon::analyze` sobre as duas listas de cima
    pub team: Option<TeamEstimate>,  // chance do time; só no draft (`from_context`) e só com a base
    pub source: &'static str,        // "draft" (`from_context`) ou "partida" (casca, `analysis.rs`)
    pub roles_assumed: bool,         // alguma rota saiu da inferência, não do cliente; a interface avisa
}
```

**Invariantes:**
- `team` é `None` sempre que `source` é `"partida"`: só `from_context` o
  preenche (`report.rs:71`).
- No draft, os dez estão no `PREGAME_LEVEL` (6) e sem itens (`report.rs:68`).
- `source` só leva os dois literais acima; o espelho TS é a união
  `"draft" | "partida"`.

## 17. `core::engine::poolgap` — Pool de oito

As características de cada campeão (`Trait`), o retrato dos três core e os
candidatos para o próximo lugar. Tudo sai do Data Dragon — classes, `info`,
atributos base — e de pesos por rota escritos à mão (`weight`,
`poolgap.rs:183`); da base, só o filtro de quem joga a rota. Não sai winrate. O
desenho está em [02 §7](02-motor-de-draft.md#7-pool) e no
[ADR-0016](../adr/0016-pool-de-oito-tres-escolhidos-cinco-por-cobertura.md).

### 17.1 `POOL_SIZE`, `CORE_SIZE`

```rust
pub const POOL_SIZE: usize = 8; // escolhido à mão (ADR-0016): campeões na pool completa de uma rota
pub const CORE_SIZE: usize = 3; // escolhido à mão (ADR-0016): quantos dela o jogador escolhe
```

O front tem a própria cópia das duas (`src/lib/types.ts`), e é ela que limita os
core na tela; nenhum código Rust lê `CORE_SIZE`.

### 17.2 `Trait` ↔ `PoolTrait`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Trait {
    Fisico,      // "fisico": `physical_share() ≥ 0,65`, escolhido à mão
    Magico,      // "magico": `physical_share() ≤ 0,35`, escolhido à mão
    Frontline,   // "frontline": `is_frontline()` — classe Tank ou `info.defense ≥ 7`
    Alcance,     // "alcance": `is_ranged()` — alcance de ataque ≥ 500
    CorpoACorpo, // "corpoACorpo": o contrário de `Alcance`
    Robusto,     // "robusto": `robustness ≥ 0,55` (`ROBUST`, escolhido à mão), no nível 11, sem itens
    Lutador,     // "lutador": tem a classe `Fighter` do Data Dragon, em qualquer posição de `tags`
    Tanque,      // "tanque": classe `Tank`
    Mago,        // "mago": classe `Mage`
    Assassino,   // "assassino": classe `Assassin`
    Atirador,    // "atirador": classe `Marksman`
    Suporte,     // "suporte": classe `Support`
}

impl Trait {
    // as 12 variantes, na ordem da declaração: a ordem em que `analyze` lê a pool
    pub const ALL: [Trait; 12] = [
        Trait::Fisico,
        Trait::Magico,
        Trait::Frontline,
        Trait::Alcance,
        Trait::CorpoACorpo,
        Trait::Robusto,
        Trait::Lutador,
        Trait::Tanque,
        Trait::Mago,
        Trait::Assassino,
        Trait::Atirador,
        Trait::Suporte,
    ];
}
```

**Invariantes:**
- `Fisico` e `Magico` nunca valem juntos, nem `Alcance` e `CorpoACorpo`.
- Os seis estilos são as classes do Data Dragon; um campeão com duas classes tem
  os dois estilos.

### 17.3 `Finding` ↔ `PoolFinding`

```rust
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Finding {
    #[serde(rename = "trait")]
    pub kind: Trait,  // a característica; vai como `trait` porque `trait` é palavra reservada no Rust
    pub text: String, // frase pronta em pt-BR: "2 de 3 com dano mágico", "nenhum segura a linha de frente"
}
```

### 17.4 `Analysis` ↔ `PoolAnalysis`

```rust
#[derive(Debug, Clone, Default, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Analysis {
    pub strengths: Vec<Finding>,  // o que a rota pesa 0,5 ou mais (`RELEVANT`) e ao menos um core tem; por peso, depois por quantos têm
    pub weaknesses: Vec<Finding>, // o que a rota pesa 0,5 ou mais e nenhum tem; e "um estilo só" quando todos dividem a classe principal
}
```

**Invariantes:**
- Estilo nunca é ponto forte. O único fraco de estilo é "um estilo só", com dois
  ou mais core (`poolgap.rs:311`).
- Sem core conhecido no índice, os dois vêm vazios.

### 17.5 `Candidate` ↔ `PoolCandidate`

```rust
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Candidate {
    pub champion_id: i64,     // id numérico do Data Dragon (`key`)
    pub name: String,         // nome do Data Dragon, no idioma do índice
    pub score: f64,           // índice de encaixe 0–1, não winrate: o ganho dele ÷ a soma dos ganhos de todas as características
    pub fills: Vec<Trait>,    // o que ninguém da pool tinha (n = 0) e ele traz
    pub reasons: Vec<String>, // até 3 (`MAX_REASONS`) frases, do maior ganho para o menor
}
```

**Invariantes:**
- `score` ∈ [0, 1] (`poolgap.rs:386`).
- `reasons` nunca vem vazio: candidato sem ganho sai da lista
  (`poolgap.rs:372`).
- O candidato não está na pool e joga a rota (`roles::plays`).

## 18. `core::engine::runes` — Runas do confronto

O índice de runas do Data Dragon, por chave textual, e o plano de runas para o
confronto da sua rota. O plano sai de regras mecânicas escritas à mão, nunca de
winrate ([03 §9](03-motor-da-partida.md#9-runas)).

### 18.1 `Rune`

```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Rune {
    pub id: i64,           // id numérico do Data Dragon: muda de patch, nunca é chave de busca (L-17)
    pub key: String,       // chave textual do Data Dragon (`Conqueror`), estável entre patches e idiomas
    pub name: String,      // nome no idioma pedido
    pub icon: String,      // caminho relativo do ícone no CDN (`perk-images/...`)
    pub tree_key: String,  // chave da árvore (`Precision`, `Resolve`, ...)
    pub tree_name: String, // nome da árvore no idioma pedido
}
```

A casca preenche a partir de `runesReforged.json` (`src-tauri/src/ddragon.rs`,
`fetch_runes`), achatando árvore → linha → runa. Nenhum teste trava os nomes de
campo desse JSON. Não cruza a ponte; vai para o cache `runas.json`.

### 18.2 `RuneIndex`

```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RuneIndex {
    pub version: String,               // versão do Data Dragon de onde saiu (`latest_version`)
    pub by_key: HashMap<String, Rune>, // chave textual da runa → runa
}
```

Sem `rename_all`: em `runas.json`, `version` e `by_key` ficam em snake_case e as
runas dentro em camelCase.

**Invariantes:**
- A casca nunca guarda um índice vazio vindo do Data Dragon: zero runas é erro
  (`ddragon.rs`, `fetch_runes`). Vazio só fica o `RuneIndex::default()` de antes
  da primeira carga, ou de quando o Data Dragon e o cache falham; com ele `plan`
  devolve `None`.

### 18.3 `RuneSuggestion` ↔ `RuneSuggestion`

```rust
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RuneSuggestion {
    pub key: String,          // chave textual, copiada de `Rune::key`
    pub id: i64,              // id do Data Dragon, copiado de `Rune::id`
    pub name: String,         // copiado de `Rune::name`
    pub icon: String,         // copiado de `Rune::icon`; o front prefixa o CDN
    pub tree_name: String,    // copiado de `Rune::tree_name`
    pub score: f64,           // soma dos pesos escritos à mão das regras que dispararam: índice de encaixe, não winrate, e passa de 1
    pub reasons: Vec<String>, // o motivo de cada regra que disparou, na ordem de `REGRAS`
}
```

**Invariantes:**
- `reasons` tem uma frase por regra somada; nunca vem vazio.
- `key` existe no `RuneIndex` usado: regra cuja runa sumiu não soma
  (`runes.rs:266`).

### 18.4 `RunePlan` ↔ `RunePlan`

```rust
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RunePlan {
    pub keystones: Vec<RuneSuggestion>,    // até 3 (`MAX_SUGGESTIONS`) runas principais, por `score` decrescente, depois nome
    pub secondary: Option<RuneSuggestion>, // uma runa da árvore secundária (`SECUNDARIAS`), `score` 1; `None` quando nenhuma regra casa
    pub matchup: String,                   // "<seu campeão> contra <oponente>", nomes do Data Dragon
    pub from_data: bool,                   // sempre `false` hoje: com ele a interface não escreve "segundo N partidas"
}
```

**Invariantes:**
- `keystones.len() ≤ 3` e `score` não cresce ao descer a lista
  (`runes.rs:281`).
- `from_data` é `false` em todo caminho (`runes.rs:298`).

## 19. `core::engine::stats` — Registro, estimativa e a loja de estatísticas

O que o motor sabe de vitória e derrota: o par partidas/vitórias, a estimativa
encolhida, a faixa de duração, o contrato da loja e o histórico da própria conta.
A conta do shrinkage está em [02 §2](02-motor-de-draft.md#2-shrinkage), e a
referência em [02 §3](02-motor-de-draft.md#3-a-referência).

**Unidade:** toda taxa deste módulo é **fração 0–1**, nunca pp. Quem passa para
pp é quem mostra (`Scored::delta`, 10 §9).

### 19.1 `Record`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Record {
    pub games: u32, // contagem de partidas; da match-v5 (coletor ou conta)
    pub wins: u32,  // contagem de vitórias entre essas partidas
}
```

Sem `rename_all`: no disco é `{"games": 40000, "wins": 21000}`, tanto em
`base-de-partidas.json` quanto em `personal-v2.json`.

**Invariantes:**
- `wins ≤ games` só quando o registro nasce de `Record::new` (`stats.rs:14`), que
  corta `wins` em `games`. O literal `Record { games, wins }` e o `Deserialize`
  não passam por ali: um arquivo com `wins > games` carrega como está.

### 19.2 `Estimate`

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Estimate {
    pub rate: f64,        // fração: a taxa encolhida, em [0, 1], quando vem de `shrink`; o desvio da referência, em [−1, 1], quando vem de `shrunk_delta`
    pub effective_n: f64, // partidas reais + força do prior (partidas fictícias)
    pub games: u32,       // contagem de partidas reais da amostra
}
```

**Invariantes:**
- `effective_n ≥ 1`: `shrink` passa a força por `max(1.0)` (`stats.rs:44`).
- O tipo não diz se `rate` é taxa ou desvio: depende de quem o criou.

### 19.3 `Length`

A faixa de duração da partida, que mede a fase do jogo em que um campeão rende.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Length {
    Curta, // gameDuration < 1500 s: termina antes de 25:00
    Media, // 1500 s ≤ gameDuration < 1950 s: de 25:00 a 32:29
    Longa, // gameDuration ≥ 1950 s: 32:30 em diante
}

impl Length {
    pub const ALL: [Length; 3] = [Length::Curta, Length::Media, Length::Longa]; // as três faixas, da mais curta à mais longa; os cortes de `Length::of` são escolhidos à mão
}
```

O nome da variante vai para a chave de `length` na base (`length_key`, 10 §20.2)
pelo `Debug`, não pelo `serde`.

### 19.4 `StatsStore`

O contrato que o motor consulta. `None` quer dizer **sem amostra**, e o
componente vale o prior ([ADR-0007](../adr/0007-sem-dado-o-componente-vale-zero.md)).

```rust
pub trait StatsStore: Send + Sync {
    fn base(&self, champion: i64, role: Role) -> Option<Record>;                       // o campeão na rota
    fn matchup(&self, champion: i64, opponent: i64, role: Role) -> Option<Record>;     // `champion` contra `opponent` na mesma rota; direcional
    fn synergy(&self, champion: i64, ally: i64) -> Option<Record>;                     // os dois no mesmo time; sem ordem
    fn personal(&self, champion: i64) -> Option<Record>;                               // a conta do jogador com o campeão
    fn versus(&self, _champion: i64, _opponent: i64) -> Option<Record>;                // padrão `None`: contra o oponente em qualquer rota inimiga; direcional
    fn length(&self, _champion: i64, _role: Role, _bucket: Length) -> Option<Record>;  // padrão `None`: na rota, numa faixa de duração
    fn has_global(&self) -> bool;                                                      // padrão `false`: a base agregada está ligada
    fn has_personal(&self) -> bool;                                                    // padrão `false`: há histórico da conta
    fn personal_games(&self) -> u32;                                                   // padrão `0`: contagem de partidas do histórico
    fn lane_profile(&self) -> Option<LaneProfile>;                                     // padrão `None`: o perfil de rota (10 §21), recalculado a cada chamada
    fn played_in(&self, _champion: i64, _role: Role) -> bool;                          // padrão `false`: a conta já jogou o campeão na rota
}
```

Os ids são o id numérico do Data Dragon (`Champion::id`, 10 §23.3). Quem implementa
e o que cada implementação responde está em 11 §19.10.

### 19.5 `NoStats`

```rust
pub struct NoStats; // sem estado: tudo `None`, `false` ou `0`
```

A casca começa com ela (`watcher.rs:103`) e a troca por `Layered` no `setup`
(`lib.rs:634`). Os testes do motor a usam como "sem base nenhuma".

### 19.6 `PersonalStats`

O histórico da própria conta: vitórias por campeão e a comparação com o oponente
direto em cada partida. Cobre só o componente pessoal, de propósito
([02 §1](02-motor-de-draft.md#1-rank)).

```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PersonalStats {
    pub puuid: String,                     // a conta dona do histórico (account-v1); a casca recomeça do zero quando ele muda
    pub by_champion: HashMap<i64, Record>, // id do Data Dragon → partidas e vitórias da conta, da match-v5
    pub seen: Vec<String>,                 // ids de partida da match-v5 já lidos: contados ou descartados
    #[serde(default)]
    pub lanes: Vec<LaneGame>,              // uma por partida com oponente direto (10 §5)
}
```

Quem preenche é a casca (`profile.rs`), com `add_played` e `mark_seen`, e grava em
`personal-v2.json` ([04 §7](04-perfil-de-rota.md#7-o-cache)). Sem `rename_all`: as
chaves são `puuid`, `by_champion`, `seen` e `lanes`; dentro de `lanes`, cada
`LaneGame` é camelCase.

**Invariantes:**
- `seen` não repete id: `add_match` e `mark_seen` conferem antes de inserir
  (`stats.rs:233`, `stats.rs:239`).
- Cada partida soma no máximo uma vez em `by_champion`, e `total_games()` é a soma
  de `games` sobre `by_champion`.
- `lanes` tem no máximo uma entrada por partida contada.
- `lanes` é `default`: o cache sem a comparação de rota carrega
  (`cache_antigo_carrega_sem_rotas`).

## 20. `core::engine::dataset` — A base de partidas

O formato do arquivo que o coletor produz e a camada que junta a base agregada ao
histórico da conta. O desenho está em
[02 §8](02-motor-de-draft.md#8-a-base-de-partidas); aqui ficam os tipos exatos.

### 20.1 `Dataset`

```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Dataset {
    pub patch: String,        // patch das partidas ("16.21"); a interface mostra, porque base de patch velho recomenda build velha
    pub region: String,       // plataforma da coleta ("BR1"); o motor não lê
    pub tier: String,         // faixa de elo semeada ("EMERALD+"); o motor não lê
    pub generated_at: String, // ISO 8601, quando o coletor gravou; o motor não lê
    pub matches: u64,         // contagem de partidas que entraram na agregação
    #[serde(default)]
    pub base: HashMap<String, Record>,    // chave `base_key`
    #[serde(default)]
    pub matchup: HashMap<String, Record>, // chave `matchup_key`
    #[serde(default)]
    pub synergy: HashMap<String, Record>, // chave `synergy_key`
    #[serde(default)]
    pub versus: HashMap<String, Record>,  // chave `versus_key`
    #[serde(default)]
    pub length: HashMap<String, Record>,  // chave `length_key`
}
```

Quem preenche é o coletor (`tools/ingest/src/aggregate.rs`, `into_dataset`), que
corta as entradas com menos de `min_games` partidas. A casca lê o arquivo no
`setup` (`lib.rs:622`) e só o aceita se `is_usable()`.

**Invariantes:**
- O `default` do struct inteiro faz qualquer chave ausente virar vazia: arquivo
  truncado, de coletor mais velho ou escrito à mão carrega
  (`arquivo_so_com_o_cabecalho_carrega_sem_quebrar`, `base_antiga_sem_contra_carrega`).
- `is_usable()` é `matches > 0` e `base` não vazio (`dataset.rs:77`).
- Chave desconhecida no JSON é ignorada, sem erro.

### 20.2 O arquivo `base-de-partidas.json`

Fica em `%LOCALAPPDATA%\Presage\`, ao lado dos outros caches. O exemplo completo
está em [02 §8](02-motor-de-draft.md#8-a-base-de-partidas).

| Chave JSON | Campo | Tipo | Formato da chave do mapa | Exemplo |
|---|---|---|---|---|
| `patch` | `patch` | texto | — | `"16.21"` |
| `region` | `region` | texto | — | `"BR1"` |
| `tier` | `tier` | texto | — | `"EMERALD+"` |
| `generatedAt` | `generated_at` | texto, ISO 8601 | — | `"2026-10-27T00:00:00Z"` |
| `matches` | `matches` | inteiro ≥ 0 (`u64`) | — | `1200000` |
| `base` | `base` | objeto de `Record` | `<id>:<Rota>` | `"103:Mid"` |
| `matchup` | `matchup` | objeto de `Record` | `<id>:<oponente>:<Rota>`, direcional | `"103:238:Mid"` |
| `synergy` | `synergy` | objeto de `Record` | `<menor id>:<maior id>` | `"64:103"` |
| `versus` | `versus` | objeto de `Record` | `<id>:<oponente>`, direcional, qualquer rota inimiga | `"103:64"` |
| `length` | `length` | objeto de `Record` | `<id>:<Rota>:<Faixa>` | `"103:Mid:Longa"` |

- Cada `Record` é `{"games": <u32>, "wins": <u32>}`.
- `<id>` é o id numérico do Data Dragon, em decimal.
- `<Rota>` é o `Debug` de `Role` (`Top`, `Jungle`, `Mid`, `Bot`, `Support`,
  `Unknown`), e `<Faixa>` o de `Length` (`Curta`, `Media`, `Longa`).
- As chaves saem das funções de 11 §20.1, que o coletor e o app compartilham.

### 20.3 `Layered`

```rust
#[derive(Debug, Clone, Default)]
pub struct Layered {
    pub global: Option<Dataset>,   // a base agregada; `None` sem arquivo ou com arquivo inutilizável
    pub personal: PersonalStats,   // o histórico da conta; vazio sem chave da Riot
}
```

É a `StatsStore` que o app usa (`lib.rs:91`). A camada global responde base,
matchup, sinergia, contra e duração; a pessoal responde o componente pessoal, o
perfil de rota e `played_in`.

**Invariantes:**
- As duas camadas não se cruzam: ter a conta não produz matchup, ter a base não
  produz componente pessoal (`as_camadas_sao_independentes`).
- `with_global` guarda a base só se `is_usable()`. A casca monta o `Layered` com
  literal (`lib.rs:91`) e não passa por ali: a mesma trava está no `setup`.

## 21. `core::engine::profile` — Perfil de rota

O quanto você tira do oponente direto, em três níveis, cada um encolhido para o de
cima. O desenho está em [04 §4](04-perfil-de-rota.md#4-três-níveis). Quem monta é
`profile::build`, a partir de `PersonalStats::lanes`; o front recebe pelo comando
`lane_profile`.

**Unidade:** `raw`, `value` e `sd` estão na unidade da métrica, que vem de
`history::Metric` (10 §5):

| Métrica | Unidade |
|---|---|
| `Cs10` | tropas aos 10 minutos (campos da selva, para o caçador) |
| `KillParticipation` | **pp** (a fração da match-v5 vezes 100) |
| `DamageShare` | **pp** (idem) |
| `VisionPerMinute` | placar de visão por minuto |
| `Deaths` | mortes por partida; a única em que menos é melhor |

### 21.1 `Reading` ↔ `LaneReading`

```rust
#[derive(Debug, Clone, Copy, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Reading {
    pub metric: Metric, // `history::Metric`: diz a unidade dos três números abaixo
    pub games: u32,     // contagem de partidas deste nível que têm a métrica nos dois lados
    pub raw: f64,       // média crua de você menos o oponente, na unidade da métrica
    pub value: f64,     // a média encolhida para o nível de cima; é o número da tela
    pub sd: f64,        // desvio padrão de `value`, na mesma unidade
}
```

**Invariantes:**
- `games ≥ 1` e `sd ≥ 0`.
- No geral, `value == raw` e `sd` é o erro padrão da média
  (`media_geral_e_medida_nao_encolhida`).
- Métrica com menos de `MIN_GAMES` partidas no geral não aparece em nenhum nível
  (`profile.rs:191`).

### 21.2 `MatchupLane` ↔ `MatchupLane`

```rust
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MatchupLane {
    pub opponent: i64,          // id do Data Dragon do oponente direto
    pub games: u32,             // contagem de todas as partidas com o campeão contra ele, com ou sem cada métrica
    pub readings: Vec<Reading>, // encolhidas para o campeão; na ordem de `Metric::ALL`
}
```

### 21.3 `ChampionLane` ↔ `ChampionLane`

```rust
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ChampionLane {
    pub champion: i64,              // id do Data Dragon
    pub games: u32,                 // contagem de todas as partidas com ele, com ou sem cada métrica
    pub readings: Vec<Reading>,     // encolhidas para a sua média geral; na ordem de `Metric::ALL`
    pub matchups: Vec<MatchupLane>, // mais jogado primeiro, empate pelo menor id
}
```

**Invariantes:**
- `games` conta partidas, e `Reading::games` conta partidas com a métrica: o
  suporte tem `games = 6` e nenhuma leitura de `Cs10`
  (`metrica_que_falta_nao_entra_na_conta`).
- Um confronto de uma partida só entra na lista; quem esconde é a tela
  ([04 §5](04-perfil-de-rota.md#5-na-tela-e-no-draft)).

### 21.4 `LaneProfile` ↔ `LaneProfile`

```rust
#[derive(Debug, Clone, Default, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LaneProfile {
    pub games: u32,                   // contagem de `LaneGame` lidas
    pub overall: Vec<Reading>,        // você no geral: média e erro padrão, sem encolher
    pub champions: Vec<ChampionLane>, // mais jogado primeiro, empate pelo menor id
}
```

**Invariantes:**
- Nenhuma nota única: não há campo que junte as métricas (C-7,
  [04 §6](04-perfil-de-rota.md#6-o-que-ele-não-separa)).
- Com menos de cinco partidas em toda métrica, `overall` e `champions` saem vazios
  (`menos_de_cinco_partidas_nao_vira_perfil`).

## 22. `core::engine::rival` — Sua rota ao vivo

Você contra o oponente direto durante a partida, na métrica que a rota cobra e em
ouro de itens. Tudo vem do `LiveGame` (10 §4), que a casca lê da Live Client a
cada segundo. O desenho está em [03 §6.1](03-motor-da-partida.md#61-sua-rota-ao-vivo).

### 22.1 Constantes

```rust
pub const MIN_SECONDS: f64 = 90.0;  // escolhido à mão, segundos de `gameTime`: antes de 1:30 não há painel
pub const AHEAD_FLOOR: f64 = 300.0; // escolhido à mão, ouro: o piso da margem de "na frente"
pub const AHEAD_SHARE: f64 = 0.10;  // escolhido à mão, fração 0–1 do ouro em itens do oponente
```

A margem é `max(AHEAD_FLOOR, AHEAD_SHARE × ouro em itens do oponente)`. É regra,
não estatística.

### 22.2 `Metric`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Metric {
    Farm,   // "farm": tropas por minuto, de `scores.creepScore` da Live Client
    Vision, // "vision": placar de visão por minuto, de `scores.wardScore`
}
```

O front não tem um tipo com nome para ela: a união `"farm" | "vision" | null` está
escrita dentro de `Rival`, em `src/lib/types.ts`.

### 22.3 `Standing`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Standing {
    Behind, // "behind": diferença de ouro em itens ≤ −margem
    Even,   // "even": dentro da margem
    Ahead,  // "ahead": diferença ≥ margem
}
```

Como `Metric`, chega ao front como união escrita dentro de `Rival`.

### 22.4 `Rival` ↔ `Rival`

```rust
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Rival {
    pub role: Role,              // a sua rota, de `position` da Live Client por `Role::from_lcu`; nunca `Unknown`
    pub opponent_id: i64,        // id do Data Dragon do oponente; 0 quando o `ChampionIndex` não conhece a chave
    pub opponent_key: String,    // chave textual do oponente ("LeeSin"), como a Live Client manda
    pub metric: Option<Metric>,  // o que a rota cobra, por `cobranca`; `None` se não cobra farm nem visão
    pub mine_per_min: f64,       // a sua métrica por minuto de `gameTime`, arredondada a 0,1; 0 com `metric` `None`
    pub theirs_per_min: f64,     // a do oponente, mesma conta
    pub gold_diff: f64,          // ouro: o seu ouro em itens menos o dele (soma de `price` dos itens da Live Client)
    pub standing: Standing,      // atrás, igual ou na frente por `gold_diff` e a margem de 22.1
}
```

A casca põe o `Rival` no `LiveSnapshot` (`watcher.rs:301`).

**Invariantes:**
- Só existe com `gameTime ≥ 90`, jogador local com rota conhecida e um inimigo na
  mesma rota (`rival.rs:81`–`rival.rs:92`).
- `metric` nunca é `Vision` para quem farma nem `Farm` para o suporte
  (`suporte_ao_vivo_e_comparado_pela_visao_e_nunca_por_tropas`).

## 23. `core::engine::champion` — Campeão, rota e atributos base

O campeão como o Data Dragon o descreve, a rota e o crescimento de atributo por
nível. A casca preenche tudo em `ddragon.rs` a partir de `champion.json`, no idioma
do usuário, e guarda em `champions.json`. A fórmula de crescimento está em
[03 §2](03-motor-da-partida.md#de-onde-vêm-os-números).

### 23.1 `Role` ↔ `Role`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Role {
    Top,     // "top" no LCU, "TOP" na Live Client e na match-v5
    Jungle,  // "jungle"
    Mid,     // "middle"
    Bot,     // "bottom"
    Support, // "utility"
    Unknown, // posição em branco ou fora das cinco
}
```

**Invariantes:**
- O nome da variante é formato em disco duas vezes: pelo `serde` em
  `personal-v2.json` e `pool.json`, e pelo `Debug` nas chaves de
  `base-de-partidas.json` (10 §20.2). Renomear uma variante invalida os dois sem
  erro.

### 23.2 `BaseStats`

```rust
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BaseStats {
    pub health: f64,                  // vida no nível 1, `stats.hp` do Data Dragon
    pub health_per_level: f64,        // ganho por nível, `stats.hpperlevel`
    pub armor: f64,                   // armadura no nível 1, `stats.armor`
    pub armor_per_level: f64,         // `stats.armorperlevel`
    pub magic_resist: f64,            // resistência mágica no nível 1, `stats.spellblock`
    pub magic_resist_per_level: f64,  // `stats.spellblockperlevel`
    pub attack_damage: f64,           // dano de ataque no nível 1, `stats.attackdamage`
    pub attack_damage_per_level: f64, // `stats.attackdamageperlevel`
}
```

### 23.3 `Champion`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Champion {
    pub id: i64,           // id numérico: o campo `key` do Data Dragon ("103"), convertido
    pub key: String,       // chave textual: o campo `id` do Data Dragon ("Ahri", "LeeSin"); é o que a Live Client manda e não muda com o idioma
    pub name: String,      // nome no idioma do usuário
    pub tags: Vec<String>, // classes do Data Dragon; a primeira é a principal ("Marksman", "Mage")
    pub attack: f64,       // `info.attack`, nota 0–10 do Data Dragon
    pub magic: f64,        // `info.magic`, nota 0–10
    pub defense: f64,      // `info.defense`, nota 0–10
    pub attack_range: f64, // `stats.attackrange`, em unidades de distância do jogo
    #[serde(default)]
    pub base: BaseStats,   // atributos no nível 1 e ganho por nível
}
```

**Invariantes:**
- Os nomes cruzam: `Champion::id` vem do `key` do Data Dragon, e `Champion::key`
  do `id` (`ddragon.rs:166`, `ddragon.rs:171`).
- `base` é `default`: um `champions.json` gravado antes dos atributos base carrega
  com tudo em zero.

### 23.4 `ChampionIndex`

```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChampionIndex {
    pub version: String,               // versão do Data Dragon ("16.21.1"), a mais nova de `versions.json`
    pub by_id: HashMap<i64, Champion>, // id numérico → campeão
}
```

**Invariantes:**
- A casca recusa índice vazio (`ddragon.rs:193`): sem campeão, cai para o cache.

## 24. `core::engine::items` — Itens e atributos

Os itens do Data Dragon e os atributos lidos da `description` em en_US, porque o
bloco `stats` não traz penetração, letalidade nem aceleração de habilidade
([ADR-0015](../adr/0015-atributos-de-item-pela-descricao.md)). A casca preenche
em `ddragon.rs` e guarda em `items.json`.

### 24.1 `ItemStats`

```rust
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemStats {
    pub health: f64,            // pontos, rótulo "Health"
    pub armor: f64,             // pontos, "Armor"
    pub magic_resist: f64,      // pontos, "Magic Resist" ou "Magic Resistance"
    pub attack_damage: f64,     // pontos, "Attack Damage"
    pub ability_power: f64,     // pontos, "Ability Power"
    pub lethality: f64,         // pontos, "Lethality": penetração plana de armadura
    pub armor_pen_percent: f64, // fração 0–1 ("35%" → 0,35), "Armor Penetration"; fontes combinam multiplicativo
    pub magic_pen_flat: f64,    // pontos, "Magic Penetration" sem `%`
    pub magic_pen_percent: f64, // fração 0–1, "Magic Penetration" com `%`; combina multiplicativo
    pub crit_chance: f64,       // fração 0–1, "Critical Strike Chance"; `plus` corta em 1
    pub attack_speed: f64,      // fração da velocidade de ataque bônus ("25%" → 0,25), "Attack Speed"; soma sem teto
    pub ability_haste: f64,     // pontos, "Ability Haste"
}
```

**Invariantes:**
- `armor_pen_percent` e `magic_pen_percent` ficam em [0, 1]: a combinação é
  `1 − (1 − a)(1 − b)`, com cada lado cortado em [0, 1] (`items.rs:80`).
- `crit_chance ≤ 1` depois de `plus`.
- Rótulo desconhecido não muda nada (`rotulo_desconhecido_e_ignorado_sem_perder_o_resto`).

### 24.2 `Item`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub id: i64,           // id do item no Data Dragon (a chave de `item.json`)
    pub name: String,      // nome no idioma do usuário; o de en_US se faltar
    pub cost: f64,         // ouro: `gold.total`, o preço cheio com os componentes
    pub purchasable: bool, // `gold.purchasable`
    pub stats: ItemStats,  // `parse_stats` da `description` em en_US
}
```

### 24.3 `ItemIndex`

```rust
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ItemIndex {
    pub version: String,           // versão do Data Dragon ("16.21.1")
    pub by_id: HashMap<i64, Item>, // id do item → item
}
```

**Invariantes:**
- A casca recusa índice vazio (`ddragon.rs:310`).

## 25. `core::engine::damage` — Dano e resistência

As regras de resistência do jogo, sem amostra nem prior. A ordem está em
[03 §3](03-motor-da-partida.md#3-dano).

### 25.1 `Penetration`

```rust
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Penetration {
    pub flat_reduction: f64,    // pontos de redução plana; aplica primeiro e pode levar a resistência abaixo de zero
    pub percent_reduction: f64, // fração 0–1 de redução; segundo passo
    pub percent: f64,           // fração 0–1 de penetração; terceiro passo
    pub flat: f64,              // pontos de penetração plana (letalidade, pen mágica plana); por último, para em zero
}
```

Quem preenche é `ItemStats::physical_penetration` e `magic_penetration`, com as
duas reduções sempre em zero: nenhum item do índice carrega redução hoje.

**Invariantes:**
- `percent` e `percent_reduction` fora de [0, 1] são cortados na conta
  (`damage.rs:29`, `damage.rs:32`).

## 26. `core::engine::build` — Build adaptativa

O combatente como a partida ou o draft o descrevem, os atributos dele somados e
o que a build adaptativa manda para o painel. Os atributos base vêm do Data
Dragon, pela curva de crescimento por nível. Os de item vêm da `description` em
en_US ([ADR-0015](../adr/0015-atributos-de-item-pela-descricao.md)). Inventário e
nível vêm da Live Client. O desenho está em [03 §2](03-motor-da-partida.md#2-build-adaptativa).

### 26.1 `Combatant`

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Combatant {
    pub champion_id: i64, // id numérico do Data Dragon (`key`); 0 quando a Live Client deu uma chave fora do índice
    pub level: u8,        // nível 1–18; `grown` corta fora dessa faixa
    pub items: Vec<i64>,  // ids de item do Data Dragon no inventário; id desconhecido some em `ItemIndex::totals`
}
```

Quem preenche: `LivePlayer::as_combatant` na partida, `Lineup::new` no draft e na
análise de rota.

### 26.2 `Loadout`

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Loadout {
    pub health: f64,               // pontos de vida: base no nível (`health_at`) + itens
    pub armor: f64,                // armadura: base no nível + itens
    pub magic_resist: f64,         // resistência mágica: base no nível + itens
    pub attack_damage: f64,        // dano de ataque total: base no nível + itens
    pub ability_power: f64,        // poder de habilidade: só dos itens; o Data Dragon não dá base
    pub bonus_attack_damage: f64,  // dano de ataque só dos itens; ninguém lê hoje
    pub physical_pen: Penetration, // penetração de armadura dos seus itens: percentual (fração 0–1) e letalidade; redução 0
    pub magic_pen: Penetration,    // penetração mágica dos seus itens: percentual (fração 0–1) e plana; redução 0
}
```

**Invariantes:**
- Só sai de `loadout` (e de `loadout_from`, dentro de `suggest`): campeão fora do
  índice não tem `Loadout`.
- `physical_pen.percent` e `magic_pen.percent` estão em [0, 1]: a soma de itens
  combina percentuais de forma multiplicativa (`ItemStats::plus`).

### 26.3 `BuildContext` ↔ `BuildContext`

```rust
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BuildContext {
    pub incoming_physical_share: f64, // fração 0–1 do dano inimigo que é físico (`incoming_physical_share`); 0,5 sem campeão seu conhecido
    pub enemy_armor_pen_percent: f64, // fração 0–1: a maior penetração percentual de armadura entre os inimigos
    pub enemy_lethality: f64,         // pontos de letalidade (penetração plana): a maior entre os inimigos
    pub enemy_magic_pen_percent: f64, // fração 0–1: a maior penetração mágica percentual; o front não mostra
    pub your_effective_hp: f64,       // vida efetiva contra essa mistura e essa penetração; 0 sem campeão seu conhecido
}
```

Quem preenche: `suggest`. Vai para a ponte como o primeiro elemento da tupla de
`build_suggestions`.

**Invariantes:**
- Os três campos de penetração vêm de `team_penetration`, cada um o máximo do
  seu componente entre os inimigos, e não necessariamente do mesmo inimigo.

### 26.4 `ItemSuggestion` ↔ `ItemSuggestion`

```rust
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ItemSuggestion {
    pub item_id: i64,              // id do item no Data Dragon
    pub name: String,              // nome no idioma do usuário (Data Dragon localizado); os atributos vêm do en_US
    pub cost: f64,                 // ouro: `gold.total` do Data Dragon, o preço cheio, sem descontar componente que você tem
    pub value_per_1000g: f64,      // % de ganho de poder (vida efetiva × dano efetivo) por 1000 de ouro, arredondado a 0,1; índice, não DPS
    pub ehp_gain: f64,             // pontos de vida efetiva a mais, arredondado a 0,1
    pub offense_gain_percent: f64, // % de ganho do dano efetivo (`offense_against`), arredondado a 0,1
    pub reasons: Vec<String>,      // até 2 frases em pt-BR (`reasons_for`)
}
```

**Invariantes** (todos de `suggest`):
- O item é comprável, custa pelo menos `MIN_ITEM_COST` e não está no seu inventário.
- `value_per_1000g` é finito.
- A lista vem em `value_per_1000g` decrescente, empate por `item_id` crescente, e
  tem no máximo `limit` itens.

## 27. `core::engine::lanes` — Rotas e escaramuças

A chance de cada rota, as escaramuças de 2 e 3 contra 2 e 3, e a escaramuça que o
overlay destaca. Tudo sai da conta de duelo de `build` (`skirmish_odds`) sobre
Data Dragon e itens; o matchup vem da base de partidas e, sem ela, vale zero
([ADR-0007](../adr/0007-sem-dado-o-componente-vale-zero.md)). A fórmula, o peso e
as curvas estão em [03 §4](03-motor-da-partida.md#4-chance-por-rota-e-escaramuça).

### 27.1 `BREAKPOINTS`

```rust
pub const BREAKPOINTS: [u8; 4] = [3, 6, 11, 16]; // níveis 1–18. 6, 11 e 16 são mecânica do jogo (as três cargas da suprema); 3 ("todas as básicas") é escolhido à mão
```

Define `by_level` e `gaps` de `LaneOutlook` e `by_level` de `Skirmish`.

### 27.2 `LevelOdds` ↔ `LevelOdds`

```rust
#[derive(Debug, Clone, Copy, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LevelOdds {
    pub level: u8, // nível 1–18 do inimigo; em `by_level` e `curve`, o dos dois
    pub gap: i8,   // o seu nível menos o dele: 0 em `by_level` e `curve`; +1, 0 ou −1 em `gaps`
    pub odds: f64, // fração 0–1: `skirmish_odds` com esses níveis e os itens de agora, sem `W_DUEL` nem a base
}
```

**Invariantes:**
- `odds` está em [0, 1]: razão de dois poderes não negativos, ou 0,5 quando os
  dois são zero.
- `level + gap` está em 1..=18 (`level_gaps` pula o que sair da faixa).
- Trocar os lados dá `1 − odds` em cada ponto (`a_curva_tambem_e_simetrica`).

### 27.3 `LaneOutlook` ↔ `LaneOutlook`

```rust
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LaneOutlook {
    pub role: Role,              // a rota do confronto; serializa como o nome da variante ("Top"…)
    pub lane: &'static str,      // rótulo curto de `Role::short`: "TOP", "JG", "MID", "ADC", "SUP"
    pub ally: i64,               // id do Data Dragon do seu campeão na rota
    pub enemy: i64,              // id do Data Dragon do oponente direto
    pub ally_name: String,       // nome do Data Dragon; `#id` quando o campeão não está no índice
    pub enemy_name: String,      // idem, do oponente
    pub win_probability: f64,    // fração 0–1, cortada em [`FLOOR`, `CEILING`]; fórmula em 03 §4
    pub margin: f64,             // pp: (`win_probability` − 0,5) × 100
    pub sample: u32,             // contagem: partidas do matchup na base; 0 sem base
    pub from_data: bool,         // verdadeiro quando o matchup tem partidas ou um dos dois tem winrate base na rota
    pub by_level: Vec<LevelOdds>, // 4 pontos, os níveis de `BREAKPOINTS`, os dois nivelados, com os itens de agora
    pub curve: Vec<LevelOdds>,   // 18 pontos, níveis 1 a 18, os dois nivelados
    pub gaps: Vec<LevelOdds>,    // 12 pontos: cada nível de `BREAKPOINTS` com você um à frente, nivelado e um atrás
    pub duel: f64,               // fração 0–1: o 1v1 nos níveis e itens reais, sem `W_DUEL` nem a base
    pub my_level: u8,            // nível 1–18 do seu `Lineup`: o da Live Client na partida, `PREGAME_LEVEL` no draft
    pub their_level: u8,         // idem, do oponente
    pub reasons: Vec<String>,    // até `MAX_REASONS` frases em pt-BR
}
```

**Invariantes** (todos de `lane_outlooks`):
- `win_probability` está em [`FLOOR`, `CEILING`]
  (`a_probabilidade_nunca_chega_ao_extremo`).
- Sem base, `from_data` é falso, `sample` é 0 e nenhuma razão cita `n=`
  (`sem_base_de_partidas_o_numero_vem_so_do_duelo`).
- `by_level` e `curve` não dependem dos níveis de entrada; `duel`, `my_level` e
  `their_level`, sim (`a_curva_nivela_os_dois_lados`).

### 27.4 `Skirmish` ↔ `Skirmish`

```rust
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Skirmish {
    pub key: &'static str,       // chave estável do grupo em `GROUPS`: "jg_top", "jg_mid", "bot", "dragao", "arauto"
    pub label: String,           // "JG+TOP x JG+TOP": `Role::short` das rotas do grupo, o mesmo rótulo dos dois lados
    pub detail: &'static str,    // texto fixo do grupo em `GROUPS` ("gank ou invade no topo"…)
    pub size: usize,             // contagem: 2 ou 3, as rotas do grupo
    pub ours: Vec<i64>,          // ids do Data Dragon do seu lado, na ordem das rotas do grupo
    pub theirs: Vec<i64>,        // ids do Data Dragon do lado deles, na mesma ordem
    pub odds: f64,               // fração 0–1: `skirmish_odds` do grupo com os níveis e itens reais
    pub by_level: Vec<LevelOdds>, // 4 pontos, os níveis de `BREAKPOINTS`, nivelados; não há `curve` nem `gaps`
    pub reasons: Vec<String>,    // até `MAX_REASONS` frases: quem mais pesa de cada lado e quem tem frontline
}
```

**Invariantes:**
- `ours.len() == theirs.len() == size`: grupo com uma rota faltando de qualquer
  lado não sai (`grupo_incompleto_fica_de_fora_em_vez_de_virar_2v1`).
- A interface escolhe por `key`, nunca pelo texto de `detail`.

### 27.5 `Lineup`

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Lineup {
    pub role: Role,           // rota já resolvida: a posição da Live Client na partida; a do `DraftContext` no draft (informada, marcada ou inferida)
    pub combatant: Combatant, // campeão, nível e itens
}
```

Não cruza a ponte. Quem monta: `DraftContext::lineups` e `analysis.rs` na casca,
sempre por `Lineup::new`.

### 27.6 `Focus` ↔ `SkirmishFocus`

```rust
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Focus {
    pub skirmish: Skirmish, // cópia da escaramuça escolhida por `focus`
    pub because: String,    // o porquê, em pt-BR: "Dragão em 0:40", "Dragão disponível", "a briga da sua rota", "a briga mais decidida agora"
}
```

O nome muda na ponte: no front a interface é `SkirmishFocus`. Vai para o overlay
pelo comando `skirmish_focus`.

## 28. `core::engine::gold` — Ouro dos dez

Duas colunas por jogador: o ouro **investido**, soma exata do `price` de cada item
que a Live Client entrega, e o ouro **ganho**, reconstruído a partir de tempo,
tropas, abates e assistências. O desenho está em [03 §5](03-motor-da-partida.md#5-ouro).

### 28.1 `GoldPlayer` ↔ `GoldPlayer`

```rust
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GoldPlayer {
    pub champion_key: String,  // chave interna do Data Dragon, tirada do `rawChampionName` da Live Client (`MonkeyKing`, não "Wukong")
    pub champion_id: i64,      // id do Data Dragon; 0 quando a chave não está no índice (`to_live_game`)
    pub display_name: String,  // Riot ID, ou o nome de invocador quando ele vem vazio, da Live Client
    pub team: String,          // "ORDER" ou "CHAOS", da Live Client
    pub position: String,      // posição crua da Live Client ("TOP", "JUNGLE", "MIDDLE", "BOTTOM", "UTILITY" ou vazia)
    pub is_local: bool,        // o jogador desta máquina (`LivePlayer::is_local`)
    pub is_ally: bool,         // mesmo `team` do jogador local; falso para todos quando não há jogador local
    pub invested: f64,         // ouro, exato: soma dos `price` dos itens (`LivePlayer::gold_invested`)
    pub earned: f64,           // ouro, estimativa: `earned_at`, nunca abaixo de `invested`
    pub unspent: Option<f64>,  // ouro, estimativa: `earned − invested`, nunca negativo; só do jogador local, `None` para os outros nove (C-4)
    pub gold_per_minute: f64,  // ouro por minuto: `earned` dividido pelos minutos de partida, com piso de 1 s no divisor
}
```

**Invariantes:**
- `earned ≥ invested` e, para o jogador local, `unspent = Some(earned − invested)`
  (`earned_at` devolve o máximo entre a estimativa e o investido).
- `unspent` é `None` para todo jogador com `is_local = false`: ouro em mão de
  outro jogador nunca cruza a ponte (C-4, R-12; T-OURO-10).

### 28.2 `GoldReport` ↔ `GoldReport`

```rust
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GoldReport {
    pub game_time: f64,          // segundos de partida, da Live Client
    pub players: Vec<GoldPlayer>, // os jogadores de `LiveGame`, na ordem da Live Client, sem bots
    pub ours_invested: f64,      // ouro, exato: soma de `invested` de quem `is_ally`
    pub theirs_invested: f64,    // ouro, exato: soma do resto; sem jogador local, de todos
    pub invested_lead: f64,      // ouro, exato: `ours_invested − theirs_invested`; positivo quando o seu time está à frente
    pub ours_earned: f64,        // ouro, estimativa: soma de `earned` de quem `is_ally`
    pub theirs_earned: f64,      // ouro, estimativa: soma do resto
    pub earned_lead: f64,        // ouro, estimativa: `ours_earned − theirs_earned`
    pub notes: Vec<String>,      // até 3 frases em pt-BR; vazio sem jogador local
}
```

**Invariantes:**
- Sem jogador local, ninguém é aliado, `ours_invested` e `ours_earned` são 0 e
  `notes` vem vazio (`sem_jogador_local_nao_inventa_um_lado`).

## 29. `core::engine::timeline` — Timers de objetivo, estruturas e buffs

O que o fluxo de eventos da Live Client (`/liveclientdata/eventdata`) diz sobre
objetivos: quantos cada lado tomou, quando cada objetivo volta, que estrutura
espera renascer e quem ainda tem buff. Todo tempo é em segundos de partida, o
`game_time` da Live Client. A diferença entre número observado e estimado está
em [03 §6](03-motor-da-partida.md#6-timers).

### 29.1 `Timings`

```rust
#[serde(rename_all = "camelCase")]
pub struct Timings {
    pub dragon_first: f64,         // s de partida: primeiro dragão. Mecânica do jogo
    pub dragon_respawn: f64,       // s depois do abate: próximo dragão. Mecânica do jogo
    pub elder_respawn: f64,        // s depois do abate: Ancião, com alma fechada. Mecânica do jogo
    pub baron_first: f64,          // s de partida: primeiro Barão. Mecânica do jogo
    pub baron_respawn: f64,        // s depois do abate: próximo Barão. Mecânica do jogo
    pub herald_spawn: f64,         // s de partida: Arauto, único. Mecânica do jogo
    pub grubs_spawn: f64,          // s de partida: Grubs, únicos. Mecânica do jogo
    pub dragons_for_soul: u32,     // contagem de dragões de um lado que fecha a alma. Mecânica do jogo
    pub inhibitor_respawn: f64,    // s depois de cair: o inibidor volta. Mecânica do jogo
    pub nexus_turret_respawn: f64, // s depois de cair: a torre do nexo volta. Mecânica do jogo
    pub baron_buff: f64,           // s de duração do buff do Barão. Mecânica do jogo
    pub elder_buff: f64,           // s de duração do buff do Ancião. Mecânica do jogo
}
```

Os valores de `Timings::default()` (`timeline.rs:48`). São regras do jogo
conferidas à mão contra o patch 16, e mudam de temporada em temporada. Este é o
lugar deles nos docs. O doccheck não os compara, porque vivem num `impl Default`
e não em `const`.

| Campo | Valor | mm:ss | Linha |
|---|---|---|---|
| `dragon_first` | `300.0` | 5:00 | `timeline.rs:51` |
| `dragon_respawn` | `300.0` | 5:00 | `timeline.rs:52` |
| `elder_respawn` | `360.0` | 6:00 | `timeline.rs:53` |
| `baron_first` | `1200.0` | 20:00 | `timeline.rs:54` |
| `baron_respawn` | `360.0` | 6:00 | `timeline.rs:55` |
| `herald_spawn` | `900.0` | 15:00 | `timeline.rs:56` |
| `grubs_spawn` | `480.0` | 8:00 | `timeline.rs:57` |
| `dragons_for_soul` | `4` | — | `timeline.rs:58` |
| `inhibitor_respawn` | `300.0` | 5:00 | `timeline.rs:59` |
| `nexus_turret_respawn` | `180.0` | 3:00 | `timeline.rs:60` |
| `baron_buff` | `180.0` | 3:00 | `timeline.rs:61` |
| `elder_buff` | `150.0` | 2:30 | `timeline.rs:62` |

**Invariantes:**
- A casca só usa `Timings::default()` (`watcher.rs:298`, `watcher.rs:300`). Nada
  lê outro valor de disco ou da rede.
- O tipo é `Serialize`, mas nada o envia ao front, e não há espelho em `src/`.
- Os testes trocam uma constante de cada vez (`mudar_a_constante_nao_quebra_a_logica`):
  a lógica não depende do valor.

### 29.2 `RawEvent`

JSON cru da Live Client: um item de `Events` em `/liveclientdata/eventdata`.

```rust
pub struct RawEvent {
    #[serde(rename = "EventName", default)]
    pub name: String,        // tipo do evento: `DragonKill`, `BaronKill`, `HeraldKill`, `VoidgrubKill`/`HordeKill`, `TurretKilled`, `InhibKilled`, `ChampionKill`, `GameEnd`
    #[serde(rename = "EventTime", default)]
    pub time: f64,           // s de partida em que aconteceu
    #[serde(rename = "DragonType", default)]
    pub dragon_type: String, // só no `DragonKill`: `Fire`, `Ocean`, `Elder`…
    #[serde(rename = "KillerName", default)]
    pub killer: String,      // nome de invocador de quem deu o golpe, sem a tag; tropa vem como `Minion_…`
    #[serde(rename = "TurretKilled", default)]
    pub turret: String,      // só no `TurretKilled`: nome da torre (`Turret_TOrder_L1_P5`, `Turret_T1_C_01_A`)
    #[serde(rename = "InhibKilled", default)]
    pub inhibitor: String,   // só no `InhibKilled`: nome do inibidor (`Inhib_TChaos_L1`, `Barracks_T2_C1`)
    #[serde(rename = "VictimName", default)]
    pub victim: String,      // só no `ChampionKill`: nome de invocador de quem morreu
    #[serde(rename = "Result", default)]
    pub result: String,      // só no `GameEnd`: `Win` ou `Lose`, do ponto de vista do jogador local
}
```

**Invariantes:**
- Todo campo tem `default`: um evento sem o campo vira texto vazio ou `0.0`, e
  não derruba a leitura.
- A Live Client devolve todos os eventos desde o começo da partida, a cada
  leitura. `build` e `BuffTracker::update` recontam tudo, toda vez.
- Os dois formatos de nome de estrutura são aceitos ([P-14](../produto/perguntas-em-aberto.md)).
- **Nenhum teste trava estes nomes de campo da Riot.** Os testes montam
  `RawEvent` direto em Rust, sem passar por JSON.

### 29.3 `EventFeed`

JSON cru da Live Client: a resposta inteira de `/liveclientdata/eventdata`.

```rust
pub struct EventFeed {
    #[serde(rename = "Events", default)]
    pub events: Vec<RawEvent>, // todos os eventos da partida, em ordem de tempo
}
```

**Invariantes:**
- Se a chamada falhar, a casca usa `EventFeed::default()` (`src-tauri/src/live.rs:72`): a
  partida segue sem timers.
- Nenhum teste trava o nome `Events`.

### 29.4 `Sides` ↔ `Sides`

```rust
pub struct Sides {
    pub ally: u32,  // contagem do seu time
    pub enemy: u32, // contagem do time inimigo
}
```

**Invariantes:**
- Só conta o evento cujo `killer` é um dos jogadores da partida, pelo nome
  inteiro ou pelo nome antes do `#`. Torre que cai por tropa não conta para
  ninguém.

### 29.5 `Spawn` ↔ `Spawn`

```rust
#[serde(rename_all = "camelCase")]
pub struct Spawn {
    pub name: String,      // `Dragão`, `Ancião`, `Barão`, `Arauto` ou `Grubs`, escrito por `build`
    pub at: f64,           // s de partida do spawn: abate visto + respawn, ou a constante de `Timings`
    pub seconds_left: f64, // s até o spawn: `at − game_time`, cortado em 0
    pub observed: bool,    // `true` quando sai de um abate visto; `false` quando é a constante
}
```

**Invariantes:**
- `seconds_left ≥ 0`. Um objetivo que já nasceu e está vivo fica em `0`.
- `observed = false` só sai da constante: primeiro dragão, primeiro Barão,
  Arauto e Grubs.

### 29.6 `Timeline` ↔ `Timeline`

```rust
#[serde(rename_all = "camelCase")]
pub struct Timeline {
    pub game_time: f64,            // s de partida, cópia de `LiveGame::game_time`
    pub dragons: Sides,            // dragões abatidos por lado, Ancião incluído
    pub dragon_types: Vec<String>, // `DragonType` de cada abate, em ordem de evento
    pub soul: Option<String>,      // `"ally"` ou `"enemy"` quando um lado chega a `dragons_for_soul`
    pub turrets: Sides,            // torres derrubadas por lado
    pub kills: Sides,              // abates de campeão por lado
    pub upcoming: Vec<Spawn>,      // próximos spawns, do mais próximo ao mais distante
    pub respawns: Vec<Respawn>,    // inibidores e torres do nexo esperando voltar, por `at`
}
```

**Invariantes:**
- `upcoming` sai em ordem crescente de `seconds_left`.
- `upcoming` sempre traz um dragão (`Dragão`, ou `Ancião` com alma fechada) e o
  `Barão`. `Arauto` e `Grubs` saem quando são tomados ou 300 s depois de nascer.
- `respawns` só traz estrutura com `at > game_time`.
- No TS, `respawns` é opcional (`respawns?`); o Rust sempre o envia.

### 29.7 `Respawn` ↔ `Respawn`

```rust
#[serde(rename_all = "camelCase")]
pub struct Respawn {
    pub name: String,      // `Inibidor do topo`, `Inibidor do meio`, `Inibidor do bot` ou `Torre do nexo`
    pub ours: bool,        // a estrutura é do seu time: o `ORDER`/`CHAOS` do nome contra o seu
    pub at: f64,           // s de partida da volta: queda + `inhibitor_respawn` ou `nexus_turret_respawn`
    pub seconds_left: f64, // s até voltar, sempre > 0
}
```

**Invariantes:**
- Nome de estrutura que nenhum dos dois formatos reconhece não vira `Respawn`.
- Torre que não é do nexo não volta, e não entra.

### 29.8 `Buff` ↔ `Buff`

```rust
#[serde(rename_all = "camelCase")]
pub struct Buff {
    pub name: String,               // `Barão` ou `Ancião`
    pub ours: bool,                 // o time de quem deu o golpe é o seu
    pub started: f64,               // s de partida: `EventTime` do abate
    pub until: f64,                 // s de partida: `started + baron_buff` ou `+ elder_buff`
    pub seconds_left: f64,          // s até acabar: `until − game_time`, refeito a cada leitura
    pub holders: Option<Vec<i64>>,  // ids do Data Dragon de quem ainda tem; `None` se o abate foi lido mais de 5 s depois
}
```

**Invariantes:**
- `BuffTracker::update` só devolve buff com `until > game_time` e com `holders`
  `None` ou não vazio.
- Quem morre depois de `started` sai de `holders`, mesmo que já tenha renascido
  na leitura seguinte.

### 29.9 `BuffTracker`

```rust
pub struct BuffTracker {
    buffs: Vec<Buff>,                // buffs das leituras anteriores; privado
    encerrados: Vec<(String, f64)>,  // (nome, início, em segundos de jogo) dos buffs que acabaram porque todos os donos morreram; privado. Impede que o mesmo abate os recrie (A8)
}
```

O único timer com estado. A casca guarda um por partida e o zera fora dela
(`watcher.rs:162`, `watcher.rs:332`).

## 30. `core::engine::priority` — Onde está o valor no mapa

O ranking das três rotas a cada leitura da Live Client, a partir de quem está
morto, das tropas, dos itens e dos próximos objetivos. Ele diz onde está o
valor, nunca onde o inimigo está ([03 §7](03-motor-da-partida.md#7-prioridade)).

### 30.1 `LanePriority` ↔ `LanePriority`

```rust
#[serde(rename_all = "camelCase")]
pub struct LanePriority {
    pub lane: String,         // `TOP`, `MID` ou `BOT`, escrito à mão na tabela privada `LANES`
    pub score: f64,           // placar 0–100, uma casa decimal: peso de exibição, não probabilidade
    pub skirmish_odds: f64,   // fração 0–1: chance de ganhar a luta na rota, de `build::skirmish_odds`
    pub reasons: Vec<String>, // até 3 frases; a primeira é sempre `NvM a seu favor em X%`
}
```

**Invariantes:**
- `score` está em [0, 100], arredondado a 0,1.
- `skirmish_odds` é 0,5 quando nenhum dos lados tem ninguém vivo, e 1,0 quando só
  o seu lado tem.
- Nenhuma razão usa linguagem de posição (`nenhuma_razao_revela_posicao`, C-4).

### 30.2 `PriorityReport` ↔ `PriorityReport`

```rust
#[serde(rename_all = "camelCase")]
pub struct PriorityReport {
    pub lanes: Vec<LanePriority>, // as três rotas, do maior `score` ao menor
    pub focus: Option<String>,    // o objetivo mais próximo a até 90 s: `Dragão em 43s`, com ` (estimado)` quando não é observado
}
```

**Invariantes:**
- `lanes` é vazio quando não há jogador local. Senão, tem exatamente três.
- Empate de `score` sai pela ordem do nome: `BOT`, `MID`, `TOP`.

## 31. `core::engine::coach` — Leitura pós-partida

A gravação da partida, quadro a quadro, e a leitura que sai dela. Tudo fica na
máquina do jogador. `Recording` é salvo em **`gravacao.json`**, e `Report` em
**`ultima-partida.json`**, os dois no fim da partida (`recorder.rs:18`,
`recorder.rs:19`, `recorder::finish`). A casca relê `ultima-partida.json` ao
abrir (`src-tauri/src/lib.rs:636`). `gravacao.json` não é relido por nada.

O que cada rota cobra é a função privada `cobranca`. A tabela por rota está em
[03 §8](03-motor-da-partida.md#8-leitura-pós-partida), e o motivo em
[L-27](09-licoes-do-prototipo.md).

### 31.1 `Recording` — `gravacao.json`

```rust
pub struct Recording {
    pub frames: Vec<LiveGame>, // um quadro por leitura da Live Client (~1 s), `game_time` estritamente crescente
    pub won: Option<bool>,     // do evento `GameEnd` (`RawEvent::outcome`); `None` até lá
}
```

**Invariantes:**
- `game_time` cresce estritamente entre quadros: `Recording::push` descarta
  quadro repetido ou que volta no tempo.
- A casca guarda no máximo 6000 quadros, e não salva gravação com menos de 60
  (`recorder.rs:23`, `recorder.rs:27`).
- `won` não é reescrito depois de conhecido (`recorder.rs:34`).
- Cada quadro tem o formato de `LiveGame` (10 §4).

### 31.2 `Verdict`

No front, o espelho não tem nome: é a união `"good" | "bad"` dentro de
`CoachNote.verdict`.

```rust
#[serde(rename_all = "camelCase")]
pub enum Verdict {
    Good, // `good`: a manter
    Bad,  // `bad`: a corrigir
}
```

### 31.3 `Note` ↔ `CoachNote`

```rust
#[serde(rename_all = "camelCase")]
pub struct Note {
    pub verdict: Verdict, // em que lista a observação entra
    pub title: String,    // a frase com o número: `5.0 tropas por minuto contra 8.6`
    pub detail: String,   // o que o número quer dizer, com outra conta que dá para conferir
    pub weight: f64,      // fração 0–1, só para ordenar dentro de `good` ou de `bad`; não aparece como nota
}
```

**Invariantes:**
- A única observação estimada é o ouro parado, e o `detail` dela diz isso.
- `weight` fica em [0, 1], com uma exceção: o elogio de item grande antes de
  4:20 passa de 1.

### 31.4 `Point` ↔ `Point`

```rust
pub struct Point {
    pub t: f64, // s de partida: `game_time` do quadro
    pub v: f64, // valor na unidade da curva: ouro, tropas, pontos de visão ou níveis
}
```

### 31.5 `Series` ↔ `Series`

```rust
#[serde(rename_all = "camelCase")]
pub struct Series {
    pub gold_lead: Vec<Point>,   // ouro em itens do seu time menos o deles (soma dos preços, exato), a cada 30 s
    pub cs_diff: Vec<Point>,     // suas tropas menos as do oponente direto (`creepScore`), a cada 30 s
    #[serde(default)]
    pub vision_diff: Vec<Point>, // seu placar de visão menos o do oponente (`wardScore`), a cada 30 s
    pub level_diff: Vec<Point>,  // seu nível menos o do oponente, 1–18, a cada 30 s
    pub deaths: Vec<f64>,        // s de partida de cada morte sua: o quadro em que o placar subiu
    pub team_kills: Vec<f64>,    // s de partida de cada abate do seu time
}
```

**Invariantes:**
- Dois pontos seguidos de uma curva ficam a pelo menos 30 s um do outro.
  `deaths` e `team_kills` guardam o instante do quadro, sem reamostrar.
- `cs_diff`, `vision_diff` e `level_diff` têm o mesmo tamanho, e só ganham ponto
  quando há oponente direto.
- `Series::is_empty` olha só `gold_lead`. A curva sai vazia quando a leitura
  não roda (sem gravação ou com menos de 300 s).
- `vision_diff` tem `default` para ler arquivo salvo antes dela existir.

### 31.6 `Report` ↔ `CoachReport` — `ultima-partida.json`

```rust
#[serde(rename_all = "camelCase")]
pub struct Report {
    pub champion: String,    // `champion_key` do jogador local no último quadro; vazio sem jogador local
    #[serde(default = "sem_rota")]
    pub role: Role,          // `Role::from_lcu` da posição no último quadro; `Unknown` em arquivo antigo
    pub duration: f64,       // s: a duração da partida, o `game_time` do último quadro, mesmo quando a gravação começou tarde (A9)
    pub won: Option<bool>,   // cópia de `Recording::won`
    pub good: Vec<Note>,     // a manter, da mais pesada para a menos
    pub bad: Vec<Note>,      // a corrigir, da mais pesada para a menos
    pub summary: String,     // `30:00 de Ahri (MID) · 2 a corrigir, 3 a manter.`, ou o motivo de não haver leitura
    #[serde(default)]
    pub series: Series,      // as curvas por trás das observações
}
```

**Invariantes:**
- Sem jogador local, sem quadros ou com menos de 300 s gravados, `good` e `bad`
  saem vazios e `summary` diz por quê.
- `role` e `series` têm `default`: um `ultima-partida.json` de antes da rota
  ainda carrega (`leitura_salva_antes_da_rota_ainda_carrega`).
- Nada aqui é nota ou pontuação do jogador (C-7).

## 32. `casca` — Os tipos que a casca serializa

A casca monta poucos tipos próprios. Quase tudo o que ela devolve é tipo do core
(`Scored`, `LaneReport`, `GoldReport`…), descrito na seção do módulo dele. Aqui
ficam os quatro `#[derive(Serialize)]` de `src-tauri/src`, e o `Shared`, que não
cruza a ponte mas está por trás de todo comando. Quem pede e quem recebe cada um
está no contrato da casca (12 §2 e 12 §3).

### 32.1 `EngineStatus` ↔ `EngineStatus`

Em `lib.rs:47`, privado. Sai de `engine_status_of` (`lib.rs:61`), pelo comando
`engine_status` e pelo evento `presage://engine`.

```rust
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct EngineStatus {
    ready: bool,                   // índice de campeões não vazio: o Data Dragon (ou o cache) carregou
    version: String,               // patch do Data Dragon de onde vieram os campeões, p.ex. "14.21.1"; vazio antes de carregar
    champions: usize,              // contagem de campeões no índice
    has_global: bool,              // há base de partidas em uso (`Layered::has_global`)
    has_personal: bool,            // o histórico pessoal tem ao menos um campeão (Riot Web API)
    personal_games: u32,           // contagem de partidas no histórico pessoal
    account: Option<String>,       // Riot ID `nome#tag` da conta do LCU; `None` antes de conectar
    global_patch: Option<String>,  // patch da base de partidas (`Dataset::patch`); `None` sem base
    global_matches: u64,           // contagem de partidas da base; 0 sem base
}
```

**Invariantes:**
- `ready` é `champions > 0`: os dois saem de `ChampionIndex::by_id` no mesmo
  instante (`lib.rs:67`).
- `global_patch` é `Some` exatamente quando `has_global`: `Shared::dataset` só
  recebe base que passou em `Dataset::is_usable` (`lib.rs:623`).
- `account` nunca leva o puuid: só o `riot_id` de `Identity` (`lib.rs:78`).

### 32.2 `PoolPlan` ↔ `PoolPlan`

Em `lib.rs:346`, privado. Sai do comando `pool_suggestions`.

```rust
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct PoolPlan {
    analysis: poolgap::Analysis,          // fortes e fracos dos core da rota (10 §17); vazio sem core
    candidates: Vec<poolgap::Candidate>,  // até `limit` (6 por padrão) para o próximo lugar, por cobertura (10 §17)
}
```

**Invariantes:**
- `analysis` olha só os core da rota (`ids_for_role_tier(role, Tier::Core)`), e
  `candidates` olha a pool inteira da rota (`ids_for_role`) (`lib.rs:365`).

### 32.3 `LiveSnapshot` ↔ `LiveSnapshot` (`casca::live`)

Em `live.rs:19`. Montada pelo watcher a cada volta de `InProgress`
(`watcher.rs:303`), guardada em `Shared::live` e emitida em `presage://live`. Os
cinco campos saem da mesma leitura da Live Client, para não divergirem na tela.

```rust
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveSnapshot {
    pub game: LiveGame,            // os dez jogadores, de `to_live_game` sobre `/playerlist` (10 §4)
    pub timeline: Timeline,        // timers de objetivo, de `timeline::build` sobre `/eventdata` (10 §29)
    pub priority: PriorityReport,  // onde está o valor no mapa, de `priority::report` (10 §30)
    pub buffs: Vec<Buff>,          // Barão e Ancião em andamento, de `BuffTracker::update` (10 §29); vazio sem buff
    pub rival: Option<Rival>,      // você contra o oponente direto, de `rival::report` (10 §22); `None` antes de 90 s ou sem oponente
}
```

**Invariantes:**
- `presage://live` só sai quando a nova é diferente da anterior (`PartialEq`,
  `watcher.rs:317`).
- `BuffTracker` guarda estado entre leituras. Ele é zerado quando a fase sai de
  `InProgress` (`watcher.rs:332`), então `buffs` não atravessa partidas.
- No TS, `buffs` e `rival` são opcionais (`buffs?`, `rival?`). O Rust sempre
  manda os dois.

### 32.4 `Identity` (`casca::watcher`)

Em `watcher.rs:54`. A conta logada, lida do LCU ao conectar, ou da Riot Web API
quando só há `PRESAGE_RIOT_ID`. Deriva `Serialize`, mas nada a serializa: não tem
espelho em TS, e o front só recebe o `riot_id`, por `EngineStatus::account`.

```rust
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Identity {
    pub puuid: String,    // id da conta na Riot: LCU `/lol-summoner/v1/current-summoner`, ou account-v1
    pub riot_id: String,  // `nome#tag`, montado de `gameName` e `tagLine`; só o nome quando não há tag
    pub region: String,   // região do LCU (`/riot/v1/region-locale`, p.ex. "BR"), ou `PRESAGE_REGION`; "BR" quando a leitura falha
}
```

**Invariantes:**
- Só existe com `puuid` não vazio (`watcher.rs:186`).
- `profile::refresh` espera por ela até 120 s, de 2 em 2 s (`profile.rs:29`), antes
  de cair para a variável de ambiente.

### 32.5 `Shared` (`casca::watcher`)

Em `watcher.rs:60`. Não cruza a ponte. É o estado que o watcher, as tarefas de
carga e os 26 comandos dividem; quem escreve e quem lê cada campo está em 12 §1.

```rust
pub struct Shared {
    pub status: Mutex<Option<ConnStatus>>,       // conexão e fase do fluxo; `None` antes da primeira volta
    pub draft: Mutex<Option<DraftState>>,        // champ select do cliente; `None` fora dele
    pub champions: RwLock<ChampionIndex>,        // Data Dragon; vazio até a carga
    pub items: RwLock<ItemIndex>,                // Data Dragon; vazio até a carga
    pub runes: RwLock<RuneIndex>,                // Data Dragon; vazio até a carga
    pub live: Mutex<Option<LiveSnapshot>>,       // a última leitura da partida; `None` fora dela
    pub stats: RwLock<Arc<dyn StatsStore>>,      // `NoStats` ao nascer, `Layered` depois do setup
    pub identity: Mutex<Option<Identity>>,       // conta do LCU; `None` até conectar
    pub pool: RwLock<Pool>,                      // `pool.json`, ou vazia
    pub overlay_prefs: RwLock<OverlayPrefs>,     // `overlay.json`, ou tudo visível
    pub recording: Mutex<Recording>,             // a partida em andamento, um quadro por volta (~1 s)
    pub last_review: Mutex<Option<Report>>,      // `ultima-partida.json`, depois a leitura da partida que acabou
    pub dataset: RwLock<Option<Dataset>>,        // `base-de-partidas.json` usável, ou `None`
    pub draft_role: Mutex<Option<Role>>,         // rota escolhida na tela; `None` volta para a da pool
    pub enemy_roles: Mutex<HashMap<i64, Role>>,  // id do Data Dragon -> rota marcada; esvazia com o champ select
    pub owned: Mutex<Option<HashSet<i64>>>,      // ids que a conta pode escolher; `None` até a grade responder
    pub draft_prefs: RwLock<DraftPrefs>,         // `draft.json`, ou o padrão
    pub simulated: Mutex<Option<DraftState>>,    // draft do simulador, já normalizado; `None` fora da aba
}
```

**Invariantes:**
- `Shared::new` e `Default` dão o estado vazio. O `setup` (`lib.rs:613`) enche
  `pool`, `dataset`, `stats`, `last_review`, `draft_prefs` e `overlay_prefs` a
  partir do disco antes de `app.manage`.
- `stats` é trocado inteiro, nunca editado: `rebuild_stats` (`lib.rs:89`) monta um
  `Layered` novo com a base atual e o histórico dado, e uma camada não apaga a
  outra.

## 33. Arquivos em disco

Tudo em `%LOCALAPPDATA%\Presage`. O app lê e grava por `storage.rs`: `write_json`
grava o arquivo inteiro com `serde_json::to_vec` (JSON compacto, sem arquivo
temporário), e `read_json` devolve `None` quando o arquivo falta, não abre ou não
desserializa. Para quem chama, falta e corrupção são a mesma coisa, e nenhuma
deixa log. A lista de quem escreve e quem lê cada arquivo está em 12 §6.

**Nenhum arquivo tem campo de versão do esquema.** O que existe no lugar:

- **Nome novo:** `personal-v2.json` substituiu `personal.json` quando o formato
  ganhou a comparação de rota ([L-9](09-licoes-do-prototipo.md)).
- **Campo com `default`:** um arquivo antigo, sem o campo novo, ainda carrega.
  É o caso de `Pool::bans`, `PersonalStats::lanes`, `Report::role`,
  `Report::series` e de toda a `Dataset`.
- **Campo obrigatório:** quando um campo sem `default` falta, ou uma variante de
  enum não é conhecida, o arquivo inteiro vira `None`.

### Os arquivos do app

| Arquivo | Tipo | Chaves no JSON | Quando falta | Quando não desserializa |
|---|---|---|---|---|
| `pool.json` | `Pool` (10 §7) | `entries` (lista de `{championId, role, tier}`), `bans` (ids) | pool vazia | pool vazia; o próximo `set_pool` sobrescreve o arquivo |
| `overlay.json` | `OverlayPrefs` (10 §8) | `hidden` (lista de `Section`, `"nextItem"`) | tudo visível | tudo visível, inclusive com uma seção que esta versão não conhece |
| `draft.json` | `DraftPrefs` (10 §8) | `showAllChampions` | filtro da conta ligado | idem |
| `base-de-partidas.json` | `Dataset` (10 §20) | `patch`, `region`, `tier`, `generatedAt`, `matches`, e as tabelas `base`, `matchup`, `synergy`, `versus`, `length` (chave texto → `{games, wins}`) | sem base: os componentes estatísticos valem zero | sem base. JSON válido mas truncado carrega pelos `default`, e `is_usable` (`matches > 0` e `base` não vazia) decide se serve |
| `personal-v2.json` | `PersonalStats` (10 §19) | `puuid`, `by_champion` (id → `{games, wins}`), `seen` (ids de partida), `lanes` (lista de `LaneGame`) | histórico refeito do zero na próxima atualização | idem; e um `puuid` diferente do da conta também zera o histórico (`profile.rs:48`) |
| `gravacao.json` | `Recording` (10 §31) | `frames` (lista de `LiveGame`), `won` | — (o app não lê) | — |
| `ultima-partida.json` | `Report` (10 §31) | `champion`, `role`, `duration`, `won`, `good`, `bad`, `summary`, `series` | aba Pós-jogo vazia até a próxima partida | idem |
| `champions.json` | `ChampionIndex` (10 §23) | `version`, `by_id` (id → `Champion`, campos em snake_case) | sem rede, motor inativo | idem |
| `items.json` | `ItemIndex` (10 §24) | `version`, `by_id` (id → `Item`, camelCase) | sem rede, build adaptativa inativa | idem |
| `runas.json` | `RuneIndex` (10 §18) | `version`, `by_key` (chave textual → `Rune`, camelCase) | sem rede, runas inativas | idem |

Como ler a tabela:

- **Caixa das chaves.** Ela não é a mesma em todo arquivo: `PersonalStats`,
  `ChampionIndex` e `Champion` não têm `rename_all` e gravam em snake_case
  (`by_champion`, `by_id`). Os outros gravam em camelCase. Não quebra nada,
  porque quem lê cada arquivo é o mesmo tipo que o gravou.
- **Mapa com chave numérica.** `HashMap<i64, _>` vira objeto JSON com o id em
  texto (`"103"`).
- **Chaves da base.** As de `Dataset` são texto montado com o `Debug` de `Role`:
  `"103:Mid"`, `"103:7:Mid"` (10 §20).
- **O `version` dos índices** é o patch do Data Dragon, não versão de esquema.
- **Os índices do Data Dragon** são regravados a cada abertura com rede e só são
  lidos quando a rede falha. `ChampionIndex::version` é o que a interface mostra
  como patch, então um cache velho aparece como patch velho, sem outro aviso.

### O que cada arquivo guarda de pessoa

- **`personal-v2.json`** guarda o puuid do jogador.
- **`gravacao.json`** guarda o `displayName` dos dez jogadores em cada quadro: até
  6000 quadros de dez nomes. É sobrescrito a cada partida que passa de 60 quadros.
- **Nenhum dos dois sai da máquina** (RNF-8,
  [07 §1](07-seguranca.md#1-ativos)).

### O arquivo do coletor

| Arquivo | Tipo | Chaves no JSON | Quando falta | Quando não desserializa |
|---|---|---|---|---|
| `coleta-em-andamento.json` | `Estado`, privado de `tools/ingest/src/main.rs:163`, `#[serde(default)]` | `regiao`, `tier`, `fila`, `jogadores` (puuids semeados), `tiers_semeados`, `proximo`, `vistas` (ids de partida), `contagens` | `--retomar` começa do zero | idem |

O coletor recusa retomar um estado de outro recorte, ou seja, outra região, tier
ou fila (`Estado::combina`). Também recusa um estado de versão sem as tabelas de
contra e duração (`Counts::resumable`).

Ele grava os dois arquivos ao lado, em `.parcial`, e renomeia
(`tools/ingest/src/main.rs:189`): uma interrupção não deixa um arquivo pela
metade. O app não faz isso
([05 §1](05-coletor.md#1-uso)).
