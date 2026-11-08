# 01 — Core e casca

O workspace separa a lógica pura da casca ([ADR-0001](../adr/0001-core-puro-separado-da-casca.md)).

## 1. O workspace

```
core/src/                    presage-core — sem Tauri, sem rede, sem disco
  draft.rs                   LCU cru            ->  DraftState
  live.rs                    Live Client cru    ->  LiveGame
  history.rs                 match-v5 crua      ->  PlayedGame, LaneGame
  lockfile.rs                credenciais do cliente: lockfile (a linha de comando, sem uso)
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
  engine/team.rs             a chance do time, com os dez campeões, em log-odds
  engine/wincon.rs           condição de vitória: composição por regra, duelos por atributos
  engine/rival.rs            você contra o oponente direto, durante a partida
  engine/minimax.rs          draft com resposta e sugestão de ban
  engine/poolgap.rs          retrato dos core e os cinco que completam a pool
  engine/stats.rs            shrinkage, referência, StatsStore, PersonalStats
  engine/dataset.rs          formato da base de partidas, Layered
  engine/profile.rs          perfil de rota em três níveis
  engine/mod.rs              rank(): junta tudo e propaga incerteza

src-tauri/src/               a casca
  lcu/credentials.rs         acha o lockfile: pastas conhecidas, configuração da Riot, executável
  lcu/client.rs              HTTP com Basic Auth sobre o TLS autoassinado, só GET
  live.rs                    Live Client Data API, 127.0.0.1:2999
  riot.rs                    Riot Web API, chave vinda do ambiente
  profile.rs                 histórico pessoal, incremental e cacheado
  ddragon.rs                 campeões, itens e runas, com cache em disco
  analysis.rs                escolhe a fonte (partida ou draft) e chama engine::report
  recorder.rs                grava a partida e dispara a leitura no fim
  overlay.rs                 a janela do overlay e os atalhos
  watcher.rs                 polling adaptativo, o estado compartilhado, emite eventos
  storage.rs                 JSON em %LOCALAPPDATA%\Presage
  lib.rs                     os comandos, a bandeja, o autostart e a subida do app
  main.rs                    só chama lib::run

src/                         React
  App.tsx                    a janela principal: todos os invoke e listen dela
  overlay.tsx                a janela do overlay: os invoke e listen dela
  lib/ddragon.ts             ícones e nomes, baixados direto do Data Dragon
  lib/types.ts               os tipos que a casca devolve, espelhados à mão
  components/                as telas, puramente visuais
  demo.tsx                   a interface com dados de exemplo, sem Tauri

tools/ingest/src/            o coletor — binário à parte, fora do instalador
  aggregate.rs               partida crua -> contagens (puro, testado)
  limiter.rs                 janela deslizante no formato do cabeçalho da Riot
  api.rs                     cliente com dois limites, 429 e retentativa
  main.rs                    CLI, semeadura, retomada e cobertura
```

Os componentes de `src/components/` e o que cada tela mostra estão em
[frontend 02](../frontend/02-telas-e-fluxos.md).

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
dá erro ([P-5](../produto/perguntas-em-aberto.md)), e por isso cada fronteira tem
um teste que lê JSON com os nomes de fora e confere cada campo fora do padrão
(A19):

| Fronteira | Teste com JSON |
|---|---|
| match-v5 (`history::played`) | `le_os_nomes_de_campo_da_riot`, com JSON escrito à mão a partir da documentação, `jungleCsBefore10Minutes` inclusive |
| grade do champ select, sessão do fluxo de jogo | `na_ranqueada_a_rotacao_gratis_nao_vale`, `a_fila_sai_da_sessao_do_fluxo_de_jogo` |
| sessão do champ select e invocador (`LcuSession`, `LcuAction`, `LcuPlayer`, `LcuTimer`, `LcuSummoner`) | `le_os_nomes_de_campo_do_lcu` |
| Live Client (`RawPlayer`, `RawItem`, `RawScores`) | `le_os_nomes_de_campo_da_live_client`, com o `itemID`, o caso em que `camelCase` não basta |
| eventos da Live Client (`RawEvent`, `EventFeed`) | `le_os_nomes_de_campo_dos_eventos`, em PascalCase |

O JSON desses testes é escrito à mão a partir do que a Riot documenta e do que
apps em produção leem. Ele trava o que assumimos, não o que a Riot manda: a
conferência numa partida real continua em P-5 e P-14.

A falta está no `build/audit.md` (A19).

O coletor tem os próprios tipos crus da match-v5 em `aggregate.rs`. Quatro regras
de descarte são as de `history::played` escritas de novo (modo, terminada,
remake, curta); o coletor tem mais cinco, e a rota em branco tem tratamento
oposto nos dois ([05 §5](05-coletor.md#5-a-duplicação-com-historyrs)). São duas
cópias da mesma leitura — unificar é trabalho a fazer, e até lá as duas precisam
mudar juntas.

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

A interface chama 27 comandos por `invoke`, todos registrados em
`generate_handler!` (`lib.rs`). Argumentos, retorno, erros e quem chama cada um
estão no [contrato da casca](12-contrato-da-casca.md#2-comandos).

### Eventos

A casca empurra seis eventos `presage://…`, quase todos do `watcher.rs`. A carga,
o momento e quem ouve cada um estão no
[contrato da casca](12-contrato-da-casca.md#3-eventos).

### O que fica em disco

Tudo em `%LOCALAPPDATA%\Presage` (`storage.rs`). Os arquivos, quem escreve e quem
lê cada um estão no [contrato da casca](12-contrato-da-casca.md#6-disco).

## 4. O front

Só `App.tsx` e `overlay.tsx` falam com a casca: fazem os `invoke`, ouvem os eventos
e passam os dados aos componentes. Os componentes são visuais. A exceção é
`Atualizacao.tsx`, que fala com o plugin do atualizador, não com um comando nosso.
Por isso `demo.tsx` renderiza os componentes com dados de exemplo, sem cliente nem
casca ([08 §2](08-desenvolvimento-e-testes.md#2-olhar-a-interface-sem-o-cliente)).

`lib/types.ts` espelha à mão os tipos que a casca serializa (`camelCase`). Mudar
um tipo no Rust sem mudar aqui não quebra a compilação de nenhum dos dois lados:
quebra em tempo de execução. Quem pega a divergência é o `doccheck`, nos tipos
que o modelo de dados marca com `↔`. Os nomes que mudam de um lado para o outro
estão no [contrato da casca](12-contrato-da-casca.md#4-nomes-de-um-lado-e-do-outro).
