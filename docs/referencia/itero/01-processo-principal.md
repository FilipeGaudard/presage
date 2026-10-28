# 01 — Processo principal do iTero

Tudo o que roda no processo Electron principal (`dist-electron/main.js`, 38.679
linhas formatadas, nomes de variável minificados, rótulos de log legíveis). Cada
módulo é nomeado pelo **rótulo de log** que o próprio iTero usa (`[Phase]`,
`[Draft]`), porque é o único nome estável no bundle. A linha é a do `main.js` da
versão 3.11.7.

## 1. Inicialização

| Ordem | O que faz | Onde |
|---|---|---|
| 1 | Detecta se abriu pelo autostart do Windows e se roda como administrador (`--admin` ou um arquivo-marca em `%TEMP%\itero-is-running-in-admin.flag`) | `main.js:38592`, `38609` |
| 2 | Trava de instância única. Se já existe uma instância **sem** admin e esta tem admin, a nova continua e a antiga se fecha (`second-instance` com `isAdmin`) | `main.js:38609-38650` |
| 3 | `before-quit`: segura o fechamento 800 ms, manda `before-quit-cleanup` com as configurações para a janela `background` (que salva no Firestore) e só então sai | `main.js:38630` |
| 4 | `resume` do sistema: 3 s depois, se online, dispara o refetch de rede | `main.js:38636` |
| 5 | `onAppReady`: lê a marca de "reaberto após atualização" (válida por 5 min), loga a sessão, incrementa `launchCount` (exceto após atualização), inicia os pacotes Overwolf, registra o protocolo `itero-app://`, cria o ícone da bandeja, observa monitores, cria a janela `background` e abre a principal se não veio do autostart | `main.js:38652-38674` |
| 6 | `ConfigData.updateAll()` agora e a cada **12 h** | `main.js:38670` |

**Bandeja** (`main.js:38547`): "Show Main Window", "Open Logs Folder",
"Relaunch", "Quit"; clique abre a janela principal.

**Deeplink** (`[Deeplink]`, `main.js:38560`): registra `itero-app` como
protocolo padrão. O manipulador só loga a URL — não há rota acionada por ela
nesta versão.

**Atualizador** (`[AutoUpdater]`, `main.js:35876`): `electron-updater` com feed
genérico em `electron-updates.overwolf.com/electron-updates/electron/<appId>`.
Ao terminar o download grava a marca de relançamento (lembrando se a janela
principal estava aberta) e chama `quitAndInstall(silencioso, reabrir)`. O
`package.json` ainda aponta um provedor GitHub (`nhui98/itero-electron`), que o
`setFeedURL` sobrescreve.

**Desinstalar** (`[Uninstall]`, `main.js:26384`): executa
`Uninstall The iTero Coach.exe` ao lado do executável e sai.

**Reiniciar como admin** (`main.js:26371`): caixa "League is running as
administrator — Restart iTero with admin privileges?"; aceita → relança elevado.
Necessário porque o Overwolf não injeta em jogo elevado.

## 2. Pacotes Overwolf

O iTero é um app **Overwolf-Electron**: o Overwolf entrega dois pacotes nativos,
e quase todo dado ao vivo vem deles, não da API da Riot.

| Pacote | Para quê | Onde |
|---|---|---|
| `gep` (Game Events Provider) | eventos do **launcher** (id 10902) e do **jogo** (id 5426) | `[OverwolfGepPackage]`, `main.js:38399` |
| `overlay` | janelas injetadas no jogo + atalhos globais | `[OverwolfOverlayPackage]`, `main.js:38453`; `[Hotkey]`, `main.js:14819` |

`[OverwolfService]` (`main.js:38518`) espera o evento `ready` de cada pacote e
inicia o módulo correspondente; também liga o atualizador.

**GEP — launcher** (`[Launcher]`, `main.js:38365`), recursos pedidos:
`summoner_info`, `lcu_info`, `game_flow`, `lobby_info`, `champ_select`,
`game_info`. Ao detectar o launcher, abre a janela principal se
`autoLaunchDesktop`.

