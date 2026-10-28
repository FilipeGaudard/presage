# Glossário

O vocabulário do jogo, do produto e do código, com o nome que cada coisa tem no
código. Quando o doc diz "confronto", o código diz `matchup`.

## Jogo e Riot

| Termo | No código | O que é |
|---|---|---|
| rota | `Role`: `Top`, `Jungle`, `Mid`, `Bot`, `Support`, `Unknown` | A posição. O LCU escreve `top/jungle/middle/bottom/utility`; a match-v5, `TOP/JUNGLE/MIDDLE/BOTTOM/UTILITY`. `Role::from_lcu` lê os dois |
| draft, champ select | `DraftState`, `DraftSlot` | A fase de 30 segundos por turno em que se bane e escolhe |
| hover | `intent_id` | Campeão em destaque antes de travar. **Só existe para o seu time**: o LCU não revela o do inimigo |
| travar | `champion_id > 0`, `locked` | Confirmar o pick |
| patch | `patch_of("15.21.704.1234") == "15.21"` | A versão do jogo. A base de partidas vale para um só |
| remake | `gameEndedInEarlySurrender` | Partida encerrada no começo. Não conta para nada |
| puuid | `puuid` | O identificador permanente da conta |
| Riot ID | `nome#tag`, `split_riot_id` | O nome visível da conta |
| plataforma, região | `Region { platform, routing }` | `br1` (plataforma, para `league-v4`) e `americas` (região, para `match-v5` e `account-v1`). A Riot conta limite de taxa por host |
| Vanguard | — | O anticheat. Bloqueia hook de render, e por isso o overlay é janela externa ([ADR-0003](../adr/0003-overlay-como-janela-externa.md)) |

## Fontes de dado

| Termo | No código | O que é |
|---|---|---|
| LCU | `lcu/`, `LcuSession` | A API local do cliente da Riot: HTTPS no loopback, porta e senha novas a cada execução |
| lockfile | `parse_lockfile` | `LeagueClient:<pid>:<porta>:<senha>:https` na raiz da instalação |
| Live Client Data API | `live.rs`, `LiveGame` | A API local da partida, `127.0.0.1:2999`, sem autenticação |
| Riot Web API | `riot.rs`, `RiotClient` | A API pública: `account-v1`, `league-v4`, `match-v5`. Exige chave |
| match-v5 | `MatchDto` | O formato da partida completa: ~150 campos por participante |
| challenges | `Challenges` | O bloco da match-v5 com métricas derivadas (`laneMinionsFirst10Minutes`, `killParticipation`…) |
| timeline | — | Endpoint à parte da match-v5, quadro a quadro. Não usado ([P-3](perguntas-em-aberto.md)) |
| Data Dragon | `ddragon.rs`, `ChampionIndex`, `ItemIndex`, `RuneIndex` | O CDN público de campeões, itens, runas e ícones |

## Motor de draft

