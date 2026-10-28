# 06 — Dados e backend do iTero

O que sai da máquina, para onde, com que forma. Hosts, ids de distribuição e o
token público da loja **não** são reproduzidos aqui: não acrescentam nada ao
entendimento e não são nossos.

## 1. Fontes

| Fonte | Para quê |
|---|---|
| **Backend do iTero** (AWS API Gateway em 5 regiões: eu-central-1, us-east-2, sa-east-1, ap-southeast-1, ap-northeast-2) | todo o modelo: draft, scouting, conta, coach, pós-partida |
| **CloudFront do iTero** (duas distribuições) | arquivos estáticos: winrate de ARAM, tier list, médias de stat por elo, rotas padrão, tabela de stats por campeão, filtro de itens; reserva de dados do jogo |
| `api.itero.gg` | quiz de personalidade |
| **Firebase** (Auth, Firestore, Functions) | conta, configurações, assinatura, relatório de erro |
| **Tebex** headless | pacotes e checkout da assinatura |
| **Overwolf** | GEP (eventos de launcher e jogo), overlay, status dos eventos, atualização, anúncios, CMP, hash de e-mail |
| **LCU** (cliente do League, local) | campeões que você tem, configurações de jogo, champ select (leitura pelo GEP, escrita direta), runas, conjuntos de itens |
| **Live Client Data** (local, via GEP `live_client_data`) | jogadores, eventos, jogador ativo |
| CommunityDragon e Data Dragon | dados de campeão, item, runa, feitiço, aprimoramento; versão do patch |

O iTero **não** chama a Riot Web API do cliente; quem tem a chave é o backend
dele.

Roteamento por região (`main.js:25168`): `euw1`, `eun1`, `me1`, `ru`, `tr1` →
eu-central-1; `na1` → us-east-2; `br1`, `la1`, `la2` → sa-east-1; `jp1`, `kr` →
ap-northeast-2; `oc1`, `ph2`, `sg2`, `th2`, `tw2`, `vn2` → ap-southeast-1.
Região desconhecida cai em `euw1`.

As chamadas ao backend são JSON sem cabeçalho de autenticação; o usuário é
identificado pelo puuid ou pelo Riot ID no corpo. Toda resposta vem como
`{ status_code, data }` e é validada com zod; divergência de esquema vira evento
de telemetria (`api.schema_mismatch.*`) e o valor de reserva.

## 2. Endpoints do backend

`{r}` é a base regional; `{rota}` em minúsculas (`top`, `jungle`, `middle`,
`bottom`, `utility`).

