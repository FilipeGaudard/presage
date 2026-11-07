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
| célula | `cell_id`, `local_player_cell_id` | O lugar de cada jogador no champ select. O seu vem em `localPlayerCellId` |
| fase do champ select | `LcuTimer::phase`, `DraftState::phase` | `PLANNING`, `BAN_PICK` ou `FINALIZATION` |
| fase do cliente | `ConnStatus::phase` | A fase do fluxo de jogo (`/lol-gameflow/v1/gameflow-phase`): `Lobby`, `ChampSelect`, `InProgress`…; `Disconnected` sem cliente. Decide o ritmo do watcher |
| grade do champ select | `LcuGridChampion` | `/lol-champ-select/v1/all-grid-champions`: de onde sai a lista da conta |
| rotação grátis, Game Pass | `free_to_play`, `xbox_gp_reward` | Campeões que a conta escolhe sem ter. A rotação não vale na ranqueada (`RANKED_QUEUES`) |
| jogador local | `is_local`, `LiveGame::me` | Você, na Live Client: o jogador de `/activeplayername` |
| Dragão, Ancião, Barão, Arauto, Grubs | `Spawn::name` | Os nomes dos objetivos. `priority` e `lanes` os reconhecem pelo texto |
| alma | `Timeline::soul`, `Timings::dragons_for_soul` | Quatro dragões de um lado; depois dela o próximo dragão é o Ancião |
| classe | `Champion::tags`, `primary_tag` | As tags do Data Dragon: `Assassin`, `Fighter`, `Mage`, `Marksman`, `Support`, `Tank`. A primeira é a principal |
| chave textual, id numérico | `Champion::key`, `Champion::id` | `key` é o `id` textual do Data Dragon (`MonkeyKing`); `id` é o `key` numérico dele. Os nomes cruzam, e a chave não é o nome que a tela mostra |

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
| casca | `src-tauri`, `presage_lib` | A parte que faz I/O: LCU, Live Client, Riot Web API, Data Dragon, disco |
| comando | `#[tauri::command]`, `invoke` | O que a interface pede à casca ([12 §2](../arquitetura/12-contrato-da-casca.md#2-comandos)) |
| evento | `presage://…`, `listen` | O que a casca empurra sem pedido ([12 §3](../arquitetura/12-contrato-da-casca.md#3-eventos)) |
| estado compartilhado | `Shared` | O que o watcher, as cargas e os comandos dividem |
| espelho de tipos | `src/lib/types.ts` | A cópia à mão, em TS, dos tipos que a casca serializa. O doccheck confere os marcados com `↔` no 10 |
| instantâneo da partida | `LiveSnapshot` | Jogo, objetivos, prioridade, buffs e sua rota ao vivo, lidos no mesmo instante |

## Motor de draft

| Termo | No código | O que é |
|---|---|---|
| componente | — | Uma das seis parcelas do ranking: base, matchup, contra, sinergia, pessoal, composição |
| winrate base | `StatsStore::base`, `base_key` (`"103:Mid"`) | Taxa de vitória de um campeão numa rota |
| confronto | `StatsStore::matchup`, `matchup_key` (`"103:238:Mid"`) | A de um campeão contra outro na mesma rota. Direcional |
| rota marcada | `Leitura::enemy_roles`, `set_enemy_role` | A rota que você deu a um inimigo quando a presumida estava errada. Vence a inferência, nunca a posição do cliente |
| lista da conta | `Leitura::owned`, `draft::selectable` | Quem a sua conta pode escolher nesta fila: próprios, Game Pass e, fora da ranqueada, a rotação grátis |
| sua lista de bans | `Pool::bans`, `bans_with_pool` | Até cinco bans seus, em ordem, sugeridos antes da conta da composição |
| simulador | `set_simulated_draft`, `current_draft` | Draft montado à mão que o motor lê no lugar do champ select enquanto a aba está aberta |
| sua rota ao vivo | `rival.rs`, `Rival` | Você contra o oponente direto durante a partida, no que a rota cobra e em ouro de itens |
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
| contra | `StatsStore::versus`, `versus_key` (`"103:64"`) | O resultado de um campeão com outro em qualquer rota do time inimigo. Direcional |
| peso de rota | `Foe::lane_weight`, `lane_share` | O quanto um inimigo é mesmo o seu oponente de rota: 1 informado, a fração do encaixe quando inferido |
| duração | `Length`, `length_key` | Partida curta (abaixo de 25:00), média ou longa (a partir de 32:30), por `Length::of` |
| chance do time | `team::TeamEstimate` | Os dez campeões num modelo só, em log-odds. Só com a base |
| contexto do draft | `DraftContext` | A leitura única do champ select: sua rota, candidatos, aliados com hover, inimigos com rota inferida |
| rota inferida | `infer`, `roles_assumed` | A rota do inimigo que o cliente não informa, pela atribuição mais plausível. Na tela, "presumida" |
| quem joga a rota | `roles::plays`, `role_fit` | A base (5% das partidas), as classes, ou o seu próprio histórico |
| curva de nível | `LaneOutlook::curve` | O duelo de atributos do 1 ao 18, nivelados |
| vantagem de nível | `LaneOutlook::gaps`, `LevelOdds::gap` | Você um nível à frente ou atrás, no 3, 6, 11 e 16 |
| condição de vitória | `wincon::WinCondition` | Fatos de composição (regra) e onde a conta de atributos pende. Sem fase sem a base |
| prévia | `lane_analysis(candidate)` | A análise do draft com o candidato no seu lugar |
| pool | `Pool`, `PoolEntry`, `Tier` | Os campeões do jogador por rota: oito, três escolhidos e cinco recomendados |
| core | `Tier::Core`, `CORE_SIZE` | Os três que o jogador escolhe; a pool é montada em volta deles |
| situacionais | `Tier::Situational` | Os cinco que cobrem o que os core não cobrem |
| característica | `poolgap::Trait` | Dano físico ou mágico, linha de frente, alcance, corpo a corpo, resistências equilibradas, estilo. A pool cobre ou não |
| cobertura | `poolgap::suggest` | `1 − ½ⁿ` por característica; o próximo lugar é quem mais a sobe. Não é a cobertura da base |
| candidato | `DraftContext::candidates` | Disponível, na lista da conta, e que joga a sua rota ou está na pool |
| indisponível | `DraftContext::unavailable` | Banido, escolhido ou em hover de aliado; a sua escolha só depois de travar |
| oponente de rota | `DraftContext::lane_opponent` | O inimigo na sua rota no draft: informado, marcado ou inferido. Não é o oponente direto do perfil, que é o da partida real |
| posição informada | `Placed::informed` | A rota veio do cliente ou da sua marca, não da inferência |
| fração de rota | `roles::role_share` | A parte das partidas de um campeão em cada rota, pela base |
| razão | `Scored::reasons` | Até três motivos em texto de um candidato |
| completude | `TeamProfile::completeness` | Quanto do time já está definido: `known / 4` no ranking; o `team_value` usa `known / 5` |
| linha de frente | `Champion::is_frontline` | Classe `Tank` ou defesa ≥ 7 no `info` do Data Dragon |
| à distância | `Champion::is_ranged` | Alcance de ataque ≥ 500 |
| fração física | `Champion::physical_share` | Quanto do dano do campeão sai físico: metade `info`, metade prior de classe, escritos à mão |
| guloso | `MinimaxPick::greedy` | O valor do time com o candidato, sem a resposta deles. É também o que poda a busca (`breadth`). Não é o `rank()` |
| melhor resposta | `MinimaxPick::best_response` | O campeão deles que mais derruba o seu valor do time |
| valor do time | `minimax::team_value` | Índice de composição de um time contra o outro, em torno de 0,5. Não é a chance do time |
| ameaça | `BanSuggestion::threat` | Quanto um campeão completa a composição deles; é o que ordena o ban |
| nível de referência | `context::PREGAME_LEVEL` | O nível em que a prévia compara as rotas antes da partida |
| parcela | `TeamTerm` | Um termo da chance do time, em log-odds, com quanto ele move a chance sozinho e o n |
| assento | `team::Seat`, `Seat::certainty` | Um campeão de um lado, com a certeza de que a rota é aquela |
| fato, vantagem | `WinCondition::facts`, `WinCondition::edges` | Na condição de vitória: frase de composição por regra (`regra`) e rota ou escaramuça em que a conta de atributos pende (`atributos`) |
| inclinação | `wincon::analyze`, `LEAN` | `logit(longa) − logit(curta)` de um campeão na rota; a fase do jogo compara a média dos dois times |
| extra | `Tier::Extra` | Qualquer outro campeão que o jogador queira na pool, além dos core e dos situacionais. Vem por último na exibição (`Tier::weight`) |
| robustez | `poolgap::robustness` | A menor vida efetiva (dano todo físico ou todo mágico) dividida pela maior, no nível 11, sem itens |
| ponto forte, ponto fraco | `Analysis::strengths`, `Analysis::weaknesses` | O que a rota valoriza e algum core tem, ou nenhum tem |
| índice de encaixe | `Candidate::score` | Quanto um candidato da pool sobe a cobertura, de 0 a 1. Não é winrate |
| pontuação da runa | `RuneSuggestion::score` | A soma dos pesos das regras que dispararam. Passa de 1 e não é winrate |
| runa principal, secundária | `RunePlan::keystones`, `RunePlan::secondary` | Até três principais, pela soma das regras; uma secundária, a primeira regra que casa |
| explode | `runes::explode` | Assassino, ou mago que não é suporte: quem mata antes de a troca longa começar |
| estimativa | `Estimate` | `{ rate, effective_n, games }`: a taxa encolhida, ou o desvio dela quando sai de `shrunk_delta` |
| n efetivo | `Estimate::effective_n` | Partidas reais mais a força do prior |
| erro padrão | `Estimate::se` | O `±` da winrate base |
| base utilizável | `Dataset::is_usable` | Tem partidas e tem `base`; sem isso o app fica sem base |
| partida vista | `PersonalStats::seen`, `mark_seen` | Já buscada no histórico pessoal: contada ou descartada |

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
| observado, estimado | `Spawn::observed`, `Timings` | Timer de objetivo visto no fluxo de eventos ou tirado de constante. O código chama de "assumido" o que a tela chama de "estimado" |
| gravação | `Recording`, `gravacao.json` | A partida quadro a quadro, gravada na máquina do jogador |
| leitura pós-partida | `coach::review`, `Report`, `ultima-partida.json` | Notas boas e ruins sobre a partida gravada |
| duelo | `LaneOutlook::duel`, `skirmish_odds` | O 1v1 de atributos nos níveis e itens de agora, antes do peso da rota |
| níveis-chave | `BREAKPOINTS` | 3, 6, 11 e 16: onde a curva e a vantagem de nível olham |
| divisão do dano | `incoming_physical_share` | A fração física do dano que vem do time inimigo |
| dano efetivo | `offense_against` | AD e AP passados pelas resistências do alvo, somados um para um. Proxy, não DPS |
| valor por 1000 de ouro | `ItemSuggestion::value_per_1000g` | Ganho de poder em % por 1000 de ouro. Índice |
| combatente | `Combatant`, `Loadout` | Campeão, nível e itens; o `Loadout` são os atributos somados |
| pior penetração | `team_penetration` | O maior percentual e a maior letalidade entre os inimigos, juntos, mesmo que de campeões diferentes |
| penetração | `Penetration` | O quanto da resistência do alvo o dano ignora: percentual e plana |
| letalidade | `ItemStats::lethality` | A penetração plana de armadura |
| resistência efetiva | `effective_resist` | A resistência que sobra depois da redução e da penetração |
| crescimento por nível | `grown` | `base + ganho × n × (0,7025 + 0,0175n)`, com n = nível − 1 |
| ouro em itens | `GoldPlayer::invested`, `LivePlayer::gold_invested` | Soma do preço dos itens do inventário: exato |
| ouro ganho | `GoldPlayer::earned`, `earned_at` | Renda reconstruída de tempo, tropas, abates e assistências: estimativa |
| ouro em mão | `GoldPlayer::unspent` | Ganho menos investido: estimativa. Pela C-4 só o seu pode aparecer |
| ouro parado | `IDLE_GOLD` | No pós-jogo: ouro em mão estimado acima de 1300, vivo, por tempo |
| destaque | `lanes::focus`, `Focus` | A escaramuça que importa para a sua rota agora, a única que cabe no overlay. `SkirmishFocus` no front |
| foco do objetivo | `PriorityReport::focus` | O texto do objetivo mais próximo a até 90 s. Não é o destaque |
| agrupamento | `GROUPS`, `Skirmish::key` | Topo, meio, bot, dragão, arauto: quem briga junto numa escaramuça |
| na frente, igual, atrás | `rival::Standing` | `Ahead`, `Even`, `Behind`: você contra o oponente direto, pela margem de ouro em itens |
| margem de ouro | `AHEAD_FLOOR`, `AHEAD_SHARE` | `max(300, 10% do ouro em itens do oponente)` |
| estrutura | `Respawn` | Inibidor ou torre do nexo esperando voltar |
| buff | `Buff`, `BuffTracker` | Barão ou Ancião: tempo restante e quem ainda tem (`holders`) |
| cobrança | `coach::cobranca` | O que a rota tem como trabalho, e portanto o que se cobra: farm, visão e os limiares de participação. `rival.rs` usa a mesma |
| participação em abates | `kp_bad`, `kp_good` | (abates + assistências) / abates do time, em fração |
| quadro | `Recording::frames` | Um `LiveGame` guardado na gravação, cerca de um por segundo |
| curva | `Series` | As séries do pós-jogo, reamostradas a cada 30 s |
| observação | `Note`, `Verdict` | Uma linha do pós-jogo, a manter (`good`) ou a corrigir (`bad`) |
| item grande | `LEGENDARY_COST` | Item de 2500 de ouro ou mais |
| mortes em sequência | `death_clusters` | Três ou mais mortes em 240 s |
| build adaptativa | `build::suggest`, `LiveBuild` | Os próximos itens contra o que eles têm agora |
| overlay | janela `overlay`, `OverlayPrefs` | A janela externa por cima do jogo ([ADR-0003](../adr/0003-overlay-como-janela-externa.md)) |
| seção do overlay | `Section`, `OverlayPrefs::hidden` | Uma das seis linhas que o jogador liga e desliga; guarda-se o que está desligado |
| atravessável, clicável | `set_ignore_cursor_events` | Os dois modos do overlay; `Ctrl+Shift+O` alterna |
| bandeja | `bandeja` | O ícone perto do relógio, com "Mostrar o Presage" e "Sair" |
| iniciar com o Windows | `set_autostart` | Desligado até o jogador ligar ([ADR-0026](../adr/0026-lcu-so-leitura-e-autostart-desligado.md)) |
| demonstração | `demo.tsx` | As telas com dados de exemplo, sem cliente nem casca |

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
| descarte | `Skip` | Por que uma partida ficou de fora. Há dois tipos com esse nome: `history::Skip`, do histórico pessoal, e o do coletor (`aggregate.rs`) |
| arquivo de estado | `Estado`, `coleta-em-andamento.json` | O que o coletor grava para `--retomar`: jogadores, partidas vistas e contagens |
| recorte | `Estado::combina` | Região, tier e fila: o que `--retomar` confere antes de continuar |
| rótulo de elo | `rotulo_de_elo` | `EMERALD+` só quando a semeadura chegou ao topo |
| liga de topo | `is_apex` | Mestre, grão-mestre e desafiante, cada um numa lista única |
| limitador | `Limiter` | Segura as chamadas dentro do limite de taxa da chave |
| taxa sustentável | `sustained_rate` | O ritmo que a janela mais apertada permite no longo prazo |
| poda | `--min-jogos`, `Counts::into_dataset` | Tirar da base as células com poucas partidas |
| entrada útil | `USEFUL` | Célula da base com 200 partidas ou mais |
| cobertura da base | `Coverage` | Quanto da base tem amostra útil. Não é a cobertura da pool |
| a revisar | — | Decisão do cruzamento com o iTero que o dono do produto ainda não confirmou ([07](../referencia/itero/07-cruzamento.md)) |
| calibração | — | Quantas vezes o resultado bateu com a chance que o app disse (I-75). Não existe ainda |