**GEP — jogo** (`[Game]`, `main.js:37373`), recursos pedidos:
`live_client_data`, `match_info`, `death`, `kill`, `assist`, `gold`, `minions`,
`summoner_info`, `teams`, `level`, `announcer`, `damage`, `counters`,
`jungle_camps`, `team_frames`, `augments`. `setRequiredFeatures` tem
retentativas.

**GEP — eventos de erro**: `elevated-privileges-required` marca
`elevatedPrivilegesRequired` no estado do launcher (a UI pede reiniciar como
admin).

**Overlay** (`main.js:38453`): registra o jogo 5426; em `game-launched` injeta,
exceto se o processo do jogo estiver elevado; em `game-window-changed` atualiza
o monitor e o tamanho reais do jogo (`[Display]`), que as janelas de overlay usam
para se posicionar sobre o minimapa e os quadros do time.

**Canais Overwolf** (`[OverwolfPackages]`, `main.js:36531`): listar, ler e trocar
o canal de cada pacote (estável/beta); ao baixar a troca, avisa a janela
principal com `overwolf-packages:channel-ready`.

## 3. Estado: as stores

Todo estado vive em stores Zustand no processo principal, espelhadas para cada
janela por IPC (`store:<nome>:get|set|subscribe`, ver [03](03-ipc-e-ponte.md)).

| Store | Persistida | Conteúdo | Onde |
|---|---|---|---|
| `settings` | sim (`settings-store`, versão 2) | todas as preferências do usuário (§8) | `main.js:14936` |
| `config` | sim (`config-store`) | caches de dados (§7), dados do usuário, contadores, flags de primeira vez | `main.js:13141` |
| `window-state` | sim | posição/tamanho/maximizado de cada janela | — |
| `launcher` | não | nome, tag, região, idioma, fila, token/porta do LCU, fase do cliente e do app, posição e campeão do quick play, última partida analisada, `elevatedPrivilegesRequired` | `main.js:14142` |
| `champ-select` | não | leitura corrente do champ select e da análise de draft (§5) | `main.js:24899` |
| `q7z2k` | não | flags de assinatura `f1..f7` (ativa, em teste, elegível ao teste, início, fim, renova, fim do teste), escritas pela janela `background` a partir do Firestore | `main.js:14214` |
| `game-clock` | não | relógio, faixa de tempo (`7/14/21/28/99`), fase de escala (`early/mid/late`), estado | `main.js:14049` |
| `damage` | não | dano físico/mágico/verdadeiro causado e recebido (acumulado) | idem |
| `ult-cooldowns` | não | `teamUltCd0..3`: recarga da suprema dos **quatro aliados** | idem |
| `live-players` | não | os dez jogadores da Live Client | idem |
| `gold` | não | ouro atual e total do jogador | idem |
| `player-stats` | não | abates, mortes, assistências, tropas, monstros, nível, runas, pontos de habilidade gastos | idem |
| `objectives` | não | timers de buff de Barão/Ancião (e quem tem), respawn de inibidor e torre do nexo, relíquias de vida (ARAM) | idem |
| `overlay-status` | não | `tabbed` (atalho de overlay segurado), qual tela in-game está ativa | idem |
| `aram-augments` | não | aprimoramentos oferecidos agora | idem |
| `scouting` | não | resultado do scouting e `draftVerified` | idem |

## 4. A máquina de fases

`[Phase]` traduz a fase do cliente (`game_flow.phase`) + fila + estado do champ
select em uma **fase do app** (`main.js:37722`) e reage à transição
(`main.js:37947`).

| Fase do cliente | Fase do app |
|---|---|
| `None` | `None` |
| `Lobby`, `Matchmaking`, `ReadyCheck` | `Lobby` |
| `ChampSelect`, fila 450 | `ChampSelectAram` |
| `ChampSelect`, filas 2400/3270 | `ChampSelectAramMayhem` |
| `ChampSelect`, fila 3120 + assinante | `ChampSelectCustomRandom` |
| `ChampSelect`, ranqueada/normal (400/420/440/700) | `EveryoneLocked` › `ChampLocked` › `Picking` › `ChampSelectNormal/RankedSolo/RankedFlex/Ranked5s` |
| `ChampSelect`, personalizada (3100/3110/3140/torneio) + assinante | idem, com `ChampSelectCustomBlind/CustomDraft/PracticeTool/Tournament` |
| `ChampSelect`, o resto | `ChampSelectUnsupported` |
| `GameStart`, `InProgress`, `WaitingForStats`, `Reconnect` | `InGameAram`, `InGame` ou `InGameUnsupported` |
| `PreEndOfGame`, `EndOfGame` | `None` |

