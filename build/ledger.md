# Ledger

Gerado por `node tools/doccheck/doccheck.mjs`. Não edite à mão: a próxima execução reescreve.

| Tipo | Total | Feito | Adiado | Pendente |
|---|---|---|---|---|
| type | 112 | 112 | 0 | 0 |
| func | 167 | 167 | 0 | 0 |
| const | 84 | 84 | 0 | 0 |
| wire | 55 | 55 | 0 | 0 |
| command | 26 | 26 | 0 | 0 |
| event | 6 | 6 | 0 | 0 |
| file | 16 | 16 | 0 | 0 |
| flag | 16 | 16 | 0 | 0 |
| test | 374 | 374 | 0 | 0 |
| ui-test | 48 | 15 | 33 | 0 |
| trace | 43 | 39 | 4 | 0 |
| itero | 83 | 83 | 0 | 0 |
| todo | 2 | 0 | 2 | 0 |

- testes no código: 503; nomes no catálogo (13): 503; fora dele: 0
- links relativos conferidos: 871
- IDs citados conferidos: 3077

## type

| Item | Fonte | Estado | Onde / motivo |
|---|---|---|---|
| `casca::EngineStatus` | arquitetura/10-modelo-de-dados §32.1 | done | src-tauri/src/lib.rs:48 |
| `casca::live::LiveSnapshot` | arquitetura/10-modelo-de-dados §32.3 | done | src-tauri/src/live.rs:17 |
| `casca::PoolPlan` | arquitetura/10-modelo-de-dados §32.2 | done | src-tauri/src/lib.rs:371 |
| `casca::watcher::Identity` | arquitetura/10-modelo-de-dados §32.4 | done | src-tauri/src/watcher.rs:52 |
| `casca::watcher::Shared` | arquitetura/10-modelo-de-dados §32.5 | done | src-tauri/src/watcher.rs:60 |
| `core::draft::ConnStatus` | arquitetura/10-modelo-de-dados §3.8 | done | core/src/draft.rs:116 |
| `core::draft::DraftSlot` | arquitetura/10-modelo-de-dados §3.6 | done | core/src/draft.rs:78 |
| `core::draft::DraftState` | arquitetura/10-modelo-de-dados §3.7 | done | core/src/draft.rs:95 |
| `core::draft::LcuAction` | arquitetura/10-modelo-de-dados §3.2 | done | core/src/draft.rs:27 |
| `core::draft::LcuGameData` | arquitetura/10-modelo-de-dados §3.11 | done | core/src/draft.rs:261 |
| `core::draft::LcuGameflowSession` | arquitetura/10-modelo-de-dados §3.10 | done | core/src/draft.rs:255 |
| `core::draft::LcuGridChampion` | arquitetura/10-modelo-de-dados §3.9 | done | core/src/draft.rs:245 |
| `core::draft::LcuPlayer` | arquitetura/10-modelo-de-dados §3.3 | done | core/src/draft.rs:41 |
| `core::draft::LcuQueue` | arquitetura/10-modelo-de-dados §3.12 | done | core/src/draft.rs:267 |
| `core::draft::LcuSession` | arquitetura/10-modelo-de-dados §3.1 | done | core/src/draft.rs:15 |
| `core::draft::LcuSummoner` | arquitetura/10-modelo-de-dados §3.5 | done | core/src/draft.rs:63 |
| `core::draft::LcuTimer` | arquitetura/10-modelo-de-dados §3.4 | done | core/src/draft.rs:54 |
| `core::engine::build::BuildContext` | arquitetura/10-modelo-de-dados §26.3 | done | core/src/engine/build.rs:45 |
| `core::engine::build::Combatant` | arquitetura/10-modelo-de-dados §26.1 | done | core/src/engine/build.rs:26 |
| `core::engine::build::ItemSuggestion` | arquitetura/10-modelo-de-dados §26.4 | done | core/src/engine/build.rs:56 |
| `core::engine::build::Loadout` | arquitetura/10-modelo-de-dados §26.2 | done | core/src/engine/build.rs:33 |
| `core::engine::champion::BaseStats` | arquitetura/10-modelo-de-dados §23.2 | done | core/src/engine/champion.rs:57 |
| `core::engine::champion::Champion` | arquitetura/10-modelo-de-dados §23.3 | done | core/src/engine/champion.rs:85 |
| `core::engine::champion::ChampionIndex` | arquitetura/10-modelo-de-dados §23.4 | done | core/src/engine/champion.rs:135 |
| `core::engine::champion::Role` | arquitetura/10-modelo-de-dados §23.1 | done | core/src/engine/champion.rs:4 |
| `core::engine::coach::Note` | arquitetura/10-modelo-de-dados §31.3 | done | core/src/engine/coach.rs:111 |
| `core::engine::coach::Point` | arquitetura/10-modelo-de-dados §31.4 | done | core/src/engine/coach.rs:122 |
| `core::engine::coach::Recording` | arquitetura/10-modelo-de-dados §31.1 | done | core/src/engine/coach.rs:76 |
| `core::engine::coach::Report` | arquitetura/10-modelo-de-dados §31.6 | done | core/src/engine/coach.rs:159 |
| `core::engine::coach::Series` | arquitetura/10-modelo-de-dados §31.5 | done | core/src/engine/coach.rs:133 |
| `core::engine::coach::Verdict` | arquitetura/10-modelo-de-dados §31.2 | done | core/src/engine/coach.rs:104 |
| `core::engine::comp::TeamProfile` | arquitetura/10-modelo-de-dados §13.1 | done | core/src/engine/comp.rs:3 |
| `core::engine::context::DraftContext` | arquitetura/10-modelo-de-dados §10.4 | done | core/src/engine/context.rs:60 |
| `core::engine::context::Leitura` | arquitetura/10-modelo-de-dados §10.3 | done | core/src/engine/context.rs:49 |
| `core::engine::context::Placed` | arquitetura/10-modelo-de-dados §10.2 | done | core/src/engine/context.rs:38 |
| `core::engine::damage::Penetration` | arquitetura/10-modelo-de-dados §25.1 | done | core/src/engine/damage.rs:11 |
| `core::engine::dataset::Dataset` | arquitetura/10-modelo-de-dados §20.1 | done | core/src/engine/dataset.rs:27 |
| `core::engine::dataset::Layered` | arquitetura/10-modelo-de-dados §20.3 | done | core/src/engine/dataset.rs:136 |
| `core::engine::gold::GoldPlayer` | arquitetura/10-modelo-de-dados §28.1 | done | core/src/engine/gold.rs:41 |
| `core::engine::gold::GoldReport` | arquitetura/10-modelo-de-dados §28.2 | done | core/src/engine/gold.rs:61 |
| `core::engine::items::Item` | arquitetura/10-modelo-de-dados §24.2 | done | core/src/engine/items.rs:84 |
| `core::engine::items::ItemIndex` | arquitetura/10-modelo-de-dados §24.3 | done | core/src/engine/items.rs:94 |
| `core::engine::items::ItemStats` | arquitetura/10-modelo-de-dados §24.1 | done | core/src/engine/items.rs:20 |
| `core::engine::lanes::Focus` | arquitetura/10-modelo-de-dados §27.6 | done | core/src/engine/lanes.rs:466 |
| `core::engine::lanes::LaneOutlook` | arquitetura/10-modelo-de-dados §27.3 | done | core/src/engine/lanes.rs:155 |
| `core::engine::lanes::LevelOdds` | arquitetura/10-modelo-de-dados §27.2 | done | core/src/engine/lanes.rs:50 |
| `core::engine::lanes::Lineup` | arquitetura/10-modelo-de-dados §27.5 | done | core/src/engine/lanes.rs:224 |
| `core::engine::lanes::Skirmish` | arquitetura/10-modelo-de-dados §27.4 | done | core/src/engine/lanes.rs:187 |
| `core::engine::minimax::BanSuggestion` | arquitetura/10-modelo-de-dados §12.2 | done | core/src/engine/minimax.rs:50 |
| `core::engine::minimax::MinimaxPick` | arquitetura/10-modelo-de-dados §12.1 | done | core/src/engine/minimax.rs:36 |
| `core::engine::poolgap::Analysis` | arquitetura/10-modelo-de-dados §17.4 | done | core/src/engine/poolgap.rs:250 |
| `core::engine::poolgap::Candidate` | arquitetura/10-modelo-de-dados §17.5 | done | core/src/engine/poolgap.rs:257 |
| `core::engine::poolgap::Finding` | arquitetura/10-modelo-de-dados §17.3 | done | core/src/engine/poolgap.rs:241 |
| `core::engine::poolgap::Trait` | arquitetura/10-modelo-de-dados §17.2 | done | core/src/engine/poolgap.rs:41 |
| `core::engine::priority::LanePriority` | arquitetura/10-modelo-de-dados §30.1 | done | core/src/engine/priority.rs:32 |
| `core::engine::priority::PriorityReport` | arquitetura/10-modelo-de-dados §30.2 | done | core/src/engine/priority.rs:42 |
| `core::engine::profile::ChampionLane` | arquitetura/10-modelo-de-dados §21.3 | done | core/src/engine/profile.rs:69 |
| `core::engine::profile::LaneProfile` | arquitetura/10-modelo-de-dados §21.4 | done | core/src/engine/profile.rs:88 |
| `core::engine::profile::MatchupLane` | arquitetura/10-modelo-de-dados §21.2 | done | core/src/engine/profile.rs:61 |
| `core::engine::profile::Reading` | arquitetura/10-modelo-de-dados §21.1 | done | core/src/engine/profile.rs:37 |
| `core::engine::report::LaneReport` | arquitetura/10-modelo-de-dados §16.1 | done | core/src/engine/report.rs:17 |
| `core::engine::rival::Metric` | arquitetura/10-modelo-de-dados §22.2 | done | core/src/engine/rival.rs:29 |
| `core::engine::rival::Rival` | arquitetura/10-modelo-de-dados §22.4 | done | core/src/engine/rival.rs:46 |
| `core::engine::rival::Standing` | arquitetura/10-modelo-de-dados §22.3 | done | core/src/engine/rival.rs:38 |
| `core::engine::runes::Rune` | arquitetura/10-modelo-de-dados §18.1 | done | core/src/engine/runes.rs:29 |
| `core::engine::runes::RuneIndex` | arquitetura/10-modelo-de-dados §18.2 | done | core/src/engine/runes.rs:43 |
| `core::engine::runes::RunePlan` | arquitetura/10-modelo-de-dados §18.4 | done | core/src/engine/runes.rs:72 |
| `core::engine::runes::RuneSuggestion` | arquitetura/10-modelo-de-dados §18.3 | done | core/src/engine/runes.rs:59 |
| `core::engine::Scored` | arquitetura/10-modelo-de-dados §9.1 | done | core/src/engine/mod.rs:57 |
| `core::engine::stats::Estimate` | arquitetura/10-modelo-de-dados §19.2 | done | core/src/engine/stats.rs:37 |
| `core::engine::stats::Length` | arquitetura/10-modelo-de-dados §19.3 | done | core/src/engine/stats.rs:131 |
| `core::engine::stats::NoStats` | arquitetura/10-modelo-de-dados §19.5 | done | core/src/engine/stats.rs:197 |
| `core::engine::stats::PersonalStats` | arquitetura/10-modelo-de-dados §19.6 | done | core/src/engine/stats.rs:221 |
| `core::engine::stats::Record` | arquitetura/10-modelo-de-dados §19.1 | done | core/src/engine/stats.rs:7 |
| `core::engine::stats::StatsStore` | arquitetura/10-modelo-de-dados §19.4 | done | core/src/engine/stats.rs:153 |
| `core::engine::team::Seat` | arquitetura/10-modelo-de-dados §14.1 | done | core/src/engine/team.rs:28 |
| `core::engine::team::TeamEstimate` | arquitetura/10-modelo-de-dados §14.3 | done | core/src/engine/team.rs:47 |
| `core::engine::team::TeamTerm` | arquitetura/10-modelo-de-dados §14.2 | done | core/src/engine/team.rs:38 |
| `core::engine::timeline::Buff` | arquitetura/10-modelo-de-dados §29.8 | done | core/src/engine/timeline.rs:216 |
| `core::engine::timeline::BuffTracker` | arquitetura/10-modelo-de-dados §29.9 | done | core/src/engine/timeline.rs:236 |
| `core::engine::timeline::EventFeed` | arquitetura/10-modelo-de-dados §29.3 | done | core/src/engine/timeline.rs:105 |
| `core::engine::timeline::RawEvent` | arquitetura/10-modelo-de-dados §29.2 | done | core/src/engine/timeline.rs:67 |
| `core::engine::timeline::Respawn` | arquitetura/10-modelo-de-dados §29.7 | done | core/src/engine/timeline.rs:144 |
| `core::engine::timeline::Sides` | arquitetura/10-modelo-de-dados §29.4 | done | core/src/engine/timeline.rs:111 |
| `core::engine::timeline::Spawn` | arquitetura/10-modelo-de-dados §29.5 | done | core/src/engine/timeline.rs:117 |
| `core::engine::timeline::Timeline` | arquitetura/10-modelo-de-dados §29.6 | done | core/src/engine/timeline.rs:128 |
| `core::engine::timeline::Timings` | arquitetura/10-modelo-de-dados §29.1 | done | core/src/engine/timeline.rs:30 |
| `core::engine::wincon::Point` | arquitetura/10-modelo-de-dados §15.3 | done | core/src/engine/wincon.rs:55 |
| `core::engine::wincon::Side` | arquitetura/10-modelo-de-dados §15.1 | done | core/src/engine/wincon.rs:38 |
| `core::engine::wincon::Source` | arquitetura/10-modelo-de-dados §15.2 | done | core/src/engine/wincon.rs:46 |
| `core::engine::wincon::WinCondition` | arquitetura/10-modelo-de-dados §15.4 | done | core/src/engine/wincon.rs:63 |
| `core::history::Challenges` | arquitetura/10-modelo-de-dados §5.4 | done | core/src/history.rs:61 |
| `core::history::LaneGame` | arquitetura/10-modelo-de-dados §5.7 | done | core/src/history.rs:157 |
| `core::history::MatchDto` | arquitetura/10-modelo-de-dados §5.1 | done | core/src/history.rs:30 |
| `core::history::MatchInfo` | arquitetura/10-modelo-de-dados §5.2 | done | core/src/history.rs:36 |
| `core::history::MatchParticipant` | arquitetura/10-modelo-de-dados §5.3 | done | core/src/history.rs:46 |
| `core::history::Metric` | arquitetura/10-modelo-de-dados §5.5 | done | core/src/history.rs:90 |
| `core::history::PlayedGame` | arquitetura/10-modelo-de-dados §5.6 | done | core/src/history.rs:148 |
| `core::history::Skip` | arquitetura/10-modelo-de-dados §5.8 | done | core/src/history.rs:170 |
| `core::live::LiveGame` | arquitetura/10-modelo-de-dados §4.5 | done | core/src/live.rs:72 |
| `core::live::LivePlayer` | arquitetura/10-modelo-de-dados §4.4 | done | core/src/live.rs:54 |
| `core::live::RawItem` | arquitetura/10-modelo-de-dados §4.2 | done | core/src/live.rs:35 |
| `core::live::RawPlayer` | arquitetura/10-modelo-de-dados §4.1 | done | core/src/live.rs:16 |
| `core::live::RawScores` | arquitetura/10-modelo-de-dados §4.3 | done | core/src/live.rs:44 |
| `core::lockfile::Credentials` | arquitetura/10-modelo-de-dados §2.1 | done | core/src/lockfile.rs:4 |
| `core::pool::Pool` | arquitetura/10-modelo-de-dados §7.4 | done | core/src/pool.rs:44 |
| `core::pool::PoolEntry` | arquitetura/10-modelo-de-dados §7.2 | done | core/src/pool.rs:32 |
| `core::pool::Tier` | arquitetura/10-modelo-de-dados §7.1 | done | core/src/pool.rs:13 |
| `core::prefs::DraftPrefs` | arquitetura/10-modelo-de-dados §8.3 | done | core/src/prefs.rs:130 |
| `core::prefs::OverlayPrefs` | arquitetura/10-modelo-de-dados §8.2 | done | core/src/prefs.rs:64 |
| `core::prefs::Section` | arquitetura/10-modelo-de-dados §8.1 | done | core/src/prefs.rs:13 |
| `core::riot::Region` | arquitetura/10-modelo-de-dados §6.1 | done | core/src/riot.rs:3 |

## func

