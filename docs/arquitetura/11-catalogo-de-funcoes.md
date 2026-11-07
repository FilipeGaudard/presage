# 11 — Catálogo de funções

Cada função e cada método público do `presage-core`: assinatura, o que faz, o
que devolve, quando falha, a confiança do número que produz e o requisito que
atende. As seções têm a numeração do [10](10-modelo-de-dados.md), onde estão os
tipos. Os comandos que a casca expõe ao front estão no
[12](12-contrato-da-casca.md).

Este doc é contrato ([ADR-0020](../adr/0020-o-doc-e-o-contrato-e-o-doccheck-prova.md)):

- o `doccheck` compara cada assinatura daqui com a do código, sem espaços;
- uma função pública nova sem entrada aqui falha o commit;
- métodos de `impl Trait for Tipo` (`Default`, `StatsStore`…) não entram, a não
  ser quando o trait é do próprio core.

## 1. Como ler

### Os campos de cada entrada

| Campo | O que diz |
|---|---|
| **Faz** | uma ou duas frases, ou os passos |
| **Params** | só os que não são óbvios, com unidade |
| **Retorno** | o que volta, com unidade |
| **Erros** | `None` ou `Err` e quando; "nenhum" quando não falha |
| **Confiança** | a etiqueta do número que vai para a tela: exata, alta, fraca ou regra ([README](../README.md#confiança-do-número)) |
| **Atende** | os requisitos (R-, C-, RNF-), ADRs e lições. A [matriz](../produto/requisitos.md#4-matriz-de-rastreabilidade) sai desta coluna |

Toda função daqui é `core`: pura, sem I/O, sem relógio, sem aleatoriedade.
Determinismo é parte do contrato: a mesma entrada dá a mesma ordem, e há teste
para os desempates.

### Quem chama quem

```text
casca (src-tauri)                       core
─────────────────                       ────
lcu ──▶ draft::to_draft_state ──▶ DraftState
                                      │
                   engine::context::DraftContext::new_with ◀── pool, prefs, marcas
                                      │
          ┌───────────────┬───────────┼──────────────┬─────────────────┐
     engine::rank_with  minimax  report::from_context  runes::plan   poolgap
          │               │           │ (lanes, team, wincon)
          └── stats (shrink, dataset, PersonalStats) ◀── profile, history
live ──▶ live::to_live_game ──▶ LiveGame ──▶ build::suggest, lanes, gold::report,
                                             timeline::build, priority::report, rival
recorder ──▶ coach::Recording ──▶ coach::review ──▶ Report
```

## 2. `core::lockfile` — Credenciais do cliente (`core`, pura)

### 2.1 `Credentials`

```rust
impl Credentials {
    pub fn base_url(&self) -> String;
}
```

| Método | Faz | Atende |
|---|---|---|
| `base_url` | `https://127.0.0.1:<porta>`. Sempre o loopback, nunca `localhost` | R-18 |

### 2.2 `parse_lockfile`

```rust
pub fn parse_lockfile(contents: &str) -> Option<Credentials>;
```

- **Faz:** corta o texto por `:` no formato `LeagueClient:<pid>:<porta>:<senha>:https`
  e pega o 3º campo como porta e o 4º como senha. Não confere o nome do
  processo nem o protocolo.
- **Retorno:** `Credentials`.
- **Erros:** `None` com menos de quatro campos ou porta que não cabe em `u16`.
  Senha vazia passa.
- **Atende:** R-18

### 2.3 `parse_cmdline`

```rust
pub fn parse_cmdline(cmdline: &str) -> Option<Credentials>;
```

- **Faz:** tira `--app-port=` e `--remoting-auth-token=` da linha de comando do
  `LeagueClientUx.exe`, com ou sem aspas.
- **Erros:** `None` sem um dos dois, com valor vazio ou com porta inválida.
- **Atende:** R-18. A casca não chama esta função: `lcu/credentials.rs` só lê o
  `lockfile`.

### 2.4 `extract_arg`

```rust
pub fn extract_arg(haystack: &str, key: &str) -> Option<String>;
```

- **Faz:** acha a primeira ocorrência de `key` (com o `=`) e lê até aspas ou
  espaço.
- **Erros:** `None` sem a chave ou com valor vazio.
- **Atende:** R-18

## 3. `core::draft` — Champ select: o JSON do LCU e o draft do domínio (`core`, pura)

### 3.1 `to_draft_state`

```rust
pub fn to_draft_state(session: &LcuSession) -> DraftState;
```

- **Faz:**
  1. Achata os turnos de `actions`.
  2. `ban` concluído com campeão > 0 vai para `my_bans` se `is_ally_action`,
     senão para `their_bans`.
  3. `pick` concluído com campeão > 0 fica guardado por célula. É daqui que sai
     o campeão do time deles, que vem zerado no jogador
     ([L-10](09-licoes-do-prototipo.md)).
  4. Ação em andamento e não concluída marca a célula como agindo. O `type` da
     primeira dessas vira `current_action`.
  5. Cada `LcuPlayer` vira `DraftSlot`: `champion_id` do jogador quando > 0,
     senão o do `pick` concluído; `locked` só com `pick` concluído.
  6. `local_position` é a do slot local. `unavailable` junta bans e campeões dos
     slots, ordenado, sem repetição e sem 0.
- **Erros:** nenhum. Sessão vazia dá draft vazio, sem turno local.
- **Atende:** R-1, R-18, ADR-0017, L-10

### 3.2 `DraftState`

```rust
impl DraftState {
    pub fn normalized(mut self) -> Self;
}
```

| Método | Faz | Atende |
|---|---|---|
| `normalized` | Refaz `unavailable` pela mesma conta de `to_draft_state`, e `local_cell_id` e `local_position` pelo slot com `is_local`. Sem slot local, os dois ficam como vieram. O simulador chama sobre o draft que o front montou | R-8 |

### 3.3 `selectable`

```rust
pub fn selectable(grid: &[LcuGridChampion], queue_id: Option<i64>) -> HashSet<i64>;
```

- **Faz:** os ids da grade com `id > 0` que a conta tem (`owned`) ou que o Game
  Pass libera (`xbox_gp_reward`). A rotação grátis (`free_to_play`) só entra
  com fila conhecida, maior que 0 e fora de `RANKED_QUEUES`.
- **Params:** `queue_id` — o `id` de `/lol-gameflow/v1/session`; `None` quando a
  casca não conseguiu ler.
- **Retorno:** ids do Data Dragon. Grade vazia dá conjunto vazio: que lista vazia
  não tire ninguém é decisão de quem chama (`02 §0`).
- **Erros:** nenhum. Fila desconhecida fica do lado seguro, sem a rotação.
- **Atende:** R-5

## 4. `core::live` — A partida ao vivo (`core`, pura)

### 4.1 `to_live_game`

```rust
pub fn to_live_game(
    raw: &[RawPlayer],
    active: &str,
    game_time: f64,
    champions: &ChampionIndex,
) -> LiveGame;
```

- **Faz:**
  1. Tira os bots.
  2. A chave do campeão sai de `raw_champion_name` sem o prefixo, e o id, do
     `ChampionIndex` por essa chave.
  3. `display_name` é o `riot_id`, ou o `summoner_name` quando ele vem vazio.
  4. `gold_invested` soma o `price` de cada item.
  5. `is_local` quando `active` não é vazio e casa com `riot_id` ou com
     `summoner_name`: `/activeplayername` devolve um ou outro conforme a versão
     do cliente.
- **Params:** `active` — a resposta de `/activeplayername`. `game_time` —
  segundos, de `/gamestats`.
- **Retorno:** `LiveGame`, com os humanos na ordem da resposta.
- **Erros:** nenhum. Campeão que o índice não conhece fica com id 0.
- **Confiança:** exata (`gold_invested`)
- **Atende:** R-10, R-11, R-12

### 4.2 `champion_key`

```rust
pub fn champion_key(raw_champion_name: &str) -> &str;
```

- **Faz:** `game_character_displayname_Annie` → `Annie`. Sem o prefixo, devolve o
  texto como veio.
- **Atende:** R-11

### 4.3 `LiveGame`

```rust
impl LiveGame {
    pub fn me(&self) -> Option<&LivePlayer>;
    pub fn allies(&self) -> Vec<&LivePlayer>;
    pub fn enemies(&self) -> Vec<&LivePlayer>;
    pub fn gold_invested(&self) -> (f64, f64);
}
```

| Método | Faz | Atende |
|---|---|---|
| `me` | O primeiro jogador com `is_local`, ou `None` | R-11 |
| `allies` | O seu time sem você. Vazio sem jogador local | R-11 |
| `enemies` | Quem tem `team` diferente do seu. Vazio sem jogador local, em vez de chutar um lado | R-11 |
| `gold_invested` | (seu time, o deles), em ouro: a soma de `gold_invested` de cada lado. Sem jogador local, (0, 0). Confiança: exata | R-12, C-4 |

### 4.4 `LivePlayer`

```rust
impl LivePlayer {
    pub fn as_combatant(&self) -> Combatant;
}
```

| Método | Faz | Atende |
|---|---|---|
| `as_combatant` | O `Combatant` do motor de build: id do campeão, nível e itens | R-10 |

## 5. `core::history` — A partida da match-v5 e o oponente direto (`core`, pura)

### 5.1 `played`

```rust
pub fn played(game: &MatchDto, puuid: &str) -> Result<PlayedGame, Skip>;
```

- **Faz:**
  1. Acha o participante de `puuid`.
  2. Aplica os cortes, nesta ordem: modo `CLASSIC`, `endOfGameResult` vazio ou
     `GameComplete`, ninguém com `gameEndedInEarlySurrender`, duração de pelo
     menos 300 s. Valem também para vitória e derrota
     ([L-5](09-licoes-do-prototipo.md)).
  3. Rota pelo `team_position` (`Role::from_lcu`). Em branco: a partida conta,
     sem `lane`.
  4. O oponente direto é quem tem o mesmo `team_position` no outro time. Nenhum,
     ou dois: sem `lane`.
  5. Para cada métrica de `Metric::ALL` que os dois lados têm, você menos ele.
     `Cs10` lê `jungle_cs_before_10` para o caçador e nada para o suporte;
     participação e fatia de dano viram pp.
- **Retorno:** `PlayedGame`, com `lane` quando há oponente direto.
- **Erros:** `Err(Skip::Ausente)` sem o jogador; depois `Modo`, `Terminada`,
  `Remake`, `Curta`, na ordem do passo 2.
- **Atende:** R-17, R-20, ADR-0010, L-5, L-6

### 5.2 `Metric`

```rust
impl Metric {
    pub fn higher_is_better(self) -> bool;
    pub fn phrase(self, diff: f64) -> String;
}
```

| Método | Faz | Atende |
|---|---|---|
| `higher_is_better` | `false` só para `Deaths`. Nenhum código do produto chama | R-17 |
| `phrase` | A diferença em texto para uma razão do draft, com sinal: `+12 tropas aos 10`, `+5pp de participação em abates`, `+5pp da fatia de dano`, `+0.3 de visão por minuto`, `-1.5 mortes por partida`. Inteiro nas três primeiras, uma casa nas outras duas | R-1, R-17, ADR-0011 |

## 6. `core::riot` — Endereçamento da Riot Web API (`core`, pura)

### 6.1 `Region`

```rust
impl Region {
    pub fn from_lcu(region: &str) -> Region;
    pub fn from_platform(platform: &str) -> Option<Region>;
    pub fn platforms() -> impl Iterator<Item = &'static str>;
}
```

| Método | Faz | Atende |
|---|---|---|
| `from_lcu` | A região que o LCU dá (`BR`, `EUW`, `OCE`…, sem distinguir maiúscula) → plataforma e host regional. Região desconhecida cai em `br1`/`americas`. A tabela não tem o Oriente Médio, que `PLATFORMS` tem (`me1`) | R-17 |
| `from_platform` | O host de plataforma (`br1`, `euw1`, `kr`, sem distinguir maiúscula) → `Region`. Desconhecido dá `None`, nunca um padrão: no coletor, região errada agrega partidas de outro servidor sob o rótulo errado | R-20 |
| `platforms` | Os 17 hosts de plataforma, na ordem da tabela. O coletor os lista no erro de `--regiao` | R-20 |

### 6.2 Riot ID

```rust
pub fn split_riot_id(riot_id: &str) -> Option<(String, String)>;
pub fn urlencode(value: &str) -> String;
```

| Função | Faz | Atende |
|---|---|---|
| `split_riot_id` | `nome#tag` → (`nome`, `tag`), pela **última** `#`, sem espaço nas pontas. `None` sem `#` ou com nome ou tag vazios | R-17 |
| `urlencode` | Percent-encoding byte a byte do UTF-8: só `A–Z a–z 0–9 - _ . ~` passam. Espaço vira `%20`, nunca `+` | R-17 |

## 7. `core::pool` — A pool do jogador (`core`, pura)

### 7.1 `Tier`

```rust
impl Tier {
    pub fn weight(self) -> u8;
}
```

| Método | Faz | Atende |
|---|---|---|
| `weight` | A ordem de exibição dentro da pool: `Core` 0, `Situational` 1, `Extra` 2. Menor ganha em `tier_of` | R-9 |

### 7.2 `Pool`

```rust
impl Pool {
    pub fn is_empty(&self) -> bool;
    pub fn ids(&self) -> Vec<i64>;
    pub fn ids_for_role(&self, role: Role) -> Vec<i64>;
    pub fn upsert(&mut self, entry: PoolEntry);
    pub fn remove(&mut self, champion_id: i64, role: Option<Role>);
    pub fn ids_for_role_tier(&self, role: Role, tier: Tier) -> Vec<i64>;
    pub fn tier_of(&self, champion_id: i64, role: Role) -> Option<Tier>;
}
```

| Método | Faz | Atende |
|---|---|---|
| `is_empty` | Sem nenhuma entrada. A lista de bans não conta | R-9 |
| `ids` | Todos os campeões da pool, ordenados e sem repetição | R-9 |
| `ids_for_role` | Os marcados na rota mais os sem rota, ordenados e sem repetição | R-1, R-9 |
| `upsert` | Insere, ou troca o tier se o par (campeão, rota) já existe | R-9 |
| `remove` | Tira só a entrada daquele par (campeão, rota) | R-9 |
| `ids_for_role_tier` | Os de um tier marcados **naquela** rota, sem os curingas, na ordem de inserção | R-9, ADR-0016 |
| `tier_of` | O tier mais forte do campeão entre a entrada da rota e a sem rota; `None` fora da pool | R-1, R-9 |

## 8. `core::prefs` — O que o jogador quer ver (`core`, pura)

### 8.1 `Section`

```rust
impl Section {
    pub fn label(self) -> &'static str;
    pub fn detail(self) -> &'static str;
}
```

| Método | Faz | Atende |
|---|---|---|
| `label` | O nome da seção em pt-BR (`Ouro em itens`, `Próximo item`…). Só os testes chamam: a tela usa a cópia em `src/lib/types.ts` | R-15 |
| `detail` | A explicação de uma linha. Mesma cópia no TS | R-15 |

### 8.2 `OverlayPrefs`

```rust
impl OverlayPrefs {
    pub fn shows(&self, section: Section) -> bool;
    pub fn set(&mut self, section: Section, visible: bool);
    pub fn anything_visible(&self) -> bool;
    pub fn visible(&self) -> Vec<Section>;
}
```

| Método | Faz | Atende |
|---|---|---|
| `shows` | A seção não está em `hidden` | R-15 |
| `set` | Tira a seção de `hidden` e, se `visible` é falso, põe de volta no fim. Nunca duplica | R-15 |
| `anything_visible` | Alguma seção de `Section::ALL` aparece. Com tudo desligado, quem chama decide o que fazer com a janela | R-15 |
| `visible` | As seções que aparecem, na ordem de `Section::ALL` | R-15 |

## 9. `core::engine` — Ranking do draft (`core`, pura)

### 9.1 `rank`

```rust
pub fn rank(
    draft: &DraftState,
    index: &ChampionIndex,
    store: &dyn StatsStore,
    pool: &[i64],
) -> Vec<Scored>;
```

- **Faz:** monta o `DraftContext` com a rota que o cliente informa
  (`Role::from_lcu(local_position)`), sem marcas e sem lista da conta, e chama
  `rank_with`. O app não a usa: a casca resolve a rota antes e chama `rank_with`.
- **Params:** `pool`, ids do Data Dragon que passam por cima da regra de rota.
- **Retorno:** o mesmo de `rank_with`.
- **Erros:** nenhum. Índice vazio devolve vazio.
- **Confiança:** fraca
- **Atende:** R-1

### 9.2 `rank_with`

```rust
pub fn rank_with(ctx: &DraftContext, index: &ChampionIndex, store: &dyn StatsStore) -> Vec<Scored>;
```

- **Faz:** para cada id de `ctx.candidates` que o índice conhece
  ([02 §1](02-motor-de-draft.md#1-rank)):
  1. a winrate base na sua rota, encolhida para 50% com `BASE_STRENGTH`;
  2. o matchup contra `ctx.lane_opponent()`, medido contra `expected_against` das
     duas bases ([02 §3](02-motor-de-draft.md#3-a-referência)), com peso `w`;
  3. o contra de cada inimigo travado, medido contra a força de cada um somada nas
     cinco rotas, com peso `1 − w` para o oponente de rota e 1 para os outros
     ([02 §1](02-motor-de-draft.md#contra-o-time-inimigo-inteiro));
  4. a sinergia com cada aliado, hover inclusive, contra `expected_together` com a
     base do aliado na rota dele;
  5. o componente pessoal, contra a base do próprio candidato;
  6. a composição: `W_DAMAGE_BALANCE × damage_balance_gain`, mais
     `W_FRONTLINE × frontline_need` se ele é frontline
     ([02 §4](02-motor-de-draft.md#4-fit-de-composição));
  7. soma tudo em log-odds, `winrate = σ(z)` cortada em [0,25; 0,75]; a variância
     soma a de cada componente ([02 §2](02-motor-de-draft.md#incerteza));
  8. monta até três razões ([02 §5](02-motor-de-draft.md#5-componente-pessoal-e-razões)),
     e a lista sai ordenada por `winrate`, depois pool, depois id.
- **Params:** `store` é a loja em camadas (`Layered`) na casca, e `NoStats` nos
  testes. O peso `w` vale 1 com a posição do inimigo informada ou marcada, e
  `lane_share` (a fração do encaixe dele na sua rota) quando inferida.
- **Retorno:** um `Scored` por candidato que o índice conhece.
- **Erros:** nenhum. Índice vazio devolve vazio; candidato fora do índice é pulado.
- **Confiança:** fraca. Sem base, a ordem é só composição, com pesos à mão; com a
  base, os espalhamentos dos priors continuam escritos à mão.
- **Atende:** R-1, R-21, RNF-1, RNF-2, RNF-3, ADR-0007, ADR-0008, ADR-0009,
  ADR-0011, ADR-0012, ADR-0019, L-1, L-2, L-23

## 10. `core::engine::context` — Contexto do draft (`core`, pura)

### 10.1 `resolve_role`

```rust
pub fn resolve_role(local_position: &str, chosen: Option<Role>, pool: &Pool) -> Role;
```

- **Faz:** devolve a rota do cliente; senão `chosen`, a escolhida no seletor do
  painel (`set_draft_role`); senão a rota da pool, quando todas as entradas com
  rota têm a mesma.
- **Params:** `local_position` é o `DraftState::local_position` do LCU (`top`,
  `jungle`, `middle`, `bottom`, `utility`). `chosen` igual a `Unknown` é ignorado.
  Entradas da pool sem rota não contam.
- **Retorno:** a rota, ou `Role::Unknown` quando nada decide (pool vazia ou de
  mais de uma rota).
- **Erros:** nenhum.
- **Atende:** R-1, ADR-0017

### 10.2 `DraftContext`

```rust
impl DraftContext {
    pub fn new(
        draft: &DraftState,
        index: &ChampionIndex,
        store: &dyn StatsStore,
        pool: &[i64],
        my_role: Role,
    ) -> Self;
    pub fn new_with(
        draft: &DraftState,
        index: &ChampionIndex,
        store: &dyn StatsStore,
        pool: &[i64],
        my_role: Role,
        leitura: &Leitura,
    ) -> Self;
    pub fn with_candidate(&self, champion_id: i64) -> Self;
    pub fn lane_opponent(&self) -> Option<&Placed>;
    pub fn open_enemy_roles(&self) -> Vec<Role>;
    pub fn ally_ids(&self) -> Vec<i64>;
    pub fn enemy_ids(&self) -> Vec<i64>;
    pub fn lineups(&self, level: u8) -> (Vec<Lineup>, Vec<Lineup>);
}
```

| Método | Faz | Atende |
|---|---|---|
| `new` | `new_with` com `Leitura::default()`: sem marcas e sem lista da conta. Usam: `rank`, `minimax` e os testes | ADR-0017 |
| `new_with` | 1. `me`: o campeão do slot local ou, sem ele, o hover. 2. `allies`: os outros slots do seu time com campeão ou hover, com a posição do cliente. 3. `enemies`: os slots deles com `champion_id > 0`. 4. Aplica as marcas de `leitura.enemy_roles`, na ordem dos slots, a quem está sem rota e numa rota ainda livre. 5. Infere a rota de quem sobrou: aliados com `my_role` reservada, inimigos à parte. A inferência faz uma busca exaustiva das atribuições das rotas livres (no máximo 120), maximiza a soma de `ln(role_fit + EPS)` e fica com a primeira melhor em ordem lexicográfica. 6. `unavailable`: bans, aliados, inimigos, e o seu só travado. 7. `candidates`: ids do índice fora de `unavailable`, dentro de `owned` quando ele não é vazio, e que jogam `my_role` (`roles::plays`) ou estão na pool | R-1, R-4, R-5, ADR-0017, L-23, L-25, L-26, P-12, P-13 |
| `with_candidate` | Cópia com `me = Some(champion_id)`, e o resto igual. É a prévia: a mesma conta de quando ele estiver travado | R-6, ADR-0017 |
| `lane_opponent` | O primeiro inimigo com `role == my_role`. `None` com `my_role` desconhecida ou sem ninguém nela | R-1, ADR-0019, L-25 |
| `open_enemy_roles` | As rotas de `ROLES`, nessa ordem, que nenhum inimigo ocupa, seja a rota informada, marcada ou inferida. É de onde a resposta do minimax sai | R-2, L-24 |
| `ally_ids` | Os ids de `allies`, na ordem dos slots, hover inclusive | R-1 |
| `enemy_ids` | Os ids de `enemies`, na ordem dos slots | R-2 |
| `lineups` | Os dois lados no nível dado, sem itens: os aliados com rota, você em `my_role` quando há `me` e rota, os inimigos com rota. Quem está em `Unknown` fica de fora. A prévia usa `PREGAME_LEVEL` | R-6 |

## 11. `core::engine::roles` — Quem joga a rota (`core`, pura)

### 11.1 `role_share`

```rust
pub fn role_share(champion: i64, store: &dyn StatsStore) -> Option<[f64; 5]>;
```

- **Faz:** com a base ligada (`has_global`), divide as partidas do campeão em cada
  rota de `ROLES` pelo total dele.
- **Params:** `champion` é o id do Data Dragon.
- **Retorno:** frações 0–1 na ordem de `ROLES`, que somam 1.
- **Erros:** `None` sem a base ou sem nenhuma partida do campeão nela.
- **Atende:** R-4, R-21, ADR-0017

### 11.2 `role_fit`

```rust
pub fn role_fit(champion: &Champion, role: Role, store: &dyn StatsStore) -> f64;
```

- **Faz:** com `role_share`, devolve a fração da rota. Sem ela, olha as classes: 1
  se a classe principal (a primeira tag) conta para a rota, 0,5 se só uma
  secundária conta, 0 se nenhuma. As classes de cada rota são as de
  [02 §7](02-motor-de-draft.md#quem-joga-a-rota); o suporte exige a classe Suporte.
- **Retorno:** 0–1. `Role::Unknown` dá 0.
- **Erros:** nenhum.
- **Atende:** R-4, ADR-0017, L-25, P-13

### 11.3 `plays`

```rust
pub fn plays(champion: &Champion, role: Role, store: &dyn StatsStore) -> bool;
```

- **Faz:** **você** joga o campeão nesta rota? Decide nesta ordem:
  1. rota desconhecida: sim;
  2. você já jogou o campeão ali (`played_in`): sim;
  3. senão, `others_play`.
- **Retorno:** verdadeiro quando o campeão joga a rota.
- **Erros:** nenhum.
- **Quem usa:** os seus candidatos (`DraftContext`) e a sugestão da sua pool
  (`poolgap::suggest`).
- **Atende:** R-1, R-9, L-22, L-23

### 11.4 `others_play`

```rust
pub fn others_play(champion: &Champion, role: Role, store: &dyn StatsStore) -> bool;
```

- **Faz:** **alguém que não é você** joga o campeão nesta rota? Rota desconhecida:
  sim; com `role_share`, a fração da rota é ao menos `ROLE_SHARE`; senão,
  `role_fit > 0`, pela classe. O seu histórico fica de fora.
- **Retorno:** verdadeiro quando o campeão joga a rota.
- **Erros:** nenhum.
- **Quem usa:** as respostas deles no `minimax_with`. O que você já jogou no topo
  não diz nada do que o outro time escolhe (A12, T-DRAFT-64).
- **Atende:** R-2, L-4, L-24

## 12. `core::engine::minimax` — Draft com resposta e bans (`core`, pura)

### 12.1 `team_value`

```rust
pub fn team_value(ours: &[i64], theirs: &[i64], index: &ChampionIndex) -> f64;
```

- **Faz:** calcula `0,5 + penalidade(theirs) − penalidade(ours)`, cortado em
  [0,05; 0,95]. A penalidade de um time é `|physical_share − 0,5| × 2 × W_BALANCE`,
  mais `W_FRONTLINE` quando nenhum é frontline e o time tem ao menos
  `FRONTLINE_FROM` campeões conhecidos, tudo vezes `min(known / 5, 1)`. Time sem campeão conhecido
  não tem penalidade.
- **Params:** ids do Data Dragon; os que o índice não conhece são ignorados.
- **Retorno:** fração 0–1. É simétrica: `team_value(A, B) + team_value(B, A) = 1`.
  Um índice de composição, não uma probabilidade medida.
- **Erros:** nenhum.
- **Confiança:** fraca
- **Atende:** R-2, R-3, ADR-0013, P-11

### 12.2 `minimax`

```rust
pub fn minimax(draft: &DraftState, index: &ChampionIndex, pool: &[i64], breadth: usize) -> Vec<MinimaxPick>;
```

- **Faz:** monta o contexto com a rota do cliente, `NoStats` e sem `Leitura`, e
  chama `minimax_with`. O app não a usa.
- **Retorno:** o mesmo de `minimax_with`.
- **Erros:** nenhum.
- **Confiança:** fraca
- **Atende:** R-2

### 12.3 `minimax_with`

```rust
pub fn minimax_with(
    ctx: &DraftContext,
    index: &ChampionIndex,
    store: &dyn StatsStore,
    breadth: usize,
) -> Vec<MinimaxPick>;
```

- **Faz:** ([02 §6](02-motor-de-draft.md#6-draft-com-resposta))
  1. as respostas possíveis: todo id do índice fora de `ctx.unavailable` que joga
     (`roles::plays`) alguma rota de `open_enemy_roles`;
  2. o guloso: `team_value(aliados + candidato, inimigos)` para cada candidato do
     contexto, ordenado por valor e depois id, cortado em `max(breadth, 1)`;
  3. para cada um, põe cada resposta (menos ele mesmo) no time deles e guarda o
     menor valor e quem o deu;
  4. ordena por `after_response`, depois id.
- **Params:** `breadth` é quantos candidatos passam da poda; a casca usa 12
  (`src-tauri/src/lib.rs:226`). `store` só decide quem joga a rota, e o valor não
  o lê.
- **Retorno:** até `breadth` itens.
- **Erros:** nenhum. Índice vazio devolve vazio.
- **Confiança:** fraca
- **Atende:** R-2, C-5, RNF-4, L-24

### 12.4 `suggest_bans`

```rust
pub fn suggest_bans(
    draft: &DraftState,
    index: &ChampionIndex,
    limit: usize,
) -> Vec<BanSuggestion>;
```

- **Faz:** quando o outro lado tem algum campeão, calcula para cada id fora de
  `draft.unavailable` `threat = team_value(nós, eles) − team_value(nós, eles + id)`.
  Ordena por `threat` decrescente, depois id, e corta em `limit`
  ([02 §6](02-motor-de-draft.md#por-que-o-ban-ainda-não-é-pessoal)).
- **Params:** lê o `DraftState` direto, não o `DraftContext`. "Nós" é o seu time
  sem o slot local; um slot vale o travado ou, sem ele, o hover.
- **Retorno:** até `limit` itens, todos com `from_pool = false`.
- **Erros:** vazio sem campeão do outro lado ou com índice vazio.
- **Atende:** R-3, ADR-0007, ADR-0013

### 12.5 `bans_with_pool`

```rust
pub fn bans_with_pool(
    draft: &DraftState,
    index: &ChampionIndex,
    ban_pool: &[i64],
    limit: usize,
) -> Vec<BanSuggestion>;
```

- **Faz:** ([02 §6](02-motor-de-draft.md#a-sua-lista-de-bans))
  1. da sua lista, em ordem e sem repetir: quem tem id > 0, não está em
     `draft.unavailable` nem nos bans dos dois lados, e não é do seu time (travado
     ou em hover, você inclusive);
  2. depois, `suggest_bans` com limite folgado, com os mesmos filtros e sem repetir
     a lista;
  3. corta em `limit`.
- **Params:** `ban_pool` é `Pool::bans`, até cinco ids em ordem. `limit` é 10 na
  casca (`src-tauri/src/lib.rs:237`).
- **Retorno:** a lista primeiro (`from_pool`, `threat = 0`), a composição depois.
  Vale desde o primeiro ban, quando `suggest_bans` ainda devolve vazio.
- **Erros:** nenhum.
- **Atende:** R-3, C-5, ADR-0013

## 13. `core::engine::comp` — Perfil de composição (`core`, pura)

### 13.1 `profile`

```rust
pub fn profile(ids: &[i64], index: &ChampionIndex) -> TeamProfile;
```

- **Faz:** tira a média de `Champion::physical_share` e conta frontline e alcance
  dos ids que o índice conhece.
- **Retorno:** um `TeamProfile`. Sem nenhum id conhecido, `physical_share = 0,5` e
  as contagens são zero.
- **Erros:** nenhum.
- **Confiança:** fraca. O "74%" de "Equilibra o dano: 74% → 59% físico" sai daqui.
- **Atende:** R-1, R-6

### 13.2 `damage_balance_gain`

```rust
pub fn damage_balance_gain(team: &TeamProfile, candidate: &Champion) -> f64;
```

- **Faz:** mede o quanto o candidato aproxima a média física do time de 0,5:
  `|antes − 0,5| − |depois − 0,5|`, vezes `min(known / 4, 1)`.
- **Retorno:** uma fração. É positiva quando corrige um time unilateral, e zero ou
  negativa quando mantém ou piora. Time vazio dá 0.
- **Erros:** nenhum.
- **Atende:** R-1

### 13.3 `frontline_need`

```rust
pub fn frontline_need(team: &TeamProfile) -> f64;
```

- **Faz:** devolve `min(known / 4, 1)` quando o time não tem frontline, e 0 quando
  tem ou está vazio. Uma frontline basta.
- **Retorno:** 0–1.
- **Erros:** nenhum.
- **Atende:** R-1

### 13.4 `share_after`

```rust
pub fn share_after(team: &TeamProfile, candidate: &Champion) -> f64;
```

- **Faz:** dá a média física do time com o candidato, ou a do candidato quando o
  time está vazio.
- **Retorno:** fração 0–1. É o "59%" da razão de equilíbrio de dano.
- **Erros:** nenhum.
- **Confiança:** fraca
- **Atende:** R-1

## 14. `core::engine::team` — A chance do time (`core`, pura)

A conta está em [02 §1](02-motor-de-draft.md#a-chance-do-time); as decisões, no
[ADR-0019](../adr/0019-contra-em-qualquer-rota-e-chance-do-time.md).

### 14.1 `seats`

```rust
pub fn seats(ctx: &DraftContext, index: &ChampionIndex, store: &dyn StatsStore) -> (Vec<Seat>, Vec<Seat>);
```

- **Faz:**
  1. Cada aliado com rota vira assento, com certeza 1 se a rota veio do cliente
     ou da sua marca, e `lane_share` se foi inferida.
  2. Você (`ctx.me`) entra com certeza 1 quando a sua rota é conhecida.
  3. Cada inimigo com rota vira assento pela mesma regra do passo 1.
- **Params:** `store` só serve a `lane_share`: o encaixe pela base quando há,
  pelas classes quando não.
- **Retorno:** `(nossos, deles)`. Quem está em `Role::Unknown` fica de fora.
- **Erros:** nenhum.
- **Atende:** R-4, ADR-0019, L-25

### 14.2 `estimate`

```rust
pub fn estimate(
    ours: &[Seat],
    theirs: &[Seat],
    index: &ChampionIndex,
    store: &dyn StatsStore,
) -> Option<TeamEstimate>;
```

- **Faz:**
  1. A força de cada um na rota: `logit` da base encolhida (`PRIOR_RATE`,
     `BASE_STRENGTH`), somada para `ours` e subtraída para `theirs`.
  2. Cada par de adversários: o matchup de rota com peso `w = certeza ×
     certeza` quando os dois estão na mesma rota (0 nas outras), contra
     `expected_against` das duas forças na rota; e o contra (`versus`) com peso
     `1 − w`, contra `expected_against` das forças somadas nas cinco rotas.
  3. Cada par de companheiros: a sinergia, contra `expected_together`, a favor
     do lado que a tem.
  4. Cada efeito entra como `logit(referência + δ) − logit(referência)`, e a
     variância de cada parcela vem de `posterior_sd` (ou do `se()` da base).
  5. `win = σ(z)` presa em [0,35; 0,65]; o `±` é `1,96 × √Σ variância`, em pp;
     ficam as quatro parcelas de maior |shift|.
- **Params:** `ours` é o lado de quem a chance fala; trocar os dois dá o
  complemento.
- **Retorno:** `TeamEstimate` (10 §14.3).
- **Erros:** `None` sem `store.has_global()` ou com um dos lados vazio.
- **Confiança:** fraca — as parcelas são medidas, mas o piso, o teto e os
  desvios de prior são escolhidos à mão até o `--verificar` medir.
- **Atende:** R-6, R-21, RNF-1, RNF-2, RNF-3, ADR-0007, ADR-0019

## 15. `core::engine::wincon` — Condição de vitória (`core`, pura)

O que a condição de vitória pode afirmar está no
[ADR-0018](../adr/0018-o-que-a-condicao-de-vitoria-pode-afirmar.md) e em
[03 §4](03-motor-da-partida.md#condição-de-vitória).

### 15.1 `analyze`

```rust
pub fn analyze(
    ours: &[Lineup],
    theirs: &[Lineup],
    lanes: &[LaneOutlook],
    skirmishes: &[Skirmish],
    index: &ChampionIndex,
    store: &dyn StatsStore,
) -> WinCondition;
```

- **Faz:**
  1. **Fatos**, para cada time com 3 ou mais conhecidos (`comp::profile`): 70%
     ou mais do dano de um tipo (`ONE_SIDED`); ninguém, ou 3 ou mais, na linha
     de frente; 4 ou mais à distância, ou 4 ou mais corpo a corpo.
  2. **Vantagens:** cada rota com `|duel − 0,5| ≥ 0,05` (`EDGE`) e cada
     escaramuça com `|odds − 0,5| ≥ 0,05`, a favor de quem passa de 50%, com a
     porcentagem do lado que vence. Ficam as 5 mais distantes de 50%.
  3. **Fase**, só com a base: a inclinação de um time é a média de
     `logit(longa) − logit(curta)` dos seus, cada taxa encolhida para a força
     do campeão na rota (`BASE_STRENGTH`). Diferença acima de 0,08 log-odds
     (`LEAN`, ~2 pontos) vira "partidas longas", abaixo de −0,08 "partidas
     curtas", no meio "nenhum dos dois pende".
  4. O aviso fixo `CAVEAT`.
- **Params:** `lanes` e `skirmishes` chegam prontos de `lanes::lane_outlooks` e
  `lanes::skirmishes`; a conta de duelo não é refeita aqui. Na fase, campeão
  sem dado de duração fica de fora da média; time inimigo sem nenhum entra com
  inclinação 0.
- **Retorno:** `WinCondition` (10 §15.4).
- **Erros:** nenhum. Time com menos de três conhecidos fica sem fatos; sem a
  base, `phase = None`.
- **Confiança:** regra — fatos e limiares escritos à mão; as vantagens herdam a
  conta de duelo ([03 §4](03-motor-da-partida.md#4-chance-por-rota-e-escaramuça)).
- **Atende:** R-6, R-11, R-21, C-5, RNF-1, ADR-0018

## 16. `core::engine::report` — A análise de rota inteira (`core`, pura)

A prévia no draft está em [03 §4](03-motor-da-partida.md#a-prévia-no-draft).

### 16.1 `build`

```rust
pub fn build(
    ours: &[Lineup],
    theirs: &[Lineup],
    source: &'static str,
    roles_assumed: bool,
    champions: &ChampionIndex,
    items: &ItemIndex,
    store: &dyn StatsStore,
) -> Option<LaneReport>;
```

- **Faz:** rotas (`lanes::lane_outlooks`), escaramuças (`lanes::skirmishes`) e
  condição de vitória (`wincon::analyze`) sobre os dois lados já resolvidos,
  com `team = None`.
- **Params:** `source` é `"draft"` ou `"partida"` e vai direto para a tela;
  `roles_assumed` vem de quem chama, que sabe se alguma rota foi inferida. A
  casca chama com `"partida"` (`analysis.rs`).
- **Retorno:** `LaneReport` (10 §16.1).
- **Erros:** `None` com um dos lados vazio.
- **Atende:** R-6, R-11

### 16.2 `from_context`

```rust
pub fn from_context(
    ctx: &DraftContext,
    champions: &ChampionIndex,
    items: &ItemIndex,
    store: &dyn StatsStore,
) -> Option<LaneReport>;
```

- **Faz:**
  1. Os dois lados do contexto no `PREGAME_LEVEL` (6), sem itens
     (`DraftContext::lineups`).
  2. `build` com `"draft"` e `ctx.roles_assumed`.
  3. A chance do time: `team::seats` e `team::estimate`.
- **Params:** a prévia de um candidato é esta função com
  `ctx.with_candidate(id)`, a mesma conta de quando ele estiver travado.
- **Retorno:** `LaneReport` com `team` preenchido quando há base.
- **Erros:** `None` quando `build` devolve `None`: um dos lados sem ninguém com
  rota (o seu lado conta você).
- **Atende:** R-6, R-21, ADR-0019, L-25

## 17. `core::engine::poolgap` — Pool de oito (`core`, pura)

O retrato e a cobertura estão em [02 §7](02-motor-de-draft.md#7-pool); as
decisões, no
[ADR-0016](../adr/0016-pool-de-oito-tres-escolhidos-cinco-por-cobertura.md). Os
pesos por rota são a função privada `weight` (`poolgap.rs:183`).

### 17.1 `Trait`

```rust
impl Trait {
    pub fn has(self, champion: &Champion) -> bool;
}
```

| Método | Faz | Atende |
|---|---|---|
| `Trait::has` | Se o campeão tem a característica: `physical_share` ≥ 0,65 ou ≤ 0,35, `is_frontline`, `is_ranged`, `robustness` ≥ 0,55; estilo é ter a classe em qualquer posição de `tags` | R-9, ADR-0016 |

### 17.2 `robustness`

```rust
pub fn robustness(champion: &Champion) -> f64;
```

- **Faz:** a vida efetiva no nível 11, sem itens, contra dano todo físico e todo
  mágico (`vida / (100 / (100 + resistência))`, a fórmula do jogo); devolve a
  menor dividida pela maior.
- **Retorno:** fração 0–1. Perto de 1, tanto faz o tipo de dano que vem; perto
  de 0, forte contra um e frágil contra o outro. Não é taxa de derrota para
  counter ([02 §7](02-motor-de-draft.md#o-que-não-é)).
- **Erros:** `0.0` quando alguma das duas vidas efetivas não é finita, ou a
  maior é ≤ 0.
- **Atende:** R-9, ADR-0016

### 17.3 `analyze`

```rust
pub fn analyze(core: &[i64], champions: &ChampionIndex, role: Role) -> Analysis;
```

- **Faz:**
  1. Para cada característica que não é estilo e que a rota pesa 0,5 ou mais
     (`RELEVANT`), conta quantos core a têm.
  2. Forte: as que ao menos um tem, por peso e depois por contagem. Fraco: as
     que nenhum tem.
  3. Com dois ou mais core da mesma classe principal, soma o fraco "um estilo
     só".
- **Params:** `core` são os ids dos core da rota (na casca,
  `Pool::ids_for_role_tier(role, Tier::Core)`). Id fora do índice é ignorado.
- **Retorno:** `Analysis` (10 §17.4).
- **Erros:** nenhum; sem campeão conhecido, `Analysis::default()`.
- **Confiança:** regra — pesos por rota escritos à mão.
- **Atende:** R-9, ADR-0016, L-22

### 17.4 `suggest`

```rust
pub fn suggest(
    pool: &[i64],
    champions: &ChampionIndex,
    role: Role,
    store: &dyn StatsStore,
    limit: usize,
) -> Vec<Candidate>;
```

- **Faz:**
  1. Conta quantos da pool têm cada uma das 12 características (`n`).
  2. Cada campeão fora da pool que joga a rota (`roles::plays`) vale
     `Σ weight(rota, t) × ½ⁿ⁺¹` sobre o que ele tem
     ([02 §7](02-motor-de-draft.md#os-cinco-um-lugar-por-vez)); com ganho 0,
     sai.
  3. `score` = o ganho dele ÷ a soma dos ganhos de todas as características.
  4. Ordena por `score` decrescente, depois nome, e corta em `limit`.
- **Params:** `pool` é a pool inteira da rota, core e situacionais
  (`Pool::ids_for_role`). `store` só serve a `plays`
  ([02 §7](02-motor-de-draft.md#quem-joga-a-rota)). A casca pede `limit = 6`.
- **Retorno:** até `limit` candidatos (10 §17.5).
- **Erros:** lista vazia com a pool sem campeão conhecido ou com 8 ids ou mais
  (`POOL_SIZE`).
- **Confiança:** regra — `score` é índice de encaixe, não winrate.
- **Atende:** R-9, C-5, RNF-1, ADR-0016, L-22

### 17.5 `role_label`

```rust
pub fn role_label(role: Role) -> &'static str;
```

- **Faz:** devolve `Role::short`: `TOP`, `JG`, `MID`, `ADC`, `SUP` ou `—`.
  Nenhum código chama hoje.
- **Erros:** nenhum.
- **Atende:** 02 §7

## 18. `core::engine::runes` — Runas do confronto (`core`, pura)

As regras e a chave textual estão em [03 §9](03-motor-da-partida.md#9-runas).

### 18.1 `RuneIndex`

```rust
impl RuneIndex {
    pub fn get(&self, key: &str) -> Option<&Rune>;
    pub fn is_empty(&self) -> bool;
}
```

| Método | Faz | Atende |
|---|---|---|
| `RuneIndex::get` | A runa pela chave textual do Data Dragon (`Conqueror`); `None` quando a chave sumiu do patch | R-7, L-17 |
| `RuneIndex::is_empty` | Verdadeiro antes da primeira carga, ou quando o Data Dragon e o cache `runas.json` falharam | R-7 |

### 18.2 `plan`

```rust
pub fn plan(
    mine: i64,
    opponent: i64,
    role: Role,
    champions: &ChampionIndex,
    runes: &RuneIndex,
) -> Option<RunePlan>;
```

- **Faz:**
  1. Passa as 13 regras de `REGRAS` sobre (você, ele, rota). Cada regra que
     dispara, e cuja runa existe no índice, soma o seu peso à runa e o seu
     motivo às razões.
  2. Ordena por soma decrescente, depois nome; ficam 3 (`MAX_SUGGESTIONS`).
  3. A secundária é a primeira de `SECUNDARIAS` (`SecondWind`, `BonePlating`,
     `TasteOfBlood`) que casa, existe no índice e é de **outra árvore** que a
     primeira principal, com `score` 1. Sem nenhuma assim, `None`: o cliente não
     aceita principal e secundária da mesma árvore (A10).
  4. `from_data = false`.
- **Params:** `mine` e `opponent` são ids do Data Dragon da sua rota (na casca,
  `LaneOutlook::ally` e `LaneOutlook::enemy`). `role` é a sua rota; só
  `Aftershock` e `Guardian` olham para ela.
- **Retorno:** `RunePlan` (10 §18.4).
- **Erros:** `None` com o índice de runas vazio ou com um dos dois campeões fora
  do índice de campeões.
- **Confiança:** regra.
- **Atende:** R-7, C-5, C-6, RNF-1, L-17

## 19. `core::engine::stats` — Registro, shrinkage e a loja (`core`, pura)

Toda taxa que entra ou sai daqui é fração 0–1. O desenho está em
[02 §2](02-motor-de-draft.md#2-shrinkage) e [02 §3](02-motor-de-draft.md#3-a-referência).

### 19.1 `Record::new`

```rust
impl Record {
    pub fn new(games: u32, wins: u32) -> Self;
}
```

- **Faz:** monta o registro com `wins` cortado em `games`.
- **Erros:** nenhum. Vitórias acima de partidas viram `games`.
- **Atende:** 02 §2

### 19.2 `Estimate::se`

```rust
impl Estimate {
    pub fn se(&self) -> f64;
}
```

- **Faz:** erro padrão da proporção, `√(p(1 − p) / effective_n)`, com `p = rate`
  cortado em [0, 1]. É o `±` da winrate base.
- **Retorno:** fração. `0,5` se `effective_n ≤ 0`, o que `shrink` nunca produz.
- **Confiança:** fraca — a força da base é `BASE_STRENGTH`, escolhida à mão.
- **Atende:** RNF-2, ADR-0009

### 19.3 `shrink`

```rust
pub fn shrink(record: Option<Record>, prior: f64, strength: f64) -> Estimate;
```

- **Faz:** média posterior de uma Beta(`prior × strength`, `(1 − prior) × strength`)
  atualizada pela amostra: `(wins + strength × prior) / (games + strength)`.
- **Params:** `prior` em fração; `strength` em partidas fictícias, levada a no
  mínimo 1.
- **Retorno:** `rate` em [0, 1]; `effective_n = games + strength`. Sem registro,
  `rate == prior` e `games == 0`.
- **Erros:** nenhum.
- **Confiança:** fraca — sem base, a taxa é o prior escrito à mão (50%).
- **Atende:** RNF-1, R-21, ADR-0007

### 19.4 `shrunk_delta`

```rust
pub fn shrunk_delta(record: Option<Record>, baseline: f64, strength: f64) -> Estimate;
```

- **Faz:** `shrink` com `prior = baseline`, e devolve `rate − baseline`.
- **Params:** `baseline` é a referência, em fração (`expected_against`,
  `expected_together` ou a base do candidato).
- **Retorno:** `rate` é **desvio** em fração, não taxa. Sem registro, exatamente 0.
- **Atende:** RNF-1, RNF-3, ADR-0007, ADR-0008

### 19.5 `strength_for_sd`

```rust
pub fn strength_for_sd(sd: f64) -> f64;
```

- **Faz:** a força da Beta em torno de 50% cujo desvio é `sd`:
  `0,25 / sd² − 1`, no mínimo 1.
- **Params:** `sd` em fração (`0.030` são 3pp).
- **Retorno:** partidas fictícias. Com os espalhamentos de `engine/mod.rs`: 3pp →
  ≈ 277, 1,2pp → ≈ 1735, 3,5pp → ≈ 203, 1pp → ≈ 2499
  ([02 §2](02-motor-de-draft.md#força-e-desvio)).
- **Atende:** RNF-2, ADR-0009, L-2

### 19.6 `posterior_sd`

```rust
pub fn posterior_sd(games: u32, prior_sd: f64) -> f64;
```

- **Faz:** o desvio de um componente com `games` partidas:
  `√(0,25 / (strength_for_sd(prior_sd) + 1 + games))`.
- **Retorno:** fração. Com `games = 0`, exatamente `prior_sd`; depois cai com a raiz
  da amostra.
- **Confiança:** fraca — os espalhamentos são escolhidos à mão.
- **Atende:** RNF-2, ADR-0009, L-3

### 19.7 Log-odds e referência

```rust
pub fn logit(p: f64) -> f64;
pub fn logistic(x: f64) -> f64;
pub fn expected_against(champion: f64, opponent: f64) -> f64;
pub fn expected_together(champion: f64, ally: f64) -> f64;
```

| Função | Faz | Atende |
|---|---|---|
| `logit` | `ln(p / (1 − p))`, com `p` cortado em [10⁻⁶, 1 − 10⁻⁶]: nunca infinito | 02 §3 |
| `logistic` | `1 / (1 + e⁻ˣ)`, o inverso de `logit` | 02 §3 |
| `expected_against` | a winrate esperada de um contra o outro sem nada de especial no confronto: `σ(logit champion − logit opponent)`. Rival em 50% não muda nada | RNF-3, ADR-0008, L-1 |
| `expected_together` | a esperada de dois no mesmo time sem interação: `σ(logit champion + logit ally)`. Aliado em 50% não muda nada | RNF-3, ADR-0008, L-1 |

As quatro recebem e devolvem fração; `logit` devolve log-odds.

### 19.8 `Length::of`

```rust
impl Length {
    pub fn of(game_duration_secs: i64) -> Length;
}
```

- **Faz:** `< 1500` → `Curta`, `< 1950` → `Media`, o resto → `Longa`.
- **Params:** `game_duration_secs` é o `gameDuration` da match-v5, em segundos.
- **Atende:** R-21, ADR-0018

### 19.9 `PersonalStats`

```rust
impl PersonalStats {
    pub fn add_played(&mut self, match_id: &str, game: PlayedGame) -> bool;
    pub fn mark_seen(&mut self, match_id: &str);
    pub fn add_match(&mut self, match_id: &str, champion: i64, won: bool) -> bool;
    pub fn total_games(&self) -> u32;
}
```

| Método | Faz | Atende |
|---|---|---|
| `add_played` | soma a partida lida da match-v5 (`history::played`): vitória ou derrota sempre, e a `LaneGame` quando houver. `false` se a partida já foi vista | R-17, ADR-0010, L-6 |
| `mark_seen` | marca como vista a partida que não conta (remake, curta), para a próxima atualização não buscá-la. Não soma nada; repetir não duplica | R-17, L-5 |
| `add_match` | soma uma vitória ou derrota ao campeão e marca a partida como vista. `false`, sem somar, se já estava em `seen` | R-1, 02 §5 |
| `total_games` | soma de `games` sobre `by_champion` | R-1 |

### 19.10 Quem implementa `StatsStore`

As assinaturas estão em 10 §19.4. O que cada implementação responde:

| Método | `NoStats` | `PersonalStats` | `Dataset` | `Layered` |
|---|---|---|---|---|
| `base`, `matchup`, `synergy` | `None` | `None` | o mapa, pela chave de 11 §20.1 | da camada global |
| `versus`, `length` | `None` (padrão) | `None` (padrão) | o mapa, pela chave | da camada global |
| `personal` | `None` | `by_champion[id]` | `None` | da camada pessoal |
| `has_global` | `false` | `false` | `is_usable()` | `global.is_some()` |
| `has_personal` | `false` | `by_champion` não vazio | `false` | da camada pessoal |
| `personal_games` | `0` | `total_games()` | `0` | da camada pessoal |
| `lane_profile` | `None` | `profile::build(&lanes)`, ou `None` sem `lanes` | `None` | da camada pessoal |
| `played_in` | `false` | alguma `LaneGame` com o campeão na rota | `false` | da camada pessoal |

`lane_profile` recalcula o perfil inteiro a cada chamada: não há cache.

## 20. `core::engine::dataset` — A base de partidas (`core`, pura)

### 20.1 As chaves da base

```rust
pub fn base_key(champion: i64, role: Role) -> String;
pub fn matchup_key(champion: i64, opponent: i64, role: Role) -> String;
pub fn synergy_key(a: i64, b: i64) -> String;
pub fn versus_key(champion: i64, opponent: i64) -> String;
pub fn length_key(champion: i64, role: Role, bucket: Length) -> String;
```

| Função | Faz | Atende |
|---|---|---|
| `base_key` | `"103:Mid"`: id e o `Debug` da rota | R-21 |
| `matchup_key` | `"103:238:Mid"`: direcional, conta o resultado do primeiro | R-21 |
| `synergy_key` | `"64:103"`: o menor id sempre antes, para o par não ter ordem | R-21 |
| `versus_key` | `"103:64"`: direcional, o primeiro contra o segundo em qualquer rota inimiga | R-21, ADR-0019 |
| `length_key` | `"103:Mid:Longa"`: id, rota e faixa, pelo `Debug` | R-21, ADR-0018 |

O coletor grava com as mesmas funções (`aggregate.rs`), e o formato do arquivo está
em 10 §20.2.

### 20.2 `Dataset::is_usable`

```rust
impl Dataset {
    pub fn is_usable(&self) -> bool;
}
```

- **Faz:** `true` quando `matches > 0` e `base` não está vazio.
- **Retorno:** `false` para arquivo truncado ou de coletor que falhou no meio, mesmo
  que `matches` diga milhões.
- **Atende:** RNF-1, R-21, ADR-0007

### 20.3 `Layered`

```rust
impl Layered {
    pub fn with_global(mut self, dataset: Dataset) -> Self;
    pub fn patch(&self) -> Option<&str>;
    pub fn matches(&self) -> u64;
}
```

| Método | Faz | Atende |
|---|---|---|
| `with_global` | põe a base na camada global só se `is_usable()`; senão a camada fica `None` | RNF-1, R-21 |
| `patch` | o `patch` da base, ou `None` sem camada global | R-21 |
| `matches` | as partidas da base, ou `0` sem camada global | R-21 |

## 21. `core::engine::profile` — Perfil de rota (`core`, pura)

### 21.1 `build`

```rust
pub fn build(games: &[LaneGame]) -> LaneProfile;
```

- **Faz:**
  1. Para cada métrica de `Metric::ALL`, junta as diferenças das partidas que a
     têm. Com menos de cinco (`MIN_GAMES`), a métrica sai de todos os níveis.
  2. No geral: média e erro padrão (`s / √n`), sem encolher.
  3. O ruído de uma partida, `σ²`: a variância dentro de cada campeão, somada.
     Com menos de 2 graus de liberdade, entra a variância total.
  4. O espalhamento entre campeões, `τ²`, por DerSimonian–Laird
     (`between_variance`, um grupo só), e cada campeão encolhido para o geral.
  5. O espalhamento entre confrontos, um grupo por campeão, e cada confronto
     encolhido para o seu campeão.
  6. Campeões e confrontos em ordem de partidas, empate pelo menor id.
- **Params:** `games` são as `LaneGame` de `PersonalStats::lanes`, uma por partida.
- **Retorno:** o perfil; vazio com menos de cinco partidas em toda métrica.
- **Erros:** nenhum. Partidas todas iguais não dividem por zero (piso `EPS`).
- **Confiança:** exata no geral (medição direta); por campeão e confronto, a
  medição encolhida sem prior escrito à mão. Mistura você e o campeão
  ([04 §6](04-perfil-de-rota.md#6-o-que-ele-não-separa)).
- **Atende:** R-17, RNF-11, C-7, ADR-0010, L-6, L-19

As fórmulas de cada passo estão em
[04 §4](04-perfil-de-rota.md#quanto-encolhe-dersimonianlaird).

### 21.2 Leitura, campeão e perfil

```rust
impl Reading {
    pub fn is_clear(&self) -> bool;
}

impl ChampionLane {
    pub fn clearest(&self) -> Option<&Reading>;
}

impl LaneProfile {
    pub fn champion(&self, id: i64) -> Option<&ChampionLane>;
}
```

| Método | Faz | Atende |
|---|---|---|
| `Reading::is_clear` | `value` em valor absoluto acima de `1,96 × sd`: o intervalo de 95% não cruza o zero. Só o que é claro ganha cor ou vira razão | R-17, RNF-11, L-19 |
| `ChampionLane::clearest` | entre as leituras claras, a de maior razão entre `value` em valor absoluto e `sd`; `None` se nenhuma é clara | R-1, ADR-0011 |
| `LaneProfile::champion` | o campeão pelo id do Data Dragon, ou `None` | R-1, ADR-0011 |

## 22. `core::engine::rival` — Sua rota ao vivo (`core`, pura)

### 22.1 `report`

```rust
pub fn report(game: &LiveGame) -> Option<Rival>;
```

- **Faz:**
  1. Antes de `MIN_SECONDS` (1:30 de `gameTime`) devolve `None`.
  2. Acha o jogador local e a rota dele por `Role::from_lcu`.
  3. O oponente é o primeiro inimigo com a mesma rota.
  4. A métrica sai de `cobranca(role)`, a mesma regra do pós-jogo: `Farm` para quem
     farma, `Vision` para o suporte.
  5. Por minuto: o total da Live Client dividido por `gameTime / 60`, arredondado a
     0,1.
  6. `gold_diff` é o seu ouro em itens menos o dele; `standing` sai dele.
- **Retorno:** `None` sem jogador local, com rota `Unknown` ou sem inimigo na rota.
- **Confiança:** exata nas taxas e em `gold_diff`; regra em `standing`.
- **Atende:** R-11, C-4, L-27

### 22.2 `standing`

```rust
pub fn standing(mine: f64, theirs: f64) -> Standing;
```

- **Faz:** a margem é `max(AHEAD_FLOOR, AHEAD_SHARE × média dos dois)`. `Ahead`
  com `mine − theirs ≥ margem`, `Behind` com `≤ −margem`, `Even` no meio. Como a
  margem sai dos dois, o veredito de um lado é sempre o espelho do outro (A24,
  T-RIVAL-6).
- **Params:** `mine` e `theirs` em ouro: o seu ouro em itens e o do oponente.
- **Confiança:** regra.
- **Atende:** R-11

## 23. `core::engine::champion` — Campeão, rota e atributos base (`core`, pura)

### 23.1 `Role`

```rust
impl Role {
    pub fn from_lcu(position: &str) -> Self;
    pub fn short(self) -> &'static str;
}
```

| Método | Faz | Atende |
|---|---|---|
| `from_lcu` | `top`, `jungle`, `middle`, `bottom`, `utility` em qualquer caixa; o resto vira `Unknown`. Lê o LCU, a Live Client e a match-v5 | R-1, R-4, R-11 |
| `short` | o rótulo da tela: `TOP`, `JG`, `MID`, `ADC`, `SUP`, e `—` para `Unknown` | R-11 |

### 23.2 `grown`

```rust
pub fn grown(base: f64, per_level: f64, level: u8) -> f64;
```

- **Faz:** `base + per_level × n × (0,7025 + 0,0175 × n)`, com `n = nível − 1`.
- **Params:** `level` é cortado em 1–18.
- **Retorno:** no nível 1, `base`; no 18, exatamente `base + 17 × per_level`.
- **Confiança:** alta — é a conta do jogo.
- **Atende:** R-6, R-10

### 23.3 `BaseStats`

```rust
impl BaseStats {
    pub fn health_at(&self, level: u8) -> f64;
    pub fn armor_at(&self, level: u8) -> f64;
    pub fn magic_resist_at(&self, level: u8) -> f64;
    pub fn attack_damage_at(&self, level: u8) -> f64;
}
```

| Método | Faz | Atende |
|---|---|---|
| `health_at` | `grown(health, health_per_level, level)` | R-6, R-10 |
| `armor_at` | idem, armadura | R-6, R-10 |
| `magic_resist_at` | idem, resistência mágica | R-6, R-10 |
| `attack_damage_at` | idem, dano de ataque | R-6, R-10 |

### 23.4 `Champion`

```rust
impl Champion {
    pub fn primary_tag(&self) -> &str;
    pub fn physical_share(&self) -> f64;
    pub fn is_ranged(&self) -> bool;
    pub fn is_frontline(&self) -> bool;
}
```

| Método | Faz | Atende |
|---|---|---|
| `primary_tag` | a primeira classe do Data Dragon, ou `""` | 02 §4 |
| `physical_share` | a fração 0–1 do dano que sai físico: média de `attack / (attack + magic)` (0,5 sem `info`) com um prior pela classe principal — `Marksman` 0,90, `Mage` 0,10, `Support` 0,40, `Assassin` 0,55, o resto 0,70. **Confiança:** fraca | R-1, R-10 |
| `is_ranged` | `attack_range ≥ 500` | R-1, R-6 |
| `is_frontline` | classe `Tank` ou `defense ≥ 7` | R-1 |

### 23.5 `ChampionIndex`

```rust
impl ChampionIndex {
    pub fn get(&self, id: i64) -> Option<&Champion>;
    pub fn by_key(&self, key: &str) -> Option<&Champion>;
    pub fn is_empty(&self) -> bool;
    pub fn ids(&self) -> Vec<i64>;
}
```

| Método | Faz | Atende |
|---|---|---|
| `get` | o campeão pelo id numérico | R-1 |
| `by_key` | o campeão pela chave textual (`LeeSin`), sem diferenciar caixa. É como a Live Client nomeia. Varre o índice inteiro | R-10, R-11 |
| `is_empty` | o índice não tem campeão: o motor ainda não está pronto (`EngineStatus::ready`) | R-1 |
| `ids` | todos os ids, em ordem crescente | R-2 |

## 24. `core::engine::items` — Itens e atributos (`core`, pura)

### 24.1 `parse_stats`

```rust
pub fn parse_stats(description: &str) -> ItemStats;
```

- **Faz:** acha o bloco `<stats>…</stats>` da `description` e, para cada
  `<attention>valor</attention> Rótulo`, soma o valor no campo do rótulo.
  Percentual vira fração (`35%` → 0,35); penetração percentual combina
  multiplicativo.
- **Params:** `description` em en_US: os rótulos são a chave.
- **Retorno:** `ItemStats::default()` sem bloco ou com bloco truncado. Rótulo
  desconhecido é ignorado sem perder o resto.
- **Erros:** nenhum; nunca entra em pânico.
- **Atende:** R-10, ADR-0015, L-11

### 24.2 `ItemStats`

```rust
impl ItemStats {
    pub fn plus(self, other: ItemStats) -> ItemStats;
    pub fn physical_penetration(&self) -> Penetration;
    pub fn magic_penetration(&self) -> Penetration;
}
```

| Método | Faz | Atende |
|---|---|---|
| `plus` | soma dois conjuntos: os planos somam, os percentuais de penetração combinam `1 − (1 − a)(1 − b)` (35% + 35% = 57,75%), o crítico para em 1 | R-10 |
| `physical_penetration` | `Penetration` com `percent = armor_pen_percent`, `flat = lethality` e reduções em zero | R-10 |
| `magic_penetration` | idem com `magic_pen_percent` e `magic_pen_flat` | R-10 |

### 24.3 `ItemIndex`

```rust
impl ItemIndex {
    pub fn get(&self, id: i64) -> Option<&Item>;
    pub fn is_empty(&self) -> bool;
    pub fn totals(&self, item_ids: &[i64]) -> ItemStats;
    pub fn gold_invested(&self, item_ids: &[i64]) -> f64;
}
```

| Método | Faz | Atende |
|---|---|---|
| `get` | o item pelo id | R-10 |
| `is_empty` | o índice não tem item | R-10 |
| `totals` | `plus` de todos os itens conhecidos da lista; id desconhecido é ignorado | R-10 |
| `gold_invested` | soma de `cost` dos itens conhecidos, em ouro. Nenhum código de produção chama: o ouro em itens da partida vem do `price` da Live Client | R-12 |

## 25. `core::engine::damage` — Dano e resistência (`core`, pura)

### 25.1 `effective_resist`

```rust
pub fn effective_resist(base: f64, pen: &Penetration) -> f64;
```

- **Faz:**
  1. Tira a redução plana; pode ficar negativa.
  2. Se ainda positiva, aplica a redução percentual.
  3. Se ainda positiva, aplica a penetração percentual.
  4. Se ainda positiva, tira a penetração plana, sem passar de zero.
- **Retorno:** a resistência que sobra. Negativa só por redução plana.
- **Confiança:** alta — é a ordem do jogo.
- **Atende:** R-10

### 25.2 `damage_multiplier`

```rust
pub fn damage_multiplier(resist: f64) -> f64;
```

- **Faz:** `100 / (100 + resist)` com resistência ≥ 0; `2 − 100 / (100 − resist)`
  abaixo de zero.
- **Retorno:** fração do dano que passa. Fica abaixo de 2 com qualquer resistência
  negativa.
- **Confiança:** alta.
- **Atende:** R-10

### 25.3 Vida efetiva e dano final

```rust
pub fn effective_hp(health: f64, resist: f64) -> f64;
pub fn damage_after_resist(raw: f64, base_resist: f64, pen: &Penetration) -> f64;
```

| Função | Faz | Atende |
|---|---|---|
| `effective_hp` | `health / damage_multiplier(resist)`: quanto dano bruto o alvo aguenta. Só os testes chamam; a build usa `effective_hp_against` | R-10 |
| `damage_after_resist` | `raw × damage_multiplier(effective_resist(base_resist, pen))`. Só os testes chamam | R-10 |

## 26. `core::engine::build` — Build adaptativa (`core`, pura)

### 26.1 `suggest`

```rust
pub fn suggest(
    you: &Combatant,
    enemies: &[Combatant],
    champions: &ChampionIndex,
    items: &ItemIndex,
    limit: usize,
) -> (BuildContext, Vec<ItemSuggestion>);
```

- **Faz:**
  1. Monta o seu `Loadout`. Campeão fora do índice: contexto vazio (`incoming_physical_share`
     0,5, o resto 0) e lista vazia.
  2. Lê o time inimigo: a divisão do dano que vem (`incoming_physical_share`), a
     maior penetração de cada componente (`team_penetration`) e os `Loadout` dos
     alvos.
  3. Poder base = vida efetiva × dano efetivo. O dano efetivo pesa AD e AP pelo
     jeito que o seu campeão bate: as notas de ataque e magia do Data Dragon,
     `attack / (attack + magic)` para o AD e o resto para o AP; sem nota, metade
     de cada (A4). Poder não positivo ou infinito: devolve o contexto preenchido e
     a lista vazia.
  4. Para cada item comprável de custo ≥ `MIN_ITEM_COST` que você não tem, soma o
     item aos seus e mede o ganho de poder em % por 1000 de ouro, a vida efetiva a
     mais, o ganho de dano efetivo e até duas razões (`reasons_for`).
  5. Ordena por `value_per_1000g` decrescente, empate por `item_id`, e corta em
     `limit`.
- **Params:** `limit` é contagem. A casca usa 8 quando o front não manda nada; o
  painel pede 8 e o overlay, 2.
- **Retorno:** o `BuildContext` e as sugestões. Na ponte, a tupla vira o par JSON
  `[contexto, lista]` de `build_suggestions`.
- **Erros:** nenhum. Campeão desconhecido ou poder base inválido dão lista vazia.
- **Confiança:** alta na ordem entre itens do mesmo tipo de dano; regra entre AD
  e AP (o peso sai de uma nota da Riot, não de dano medido); o `value_per_1000g` é
  índice, não DPS ([visão geral §2](../produto/visao-geral.md#2-de-onde-vem-cada-número)).
- **Atende:** R-10, C-6, ADR-0015

O desenho, os limites e o que o motor não trata estão em
[03 §2](03-motor-da-partida.md#2-build-adaptativa).

### 26.2 `loadout`

```rust
pub fn loadout(
    combatant: &Combatant,
    champions: &ChampionIndex,
    items: &ItemIndex,
) -> Option<Loadout>;
```

- **Faz:** atributos base no nível (`BaseStats::health_at` e irmãos, a curva de
  `grown`) mais a soma dos itens (`ItemIndex::totals`). AP sai só dos itens.
- **Erros:** `None` quando o campeão não está no índice. Item desconhecido é
  ignorado.
- **Atende:** R-10

### 26.3 `incoming_physical_share`

```rust
pub fn incoming_physical_share(
    enemies: &[Combatant],
    champions: &ChampionIndex,
    items: &ItemIndex,
) -> f64;
```

- **Faz:** média ponderada da fração física de cada inimigo.
  - Com algum AD ou AP de item, a fração é AD/(AD + AP) **dos itens**.
  - Sem isso, é o prior de `Champion::physical_share` (classe e `info` do Data
    Dragon).
  - O peso é o AD base no nível mais o AD e o AP dos itens.
  - Campeão desconhecido fica de fora; sem peso nenhum, 0,5.
- **Retorno:** fração 0–1.
- **Confiança:** alta depois dos primeiros itens; fraca antes, porque o prior de
  classe é escrito à mão ([visão geral §2](../produto/visao-geral.md#2-de-onde-vem-cada-número)).
- **Atende:** R-10, C-6

### 26.4 A conta do duelo

```rust
pub fn effective_hp_against(
    me: &Loadout,
    physical_share: f64,
    enemy_physical_pen: &Penetration,
    enemy_magic_pen: &Penetration,
) -> f64;
pub fn offense_against(me: &Loadout, targets: &[Loadout]) -> f64;
pub fn side_power(
    side: &[Combatant],
    foes: &[Combatant],
    champions: &ChampionIndex,
    items: &ItemIndex,
) -> f64;
pub fn skirmish_odds(
    side: &[Combatant],
    foes: &[Combatant],
    champions: &ChampionIndex,
    items: &ItemIndex,
) -> f64;
pub fn team_penetration(enemies: &[Combatant], items: &ItemIndex) -> (Penetration, Penetration);
```

| Função | Faz | Custo |
|---|---|---|
| `effective_hp_against` | Vida ÷ (s × mult(armadura) + (1 − s) × mult(RM)), com `s = physical_share` cortado em [0, 1], cada resistência passada pela penetração dada (`effective_resist`) e `mult` = `damage_multiplier`. Fração que passa ≤ 0 dá `f64::INFINITY`. Em pontos de vida efetiva | O(1) |
| `offense_against` | Dano efetivo, um proxy e não DPS: AD × mult(armadura do alvo) + AP × mult(RM do alvo), com a sua penetração, na média dos alvos. Sem alvos, AD + AP crus. AD e AP pesam 1 para 1 | O(alvos) |
| `side_power` | Soma, por membro conhecido de `side`, da vida efetiva contra a divisão e a penetração dos `foes` × o dano efetivo contra os `Loadout` deles. Vida efetiva infinita conta 0 | O(membros × foes), mais três `ItemIndex::totals` por foe (divisão, penetração, `loadout`) |
| `skirmish_odds` | `ours / (ours + theirs)`, com os dois `side_power`; soma ≤ 0 dá 0,5. Fração 0–1, e trocar os lados dá o complemento | Dois `side_power`. `lane_outlooks` chama 35 vezes por rota (1 + 4 + 18 + 12), `skirmishes` 5 por grupo, e `priority` também usa |
| `team_penetration` | O máximo, entre os inimigos, de cada componente por conta própria: percentual e plano, físico e mágico. Redução fica em 0. É o pior caso para decidir defesa | O(inimigos) |

Sem estado e sem alocação além dos `Vec<Loadout>` de `side_power`. A regra de
penetração e o multiplicador estão em [03 §3](03-motor-da-partida.md#3-dano); a
chance que sai daqui é **fraca como probabilidade**
([visão geral §2](../produto/visao-geral.md#2-de-onde-vem-cada-número)).
Atendem R-10 e R-11.

## 27. `core::engine::lanes` — Rotas e escaramuças (`core`, pura)

### 27.1 `lane_outlooks`

```rust
pub fn lane_outlooks(
    allies: &[Lineup],
    enemies: &[Lineup],
    champions: &ChampionIndex,
    items: &ItemIndex,
    store: &dyn StatsStore,
) -> Vec<LaneOutlook>;
```

- **Faz:** para cada rota, na ordem Top, Jungle, Mid, Bot, Support, em que os dois
  lados têm alguém (o primeiro `Lineup` com a rota) **com o campeão no índice**,
  monta a `LaneOutlook` (`one_lane`). Rota com um campeão fora do índice sai da
  lista: sem atributos, o duelo cairia no piso ou no teto por falta de dado (A13):
  1. `duel`: `skirmish_odds` 1v1 com os níveis e itens reais.
  2. A estimativa da base: o que as duas forças explicam (`expected_against` das
     duas `base_rate`) mais o matchup encolhido para essa referência, com
     `strength_for_sd(MATCHUP_PRIOR_SD)`, a mesma crença do draft.
  3. `0,5 + W_DUEL × (duel − 0,5) + (estimativa − 0,5)`, cortada em
     [`FLOOR`, `CEILING`]. A fórmula e os valores estão em
     [03 §4](03-motor-da-partida.md#4-chance-por-rota-e-escaramuça); o peso é o
     [ponto fraco a corrigir primeiro](../produto/visao-geral.md#o-ponto-fraco-a-corrigir-primeiro).
  4. `by_level`, `curve` e `gaps`, a mesma conta de duelo com os níveis trocados
     e os itens mantidos ([03 §4](03-motor-da-partida.md#curva-por-nível-e-vantagem-de-nível)).
  5. Até `MAX_REASONS` razões, nesta ordem: o matchup com pp e n, quando há
     partidas; o 1v1, a partir de 2 pontos de 50%; quem tem alcance
     (`Champion::is_ranged`); e a diferença de nível, ou, nivelados, o oponente
     com mais de 15% de vida a mais.
- **Params:** `store` é a base; `NoStats` sem ela.
- **Retorno:** uma `LaneOutlook` por rota com os dois lados conhecidos. Rota sem
  oponente fica de fora em vez de virar 50%.
- **Erros:** nenhum. Campeão fora do índice entra com o nome `#id` e poder zero.
- **Confiança:** fraca como probabilidade; a ordem entre rotas é a da conta de
  atributos ([visão geral §2](../produto/visao-geral.md#2-de-onde-vem-cada-número)).
- **Atende:** R-11, R-6, RNF-1, RNF-3, ADR-0007, ADR-0008, L-4

Se duelo e matchup contam a mesma coisa duas vezes quando a base existir está em
[P-7](../produto/perguntas-em-aberto.md#p-7--a-chance-por-rota-soma-duas-coisas-que-se-sobrepõem-).

### 27.2 `skirmishes`

```rust
pub fn skirmishes(
    allies: &[Lineup],
    enemies: &[Lineup],
    champions: &ChampionIndex,
    items: &ItemIndex,
) -> Vec<Skirmish>;
```

- **Faz:** para cada grupo de `GROUPS` (topo, meio, bot, dragão, arauto), na ordem,
  quando os dois lados têm todas as rotas do grupo, todas com o campeão no índice
  (A13):
  - `odds`: `skirmish_odds` do grupo com os níveis e itens reais;
  - `by_level`: a mesma conta nivelada nos `BREAKPOINTS`;
  - razões (`skirmish_reasons`): quem mais pesa de cada lado, pelo `side_power`
    sozinho contra o grupo deles, e a frontline (`Champion::is_frontline`) quando
    só um lado tem.
- **Retorno:** até cinco `Skirmish`. Grupo incompleto fica de fora: 2v2 contra 2v1
  é outra pergunta.
- **Erros:** nenhum.
- **Confiança:** fraca como probabilidade.
- **Atende:** R-11, R-6

### 27.3 `focus`

```rust
pub fn focus(skirmishes: &[Skirmish], upcoming: &[Spawn], my_role: Role) -> Option<Focus>;
```

- **Faz:** escolhe uma escaramuça para o overlay, pela primeira regra que acerta:
  1. O objetivo mais próximo, a até `OBJECTIVE_WINDOW` segundos (o já disponível
     conta). Dragão e Ancião puxam `dragao`; Barão, Arauto e Grubs puxam `arauto`.
     Se esse grupo não saiu, segue.
  2. A dupla da sua rota: Top `jg_top`, Mid `jg_mid`, Bot e Support `bot`. Jungle
     e Unknown não têm, porque o jungle está em todos.
  3. A de maior |`odds` − 0,5|, a que mais informa.
- **Params:** `upcoming` é `Timeline::upcoming`, com `seconds_left` em segundos. O
  nome do objetivo casa por texto exato com o que `timeline` emite.
- **Retorno:** `Focus` com o porquê em `because`. `None` só com `skirmishes` vazio.
- **Erros:** nenhum.
- **Atende:** R-15, R-11

### 27.4 `Lineup`

```rust
impl Lineup {
    pub fn new(role: Role, champion_id: i64, level: u8, items: Vec<i64>) -> Self;
}
```

| Método | Faz | Atende |
|---|---|---|
| `new` | Monta o `Lineup` com o `Combatant` dentro. Quem chama: `DraftContext::lineups` (nível `PREGAME_LEVEL`, sem itens) e `analysis.rs` na casca (níveis e itens reais) | R-6, R-11 |

## 28. `core::engine::gold` — Ouro dos dez (`core`, pura)

### 28.1 `earned_at`

```rust
pub fn earned_at(game_time: f64, cs: i64, kills: i64, assists: i64, invested: f64) -> f64;
```

- **Faz:** `STARTING + max(0, game_time − PASSIVE_START) × PASSIVE_PER_SECOND +
  cs × PER_CS + kills × PER_KILL + assists × PER_ASSIST`, e devolve o maior entre
  isso e `invested`: ninguém gasta o que não ganhou.
- **Params:** `game_time` em segundos; `cs`, `kills` e `assists` em contagem, da aba
  de pontuação (`RawScores`); `invested` em ouro.
- **Retorno:** ouro ganho até `game_time`. Torres, placas, objetivos e recompensa
  por sequência ficam de fora.
- **Erros:** nenhum.
- **Confiança:** fraca: constantes escritas à mão, e a tela diz que é estimativa.
- **Atende:** R-12

Também é o que o pós-jogo usa para medir ouro parado (`coach`).

### 28.2 `report`

```rust
pub fn report(game: &LiveGame) -> GoldReport;
```

- **Faz:**
  1. Para cada jogador: `earned_at` com tempo, tropas, abates, assistências e o
     investido; `unspent = max(0, earned − invested)` só para o jogador local, e
     `None` para os outros nove (C-4); ouro por minuto com piso de
     1 s no divisor; `is_ally` pelo `team` do jogador local.
  2. Soma investido e ganho por lado.
  3. Com jogador local, até três notas: a diferença de investido, a partir de 1000
     de ouro; o seu `unspent`, a partir de `UNSPENT_ALERT`; e o inimigo com mais
     investido, se algum investiu.
- **Retorno:** o `GoldReport`. Sem jogador local, ninguém é aliado, os totais do seu
  lado ficam em 0 e não há nota.
- **Erros:** nenhum.
- **Confiança:** exata no investido e em `invested_lead`; fraca em ganho,
  `unspent`, ouro por minuto e `earned_lead`.
- **Atende:** R-12, C-4

A casca chama a cada pedido do front (`gold_report`), sobre o último `LiveGame`
lido. O desenho está em [03 §5](03-motor-da-partida.md#5-ouro).

## 29. `core::engine::timeline` — Timers de objetivo, estruturas e buffs (`core`, pura)

### 29.1 `build`

```rust
pub fn build(events: &[RawEvent], game: &LiveGame, timings: &Timings) -> Timeline;
```

- **Faz:**
  1. Passa por todos os eventos:
     - `DragonKill` soma o dragão ao lado de quem matou e guarda o `DragonType`;
     - `BaronKill` guarda o instante;
     - `HeraldKill` e `VoidgrubKill`/`HordeKill` marcam o objetivo como tomado;
     - `TurretKilled` e `ChampionKill` somam ao lado de quem matou.
  2. Um lado com `dragons_for_soul` dragões fecha a alma. Daí em diante o pit é
     do `Ancião`, que volta em `elder_respawn`.
  3. Dragão e Barão: abate visto vira `at = abate + respawn` e
     `observed = true`. Sem abate, `at` é a constante e `observed = false`.
  4. Arauto e Grubs entram com a constante até serem tomados, ou até 300 s
     depois do spawn.
  5. Ordena `upcoming` pelo tempo restante. Monta `respawns` com os `InhibKilled`
     e os `TurretKilled` de torre do nexo que ainda não voltaram, nos dois
     formatos de nome.
- **Params:** `events` é a lista inteira da partida até agora, como a Live Client
  devolve. A casca sempre passa `Timings::default()` em `timings`.
- **Retorno:** a `Timeline` do instante `game.game_time`.
- **Erros:** nenhum.
  - Abate cujo `killer` não é um jogador (tropa) não conta.
  - Sem jogador local, todo abate com dono conta como `enemy`, e nenhuma
    estrutura é `ours`.
  - Nome de estrutura desconhecido não vira timer ([P-14](../produto/perguntas-em-aberto.md)).
- **Confiança:**
  - exata nas contagens e no instante de um abate visto;
  - regra no intervalo de respawn e no spawn estimado.
- **Atende:** R-13, C-1, C-4, RNF-1, P-14

A diferença entre observado e estimado, e por que ela aparece na tela, está em
[03 §6](03-motor-da-partida.md#6-timers).

### 29.2 `RawEvent::outcome`

```rust
impl RawEvent {
    pub fn outcome(&self) -> Option<bool>;
}
```

| Método | Faz | Atende |
|---|---|---|
| `outcome` | `Some(true)` no `GameEnd` com `Win` e `Some(false)` com `Lose` ou `Loss`, sem diferenciar maiúsculas. Qualquer outro evento ou resultado dá `None`. É o único jeito de saber o resultado sem a chave da Riot. A casca chama em `recorder::observe` | R-16, RNF-5 |

### 29.3 `BuffTracker::update`

```rust
impl BuffTracker {
    pub fn update(&mut self, events: &[RawEvent], game: &LiveGame, timings: &Timings) -> Vec<Buff>;
}
```

| Método | Faz | Atende |
|---|---|---|
| `update` | 1) Cada `BaronKill` e cada `DragonKill` de `Elder` ainda não visto e não acabado vira um `Buff` do time de quem matou. 2) Lido até 5 s depois do abate, `holders` são os vivos desse time; depois disso, `None`. 3) A cada leitura, sai de `holders` quem está morto ou teve `ChampionKill` depois do abate. 4) Descarta o buff que acabou ou ficou sem ninguém, e devolve os que restam | R-13, C-4 |

- **Confiança:** exata em `until`, que é o abate visto mais a duração do jogo.
  `holders` é exata quando não é `None`.
- **Buff descartado não volta:** um buff que acabou por ficar sem ninguém entra
  em `encerrados`, e o mesmo abate, que continua na lista de eventos, não o
  recria na leitura seguinte (A8, T-OBJETIVO-13).

## 30. `core::engine::priority` — Onde está o valor no mapa (`core`, pura)

### 30.1 `report`

```rust
pub fn report(
    game: &LiveGame,
    timeline: &Timeline,
    champions: &ChampionIndex,
    items: &ItemIndex,
) -> PriorityReport;
```

- **Faz:** pontua cada rota de `LANES`:
  - `TOP`, com Barão, Arauto e Grubs;
  - `MID`, sem objetivo;
  - `BOT`, que junta `BOTTOM` e `UTILITY`, com Dragão e Ancião.

  Para cada uma:
  1. Monta a escaramuça. De um lado, os vivos do seu time naquela rota, mais
     você se não for dela e estiver vivo. Do outro, os vivos deles na rota.
     `score = skirmish_odds × 100`, e a razão `NvM: X% para o seu lado`.
  2. Primeiro inimigo morto da rota: `+ min(respawn_timer / 2, 25)`.
  3. Jungler inimigo morto: `+ 12` em todas as rotas.
  4. Seu time à frente em tropas na rota: `+ min(diferença / 20 × 10, 10)`.
  5. Objetivo da rota a até 90 s: `+ (1 − faltam / 90) × 15`.
  6. Corta o placar em [0, 100] e arredonda a 0,1. Cada sinal que somou ponto
     deixa a sua razão, até cinco.

  As rotas saem do maior placar ao menor. `focus` é o primeiro de
  `timeline.upcoming` a até 90 s, com ` (estimado)` quando ele não é observado.
- **Params:** `timeline` vem do `timeline::build` da mesma leitura. O `focus`
  conta com `upcoming` já ordenado.
- **Retorno:** as três rotas e o foco.
- **Erros:** sem jogador local, `lanes` vazio e `focus` `None`.
- **Confiança:**
  - alta em `skirmish_odds`, que é o motor de dano;
  - fraca em `score`, porque os pesos são de exibição e escolhidos à mão.
- **Atende:** R-14, C-4, C-5

As razões dizem que a rota vale mais e por quê, nunca onde o inimigo está
([03 §7](03-motor-da-partida.md#7-prioridade)). Todo ponto do `score` tem a
razão dele: até a 0.7.0 as razões eram cortadas em três e os pontos ficavam
(A23, T-PRIORIDADE-11).

## 31. `core::engine::coach` — Leitura pós-partida (`core`, pura)

### 31.1 `review`

```rust
pub fn review(recording: &Recording, items: &ItemIndex) -> Report;
```

- **Faz:**
  1. Lê o jogador local e a rota no último quadro. Sem jogador local ou sem
     quadros, devolve o relatório vazio `Sem gravação desta partida.`. Com a
     partida abaixo de 300 s no relógio do último quadro, devolve o vazio
     `… curta demais para tirar conclusão.`.
  2. Monta as curvas (`Series`), reamostradas a cada 30 s. Mortes e abates do
     time entram com o instante exato.
  3. Roda as observações da tabela abaixo. Farm e visão só entram quando a
     `cobranca` da rota pede ([03 §8](03-motor-da-partida.md#8-leitura-pós-partida)).
  4. Separa em `good` e `bad`, cada lista do maior `weight` ao menor, e escreve
     o `summary`.
- **Params:** `items` é o `ItemIndex` do Data Dragon. Sem ele, a observação do
  item grande não sai.
- **Retorno:** o `Report`, que a casca salva em `ultima-partida.json`.
- **Erros:** nenhum. Quando o relatório sai vazio, o `summary` diz por quê.
- **Confiança:**
  - exata nas observações de tropas, visão, abates, mortes, níveis e ouro em
    itens;
  - fraca no ouro parado, que vem da renda reconstruída por `gold::earned_at`;
  - os limiares são regra.
- **Atende:** R-16, RNF-5, RNF-8, RNF-11, C-7, L-27

As observações. "Por minuto" divide o placar, que é da partida inteira, pelo
relógio da partida no último quadro, e não pelo tempo gravado: um app aberto aos
20:00 não divide 30 minutos de tropas por 10 (A9, T-POS-23). A fatia de tempo
morto continua sobre o tempo gravado, porque as duas pontas dela são gravação.
O primeiro item grande só vira observação quando a gravação viu a compra: item
que já estava no primeiro quadro não tem hora conhecida. Todo limiar é escrito à
mão.

| Observação | Ruim | Boa | `weight` | Onde |
|---|---|---|---|---|
| Tropas por minuto contra o oponente direto, só com `cobranca.farm` | ele ≥ 0,8/min à frente | você ≥ 0,5/min à frente | a diferença / 3, até 1 | `coach.rs:362` |
| Visão por minuto contra o oponente direto, só com `cobranca.vision` | ele ≥ 0,4/min à frente | você ≥ 0,3/min à frente | a diferença / 1,5, até 1 | `coach.rs:401` |
| Participação: (abates + assistências) / abates do time, só com ≥ 5 abates do time | ≤ `kp_bad` | ≥ `kp_good` | ruim: 1 − fatia; boa: a fatia | `coach.rs:434` |
| Tempo morto, somado entre quadros a até 10 s um do outro | ≥ 12% do tempo gravado | zero mortes (`Nenhuma morte`, peso 1) | a fatia | `coach.rs:469` |
| Mortes em sequência | ≥ 3 mortes em 240 s | — | mortes na janela / total | `coach.rs:511` |
| Primeiro item de custo ≥ 2500 | aos 15:00 ou depois | até 11:00 | ruim: (t − 900) / 600, até 1; boa: (660 − t) / 400 | `coach.rs:561` |
| Ouro parado: renda estimada menos ouro em itens ≥ 1300, vivo | ≥ 120 s no total | — | tempo / 600, até 1 | `coach.rs:594` |
| Vantagem do time em ouro de itens | pico ≥ 3000 e fim ≤ pico − 3000 | fim ≥ 3000 | ruim: (pico − fim) / 8000; boa: fim / 8000; até 1 | `coach.rs:636` |
| Nível contra o oponente direto, no fim | 2 ou mais atrás | 2 ou mais à frente | a diferença / 5, até 1 | `coach.rs:673` |

### 31.2 `Recording` — métodos

```rust
impl Recording {
    pub fn push(&mut self, game: LiveGame);
    pub fn duration(&self) -> f64;
}
```

| Método | Faz | Atende |
|---|---|---|
| `push` | Acrescenta o quadro só se o `game_time` dele for maior que o do último. Relógio parado ou voltando é descartado: o quadro repetido só engordaria o arquivo e distorceria o tempo morto | R-16 |
| `duration` | Segundos gravados: o `game_time` do último quadro menos o do primeiro. Sem quadros, 0 | R-16 |

### 31.3 `Series::is_empty`

```rust
impl Series {
    pub fn is_empty(&self) -> bool;
}
```

| Método | Faz | Atende |
|---|---|---|
| `is_empty` | `true` quando `gold_lead` está vazio, isto é, quando a leitura não rodou | R-16 |

### 31.4 `worth_reviewing`

```rust
pub fn worth_reviewing(recording: &Recording) -> bool;
```

- **Faz:** diz se a gravação vale uma leitura: ao menos `MIN_FRAMES` quadros
  (60, um minuto a um por segundo) **e** o relógio do último quadro em pelo menos
  `MIN_DURATION` (300 s). Um remake de três minutos passa no primeiro e não no
  segundo; trinta quadros no fim de uma partida longa, o contrário.
- **Retorno:** verdadeiro quando há o que ler.
- **Erros:** nenhum. Gravação vazia: falso.
- **Quem usa:** `recorder::finish` na casca, que não grava nem avisa quando é
  falso. O corte é o mesmo de `review`, num lugar só (A6, T-POS-24).
- **Atende:** R-16, RNF-1