| Endpoint | Corpo | Resposta | Quem chama |
|---|---|---|---|
| `POST {r}/lake-draft-v2-{rota}` `action: "recommend"` | `mastery_data`, `t1_side`, `bans`, `rank_class`, composição (`t1_*`, `t2_*`), `user_model_weightings` | lista de picks: campeão, `pick_perc`, `laning_pred_rating`, notas | `[Draft]`, `main.js:25331` |
| idem `action: "analyse"` | + `cs_rank_class`, `ignore_builds`, `extra_logs` | análise: notas, matchups, fase de rota, stats do time, `team_comps`, builds, `cs_tracker` | `main.js:25296` |
| `POST {r}/lake-draft-{rota}` `action: "cpc_blinds"`, `"cpc_recs"`, `"cpc_damage_check"`, `"cpc_summary"` | `rank_class`, maestria, `inc_mastery`, picks atuais, rota alternativa, pocket pick | sugestões do construtor de pool | `a3MS-J-6.js` |
| `POST {r}/lake-team-scouting` | puuid, região, os dez `{game_name, tag_line, champion}` por posição | os dez cartões ([05 §2](05-funcionalidades.md#2-scouting-de-jogadores)) | `main.js:36862` |
| `POST {r}/lake-basic-info` `update_type: "ALL"` | `game_name`, `tag_line`, região, `itero_plus`, `match_data`, `force_account_summary` | info (nome, puuid, nível, ícone, região, região maior), maestria por campeão, elo | `main.js:37442` |
| idem `update_type: "RANK"` | puuid, região | elo atual | `main.js:37780` |
| `POST {r}/lake-match-analysis` | puuid, região, região maior, maestria, `rank_data`, `itero_plus` | dispara a análise da última partida | `main.js:37805` |
| `POST {r}/lake-request-data` `stat_name: "match_history"` | puuid, `limit` (padrão 10), `before_ts`, fila, rota, campeões | partidas | `main.js:37568` |
| idem `"match_timeline"`, `"account_summary"`, e pedidos `match_detail`, `individual_stat`, `similar_stats` | puuid, partida, estatísticas, faixa | linha do tempo, resumo da conta, detalhe, estatística individual, parecidas | renderer |
| `POST {r}/lake-leaderboard` `action: "update"` | puuid, `itero_plus`, `itero_ultra` | amigos com elo e `data_available` | `main.js:37492` |
| `POST {r}/lake-coach-{rota}` | `request_type`, puuid, fila | forças e pontos de melhoria | `_89qKOxY.js` |
| `POST {r}/lake-flame-me` | nome, puuid, tipo (elogio/crítica), rota, fila, idioma | texto gerado | `DF5z8Cgx.js` |
| `GET {r}/lake-regional-pricing` | — | moeda e preço | `main.js:36207` |
| backend: builds e aprimoramentos de ARAM por campeão | apelido do campeão | builds com raridade; tiers S–D | `api:get-aram-champion-*` |

## 3. Esquemas de destaque

**Jogador no scouting** (`main.js:36809`): `game_name`, `tag_line`,
`champion`, `champ_id`, `puuid?`, `rank {tier, rank, lp}` (PDL negativo vira
nulo), `stats {CHAMPION, OVERALL}` com as 19 métricas, `labels[] {name, stat,
stat_type, category}`.

**Info básica** (`main.js:37425`): `info {name, player_id, player_puuid,
level, icon, region, major_region}`, `mastery {campeão: pontos}`,
`current_rank {rank, tier, lp} | null`.

**Live Client — todos os jogadores** (`main.js:36763`+): `championName`,
`rawChampionName`, `isDead`, `isBot`, `level`, `team` (`ORDER`/`CHAOS`),
`riotIdGameName`, `riotIdTagLine`, `items[] {itemID, price, count}`,
`position`.

**Eventos do launcher** (`main.js:38193`): `summoner_info.player_info`
(nome, tag, summonerId), `.platform_id`, `.profile_icon_id`, `.locale`;
`game_info.game_version` (o iTero soma 10 ao primeiro número para casar com o
patch exibido), `game_info.selected_positions` (primeira e segunda preferência;
vira a posição do quick play), `game_info.quickplay_champions` (o campeão do
quick play); `lobby_info.queueId`; `lcu_info.token`, `.port`;
`game_flow.phase` (13 fases); `champ_select.raw` (a sessão do LCU) e
`.aram_champ_select`.

**Configurações de jogo** (LCU): `windowMode`, `globalScale`, `minimapScale`,
`flipMiniMap`, `showTeamFramesOnLeft`.

**Grade de campeões** (LCU): `id`, `owned`, `freeToPlay`, `xboxGPReward`.

## 4. Chamadas ao LCU

Autenticação Basic com o token que o GEP entrega (`lcu_info`), na porta
informada (`main.js:15165`).

| Método e caminho | Para quê |
|---|---|
| `GET /lol-champ-select/v1/all-grid-champions` | campeões que você tem |
| `GET /lol-game-settings/v1/game-settings` | posicionar overlays |
| `PATCH /lol-champ-select/v1/session/actions/{id}` | hover, lock, hover de ban |
| `POST /lol-champ-select/v1/session/bench/swap/{id}` | trocar com o banco do ARAM |
| `PATCH /lol-champ-select/v1/session/my-selection` | feitiços |
| `GET/POST/PUT /lol-item-sets/v1/item-sets/{summonerId}/sets` | conjunto de itens "iTero" |
| `GET /lol-perks/v1/pages`, `GET /lol-perks/v1/currentpage`, `POST /lol-perks/v1/pages`, `PUT /lol-perks/v1/pages/{id}` | página de runas "iTero" |

## 5. Firestore

| Caminho | Conteúdo |
|---|---|
| `users/{uid}` | configurações do usuário (merge) |
| `users/{uid}/subscription/current` | `status`, `startedAt`, `expiresAt`, `willRenew`, `trialActivatedAt`, `trialEndAt` |

Função na nuvem: `reporterror`.

## 6. Telemetria e anúncios

- Eventos nomeados (`api.request_failed.*`, `api.schema_mismatch.*`,
  `auto_launch.state_mismatch`) com URL, método, corpo, status e erro; chaves
  `Authorization` e `Password` são removidas antes do envio (`main.js:29003`).
- Anúncios do Overwolf na janela principal para quem não assina; e-mail em hash
  para personalização (padrão ligado, desligável); CMP do Overwolf para
  consentimento.

## 7. O que fica na máquina

`electron-store`: `settings` ([01 §8](01-processo-principal.md#8-configurações-do-usuário-settings)),
`window-state` e `config` (`main.js:13141`), com estas chaves:

| Grupo | Chaves |
|---|---|
| primeira vez | `ftueSeen`, `draftingFtueSeen`, `aiCoachIntroSeen`, `iteroPlusRequiredForCustomPopupSeen`, `accountStatisticsInfoAlertSeen`, `freeTrialEndedPopupSeen`, `showPopup` |
| versões e status | `appVersion`, `leagueVersion`, `owGameEventsStatus`, `owLauncherEventsStatus` |
| dados do jogo | `leagueChampSummary`, `leagueItemData`, `leagueRunes`, `leagueRuneTrees`, `leagueSummonerSpells`, `cherryAugmentsDefaultNames`, `cherryAugmentsLocaleIds` |
| dados do iTero | `aramWinRates`, `matchItemsFilter`, `eloStatAverages`, `banRecommendations`, `keyMatchups`, `champDefaultLanes`, `championTableStats`, `tebexPackages`, `subscriptionPrice` |
| monitores | `primaryDisplay`, `secondaryDisplay`, `gameDisplay` |
| usuário | `basicInfo`, `friends`, `latestMatchId`, `ownedChampions`, `ownedChampionsInclF2P` |
| configurações de jogo | `windowMode`, `globalScale`, `minimapScale`, `flipMiniMap`, `showTeamFramesOnLeft` |
| autostart e uso | `hasSetAutoLaunch`, `autoLaunchApp`, `resetAutoLaunch`, `launchCount` | Logs em pasta própria (abríveis pela UI). Arquivos-marca em
`%TEMP%` (admin) e em `userData` (relançamento pós-atualização).