| Item | Fonte | Estado | Onde / motivo |
|---|---|---|---|
| `core::draft::DraftState::normalized` | arquitetura/11-catalogo-de-funcoes §3.2 | done | core/src/draft.rs:234 |
| `core::draft::selectable` | arquitetura/11-catalogo-de-funcoes §3.3 | done | core/src/draft.rs:280 |
| `core::draft::to_draft_state` | arquitetura/11-catalogo-de-funcoes §3.1 | done | core/src/draft.rs:129 |
| `core::engine::build::effective_hp_against` | arquitetura/11-catalogo-de-funcoes §26.4 | done | core/src/engine/build.rs:94 |
| `core::engine::build::incoming_physical_share` | arquitetura/11-catalogo-de-funcoes §26.3 | done | core/src/engine/build.rs:152 |
| `core::engine::build::loadout` | arquitetura/11-catalogo-de-funcoes §26.2 | done | core/src/engine/build.rs:69 |
| `core::engine::build::offense_against` | arquitetura/11-catalogo-de-funcoes §26.4 | done | core/src/engine/build.rs:112 |
| `core::engine::build::side_power` | arquitetura/11-catalogo-de-funcoes §26.4 | done | core/src/engine/build.rs:186 |
| `core::engine::build::skirmish_odds` | arquitetura/11-catalogo-de-funcoes §26.4 | done | core/src/engine/build.rs:213 |
| `core::engine::build::suggest` | arquitetura/11-catalogo-de-funcoes §26.1 | done | core/src/engine/build.rs:247 |
| `core::engine::build::team_penetration` | arquitetura/11-catalogo-de-funcoes §26.4 | done | core/src/engine/build.rs:228 |
| `core::engine::champion::BaseStats::armor_at` | arquitetura/11-catalogo-de-funcoes §23.3 | done | core/src/engine/champion.rs:74 |
| `core::engine::champion::BaseStats::attack_damage_at` | arquitetura/11-catalogo-de-funcoes §23.3 | done | core/src/engine/champion.rs:80 |
| `core::engine::champion::BaseStats::health_at` | arquitetura/11-catalogo-de-funcoes §23.3 | done | core/src/engine/champion.rs:71 |
| `core::engine::champion::BaseStats::magic_resist_at` | arquitetura/11-catalogo-de-funcoes §23.3 | done | core/src/engine/champion.rs:77 |
| `core::engine::champion::Champion::is_frontline` | arquitetura/11-catalogo-de-funcoes §23.4 | done | core/src/engine/champion.rs:130 |
| `core::engine::champion::Champion::is_ranged` | arquitetura/11-catalogo-de-funcoes §23.4 | done | core/src/engine/champion.rs:126 |
| `core::engine::champion::Champion::physical_share` | arquitetura/11-catalogo-de-funcoes §23.4 | done | core/src/engine/champion.rs:113 |
| `core::engine::champion::Champion::primary_tag` | arquitetura/11-catalogo-de-funcoes §23.4 | done | core/src/engine/champion.rs:100 |
| `core::engine::champion::ChampionIndex::by_key` | arquitetura/11-catalogo-de-funcoes §23.5 | done | core/src/engine/champion.rs:148 |
| `core::engine::champion::ChampionIndex::get` | arquitetura/11-catalogo-de-funcoes §23.5 | done | core/src/engine/champion.rs:142 |
| `core::engine::champion::ChampionIndex::ids` | arquitetura/11-catalogo-de-funcoes §23.5 | done | core/src/engine/champion.rs:158 |
| `core::engine::champion::ChampionIndex::is_empty` | arquitetura/11-catalogo-de-funcoes §23.5 | done | core/src/engine/champion.rs:154 |
| `core::engine::champion::grown` | arquitetura/11-catalogo-de-funcoes §23.2 | done | core/src/engine/champion.rs:52 |
| `core::engine::champion::Role::from_lcu` | arquitetura/11-catalogo-de-funcoes §23.1 | done | core/src/engine/champion.rs:20 |
| `core::engine::champion::Role::short` | arquitetura/11-catalogo-de-funcoes §23.1 | done | core/src/engine/champion.rs:32 |
| `core::engine::coach::Recording::duration` | arquitetura/11-catalogo-de-funcoes §31.2 | done | core/src/engine/coach.rs:96 |
| `core::engine::coach::Recording::push` | arquitetura/11-catalogo-de-funcoes §31.2 | done | core/src/engine/coach.rs:85 |
| `core::engine::coach::review` | arquitetura/11-catalogo-de-funcoes §31.1 | done | core/src/engine/coach.rs:229 |
| `core::engine::coach::Series::is_empty` | arquitetura/11-catalogo-de-funcoes §31.3 | done | core/src/engine/coach.rs:151 |
| `core::engine::coach::worth_reviewing` | arquitetura/11-catalogo-de-funcoes §31.4 | done | core/src/engine/coach.rs:220 |
| `core::engine::comp::damage_balance_gain` | arquitetura/11-catalogo-de-funcoes §13.2 | done | core/src/engine/comp.rs:42 |
| `core::engine::comp::frontline_need` | arquitetura/11-catalogo-de-funcoes §13.3 | done | core/src/engine/comp.rs:53 |
| `core::engine::comp::profile` | arquitetura/11-catalogo-de-funcoes §13.1 | done | core/src/engine/comp.rs:19 |
| `core::engine::comp::share_after` | arquitetura/11-catalogo-de-funcoes §13.4 | done | core/src/engine/comp.rs:60 |
| `core::engine::context::DraftContext::ally_ids` | arquitetura/11-catalogo-de-funcoes §10.2 | done | core/src/engine/context.rs:309 |
| `core::engine::context::DraftContext::enemy_ids` | arquitetura/11-catalogo-de-funcoes §10.2 | done | core/src/engine/context.rs:313 |
| `core::engine::context::DraftContext::lane_opponent` | arquitetura/11-catalogo-de-funcoes §10.2 | done | core/src/engine/context.rs:293 |
| `core::engine::context::DraftContext::lineups` | arquitetura/11-catalogo-de-funcoes §10.2 | done | core/src/engine/context.rs:319 |
| `core::engine::context::DraftContext::new` | arquitetura/11-catalogo-de-funcoes §10.2 | done | core/src/engine/context.rs:177 |
| `core::engine::context::DraftContext::new_with` | arquitetura/11-catalogo-de-funcoes §10.2 | done | core/src/engine/context.rs:187 |
| `core::engine::context::DraftContext::open_enemy_roles` | arquitetura/11-catalogo-de-funcoes §10.2 | done | core/src/engine/context.rs:301 |
| `core::engine::context::DraftContext::with_candidate` | arquitetura/11-catalogo-de-funcoes §10.2 | done | core/src/engine/context.rs:285 |
| `core::engine::context::resolve_role` | arquitetura/11-catalogo-de-funcoes §10.1 | done | core/src/engine/context.rs:84 |
| `core::engine::damage::damage_after_resist` | arquitetura/11-catalogo-de-funcoes §25.3 | done | core/src/engine/damage.rs:59 |
| `core::engine::damage::damage_multiplier` | arquitetura/11-catalogo-de-funcoes §25.2 | done | core/src/engine/damage.rs:45 |
| `core::engine::damage::effective_hp` | arquitetura/11-catalogo-de-funcoes §25.3 | done | core/src/engine/damage.rs:54 |
| `core::engine::damage::effective_resist` | arquitetura/11-catalogo-de-funcoes §25.1 | done | core/src/engine/damage.rs:24 |
| `core::engine::dataset::base_key` | arquitetura/11-catalogo-de-funcoes §20.1 | done | core/src/engine/dataset.rs:76 |
| `core::engine::dataset::Dataset::is_usable` | arquitetura/11-catalogo-de-funcoes §20.2 | done | core/src/engine/dataset.rs:100 |
| `core::engine::dataset::Layered::matches` | arquitetura/11-catalogo-de-funcoes §20.3 | done | core/src/engine/dataset.rs:152 |
| `core::engine::dataset::Layered::patch` | arquitetura/11-catalogo-de-funcoes §20.3 | done | core/src/engine/dataset.rs:148 |
| `core::engine::dataset::Layered::with_global` | arquitetura/11-catalogo-de-funcoes §20.3 | done | core/src/engine/dataset.rs:143 |
| `core::engine::dataset::length_key` | arquitetura/11-catalogo-de-funcoes §20.1 | done | core/src/engine/dataset.rs:95 |
| `core::engine::dataset::matchup_key` | arquitetura/11-catalogo-de-funcoes §20.1 | done | core/src/engine/dataset.rs:80 |
| `core::engine::dataset::synergy_key` | arquitetura/11-catalogo-de-funcoes §20.1 | done | core/src/engine/dataset.rs:85 |
| `core::engine::dataset::versus_key` | arquitetura/11-catalogo-de-funcoes §20.1 | done | core/src/engine/dataset.rs:91 |
| `core::engine::gold::earned_at` | arquitetura/11-catalogo-de-funcoes §28.1 | done | core/src/engine/gold.rs:77 |
| `core::engine::gold::report` | arquitetura/11-catalogo-de-funcoes §28.2 | done | core/src/engine/gold.rs:90 |
| `core::engine::items::ItemIndex::get` | arquitetura/11-catalogo-de-funcoes §24.3 | done | core/src/engine/items.rs:101 |
| `core::engine::items::ItemIndex::gold_invested` | arquitetura/11-catalogo-de-funcoes §24.3 | done | core/src/engine/items.rs:117 |
| `core::engine::items::ItemIndex::is_empty` | arquitetura/11-catalogo-de-funcoes §24.3 | done | core/src/engine/items.rs:105 |
| `core::engine::items::ItemIndex::totals` | arquitetura/11-catalogo-de-funcoes §24.3 | done | core/src/engine/items.rs:110 |
| `core::engine::items::ItemStats::magic_penetration` | arquitetura/11-catalogo-de-funcoes §24.2 | done | core/src/engine/items.rs:68 |
| `core::engine::items::ItemStats::physical_penetration` | arquitetura/11-catalogo-de-funcoes §24.2 | done | core/src/engine/items.rs:59 |
| `core::engine::items::ItemStats::plus` | arquitetura/11-catalogo-de-funcoes §24.2 | done | core/src/engine/items.rs:42 |
| `core::engine::items::parse_stats` | arquitetura/11-catalogo-de-funcoes §24.1 | done | core/src/engine/items.rs:127 |
| `core::engine::lanes::focus` | arquitetura/11-catalogo-de-funcoes §27.3 | done | core/src/engine/lanes.rs:505 |
| `core::engine::lanes::lane_outlooks` | arquitetura/11-catalogo-de-funcoes §27.1 | done | core/src/engine/lanes.rs:258 |
| `core::engine::lanes::Lineup::new` | arquitetura/11-catalogo-de-funcoes §27.4 | done | core/src/engine/lanes.rs:231 |
| `core::engine::lanes::skirmishes` | arquitetura/11-catalogo-de-funcoes §27.2 | done | core/src/engine/lanes.rs:422 |
| `core::engine::minimax::bans_with_pool` | arquitetura/11-catalogo-de-funcoes §12.5 | done | core/src/engine/minimax.rs:249 |
| `core::engine::minimax::minimax` | arquitetura/11-catalogo-de-funcoes §12.2 | done | core/src/engine/minimax.rs:110 |
| `core::engine::minimax::minimax_with` | arquitetura/11-catalogo-de-funcoes §12.3 | done | core/src/engine/minimax.rs:118 |
| `core::engine::minimax::suggest_bans` | arquitetura/11-catalogo-de-funcoes §12.4 | done | core/src/engine/minimax.rs:197 |
| `core::engine::minimax::team_value` | arquitetura/11-catalogo-de-funcoes §12.1 | done | core/src/engine/minimax.rs:69 |
| `core::engine::poolgap::analyze` | arquitetura/11-catalogo-de-funcoes §17.3 | done | core/src/engine/poolgap.rs:275 |
| `core::engine::poolgap::robustness` | arquitetura/11-catalogo-de-funcoes §17.2 | done | core/src/engine/poolgap.rs:224 |
| `core::engine::poolgap::role_label` | arquitetura/11-catalogo-de-funcoes §17.5 | done | core/src/engine/poolgap.rs:403 |
| `core::engine::poolgap::suggest` | arquitetura/11-catalogo-de-funcoes §17.4 | done | core/src/engine/poolgap.rs:343 |
| `core::engine::poolgap::Trait::has` | arquitetura/11-catalogo-de-funcoes §17.1 | done | core/src/engine/poolgap.rs:93 |
| `core::engine::priority::report` | arquitetura/11-catalogo-de-funcoes §30.1 | done | core/src/engine/priority.rs:74 |
| `core::engine::profile::build` | arquitetura/11-catalogo-de-funcoes §21.1 | done | core/src/engine/profile.rs:103 |
| `core::engine::profile::ChampionLane::clearest` | arquitetura/11-catalogo-de-funcoes §21.2 | done | core/src/engine/profile.rs:80 |
| `core::engine::profile::LaneProfile::champion` | arquitetura/11-catalogo-de-funcoes §21.2 | done | core/src/engine/profile.rs:98 |
| `core::engine::profile::Reading::is_clear` | arquitetura/11-catalogo-de-funcoes §21.2 | done | core/src/engine/profile.rs:52 |
| `core::engine::rank` | arquitetura/11-catalogo-de-funcoes §9.1 | done | core/src/engine/mod.rs:74 |
| `core::engine::rank_with` | arquitetura/11-catalogo-de-funcoes §9.2 | done | core/src/engine/mod.rs:86 |
| `core::engine::report::build` | arquitetura/11-catalogo-de-funcoes §16.1 | done | core/src/engine/report.rs:35 |
| `core::engine::report::from_context` | arquitetura/11-catalogo-de-funcoes §16.2 | done | core/src/engine/report.rs:62 |
| `core::engine::rival::report` | arquitetura/11-catalogo-de-funcoes §22.1 | done | core/src/engine/rival.rs:84 |
| `core::engine::rival::standing` | arquitetura/11-catalogo-de-funcoes §22.2 | done | core/src/engine/rival.rs:72 |
| `core::engine::roles::others_play` | arquitetura/11-catalogo-de-funcoes §11.4 | done | core/src/engine/roles.rs:86 |
| `core::engine::roles::plays` | arquitetura/11-catalogo-de-funcoes §11.3 | done | core/src/engine/roles.rs:74 |
| `core::engine::roles::role_fit` | arquitetura/11-catalogo-de-funcoes §11.2 | done | core/src/engine/roles.rs:55 |
| `core::engine::roles::role_share` | arquitetura/11-catalogo-de-funcoes §11.1 | done | core/src/engine/roles.rs:41 |
| `core::engine::runes::plan` | arquitetura/11-catalogo-de-funcoes §18.2 | done | core/src/engine/runes.rs:246 |
| `core::engine::runes::RuneIndex::get` | arquitetura/11-catalogo-de-funcoes §18.1 | done | core/src/engine/runes.rs:51 |
| `core::engine::runes::RuneIndex::is_empty` | arquitetura/11-catalogo-de-funcoes §18.1 | done | core/src/engine/runes.rs:54 |
| `core::engine::stats::Estimate::se` | arquitetura/11-catalogo-de-funcoes §19.2 | done | core/src/engine/stats.rs:46 |
| `core::engine::stats::expected_against` | arquitetura/11-catalogo-de-funcoes §19.7 | done | core/src/engine/stats.rs:119 |
| `core::engine::stats::expected_together` | arquitetura/11-catalogo-de-funcoes §19.7 | done | core/src/engine/stats.rs:125 |
| `core::engine::stats::Length::of` | arquitetura/11-catalogo-de-funcoes §19.8 | done | core/src/engine/stats.rs:144 |
| `core::engine::stats::logistic` | arquitetura/11-catalogo-de-funcoes §19.7 | done | core/src/engine/stats.rs:109 |
| `core::engine::stats::logit` | arquitetura/11-catalogo-de-funcoes §19.7 | done | core/src/engine/stats.rs:104 |
| `core::engine::stats::PersonalStats::add_match` | arquitetura/11-catalogo-de-funcoes §19.9 | done | core/src/engine/stats.rs:253 |
| `core::engine::stats::PersonalStats::add_played` | arquitetura/11-catalogo-de-funcoes §19.9 | done | core/src/engine/stats.rs:235 |
| `core::engine::stats::PersonalStats::mark_seen` | arquitetura/11-catalogo-de-funcoes §19.9 | done | core/src/engine/stats.rs:247 |
| `core::engine::stats::PersonalStats::total_games` | arquitetura/11-catalogo-de-funcoes §19.9 | done | core/src/engine/stats.rs:269 |
| `core::engine::stats::posterior_sd` | arquitetura/11-catalogo-de-funcoes §19.6 | done | core/src/engine/stats.rs:100 |
| `core::engine::stats::Record::new` | arquitetura/11-catalogo-de-funcoes §19.1 | done | core/src/engine/stats.rs:29 |
| `core::engine::stats::shrink` | arquitetura/11-catalogo-de-funcoes §19.3 | done | core/src/engine/stats.rs:58 |
| `core::engine::stats::shrunk_delta` | arquitetura/11-catalogo-de-funcoes §19.4 | done | core/src/engine/stats.rs:77 |
| `core::engine::stats::StatsStore::base` | arquitetura/10-modelo-de-dados §19.4 | done | core/src/engine/stats.rs:154 |
| `core::engine::stats::StatsStore::has_global` | arquitetura/10-modelo-de-dados §19.4 | done | core/src/engine/stats.rs:171 |
| `core::engine::stats::StatsStore::has_personal` | arquitetura/10-modelo-de-dados §19.4 | done | core/src/engine/stats.rs:177 |
| `core::engine::stats::StatsStore::lane_profile` | arquitetura/10-modelo-de-dados §19.4 | done | core/src/engine/stats.rs:187 |
| `core::engine::stats::StatsStore::length` | arquitetura/10-modelo-de-dados §19.4 | done | core/src/engine/stats.rs:165 |
| `core::engine::stats::StatsStore::matchup` | arquitetura/10-modelo-de-dados §19.4 | done | core/src/engine/stats.rs:155 |
| `core::engine::stats::StatsStore::personal` | arquitetura/10-modelo-de-dados §19.4 | done | core/src/engine/stats.rs:157 |
| `core::engine::stats::StatsStore::personal_games` | arquitetura/10-modelo-de-dados §19.4 | done | core/src/engine/stats.rs:181 |
| `core::engine::stats::StatsStore::played_in` | arquitetura/10-modelo-de-dados §19.4 | done | core/src/engine/stats.rs:192 |
| `core::engine::stats::StatsStore::synergy` | arquitetura/10-modelo-de-dados §19.4 | done | core/src/engine/stats.rs:156 |
| `core::engine::stats::StatsStore::versus` | arquitetura/10-modelo-de-dados §19.4 | done | core/src/engine/stats.rs:160 |
| `core::engine::stats::strength_for_sd` | arquitetura/11-catalogo-de-funcoes §19.5 | done | core/src/engine/stats.rs:92 |
| `core::engine::team::estimate` | arquitetura/11-catalogo-de-funcoes §14.2 | done | core/src/engine/team.rs:98 |
| `core::engine::team::seats` | arquitetura/11-catalogo-de-funcoes §14.1 | done | core/src/engine/team.rs:68 |
| `core::engine::timeline::BuffTracker::update` | arquitetura/11-catalogo-de-funcoes §29.3 | done | core/src/engine/timeline.rs:246 |
| `core::engine::timeline::build` | arquitetura/11-catalogo-de-funcoes §29.1 | done | core/src/engine/timeline.rs:332 |
| `core::engine::timeline::RawEvent::outcome` | arquitetura/11-catalogo-de-funcoes §29.2 | done | core/src/engine/timeline.rs:93 |
| `core::engine::wincon::analyze` | arquitetura/11-catalogo-de-funcoes §15.1 | done | core/src/engine/wincon.rs:179 |
| `core::history::corte_comum` | arquitetura/11-catalogo-de-funcoes §5.3 | done | core/src/history.rs:183 |
| `core::history::Metric::higher_is_better` | arquitetura/11-catalogo-de-funcoes §5.2 | done | core/src/history.rs:114 |
| `core::history::Metric::phrase` | arquitetura/11-catalogo-de-funcoes §5.2 | done | core/src/history.rs:119 |
| `core::history::played` | arquitetura/11-catalogo-de-funcoes §5.1 | done | core/src/history.rs:203 |
| `core::live::champion_key` | arquitetura/11-catalogo-de-funcoes §4.2 | done | core/src/live.rs:129 |
| `core::live::LiveGame::allies` | arquitetura/11-catalogo-de-funcoes §4.3 | done | core/src/live.rs:84 |
| `core::live::LiveGame::enemies` | arquitetura/11-catalogo-de-funcoes §4.3 | done | core/src/live.rs:95 |
| `core::live::LiveGame::gold_invested` | arquitetura/11-catalogo-de-funcoes §4.3 | done | core/src/live.rs:107 |
| `core::live::LiveGame::me` | arquitetura/11-catalogo-de-funcoes §4.3 | done | core/src/live.rs:80 |
| `core::live::LivePlayer::as_combatant` | arquitetura/11-catalogo-de-funcoes §4.4 | done | core/src/live.rs:119 |
| `core::live::to_live_game` | arquitetura/11-catalogo-de-funcoes §4.1 | done | core/src/live.rs:140 |
| `core::lockfile::Credentials::base_url` | arquitetura/11-catalogo-de-funcoes §2.1 | done | core/src/lockfile.rs:11 |
| `core::lockfile::extract_arg` | arquitetura/11-catalogo-de-funcoes §2.4 | done | core/src/lockfile.rs:37 |
| `core::lockfile::parse_cmdline` | arquitetura/11-catalogo-de-funcoes §2.3 | done | core/src/lockfile.rs:28 |
| `core::lockfile::parse_lockfile` | arquitetura/11-catalogo-de-funcoes §2.2 | done | core/src/lockfile.rs:17 |
| `core::pool::Pool::ids` | arquitetura/11-catalogo-de-funcoes §7.2 | done | core/src/pool.rs:57 |
| `core::pool::Pool::ids_for_role` | arquitetura/11-catalogo-de-funcoes §7.2 | done | core/src/pool.rs:65 |
| `core::pool::Pool::ids_for_role_tier` | arquitetura/11-catalogo-de-funcoes §7.2 | done | core/src/pool.rs:115 |
| `core::pool::Pool::is_empty` | arquitetura/11-catalogo-de-funcoes §7.2 | done | core/src/pool.rs:53 |
| `core::pool::Pool::normalized` | arquitetura/11-catalogo-de-funcoes §7.2 | done | core/src/pool.rs:95 |
| `core::pool::Pool::remove` | arquitetura/11-catalogo-de-funcoes §7.2 | done | core/src/pool.rs:108 |
| `core::pool::Pool::tier_of` | arquitetura/11-catalogo-de-funcoes §7.2 | done | core/src/pool.rs:123 |
| `core::pool::Pool::upsert` | arquitetura/11-catalogo-de-funcoes §7.2 | done | core/src/pool.rs:78 |
| `core::pool::Tier::weight` | arquitetura/11-catalogo-de-funcoes §7.1 | done | core/src/pool.rs:23 |
| `core::prefs::OverlayPrefs::anything_visible` | arquitetura/11-catalogo-de-funcoes §8.2 | done | core/src/prefs.rs:111 |
| `core::prefs::OverlayPrefs::atravessa_o_mouse` | arquitetura/11-catalogo-de-funcoes §8.2 | done | core/src/prefs.rs:124 |
| `core::prefs::OverlayPrefs::set` | arquitetura/11-catalogo-de-funcoes §8.2 | done | core/src/prefs.rs:102 |
| `core::prefs::OverlayPrefs::shows` | arquitetura/11-catalogo-de-funcoes §8.2 | done | core/src/prefs.rs:98 |
| `core::prefs::OverlayPrefs::visible` | arquitetura/11-catalogo-de-funcoes §8.2 | done | core/src/prefs.rs:115 |
| `core::prefs::Section::detail` | arquitetura/11-catalogo-de-funcoes §8.1 | done | core/src/prefs.rs:46 |
| `core::prefs::Section::label` | arquitetura/11-catalogo-de-funcoes §8.1 | done | core/src/prefs.rs:35 |
| `core::riot::Region::from_lcu` | arquitetura/11-catalogo-de-funcoes §6.1 | done | core/src/riot.rs:16 |
| `core::riot::Region::from_platform` | arquitetura/11-catalogo-de-funcoes §6.1 | done | core/src/riot.rs:46 |
| `core::riot::Region::platforms` | arquitetura/11-catalogo-de-funcoes §6.1 | done | core/src/riot.rs:57 |
| `core::riot::split_riot_id` | arquitetura/11-catalogo-de-funcoes §6.2 | done | core/src/riot.rs:84 |
| `core::riot::urlencode` | arquitetura/11-catalogo-de-funcoes §6.2 | done | core/src/riot.rs:95 |

