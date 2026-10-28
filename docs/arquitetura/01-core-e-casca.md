# 01 — Core e casca

O workspace separa a lógica pura da casca ([ADR-0001](../adr/0001-core-puro-separado-da-casca.md)).

## 1. O workspace

```
core/src/                    presage-core — sem Tauri, sem rede, sem disco
  draft.rs                   LCU cru            ->  DraftState
  live.rs                    Live Client cru    ->  LiveGame
  history.rs                 match-v5 crua      ->  PlayedGame, LaneGame
  lockfile.rs                credenciais do cliente (lockfile e linha de comando)
  pool.rs                    pool do jogador, por rota e tier
  prefs.rs                   o que o jogador quer ver no overlay
  riot.rs                    endereçamento da Riot API (região, Riot ID)
  engine/champion.rs         atributos, classes e stats base por nível
  engine/damage.rs           resistência efetiva, penetração, vida efetiva
  engine/items.rs            atributos de item e parser da descrição
  engine/build.rs            build adaptativa por valor/ouro
  engine/lanes.rs            chance por rota e escaramuça 2v2 / 3v3
  engine/gold.rs             ouro dos dez: investido exato, ganho estimado
  engine/timeline.rs         timers de objetivo, observados e estimados
  engine/priority.rs         onde está o valor no mapa
  engine/runes.rs            runa do confronto, por regra
  engine/coach.rs            leitura da partida a partir da gravação
  engine/comp.rs             perfil de composição de time
  engine/context.rs          a leitura do champ select: rota, candidatos, inimigo inferido
  engine/roles.rs            quem joga cada rota
  engine/report.rs           a análise de rota inteira, do draft e da partida
  engine/wincon.rs           condição de vitória: composição por regra, duelos por atributos
  engine/minimax.rs          draft com resposta e sugestão de ban
  engine/poolgap.rs          retrato dos core e os cinco que completam a pool
  engine/stats.rs            shrinkage, referência, StatsStore, PersonalStats
  engine/dataset.rs          formato da base de partidas, Layered
  engine/profile.rs          perfil de rota em três níveis
  engine/mod.rs              rank(): junta tudo e propaga incerteza

src-tauri/src/               a casca
  lcu/credentials.rs         acha o cliente (linha de comando, depois lockfile)
  lcu/client.rs              HTTP com Basic Auth sobre o TLS autoassinado
  live.rs                    Live Client Data API, 127.0.0.1:2999
  riot.rs                    Riot Web API, chave vinda do ambiente
  profile.rs                 histórico pessoal, incremental e cacheado
  ddragon.rs                 campeões, itens e runas, com cache em disco
  analysis.rs                lê a partida e chama engine::report
  recorder.rs                grava a partida e dispara a leitura no fim
  overlay.rs                 a janela do overlay e os atalhos
  watcher.rs                 polling adaptativo, emite eventos
  storage.rs                 JSON em %LOCALAPPDATA%\Presage
  lib.rs                     comandos expostos ao front

src/                         React
  lib/ddragon.ts             ícones e nomes
  lib/types.ts               os tipos que a casca devolve, espelhados
  components/                DraftBoard, RecommendationPanel, LiveBuild,
                             LaneAnalysis, CurvaDeNivel, CondicaoDeVitoria,
                             GoldTracker, PostGame, PerfilDeRota,
                             Runas, PoolBuilder, OverlayPrefsPanel, ConnectionStatus
  demo.tsx                   a interface com dados de exemplo, sem Tauri

tools/ingest/src/            o coletor — binário à parte, fora do instalador
  aggregate.rs               partida crua -> contagens (puro, testado)
  limiter.rs                 janela deslizante no formato do cabeçalho da Riot
  api.rs                     cliente com dois limites, 429 e retentativa
  main.rs                    CLI, semeadura, retomada e cobertura
```

A separação não é cosmética: `cargo test -p presage-core` roda em segundos e
funciona numa máquina sem MSVC — e numa Linux sem WebKitGTK — porque o core não
puxa a árvore do Tauri.

## 2. Fronteiras entre JSON cru e domínio

Três fontes de JSON que a Riot pode mudar sem aviso, e o mesmo desenho nas três:
tipos que espelham o formato cru, tipos de domínio que o resto consome, e **uma
função só** entre eles.