| Transição para | Efeito |
|---|---|
| `None` | fecha draft, in-game e overlays |
| `Lobby` | idem + zera as stores de partida |
| `ChampSelectAram*` | abre `drafting-aram` (se `autoLaunchDraft`) |
| champ select de SR | abre `drafting` e busca dados de draft (§5) |
| `ChampSelectUnsupported` em personalizada sem assinatura | abre `drafting`, que mostra "Custom Game support is only available for iTero+ subscribers" |
| `InGame`/`InGameAram` | fecha draft; abre in-game (overlay num monitor só, janela solta havendo segundo monitor) se `autoLaunchInGame`; esconde a principal se `autoCloseDesktopInGame` |
| saída de champ select sem jogo | "dodged": zera stores |
| fim de `InGame` | zera stores, reabre a principal e roda a **análise pós-partida** (§6) |

## 5. Champ select e draft

### Leitura do champ select

`[ChampSelect]` (`main.js:38121`) recebe a sessão crua do LCU pelo GEP
(`champ_select.raw`) e grava no `champ-select`: fase do timer e tempo restante,
o seu jogador e os dois times, ações de ban e pick, a **sua** ação de ban e de
pick, sua posição (no Practice Tool, a escolhida na UI), lado (`Blue`/`Red`),
se é a sua vez, o hover atual e o anterior, bans concluídos, intenções de pick e
ban dos aliados, e o banco do ARAM. Ignora espectador e filas não suportadas.