| Termo | No código | O que é |
|---|---|---|
| componente | — | Uma das cinco parcelas do ranking: base, matchup, sinergia, pessoal, composição |
| winrate base | `StatsStore::base`, `base_key` (`"103:Mid"`) | Taxa de vitória de um campeão numa rota |
| confronto | `StatsStore::matchup`, `matchup_key` (`"103:238:Mid"`) | A de um campeão contra outro na mesma rota. Direcional |
| sinergia | `StatsStore::synergy`, `synergy_key` (`"64:103"`, menor id primeiro) | A de dois campeões no mesmo time |
| registro | `Record { games, wins }` | Partidas e vitórias. `wins` nunca passa de `games` |
| encolher (shrinkage) | `shrink`, `shrunk_delta` | Puxar uma taxa de amostra pequena para o prior: média posterior de uma Beta |
| força do prior | `strength_for_sd` | Quantas partidas fictícias o prior vale |
| espalhamento | `*_PRIOR_SD` | O desvio típico de um efeito na população. É dele que a força sai ([ADR-0009](../adr/0009-forca-do-shrinkage-derivada-do-desvio.md)) |
| referência | `expected_against`, `expected_together` | Contra o que um efeito é medido: o que as duas forças já explicam ([ADR-0008](../adr/0008-matchup-e-sinergia-contra-a-soma-das-forcas.md)) |
| log-odds | `logit` | `ln(p / (1 − p))`. Forças se somam aqui, não em probabilidade |
| desvio posterior | `posterior_sd` | A incerteza de um componente depois da amostra. Cai com a raiz dela |
| delta | `Scored::delta` | Desvio em pontos percentuais contra uma partida neutra. É o número embaixo do retrato |
| fit de composição | `comp::profile`, `damage_balance_gain`, `frontline_need` | Equilíbrio de dano e frontline do time com o candidato |
| exposição | `MinimaxPick` | Quanto um pick perde depois da melhor resposta deles |
| fase da conta | — | O quanto a conta está ganhando agora, em todos os campeões. Vaza no componente pessoal ([ADR-0012](../adr/0012-vazamento-da-fase-da-conta-mantido.md)) |
| termo de experiência | — | O que falta para corrigir o vazamento: campeão nunca jogado rende abaixo da sua média ([P-4](perguntas-em-aberto.md)) |
| camada global, camada pessoal | `Layered { global, personal }` | A base de partidas e o histórico da conta, independentes |
| base de partidas | `Dataset`, `base-de-partidas.json` | O arquivo agregado que o coletor produz |
| pool | `Pool`, `PoolEntry`, `Tier` | Os campeões do jogador por rota: oito, três escolhidos e cinco recomendados |
| core | `Tier::Core`, `CORE_SIZE` | Os três que o jogador escolhe; a pool é montada em volta deles |
| situacionais | `Tier::Situational` | Os cinco que cobrem o que os core não cobrem |
| característica | `poolgap::Trait` | Dano físico ou mágico, linha de frente, alcance, corpo a corpo, resistências equilibradas, estilo. A pool cobre ou não |
| cobertura | `poolgap::suggest` | `1 − ½ⁿ` por característica; o próximo lugar é quem mais a sobe |

## Perfil de rota

| Termo | No código | O que é |
|---|---|---|
| oponente direto | `LaneGame::opponent` | Quem jogou a mesma rota do outro lado, na mesma partida |
| diferença | `LaneGame::diffs` | Você menos ele, por métrica, na unidade de tela |
| métrica | `Metric`: `Cs10`, `KillParticipation`, `DamageShare`, `VisionPerMinute`, `Deaths` | O que se compara |
| leitura | `Reading { raw, value, sd }` | Uma métrica num nível: média crua, valor encolhido, desvio |
| clara | `Reading::is_clear` | O intervalo de 95% não cruza o zero. Só o que é claro ganha cor ou vira razão |
| nível | `LaneProfile`, `ChampionLane`, `MatchupLane` | Você; você com o campeão; você com o campeão contra aquele oponente |
| espalhamento entre níveis | `between_variance` | O τ² de DerSimonian–Laird: quanto as médias se espalham além do ruído |

## Partida e pós-jogo

| Termo | No código | O que é |
|---|---|---|
| poder de combate | `side_power` | Vida efetiva × dano efetivo |
| vida efetiva | `effective_hp`, `effective_hp_against` | Vida corrigida pelas resistências contra o dano que vem |
| escaramuça | `skirmishes`, `Skirmish` | Luta 2v2 ou 3v3, com a mesma conta do duelo somada por grupo |
| prioridade | `priority::report` | Onde está o valor no mapa. Nunca onde o inimigo está |
| observado, estimado | `Timeline`, `Timings` | Timer de objetivo visto no fluxo de eventos ou tirado de constante |
| gravação | `Recording`, `gravacao.json` | A partida quadro a quadro, gravada na máquina do jogador |
| leitura pós-partida | `coach::review`, `Report`, `ultima-partida.json` | Notas boas e ruins sobre a partida gravada |

## Coletor e distribuição

| Termo | No código | O que é |
|---|---|---|
| coletor | `tools/ingest`, `presage-ingest` | O binário que produz a base de partidas |
| semear | `--tier`, `league-v4` | Escolher os jogadores de partida do crawl |
| bola de neve | — | Expandir a partir dos dez de cada partida. Recusada ([ADR-0006](../adr/0006-coletor-sem-bola-de-neve-um-patch-partida-inteira.md)) |
| retomar | `--retomar` | Continuar um crawl interrompido |
| atualizador | `plugins.updater` | O plugin do Tauri que baixa e instala versão nova |
| manifesto | `latest.json` | Versão, URL e assinatura da versão nova |
| assinatura | `.sig`, minisign | O que o app confere com a chave pública embutida antes de executar |