| Fonte | Cru | Domínio | Fronteira |
|---|---|---|---|
| LCU | `LcuSession`, `LcuAction`, `LcuPlayer` | `DraftState`, `DraftSlot` | `to_draft_state` |
| Live Client | `RawPlayer`, `RawItem`, `RawScores` | `LiveGame`, `LivePlayer` | `to_live_game` |
| match-v5 | `MatchDto`, `MatchParticipant`, `Challenges` | `PlayedGame`, `LaneGame` | `history::played` |

Todos os tipos crus são `#[serde(default)]`: campo que falta derruba a métrica
ou a regra de negócio, não a desserialização. O preço é que um nome errado não
dá erro ([P-5](../produto/perguntas-em-aberto.md)), e por isso cada fronteira
tem um teste com o JSON de verdade.

O coletor tem os próprios tipos crus da match-v5 em `aggregate.rs`, com as mesmas
regras de descarte que `history::played`. São duas cópias da mesma leitura —
unificar é trabalho a fazer, e até lá as duas precisam mudar juntas.

## 3. A casca

### Polling

`watcher.rs` lê o estado no ritmo da fase ([ADR-0002](../adr/0002-polling-em-vez-de-websocket.md)):

| Fase | Intervalo |
|---|---|
| champ select | 350 ms |
| partida | 1 s |
| cliente aberto, parado | 1,5 s |
| sem cliente | 3 s |

### Comandos

Os que o front chama por `invoke` (`lib.rs`, `generate_handler!`):

| Comando | Devolve |
|---|---|
| `get_snapshot` | estado da conexão e o draft atual |
| `engine_status` | se o motor está pronto, patch, se há base global e histórico pessoal |
| `lane_profile` | o perfil de rota, ou `None` |
| `rank_draft`, `minimax_draft`, `ban_suggestions` | o draft ([02](02-motor-de-draft.md)) |
| `set_draft_role` | a rota escolhida na tela, para blind e normal |
| `set_enemy_role`, `get_enemy_roles` | a rota marcada à mão em cada inimigo |
| `set_simulated_draft` | o draft do simulador; `None` volta para o cliente |
| `get_draft_prefs`, `set_draft_prefs` | incluir quem a conta não pode escolher |
| `get_autostart`, `set_autostart` | iniciar com o Windows (desligado até o jogador ligar) |
| `get_pool`, `set_pool`, `pool_suggestions` | a pool |
| `lane_analysis(candidate)` | rotas, escaramuças e condição de vitória; com `candidate`, a prévia do draft com ele no seu lugar |
| `get_live`, `build_suggestions`, `skirmish_focus`, `gold_report` | a partida ([03](03-motor-da-partida.md)) |
| `rune_plan` | runas do confronto |
| `last_review` | a última leitura pós-partida |
| `get_overlay_prefs`, `set_overlay_prefs` | o que aparece no overlay |

### Eventos

| Evento | Quando |
|---|---|
| `presage://status` | a conexão com o cliente mudou |
| `presage://draft` | o champ select mudou |
| `presage://engine` | campeões carregados, ou o histórico pessoal terminou de atualizar — o front relê `lane_profile` aqui |
| `presage://live` | a partida mudou |
| `presage://review` | a leitura pós-partida ficou pronta; o front traz a aba para a frente |
| `presage://prefs` | as preferências do overlay mudaram |

### O que fica em disco

Tudo em `%LOCALAPPDATA%\Presage` (`storage.rs`):

| Arquivo | O quê |
|---|---|
| `pool.json` | a pool |
| `overlay.json` | preferências do overlay |
| `draft.json` | preferências do draft |
| `personal-v2.json` | histórico pessoal: vitórias por campeão e comparação de rota por partida ([04 §7](04-perfil-de-rota.md#7-o-cache)) |
| `base-de-partidas.json` | a base agregada, quando alguém a coloca ali ([05](05-coletor.md)) |
| `gravacao.json`, `ultima-partida.json` | a gravação da partida e a leitura dela |
| `champions.json`, `items.json`, `runas.json` | cache do Data Dragon, que não expira ([00 §6](00-contexto-e-limites.md#6-limites-conhecidos)) |

## 4. O front

Os componentes são puramente visuais — nenhum fala com o Tauri; `App.tsx` faz os
`invoke` e passa os dados. Por isso `demo.tsx` renderiza todos com dados de
exemplo, sem cliente nem casca ([08 §2](08-desenvolvimento-e-testes.md#2-olhar-a-interface-sem-o-cliente)).

`lib/types.ts` espelha à mão os tipos que a casca serializa (`camelCase`). Mudar
um tipo no Rust sem mudar aqui não quebra a compilação de nenhum dos dois lados:
quebra em tempo de execução.