- **Hover manual desliga o automático.** Se o seu hover no cliente difere do
  último hover que o app fez, grava `disableAppHover` e para de mexer
  (`appHoverDisabled`: "Client hovered detected. App hovering and locking is
  disabled.").
- **Override de posição dos aliados** é inicializado com a posição atribuída de
  cada `gameName#tag`; a UI deixa corrigir (ver o de inimigos abaixo).

`[ChampSelectDraft]` (`main.js:38058`): detecta hover manual, dispara
auto-hover de ban na fase de ban, agenda o **auto-lock** para o fim do
cronômetro, nota novos picks travados (refaz a recomendação para assinantes),
marca `isEveryoneLocked` na finalização, nota troca de rota com aliado e troca de
hover depois de travar.

`[ChampSelectAram]` (`main.js:37713`): zera a build ativa quando o hover muda.

### Composição enviada ao backend

O backend recebe a composição como chaves por posição (`main.js:25039`,
`25056`):

- aliado com posição atribuída única → `t1_<POS>_champion`;
- aliado sem posição, ou com posição repetida → `t1_champion_<n>`;
- inimigo com override manual → `t2_<POS>_champion`, senão `t2_champion_<n>`;
- slot vazio → `"avg"`.

A ranqueada não informa a posição do inimigo; o iTero **não infere**: manda
`t2_champion_<n>` e deixa o modelo lidar, ou usa o override que o usuário marca
em "Draft Override: Assign a Champion's Position".

Os demais campos (`main.js:25007`): região, rota, lado, **classe de elo** (a
sua, ou a escolhida em "Use my Elo"), maestria (a sua ou a soma de todas as
contas cadastradas, se "Aggregate Mastery" e assinante), bans e a classe de elo
do rastreador de CS.

### As duas chamadas

`[Draft]` (`main.js:26022`), ambas com debounce de 500 ms e cancelamento da
anterior:

| Chamada | Corpo | Quando |
|---|---|---|
| `POST {região}/lake-draft-v2-{rota}` `action: "recommend"` | maestria, lado, bans, classe de elo, composição, pesos do modelo | antes de travar: lista de picks |
| idem `action: "analyse"` | + classe de elo do CS, `ignore_builds`, e um log extra quando há `"avg"` na composição | depois de travar: análise do seu campeão (+ builds quando todos travaram) |

`[DraftData]` (`main.js:26091`) escolhe qual chamar pela fase: picks em
`ChampSelect*`/`Picking`; em `ChampLocked`, picks no torneio e análise sem build
no resto; análise com build em `EveryoneLocked`. Depois de uma lista de picks,
**2 s** depois tenta o auto-hover.

**Pesos do modelo** (`main.js:26073`): o usuário ajusta, por rota, o peso de
cada termo, e eles vão como `user_model_weightings`:
`t1_<ROTA>_mastery` (padrão 0,3), `t1_early/mid/late_wr_adj_diff`,
`t1_ad_ratio`, `t1_champ_total_damage`, `t1_champ_dmg_taken`,
`t1_<ROTA>_champ_wr`, `t1_syn_LANE_<POS>_wr` (sinergia com cada aliado) e
`t1_counter_LANE_<POS>_wr` (contra cada inimigo). Padrão 1 (100%).

### Automação do champ select

| Módulo | O que faz | Onde |
|---|---|---|
| `[AutoHoverPick]` hover | escolhe o melhor da lista — primeiro a sua pool da rota, depois quem passa no filtro de pick rate e de nota mínima de rota, só campeões que você tem (ou todos, com "Display all Champions") — e faz `PATCH` na sua ação de pick | `main.js:25075` |
| `[AutoHoverPick]` lock | no fim do cronômetro trava o hover; sem hover, trava o melhor da lista | idem |
| `[AutoHoverBan]` | na fase de ban faz hover do primeiro da sua ban pool (até 5, em ordem), ou da recomendação de ban **da rota** (contra o seu hover ou o seu lado) ou **geral**; nunca algo que um aliado quer pegar ou banir | `main.js:38019` |
| bench swap, seleção de feitiços | `api:bench-swap`, `api:edit-session-selection` | `main.js:15192`, `15201` |

Todas desligadas por padrão.

### Verificação do draft no jogo

`[VerifyDraft]` (`main.js:36986`): quando a partida começa, a Live Client
entrega a **posição real** de cada jogador. O iTero remonta a composição com elas
e, se difere da que usou no champ select (por exemplo, inimigo inferido na rota
errada), refaz a análise. Ignora composição incompleta (fora das personalizadas)
e campeão duplicado.

`[TeamScouting]` (`main.js:36881`): com a composição e os Riot IDs da Live
Client, pede `POST {região}/lake-team-scouting` e grava os dez cartões de
jogador (§[05](05-funcionalidades.md#2-scouting-de-jogadores)).

## 6. Pós-partida

`postGameMatchAnalysis()` (`main.js:37805-37860`):

1. atualiza o elo (`lake-basic-info`, `update_type: "RANK"`);
2. avisa a principal (`post-match-analysis:start`);
3. `POST {região}/lake-match-analysis` com puuid, região, região maior,
   maestria, elo e `itero_plus` — até 3 tentativas;
4. espera 4 s e consulta o histórico (`lake-request-data`,
   `stat_name: "match_history"`, `limit: 1`, sem cache) até 2 vezes, 2 s entre
   elas;
5. se o id da última partida mudou, grava `latestAnalyzedMatch`, pisca a janela
   principal e manda `post-match-analysis:end` (a UI abre a partida); senão
   `post-match-analysis:cancel`; erro → `post-match-analysis:error`.

A análise em si é do servidor; o cliente só dispara e exibe.

## 7. Dados de configuração

`[ConfigData]` (`main.js:36262`), a cada 12 h e ao reconectar
(`[NetworkStatus]`, `main.js:36508`, debounce de 2 s; se a busca de draft tinha
falhado, ela é refeita):

| Dado | Fonte |
|---|---|
| versão do app, contagem de aberturas | local |
| status dos eventos Overwolf do jogo e do launcher | API de status do Overwolf |
| resumo de campeões, itens, runas, árvores, feitiços | CommunityDragon, com o CloudFront do iTero de reserva; **só refaz se a versão do Data Dragon mudou** |
| winrate de ARAM | CloudFront do iTero (`champ_wrs.json`) |
| filtro de itens de partida, médias de stat por elo, rotas padrão por campeão, tabela de stats por campeão | CloudFront do iTero |
| recomendações de ban e confrontos-chave | backend, pelo seu tier |
| pacotes e preço da assinatura | Tebex headless; preço regional em `lake-regional-pricing` (padrão USD 7,99) |
| aprimoramentos do Arena/ARAM: nome em inglês por id; nome localizado → ids, para o idioma do cliente | CommunityDragon |
| autostart com o Windows | ligado na primeira execução; depois respeita o estado real do Windows |

`[UserData]` (`main.js:37593`), 3 s depois de saber nome/tag/região:
`lake-basic-info` (`update_type: "ALL"`, 3 tentativas com espera exponencial) →
informações, maestria e elo; em paralelo recomendações de ban e confrontos-chave
pelo tier; histórico (última partida); contas Riot adicionais; e o leaderboard,
se houver amigos.

`[UserLCUData]` (`main.js:26198`), 5 s depois de saber token/porta do LCU:
campeões que você tem (`all-grid-champions`: próprios, Game Pass, grátis) e as
configurações de jogo (modo de janela, escala global e do minimapa, minimapa
invertido, quadros do time à esquerda), que os overlays usam para se posicionar.

## 8. Configurações do usuário (`settings`)

Padrões em `main.js:14936`:

| Grupo | Chaves (padrão) |
|---|---|
| Janela | `locale` (en), `autoLaunchDesktop` (sim), `autoLaunchDraft` (sim), `autoLaunchInGame` (sim), `autoCloseDesktopInGame` (não), `draftScreenSize` (lg), `hardwareAccelerationEnabled` (sim) |
| Atalhos | `ingameToggleHotkey` (Alt+I), `ingameTabToggleHotkey` (Alt+O), `overlayToggleHotkey` (Tab) + `overlayToggleType` (segurar), `flashKey` (F), `summonerSpellsConfig` |
| Build | `pushBuildItems`, `pushBuildRunes`, `pushBuildSummonerSpells` (sim); `autoPushBuild` (não) com `autoPushBuildItems`, `autoPushBuildRunes`, `autoPushBuildSummonerSpells` (sim) |
| Automação | `autoHoverChampion`, `autoLockChampion`, `autoHoverChampionBan` (não), `championBanType` (rota), `championPoolEnabled`/`championPool`, `championBanPoolEnabled`/`championBanPool` |
| Recomendação | `displayAllChampions` (não), `showLaneScore` (não), `sortByLaneScore` (não), `minimumLaneScore` (0), `considerTeamHover` (sim), `pickRateSliderValue` (3) / `pickRateFilterValue` (0,25%), `draftingModelCustomization` (§5), `draftingAdvancedMode` (não), `aggregateMastery` (não), `riotAccounts` |
| Overlays | `mapTimers` (não, texto 12), `aramMapTimers` (não; relíquias e inibidores), `skillLevelUpEnabled` (não), `aramMayhemAugmentsEnabled` (sim), `teamGoldTrackerEnabled` (não), `damageTracker` (não; causado e recebido), `monsterBuffTimersEnabled` (não), `ultCooldownTimersEnabled` (não), `statTracker` (não; elo seu ou fixo, rotas) |
| In-game | `ingameScreenDefaultTab` (scouting), `teamScoutingAdvancedMode` (não) |
| Pós-jogo | `matchBreakdownStats`: as seis estatísticas do gráfico por rota — topo/selva/meio/atirador: KDA, **CS/min**, objetivos neutros, dano recebido, dano causado, CS/min aos 7; **suporte: KDA, placar de visão/min, objetivos neutros, dano recebido, dano causado, sentinelas de controle** (`main.js:14905`) |
| Privacidade | `useHashedEmails` (sim) |
| ARAM | `aramRandomBuild`: faixa de raridade da build aleatória (1 Exótica … 6 Meta) |

Mudanças em `settings` vão para o Firestore com debounce de 5 s (`[Store]` →
janela `background`).

## 9. Eventos ao vivo

`[GameEvent]` (`main.js:37242`) recebe cada atualização do GEP do jogo como
`caminho → valor` e distribui, agrupando rajadas em 500 ms:

| Caminho | Efeito |
|---|---|
| `kill.kills`, `death.deaths`, `assist.assists`, `level.level`, `game_info.minionKills`, `game_info.neutralMinionKills` | `player-stats` |
| `gold.gold` | `gold` + **estado da rota** (abaixo) |
| `game_info.team_frames_0`, `game_info.team_frames_1`, `game_info.team_frames_2`, `game_info.team_frames_3` | `ult-cooldowns` (suprema dos aliados) |
| `events.match_clock` | relógio, faixa de tempo, fase de escala; expira buffs |
| `events.physical_damage_dealt_to_champions`, `events.magic_damage_dealt_to_champions`, `events.true_damage_dealt_to_champions`, `events.physical_damage_taken`, `events.magic_damage_taken`, `events.true_damage_taken` | `damage` |
| `live_client_data.events` | `[LiveClientEvents]` |
| `live_client_data.all_players` | `live-players`; expira buff de quem morreu; 1ª vez: scouting e verificação do draft |
| `live_client_data.active_player` | pontos de habilidade gastos, runas |
| `jungle_camps.jungle_camp_0`, `jungle_camps.jungle_camp_1`, `jungle_camps.jungle_camp_2`, `jungle_camps.jungle_camp_3` | relíquias de vida do ARAM (90 s): bot, topo-meio, bot-meio, topo |
| `me.augments` / `me.picked_augment` | abre/esconde o overlay de aprimoramentos do ARAM Mayhem |

**Faixas de tempo** (`main.js:37223`): `7` até 7 min, `14` até 14, `21` até 21,
`28` até 28, `99` depois. **Fase de escala**: `early` até 27 min, `mid` até 33,
`late` depois.

**Estado da rota** (`main.js:36655`): o seu ouro total contra dois limiares
**por rota e faixa de tempo** — abaixo do primeiro é `behind`, acima do segundo
é `ahead`, entre eles `even`; na faixa `7` é sempre `even`. Os limiares do
suporte são ~20% menores que os das outras rotas (aos 14 min: suporte
1.428/1.659, topo 1.771/2.018, selva 1.925/2.197). A tela in-game troca a dica e
a build pelo estado.

**`[LiveClientEvents]`** (`main.js:37147`), olhando o último evento:

| Evento | Efeito |
|---|---|
| primeiro evento da partida | `[GAME STARTED]`: abre os overlays habilitados |
| `DragonKill` com `Elder` | buff de Ancião: relógio + **150 s**, campeões vivos do time que matou |
| `BaronKill` | buff de Barão: relógio + **180 s**, campeões vivos do time |
| `InhibKilled` | respawn do inibidor (qual time, qual rota): relógio + **300 s** |
| `TurretKilled` de torre do nexo | respawn da torre do nexo: relógio + **180 s** |
| `GameEnd` | só log |

Quem morre perde o buff (`live-players`), e o timer some quando ninguém mais o
tem.

## 10. Atalhos e janelas

`[Hotkey]` (`main.js:14783`, `14819`) registra três atalhos no pacote overlay,
todos com *passthrough* (a tecla também chega ao jogo):

| Atalho | Efeito |
|---|---|
| `ingameToggleHotkey` | alterna a tela in-game (ARAM ou SR; overlay ou janela solta conforme haja segundo monitor) |
| `ingameTabToggleHotkey` | alterna entre as duas telas in-game |
| `overlayToggleHotkey` (Tab) | segurar ou alternar: mostra os overlays e deixa interagir com eles |

Gerência de janelas (`main.js:14454-14737`): duas classes, uma para janela
comum do Electron e outra para janela de **overlay injetada** pelo Overwolf
(detalhe em [02](02-janelas.md)). A comum guarda posição e tamanho com debounce
de 500 ms, recentraliza se abriu fora de qualquer monitor (as `-second` no
segundo monitor), avisa maximizar/restaurar, e pode piscar na barra de tarefas
até ganhar foco. `forceOpenOnTop` põe a janela por cima de tudo por 300 ms.
`[Display]` acompanha monitores (adicionado, removido, métrica) e o monitor do
jogo.