## const

| Item | Fonte | Estado | Onde / motivo |
|---|---|---|---|
| `casca::recorder::MAX_FRAMES` | produto/regras-e-pesos RN-28 — O teto e o piso da gravação (`ca | done | src-tauri/src/recorder.rs:23 |
| `casca::riot::RANKED_SOLO_QUEUE` | produto/regras-e-pesos RN-31 — O histórico pessoal da casca (`c | done | src-tauri/src/riot.rs:26 |
| `coletor::aggregate::MIN_SPREAD_ENTRIES` | produto/regras-e-pesos RN-32 — O que entra na base | done | tools/ingest/src/aggregate.rs:342 |
| `coletor::aggregate::PARTICIPANTS` | produto/regras-e-pesos RN-32 — O que entra na base | done | tools/ingest/src/aggregate.rs:36 |
| `coletor::aggregate::TEAM_SIZE` | produto/regras-e-pesos RN-32 — O que entra na base | done | tools/ingest/src/aggregate.rs:35 |
| `coletor::aggregate::USEFUL` | produto/regras-e-pesos RN-32 — O que entra na base | done | tools/ingest/src/aggregate.rs:390 |
| `core::draft::RANKED_QUEUES` | arquitetura/10-modelo-de-dados §3.13 | done | core/src/draft.rs:274 |
| `core::engine::BASE_STRENGTH` | produto/regras-e-pesos RN-2 — A força da winrate base é escolhi | done | core/src/engine/mod.rs:36 |
| `core::engine::build::MIN_ITEM_COST` | produto/regras-e-pesos RN-19 — O que entra na lista e como o it | done | core/src/engine/build.rs:24 |
| `core::engine::coach::BUCKET` | produto/regras-e-pesos RN-27 — Os limiares das observações | done | core/src/engine/coach.rs:157 |
| `core::engine::coach::CLUSTER_DEATHS` | produto/regras-e-pesos RN-27 — Os limiares das observações | done | core/src/engine/coach.rs:28 |
| `core::engine::coach::CLUSTER_WINDOW` | produto/regras-e-pesos RN-27 — Os limiares das observações | done | core/src/engine/coach.rs:27 |
| `core::engine::coach::IDLE_GOLD` | produto/regras-e-pesos RN-27 — Os limiares das observações | done | core/src/engine/coach.rs:31 |
| `core::engine::coach::LEGENDARY_COST` | produto/regras-e-pesos RN-27 — Os limiares das observações | done | core/src/engine/coach.rs:24 |
| `core::engine::coach::MIN_DURATION` | produto/regras-e-pesos RN-27 — Os limiares das observações | done | core/src/engine/coach.rs:34 |
| `core::engine::coach::MIN_FRAMES` | produto/regras-e-pesos O piso da RN-28, em `core::engine::coach | done | core/src/engine/coach.rs:38 |
| `core::engine::COMP_SD` | produto/regras-e-pesos RN-4 — A composição tem sempre um desvio | done | core/src/engine/mod.rs:47 |
| `core::engine::context::EPS` | produto/regras-e-pesos RN-7 — O nível da prévia e a plausibilid | done | core/src/engine/context.rs:36 |
| `core::engine::context::PREGAME_LEVEL` | arquitetura/10-modelo-de-dados §10.1 | done | core/src/engine/context.rs:32 |
| `core::engine::context::PREGAME_LEVEL` | produto/regras-e-pesos RN-7 — O nível da prévia e a plausibilid | done | core/src/engine/context.rs:32 |
| `core::engine::gold::PASSIVE_PER_SECOND` | produto/regras-e-pesos RN-22 — A renda reconstruída | done | core/src/engine/gold.rs:26 |
| `core::engine::gold::PASSIVE_START` | produto/regras-e-pesos RN-22 — A renda reconstruída | done | core/src/engine/gold.rs:25 |
| `core::engine::gold::PER_ASSIST` | produto/regras-e-pesos RN-22 — A renda reconstruída | done | core/src/engine/gold.rs:35 |
| `core::engine::gold::PER_CS` | produto/regras-e-pesos RN-22 — A renda reconstruída | done | core/src/engine/gold.rs:31 |
| `core::engine::gold::PER_KILL` | produto/regras-e-pesos RN-22 — A renda reconstruída | done | core/src/engine/gold.rs:34 |
| `core::engine::gold::STARTING` | produto/regras-e-pesos RN-22 — A renda reconstruída | done | core/src/engine/gold.rs:22 |
| `core::engine::gold::UNSPENT_ALERT` | produto/regras-e-pesos RN-22 — A renda reconstruída | done | core/src/engine/gold.rs:39 |
| `core::engine::lanes::BREAKPOINTS` | arquitetura/10-modelo-de-dados §27.1 | done | core/src/engine/lanes.rs:38 |
| `core::engine::lanes::BREAKPOINTS` | produto/regras-e-pesos RN-20 — O peso do duelo na chance da rot | done | core/src/engine/lanes.rs:38 |
| `core::engine::lanes::CEILING` | produto/regras-e-pesos RN-20 — O peso do duelo na chance da rot | done | core/src/engine/lanes.rs:32 |
| `core::engine::lanes::FLOOR` | produto/regras-e-pesos RN-20 — O peso do duelo na chance da rot | done | core/src/engine/lanes.rs:31 |
| `core::engine::lanes::MAX_REASONS` | produto/regras-e-pesos RN-20 — O peso do duelo na chance da rot | done | core/src/engine/lanes.rs:34 |
| `core::engine::lanes::OBJECTIVE_WINDOW` | produto/regras-e-pesos RN-21 — Quem briga junto, e qual escaram | done | core/src/engine/lanes.rs:464 |
| `core::engine::lanes::W_DUEL` | produto/regras-e-pesos RN-20 — O peso do duelo na chance da rot | done | core/src/engine/lanes.rs:28 |
| `core::engine::MATCHUP_PRIOR_SD` | produto/regras-e-pesos RN-3 — O quanto cada efeito costuma se e | done | core/src/engine/mod.rs:41 |
| `core::engine::MAX_REASONS` | produto/regras-e-pesos RN-6 — O corte da winrate e os limiares  | done | core/src/engine/mod.rs:55 |
| `core::engine::minimax::FRONTLINE_FROM` | produto/regras-e-pesos RN-11 — O segundo modelo da composição | done | core/src/engine/minimax.rs:34 |
| `core::engine::minimax::W_BALANCE` | produto/regras-e-pesos RN-11 — O segundo modelo da composição | done | core/src/engine/minimax.rs:30 |
| `core::engine::minimax::W_FRONTLINE` | produto/regras-e-pesos RN-11 — O segundo modelo da composição | done | core/src/engine/minimax.rs:32 |
| `core::engine::PERSONAL_PRIOR_SD` | produto/regras-e-pesos RN-3 — O quanto cada efeito costuma se e | done | core/src/engine/mod.rs:43 |
| `core::engine::poolgap::CORE_SIZE` | arquitetura/10-modelo-de-dados §17.1 | done | core/src/engine/poolgap.rs:26 |
| `core::engine::poolgap::CORE_SIZE` | produto/regras-e-pesos RN-15 — Oito por rota, três escolhidos | done | core/src/engine/poolgap.rs:26 |
| `core::engine::poolgap::LEVEL` | produto/regras-e-pesos RN-16 — As características da pool | done | core/src/engine/poolgap.rs:29 |
| `core::engine::poolgap::MAX_REASONS` | produto/regras-e-pesos RN-16 — As características da pool | done | core/src/engine/poolgap.rs:38 |
| `core::engine::poolgap::POOL_SIZE` | arquitetura/10-modelo-de-dados §17.1 | done | core/src/engine/poolgap.rs:25 |
| `core::engine::poolgap::POOL_SIZE` | produto/regras-e-pesos RN-15 — Oito por rota, três escolhidos | done | core/src/engine/poolgap.rs:25 |
| `core::engine::poolgap::RELEVANT` | produto/regras-e-pesos RN-16 — As características da pool | done | core/src/engine/poolgap.rs:36 |
| `core::engine::poolgap::ROBUST` | produto/regras-e-pesos RN-16 — As características da pool | done | core/src/engine/poolgap.rs:32 |
| `core::engine::poolgap::Trait::ALL` | arquitetura/10-modelo-de-dados §17.2 | done | core/src/engine/poolgap.rs:61 |
| `core::engine::PRIOR_RATE` | produto/regras-e-pesos RN-1 — A partida neutra vale 50% | done | core/src/engine/mod.rs:34 |
| `core::engine::priority::BEHIND_MAX` | produto/regras-e-pesos RN-25 — Os pesos de tela da prioridade | done | core/src/engine/priority.rs:26 |
| `core::engine::priority::DEAD_ENEMY_MAX` | produto/regras-e-pesos RN-25 — Os pesos de tela da prioridade | done | core/src/engine/priority.rs:24 |
| `core::engine::priority::DEAD_JUNGLER` | produto/regras-e-pesos RN-25 — Os pesos de tela da prioridade | done | core/src/engine/priority.rs:25 |
| `core::engine::priority::OBJECTIVE_MAX` | produto/regras-e-pesos RN-25 — Os pesos de tela da prioridade | done | core/src/engine/priority.rs:27 |
| `core::engine::priority::OBJECTIVE_WINDOW` | produto/regras-e-pesos RN-25 — Os pesos de tela da prioridade | done | core/src/engine/priority.rs:30 |
| `core::engine::profile::EPS` | produto/regras-e-pesos RN-29 — Quando uma leitura do perfil con | done | core/src/engine/profile.rs:35 |
| `core::engine::profile::MIN_GAMES` | produto/regras-e-pesos RN-29 — Quando uma leitura do perfil con | done | core/src/engine/profile.rs:32 |
| `core::engine::rival::AHEAD_FLOOR` | arquitetura/10-modelo-de-dados §22.1 | done | core/src/engine/rival.rs:26 |
| `core::engine::rival::AHEAD_FLOOR` | produto/regras-e-pesos RN-23 — A margem de ouro e o começo da p | done | core/src/engine/rival.rs:26 |
| `core::engine::rival::AHEAD_SHARE` | arquitetura/10-modelo-de-dados §22.1 | done | core/src/engine/rival.rs:27 |
| `core::engine::rival::AHEAD_SHARE` | produto/regras-e-pesos RN-23 — A margem de ouro e o começo da p | done | core/src/engine/rival.rs:27 |
| `core::engine::rival::MIN_SECONDS` | arquitetura/10-modelo-de-dados §22.1 | done | core/src/engine/rival.rs:21 |
| `core::engine::rival::MIN_SECONDS` | produto/regras-e-pesos RN-23 — A margem de ouro e o começo da p | done | core/src/engine/rival.rs:21 |
| `core::engine::roles::ROLE_SHARE` | produto/regras-e-pesos RN-8 — Quem joga a rota (`core::engine:: | done | core/src/engine/roles.rs:18 |
| `core::engine::roles::ROLES` | arquitetura/10-modelo-de-dados §11.1 | done | core/src/engine/roles.rs:15 |
| `core::engine::runes::MAX_SUGGESTIONS` | produto/regras-e-pesos RN-18 — As regras do confronto | done | core/src/engine/runes.rs:26 |
| `core::engine::stats::Length::ALL` | arquitetura/10-modelo-de-dados §19.3 | done | core/src/engine/stats.rs:142 |
| `core::engine::SYNERGY_PRIOR_SD` | produto/regras-e-pesos RN-3 — O quanto cada efeito costuma se e | done | core/src/engine/mod.rs:42 |
| `core::engine::team::CEILING` | produto/regras-e-pesos RN-12 — A trava da chance do time | done | core/src/engine/team.rs:24 |
| `core::engine::team::FLOOR` | produto/regras-e-pesos RN-12 — A trava da chance do time | done | core/src/engine/team.rs:23 |
| `core::engine::team::MAX_TERMS` | produto/regras-e-pesos RN-12 — A trava da chance do time | done | core/src/engine/team.rs:25 |
| `core::engine::timeline::BUFF_FRESH` | produto/regras-e-pesos RN-24 — Os tempos dos objetivos e a leit | done | core/src/engine/timeline.rs:233 |
| `core::engine::VERSUS_PRIOR_SD` | produto/regras-e-pesos RN-3 — O quanto cada efeito costuma se e | done | core/src/engine/mod.rs:46 |
| `core::engine::W_DAMAGE_BALANCE` | produto/regras-e-pesos RN-5 — Os pesos da composição no ranking | done | core/src/engine/mod.rs:52 |
| `core::engine::W_FRONTLINE` | produto/regras-e-pesos RN-5 — Os pesos da composição no ranking | done | core/src/engine/mod.rs:53 |
| `core::engine::wincon::EDGE` | produto/regras-e-pesos RN-13 — O que a condição de vitória afir | done | core/src/engine/wincon.rs:27 |
| `core::engine::wincon::LEAN` | produto/regras-e-pesos RN-13 — O que a condição de vitória afir | done | core/src/engine/wincon.rs:36 |
| `core::engine::wincon::MAX_EDGES` | produto/regras-e-pesos RN-13 — O que a condição de vitória afir | done | core/src/engine/wincon.rs:32 |
| `core::engine::wincon::MIN_KNOWN` | produto/regras-e-pesos RN-13 — O que a condição de vitória afir | done | core/src/engine/wincon.rs:30 |
| `core::engine::wincon::ONE_SIDED` | produto/regras-e-pesos RN-13 — O que a condição de vitória afir | done | core/src/engine/wincon.rs:24 |
| `core::history::Metric::ALL` | arquitetura/10-modelo-de-dados §5.5 | done | core/src/history.rs:106 |
| `core::history::MIN_DURATION` | produto/regras-e-pesos RN-30 — Os cortes de partida do históric | done | core/src/history.rs:22 |
| `core::pool::BAN_POOL_MAX` | arquitetura/10-modelo-de-dados §7.3 | done | core/src/pool.rs:42 |
| `core::prefs::Section::ALL` | arquitetura/10-modelo-de-dados §8.1 | done | core/src/prefs.rs:26 |

## wire

| Item | Fonte | Estado | Onde / motivo |
|---|---|---|---|
| `Analysis ↔ PoolAnalysis` | arquitetura/10-modelo-de-dados §17.4 | done | core/src/engine/poolgap.rs:250 |
| `BanSuggestion ↔ BanSuggestion` | arquitetura/10-modelo-de-dados §12.2 | done | core/src/engine/minimax.rs:50 |
| `Buff ↔ Buff` | arquitetura/10-modelo-de-dados §29.8 | done | core/src/engine/timeline.rs:216 |
| `BuildContext ↔ BuildContext` | arquitetura/10-modelo-de-dados §26.3 | done | core/src/engine/build.rs:45 |
| `Candidate ↔ PoolCandidate` | arquitetura/10-modelo-de-dados §17.5 | done | core/src/engine/poolgap.rs:257 |
| `ChampionLane ↔ ChampionLane` | arquitetura/10-modelo-de-dados §21.3 | done | core/src/engine/profile.rs:69 |
| `ConnStatus ↔ ConnStatus` | arquitetura/10-modelo-de-dados §3.8 | done | core/src/draft.rs:116 |
| `DraftPrefs ↔ DraftPrefs` | arquitetura/10-modelo-de-dados §8.3 | done | core/src/prefs.rs:130 |
| `DraftSlot ↔ DraftSlot` | arquitetura/10-modelo-de-dados §3.6 | done | core/src/draft.rs:78 |
| `DraftState ↔ DraftState` | arquitetura/10-modelo-de-dados §3.7 | done | core/src/draft.rs:95 |
| `EngineStatus ↔ EngineStatus` | arquitetura/10-modelo-de-dados §32.1 | done | src-tauri/src/lib.rs:48 |
| `Finding ↔ PoolFinding` | arquitetura/10-modelo-de-dados §17.3 | done | core/src/engine/poolgap.rs:241 |
| `Focus ↔ SkirmishFocus` | arquitetura/10-modelo-de-dados §27.6 | done | core/src/engine/lanes.rs:466 |
| `GoldPlayer ↔ GoldPlayer` | arquitetura/10-modelo-de-dados §28.1 | done | core/src/engine/gold.rs:41 |
| `GoldReport ↔ GoldReport` | arquitetura/10-modelo-de-dados §28.2 | done | core/src/engine/gold.rs:61 |
| `ItemSuggestion ↔ ItemSuggestion` | arquitetura/10-modelo-de-dados §26.4 | done | core/src/engine/build.rs:56 |
| `LaneOutlook ↔ LaneOutlook` | arquitetura/10-modelo-de-dados §27.3 | done | core/src/engine/lanes.rs:155 |
| `LanePriority ↔ LanePriority` | arquitetura/10-modelo-de-dados §30.1 | done | core/src/engine/priority.rs:32 |
| `LaneProfile ↔ LaneProfile` | arquitetura/10-modelo-de-dados §21.4 | done | core/src/engine/profile.rs:88 |
| `LaneReport ↔ LaneReport` | arquitetura/10-modelo-de-dados §16.1 | done | core/src/engine/report.rs:17 |
| `LevelOdds ↔ LevelOdds` | arquitetura/10-modelo-de-dados §27.2 | done | core/src/engine/lanes.rs:50 |
| `LiveGame ↔ LiveGame` | arquitetura/10-modelo-de-dados §4.5 | done | core/src/live.rs:72 |
| `LivePlayer ↔ LivePlayer` | arquitetura/10-modelo-de-dados §4.4 | done | core/src/live.rs:54 |
| `LiveSnapshot ↔ LiveSnapshot` | arquitetura/10-modelo-de-dados §32.3 | done | src-tauri/src/live.rs:17 |
| `MatchupLane ↔ MatchupLane` | arquitetura/10-modelo-de-dados §21.2 | done | core/src/engine/profile.rs:61 |
| `Metric ↔ LaneMetric` | arquitetura/10-modelo-de-dados §5.5 | done | core/src/history.rs:90 |
| `MinimaxPick ↔ MinimaxPick` | arquitetura/10-modelo-de-dados §12.1 | done | core/src/engine/minimax.rs:36 |
| `Note ↔ CoachNote` | arquitetura/10-modelo-de-dados §31.3 | done | core/src/engine/coach.rs:111 |
| `OverlayPrefs ↔ OverlayPrefs` | arquitetura/10-modelo-de-dados §8.2 | done | core/src/prefs.rs:64 |
| `Point ↔ Point` | arquitetura/10-modelo-de-dados §31.4 | done | core/src/engine/coach.rs:122 |
| `Point ↔ WinPoint` | arquitetura/10-modelo-de-dados §15.3 | done | core/src/engine/wincon.rs:55 |
| `Pool ↔ Pool` | arquitetura/10-modelo-de-dados §7.4 | done | core/src/pool.rs:44 |
| `PoolEntry ↔ PoolEntry` | arquitetura/10-modelo-de-dados §7.2 | done | core/src/pool.rs:32 |
| `PoolPlan ↔ PoolPlan` | arquitetura/10-modelo-de-dados §32.2 | done | src-tauri/src/lib.rs:371 |
| `PriorityReport ↔ PriorityReport` | arquitetura/10-modelo-de-dados §30.2 | done | core/src/engine/priority.rs:42 |
| `RawScores ↔ LiveScores` | arquitetura/10-modelo-de-dados §4.3 | done | core/src/live.rs:44 |
| `Reading ↔ LaneReading` | arquitetura/10-modelo-de-dados §21.1 | done | core/src/engine/profile.rs:37 |
| `Report ↔ CoachReport` | arquitetura/10-modelo-de-dados §31.6 | done | core/src/engine/coach.rs:159 |
| `Respawn ↔ Respawn` | arquitetura/10-modelo-de-dados §29.7 | done | core/src/engine/timeline.rs:144 |
| `Rival ↔ Rival` | arquitetura/10-modelo-de-dados §22.4 | done | core/src/engine/rival.rs:46 |
| `Role ↔ Role` | arquitetura/10-modelo-de-dados §23.1 | done | core/src/engine/champion.rs:4 |
| `RunePlan ↔ RunePlan` | arquitetura/10-modelo-de-dados §18.4 | done | core/src/engine/runes.rs:72 |
| `RuneSuggestion ↔ RuneSuggestion` | arquitetura/10-modelo-de-dados §18.3 | done | core/src/engine/runes.rs:59 |
| `Scored ↔ ScoredChampion` | arquitetura/10-modelo-de-dados §9.1 | done | core/src/engine/mod.rs:57 |
| `Section ↔ OverlaySection` | arquitetura/10-modelo-de-dados §8.1 | done | core/src/prefs.rs:13 |
| `Series ↔ Series` | arquitetura/10-modelo-de-dados §31.5 | done | core/src/engine/coach.rs:133 |
| `Sides ↔ Sides` | arquitetura/10-modelo-de-dados §29.4 | done | core/src/engine/timeline.rs:111 |
| `Skirmish ↔ Skirmish` | arquitetura/10-modelo-de-dados §27.4 | done | core/src/engine/lanes.rs:187 |
| `Spawn ↔ Spawn` | arquitetura/10-modelo-de-dados §29.5 | done | core/src/engine/timeline.rs:117 |
| `TeamEstimate ↔ TeamEstimate` | arquitetura/10-modelo-de-dados §14.3 | done | core/src/engine/team.rs:47 |
| `TeamTerm ↔ TeamTerm` | arquitetura/10-modelo-de-dados §14.2 | done | core/src/engine/team.rs:38 |
| `Tier ↔ Tier` | arquitetura/10-modelo-de-dados §7.1 | done | core/src/pool.rs:13 |
| `Timeline ↔ Timeline` | arquitetura/10-modelo-de-dados §29.6 | done | core/src/engine/timeline.rs:128 |
| `Trait ↔ PoolTrait` | arquitetura/10-modelo-de-dados §17.2 | done | core/src/engine/poolgap.rs:41 |
| `WinCondition ↔ WinCondition` | arquitetura/10-modelo-de-dados §15.4 | done | core/src/engine/wincon.rs:63 |

## command

| Item | Fonte | Estado | Onde / motivo |
|---|---|---|---|
| `ban_suggestions` | arquitetura/12-contrato-da-casca §2 | done | src-tauri/src/lib.rs:256 |
| `build_suggestions` | arquitetura/12-contrato-da-casca §2 | done | src-tauri/src/lib.rs:273 |
| `engine_status` | arquitetura/12-contrato-da-casca §2 | done | src-tauri/src/lib.rs:129 |
| `get_autostart` | arquitetura/12-contrato-da-casca §2 | done | src-tauri/src/lib.rs:480 |
| `get_draft_prefs` | arquitetura/12-contrato-da-casca §2 | done | src-tauri/src/lib.rs:223 |
| `get_enemy_roles` | arquitetura/12-contrato-da-casca §2 | done | src-tauri/src/lib.rs:205 |
| `get_live` | arquitetura/12-contrato-da-casca §2 | done | src-tauri/src/lib.rs:267 |
| `get_overlay_prefs` | arquitetura/12-contrato-da-casca §2 | done | src-tauri/src/lib.rs:455 |
| `get_pool` | arquitetura/12-contrato-da-casca §2 | done | src-tauri/src/lib.rs:441 |
| `get_snapshot` | arquitetura/12-contrato-da-casca §2 | done | src-tauri/src/lib.rs:121 |
| `gold_report` | arquitetura/12-contrato-da-casca §2 | done | src-tauri/src/lib.rs:359 |
| `lane_analysis` | arquitetura/12-contrato-da-casca §2 | done | src-tauri/src/lib.rs:306 |
| `lane_profile` | arquitetura/12-contrato-da-casca §2 | done | src-tauri/src/lib.rs:237 |
| `last_review` | arquitetura/12-contrato-da-casca §2 | done | src-tauri/src/lib.rs:366 |
| `minimax_draft` | arquitetura/12-contrato-da-casca §2 | done | src-tauri/src/lib.rs:243 |
| `pool_suggestions` | arquitetura/12-contrato-da-casca §2 | done | src-tauri/src/lib.rs:384 |
| `rank_draft` | arquitetura/12-contrato-da-casca §2 | done | src-tauri/src/lib.rs:168 |
| `rune_plan` | arquitetura/12-contrato-da-casca §2 | done | src-tauri/src/lib.rs:408 |
| `set_autostart` | arquitetura/12-contrato-da-casca §2 | done | src-tauri/src/lib.rs:486 |
| `set_draft_prefs` | arquitetura/12-contrato-da-casca §2 | done | src-tauri/src/lib.rs:228 |
| `set_draft_role` | arquitetura/12-contrato-da-casca §2 | done | src-tauri/src/lib.rs:182 |
| `set_enemy_role` | arquitetura/12-contrato-da-casca §2 | done | src-tauri/src/lib.rs:191 |
| `set_overlay_prefs` | arquitetura/12-contrato-da-casca §2 | done | src-tauri/src/lib.rs:464 |
| `set_pool` | arquitetura/12-contrato-da-casca §2 | done | src-tauri/src/lib.rs:446 |
| `set_simulated_draft` | arquitetura/12-contrato-da-casca §2 | done | src-tauri/src/lib.rs:218 |
| `skirmish_focus` | arquitetura/12-contrato-da-casca §2 | done | src-tauri/src/lib.rs:337 |

## event

| Item | Fonte | Estado | Onde / motivo |
|---|---|---|---|
| `presage://draft` | arquitetura/12-contrato-da-casca §3 | done | src-tauri/src/watcher.rs:36 |
| `presage://engine` | arquitetura/12-contrato-da-casca §3 | done | src-tauri/src/lib.rs:40 |
| `presage://live` | arquitetura/12-contrato-da-casca §3 | done | src-tauri/src/watcher.rs:37 |
| `presage://prefs` | arquitetura/12-contrato-da-casca §3 | done | src-tauri/src/watcher.rs:41 |
| `presage://review` | arquitetura/12-contrato-da-casca §3 | done | src-tauri/src/watcher.rs:38 |
| `presage://status` | arquitetura/12-contrato-da-casca §3 | done | src-tauri/src/watcher.rs:35 |

## file

| Item | Fonte | Estado | Onde / motivo |
|---|---|---|---|
| `base-de-partidas.json` | arquitetura/12-contrato-da-casca §6 | done | src-tauri/src/lib.rs:46 |
| `champion.json` | arquitetura/12-contrato-da-casca §6 | done | src-tauri/src/ddragon.rs:156 |
| `champions.json` | arquitetura/12-contrato-da-casca §6 | done | src-tauri/src/ddragon.rs:11 |
| `coleta-em-andamento.json` | arquitetura/12-contrato-da-casca §6 | done | tools/ingest/src/main.rs:95 |
| `draft.json` | arquitetura/12-contrato-da-casca §6 | done | src-tauri/src/lib.rs:43 |
| `gravacao.json` | arquitetura/12-contrato-da-casca §6 | done | src-tauri/src/recorder.rs:19 |
| `item.json` | arquitetura/12-contrato-da-casca §6 | done | src-tauri/src/ddragon.rs:274 |
| `items.json` | arquitetura/12-contrato-da-casca §6 | done | src-tauri/src/ddragon.rs:12 |
| `LolClientInstalls.json` | arquitetura/12-contrato-da-casca §6 | done | src-tauri/src/lcu/credentials.rs:85 |
| `overlay.json` | arquitetura/12-contrato-da-casca §6 | done | src-tauri/src/arquitetura.rs:186 |
| `personal-v2.json` | arquitetura/12-contrato-da-casca §6 | done | src-tauri/src/profile.rs:21 |
| `pool.json` | arquitetura/12-contrato-da-casca §6 | done | src-tauri/src/lib.rs:41 |
| `runas.json` | arquitetura/12-contrato-da-casca §6 | done | src-tauri/src/ddragon.rs:13 |
| `runesReforged.json` | arquitetura/12-contrato-da-casca §6 | done | src-tauri/src/ddragon.rs:221 |
| `ultima-partida.json` | arquitetura/12-contrato-da-casca §6 | done | src-tauri/src/recorder.rs:18 |
| `versions.json` | arquitetura/12-contrato-da-casca §6 | done | src-tauri/src/ddragon.rs:112 |

## flag

| Item | Fonte | Estado | Onde / motivo |
|---|---|---|---|
| `--ajuda` | arquitetura/05-coletor §1 | done | tools/ingest/src/main.rs:149 |
| `--estado` | arquitetura/05-coletor §1 | done | tools/ingest/src/main.rs:145 |
| `--fila` | arquitetura/05-coletor §1 | done | tools/ingest/src/main.rs:139 |
| `--help` | arquitetura/05-coletor §1 | done | tools/ingest/src/main.rs:149 |
| `--jogadores` | arquitetura/05-coletor §1 | done | tools/ingest/src/main.rs:137 |
| `--limite` | arquitetura/05-coletor §1 | done | tools/ingest/src/main.rs:143 |
| `--min-jogos` | arquitetura/05-coletor §1 | done | tools/ingest/src/main.rs:141 |
| `--paralelo` | arquitetura/05-coletor §1 | done | tools/ingest/src/main.rs:142 |
| `--patch` | arquitetura/05-coletor §1 | done | tools/ingest/src/main.rs:140 |
| `--plano` | arquitetura/05-coletor §1 | done | tools/ingest/src/main.rs:147 |
| `--por-jogador` | arquitetura/05-coletor §1 | done | tools/ingest/src/main.rs:138 |
| `--regiao` | arquitetura/05-coletor §1 | done | tools/ingest/src/main.rs:135 |
| `--retomar` | arquitetura/05-coletor §1 | done | tools/ingest/src/main.rs:146 |
| `--saida` | arquitetura/05-coletor §1 | done | tools/ingest/src/main.rs:144 |
| `--tier` | arquitetura/05-coletor §1 | done | tools/ingest/src/main.rs:136 |
| `--verificar` | arquitetura/05-coletor §1 | done | tools/ingest/src/main.rs:148 |

## test

| Item | Fonte | Estado | Onde / motivo |
|---|---|---|---|
| `T-BUILD-1` | arquitetura/13-testes §2 | done | core/src/engine/build.rs:473 |
| `T-BUILD-10` | arquitetura/13-testes §2 | done | core/src/engine/build.rs:451 |
| `T-BUILD-2` | arquitetura/13-testes §2 | done | core/src/engine/build.rs:487 |
| `T-BUILD-3` | arquitetura/13-testes §2 | done | core/src/engine/build.rs:502 |
| `T-BUILD-4` | arquitetura/13-testes §2 | done | core/src/engine/build.rs:527 |
| `T-BUILD-5` | arquitetura/13-testes §2 | done | core/src/engine/build.rs:551 |
| `T-BUILD-6` | arquitetura/13-testes §2 | done | core/src/engine/build.rs:541 |
| `T-BUILD-7` | arquitetura/13-testes §2 | done | core/src/engine/build.rs:571 |
| `T-BUILD-8` | arquitetura/13-testes §2 | done | core/src/engine/build.rs:588 |
| `T-BUILD-9` | arquitetura/13-testes §2 | done | core/src/engine/build.rs:597 |
| `T-CASCA-1` | arquitetura/13-testes §2 | done | src-tauri/src/analysis.rs:165 |
| `T-CASCA-10` | arquitetura/13-testes §2 | done | src-tauri/src/lcu/credentials.rs:211 |
| `T-CASCA-11` | arquitetura/13-testes §2 | done | src-tauri/src/lcu/credentials.rs:225 |
| `T-CASCA-12` | arquitetura/13-testes §2 | done | src-tauri/src/lcu/credentials.rs:236 |
| `T-CASCA-13` | arquitetura/13-testes §2 | done | src-tauri/src/lib.rs:617 |
| `T-CASCA-14` | arquitetura/13-testes §2 | done | src-tauri/src/lib.rs:636 |
| `T-CASCA-15` | arquitetura/13-testes §2 | done | src-tauri/src/recorder.rs:96 |
| `T-CASCA-16` | arquitetura/13-testes §2 | done | src-tauri/src/recorder.rs:113 |
| `T-CASCA-17` | arquitetura/13-testes §2 | done | src-tauri/src/recorder.rs:132 |
| `T-CASCA-18` | arquitetura/13-testes §2 | done | src-tauri/src/recorder.rs:141 |
| `T-CASCA-19` | arquitetura/13-testes §2 | done | src-tauri/src/recorder.rs:155 |
| `T-CASCA-2` | arquitetura/13-testes §2 | done | src-tauri/src/analysis.rs:182 |
| `T-CASCA-20` | arquitetura/13-testes §2 | done | src-tauri/src/watcher.rs:466 |
| `T-CASCA-21` | arquitetura/13-testes §2 | done | src-tauri/src/storage.rs:62 |
| `T-CASCA-22` | arquitetura/13-testes §2 | done | src-tauri/src/watcher.rs:456 |
| `T-CASCA-23` | arquitetura/13-testes §2 | done | src-tauri/src/storage.rs:77 |
| `T-CASCA-24` | arquitetura/13-testes §2 | done | src-tauri/src/arquitetura.rs:43 |
| `T-CASCA-25` | arquitetura/13-testes §2 | done | src-tauri/src/arquitetura.rs:54 |
| `T-CASCA-26` | arquitetura/13-testes §2 | done | src-tauri/src/arquitetura.rs:92 |
| `T-CASCA-27` | arquitetura/13-testes §2 | done | src-tauri/src/arquitetura.rs:127 |
| `T-CASCA-28` | arquitetura/13-testes §2 | done | src-tauri/src/arquitetura.rs:138 |
| `T-CASCA-29` | arquitetura/13-testes §2 | done | src-tauri/src/arquitetura.rs:150 |
| `T-CASCA-3` | arquitetura/13-testes §2 | done | src-tauri/src/analysis.rs:192 |
| `T-CASCA-30` | arquitetura/13-testes §2 | done | src-tauri/src/lib.rs:594 |
| `T-CASCA-31` | arquitetura/13-testes §2 | done | src-tauri/src/arquitetura.rs:182 |
| `T-CASCA-4` | arquitetura/13-testes §2 | done | src-tauri/src/analysis.rs:205 |
| `T-CASCA-5` | arquitetura/13-testes §2 | done | src-tauri/src/analysis.rs:216 |
| `T-CASCA-6` | arquitetura/13-testes §2 | done | src-tauri/src/analysis.rs:239 |
| `T-CASCA-7` | arquitetura/13-testes §2 | done | src-tauri/src/analysis.rs:255 |
| `T-CASCA-8` | arquitetura/13-testes §2 | done | src-tauri/src/lcu/credentials.rs:181 |
| `T-CASCA-9` | arquitetura/13-testes §2 | done | src-tauri/src/lcu/credentials.rs:192 |
| `T-COLETOR-1` | arquitetura/13-testes §2 | done | tools/ingest/src/main.rs:717 |
| `T-COLETOR-10` | arquitetura/13-testes §2 | done | tools/ingest/src/api.rs:303 |
| `T-COLETOR-11` | arquitetura/13-testes §2 | done | tools/ingest/src/api.rs:321 |
| `T-COLETOR-12` | arquitetura/13-testes §2 | done | tools/ingest/src/api.rs:328 |
| `T-COLETOR-13` | arquitetura/13-testes §2 | done | tools/ingest/src/api.rs:338 |
| `T-COLETOR-14` | arquitetura/13-testes §2 | done | tools/ingest/src/aggregate.rs:448 |
| `T-COLETOR-15` | arquitetura/13-testes §2 | done | tools/ingest/src/aggregate.rs:526 |
| `T-COLETOR-16` | arquitetura/13-testes §2 | done | tools/ingest/src/aggregate.rs:537 |
| `T-COLETOR-17` | arquitetura/13-testes §2 | done | tools/ingest/src/aggregate.rs:556 |
| `T-COLETOR-18` | arquitetura/13-testes §2 | done | tools/ingest/src/aggregate.rs:456 |
| `T-COLETOR-19` | arquitetura/13-testes §2 | done | tools/ingest/src/aggregate.rs:472 |
| `T-COLETOR-2` | arquitetura/13-testes §2 | done | tools/ingest/src/main.rs:760 |
| `T-COLETOR-20` | arquitetura/13-testes §2 | done | tools/ingest/src/aggregate.rs:613 |
| `T-COLETOR-21` | arquitetura/13-testes §2 | done | tools/ingest/src/aggregate.rs:660 |
| `T-COLETOR-22` | arquitetura/13-testes §2 | done | tools/ingest/src/aggregate.rs:671 |
| `T-COLETOR-23` | arquitetura/13-testes §2 | done | tools/ingest/src/aggregate.rs:702 |
| `T-COLETOR-24` | arquitetura/13-testes §2 | done | tools/ingest/src/aggregate.rs:739 |
| `T-COLETOR-25` | arquitetura/13-testes §2 | done | tools/ingest/src/aggregate.rs:514 |
| `T-COLETOR-26` | arquitetura/13-testes §2 | done | tools/ingest/src/aggregate.rs:755 |
| `T-COLETOR-27` | arquitetura/13-testes §2 | done | tools/ingest/src/aggregate.rs:770 |
| `T-COLETOR-28` | arquitetura/13-testes §2 | done | tools/ingest/src/aggregate.rs:792 |
| `T-COLETOR-29` | arquitetura/13-testes §2 | done | tools/ingest/src/aggregate.rs:502 |
| `T-COLETOR-3` | arquitetura/13-testes §2 | done | tools/ingest/src/main.rs:786 |
| `T-COLETOR-30` | arquitetura/13-testes §2 | done | tools/ingest/src/limiter.rs:161 |
| `T-COLETOR-31` | arquitetura/13-testes §2 | done | tools/ingest/src/limiter.rs:169 |
| `T-COLETOR-32` | arquitetura/13-testes §2 | done | tools/ingest/src/limiter.rs:183 |
| `T-COLETOR-33` | arquitetura/13-testes §2 | done | tools/ingest/src/limiter.rs:192 |
| `T-COLETOR-34` | arquitetura/13-testes §2 | done | tools/ingest/src/limiter.rs:216 |
| `T-COLETOR-35` | arquitetura/13-testes §2 | done | tools/ingest/src/limiter.rs:226 |
| `T-COLETOR-36` | arquitetura/13-testes §2 | done | tools/ingest/src/limiter.rs:244 |
| `T-COLETOR-37` | arquitetura/13-testes §2 | done | tools/ingest/src/limiter.rs:274 |
| `T-COLETOR-38` | arquitetura/13-testes §2 | done | tools/ingest/src/main.rs:750 |
| `T-COLETOR-39` | arquitetura/13-testes §2 | done | tools/ingest/src/main.rs:729 |
| `T-COLETOR-4` | arquitetura/13-testes §2 | done | tools/ingest/src/main.rs:804 |
| `T-COLETOR-40` | arquitetura/13-testes §2 | done | tools/ingest/src/main.rs:738 |
| `T-COLETOR-41` | arquitetura/13-testes §2 | done | tools/ingest/src/aggregate.rs:569 |
| `T-COLETOR-5` | arquitetura/13-testes §2 | done | tools/ingest/src/main.rs:827 |
| `T-COLETOR-6` | arquitetura/13-testes §2 | done | tools/ingest/src/main.rs:859 |
| `T-COLETOR-7` | arquitetura/13-testes §2 | done | tools/ingest/src/main.rs:890 |
| `T-COLETOR-8` | arquitetura/13-testes §2 | done | tools/ingest/src/api.rs:286 |
| `T-COLETOR-9` | arquitetura/13-testes §2 | done | tools/ingest/src/api.rs:295 |
| `T-DRAFT-1` | arquitetura/13-testes §2 | done | core/src/engine/mod.rs:781 |
| `T-DRAFT-10` | arquitetura/13-testes §2 | done | core/src/engine/mod.rs:565 |
| `T-DRAFT-11` | arquitetura/13-testes §2 | done | core/src/engine/mod.rs:576 |
| `T-DRAFT-12` | arquitetura/13-testes §2 | done | core/src/engine/mod.rs:622 |
| `T-DRAFT-13` | arquitetura/13-testes §2 | done | core/src/engine/mod.rs:529 |
| `T-DRAFT-14` | arquitetura/13-testes §2 | done | core/src/engine/mod.rs:516 |
| `T-DRAFT-15` | arquitetura/13-testes §2 | done | core/src/engine/mod.rs:523 |
| `T-DRAFT-16` | arquitetura/13-testes §2 | done | core/src/engine/mod.rs:544 |
| `T-DRAFT-17` | arquitetura/13-testes §2 | done | core/src/engine/mod.rs:595 |
| `T-DRAFT-18` | arquitetura/13-testes §2 | done | core/src/engine/mod.rs:920 |
| `T-DRAFT-19` | arquitetura/13-testes §2 | done | core/src/engine/mod.rs:927 |
| `T-DRAFT-2` | arquitetura/13-testes §2 | done | core/src/engine/mod.rs:812 |
| `T-DRAFT-20` | arquitetura/13-testes §2 | done | core/src/engine/mod.rs:560 |
| `T-DRAFT-21` | arquitetura/13-testes §2 | done | core/src/engine/mod.rs:879 |
| `T-DRAFT-22` | arquitetura/13-testes §2 | done | core/src/engine/context.rs:407 |
| `T-DRAFT-23` | arquitetura/13-testes §2 | done | core/src/engine/context.rs:427 |
| `T-DRAFT-24` | arquitetura/13-testes §2 | done | core/src/engine/context.rs:440 |
| `T-DRAFT-25` | arquitetura/13-testes §2 | done | core/src/engine/context.rs:451 |
| `T-DRAFT-26` | arquitetura/13-testes §2 | done | core/src/engine/context.rs:466 |
| `T-DRAFT-27` | arquitetura/13-testes §2 | done | core/src/engine/context.rs:479 |
| `T-DRAFT-28` | arquitetura/13-testes §2 | done | core/src/engine/context.rs:490 |
| `T-DRAFT-29` | arquitetura/13-testes §2 | done | core/src/engine/context.rs:496 |
| `T-DRAFT-3` | arquitetura/13-testes §2 | done | core/src/engine/mod.rs:719 |
| `T-DRAFT-30` | arquitetura/13-testes §2 | done | core/src/engine/context.rs:519 |
| `T-DRAFT-31` | arquitetura/13-testes §2 | done | core/src/engine/context.rs:535 |
| `T-DRAFT-32` | arquitetura/13-testes §2 | done | core/src/engine/context.rs:550 |
| `T-DRAFT-33` | arquitetura/13-testes §2 | done | core/src/engine/context.rs:562 |
| `T-DRAFT-34` | arquitetura/13-testes §2 | done | core/src/engine/context.rs:572 |
| `T-DRAFT-35` | arquitetura/13-testes §2 | done | core/src/engine/context.rs:584 |
| `T-DRAFT-36` | arquitetura/13-testes §2 | done | core/src/engine/context.rs:594 |
| `T-DRAFT-37` | arquitetura/13-testes §2 | done | core/src/engine/roles.rs:131 |
| `T-DRAFT-38` | arquitetura/13-testes §2 | done | core/src/engine/roles.rs:138 |
| `T-DRAFT-39` | arquitetura/13-testes §2 | done | core/src/engine/roles.rs:147 |
| `T-DRAFT-4` | arquitetura/13-testes §2 | done | core/src/engine/mod.rs:737 |
| `T-DRAFT-40` | arquitetura/13-testes §2 | done | core/src/engine/roles.rs:156 |
| `T-DRAFT-41` | arquitetura/13-testes §2 | done | core/src/engine/roles.rs:166 |
| `T-DRAFT-42` | arquitetura/13-testes §2 | done | core/src/engine/roles.rs:173 |
| `T-DRAFT-43` | arquitetura/13-testes §2 | done | core/src/engine/minimax.rs:367 |
| `T-DRAFT-44` | arquitetura/13-testes §2 | done | core/src/engine/minimax.rs:373 |
| `T-DRAFT-45` | arquitetura/13-testes §2 | done | core/src/engine/minimax.rs:405 |
| `T-DRAFT-46` | arquitetura/13-testes §2 | done | core/src/engine/minimax.rs:413 |
| `T-DRAFT-47` | arquitetura/13-testes §2 | done | core/src/engine/minimax.rs:427 |
| `T-DRAFT-48` | arquitetura/13-testes §2 | done | core/src/engine/minimax.rs:437 |
| `T-DRAFT-49` | arquitetura/13-testes §2 | done | core/src/engine/minimax.rs:458 |
| `T-DRAFT-5` | arquitetura/13-testes §2 | done | core/src/engine/mod.rs:751 |
| `T-DRAFT-50` | arquitetura/13-testes §2 | done | core/src/engine/minimax.rs:470 |
| `T-DRAFT-51` | arquitetura/13-testes §2 | done | core/src/engine/minimax.rs:479 |
| `T-DRAFT-52` | arquitetura/13-testes §2 | done | core/src/engine/minimax.rs:536 |
| `T-DRAFT-53` | arquitetura/13-testes §2 | done | core/src/engine/minimax.rs:543 |
| `T-DRAFT-54` | arquitetura/13-testes §2 | done | core/src/engine/minimax.rs:549 |
| `T-DRAFT-55` | arquitetura/13-testes §2 | done | core/src/engine/minimax.rs:558 |
| `T-DRAFT-56` | arquitetura/13-testes §2 | done | core/src/engine/minimax.rs:567 |
| `T-DRAFT-57` | arquitetura/13-testes §2 | done | core/src/engine/minimax.rs:574 |
| `T-DRAFT-58` | arquitetura/13-testes §2 | done | core/src/engine/minimax.rs:598 |
| `T-DRAFT-59` | arquitetura/13-testes §2 | done | core/src/engine/minimax.rs:614 |
| `T-DRAFT-6` | arquitetura/13-testes §2 | done | core/src/engine/mod.rs:767 |
| `T-DRAFT-60` | arquitetura/13-testes §2 | done | core/src/engine/comp.rs:91 |
| `T-DRAFT-61` | arquitetura/13-testes §2 | done | core/src/engine/comp.rs:104 |
| `T-DRAFT-62` | arquitetura/13-testes §2 | done | core/src/engine/comp.rs:120 |
| `T-DRAFT-63` | arquitetura/13-testes §2 | done | core/src/engine/comp.rs:132 |
| `T-DRAFT-64` | arquitetura/13-testes §2 | done | core/src/engine/minimax.rs:519 |
| `T-DRAFT-65` | arquitetura/13-testes §2 | done | core/src/engine/minimax.rs:381 |
| `T-DRAFT-7` | arquitetura/13-testes §2 | done | core/src/engine/mod.rs:842 |
| `T-DRAFT-8` | arquitetura/13-testes §2 | done | core/src/engine/mod.rs:654 |
| `T-DRAFT-9` | arquitetura/13-testes §2 | done | core/src/engine/mod.rs:605 |
| `T-ENTRADA-1` | arquitetura/13-testes §2 | done | core/src/lockfile.rs:50 |
| `T-ENTRADA-10` | arquitetura/13-testes §2 | done | core/src/draft.rs:407 |
| `T-ENTRADA-11` | arquitetura/13-testes §2 | done | core/src/draft.rs:427 |
| `T-ENTRADA-12` | arquitetura/13-testes §2 | done | core/src/draft.rs:444 |
| `T-ENTRADA-13` | arquitetura/13-testes §2 | done | core/src/draft.rs:452 |
| `T-ENTRADA-14` | arquitetura/13-testes §2 | done | core/src/live.rs:253 |
| `T-ENTRADA-15` | arquitetura/13-testes §2 | done | core/src/live.rs:259 |
| `T-ENTRADA-16` | arquitetura/13-testes §2 | done | core/src/live.rs:278 |
| `T-ENTRADA-17` | arquitetura/13-testes §2 | done | core/src/live.rs:292 |
| `T-ENTRADA-18` | arquitetura/13-testes §2 | done | core/src/live.rs:305 |
| `T-ENTRADA-19` | arquitetura/13-testes §2 | done | core/src/live.rs:317 |
| `T-ENTRADA-2` | arquitetura/13-testes §2 | done | core/src/lockfile.rs:57 |
| `T-ENTRADA-20` | arquitetura/13-testes §2 | done | core/src/live.rs:327 |
| `T-ENTRADA-21` | arquitetura/13-testes §2 | done | core/src/history.rs:304 |
| `T-ENTRADA-22` | arquitetura/13-testes §2 | done | core/src/history.rs:314 |
| `T-ENTRADA-23` | arquitetura/13-testes §2 | done | core/src/history.rs:329 |
| `T-ENTRADA-24` | arquitetura/13-testes §2 | done | core/src/history.rs:352 |
| `T-ENTRADA-25` | arquitetura/13-testes §2 | done | core/src/history.rs:359 |
| `T-ENTRADA-26` | arquitetura/13-testes §2 | done | core/src/history.rs:379 |
| `T-ENTRADA-27` | arquitetura/13-testes §2 | done | core/src/history.rs:395 |
| `T-ENTRADA-28` | arquitetura/13-testes §2 | done | core/src/history.rs:403 |
| `T-ENTRADA-29` | arquitetura/13-testes §2 | done | core/src/riot.rs:111 |
| `T-ENTRADA-3` | arquitetura/13-testes §2 | done | core/src/lockfile.rs:67 |
| `T-ENTRADA-30` | arquitetura/13-testes §2 | done | core/src/riot.rs:132 |
| `T-ENTRADA-31` | arquitetura/13-testes §2 | done | core/src/riot.rs:142 |
| `T-ENTRADA-32` | arquitetura/13-testes §2 | done | core/src/riot.rs:160 |
| `T-ENTRADA-33` | arquitetura/13-testes §2 | done | core/src/pool.rs:170 |
| `T-ENTRADA-34` | arquitetura/13-testes §2 | done | core/src/pool.rs:178 |
| `T-ENTRADA-35` | arquitetura/13-testes §2 | done | core/src/pool.rs:190 |
| `T-ENTRADA-36` | arquitetura/13-testes §2 | done | core/src/pool.rs:206 |
| `T-ENTRADA-37` | arquitetura/13-testes §2 | done | core/src/pool.rs:228 |
| `T-ENTRADA-38` | arquitetura/13-testes §2 | done | core/src/pool.rs:239 |
| `T-ENTRADA-39` | arquitetura/13-testes §2 | done | core/src/pool.rs:249 |
| `T-ENTRADA-4` | arquitetura/13-testes §2 | done | core/src/lockfile.rs:83 |
| `T-ENTRADA-40` | arquitetura/13-testes §2 | done | core/src/prefs.rs:142 |
| `T-ENTRADA-41` | arquitetura/13-testes §2 | done | core/src/prefs.rs:151 |
| `T-ENTRADA-42` | arquitetura/13-testes §2 | done | core/src/prefs.rs:182 |
| `T-ENTRADA-43` | arquitetura/13-testes §2 | done | core/src/prefs.rs:193 |
| `T-ENTRADA-44` | arquitetura/13-testes §2 | done | core/src/prefs.rs:209 |
| `T-ENTRADA-45` | arquitetura/13-testes §2 | done | core/src/prefs.rs:255 |
| `T-ENTRADA-46` | arquitetura/13-testes §2 | done | core/src/draft.rs:334 |
| `T-ENTRADA-47` | arquitetura/13-testes §2 | done | core/src/live.rs:221 |
| `T-ENTRADA-48` | arquitetura/13-testes §2 | done | core/src/prefs.rs:220 |
| `T-ENTRADA-49` | arquitetura/13-testes §2 | done | core/src/pool.rs:147 |
| `T-ENTRADA-5` | arquitetura/13-testes §2 | done | core/src/lockfile.rs:88 |
| `T-ENTRADA-50` | arquitetura/13-testes §2 | done | core/src/prefs.rs:170 |
| `T-ENTRADA-6` | arquitetura/13-testes §2 | done | core/src/draft.rs:377 |
| `T-ENTRADA-7` | arquitetura/13-testes §2 | done | core/src/draft.rs:384 |
| `T-ENTRADA-8` | arquitetura/13-testes §2 | done | core/src/draft.rs:394 |
| `T-ENTRADA-9` | arquitetura/13-testes §2 | done | core/src/draft.rs:401 |
| `T-JOGO-1` | arquitetura/13-testes §2 | done | core/src/engine/champion.rs:214 |
| `T-JOGO-10` | arquitetura/13-testes §2 | done | core/src/engine/items.rs:278 |
| `T-JOGO-11` | arquitetura/13-testes §2 | done | core/src/engine/damage.rs:71 |
| `T-JOGO-12` | arquitetura/13-testes §2 | done | core/src/engine/damage.rs:81 |
| `T-JOGO-13` | arquitetura/13-testes §2 | done | core/src/engine/damage.rs:93 |
| `T-JOGO-14` | arquitetura/13-testes §2 | done | core/src/engine/damage.rs:122 |
| `T-JOGO-15` | arquitetura/13-testes §2 | done | core/src/engine/champion.rs:193 |
| `T-JOGO-2` | arquitetura/13-testes §2 | done | core/src/engine/champion.rs:226 |
| `T-JOGO-3` | arquitetura/13-testes §2 | done | core/src/engine/champion.rs:248 |
| `T-JOGO-4` | arquitetura/13-testes §2 | done | core/src/engine/champion.rs:273 |
| `T-JOGO-5` | arquitetura/13-testes §2 | done | core/src/engine/champion.rs:281 |
| `T-JOGO-6` | arquitetura/13-testes §2 | done | core/src/engine/items.rs:206 |
| `T-JOGO-7` | arquitetura/13-testes §2 | done | core/src/engine/items.rs:229 |
| `T-JOGO-8` | arquitetura/13-testes §2 | done | core/src/engine/items.rs:250 |
| `T-JOGO-9` | arquitetura/13-testes §2 | done | core/src/engine/items.rs:270 |
| `T-OBJETIVO-1` | arquitetura/13-testes §2 | done | core/src/engine/timeline.rs:492 |
| `T-OBJETIVO-10` | arquitetura/13-testes §2 | done | core/src/engine/timeline.rs:754 |
| `T-OBJETIVO-11` | arquitetura/13-testes §2 | done | core/src/engine/timeline.rs:792 |
| `T-OBJETIVO-12` | arquitetura/13-testes §2 | done | core/src/engine/timeline.rs:805 |
| `T-OBJETIVO-13` | arquitetura/13-testes §2 | done | core/src/engine/timeline.rs:817 |
| `T-OBJETIVO-14` | arquitetura/13-testes §2 | done | core/src/engine/timeline.rs:839 |
| `T-OBJETIVO-15` | arquitetura/13-testes §2 | done | core/src/engine/timeline.rs:848 |
| `T-OBJETIVO-16` | arquitetura/13-testes §2 | done | core/src/engine/timeline.rs:855 |
| `T-OBJETIVO-17` | arquitetura/13-testes §2 | done | core/src/engine/timeline.rs:669 |
| `T-OBJETIVO-18` | arquitetura/13-testes §2 | done | core/src/engine/timeline.rs:647 |
| `T-OBJETIVO-2` | arquitetura/13-testes §2 | done | core/src/engine/timeline.rs:502 |
| `T-OBJETIVO-3` | arquitetura/13-testes §2 | done | core/src/engine/timeline.rs:525 |
| `T-OBJETIVO-4` | arquitetura/13-testes §2 | done | core/src/engine/timeline.rs:513 |
| `T-OBJETIVO-5` | arquitetura/13-testes §2 | done | core/src/engine/timeline.rs:576 |
| `T-OBJETIVO-6` | arquitetura/13-testes §2 | done | core/src/engine/timeline.rs:557 |
| `T-OBJETIVO-7` | arquitetura/13-testes §2 | done | core/src/engine/timeline.rs:602 |
| `T-OBJETIVO-8` | arquitetura/13-testes §2 | done | core/src/engine/timeline.rs:713 |
| `T-OBJETIVO-9` | arquitetura/13-testes §2 | done | core/src/engine/timeline.rs:740 |
| `T-OURO-1` | arquitetura/13-testes §2 | done | core/src/engine/gold.rs:224 |
| `T-OURO-10` | arquitetura/13-testes §2 | done | core/src/engine/gold.rs:332 |
| `T-OURO-2` | arquitetura/13-testes §2 | done | core/src/engine/gold.rs:322 |
| `T-OURO-3` | arquitetura/13-testes §2 | done | core/src/engine/gold.rs:237 |
| `T-OURO-4` | arquitetura/13-testes §2 | done | core/src/engine/gold.rs:248 |
| `T-OURO-5` | arquitetura/13-testes §2 | done | core/src/engine/gold.rs:264 |
| `T-OURO-6` | arquitetura/13-testes §2 | done | core/src/engine/gold.rs:275 |
| `T-OURO-7` | arquitetura/13-testes §2 | done | core/src/engine/gold.rs:296 |
| `T-OURO-8` | arquitetura/13-testes §2 | done | core/src/engine/gold.rs:312 |
| `T-OURO-9` | arquitetura/13-testes §2 | done | core/src/engine/gold.rs:351 |
| `T-PERFIL-1` | arquitetura/13-testes §2 | done | core/src/engine/profile.rs:375 |
| `T-PERFIL-2` | arquitetura/13-testes §2 | done | core/src/engine/profile.rs:382 |
| `T-PERFIL-3` | arquitetura/13-testes §2 | done | core/src/engine/profile.rs:405 |
| `T-PERFIL-4` | arquitetura/13-testes §2 | done | core/src/engine/profile.rs:421 |
| `T-PERFIL-5` | arquitetura/13-testes §2 | done | core/src/engine/profile.rs:450 |
| `T-PERFIL-6` | arquitetura/13-testes §2 | done | core/src/engine/profile.rs:463 |
| `T-PERFIL-7` | arquitetura/13-testes §2 | done | core/src/engine/profile.rs:483 |
| `T-PERFIL-8` | arquitetura/13-testes §2 | done | core/src/engine/profile.rs:496 |
| `T-PERFIL-9` | arquitetura/13-testes §2 | done | core/src/engine/profile.rs:361 |
| `T-POOL-1` | arquitetura/13-testes §2 | done | core/src/engine/poolgap.rs:494 |
| `T-POOL-10` | arquitetura/13-testes §2 | done | core/src/engine/poolgap.rs:583 |
| `T-POOL-11` | arquitetura/13-testes §2 | done | core/src/engine/poolgap.rs:592 |
| `T-POOL-12` | arquitetura/13-testes §2 | done | core/src/engine/poolgap.rs:601 |
| `T-POOL-13` | arquitetura/13-testes §2 | done | core/src/engine/poolgap.rs:615 |
| `T-POOL-2` | arquitetura/13-testes §2 | done | core/src/engine/poolgap.rs:501 |
| `T-POOL-3` | arquitetura/13-testes §2 | done | core/src/engine/poolgap.rs:517 |
| `T-POOL-4` | arquitetura/13-testes §2 | done | core/src/engine/poolgap.rs:525 |
| `T-POOL-5` | arquitetura/13-testes §2 | done | core/src/engine/poolgap.rs:538 |
| `T-POOL-6` | arquitetura/13-testes §2 | done | core/src/engine/poolgap.rs:548 |
| `T-POOL-7` | arquitetura/13-testes §2 | done | core/src/engine/poolgap.rs:561 |
| `T-POOL-8` | arquitetura/13-testes §2 | done | core/src/engine/poolgap.rs:569 |
| `T-POOL-9` | arquitetura/13-testes §2 | done | core/src/engine/poolgap.rs:575 |
| `T-POS-1` | arquitetura/13-testes §2 | done | core/src/engine/coach.rs:909 |
| `T-POS-10` | arquitetura/13-testes §2 | done | core/src/engine/coach.rs:1100 |
| `T-POS-11` | arquitetura/13-testes §2 | done | core/src/engine/coach.rs:1127 |
| `T-POS-12` | arquitetura/13-testes §2 | done | core/src/engine/coach.rs:1157 |
| `T-POS-13` | arquitetura/13-testes §2 | done | core/src/engine/coach.rs:1163 |
| `T-POS-14` | arquitetura/13-testes §2 | done | core/src/engine/coach.rs:1203 |
| `T-POS-15` | arquitetura/13-testes §2 | done | core/src/engine/coach.rs:1236 |
| `T-POS-16` | arquitetura/13-testes §2 | done | core/src/engine/coach.rs:1255 |
| `T-POS-17` | arquitetura/13-testes §2 | done | core/src/engine/coach.rs:1270 |
| `T-POS-18` | arquitetura/13-testes §2 | done | core/src/engine/coach.rs:1305 |
| `T-POS-19` | arquitetura/13-testes §2 | done | core/src/engine/coach.rs:1316 |
| `T-POS-2` | arquitetura/13-testes §2 | done | core/src/engine/coach.rs:920 |
| `T-POS-20` | arquitetura/13-testes §2 | done | core/src/engine/coach.rs:1333 |
| `T-POS-21` | arquitetura/13-testes §2 | done | core/src/engine/coach.rs:1351 |
| `T-POS-22` | arquitetura/13-testes §2 | done | core/src/engine/coach.rs:1373 |
| `T-POS-23` | arquitetura/13-testes §2 | done | core/src/engine/coach.rs:985 |
| `T-POS-24` | arquitetura/13-testes §2 | done | core/src/engine/coach.rs:962 |
| `T-POS-25` | arquitetura/13-testes §2 | done | core/src/engine/coach.rs:927 |
| `T-POS-3` | arquitetura/13-testes §2 | done | core/src/engine/coach.rs:978 |
| `T-POS-4` | arquitetura/13-testes §2 | done | core/src/engine/coach.rs:1019 |
| `T-POS-5` | arquitetura/13-testes §2 | done | core/src/engine/coach.rs:1054 |
| `T-POS-6` | arquitetura/13-testes §2 | done | core/src/engine/coach.rs:1063 |
| `T-POS-7` | arquitetura/13-testes §2 | done | core/src/engine/coach.rs:1073 |
| `T-POS-8` | arquitetura/13-testes §2 | done | core/src/engine/coach.rs:1081 |
| `T-POS-9` | arquitetura/13-testes §2 | done | core/src/engine/coach.rs:1386 |
| `T-PREVIA-1` | arquitetura/13-testes §2 | done | core/src/engine/team.rs:285 |
| `T-PREVIA-10` | arquitetura/13-testes §2 | done | core/src/engine/wincon.rs:373 |
| `T-PREVIA-11` | arquitetura/13-testes §2 | done | core/src/engine/wincon.rs:388 |
| `T-PREVIA-12` | arquitetura/13-testes §2 | done | core/src/engine/report.rs:139 |
| `T-PREVIA-13` | arquitetura/13-testes §2 | done | core/src/engine/report.rs:149 |
| `T-PREVIA-14` | arquitetura/13-testes §2 | done | core/src/engine/report.rs:161 |
| `T-PREVIA-2` | arquitetura/13-testes §2 | done | core/src/engine/team.rs:292 |
| `T-PREVIA-3` | arquitetura/13-testes §2 | done | core/src/engine/team.rs:302 |
| `T-PREVIA-4` | arquitetura/13-testes §2 | done | core/src/engine/team.rs:311 |
| `T-PREVIA-5` | arquitetura/13-testes §2 | done | core/src/engine/team.rs:321 |
| `T-PREVIA-6` | arquitetura/13-testes §2 | done | core/src/engine/wincon.rs:303 |
| `T-PREVIA-7` | arquitetura/13-testes §2 | done | core/src/engine/wincon.rs:320 |
| `T-PREVIA-8` | arquitetura/13-testes §2 | done | core/src/engine/wincon.rs:329 |
| `T-PREVIA-9` | arquitetura/13-testes §2 | done | core/src/engine/wincon.rs:335 |
| `T-PRIORIDADE-1` | arquitetura/13-testes §2 | done | core/src/engine/priority.rs:337 |
| `T-PRIORIDADE-10` | arquitetura/13-testes §2 | done | core/src/engine/priority.rs:487 |
| `T-PRIORIDADE-11` | arquitetura/13-testes §2 | done | core/src/engine/priority.rs:427 |
| `T-PRIORIDADE-2` | arquitetura/13-testes §2 | done | core/src/engine/priority.rs:347 |
| `T-PRIORIDADE-3` | arquitetura/13-testes §2 | done | core/src/engine/priority.rs:356 |
| `T-PRIORIDADE-4` | arquitetura/13-testes §2 | done | core/src/engine/priority.rs:372 |
| `T-PRIORIDADE-5` | arquitetura/13-testes §2 | done | core/src/engine/priority.rs:387 |
| `T-PRIORIDADE-6` | arquitetura/13-testes §2 | done | core/src/engine/priority.rs:416 |
| `T-PRIORIDADE-7` | arquitetura/13-testes §2 | done | core/src/engine/priority.rs:448 |
| `T-PRIORIDADE-8` | arquitetura/13-testes §2 | done | core/src/engine/priority.rs:468 |
| `T-PRIORIDADE-9` | arquitetura/13-testes §2 | done | core/src/engine/priority.rs:476 |
| `T-RIVAL-1` | arquitetura/13-testes §2 | done | core/src/engine/rival.rs:158 |
| `T-RIVAL-2` | arquitetura/13-testes §2 | done | core/src/engine/rival.rs:176 |
| `T-RIVAL-3` | arquitetura/13-testes §2 | done | core/src/engine/rival.rs:193 |
| `T-RIVAL-4` | arquitetura/13-testes §2 | done | core/src/engine/rival.rs:214 |
| `T-RIVAL-5` | arquitetura/13-testes §2 | done | core/src/engine/rival.rs:250 |
| `T-RIVAL-6` | arquitetura/13-testes §2 | done | core/src/engine/rival.rs:236 |
| `T-ROTA-1` | arquitetura/13-testes §2 | done | core/src/engine/lanes.rs:683 |
| `T-ROTA-10` | arquitetura/13-testes §2 | done | core/src/engine/lanes.rs:883 |
| `T-ROTA-11` | arquitetura/13-testes §2 | done | core/src/engine/lanes.rs:894 |
| `T-ROTA-12` | arquitetura/13-testes §2 | done | core/src/engine/lanes.rs:935 |
| `T-ROTA-13` | arquitetura/13-testes §2 | done | core/src/engine/lanes.rs:973 |
| `T-ROTA-14` | arquitetura/13-testes §2 | done | core/src/engine/lanes.rs:985 |
| `T-ROTA-15` | arquitetura/13-testes §2 | done | core/src/engine/lanes.rs:1017 |
| `T-ROTA-16` | arquitetura/13-testes §2 | done | core/src/engine/lanes.rs:1031 |
| `T-ROTA-17` | arquitetura/13-testes §2 | done | core/src/engine/lanes.rs:1038 |
| `T-ROTA-18` | arquitetura/13-testes §2 | done | core/src/engine/lanes.rs:1043 |
| `T-ROTA-19` | arquitetura/13-testes §2 | done | core/src/engine/lanes.rs:1055 |
| `T-ROTA-2` | arquitetura/13-testes §2 | done | core/src/engine/lanes.rs:694 |
| `T-ROTA-20` | arquitetura/13-testes §2 | done | core/src/engine/lanes.rs:1070 |
| `T-ROTA-21` | arquitetura/13-testes §2 | done | core/src/engine/lanes.rs:1100 |
| `T-ROTA-22` | arquitetura/13-testes §2 | done | core/src/engine/lanes.rs:1134 |
| `T-ROTA-23` | arquitetura/13-testes §2 | done | core/src/engine/lanes.rs:1142 |
| `T-ROTA-24` | arquitetura/13-testes §2 | done | core/src/engine/lanes.rs:1170 |
| `T-ROTA-25` | arquitetura/13-testes §2 | done | core/src/engine/lanes.rs:1195 |
| `T-ROTA-26` | arquitetura/13-testes §2 | done | core/src/engine/lanes.rs:1216 |
| `T-ROTA-3` | arquitetura/13-testes §2 | done | core/src/engine/lanes.rs:711 |
| `T-ROTA-4` | arquitetura/13-testes §2 | done | core/src/engine/lanes.rs:728 |
| `T-ROTA-5` | arquitetura/13-testes §2 | done | core/src/engine/lanes.rs:766 |
| `T-ROTA-6` | arquitetura/13-testes §2 | done | core/src/engine/lanes.rs:807 |
| `T-ROTA-7` | arquitetura/13-testes §2 | done | core/src/engine/lanes.rs:819 |
| `T-ROTA-8` | arquitetura/13-testes §2 | done | core/src/engine/lanes.rs:829 |
| `T-ROTA-9` | arquitetura/13-testes §2 | done | core/src/engine/lanes.rs:872 |
| `T-RUNAS-1` | arquitetura/13-testes §2 | done | core/src/engine/runes.rs:399 |
| `T-RUNAS-10` | arquitetura/13-testes §2 | done | core/src/engine/runes.rs:380 |
| `T-RUNAS-2` | arquitetura/13-testes §2 | done | core/src/engine/runes.rs:411 |
| `T-RUNAS-3` | arquitetura/13-testes §2 | done | core/src/engine/runes.rs:423 |
| `T-RUNAS-4` | arquitetura/13-testes §2 | done | core/src/engine/runes.rs:437 |
| `T-RUNAS-5` | arquitetura/13-testes §2 | done | core/src/engine/runes.rs:461 |
| `T-RUNAS-6` | arquitetura/13-testes §2 | done | core/src/engine/runes.rs:479 |
| `T-RUNAS-7` | arquitetura/13-testes §2 | done | core/src/engine/runes.rs:495 |
| `T-RUNAS-8` | arquitetura/13-testes §2 | done | core/src/engine/runes.rs:521 |
| `T-RUNAS-9` | arquitetura/13-testes §2 | done | core/src/engine/runes.rs:530 |
| `T-STATS-1` | arquitetura/13-testes §2 | done | core/src/engine/stats.rs:309 |
| `T-STATS-10` | arquitetura/13-testes §2 | done | core/src/engine/stats.rs:395 |
| `T-STATS-11` | arquitetura/13-testes §2 | done | core/src/engine/stats.rs:427 |
| `T-STATS-12` | arquitetura/13-testes §2 | done | core/src/engine/stats.rs:452 |
| `T-STATS-13` | arquitetura/13-testes §2 | done | core/src/engine/stats.rs:461 |
| `T-STATS-14` | arquitetura/13-testes §2 | done | core/src/engine/dataset.rs:246 |
| `T-STATS-15` | arquitetura/13-testes §2 | done | core/src/engine/dataset.rs:272 |
| `T-STATS-16` | arquitetura/13-testes §2 | done | core/src/engine/dataset.rs:279 |
| `T-STATS-17` | arquitetura/13-testes §2 | done | core/src/engine/dataset.rs:288 |
| `T-STATS-18` | arquitetura/13-testes §2 | done | core/src/engine/dataset.rs:328 |
| `T-STATS-19` | arquitetura/13-testes §2 | done | core/src/engine/dataset.rs:263 |
| `T-STATS-2` | arquitetura/13-testes §2 | done | core/src/engine/stats.rs:322 |
| `T-STATS-20` | arquitetura/13-testes §2 | done | core/src/engine/dataset.rs:218 |
| `T-STATS-3` | arquitetura/13-testes §2 | done | core/src/engine/stats.rs:335 |
| `T-STATS-4` | arquitetura/13-testes §2 | done | core/src/engine/stats.rs:342 |
| `T-STATS-5` | arquitetura/13-testes §2 | done | core/src/engine/stats.rs:357 |
| `T-STATS-6` | arquitetura/13-testes §2 | done | core/src/engine/stats.rs:367 |
| `T-STATS-7` | arquitetura/13-testes §2 | done | core/src/engine/stats.rs:375 |
| `T-STATS-8` | arquitetura/13-testes §2 | done | core/src/engine/stats.rs:388 |
| `T-STATS-9` | arquitetura/13-testes §2 | done | core/src/engine/stats.rs:415 |

## ui-test

| Item | Fonte | Estado | Onde / motivo |
|---|---|---|---|
| `TF-AJUSTES-1` | frontend/06-testes §3 | deferred | P-16: teste do front ainda não escrito |
| `TF-AJUSTES-2` | frontend/06-testes §3 | deferred | P-16: teste do front ainda não escrito |
| `TF-CASCA-1` | frontend/06-testes §3 | deferred | P-16: teste do front ainda não escrito |
| `TF-CASCA-2` | frontend/06-testes §3 | deferred | P-16: teste do front ainda não escrito |
| `TF-CASCA-3` | frontend/06-testes §3 | deferred | P-16: teste do front ainda não escrito |
| `TF-CASCA-6` | frontend/06-testes §3 | deferred | P-16: teste do front ainda não escrito |
| `TF-DRAFT-1` | frontend/06-testes §3 | deferred | P-16: teste do front ainda não escrito |
| `TF-DRAFT-2` | frontend/06-testes §3 | deferred | P-16: teste do front ainda não escrito |
| `TF-DRAFT-3` | frontend/06-testes §3 | deferred | P-16: teste do front ainda não escrito |
| `TF-DRAFT-4` | frontend/06-testes §3 | deferred | P-16: teste do front ainda não escrito |
| `TF-DRAFT-5` | frontend/06-testes §3 | deferred | P-16: teste do front ainda não escrito |
| `TF-DRAFT-6` | frontend/06-testes §3 | deferred | P-16: teste do front ainda não escrito |
| `TF-DRAFT-8` | frontend/06-testes §3 | deferred | P-16: teste do front ainda não escrito |
| `TF-DRAFT-9` | frontend/06-testes §3 | deferred | P-16: teste do front ainda não escrito |
| `TF-OVERLAY-1` | frontend/06-testes §3 | deferred | P-16: teste do front ainda não escrito |
| `TF-OVERLAY-2` | frontend/06-testes §3 | deferred | P-16: teste do front ainda não escrito |
| `TF-OVERLAY-3` | frontend/06-testes §3 | deferred | P-16: teste do front ainda não escrito |
| `TF-OVERLAY-4` | frontend/06-testes §3 | deferred | P-16: teste do front ainda não escrito |
| `TF-OVERLAY-5` | frontend/06-testes §3 | deferred | P-16: teste do front ainda não escrito |
| `TF-PARTIDA-1` | frontend/06-testes §3 | deferred | P-16: teste do front ainda não escrito |
| `TF-PARTIDA-2` | frontend/06-testes §3 | deferred | P-16: teste do front ainda não escrito |
| `TF-PARTIDA-4` | frontend/06-testes §3 | deferred | P-16: teste do front ainda não escrito |
| `TF-POOL-1` | frontend/06-testes §3 | deferred | P-16: teste do front ainda não escrito |
| `TF-POOL-2` | frontend/06-testes §3 | deferred | P-16: teste do front ainda não escrito |
| `TF-POOL-3` | frontend/06-testes §3 | deferred | P-16: teste do front ainda não escrito |
| `TF-POOL-5` | frontend/06-testes §3 | deferred | P-16: teste do front ainda não escrito |
| `TF-POS-1` | frontend/06-testes §3 | deferred | P-16: teste do front ainda não escrito |
| `TF-POS-2` | frontend/06-testes §3 | deferred | P-16: teste do front ainda não escrito |
| `TF-POS-3` | frontend/06-testes §3 | deferred | P-16: teste do front ainda não escrito |
| `TF-ROTAS-1` | frontend/06-testes §3 | deferred | P-16: teste do front ainda não escrito |
| `TF-SIM-1` | frontend/06-testes §3 | deferred | P-16: teste do front ainda não escrito |
| `TF-SIM-2` | frontend/06-testes §3 | deferred | P-16: teste do front ainda não escrito |
| `TF-SIM-3` | frontend/06-testes §3 | deferred | P-16: teste do front ainda não escrito |
| `TF-CASCA-4` | frontend/06-testes §3 | done | src/App.test.tsx:164 |
| `TF-CASCA-5` | frontend/06-testes §3 | done | src/estatico.test.ts:22 |
| `TF-CASCA-7` | frontend/06-testes §3 | done | src/estatico.test.ts:47 |
| `TF-DRAFT-10` | frontend/06-testes §3 | done | src/App.test.tsx:221 |
| `TF-DRAFT-11` | frontend/06-testes §3 | done | src/App.test.tsx:261 |
| `TF-DRAFT-12` | frontend/06-testes §3 | done | src/components/RecommendationPanel.test.tsx:41 |
| `TF-DRAFT-13` | frontend/06-testes §3 | done | src/components/RecommendationPanel.test.tsx:69 |
| `TF-DRAFT-14` | frontend/06-testes §3 | done | src/components/RecommendationPanel.test.tsx:92 |
| `TF-DRAFT-7` | frontend/06-testes §3 | done | src/App.test.tsx:243 |
| `TF-OVERLAY-6` | frontend/06-testes §3 | done | src/estatico.test.ts:35 |
| `TF-PARTIDA-3` | frontend/06-testes §3 | done | src/components/GoldTracker.test.tsx:40 |
| `TF-PARTIDA-5` | frontend/06-testes §3 | done | src/components/LiveBuild.test.tsx:14 |
| `TF-POOL-4` | frontend/06-testes §3 | done | src/App.test.tsx:137 |
| `TF-ROTAS-2` | frontend/06-testes §3 | done | src/components/Runas.test.tsx:16 |
| `TF-ROTAS-3` | frontend/06-testes §3 | done | src/App.test.tsx:189 |

## trace

| Item | Fonte | Estado | Onde / motivo |
|---|---|---|---|
| `C-10` | produto/requisitos matriz | deferred | regra de distribuição: chave de produção antes de distribuir (06, ADR-0004) |
| `C-8` | produto/requisitos matriz | deferred | regra de cobrança, não de código: conferida antes de cobrar (P-2, 00 §4) |
| `RNF-10` | produto/requisitos matriz | deferred | provado pelo build do instalador: CRT estática (ADR-0014, .cargo/config.toml) |
| `RNF-9` | produto/requisitos matriz | deferred | provado pelo próprio gate: cargo test -p presage-core roda sem MSVC e sem Tauri (ADR-0001) |
| `C-1` | produto/requisitos matriz | done | docs/produto/requisitos.md:291 |
| `C-2` | produto/requisitos matriz | done | docs/produto/requisitos.md:292 |
| `C-3` | produto/requisitos matriz | done | docs/produto/requisitos.md:293 |
| `C-4` | produto/requisitos matriz | done | docs/produto/requisitos.md:294 |
| `C-5` | produto/requisitos matriz | done | docs/produto/requisitos.md:295 |
| `C-6` | produto/requisitos matriz | done | docs/produto/requisitos.md:296 |
| `C-7` | produto/requisitos matriz | done | docs/produto/requisitos.md:297 |
| `C-9` | produto/requisitos matriz | done | docs/produto/requisitos.md:299 |
| `R-1` | produto/requisitos matriz | done | docs/produto/requisitos.md:270 |
| `R-10` | produto/requisitos matriz | done | docs/produto/requisitos.md:279 |
| `R-11` | produto/requisitos matriz | done | docs/produto/requisitos.md:280 |
| `R-12` | produto/requisitos matriz | done | docs/produto/requisitos.md:281 |
| `R-13` | produto/requisitos matriz | done | docs/produto/requisitos.md:282 |
| `R-14` | produto/requisitos matriz | done | docs/produto/requisitos.md:283 |
| `R-15` | produto/requisitos matriz | done | docs/produto/requisitos.md:284 |
| `R-16` | produto/requisitos matriz | done | docs/produto/requisitos.md:285 |
| `R-17` | produto/requisitos matriz | done | docs/produto/requisitos.md:286 |
| `R-18` | produto/requisitos matriz | done | docs/produto/requisitos.md:287 |
| `R-19` | produto/requisitos matriz | done | docs/produto/requisitos.md:288 |
| `R-2` | produto/requisitos matriz | done | docs/produto/requisitos.md:271 |
| `R-20` | produto/requisitos matriz | done | docs/produto/requisitos.md:289 |
| `R-21` | produto/requisitos matriz | done | docs/produto/requisitos.md:290 |
| `R-3` | produto/requisitos matriz | done | docs/produto/requisitos.md:272 |
| `R-4` | produto/requisitos matriz | done | docs/produto/requisitos.md:273 |
| `R-5` | produto/requisitos matriz | done | docs/produto/requisitos.md:274 |
| `R-6` | produto/requisitos matriz | done | docs/produto/requisitos.md:275 |
| `R-7` | produto/requisitos matriz | done | docs/produto/requisitos.md:276 |
| `R-8` | produto/requisitos matriz | done | docs/produto/requisitos.md:277 |
| `R-9` | produto/requisitos matriz | done | docs/produto/requisitos.md:278 |
| `RNF-1` | produto/requisitos matriz | done | docs/produto/requisitos.md:301 |
| `RNF-11` | produto/requisitos matriz | done | docs/produto/requisitos.md:311 |
| `RNF-12` | produto/requisitos matriz | done | docs/produto/requisitos.md:312 |
| `RNF-2` | produto/requisitos matriz | done | docs/produto/requisitos.md:302 |
| `RNF-3` | produto/requisitos matriz | done | docs/produto/requisitos.md:303 |
| `RNF-4` | produto/requisitos matriz | done | docs/produto/requisitos.md:304 |
| `RNF-5` | produto/requisitos matriz | done | docs/produto/requisitos.md:305 |
| `RNF-6` | produto/requisitos matriz | done | docs/produto/requisitos.md:306 |
| `RNF-7` | produto/requisitos matriz | done | docs/produto/requisitos.md:307 |
| `RNF-8` | produto/requisitos matriz | done | docs/produto/requisitos.md:308 |

## itero

| Item | Fonte | Estado | Onde / motivo |
|---|---|---|---|
| `I-1` | referencia/itero/07-cruzamento §1 | done | docs/referencia/itero/07-cruzamento.md:38 |
| `I-10` | referencia/itero/07-cruzamento §1 | done | docs/referencia/itero/07-cruzamento.md:47 |
| `I-11` | referencia/itero/07-cruzamento §1 | done | docs/referencia/itero/07-cruzamento.md:48 |
| `I-12` | referencia/itero/07-cruzamento §1 | done | docs/referencia/itero/07-cruzamento.md:49 |
| `I-13` | referencia/itero/07-cruzamento §1 | done | docs/referencia/itero/07-cruzamento.md:50 |
| `I-14` | referencia/itero/07-cruzamento §1 | done | docs/referencia/itero/07-cruzamento.md:51 |
| `I-15` | referencia/itero/07-cruzamento §1 | done | docs/referencia/itero/07-cruzamento.md:52 |
| `I-16` | referencia/itero/07-cruzamento §1 | done | docs/referencia/itero/07-cruzamento.md:53 |
| `I-17` | referencia/itero/07-cruzamento §1 | done | docs/referencia/itero/07-cruzamento.md:54 |
| `I-18` | referencia/itero/07-cruzamento §1 | done | docs/referencia/itero/07-cruzamento.md:55 |
| `I-19` | referencia/itero/07-cruzamento §1 | done | docs/referencia/itero/07-cruzamento.md:56 |
| `I-2` | referencia/itero/07-cruzamento §1 | done | docs/referencia/itero/07-cruzamento.md:39 |
| `I-20` | referencia/itero/07-cruzamento §1 | done | docs/referencia/itero/07-cruzamento.md:57 |
| `I-21` | referencia/itero/07-cruzamento §1 | done | docs/referencia/itero/07-cruzamento.md:58 |
| `I-22` | referencia/itero/07-cruzamento §1 | done | docs/referencia/itero/07-cruzamento.md:59 |
| `I-23` | referencia/itero/07-cruzamento §1 | done | docs/referencia/itero/07-cruzamento.md:60 |
| `I-24` | referencia/itero/07-cruzamento §1 | done | docs/referencia/itero/07-cruzamento.md:61 |
| `I-25` | referencia/itero/07-cruzamento §1 | done | docs/referencia/itero/07-cruzamento.md:62 |
| `I-26` | referencia/itero/07-cruzamento §1 | done | docs/referencia/itero/07-cruzamento.md:63 |
| `I-27` | referencia/itero/07-cruzamento §1 | done | docs/referencia/itero/07-cruzamento.md:64 |
| `I-28` | referencia/itero/07-cruzamento §1 | done | docs/referencia/itero/07-cruzamento.md:65 |
| `I-29` | referencia/itero/07-cruzamento §1 | done | docs/referencia/itero/07-cruzamento.md:66 |
| `I-3` | referencia/itero/07-cruzamento §1 | done | docs/referencia/itero/07-cruzamento.md:40 |
| `I-30` | referencia/itero/07-cruzamento §1 | done | docs/referencia/itero/07-cruzamento.md:67 |
| `I-31` | referencia/itero/07-cruzamento §2 | done | docs/referencia/itero/07-cruzamento.md:77 |
| `I-32` | referencia/itero/07-cruzamento §2 | done | docs/referencia/itero/07-cruzamento.md:78 |
| `I-33` | referencia/itero/07-cruzamento §2 | done | docs/referencia/itero/07-cruzamento.md:79 |
| `I-34` | referencia/itero/07-cruzamento §2 | done | docs/referencia/itero/07-cruzamento.md:80 |
| `I-35` | referencia/itero/07-cruzamento §2 | done | docs/referencia/itero/07-cruzamento.md:81 |
| `I-36` | referencia/itero/07-cruzamento §2 | done | docs/referencia/itero/07-cruzamento.md:82 |
| `I-37` | referencia/itero/07-cruzamento §2 | done | docs/referencia/itero/07-cruzamento.md:83 |
| `I-38` | referencia/itero/07-cruzamento §2 | done | docs/referencia/itero/07-cruzamento.md:84 |
| `I-39` | referencia/itero/07-cruzamento §2 | done | docs/referencia/itero/07-cruzamento.md:85 |
| `I-4` | referencia/itero/07-cruzamento §1 | done | docs/referencia/itero/07-cruzamento.md:41 |
| `I-40` | referencia/itero/07-cruzamento §2 | done | docs/referencia/itero/07-cruzamento.md:86 |
| `I-41` | referencia/itero/07-cruzamento §2 | done | docs/referencia/itero/07-cruzamento.md:87 |
| `I-42` | referencia/itero/07-cruzamento §2 | done | docs/referencia/itero/07-cruzamento.md:88 |
| `I-43` | referencia/itero/07-cruzamento §2 | done | docs/referencia/itero/07-cruzamento.md:89 |
| `I-44` | referencia/itero/07-cruzamento §2 | done | docs/referencia/itero/07-cruzamento.md:90 |
| `I-45` | referencia/itero/07-cruzamento §2 | done | docs/referencia/itero/07-cruzamento.md:91 |
| `I-46` | referencia/itero/07-cruzamento §2 | done | docs/referencia/itero/07-cruzamento.md:92 |
| `I-47` | referencia/itero/07-cruzamento §2 | done | docs/referencia/itero/07-cruzamento.md:93 |
| `I-48` | referencia/itero/07-cruzamento §3 | done | docs/referencia/itero/07-cruzamento.md:101 |
| `I-49` | referencia/itero/07-cruzamento §3 | done | docs/referencia/itero/07-cruzamento.md:102 |
| `I-5` | referencia/itero/07-cruzamento §1 | done | docs/referencia/itero/07-cruzamento.md:42 |
| `I-50` | referencia/itero/07-cruzamento §3 | done | docs/referencia/itero/07-cruzamento.md:103 |
| `I-51` | referencia/itero/07-cruzamento §3 | done | docs/referencia/itero/07-cruzamento.md:104 |
| `I-52` | referencia/itero/07-cruzamento §3 | done | docs/referencia/itero/07-cruzamento.md:105 |
| `I-53` | referencia/itero/07-cruzamento §3 | done | docs/referencia/itero/07-cruzamento.md:106 |
| `I-54` | referencia/itero/07-cruzamento §3 | done | docs/referencia/itero/07-cruzamento.md:107 |
| `I-55` | referencia/itero/07-cruzamento §3 | done | docs/referencia/itero/07-cruzamento.md:108 |
| `I-56` | referencia/itero/07-cruzamento §3 | done | docs/referencia/itero/07-cruzamento.md:109 |
| `I-57` | referencia/itero/07-cruzamento §3 | done | docs/referencia/itero/07-cruzamento.md:110 |
| `I-58` | referencia/itero/07-cruzamento §3 | done | docs/referencia/itero/07-cruzamento.md:111 |
| `I-59` | referencia/itero/07-cruzamento §4 | done | docs/referencia/itero/07-cruzamento.md:122 |
| `I-6` | referencia/itero/07-cruzamento §1 | done | docs/referencia/itero/07-cruzamento.md:43 |
| `I-60` | referencia/itero/07-cruzamento §4 | done | docs/referencia/itero/07-cruzamento.md:123 |
| `I-61` | referencia/itero/07-cruzamento §4 | done | docs/referencia/itero/07-cruzamento.md:124 |
| `I-62` | referencia/itero/07-cruzamento §4 | done | docs/referencia/itero/07-cruzamento.md:125 |
| `I-63` | referencia/itero/07-cruzamento §4 | done | docs/referencia/itero/07-cruzamento.md:126 |
| `I-64` | referencia/itero/07-cruzamento §4 | done | docs/referencia/itero/07-cruzamento.md:127 |
| `I-65` | referencia/itero/07-cruzamento §4 | done | docs/referencia/itero/07-cruzamento.md:128 |
| `I-66` | referencia/itero/07-cruzamento §1 | done | docs/referencia/itero/07-cruzamento.md:68 |
| `I-67` | referencia/itero/07-cruzamento §1 | done | docs/referencia/itero/07-cruzamento.md:69 |
| `I-68` | referencia/itero/07-cruzamento §1 | done | docs/referencia/itero/07-cruzamento.md:70 |
| `I-69` | referencia/itero/07-cruzamento §2 | done | docs/referencia/itero/07-cruzamento.md:94 |
| `I-7` | referencia/itero/07-cruzamento §1 | done | docs/referencia/itero/07-cruzamento.md:44 |
| `I-70` | referencia/itero/07-cruzamento §2 | done | docs/referencia/itero/07-cruzamento.md:95 |
| `I-71` | referencia/itero/07-cruzamento §3 | done | docs/referencia/itero/07-cruzamento.md:112 |
| `I-72` | referencia/itero/07-cruzamento §3 | done | docs/referencia/itero/07-cruzamento.md:113 |
| `I-73` | referencia/itero/07-cruzamento §3 | done | docs/referencia/itero/07-cruzamento.md:114 |
| `I-74` | referencia/itero/07-cruzamento §3 | done | docs/referencia/itero/07-cruzamento.md:115 |
| `I-75` | referencia/itero/07-cruzamento §3 | done | docs/referencia/itero/07-cruzamento.md:116 |
| `I-76` | referencia/itero/07-cruzamento §4 | done | docs/referencia/itero/07-cruzamento.md:129 |
| `I-77` | referencia/itero/07-cruzamento §4 | done | docs/referencia/itero/07-cruzamento.md:130 |
| `I-78` | referencia/itero/07-cruzamento §4 | done | docs/referencia/itero/07-cruzamento.md:131 |
| `I-79` | referencia/itero/07-cruzamento §4 | done | docs/referencia/itero/07-cruzamento.md:132 |
| `I-8` | referencia/itero/07-cruzamento §1 | done | docs/referencia/itero/07-cruzamento.md:45 |
| `I-80` | referencia/itero/07-cruzamento §4 | done | docs/referencia/itero/07-cruzamento.md:133 |
| `I-81` | referencia/itero/07-cruzamento §4 | done | docs/referencia/itero/07-cruzamento.md:134 |
| `I-82` | referencia/itero/07-cruzamento §4 | done | docs/referencia/itero/07-cruzamento.md:135 |
| `I-83` | referencia/itero/07-cruzamento §1 | done | docs/referencia/itero/07-cruzamento.md:71 |
| `I-9` | referencia/itero/07-cruzamento §1 | done | docs/referencia/itero/07-cruzamento.md:46 |

## todo

| Item | Fonte | Estado | Onde / motivo |
|---|---|---|---|
| `src-tauri/src/lcu/client.rs TODO(segurança)` | src-tauri/src/lcu/client.rs:28 | deferred | P-8: pinning do root CA da Riot antes do release público |
| `src-tauri/src/watcher.rs TODO(fase 1)` | src-tauri/src/watcher.rs:7 | deferred | ADR-0002: polling até reagirmos a hover de aliado |

## Auditoria da prosa

| Aberto | Feito | Adiado | Doc |
|---|---|---|---|
| 21 | 42 | 0 | 38 |
