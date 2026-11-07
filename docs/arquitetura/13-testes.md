# 13 — Testes

O catálogo de todo teste do Rust, por área. Cada linha tem um `T-ÁREA-n`, nomeia
as funções de teste que a provam e diz o que elas provam e o que atendem
([ADR-0021](../adr/0021-o-id-do-teste-nomeia-a-funcao.md)).

O `doccheck` exige duas coisas:

- todo teste nomeado aqui existe;
- todo `#[test]` do código está em exatamente uma linha.

A [matriz de rastreabilidade](../produto/requisitos.md#4-matriz-de-rastreabilidade)
sai da coluna **Atende**. Os testes do front estão no
[catálogo do front](../frontend/06-testes.md); hoje são todos pendentes
([P-16](../produto/perguntas-em-aberto.md)).

## 1. Estratégia

| Camada | O que prova | Onde | Ferramenta |
|---|---|---|---|
| Unidade do core | a conta, com números conhecidos, não sorteados | `core/src/**` (`mod tests`) | `cargo test -p presage-core`, em qualquer máquina |
| Fronteira | os nomes de campo que assumimos da Riot (LCU, Live Client, Web API) | os tradutores de JSON cru | idem ([P-5](../produto/perguntas-em-aberto.md)) |
| Regressão | um defeito que já aconteceu não volta | o teste citado no L-n ([09](09-licoes-do-prototipo.md)) | idem |
| Limitação | uma limitação conhecida continua conhecida: o teste falha quando ela acabar | p. ex. o ban não pessoal ([ADR-0013](../adr/0013-ban-nao-personalizado-enquanto-aditivo.md)) | idem |
| Casca | o que só a casca faz: credenciais, gravação, montagem da análise | `src-tauri/src/**` | `cargo test --workspace`, só no Windows (MSVC) |
| Coletor | crawl, limite de taxa, agregação | `tools/ingest/src/**` | `cargo test -p presage-ingest` |
| Docs ↔ código | este catálogo, o 10, o 11, o 12 e a matriz batem com o código | `tools/doccheck` | `npm run doccheck -- -strict` e `node --test tools/doccheck/doccheck.test.mjs` |

Como os testes são escritos (nomes como frases, números conhecidos, teste com
dado e não só sem dado, teste primeiro contra o código velho) está em
[08 §4](08-desenvolvimento-e-testes.md#como-os-testes-são-escritos).

## 2. Catálogo

| Tipo | Quer dizer |
|---|---|
| unidade | uma função, com entrada e saída conhecidas |
| propriedade | uma relação que vale para qualquer entrada do caso (simetria, monotonia, soma 1) |
| fronteira | JSON cru com os nomes de campo de fora |
| regressão | trava um L-n |
| limitação | trava uma limitação e deve falhar quando ela acabar |
| integração | mais de um módulo, ou a casca com o core |

### 2.1 Entrada — credenciais, champ select, Live Client, match-v5, região, pool e preferências

| ID | Teste(s) | Tipo | O que prova | Atende |
|---|---|---|---|---|
| T-ENTRADA-1 | `lockfile_valido` | unidade | `LeagueClient:21360:54321:aBcD_eFg123:https` dá porta 54321 e senha `aBcD_eFg123`: o pid (2º campo) não vira porta | R-18 |
| T-ENTRADA-2 | `lockfile_truncado_nao_entra_em_panico`, `lockfile_com_porta_invalida` | unidade | `LeagueClient:21360` (dois campos) e `LeagueClient:21360:abc:senha:https` (porta `abc`) dão `None`, sem pânico | R-18 |
| T-ENTRADA-3 | `cmdline_com_aspas`, `cmdline_sem_aspas` | unidade | `"--app-port=50123" "--remoting-auth-token=AbC_dEf-123"` e a mesma linha sem aspas dão porta 50123; sem aspas o token para no espaço (`AbC_dEf`) e não engole o `--locale=pt_BR` seguinte | R-18 |
| T-ENTRADA-4 | `cmdline_sem_token_falha` | unidade | Só `--app-port=50123`, sem `--remoting-auth-token=`, dá `None` em vez de credencial com senha vazia | R-18 |
| T-ENTRADA-5 | `base_url_usa_loopback` | unidade | Porta 1234 dá `https://127.0.0.1:1234` | R-18 |
| T-ENTRADA-6 | `separa_bans_por_time` | unidade | Ban 266 com `is_ally_action` e ban 103 sem dão `my_bans == [266]` e `their_bans == [103]` | R-1, R-3 |
| T-ENTRADA-7 | `pick_inimigo_vem_da_acao_nao_do_jogador` | regressão | Com `championId` 0 no jogador da célula 5 e um `pick` 64 concluído por ela, o slot sai com `champion_id == 64` e `locked` | R-1, L-10 |
| T-ENTRADA-8 | `detecta_a_vez_do_jogador_local` | unidade | Um `pick` em andamento da célula local (0) dá `is_local_turn` e `current_action == Some("pick")` | R-1 |
| T-ENTRADA-9 | `indisponiveis_sem_duplicatas_e_sem_zeros` | unidade | Bans 266 e 103, pick 64 concluído e um pick em andamento com campeão 0 dão `unavailable == [64, 103, 266]`: ordenado, e o 0 fica de fora | R-1, L-26 |
| T-ENTRADA-10 | `sessao_vazia_nao_entra_em_panico` | unidade | `LcuSession::default()` dá time vazio, `is_local_turn == false` e `current_action == None` | R-18 |
| T-ENTRADA-11 | `na_ranqueada_a_rotacao_gratis_nao_vale`, `fora_da_ranqueada_a_rotacao_vale`, `fila_desconhecida_fica_do_lado_seguro` | fronteira | Grade lida de JSON com `owned`, `freeToPlay` e `xboxGPReward` (1 próprio, 2 grátis, 3 Game Pass, 4 nenhum): fila 420 dá {1, 3}; fila 400 dá {1, 2, 3}; fila `None` dá {1, 3} | R-5 |
| T-ENTRADA-12 | `a_fila_sai_da_sessao_do_fluxo_de_jogo` | fronteira | `{"phase": "ChampSelect", "gameData": {"queue": {"id": 420, "type": "RANKED_SOLO_5x5"}}}` desserializa, campos a mais ignorados, e dá `game_data.queue.id == 420` | R-5 |
| T-ENTRADA-13 | `draft_montado_a_mao_ganha_indisponiveis_e_rota` | unidade | JSON com só times e bans (`isLocal` na célula 0 `middle`, 86 e 103 travados, bans `[64]` e `[0]`), depois de `normalized`, dá `unavailable == [64, 86, 103]`, `local_position == "middle"` e `local_cell_id == 0` | R-8 |
| T-ENTRADA-14 | `chave_do_campeao_ignora_o_prefixo` | unidade | `game_character_displayname_Annie` e `Annie` dão os dois `Annie` | R-11 |
| T-ENTRADA-15 | `identifica_o_jogador_local_pelo_riot_id`, `identifica_pelo_nome_de_invocador_quando_nao_ha_riot_id` | unidade | `active == "eu#BR1"` casa com o `riotId`, e o local é a Annie com um inimigo; `active == "eu"` casa com o `summonerName` | R-11, R-12 |
| T-ENTRADA-16 | `mapeia_a_chave_para_o_id_do_campeao`, `campeao_desconhecido_vira_zero_sem_quebrar` | unidade | `LeeSin` dá `champion_id == 64` pelo índice; `Inexistente` dá 0 sem erro | R-11 |
| T-ENTRADA-17 | `ouro_investido_e_soma_exata_dos_precos` | unidade | Itens de 1100 e 3000 no seu lado e um de 3000 no deles dão `gold_invested() == (4100.0, 3000.0)` | R-12, C-4 |
| T-ENTRADA-18 | `bots_ficam_de_fora` | unidade | Dois jogadores crus, um com `is_bot`, dão `players.len() == 1` | R-11 |
| T-ENTRADA-19 | `sem_jogador_local_as_listas_ficam_vazias_em_vez_de_mentir` | unidade | `active == "ninguem#BR1"` sem par: `me()` é `None`, e `allies()` e `enemies()` vêm vazios em vez de tomar um lado | R-12, RNF-1 |
| T-ENTRADA-20 | `vira_combatente_para_o_motor_de_build` | unidade | Annie nível 11 com o item 3135 vira `Combatant { champion_id: 1, level: 11, items: [3135] }` | R-10 |
| T-ENTRADA-21 | `oponente_e_a_mesma_rota_do_outro_time` | unidade | Você no `MIDDLE` do time 100 com o campeão 12: `won`, oponente 22 (o `MIDDLE` do 200) e `role == Mid` | R-17, ADR-0010 |
| T-ENTRADA-22 | `diferenca_e_voce_menos_ele_na_unidade_da_tela` | unidade | Tropas 82 contra 70, participação 0,62 contra 0,5 e mortes 1 contra 3 dão `Cs10 == 12`, `KillParticipation == 12` (pp) e `Deaths == -2` | R-17 |
| T-ENTRADA-23 | `cacador_compara_a_selva`, `suporte_nao_compara_tropa` | unidade | No `JUNGLE`, campos da selva 58 contra 50 dão `Cs10 == 8` (as tropas de rota, iguais, não entram); no `UTILITY` não há `Cs10` e há `VisionPerMinute` | R-17 |
| T-ENTRADA-24 | `remake_nao_conta_nem_para_vitoria` | regressão | Um inimigo com `gameEndedInEarlySurrender` dá `Err(Skip::Remake)` para você, que venceu | R-17, L-5 |
| T-ENTRADA-25 | `partida_curta_terminada_ou_fora_da_fenda_fica_de_fora`, `jogador_fora_da_partida` | unidade | 200 s dá `Curta`; `Abort_Unexpected` dá `Terminada`; `ARAM` dá `Modo`; um puuid que não jogou dá `Ausente` | R-17, R-20, L-5 |
| T-ENTRADA-26 | `rota_em_branco_conta_vitoria_sem_comparar_rota`, `oponente_ambiguo_nao_vira_comparacao` | unidade | `teamPosition` vazio dá `Ok` com campeão 12 e `lane == None`; dois `MIDDLE` no time inimigo também dão `lane == None` | R-17 |
| T-ENTRADA-27 | `sem_challenges_sobra_so_mortes` | unidade | `challenges == None` no seu lado deixa `diffs` só com `Deaths` | R-17 |
| T-ENTRADA-28 | `le_os_nomes_de_campo_da_riot` | fronteira | JSON da match-v5 com `gameDuration`, `teamPosition`, `laneMinionsFirst10Minutes`, `killParticipation`, `teamDamagePercentage`, `visionScorePerMinute`: 81 contra 74 tropas dá `Cs10 == 7`, 0,64 contra 0,5 dá 14pp, 0,31 contra 0,25 dá 6pp, 0,9 contra 0,7 dá 0,2 e 2 contra 6 mortes dá −4. `jungleCsBefore10Minutes` não está no JSON | R-17, P-5 |
| T-ENTRADA-29 | `riot_id_com_espaco`, `riot_id_com_cerquilha_no_nome_usa_a_ultima`, `riot_id_invalido` | unidade | `crit is back#split` dá (`crit is back`, `split`); `a#b#tag` dá (`a#b`, `tag`); `semtag`, `#tag` e `nome#` dão `None` | R-17 |
| T-ENTRADA-30 | `espaco_vira_percent_20`, `acento_vira_utf8_percent` | unidade | `crit is back` dá `crit%20is%20back`; `ç` dá `%C3%A7`, os dois bytes do UTF-8 | R-17 |
| T-ENTRADA-31 | `regiao_desconhecida_nao_vira_brasil`, `regioes_conhecidas` | regressão | `XX` e a região vazia dão `None`, e `ME` dá `me1`; `br` dá `br1`, `KR` dá `asia`, `EUW` dá `europe` e `oce` dá `oc1`. Antes, `XX` caía em `br1` calado, e o teste travava isso | R-17, A17 |
| T-ENTRADA-32 | `plataforma_resolve_o_host_regional`, `plataforma_desconhecida_nao_cai_num_padrao`, `toda_plataforma_listada_resolve` | unidade | `euw1` dá `europe`, `KR` dá `asia`, `br1` dá `br1`; `xx9` e `""` dão `None`; cada host de `platforms()` volta `Some` | R-20 |
| T-ENTRADA-33 | `pool_vazio` | unidade | `Pool::default()` é vazia, e `ids()` e `ids_for_role(Mid)` vêm vazios | R-9 |
| T-ENTRADA-34 | `os_core_de_uma_rota_sao_so_dela` | unidade | Com 103 core no meio, 7 situacional no meio, 64 core na selva e 1 core sem rota: `ids_for_role_tier(Mid, Core) == [103]` (o curinga 1 fica de fora) e `(Mid, Situational) == [7]` | R-9, ADR-0016 |
| T-ENTRADA-35 | `sem_rota_vale_em_qualquer_uma`, `rota_marcada_nao_vaza_para_outra` | unidade | 103 sem rota aparece no meio e no topo; 64 marcado na selva aparece só na selva | R-1, R-9 |
| T-ENTRADA-36 | `mesmo_campeao_em_duas_rotas_convive`, `upsert_atualiza_o_tier_sem_duplicar` | unidade | 157 core no meio e situacional no topo: duas entradas, `ids() == [157]` e um tier por rota; 103 `Extra` e depois `Core` no meio: uma entrada só, `Core` | R-9 |
| T-ENTRADA-37 | `remove_so_a_entrada_daquela_rota` | unidade | 157 no meio e no topo, `remove(157, Some(Top))`: sobra uma entrada, a do meio | R-9 |
| T-ENTRADA-38 | `sem_cota_por_tier` | unidade | Dez core no meio cabem: `ids_for_role(Mid).len() == 10` | R-9 |
| T-ENTRADA-39 | `tier_mais_forte_ganha_quando_ha_rota_e_curinga`, `campeao_fora_do_pool_nao_tem_tier` | unidade | 103 `Extra` sem rota e `Core` no meio dão `tier_of(103, Mid) == Some(Core)`; 999 dá `None` | R-1, R-9 |
| T-ENTRADA-40 | `por_padrao_tudo_aparece`, `arquivo_vazio_ou_corrompido_nao_esconde_nada` | unidade | O padrão mostra as seis seções; `{}` desserializa no padrão, com tudo visível | R-15 |
| T-ENTRADA-41 | `desligar_e_religar_volta_ao_estado_anterior`, `desligar_duas_vezes_nao_duplica` | unidade | Desligar `Gold` não mexe em `Damage`, e religar volta a `OverlayPrefs::default()`; desligar `Skirmish` duas vezes deixa `hidden.len() == 1` | R-15 |
| T-ENTRADA-42 | `tudo_desligado_e_detectavel` | unidade | Com as seis desligadas, `anything_visible()` é falso e `visible()` vem vazio | R-15 |
| T-ENTRADA-43 | `a_ordem_das_visiveis_e_a_do_overlay` | unidade | Sem `Objectives`, `visible() == [Gold, Damage, Priority, Skirmish, NextItem]` | R-15 |
| T-ENTRADA-44 | `secao_nova_nasce_visivel_para_quem_ja_tinha_configuracao`, `sobrevive_a_ida_e_volta_em_json` | unidade | `{"hidden":["gold"]}`, salvo antes de `skirmish` existir, esconde `Gold` e mostra `Skirmish`; `Gold` e `NextItem` desligados voltam iguais do JSON | R-15 |
| T-ENTRADA-45 | `toda_secao_tem_rotulo_e_explicacao` | unidade | Toda seção de `Section::ALL` tem `label()` e `detail()` não vazios | R-15 |
| T-ENTRADA-46 | `le_os_nomes_de_campo_do_lcu` | fronteira | JSON de `/lol-champ-select/v1/session` e do invocador com os nomes do cliente (`actorCellId`, `type`, `championPickIntent`, `assignedPosition`, `spell1Id`, `adjustedTimeLeftInPhase`, `isSpectating`, `gameName`, `tagLine`…): cada campo chega com o valor escrito, nenhum no padrão | R-18, A19 |
| T-ENTRADA-47 | `le_os_nomes_de_campo_da_live_client` | fronteira | JSON de um jogador de `/liveclientdata/playerlist` (`rawChampionName`, `riotId`, `isDead`, `respawnTimer`, `itemID`, `creepScore`, `wardScore`…): cada campo chega com o valor escrito | R-11, R-12, A19 |
| T-ENTRADA-48 | `secao_desconhecida_ou_repetida_nao_derruba_a_escolha` | regressão | `{"hidden": ["gold", "radar", "gold", "nextItem"]}` lê como `[Gold, NextItem]`. Antes, a seção desconhecida derrubava a leitura inteira e tudo voltava a aparecer | R-15, A31 |
| T-ENTRADA-49 | `a_pool_gravada_guarda_os_invariantes` | unidade | Uma pool com o Mid 103 duas vezes (core e situacional), um id 0, o 103 sem rota, e os bans `[1, 2, 2, 0, 3, -4, 5, 6, 7]`: `normalized` deixa o 103 do meio situacional e o 103 sem rota, e os bans `[1, 2, 3, 5, 6]` | R-3, R-9, A29 |
| T-ENTRADA-50 | `o_overlay_so_pega_clique_quando_mostra_algo` | unidade | Com o atalho e na partida, `atravessa_o_mouse` é falso com tudo ligado e com só uma seção ligada; sem o atalho, fora da partida ou com as seis desligadas, é verdadeiro | R-15, A15 |

### 2.2 Coletor — opções, estado, cliente da Riot, agregação e limite de taxa

| ID | Teste(s) | Tipo | O que prova | Atende |
|---|---|---|---|---|
| T-COLETOR-1 | `a_data_sai_em_iso` | unidade | `agora_iso()` tem 20 caracteres, `-` na posição 4, `T` na 10, termina em `Z` e o ano está em [2024, 2100) | R-20 |
| T-COLETOR-2 | `o_estado_so_retoma_o_mesmo_recorte` | unidade | Estado `br1`/`EMERALD`/420 combina com as mesmas opções; `euw1` ou fila 440 não | R-20 |
| T-COLETOR-3 | `a_saida_padrao_e_onde_o_app_procura`, `o_estado_nao_vai_para_o_mesmo_arquivo_da_base` | integração | A saída padrão termina em `Presage/base-de-partidas.json` (ou com `\`), o nome que a casca lê (`DATASET_CACHE`); estado e saída padrão são arquivos diferentes. O primeiro falha fora do Windows | R-20, R-21, L-18 |
| T-COLETOR-4 | `o_estado_sobrevive_a_ida_e_volta_em_json` | unidade | `proximo == 1`, `matches == 7`, três `Skip::Remake` e a partida `BR1_123` vista voltam iguais do JSON | R-20 |
| T-COLETOR-5 | `o_rotulo_de_elo_so_diz_mais_quando_a_semeadura_chegou_la` | unidade | Pedido `EMERALD`: semeado só esmeralda dá `EMERALD`; esmeralda e diamante dá `EMERALD, DIAMOND`; os cinco tiers dá `EMERALD+` | R-20, ADR-0006 |
| T-COLETOR-6 | `json_qualquer_nao_passa_por_base`, `base_de_verdade_nao_e_confundida_com_vazia` | unidade | `{"isto":"nao e uma base"}` parece vazio; `{"patch":"15.21","matches":0}` não; sem patch e sem partidas, uma entrada em `base` também não | R-20 |
| T-COLETOR-7 | `o_limite_padrao_e_o_da_chave_de_desenvolvedor` | unidade | `LIMITE_PADRAO` sustenta menos de 1 chamada/s | R-20 |
| T-COLETOR-8 | `o_backoff_dobra_e_para_de_dobrar` | unidade | Tentativas 1, 2, 3, 4 e 9 esperam 1, 2, 4, 8 e 8 s | R-20 |
| T-COLETOR-9 | `as_ligas_de_topo_sao_reconhecidas` | unidade | `CHALLENGER` e `MASTER` são de topo; `EMERALD` e `DIAMOND` não | R-20 |
| T-COLETOR-10 | `tier_minimo_inclui_tudo_acima`, `o_tier_mais_baixo_pega_todos`, `tier_desconhecido_nao_vira_lista_vazia_silenciosa` | unidade | `emerald` dá cinco tiers, de `EMERALD` a `CHALLENGER`; `IRON` dá os dez; `MADEIRA` dá `None` | R-20 |
| T-COLETOR-11 | `a_chave_nunca_entra_na_url` | unidade | Com a chave `RGAPI-secreta`, nenhuma das duas URLs base contém `RGAPI` | RNF-6, ADR-0004 |
| T-COLETOR-12 | `plataforma_e_regional_tem_hosts_diferentes` | unidade | `br1` e `americas` dão `https://br1.api.riotgames.com` e `https://americas.api.riotgames.com`: dois hosts, que a Riot conta em separado | R-20, P-1 |
| T-COLETOR-13 | `limite_invalido_nao_passa_na_construcao` | unidade | `--limite nada` faz `Riot::new` falhar, em vez de rodar sem limite | R-20 |
| T-COLETOR-14 | `o_patch_sai_das_duas_primeiras_partes` | unidade | `15.21.704.1234` dá `15.21`; `9.3.1` dá `9.3`; `lixo` e `""` dão `""` | R-20 |
| T-COLETOR-15 | `uma_partida_rende_dez_linhas_de_base`, `duas_partidas_somam_na_mesma_chave` | unidade | Uma partida dá `matches == 1`, dez linhas de base, `1:Top` com (1, 1) e `11:Top` com (1, 0); uma vitória e uma derrota dão `1:Top` (2, 1) e `1:11:Top` (2, 1) | R-21 |
| T-COLETOR-16 | `o_matchup_e_direcional_e_sai_nas_duas_direcoes` | unidade | Uma partida dá dez linhas de matchup: `3:13:Mid` com (1, 1) e `13:3:Mid` com (1, 0) | R-21 |
| T-COLETOR-17 | `a_sinergia_e_so_entre_companheiros`, `a_chave_de_sinergia_nao_depende_da_ordem_do_par` | unidade | Uma partida dá 20 pares: `1:2` (1, 1), `11:12` (1, 0) e nenhum `1:11`; `synergy_key(2, 1)` e `(1, 2)` são a mesma linha | R-21 |
| T-COLETOR-18 | `uma_partida_rende_cinquenta_linhas_de_contra` | unidade | Uma partida dá 50 linhas de contra; `1:11` e `11:1` somam uma vitória; `3:11`, de rotas diferentes, existe | R-21, ADR-0019 |
| T-COLETOR-19 | `a_duracao_vai_para_a_faixa_da_partida` | unidade | Partida de 1800 s dá dez linhas de duração, com `1:Top:Media` em (1, 1) | R-21, ADR-0018 |
| T-COLETOR-20 | `remake_fica_de_fora`, `partida_curta_fica_de_fora_mesmo_sem_a_flag`, `fila_diferente_fica_de_fora`, `modo_que_nao_e_a_fenda_fica_de_fora`, `partida_terminada_por_erro_fica_de_fora` | unidade | Um participante com `gameEndedInEarlySurrender` dá `Remake` com `matches == 0` e base vazia; 240 s dá `Curta`; fila 450 dá `Fila`; `ARAM` dá `Modo`; `Abort_Unexpected` dá `Terminada` | R-20, ADR-0006, L-5 |
| T-COLETOR-21 | `resposta_sem_o_campo_novo_nao_e_recusada_por_causa_dele` | unidade | `endOfGameResult` vazio passa: `add` dá `Ok` | R-20 |
| T-COLETOR-22 | `rota_em_branco_derruba_a_partida_inteira`, `rotas_que_nao_fecham_cinco_por_time_ficam_de_fora`, `partida_sem_dez_participantes_fica_de_fora` | unidade | Um `teamPosition` vazio dá `SemRota` com base vazia; dois `MIDDLE` no time 100 dão `RotaRepetida`; nove participantes dão `Incompleta` | R-20, ADR-0006 |
| T-COLETOR-23 | `o_patch_trava_na_primeira_partida_aceita`, `patch_pedido_na_linha_de_comando_manda`, `versao_ilegivel_fica_de_fora` | unidade | A primeira partida trava `15.21`, e uma `15.22` depois dá `Patch` com `matches` em 1; com `--patch 15.22`, a `15.21` dá `Patch` e a `15.22` entra; versão `???` dá `Patch` | R-20, ADR-0006 |
| T-COLETOR-24 | `os_descartes_sao_contados_por_motivo` | unidade | Duas `ARAM` e uma de 100 s dão `skipped[Modo] == 2` e `skipped[Curta] == 1` | R-20 |
| T-COLETOR-25 | `estado_sem_as_tabelas_novas_nao_e_retomado` | unidade | Depois de uma partida, `resumable()`; com `versus_matches` zerado, não; o estado vazio sempre retoma | R-20 |
| T-COLETOR-26 | `a_poda_tira_as_entradas_magras` | unidade | Cinco partidas: com mínimo 1, dez linhas de base; com mínimo 20, nenhuma, e `matches` continua 5 | R-21 |
| T-COLETOR-27 | `a_base_gerada_e_a_que_o_app_le` | integração | 30 partidas podadas em 20 dão base que o `Dataset` do core lê: `is_usable`, `has_global`, `matchup(3, 13, Mid) == (30, 30)`, `base(1, Top) == (30, 30)` e `synergy(1, 2) == synergy(2, 1)` | R-20, R-21 |
| T-COLETOR-28 | `a_cobertura_separa_o_que_ja_serve` | unidade | 210 partidas comuns e uma rara dão 20 linhas de matchup, 10 com n ≥ 200 (`USEFUL`) | R-21 |
| T-COLETOR-29 | `o_espalhamento_do_contra_e_medido_alem_do_ruido` | unidade | Oito campeões a 50% com efeitos plantados de ±300 vitórias em 10.000 (±3pp) dão `versus_sd` a menos de 0,002 de 0,03; sem efeito, `Some(0.0)` | R-21, ADR-0009 |
| T-COLETOR-30 | `le_o_formato_do_cabecalho_da_riot`, `cabecalho_meio_quebrado_aproveita_o_que_da` | unidade | `20:1,100:120` dá dois baldes, o primeiro de 20 e o segundo de janela 120 s; `20:1,lixo,100:120` aproveita os dois válidos | R-20 |
| T-COLETOR-31 | `cabecalho_ilegivel_nao_vira_limitador_sem_limite` | unidade | `""`, `lixo`, `0:1` e `20:0` dão `None` | R-20 |
| T-COLETOR-32 | `dentro_do_limite_nao_espera` | unidade | Com `3:1`, as três primeiras chamadas em t = 0 esperam 0 | R-20 |
| T-COLETOR-33 | `ao_encher_espera_a_janela_da_mais_antiga`, `a_janela_desliza_de_verdade` | unidade | `3:1` cheio em t = 0 espera 1 s em t = 0, 0,6 em t = 0,4 e 0 em t = 1; `2:10` com chamadas em 0 e 5 espera 5 em t = 5, libera em t = 10, e com a de t = 10 espera 5 de novo | R-20 |
| T-COLETOR-34 | `manda_a_janela_mais_apertada_das_duas` | unidade | `10:1,4:60` com quatro chamadas até t = 0,3 espera 59 s em t = 1 | R-20 |
| T-COLETOR-35 | `o_429_para_tudo_pelo_tempo_pedido`, `dois_429_seguidos_nao_encurtam_a_pausa` | unidade | `penalize(10, 5)` dá espera 5 em t = 10, 2 em t = 13 e 0 em t = 15; uma pausa de 30 s seguida de uma de 2 s espera 29 em t = 1 | R-20 |
| T-COLETOR-36 | `adotar_o_limite_do_servidor_nao_esquece_o_que_ja_saiu`, `adotar_o_mesmo_limite_nao_mexe_em_nada`, `adotar_lixo_mantem_o_limite_atual` | unidade | Três chamadas sob `100:1` e depois `adopt(2:10)` esperam 9,7 s em t = 0,3; adotar o mesmo formato mantém o histórico de 1 chamada; adotar um limitador vazio mantém `5:1` | R-20 |
| T-COLETOR-37 | `a_taxa_sustentavel_sai_da_janela_mais_apertada` | unidade | `20:1,100:120` sustenta 100/120 ≈ 0,83/s, não 20/s; `500:10,30000:600` sustenta 50/s | R-20 |
| T-COLETOR-38 | `a_fila_escolhe_a_liga_semeada` | unidade | `liga_da_fila(420)` é `RANKED_SOLO_5x5`, `440` é `RANKED_FLEX_SR`, e `400` e `0` são erro | R-20, A28 |
| T-COLETOR-39 | `so_resposta_definitiva_marca_a_partida_como_vista` | unidade | Partida contada (`Ok(Some)`) e recusada pela Riot (`Ok(None)`) marcam como vista; erro (`Err`, um 500 depois das tentativas) não | R-20, A26 |
| T-COLETOR-40 | `estado_que_nao_le_e_erro_e_nao_comeco_do_zero` | unidade | Sem arquivo, `ler_estado` dá `None`; com `{ quebrado`, dá erro dizendo que não lê; com um `Estado` de verdade, dá o estado | R-20, A27 |
| T-COLETOR-41 | `os_cortes_comuns_sao_os_do_historico` | unidade | A mesma partida pelo coletor e por `history::corte_comum`: `ARAM` dá `Modo` nos dois, `Abort_AntiCheatExit` dá `Terminada`, um remake dá `Remake`, 299 s dá `Curta`, e 300 s com fim vazio e 1500 s com `GameComplete` passam nos dois. Com `MIN_DURATION` em 240, o coletor deixa passar os 299 s | R-17, R-20, A18 |

### 2.3 Draft — ranking, contexto, resposta e composição

Os testes de `engine/mod.rs`, `context.rs`, `roles.rs`, `minimax.rs` e `comp.rs`.
Os do ranking usam um índice de sete campeões falsos e, quando a referência
importa, uma base montada à mão (`Montada`) com números conhecidos
([L-8](09-licoes-do-prototipo.md)). Os valores entre parênteses que o teste não
confere saem da mesma conta, feita à parte.

| ID | Teste(s) | Tipo | O que prova | Atende |
|---|---|---|---|---|
| T-DRAFT-1 | `popularidade_nao_vira_matchup` | regressão | Com o rival do meio em 47% (40000 partidas), um Mago com 5000 partidas e um Tanque com 100, ambos a 53% contra ele, mudam menos de 0,2pp quando essas linhas de matchup entram. Antes da correção, +2,6pp e +0,3pp | RNF-3, R-21, ADR-0008, L-1 |
| T-DRAFT-2 | `sinergia_nao_herda_a_forca_do_aliado` | regressão | Com o aliado do topo em 52% (40000 partidas), um Mago com 900 jogos e um Tanque com 50, ambos a 52% com ele, mudam menos de 0,2pp quando a sinergia entra | RNF-3, R-21, ADR-0008, L-1 |
| T-DRAFT-3 | `contra_no_esperado_nao_mexe_em_ninguem` | regressão | Com o atirador deles no bot em 47%, um Mago com 5000 partidas a 52,96% e um Tanque com 100 a 53% contra ele, que é o esperado (~53%), mudam menos de 0,2pp | RNF-3, R-21, ADR-0019 |
| T-DRAFT-4 | `o_oponente_de_rota_nao_conta_duas_vezes` | unidade | Com o mago deles informado no meio (peso de rota 1), um contra de 80% em 5000 partidas contra ele deixa a `winrate` do Mago idêntica, com `assert_eq` | R-1, ADR-0019 |
| T-DRAFT-5 | `contra_de_verdade_vira_peso_e_razao` | unidade | Um Mago a 60% em 5000 partidas contra o atirador deles, quando o esperado é ~53%, sobe, e ganha a razão que começa com "Bate em AtiradorLivre" (+4,7pp pela conta) | R-1, R-21, ADR-0019 |
| T-DRAFT-6 | `o_peso_de_rota_e_o_encaixe_dele_na_sua_rota` | unidade | Sem base, `lane_share` no meio vale 0,5 para o Assassino, que pela classe joga selva e meio, e 1,0 para o Mago | R-4, ADR-0019 |
| T-DRAFT-7 | `historico_pessoal_curto_nao_domina`, `historico_pessoal_entra_no_score_e_na_razao` | regressão | 15V-5D no Mago sobe a `winrate` dele, mas menos de 3pp (+2,2pp pela conta; antes, +8,3pp), e a razão traz "15V-5D" | RNF-2, ADR-0009, ADR-0012, L-2 |
| T-DRAFT-8 | `historico_pessoal_nao_preenche_matchup` | unidade | 200 vitórias suas no Mago deixam `sample = 0` e não criam razão "na rota" | RNF-1, R-1, ADR-0007 |
| T-DRAFT-9 | `sem_dados_o_intervalo_e_largo`, `com_dados_o_intervalo_encolhe` | unidade | Sem base, o primeiro da lista tem `confidence > 8` (11,2pp pela conta) e `sample = 0`. Com 20000 partidas de base e 5000 de matchup, o `±` do Mago encolhe (8,4pp pela conta) e `sample = 20000` | RNF-2, ADR-0007, ADR-0009 |
| T-DRAFT-10 | `sem_estatistica_o_ranking_vem_da_composicao`, `frontline_pesa_mais_que_dano_em_time_sem_tanque` | unidade | Com aliados Lutador, Assassino e Atirador e sem base, Mago e Tanque ficam acima de 50%, e o Tanque, que preenche a frontline (≈51,4%), vem antes do Mago, que equilibra o dano (≈50,6%) | R-1, RNF-1, ADR-0007 |
| T-DRAFT-11 | `sem_estatistica_a_razao_diz_isso` | unidade | Sem aliado, sem inimigo e sem base, todo candidato do ranking traz a razão "Sem base estatística ainda — ordenado só por composição". Antes o teste conferia a razão de frontline, e esta podia sumir | R-1, RNF-1, A20 |
| T-DRAFT-12 | `matchup_favoravel_vira_razao` | unidade | Um Mago a 60% em 5000 partidas contra o mago deles ganha a razão que começa com "Vence Inimigo" (+4,8pp pela conta) | R-1, R-21 |
| T-DRAFT-13 | `o_oponente_de_rota_e_achado_sem_posicao` | regressão | Com a posição do mago deles apagada, ele é inferido no meio, e o Mago continua com a razão "Vence Inimigo" | R-1, R-4, ADR-0017, L-25 |
| T-DRAFT-14 | `quem_nao_joga_a_rota_fica_de_fora` | regressão | No meio e sem pool, o atirador disponível (id 7) não aparece no ranking | R-1, ADR-0017, L-23 |
| T-DRAFT-15 | `a_pool_passa_por_cima_da_regra_de_rota` | unidade | Com o atirador na pool, ele aparece no ranking do meio, com `in_pool` | R-1 |
| T-DRAFT-16 | `indisponiveis_ficam_de_fora` | unidade | Com 1, 2, 3 e 6 indisponíveis e o 7 fora da rota, os ids ranqueados são exatamente [4, 5] | R-1 |
| T-DRAFT-17 | `pool_e_marcado_sem_alterar_a_ordem` | unidade | A pool [5] liga `in_pool` no Tanque, e a ordem dos ids fica igual à de sem pool | R-1 |
| T-DRAFT-18 | `ordem_e_deterministica` | propriedade | Duas chamadas com a mesma entrada devolvem vetores iguais, campo a campo | R-1 |
| T-DRAFT-19 | `winrate_fica_dentro_dos_limites` | propriedade | Com base, toda `winrate` fica em [0,25; 0,75] | R-1 |
| T-DRAFT-20 | `indice_vazio_nao_entra_em_panico` | unidade | Com o `ChampionIndex` vazio, `rank` devolve lista vazia, sem pânico | R-1 |
| T-DRAFT-21 | `rota_clara_vira_razao_sem_mexer_na_ordem`, `rota_dentro_do_ruido_nao_vira_razao` | unidade | 20 partidas suas de Mago a +10 tropas aos 10 (ruído de ±3) viram uma razão com "tropas aos 10" e "n=20", e nenhuma `winrate` muda em relação a `NoStats`. Seis partidas a ±0,5 não viram razão | R-17, RNF-11, ADR-0011 |
| T-DRAFT-22 | `a_rota_do_inimigo_sai_de_quem_ele_e_nao_da_ordem` | regressão | Atirador, Assassino e Suporte deles, sem posição e escolhidos nessa ordem, vão para bot, selva ou meio, e suporte, e `roles_assumed` liga. Pela ordem seriam topo, selva e meio | R-4, ADR-0017, L-25 |
| T-DRAFT-23 | `posicao_informada_nunca_e_contestada` | unidade | Um atirador que o cliente põe no meio fica no meio, `roles_assumed` fica desligado, e ele é o oponente de rota | R-4, ADR-0017 |
| T-DRAFT-24 | `o_empate_da_sempre_a_mesma_resposta` | propriedade | Dois magos deles sem posição: duas leituras do mesmo draft dão a mesma atribuição | R-4, ADR-0017 |
| T-DRAFT-25 | `hover_de_aliado_nao_e_recomendado` | regressão | Com um aliado em hover no Mago, o 4 entra em `unavailable` e sai dos candidatos | R-1, ADR-0017, L-26 |
| T-DRAFT-26 | `a_sua_escolha_nao_travada_continua_sua` | regressão | O seu slot com `championId` 4 ainda sem travar deixa o 4 fora de `unavailable` e dentro dos candidatos; travado, o 4 entra em `unavailable` | R-1, ADR-0017, L-26, P-12 |
| T-DRAFT-27 | `so_entra_quem_joga_a_rota_ou_esta_na_pool` | unidade | No meio, o Mago é candidato e o Atirador não; com o Atirador na pool, ele entra | R-1, L-23 |
| T-DRAFT-28 | `rota_desconhecida_nao_filtra` | unidade | Sem posição local, os sete campeões do índice são candidatos | R-1 |
| T-DRAFT-29 | `a_rota_vem_do_cliente_da_escolha_ou_da_pool` | unidade | `resolve_role` dá Top para "top" com a escolha Mid; Bot para posição vazia com a escolha Bot; Mid sem escolha e com pool de meio; e Unknown com pool vazia | R-1, ADR-0017 |
| T-DRAFT-30 | `a_rota_marcada_vence_a_inferida` | unidade | O atirador deles sem posição é inferido no bot; marcado no meio, vai para o meio, vira o oponente de rota, e `roles_assumed` fica desligado | R-4 |
| T-DRAFT-31 | `a_rota_marcada_tira_a_rota_dos_outros` | unidade | Com atirador e suporte sem posição, marcar o suporte no bot tira o atirador do bot, e `roles_assumed` continua ligado | R-4 |
| T-DRAFT-32 | `duas_marcas_na_mesma_rota_ficam_com_a_primeira` | unidade | Dois magos marcados no topo: o do primeiro slot fica no topo e o do segundo não | R-4 |
| T-DRAFT-33 | `a_marca_nao_contesta_a_posicao_do_cliente` | unidade | Um atirador que o cliente põe no meio, marcado no bot, fica no meio | R-4 |
| T-DRAFT-34 | `so_entra_quem_a_sua_conta_pode_escolher` | unidade | Com a lista da conta {7} e a pool [4], o 7 é candidato e o 4 não: a pool não passa por cima da conta | R-5 |
| T-DRAFT-35 | `sem_a_lista_da_conta_ninguem_sai` | unidade | `owned = Some(vazio)` dá os mesmos candidatos de sem lista | R-5 |
| T-DRAFT-36 | `o_candidato_entra_no_seu_lugar` | unidade | `with_candidate(4).lineups(PREGAME_LEVEL)` põe o 4 no meio do seu lado, e o lado deles traz o único inimigo travado | R-6, ADR-0017 |
| T-DRAFT-37 | `rota_desconhecida_nao_filtra_nem_com_base` | unidade | `plays` com `Role::Unknown` é verdadeiro com `NoStats` e com uma base ligada e vazia | R-1 |
| T-DRAFT-38 | `tanque_sem_classe_de_suporte_nao_vira_suporte` | unidade | Sem base, [Tank, Fighter] não joga suporte e joga topo; [Tank, Support] joga suporte | R-1, R-9 |
| T-DRAFT-39 | `classe_principal_pesa_mais_que_a_secundaria` | unidade | Sem base, `role_fit` no meio vale 1,0 para [Mage, Support] e 0,5 para [Support, Mage]; no bot, 0 para [Mage, Support] | R-4, P-13 |
| T-DRAFT-40 | `com_a_base_a_rota_sai_das_partidas` | unidade | Um lutador com 300 partidas no meio e 700 no topo joga o meio com a base (30% ≥ 5%) e não joga sem ela; `role_fit` no meio dá 0,3 | R-1, R-21 |
| T-DRAFT-41 | `campeao_sem_partida_na_base_cai_na_classe` | unidade | Com a base ligada e nenhuma partida dele, um mago novo joga o meio pela classe | R-1, R-21 |
| T-DRAFT-42 | `o_que_voce_ja_jogou_na_rota_e_da_rota` | unidade | Uma partida sua de atirador no topo torna `plays` verdadeiro no topo; sem o histórico, falso | R-1, R-9 |
| T-DRAFT-43 | `indice_vazio_devolve_vazio` | unidade | Com o índice vazio, `minimax` devolve lista vazia | R-2 |
| T-DRAFT-44 | `composicao_equilibrada_vale_mais_que_unilateral` | unidade | `team_value([Atirador, Mago], [])` é maior que `team_value([Atirador, Assassino], [])` | R-2 |
| T-DRAFT-45 | `a_funcao_de_valor_e_simetrica` | propriedade | `team_value(A, B) + team_value(B, A) = 1`, com tolerância de 1e-9 | R-2 |
| T-DRAFT-46 | `exposicao_nunca_e_negativa`, `ordena_pelo_valor_depois_da_resposta` | propriedade | Com o Lutador aliado e o Atirador deles, toda `exposure` é ≥ 0, e `after_response` não cresce ao longo da lista | R-2, C-5 |
| T-DRAFT-47 | `indisponiveis_nao_entram_nem_como_pick_nem_como_resposta` | unidade | Com o Lutador aliado e o Atirador deles, nenhum dos dois volta como pick, e o Lutador nunca é `best_response` | R-2 |
| T-DRAFT-48 | `todo_candidato_tem_a_resposta_deles` | unidade | Vinte candidatos para o meio, com o Lutador aliado e o Atirador deles: os 20 voltam do `minimax_with`, cada um com a resposta deles. Até a 0.7.0 a casca pedia largura 12 e voltavam 12, sem o Mago e o Assassino (A21); o teste da largura saiu com ela | R-2, A21 |
| T-DRAFT-49 | `pool_e_marcado_sem_mudar_a_ordem` | unidade | O Mago na pool liga `in_pool`, e a ordem dos ids fica igual à de sem pool | R-2 |
| T-DRAFT-50 | `o_minimax_so_traz_quem_joga_a_sua_rota` | unidade | No meio, só Mago e Assassino voltam do `minimax`. É o teste de `minimax.rs`; `poolgap.rs` tem outro com o mesmo nome | R-2, ADR-0017, L-23 |
| T-DRAFT-51 | `a_resposta_varre_todos_os_campeoes` | regressão | Com 40 lutadores de id 10 a 49, um mago-suporte de id 999 e o atirador deles travado, algum pick tem `best_response = 999` | R-2, L-24 |
| T-DRAFT-52 | `resultado_e_deterministico` | propriedade | Duas chamadas com a mesma entrada devolvem o mesmo vetor | R-2 |
| T-DRAFT-53 | `sem_pick_deles_nao_ha_ban_a_sugerir` | unidade | Draft sem campeão nenhum: `suggest_bans` devolve vazio | R-3, ADR-0007, ADR-0013 |
| T-DRAFT-54 | `bans_saem_ordenados_pela_ameaca`, `limite_de_bans_e_respeitado` | unidade | Com o atirador deles travado, a lista não é vazia e `threat` não cresce ao longo dela; `limit = 3` devolve 3 | R-3 |
| T-DRAFT-55 | `ban_prioriza_quem_completa_o_que_falta_neles` | unidade | Com o atirador deles travado, o Mago vem antes do Assassino | R-3, ADR-0013 |
| T-DRAFT-56 | `ban_nao_sugere_campeao_ja_fora` | unidade | Com o Lutador aliado e o Atirador deles travados, nenhum dos dois aparece entre os bans | R-3 |
| T-DRAFT-57 | `o_valor_e_aditivo_e_e_por_isso_que_nao_ha_ban_pessoal` | limitação | Contra [Atirador], pôr o Mago no time deles tira o mesmo valor do Tanque e do Assassino (diferença < 1e-9). Deve falhar quando `team_value` ganhar termo de interação | R-3, ADR-0013 |
| T-DRAFT-58 | `a_sua_lista_de_bans_vem_antes_e_vale_desde_o_primeiro_ban` | unidade | Sem pick nenhum, a lista [Tanque, Mago] devolve exatamente [Tanque, Mago], todos `from_pool`. Com o atirador deles, a lista [Tanque] e `limit = 3` devolvem o Tanque primeiro, um item da composição em seguida, 3 no total e o Tanque uma vez só | R-3 |
| T-DRAFT-59 | `nao_bane_o_que_o_time_quer_pegar_nem_o_que_ja_saiu` | unidade | Com um aliado em hover no Tanque e o Mago banido por eles, fora do `unavailable` do draft, a lista [Tanque, Mago, Assassino] começa pelo Assassino e não traz nem Tanque nem Mago | R-3, L-26 |
| T-DRAFT-60 | `time_vazio_e_neutro`, `ids_desconhecidos_sao_ignorados` | unidade | `profile([])` tem `known = 0` e `physical_share = 0,5`; `profile([1, 999])` tem `known = 1` | 02 §4 |
| T-DRAFT-61 | `mago_equilibra_time_todo_ad`, `mais_um_ad_em_time_ad_nao_ajuda` | unidade | Com o time Atirador, Lutador e Assassino, o ganho de equilíbrio do Mago é > 0, e o de mais um Atirador é ≤ 0 | R-1 |
| T-DRAFT-62 | `time_sem_tanque_precisa_de_frontline`, `time_com_tanque_nao_precisa` | unidade | `frontline_need` é > 0 para [Atirador, Assassino, Mago] e 0 para [Atirador, Assassino, Tanque] | R-1 |
| T-DRAFT-63 | `sinal_vale_menos_no_comeco_do_draft` | unidade | O ganho do Mago com um aliado (Atirador) é menor que com três. Sem a completude seria o contrário: o ganho bruto é 0,36 com um e 0,15 com três | 02 §4 |
| T-DRAFT-64 | `a_resposta_deles_nao_vem_do_seu_historico` | regressão | Com o atirador deles travado no meio e um histórico seu que só diz "você jogou o mago no topo", nenhuma resposta do `minimax_with` é o mago: pela classe ele não joga topo, selva, bot nem suporte. Antes da correção, o seu histórico o punha entre as respostas deles | R-2, A12 |
| T-DRAFT-65 | `o_valor_do_time_fica_a_8_5_pontos_de_meio` | propriedade | Cinco atiradores só físicos e sem linha de frente contra ninguém valem 0,5 − (0,9 × `W_BALANCE` + `W_FRONTLINE`) ≈ 0,421, abaixo de 0,5 − 0,9 × 0,085; atiradores contra magos, magos contra atiradores e atiradores contra atiradores ficam em 0,5 ± 0,085. É índice, não chance | R-2, RNF-1, A22 |

### 2.4 Prévia — chance do time, condição de vitória e análise de rota

Testes de `core/src/engine/team.rs`, `wincon.rs` e `report.rs`. As bases de
partidas são montadas à mão dentro de cada `mod tests`.

| ID | Teste(s) | Tipo | O que prova | Atende |
|---|---|---|---|---|
| T-PREVIA-1 | `sem_base_nao_ha_chance_do_time` | unidade | Com `NoStats` (`has_global` falso), `estimate` de A (id 1, topo) contra X (id 11, topo) devolve `None`: sem a base não sai chance do time | R-21, RNF-1, ADR-0019 |
| T-PREVIA-2 | `a_chance_dos_dois_lados_soma_um` | propriedade | Numa base coerente (A 10600/20000 no topo, matchup A×X 1800/3000, contra B×X 1400/2500, sinergia A+B 1080/2000, cada registro com o complemento), a chance de A+B contra X+Y somada à de X+Y contra A+B dá 1 com erro < 1e-9, e a de A+B passa de 0,5 | R-21, RNF-3, ADR-0019 |
| T-PREVIA-3 | `fica_entre_35_e_65` | unidade | Na mesma base, a chance de A+B contra X+Y fica em [0,35; 0,65] (`FLOOR`, `CEILING`) e o `±` (`confidence`) é maior que zero | R-21, RNF-2, ADR-0019 |
| T-PREVIA-4 | `diz_de_onde_vem_a_vantagem` | unidade | Na mesma base, as parcelas trazem "A × X na rota" com `shift` positivo, nenhuma tem `games = 0`, e são no máximo 4 (`MAX_TERMS`) | R-21, RNF-1, ADR-0019 |
| T-PREVIA-5 | `rota_incerta_divide_entre_matchup_e_contra` | unidade | A no topo com certeza 1 contra X no topo com certeza 0: `w = 0`, e nenhuma parcela com "×" e "na rota" aparece — o matchup A×X (1800/3000) sai da conta | R-4, ADR-0019, L-25 |
| T-PREVIA-6 | `time_todo_fisico_e_sem_frente_e_descrito`, `time_de_alcance_e_descrito` | unidade | Lutador, Assassino, Lutador2 e Atirador (nenhum Tank, defesa ≤ 5) dão um fato com "físico" e outro com "ninguém segura", todos do lado `Nosso`; Mago, Atirador, Suporte e Atirador2 (alcance 525 a 550) dão um fato com "atacam à distância" | R-6, ADR-0018 |
| T-PREVIA-7 | `time_incompleto_nao_vira_composicao` | unidade | Com três nossos e dois inimigos (Tanque e Mago), nenhum fato sai do lado `Deles`: menos de 3 conhecidos (`MIN_KNOWN`) não é composição | R-6, ADR-0018 |
| T-PREVIA-8 | `sem_base_nao_ha_fase` | unidade | Com `NoStats`, quatro nossos e nenhum inimigo, `phase` é `None` | R-21, RNF-1, ADR-0018 |
| T-PREVIA-9 | `com_base_a_fase_sai_da_duracao_das_partidas` | unidade | Com os nossos (ids ≤ 4) em 2800/5000 nas partidas longas e 2200/5000 nas curtas, e os deles em 2500/5000 nas duas, a fase existe, começa com "Pelos dados" e fala de partidas "longas" | R-21, ADR-0018 |
| T-PREVIA-10 | `toda_afirmacao_diz_de_onde_sai` | unidade | Em 5 contra 5 com o Gigante (vida 900, dano de ataque 85) no topo, há uma vantagem `Nosso` que começa com "TOP"; todo fato é `Regra`, toda vantagem é `Atributos`, e o aviso contém "sem habilidades" | R-6, RNF-1, ADR-0018 |
| T-PREVIA-11 | `nada_aqui_e_ordem` | unidade | No mesmo 5 contra 5, com ao menos uma vantagem, nenhum fato nem vantagem contém "jogue", "force", "evite", "foque", "priorize", "invada", "escolha" ou "pegue" | C-5, R-6, ADR-0018 |
| T-PREVIA-12 | `a_previa_e_a_mesma_conta_de_quando_ele_estiver_travado` | integração | O draft com o seu meio vazio e o candidato 7 (`with_candidate`) dá um `LaneReport` presente e igual, campo a campo, ao do draft com o 7 travado no meio | R-6 |
| T-PREVIA-13 | `o_candidato_encontra_o_oponente_inferido` | regressão | Os inimigos vêm sem posição, como na ranqueada; o Mago 6 é inferido no meio, a rota do candidato 7 sai contra ele (`enemy = 6`), e `roles_assumed` é verdadeiro | R-4, R-6, L-25 |
| T-PREVIA-14 | `sem_inimigo_travado_nao_ha_analise` | unidade | Sem nenhum inimigo no draft, a prévia do candidato 7 é `None`, e não um relatório contra ninguém | R-6, RNF-1 |

### 2.5 Pool — retrato e cobertura

Testes de `core/src/engine/poolgap.rs`. O índice de teste tem magos (1–3),
assassinos (4–6), atiradores (11–14), lutadores (21–23), tanques (31–33) e um
suporte (41).

| ID | Teste(s) | Tipo | O que prova | Atende |
|---|---|---|---|---|
| T-POOL-1 | `pool_vazia_nao_tem_retrato_nem_sugestao` | unidade | Com a pool vazia no meio, `analyze` devolve forte e fraco vazios, e `suggest` com limite 10 devolve lista vazia | R-9, ADR-0016 |
| T-POOL-2 | `tres_magos_mostram_o_que_tem_e_o_que_falta` | unidade | Três magos à distância no topo: os fortes incluem `Magico` e `Alcance`; os fracos incluem `Fisico`, `Frontline` e uma frase com "um estilo só" | R-9, ADR-0016 |
| T-POOL-3 | `o_que_a_rota_nao_cobra_nao_vira_fraqueza` | regressão | Três atiradores no bot: `Frontline` não aparece como fraco nem `Alcance` como forte, porque a rota pesa as duas em 0 | R-9, ADR-0016, L-22 |
| T-POOL-4 | `so_entra_quem_joga_a_rota` | regressão | Com três atiradores no bot e limite 50, todo candidato tem a classe `Marksman`: nenhum tanque entra, por mais que falte frente | R-9, ADR-0016, L-22 |
| T-POOL-5 | `o_primeiro_tapa_o_buraco_maior` | unidade | Três magos no topo, onde faltam dano físico e frente: o primeiro candidato preenche `Frontline` e ao menos mais uma característica que ninguém tinha | R-9, ADR-0016 |
| T-POOL-6 | `cada_escolha_muda_a_proxima` | regressão | Três magos no topo: o primeiro preenche `Frontline`; com o Tanque 31 aceito, nenhum candidato preenche `Frontline` e o primeiro passa a ser outro campeão | R-9, ADR-0016, L-22 |
| T-POOL-7 | `pool_equilibrada_ainda_recebe_sugestao_ate_oito` | regressão | Mago 1, Lutador 21 e Tanque 31 no topo, que cobrem os dois danos, a frente e o corpo a corpo, ainda recebem sugestão: a lista não sai vazia | R-9, ADR-0016, L-22 |
| T-POOL-8 | `pool_de_oito_esta_completa` | unidade | Com 8 ids no topo (`POOL_SIZE`), `suggest` devolve lista vazia | R-9, ADR-0016 |
| T-POOL-9 | `quem_ja_esta_na_pool_nao_e_sugerido` | unidade | Com Lutador 21, Lutador 22 e Tanque 31 no topo e limite 50, nenhum dos três volta como candidato | R-9 |
| T-POOL-10 | `segunda_copia_vale_menos_que_a_primeira` | unidade | Dois atiradores físicos no bot: o primeiro candidato é o AtiradorAP (14, classe principal `Mage`) e ele preenche `Magico`, à frente dos atiradores físicos que só reforçam | R-9, 02 §7 |
| T-POOL-11 | `a_robustez_premia_resistencias_equilibradas` | unidade | Com a mesma vida (650), armadura e resistência mágica 40/40 dão `robustness` maior que 80/20 (≈0,90 contra ≈0,64 no nível 11), e as duas ficam entre 0 e 1 | R-9, 02 §7 |
| T-POOL-12 | `a_lista_sai_ordenada_limitada_e_com_motivo` | unidade | Três magos no topo com limite 4: no máximo 4 candidatos, `score` não cresce ao descer a lista, cada `score` fica em [0, 1] e cada candidato tem ao menos um motivo | R-9, C-5, RNF-1 |
| T-POOL-13 | `campeao_fora_do_indice_nao_derruba_a_conta` | unidade | Com o Mago 1 e o id 999, que não está no índice, `analyze` no meio ainda dá pontos fortes e `suggest` termina sem pânico | R-9 |

### 2.6 Runas — regras do confronto

Testes de `core/src/engine/runes.rs`. O índice de teste tem as 15 runas que as
regras citam, com os ids e as árvores do Data Dragon.

| ID | Teste(s) | Tipo | O que prova | Atende |
|---|---|---|---|---|
| T-RUNAS-1 | `sem_indice_de_runas_nao_ha_plano`, `sem_oponente_conhecido_nao_ha_plano` | unidade | Lutador contra Mago no topo com `RuneIndex::default()` dá `None`; com o índice cheio e o oponente 999 fora do índice de campeões, também `None` | R-7, RNF-1 |
| T-RUNAS-2 | `corpo_a_corpo_contra_distancia_recebe_sustentacao` | unidade | Tanque corpo a corpo (alcance 150) contra Mago à distância (550) no topo: `GraspOfTheUndying` está entre as principais | R-7 |
| T-RUNAS-3 | `atirador_contra_frontline_quer_troca_longa`, `atirador_contra_alvo_fragil_quer_troca_curta`, `o_mesmo_campeao_muda_de_runa_conforme_o_oponente` | unidade | O mesmo Atirador no bot: contra o Tanque a primeira é `LethalTempo`; contra o Mago, `PressTheAttack` está na lista; e a primeira das duas listas é diferente | R-7, C-6 |
| T-RUNAS-4 | `assassino_contra_alvo_fragil_quer_explosao` | unidade | Assassino corpo a corpo (alcance 150) contra Mago no meio: a primeira é `Electrocute` | R-7 |
| T-RUNAS-5 | `contra_quem_explode_a_secundaria_corta_a_explosao`, `contra_desgaste_de_longe_a_secundaria_regenera` | unidade | Lutador contra Assassino no topo: a secundária é `BonePlating`; Assassino corpo a corpo contra Mago à distância no meio, com a principal fora de Determinação: `SecondWind` | R-7 |
| T-RUNAS-6 | `runa_que_sumiu_do_patch_nao_e_sugerida` | regressão | Com `Electrocute` tirada do índice, o Assassino contra o Mago no meio recebe um plano sem ela, e toda principal sugerida existe no índice | R-7, L-17 |
| T-RUNAS-7 | `nunca_sai_mais_que_o_teto_de_sugestoes`, `a_lista_sai_ordenada`, `toda_runa_sugerida_vem_com_motivo` | unidade | Lutador contra Tanque no topo: no máximo 3 principais (`MAX_SUGGESTIONS`); Atirador contra Tanque no bot: `score` não cresce ao descer a lista; Assassino contra Atirador no meio: toda principal tem motivo | R-7, C-5, RNF-1 |
| T-RUNAS-8 | `o_plano_nunca_se_apresenta_como_estatistica` | limitação | Lutador contra Mago no topo: `from_data` é falso. Deve mudar quando a base de partidas entrar em `plan` | R-7, RNF-1 |
| T-RUNAS-9 | `o_confronto_aparece_escrito` | unidade | Lutador contra Mago no topo: `matchup` é exatamente "Lutador contra Mago" | R-7 |
| T-RUNAS-10 | `a_secundaria_nunca_e_da_arvore_da_principal` | regressão | Tanque corpo a corpo contra Mago à distância no topo: a principal é `GraspOfTheUndying` (Determinação), e a secundária, se houver, é de outra árvore. Antes da correção, saía `SecondWind`, também de Determinação, que o cliente não aceita | R-7, A10 |

### 2.7 Estatística — shrinkage, referência e a base de partidas

`engine/stats.rs` e `engine/dataset.rs`. As taxas são fração 0–1.

| ID | Teste(s) | Tipo | O que prova | Atende |
|---|---|---|---|---|
| T-STATS-1 | `sem_amostra_devolve_o_prior`, `sem_amostra_o_desvio_e_zero` | unidade | Sem registro, `shrink(None, 0.5, 400)` devolve 0,5 com 0 partidas, e `shrunk_delta(None, 0.52, 800)` devolve desvio 0 (±10⁻¹²) | RNF-1, ADR-0007 |
| T-STATS-2 | `amostra_grande_domina_o_prior`, `amostra_pequena_fica_perto_do_prior` | unidade | Com prior 0,5 e força 400, 10 vitórias em 10 partidas ficam abaixo de 0,52 (dão 0,512), e 60 000 em 100 000 ficam a menos de 0,01 de 0,60 | R-21, ADR-0007 |
| T-STATS-3 | `erro_padrao_encolhe_com_a_amostra` | unidade | Com 60% de vitórias e força 400, o erro padrão de 50 000 partidas (≈ 0,002) é menor que o de 50 (≈ 0,024) | RNF-2 |
| T-STATS-4 | `vitorias_acima_de_jogos_sao_cortadas`, `forca_zero_nao_divide_por_zero` | unidade | `Record::new(10, 99)` guarda 10 vitórias e a taxa encolhida fica em [0, 1]; com 0 partidas e força 0, taxa e erro padrão saem finitos | 02 §2 |
| T-STATS-5 | `forca_e_desvio_sao_a_mesma_crenca` | regressão | Para desvios de 1,2pp, 3pp e 3,5pp, a Beta de força `strength_for_sd(sd)` tem desvio `sd`, e `posterior_sd(0, sd)` devolve `sd` (±10⁻¹²) | RNF-2, ADR-0009, L-2 |
| T-STATS-6 | `desvio_posterior_cai_com_a_raiz_da_amostra` | regressão | Com prior de 3pp, o desvio de 10 000 partidas é ≈ 1,98 vez o de 40 000: cai com a raiz da amostra, não com 1/n | RNF-2, ADR-0009, L-3 |
| T-STATS-7 | `rival_e_aliado_neutros_nao_mudam_a_esperada`, `rival_fraco_sobe_a_esperada_e_aliado_forte_tambem` | unidade | Rival ou aliado em 0,5 deixam 0,53 em 0,53; rival em 0,47 leva 0,5 a 0,53, aliado em 0,52 leva 0,5 a 0,52, e rival em 0,53 deixa 0,5 abaixo de 0,5 | RNF-3, ADR-0008, L-1 |
| T-STATS-8 | `nostats_nao_tem_dado` | unidade | `NoStats` não tem camada global nem pessoal, e `base(103, Mid)` é `None` | RNF-5, RNF-1 |
| T-STATS-9 | `pessoal_nao_inventa_dado_global`, `campeao_nunca_jogado_nao_tem_registro` | unidade | Com uma partida de 103 no histórico, há camada pessoal e não global; base, matchup e sinergia são `None`, e o campeão 777 nunca jogado também | RNF-1, ADR-0007 |
| T-STATS-10 | `pessoal_acumula_vitorias_e_derrotas`, `partida_repetida_nao_conta_duas_vezes` | unidade | Vitória e derrota com 103 e vitória com 64 dão 103 com 2 partidas e 1 vitória, 64 com 1 e 1, total 3; a mesma partida duas vezes soma uma (`true`, depois `false`) | R-1, 02 §5 |
| T-STATS-11 | `partida_lida_soma_vitoria_e_rota` | unidade | Uma partida com oponente direto (+7 tropas aos 10) soma 1 vitória a 103 e uma `LaneGame`; repetida, devolve `false` e não soma nada | R-17, ADR-0010, L-6 |
| T-STATS-12 | `partida_descartada_fica_vista_sem_contar` | unidade | `mark_seen` duas vezes na mesma partida deixa `seen` com um id e o total em 0 | R-17, L-5 |
| T-STATS-13 | `cache_antigo_carrega_sem_rotas` | unidade | O cache sem `lanes` carrega: 103 com 2 partidas e 1 vitória, `lanes` vazio e perfil de rota `None` | R-17, L-9 |
| T-STATS-14 | `contra_e_duracao_saem_da_base`, `a_camada_repassa_contra_e_duracao` | unidade | A base responde `versus(103, 64)` com 700 partidas e 330 vitórias e `None` para `versus(64, 103)`; `length(103, Mid, Longa)` com 9000 e 4900, `None` para `Curta`; o `Layered` repassa os dois, e sem base devolve `None` | R-21, ADR-0019 |
| T-STATS-15 | `a_duracao_cai_na_faixa_certa` | unidade | 1200 s caem em `Curta`, 1500 s já em `Media`, 2400 s em `Longa` | R-21, ADR-0018 |
| T-STATS-16 | `base_vazia_nao_responde_nada`, `base_que_diz_ter_partidas_mas_esta_vazia_nao_conta`, `base_inutilizavel_nao_e_aceita_como_global` | unidade | A base vazia não responde base, matchup nem sinergia; a que diz 9 000 000 partidas sem nenhum mapa não é utilizável; `with_global` com 10 partidas e `base` vazio deixa a camada global desligada e `patch()` em `None` | RNF-1, ADR-0007 |
| T-STATS-17 | `base_com_partidas_responde`, `a_rota_faz_parte_da_chave`, `matchup_e_direcional_ao_contrario_da_sinergia`, `a_sinergia_nao_depende_da_ordem_do_par` | unidade | Com a base montada, `matchup(103, 238, Mid)` dá 1840 partidas e 1050 vitórias e `matchup(238, 103, Mid)` é `None`; `base(103, Top)` é `None` com `103:Mid` presente; `synergy(103, 64)` é igual a `synergy(64, 103)` | R-21 |
| T-STATS-18 | `as_camadas_sao_independentes`, `as_duas_camadas_juntas_respondem_as_duas_perguntas` | unidade | Só o histórico não responde matchup; só a base não responde o pessoal de 103; as duas juntas respondem matchup, o pessoal (1 partida, 1 vitória), `patch()` "16.21" e `matches()` 1 200 000 | RNF-1, R-21 |
| T-STATS-19 | `base_antiga_sem_contra_carrega`, `arquivo_so_com_o_cabecalho_carrega_sem_quebrar`, `o_formato_sobrevive_a_ida_e_volta_em_json` | unidade | Um JSON só com `patch`, `matches` 10 e `base` é utilizável, com `versus` e `length` vazios; só com `patch` e `matches` 0 carrega e não é utilizável; a base escrita e relida mantém `matches` e o matchup de 1840 e 1050 | R-20, R-21 |
| T-STATS-20 | `as_chaves_da_base_nao_mudam`, `registro_lido_do_disco_nunca_tem_mais_vitorias_que_partidas` | fronteira | As chaves da base são `"103:Mid"`, `"103:238:Support"`, `"103:Top:Media"`, com o nome de cada rota e faixa escrito à mão; um registro lido com 12 vitórias em 10 partidas vira `Record::new(10, 12)`. Antes, o registro carregava como estava, e a chave vinha do `Debug` da variante | R-21, A25 |

### 2.8 Perfil de rota

`engine/profile.rs`. As séries usam ruído fixo de média zero (`RUIDO`, de −3 a +3),
em tropas aos 10.

| ID | Teste(s) | Tipo | O que prova | Atende |
|---|---|---|---|---|
| T-PERFIL-1 | `menos_de_cinco_partidas_nao_vira_perfil` | unidade | Com 4 partidas, `overall` e `champions` saem vazios | R-17 |
| T-PERFIL-2 | `media_geral_e_medida_nao_encolhida`, `um_campeao_so_fica_na_propria_media` | unidade | Com 10 partidas em torno de +6, o geral tem `raw` igual à média, `value == raw` e `sd = s / √10`; com 12 partidas de um campeão só, o campeão tem o mesmo valor e desvio do geral (±10⁻¹²) | R-17, ADR-0010 |
| T-PERFIL-3 | `campeoes_que_nao_se_distinguem_colapsam_na_media` | regressão | Dois campeões com centros 0 e +1, seis partidas cada: os dois ficam no valor do geral (±10⁻⁹) e nenhum é claro | R-17, RNF-11, L-19 |
| T-PERFIL-4 | `diferenca_clara_sobrevive`, `amostra_pequena_encolhe_mais` | unidade | Com +10 e −10 em 20 partidas cada, o valor fica a menos de 0,5 da média crua e os dois são claros; um terceiro campeão com 2 partidas em +25 é puxado para o geral mais que o dobro da fração do primeiro | R-17 |
| T-PERFIL-5 | `confronto_de_uma_partida_fica_perto_do_campeao` | unidade | 20 partidas em +5 e uma contra o 777 em −30: o confronto fica mais perto do valor do campeão do que de −30 | R-17 |
| T-PERFIL-6 | `metrica_que_falta_nao_entra_na_conta` | unidade | Seis partidas de meio com tropa e seis de suporte sem tropa: o geral de `Cs10` tem 6 partidas, o suporte não tem leitura de `Cs10` e o campeão dele conta 6 partidas | R-17, L-27 |
| T-PERFIL-7 | `espalhamento_conta_so_o_que_passa_do_ruido` | unidade | Médias +5 e −5 de 10 partidas com ruído 10 dão τ² = 49; médias +0,5 e −0,5 dão 0; um grupo de uma célula dá 0 | R-17, 04 §4 |
| T-PERFIL-8 | `a_razao_sai_da_leitura_mais_segura` | unidade | Com +10 e −10 em 20 partidas cada, a leitura mais clara do primeiro campeão é `Cs10`, com valor positivo | R-1, ADR-0011 |
| T-PERFIL-9 | `o_perfil_nao_resume_o_jogador_num_numero` | arquitetura | Perfil de 24 partidas em dois campeões: nenhuma chave com um número sozinho se chama nota, rating, MMR, elo, score, overall, tier ou rank. `overall` existe, mas é a lista de leituras por métrica | C-7, RNF-11 |

### 2.9 Sua rota ao vivo

`engine/rival.rs`. Partidas montadas à mão com a Live Client já traduzida.

| ID | Teste(s) | Tipo | O que prova | Atende |
|---|---|---|---|---|
| T-RIVAL-1 | `suporte_ao_vivo_e_comparado_pela_visao_e_nunca_por_tropas` | regressão | Aos 10 minutos, o suporte com 10 tropas e 15 de visão contra 40 tropas e 8 de visão é comparado por visão: 1,5 contra 0,8 por minuto | R-11, L-27 |
| T-RIVAL-2 | `quem_farma_e_comparado_em_tropas_contra_o_da_mesma_rota` | unidade | O meio com 70 tropas aos 10 minutos é comparado com o meio inimigo (85), não com o topo (95): 7,0 contra 8,5 por minuto | R-11 |
| T-RIVAL-3 | `sem_oponente_na_rota_ou_sem_rota_nao_ha_painel` | unidade | Sem inimigo na sua rota, ou com a sua posição em branco, `report` devolve `None` | R-11 |
| T-RIVAL-4 | `antes_de_um_minuto_e_meio_a_taxa_e_ruido` | unidade | Na borda de `MIN_SECONDS`: aos 89 s não há painel; aos 90 s há. Antes o teste usava 1:00, e o limite podia ir a 61 s sem ninguém ver | R-11, A20 |
| T-RIVAL-5 | `na_frente_pelo_ouro_em_itens_com_piso_e_proporcao` | unidade | Cedo vale o piso de 300: 1 300 contra 1 000 é na frente, 1 299 igual e 700 atrás; tarde vale 10% da média dos dois: 10 500 contra 10 000 é igual e 11 100 contra 10 000 na frente | R-11 |
| T-RIVAL-6 | `a_margem_e_a_mesma_dos_dois_lados` | regressão | Para 11 000 contra 10 000, 1 300 contra 1 000, 5 000 contra 4 520 e 20 000 contra 18 100, o veredito de um lado é o espelho do outro. Antes da correção, 11 000 contra 10 000 era "à frente" de um lado e "igual" do outro | R-11, A24 |

### 2.10 Jogo — campeão, itens e dano

`engine/champion.rs`, `engine/items.rs` e `engine/damage.rs`. São regras do jogo e
do Data Dragon, sem amostra.

| ID | Teste(s) | Tipo | O que prova | Atende |
|---|---|---|---|---|
| T-JOGO-1 | `marksman_pende_para_fisico_mesmo_com_info_ruim`, `mago_pende_para_magico`, `champeao_sem_info_nao_divide_por_zero` | unidade | Atirador com `info` 10/6 sai acima de 0,75 físico (0,76); mago com 2/9 abaixo de 0,25 (0,14); sem classe e com 0/0, dentro de [0, 1] (0,6) | R-1, R-10, 02 §4 |
| T-JOGO-2 | `tank_e_frontline_pela_tag`, `defesa_alta_conta_como_frontline_sem_a_tag`, `carry_nao_e_frontline` | unidade | A classe `Tank` é linha de frente; um lutador com defesa 7 também; um atirador com defesa 4 não | R-1, 02 §4 |
| T-JOGO-3 | `nivel_um_e_a_base_crua`, `nivel_dezoito_fecha_linear`, `crescimento_e_sublinear_no_meio`, `nivel_fora_da_faixa_e_limitado` | unidade | Com base 600 e ganho 100: nível 1 dá 600, nível 18 dá 2300, nível 10 fica entre 600 e 1500 (dá 1374); nível 0 vale o 1 e 99 vale o 18 | R-6, R-10, 03 §2 |
| T-JOGO-4 | `stats_base_crescem_juntos` | unidade | Vida e armadura do nível 18 passam as do 1, e o dano de ataque do 18 é 60 + 17 × 3 = 111 | R-6, R-10 |
| T-JOGO-5 | `posicoes_do_lcu` | fronteira | `utility` vira `Support`, `MIDDLE` vira `Mid` e a posição vazia vira `Unknown` | R-1, R-11 |
| T-JOGO-6 | `le_penetracao_percentual_que_o_bloco_stats_nao_tem`, `le_letalidade`, `distingue_pen_magica_plana_de_percentual` | fronteira | Nas descrições do patch 16.21.1: o Lord Dominik's dá 35 de dano de ataque, 0,35 de penetração de armadura e 0,25 de crítico; o Youmuu's, 55 e 18 de letalidade; o Cajado do Vazio, 0,40 de pen mágica percentual e 0 plana; o "18 Magic Penetration", 18 plana e 0 percentual | R-10, ADR-0015, L-11 |
| T-JOGO-7 | `descricao_sem_bloco_de_stats_nao_entra_em_panico`, `bloco_truncado_nao_entra_em_panico`, `rotulo_desconhecido_e_ignorado_sem_perder_o_resto` | unidade | Descrição sem `<stats>`, vazia ou cortada no meio de uma tag devolve tudo em zero; "10 Omnivamp" antes de "40 Armor" deixa a armadura em 40 | R-10, ADR-0015 |
| T-JOGO-8 | `penetracao_percentual_combina_multiplicativo`, `critico_nao_passa_de_cem_por_cento` | unidade | Duas fontes de 35% de penetração de armadura dão 57,75%, não 70%; dois críticos de 75% param em 100% | R-10, 03 §3 |
| T-JOGO-9 | `letalidade_vira_penetracao_plana` | unidade | Os 18 de letalidade do Youmuu's viram `Penetration` com 18 plana e 0 percentual | R-10 |
| T-JOGO-10 | `totais_ignoram_item_desconhecido` | unidade | Com só o 3036 (3300 de ouro) no índice, a lista `[3036, 99999]` soma 35 de dano de ataque e 3300 de ouro | R-10 |
| T-JOGO-11 | `sem_resistencia_o_dano_passa_inteiro`, `cem_de_resistencia_corta_metade`, `vida_efetiva_dobra_com_cem_de_resistencia`, `alvo_sem_vida_tem_vida_efetiva_zero` | unidade | O multiplicador é 1 com resistência 0 e 0,5 com 100; 1000 de vida com 100 de resistência valem 2000 efetivos, e 0 de vida vale 0 com 200 | R-10, 03 §3 |
| T-JOGO-12 | `resistencia_negativa_amplifica_com_retorno_decrescente` | unidade | Resistência −100 multiplica o dano por 1,5, e −1000 fica abaixo de 2 | R-10 |
| T-JOGO-13 | `penetracao_plana_nao_negativa_a_resistencia`, `reducao_plana_negativa_a_resistencia`, `percentual_nao_mexe_em_resistencia_negativa`, `penetracao_percentual_acima_de_um_nao_inverte` | unidade | Com 30 de armadura, 50 de penetração plana deixam 0 e 50 de redução plana deixam −20, e 35% de penetração não mexem nesses −20; penetração de 500% sobre 100 não fica negativa | R-10, 03 §3 |
| T-JOGO-14 | `ordem_importa_percentual_antes_da_plana`, `reducao_vem_antes_da_penetracao_percentual`, `dano_apos_resistencia_bate_com_a_conta_manual` | unidade | Com 100 de armadura, 35% e 18 de letalidade deixam 47, não 53,3; 30 de redução e depois 35% deixam 45,5; 1000 de dano bruto com 35% e 18 chegam como 1000 × 100/147 | R-10, 03 §3 |
| T-JOGO-15 | `a_rota_aceita_os_dois_vocabularios` | regressão | `middle` e `Mid` dão `Mid`, `utility` e `Support` dão `Support`, `bottom` e `Bot` dão `Bot`, `TOP` e `Jungle` as suas; `radar` e o vazio dão `Unknown`. Antes, `Mid` virava `Unknown` e `set_draft_role("Mid")` apagava a escolha | R-1, R-4, A30 |

### 2.11 Build adaptativa — valor por ouro contra os itens deles

Fixture comum (`build.rs`): você é um lutador de teste no nível 11, sem itens;
os inimigos são um atirador (`AtiradorAD`) e um mago (`MagoAP`) no nível 11. Os
itens são Placas (80 de armadura), Manto (80 de RM) e Vitalidade (800 de vida),
a 2800; Perfurante (40 de AD e 35% de pen. de armadura), Lâmina (70 de AD) e
Bastão (120 de AP), a 3000.

| ID | Teste(s) | Tipo | O que prova | Atende |
|---|---|---|---|---|
| T-BUILD-1 | `time_inimigo_ad_puxa_armadura_acima_de_resistencia_magica`, `inimigo_comprando_ap_inverte_a_prioridade` | unidade | Contra dois atiradores com Lâmina, a Placas fica acima do Manto no ranking; contra dois magos com Bastão, o Manto fica acima da Placas | R-10, C-6 |
| T-BUILD-2 | `penetracao_inimiga_derruba_o_valor_da_armadura` | unidade | Os mesmos dois atiradores trocando a Lâmina pela Perfurante baixam o `value_per_1000g` da Placas: mesmo time, mesma divisão de dano, só a penetração muda | R-10, C-6, ADR-0015 |
| T-BUILD-3 | `contra_penetracao_percentual_vida_ganha_da_armadura` | unidade | A mesma dupla de atiradores sem penetração e com 35% de penetração de armadura (`enemy_armor_pen_percent` de 0 para mais de 0): a razão vida/armadura no valor por 1000 de ouro sobe. Antes se chamava "contra letalidade", não tinha letalidade, e a ordem saía igual sem penetração | R-10, A20 |
| T-BUILD-4 | `inimigo_com_armadura_valoriza_penetracao_para_mim` | unidade | Um atirador inimigo que troca a Lâmina pela Placas sobe o `value_per_1000g` da Perfurante na sua lista | R-10, C-6 |
| T-BUILD-5 | `sem_itens_inimigos_cai_no_prior_da_classe`, `share_anda_quando_o_inimigo_compra` | unidade | Sem itens, `incoming_physical_share` passa de 0,7 contra o atirador e fica abaixo de 0,5 contra o mago (o prior de classe); o mesmo atirador com Bastão faz a fração cair | R-10, C-6 |
| T-BUILD-6 | `item_que_ja_tenho_sai_da_lista` | unidade | Com a Placas no seu inventário, nenhuma sugestão tem o `item_id` dela | R-10 |
| T-BUILD-7 | `campeao_desconhecido_devolve_vazio_sem_panico`, `sem_inimigos_ainda_ranqueia` | unidade | O campeão 9999, fora do índice, dá lista vazia sem pânico; você conhecido sem nenhum inimigo dá lista não vazia | R-10 |
| T-BUILD-8 | `limite_e_respeitado_e_a_ordem_e_estavel` | propriedade | Com `limit = 3` saem exatamente 3 sugestões, e duas chamadas com a mesma entrada devolvem vetores iguais | R-10 |
| T-BUILD-9 | `a_razao_cita_o_numero_real` | unidade | Contra dois atiradores com Lâmina, alguma razão da Placas contém "físico", a frase com a porcentagem de dano físico deles | R-10 |
| T-BUILD-10 | `para_um_campeao_de_ad_a_lamina_fica_acima_do_bastao`, `para_um_mago_o_bastao_fica_acima_da_lamina` | regressão | Contra dois atiradores com a Lâmina, no nível 11: para o lutador (ataque 7, magia 3) a Lâmina (70 AD, 3000) vale mais por 1000 de ouro que o Bastão (120 AP, 3000); para o mago (ataque 2, magia 9), o contrário. Antes da correção, o lutador via o Bastão em primeiro | R-10, A4 |

### 2.12 Rotas, escaramuças e destaque

Fixture comum (`lanes.rs`): dez campeões de teste com os mesmos atributos base,
diferentes em classe, alcance e nos números de `info`; nível 11 e sem itens
quando o teste não diz outro. Itens: Chapéu de Rabadon (130 de AP) e Malha de
Espinhos (70 de armadura e 350 de vida). `time_completo` põe os cinco de cada
lado.

| ID | Teste(s) | Tipo | O que prova | Atende |
|---|---|---|---|---|
| T-ROTA-1 | `so_entram_rotas_com_os_dois_lados_conhecidos` | unidade | Você com meio e topo, eles só com meio: sai uma `LaneOutlook`, a do meio; o topo não vira 50% | R-11 |
| T-ROTA-2 | `sem_base_de_partidas_o_numero_vem_so_do_duelo` | unidade | Com `NoStats`: `from_data = false`, `sample = 0`, e nenhuma razão contém `n=` | RNF-1, R-11, ADR-0007 |
| T-ROTA-3 | `matchup_com_amostra_empurra_a_probabilidade` | unidade | Um matchup de 4000 partidas a 60%, sem winrate base, sobe `win_probability` acima da do `NoStats`, liga `from_data`, põe `sample = 4000` e a primeira razão cita `n=4000` | R-11, R-21, RNF-1 |
| T-ROTA-4 | `matchup_no_esperado_nao_mexe_na_rota` | regressão | Bases de 53% e 47% (40 000 partidas cada) e um matchup de 4000 partidas com 2239 vitórias, o que as duas forças já explicam: a rota muda menos de 0,002 contra o mesmo caso sem matchup, e a base sozinha já liga `from_data` | RNF-3, R-21, ADR-0008, L-8 |
| T-ROTA-5 | `itens_do_inimigo_derrubam_a_sua_rota`, `seus_itens_sobem_a_sua_rota` | unidade | No meio, o oponente com Malha de Espinhos baixa a sua `win_probability`; você com Rabadon a sobe | R-11 |
| T-ROTA-6 | `a_probabilidade_nunca_chega_ao_extremo` | unidade | Tanque nível 18 com Malha contra atirador nível 1 sem nada: `win_probability` fica entre `FLOOR` e `CEILING` (±1e−9) | R-11 |
| T-ROTA-7 | `a_rota_e_simetrica`, `a_escaramuca_e_simetrica`, `a_curva_tambem_e_simetrica` | propriedade | Trocar os lados soma 1 (±1e−9): na `win_probability` do meio, no `odds` de cada escaramuça de `time_completo` e em cada ponto de `by_level` | R-11 |
| T-ROTA-8 | `a_ordem_sai_sempre_top_jg_mid_adc_sup` | unidade | Entrada na ordem Support, Top, Bot, Mid, Jungle; a saída vem Top, Jungle, Mid, Bot, Support | R-11 |
| T-ROTA-9 | `sai_um_grupo_para_cada_briga_que_acontece` | unidade | Com `time_completo` sai uma escaramuça por grupo de `GROUPS`, entre elas "JG+TOP x JG+TOP" e "JG+ADC+SUP x JG+ADC+SUP", e `ours.len() == size` em todas | R-11 |
| T-ROTA-10 | `grupo_incompleto_fica_de_fora_em_vez_de_virar_2v1` | unidade | Sem o jungle deles, nenhum rótulo contém "JG" e "ADC+SUP x ADC+SUP" continua na lista | R-11 |
| T-ROTA-11 | `item_no_grupo_inimigo_derruba_a_sua_chance` | unidade | O top deles comprar Malha de Espinhos baixa o `odds` de "JG+TOP x JG+TOP" | R-11 |
| T-ROTA-12 | `as_razoes_nunca_dizem_onde_alguem_esta` | propriedade | Nenhuma razão de rota nem de escaramuça de `time_completo`, em minúsculas, contém "está em", "no mapa", "visto em", "posição" ou "caminhando" | C-4, R-11 |
| T-ROTA-13 | `o_objetivo_perto_decide_qual_briga_mostrar`, `o_mais_proximo_ganha_quando_ha_dois` | unidade | Dragão em 40 s com você no topo: destaque `dragao` e `because = "Dragão em 0:40"`; Barão em 25 s com você no bot: `arauto`; Barão em 80 s e Dragão em 15 s: `dragao` | R-15, R-11 |
| T-ROTA-14 | `objetivo_longe_nao_manda_na_escolha`, `cada_rota_puxa_a_propria_dupla` | unidade | Dragão em 400 s, fora da janela, com você no topo: `jg_top` e "a briga da sua rota"; sem objetivo, Top dá `jg_top`, Mid `jg_mid`, Bot e Support `bot` | R-15 |
| T-ROTA-15 | `o_jungler_cai_na_briga_mais_decidida` | unidade | Jungle sem objetivo: `because = "a briga mais decidida agora"`, e o \|`odds` − 0,5\| do destaque é o maior das cinco (±1e−9) | R-15 |
| T-ROTA-16 | `objetivo_ja_disponivel_muda_o_texto` | unidade | Dragão com `seconds_left = 0`: `because = "Dragão disponível"` | R-15 |
| T-ROTA-17 | `sem_escaramuca_nenhuma_nao_ha_destaque` | unidade | Lista de escaramuças vazia com Dragão em 10 s: `None` | R-15 |
| T-ROTA-18 | `objetivo_perto_sem_grupo_completo_cai_para_a_regra_seguinte` | unidade | Sem o jungle deles o grupo `dragao` não sai; Dragão em 20 s com você no bot dá `bot`, e o destaque não some | R-15 |
| T-ROTA-19 | `a_curva_cobre_os_quatro_niveis`, `a_curva_da_escaramuca_tambem_sai` | unidade | O `by_level` da rota tem exatamente os níveis de `BREAKPOINTS`, com `odds` em [0, 1]; cada escaramuça de `time_completo` tem 4 pontos em `by_level` | R-6, R-11 |
| T-ROTA-20 | `a_curva_nivela_os_dois_lados` | unidade | Nível 18 contra 1 e 7 contra 7 dão o mesmo `by_level`, e `win_probability` diferente | R-6, R-11 |
| T-ROTA-21 | `crescimento_diferente_move_a_curva` | unidade | Um mago com 160 de vida e 6 de AD por nível, contra o de teste com 100 e 3: `odds` no 16 maior que no 3 | R-6 |
| T-ROTA-22 | `a_curva_inteira_vai_do_1_ao_18` | unidade | `curve` tem os níveis 1 a 18, todos com `gap = 0` e `odds` em [0, 1] | R-6 |
| T-ROTA-23 | `um_nivel_a_frente_nunca_piora`, `a_vantagem_de_nivel_respeita_o_1_e_o_18` | propriedade | Em cada nível de `BREAKPOINTS`, a linha +1 ≥ a nivelada ≥ a −1; `level_gaps` devolve 12 linhas (4 × 3), todas com o seu nível em 1..=18 | R-6, R-11 |
| T-ROTA-24 | `o_ponto_de_agora_usa_os_niveis_reais` | unidade | Você no 9 e ele no 7: `(my_level, their_level) = (9, 7)` e `duel` acima do ponto 7 de `curve` | R-11 |
| T-ROTA-25 | `os_itens_continuam_valendo_na_curva` | unidade | Você com Rabadon: cada ponto de `by_level` acima do mesmo ponto sem o item | R-6, R-11 |
| T-ROTA-26 | `campeao_desconhecido_tira_a_rota_e_a_briga_da_lista` | regressão | Com o meio aliado fora do índice (999) e as duas selvas conhecidas, `lane_outlooks` devolve só a selva, e `skirmishes` não traz `jg_mid`. Antes da correção, o meio entrava no piso de 28% e a briga da selva com o meio também, decididos por falta de dado | R-11, RNF-1, A13 |

### 2.13 Ouro dos dez

| ID | Teste(s) | Tipo | O que prova | Atende |
|---|---|---|---|---|
| T-OURO-1 | `antes_de_um_e_cinquenta_nao_ha_renda_passiva`, `a_renda_passiva_comeca_a_contar_depois` | unidade | `earned_at(60, 0, 0, 0, 0)` é exatamente `STARTING`; de `PASSIVE_START` a `PASSIVE_START + 100` a estimativa sobe `100 × PASSIVE_PER_SECOND` (±1e−9) | R-12 |
| T-OURO-2 | `abates_e_assistencias_entram_na_renda` | unidade | Aos 900 s com 100 tropas, um abate soma `PER_KILL` e uma assistência soma `PER_ASSIST` (±1e−9) | R-12 |
| T-OURO-3 | `o_investido_e_um_piso_para_a_estimativa` | unidade | Aos 600 s com 10 tropas a estimativa fica abaixo de 9000; com 9000 investidos, `earned_at` devolve 9000 | R-12 |
| T-OURO-4 | `o_investido_e_soma_exata_e_a_vantagem_tambem` | unidade | Você (ORDER, local) com 6200 investidos e um inimigo (CHAOS) com 4100: `ours_invested = 6200`, `theirs_invested = 4100` e `invested_lead = 2100`, por igualdade exata | R-12 |
| T-OURO-5 | `o_ouro_parado_e_a_diferenca_entre_ganho_e_gasto` | unidade | Aos 1500 s, com 200 tropas, 5 abates, 5 assistências e 1000 investidos: `unspent > 0` e `earned − invested − unspent = 0` (±1e−9) | R-12 |
| T-OURO-6 | `avisa_quando_ha_ouro_sobrando_no_bolso`, `nao_avisa_quando_o_ouro_esta_todo_investido` | unidade | Aos 1800 s, com 250 tropas, 8 abates, 6 assistências e nada investido: uma nota contém "para gastar"; com 99 000 investidos aos 600 s, nenhuma nota assim e `unspent = 0` | R-12 |
| T-OURO-7 | `sem_jogador_local_nao_inventa_um_lado` | unidade | Dois jogadores, nenhum local: `ours_invested = 0`, `notes` vazio e ninguém com `is_ally` | R-12 |
| T-OURO-8 | `ouro_por_minuto_nao_explode_no_segundo_zero` | unidade | Com `game_time = 0` e só o ouro inicial, `gold_per_minute` é exatamente 30 000: 500 sobre o piso de 1 s no divisor. Antes o teste só conferia `is_finite` | R-12, A20 |
| T-OURO-9 | `aponta_o_inimigo_mais_adiantado` | unidade | Inimigos com 4000 e 8000 investidos: alguma nota cita o de 8000 (Viego) | R-12 |
| T-OURO-10 | `so_o_jogador_local_tem_ouro_em_mao` | regressão | Você (ORDER, local, 1000 investidos) e Zed (CHAOS, 2000 investidos, 150 tropas e 3 abates aos 1200 s): só o seu `unspent` existe; o de Zed é `None`, mesmo com a estimativa dele bem acima do investido. Antes da correção, Zed saía com `unspent = 4773,6`, e a tela o desenhava | C-4, R-12 |

### 2.14 Objetivos — timers, estruturas e buffs

Os testes de `core/src/engine/timeline.rs`. Os jogadores são `eu#BR1` e
`aliado#BR1` (`ORDER`, você é `eu`) e `inimigo#BR1` (`CHAOS`). Os tempos vêm de
`Timings::default()`, a não ser quando o teste troca um.

| ID | Teste(s) | Tipo | O que prova | Atende |
|---|---|---|---|---|
| T-OBJETIVO-1 | `sem_eventos_usa_as_constantes_e_marca_como_assumido`, `barao_sem_abate_usa_o_primeiro_spawn` | unidade | Sem evento nenhum, o dragão sai aos 300 s (faltam 240 s aos 60 s) e o Barão aos 1200 s, os dois com `observed = false` | R-13, RNF-1 |
| T-OBJETIVO-2 | `abate_observado_vira_soma_exata`, `barao_abatido_respawna_seis_minutos_depois` | unidade | Dragão morto aos 400 s volta aos 700 s (faltam 200 s aos 500 s), e Barão morto aos 1300 s volta aos 1660 s, os dois com `observed = true` | R-13, RNF-1 |
| T-OBJETIVO-3 | `alma_fechada_troca_o_pit_para_anciao` | unidade | Quatro dragões inimigos (300, 600, 900 e 1200 s) dão `soul = "enemy"`, tiram o `Dragão` da lista e põem o `Ancião` aos 1560 s | R-13 |
| T-OBJETIVO-4 | `atribui_o_dragao_ao_time_certo`, `abates_sao_separados_por_time`, `killer_pelo_nome_curto_tambem_resolve` | unidade | Um dragão de cada lado dá `dragons = {1, 1}` e `dragon_types = [Fire, Ocean]`. Abates de `aliado#BR1`, `eu#BR1` e `inimigo#BR1` dão `kills = {2, 1}`. O `killer` `aliado`, sem a tag, conta para o seu time | R-13, C-4 |
| T-OBJETIVO-5 | `torre_derrubada_por_minions_nao_conta_para_ninguem` | unidade | `TurretKilled` com `killer = Minion_T100` deixa `turrets = {0, 0}` | R-13 |
| T-OBJETIVO-6 | `arauto_e_grubs_somem_da_lista_depois_de_tomados`, `objetivo_que_ja_passou_da_janela_nao_aparece` | unidade | Com `HeraldKill` aos 920 s e `VoidgrubKill` aos 500 s, nem Arauto nem Grubs aparecem aos 1000 s. Sem evento, os Grubs (480 s) já não aparecem aos 1800 s | R-13 |
| T-OBJETIVO-7 | `tempo_restante_nunca_fica_negativo`, `lista_sai_ordenada_pelo_mais_proximo` | propriedade | Com um dragão morto aos 100 s e a partida aos 9999 s, todo `seconds_left` é ≥ 0. Aos 60 s sem eventos, `upcoming` sai em ordem crescente de `seconds_left` | R-13 |
| T-OBJETIVO-8 | `inibidor_volta_em_cinco_minutos_nos_dois_formatos_de_nome`, `a_rota_do_inibidor_sai_do_nome` | unidade | `Inhib_TChaos_L1` e `Barracks_T2_C1`, cada um caído aos 1500 s, dão um só `Respawn`: `Inibidor do meio`, `ours = false`, `at = 1800`, faltam 200 s aos 1600 s. `Inhib_TOrder_L2` e `Barracks_T1_L1` são o topo, e `Inhib_TOrder_L0` e `Barracks_T1_R1` o bot | R-13, P-14 |
| T-OBJETIVO-9 | `torre_do_nexo_volta_e_torre_comum_nao` | unidade | Das quatro torres caídas aos 1700 s, só `Turret_TOrder_L1_P5` e `Turret_T1_C_02_A` voltam: `Torre do nexo`, `ours = true`, `at = 1880`. `Turret_TOrder_L1_P2` e `Turret_T1_L_03_A` não entram | R-13, P-14 |
| T-OBJETIVO-10 | `estrutura_que_ja_voltou_sai_da_lista`, `nome_desconhecido_nao_vira_timer` | unidade | Inibidor caído aos 1000 s já não aparece aos 1300 s. `Algo_Novo_X` não vira `Respawn` | R-13, P-14 |
| T-OBJETIVO-11 | `buff_de_barao_e_de_quem_estava_vivo` | unidade | `BaronKill` de `eu` aos 1300 s, lido aos 1301 s com `aliado` morto: um buff `Barão`, `ours = true`, `until = 1480`, `holders = [10, 12]` (sem o 11, do `aliado`) | R-13, C-4 |
| T-OBJETIVO-12 | `quem_morre_perde_o_buff_mesmo_depois_de_renascer` | unidade | `ChampionKill` de `outro` aos 1340 s tira o 12 de `holders`, embora ele esteja vivo na leitura dos 1380 s: sobra `[10, 11]` | R-13 |
| T-OBJETIVO-13 | `buff_some_quando_ninguem_mais_tem_ou_quando_acaba` | unidade | Ancião de `inimigo` aos 2000 s: `ours = false`, `until = 2150`. Com `inimigo` morto aos 2010 s, a lista fica vazia, e continua vazia aos 2020 s com ele já renascido e o abate ainda no fluxo (A8: antes, o buff voltava com 130 s e `holders = None`). Um rastreador novo, lido aos 2001 s e de novo aos 2151 s, também termina vazio | R-13 |
| T-OBJETIVO-14 | `abate_visto_tarde_mostra_o_tempo_sem_inventar_quem_tem` | unidade | `BaronKill` aos 1300 s lido pela primeira vez aos 1340 s (40 s > 5 s): `holders = None`, `seconds_left = 140` | R-13, RNF-1 |
| T-OBJETIVO-15 | `dragao_comum_nao_da_buff_de_anciao` | unidade | `DragonKill` de `Fire` não cria buff | R-13 |
| T-OBJETIVO-16 | `mudar_a_constante_nao_quebra_a_logica` | unidade | Com `baron_first = 1500`, o Barão sai aos 1500 s: a lógica não depende do valor da constante | R-13 |
| T-OBJETIVO-17 | `nenhum_timer_e_de_habilidade` | arquitetura | Todo evento da Live Client, e um `SpellCast` inventado, passam pela timeline e pelos buffs: nenhuma chave do JSON que sai fala de recarga, suprema, feitiço ou habilidade, e todo timer é de objetivo (Dragão, Ancião, Barão, Arauto, Grubs) ou de estrutura (Inibidor, Torre do nexo) | C-1, R-13 |
| T-OBJETIVO-18 | `le_os_nomes_de_campo_dos_eventos` | fronteira | JSON de `/liveclientdata/eventdata` com `EventName`, `EventTime`, `DragonType`, `KillerName`, `VictimName`, `TurretKilled`, `InhibKilled`, `Result` e `Events`: cada campo chega com o valor escrito | R-13, A19 |

### 2.15 Prioridade

Os testes de `core/src/engine/priority.rs`. A partida base tem você na selva e, de
cada lado, um jogador no topo, no meio e no bot, todos vivos, com 100 tropas na
rota e 80 na selva.

| ID | Teste(s) | Tipo | O que prova | Atende |
|---|---|---|---|---|
| T-PRIORIDADE-1 | `sem_jogador_local_devolve_vazio` | unidade | Sem ninguém com `is_local`, `lanes` sai vazio | R-14 |
| T-PRIORIDADE-2 | `as_tres_rotas_aparecem_ordenadas` | unidade | A partida base dá três rotas, com `score` que não cresce de uma para a próxima: alternativas com motivo, nunca uma ordem só | R-14, C-5 |
| T-PRIORIDADE-3 | `inimigo_morto_na_rota_sobe_a_prioridade` | unidade | Com o topo inimigo morto por 30 s, o `score` do `TOP` sobe e uma razão diz `morto` | R-14 |
| T-PRIORIDADE-4 | `jungler_inimigo_morto_levanta_todas_as_rotas` | unidade | Com o jungler inimigo morto por 25 s, `TOP`, `MID` e `BOT` sobem | R-14 |
| T-PRIORIDADE-5 | `dragao_proximo_puxa_a_rota_de_baixo`, `barao_proximo_puxa_a_rota_de_cima_e_nao_a_de_baixo` | unidade | Dragão a 20 s sobe o `BOT` e entra nas razões dele. Barão a 10 s entra nas razões do `TOP` e em nenhuma do `BOT` | R-14 |
| T-PRIORIDADE-6 | `morto_nao_entra_na_escaramuca` | unidade | Com o topo inimigo morto, a luta do `TOP` é o aliado do topo mais você contra ninguém: a primeira razão começa com `2v0` e `skirmish_odds = 1.0` | R-14 |
| T-PRIORIDADE-7 | `nenhuma_razao_revela_posicao` | propriedade | Com o jungler inimigo morto e o Dragão a 15 s, nenhuma razão das três rotas contém `está no` nem `posição`, em minúsculas. É a trava da regra da Riot contra informação que o jogador não teria | C-4, R-14 |
| T-PRIORIDADE-8 | `foco_marca_quando_o_numero_e_estimado` | unidade | Barão a 40 s com `observed = false` dá um `focus` com `estimado` | R-14, R-13, RNF-1 |
| T-PRIORIDADE-9 | `objetivo_distante_nao_vira_foco` | unidade | Barão a 400 s (> 90 s) deixa `focus = None` | R-14 |
| T-PRIORIDADE-10 | `placar_fica_na_faixa` | propriedade | Com o jungler e o topo inimigos mortos por 90 s, o topo deles com 0 tropas e o Barão a 0 s, todo `score` fica em [0, 100] | R-14 |
| T-PRIORIDADE-11 | `todo_ponto_da_prioridade_tem_razao` | regressão | Na rota de baixo com o atirador deles morto, o jungler deles morto, 30 de CS à frente e o dragão em 20 s, saem as cinco razões, e a primeira é a briga terminando em "para o seu lado". Antes da correção saíam três ("2v0 a seu favor em 100%", o morto, o jungler): o CS e o dragão somavam ponto sem razão | R-14, A23 |

### 2.16 Pós-partida

Os testes de `core/src/engine/coach.rs`. As gravações têm um quadro por segundo.
O molde `reta` é empatado: você (`Ahri`) e o oponente direto (`Zed`) com
`t / 8` tropas, o mesmo nível e `3t` de ouro em itens, no meio. Cada teste muda
só o que diz.

| ID | Teste(s) | Tipo | O que prova | Atende |
|---|---|---|---|---|
| T-POS-1 | `quadro_com_relogio_parado_ou_voltando_e_descartado` | unidade | Quadros aos 100, 100, 90 e 101 s dão uma gravação de 2 quadros | R-16 |
| T-POS-2 | `sem_gravacao_nao_ha_relatorio` | unidade | Gravação vazia: nenhuma observação, e o `summary` diz `Sem gravação` | R-16, RNF-1 |
| T-POS-3 | `partida_curta_nao_rende_conclusao`, `partida_curta_nao_rende_curva_nenhuma` | unidade | 120 s gravados (< 300 s): nenhuma observação, `summary` com `curta`, e `series` vazio | R-16, RNF-1 |
| T-POS-4 | `farm_atras_do_rival_vira_ponto_negativo`, `farm_na_frente_vira_ponto_positivo` | unidade | Em 30 min no meio, `t / 12` tropas contra `t / 7` (5,0 contra 8,6 por minuto) dá uma observação ruim de `tropas por minuto`. `t / 6` contra `t / 12` (10 contra 5) dá uma boa | R-16 |
| T-POS-5 | `suporte_nao_e_cobrado_por_farm` | regressão | Suporte com 1,0 tropa por minuto contra 2,5 do outro suporte: nenhuma observação, boa ou ruim, fala de `tropas` | R-16, L-27 |
| T-POS-6 | `suporte_e_cobrado_pela_visao` | regressão | O mesmo suporte, com 1,2 de visão por minuto contra 2,0: sai uma observação ruim de `visão` | R-16, L-27 |
| T-POS-7 | `a_curva_do_suporte_e_a_de_visao` | regressão | O relatório do suporte tem `role = Support` e pelo menos 2 pontos em `vision_diff`, e o último é negativo | R-16, L-27 |
| T-POS-8 | `participacao_cobra_mais_de_quem_vive_dela` | regressão | Em 9 dos 20 abates do time (45%), o suporte (≤ 50%) recebe a observação ruim de `abates`, e o meio (entre 40% e 65%) não | R-16, L-27 |
| T-POS-9 | `participacao_baixa_nos_abates_e_apontada` | unidade | No meio, com 0 dos 20 abates do time no último quadro, sai a observação ruim `dos abates do time` | R-16 |
| T-POS-10 | `leitura_salva_antes_da_rota_ainda_carrega` | unidade | Um `ultima-partida.json` sem `role` e sem `visionDiff` carrega com `role = Unknown` e `vision_diff` vazio | R-16 |
| T-POS-11 | `tempo_morto_sai_do_intervalo_real_entre_quadros`, `buraco_grande_na_gravacao_nao_conta_como_tempo_morto` | unidade | 60 s morto em 1800 s (3,3% < 12%) não vira `esperando`, e morto em metade dos blocos de 30 s depois dos 600 s vira. Morto nos quadros dos 0 e 300 s, com 300 s de buraco entre eles (> 10 s), não conta | R-16 |
| T-POS-12 | `partida_sem_morte_vira_elogio` | unidade | Zero mortes em 30 min dá a observação boa `Nenhuma morte` | R-16 |
| T-POS-13 | `mortes_em_sequencia_viram_uma_janela`, `mortes_espalhadas_nao_viram_janela` | unidade | Mortes aos 300, 840, 870, 900 e 960 s dão a janela `4 mortes entre …`. Uma morte a cada 400 s não dá janela | R-16 |
| T-POS-14 | `item_grande_tardio_e_apontado`, `item_barato_nao_conta_como_item_grande` | unidade | Item de 3600 de ouro comprado aos 1200 s (≥ 900 s) dá a observação ruim `Primeiro item grande`. Só o de 400 não dá observação, boa nem ruim | R-16 |
| T-POS-15 | `vantagem_que_sumiu_e_apontada` | unidade | O ouro em itens do time chega a 6000 de vantagem aos 1200 s e volta a 0 aos 1800 s: sai a observação ruim `de vantagem aos` | R-16 |
| T-POS-16 | `nivel_atras_do_rival_e_apontado` | unidade | Nível 10 contra 18 no fim dá a observação ruim `níveis atrás` | R-16 |
| T-POS-17 | `as_observacoes_saem_da_mais_pesada_para_a_menos` | propriedade | Com tropas, mortes, tempo morto e nível atrás, saem pelo menos 2 observações ruins, com `weight` que não cresce de uma para a próxima | R-16 |
| T-POS-18 | `as_curvas_saem_reamostradas_e_nao_quadro_a_quadro` | unidade | 1801 quadros em 1800 s dão 61 pontos em `gold_lead` (um a cada 30 s), e `cs_diff` tem o mesmo tamanho | R-16 |
| T-POS-19 | `a_curva_de_ouro_acompanha_a_vantagem_real` | unidade | Com `6t` contra `t` de ouro em itens, `gold_lead` sobe e termina em 9000 ±1 (10800 − 1800) | R-16 |
| T-POS-20 | `as_mortes_entram_com_o_instante_exato_nao_reamostradas` | unidade | Uma morte aos 907 s dá `deaths = [907]`, não o múltiplo de 30 mais próximo | R-16 |
| T-POS-21 | `a_curva_de_tropas_mede_a_diferenca_contra_o_oponente` | unidade | `t / 6` contra `t / 12` tropas: `cs_diff` começa com menos de 5 de diferença e termina acima de 100 | R-16 |
| T-POS-22 | `os_abates_do_time_contam_uma_vez_cada` | unidade | Um abate a cada 300 s em 1800 s dá 6 instantes em `team_kills` | R-16 |
| T-POS-23 | `gravacao_que_comeca_tarde_usa_o_relogio_da_partida` | regressão | Gravação dos 20:00 aos 30:00, com 270 tropas contra 300 e o Rabadon já no primeiro quadro: `duration = 1800`, a nota de farm é "9.0 tropas por minuto contra 10.0" e nenhuma nota de primeiro item grande. Antes da correção, `duration = 600` e a taxa saía sobre 10 minutos (27 contra 30) | R-16, A9 |
| T-POS-24 | `so_partida_de_verdade_vale_leitura` | unidade | `worth_reviewing` é falso para um remake de 3:00 (181 quadros) e para 30 quadros no fim de uma partida de 30 minutos, verdadeiro para 5:00 de partida gravada, e falso para a gravação vazia | R-16, A6 |
| T-POS-25 | `o_pos_jogo_nao_da_nota` | arquitetura | Leitura de 30 minutos com farm atrás do rival: nenhuma chave com um número sozinho se chama nota, rating, MMR, elo, score, overall, tier ou rank; o `weight` de cada observação existe e só ordena | C-7, RNF-11 |

### 2.17 Casca — fonte da análise, descoberta do cliente, atualizador e gravação

Os testes de `src-tauri`, todos em `mod tests`: `analysis.rs`, `lcu/credentials.rs`,
`lib.rs` e `recorder.rs`. Rodam com `cargo test --workspace` no CI do Windows, e
pedem a árvore do Tauri para compilar ([08 §4](08-desenvolvimento-e-testes.md#4-os-testes)).

| ID | Teste(s) | Tipo | O que prova | Atende |
|---|---|---|---|---|
| T-CASCA-1 | `a_partida_em_andamento_ganha_do_draft`, `partida_sem_jogador_local_cai_para_o_draft` | unidade | Com partida (você no meio com o campeão 1, nível 12) e draft (7 contra 8), `report` usa a partida: `source = "partida"` e `lanes[0].ally = 1`. Se a partida não tem jogador local, cai para o draft: `source = "draft"` | R-11, R-6 |
| T-CASCA-2 | `sem_partida_usa_o_draft` | unidade | Sem partida, com 7 contra 8 no meio, a análise sai do draft: `source = "draft"`, `roles_assumed = false`, `lanes[0].ally = 7` | R-6 |
| T-CASCA-3 | `rota_inimiga_em_branco_e_presumida_e_marcada` | unidade | Topo, selva e meio contra três inimigos sem posição rendem três rotas, com `roles_assumed = true` para a interface avisar | R-4, L-25, P-13 |
| T-CASCA-4 | `campeao_ainda_nao_travado_fica_de_fora` | unidade | Um slot do topo com `championId = 0` ao lado do meio 7 contra 8 rende uma rota só: slot vazio não vira rota | R-6 |
| T-CASCA-5 | `sem_draft_e_sem_partida_nao_ha_resposta`, `draft_so_de_um_lado_nao_rende_analise` | unidade | Sem partida e sem draft, `report` devolve `None`; com draft só do seu lado (meio 7, nenhum inimigo), também `None` | R-6, RNF-1 |
| T-CASCA-6 | `antes_da_partida_todos_entram_no_mesmo_nivel` | unidade | No draft (topo e meio contra topo e meio), nenhuma razão de rota cita "Nível": antes da partida ninguém tem vantagem de nível inventada | R-6 |
| T-CASCA-7 | `o_time_completo_rende_as_escaramucas` | unidade | Cinco contra cinco nas posições do LCU (`top` … `utility`) rendem 5 rotas, pelo menos 4 escaramuças e toda chance em [0, 1] | R-6 |
| T-CASCA-8 | `acha_caminhos_num_json_com_barras_escapadas`, `acha_caminho_em_yaml_sem_aspas` | fronteira | O caminho do JSON da Riot com barras escapadas (`"C:\\Riot Games\\League of Legends\\"`) sai como `C:\Riot Games\League of Legends`; o do YAML sem aspas (`D:\Jogos\League of Legends`) também, sem que o `:` da chave corte o caminho | R-18 |
| T-CASCA-9 | `acrescenta_a_subpasta_do_jogo` | unidade | `"C:\\Riot Games"` rende as duas pastas: `C:\Riot Games` e `C:\Riot Games\League of Legends` | R-18 |
| T-CASCA-10 | `texto_sem_caminho_nao_rende_nada`, `nao_confunde_url_com_caminho` | unidade | `{}`, texto vazio, texto sem `X:\` e `"https://riotgames.com/algo"` não rendem caminho nenhum | R-18 |
| T-CASCA-11 | `os_caminhos_conhecidos_entram_sempre` | integração | As três pastas de `KNOWN_DIRS` estão em `candidate_dirs()` qualquer que seja a máquina, com ou sem configuração da Riot | R-18 |
| T-CASCA-12 | `sem_cliente_rodando_a_descoberta_falha_com_mensagem_util` | integração | Sem lockfile nas pastas conhecidas, o erro de `discover()` contém "League of Legends" em vez de um "None". Com o League instalado numa pasta conhecida, o teste retorna cedo | R-18 |
| T-CASCA-13 | `reconhece_a_mensagem_real_do_tauri`, `reconhece_outras_falhas_do_mesmo_plugin` | fronteira | A mensagem copiada de um crash (endpoint `http://` numa release, `PluginInitialization("updater", …)`) e três variantes (campo `pubkey` faltando, "failed to initialize plugin updater", "Error deserializing 'plugins.updater'") são reconhecidas: o app reabre sem o atualizador | R-19, RNF-7, ADR-0005 |
| T-CASCA-14 | `nao_engole_erro_que_nao_e_do_atualizador` | unidade | Falha do `global-shortcut`, WebView2 ausente, janela que não abre e texto vazio não são do atualizador: o app cai com pânico em vez de reabrir calado | R-19 |
| T-CASCA-15 | `o_resultado_sai_do_evento_de_fim_de_jogo`, `derrota_tambem_e_lida` | fronteira | Sem evento, `won = None`. `GameEnd` com `result = "Win"` dá `Some(true)`, e com `"Lose"`, `Some(false)` | R-16 |
| T-CASCA-16 | `evento_que_nao_e_fim_de_jogo_nao_decide_resultado`, `o_resultado_nao_e_reescrito_depois_de_conhecido` | unidade | `DragonKill` deixa `won = None`; um `Lose` depois de um `Win` não troca o `Some(true)` | R-16 |
| T-CASCA-17 | `a_gravacao_tem_teto` | unidade | 6500 quadros observados, com o relógio andando, guardam 6000 (`MAX_FRAMES`) | R-16, RNF-8 |
| T-CASCA-18 | `partida_curta_nao_rende_relatorio_mas_limpa_a_gravacao` | unidade | Com 10 quadros, abaixo de `MIN_FRAMES` (60), `finish` devolve `None` e a gravação fica vazia para a próxima partida | R-16 |
| T-CASCA-19 | `remake_de_tres_minutos_nao_rende_leitura_nem_apaga_a_anterior` | regressão | 181 quadros (0 a 3:00): `finish` devolve `None` e a gravação fica vazia. Antes, passava do piso de 60 quadros e gravava um "curta demais" por cima de `ultima-partida.json`, com o aviso que traz a aba Pós-jogo | R-16, A6 |
| T-CASCA-20 | `perder_o_cliente_avisa_a_tela_e_espera_antes_de_procurar_de_novo` | regressão | Com a leitura da fase falhando ("conexão recusada"): o draft é encerrado com aviso (não apagado calado), sai um `ConnStatus` desconectado com `phase = "Disconnected"` e o erro em `detail`, e a próxima volta espera `POLL_DISCONNECTED`. Antes, voltava na hora, sem aviso nem status | R-18, RNF-4, A5, A11 |
| T-CASCA-21 | `escrita_que_morre_no_meio_deixa_o_arquivo_anterior_inteiro`, `escrita_que_termina_troca_o_arquivo_e_nao_deixa_resto` | regressão | Com `pool.json` = `[1, 2, 3]` e o temporário impossível de criar, a escrita de `[9, 9]` volta como erro e o arquivo continua `[1, 2, 3]`; a escrita que termina troca o arquivo e não deixa `pool.json.parcial`. Antes, a escrita ia direto no arquivo final | R-9, R-15, A7 |
| T-CASCA-22 | `o_ritmo_segue_a_fase` | unidade | `ritmo` dá 350 ms no champ select, 1 s na partida e 1,5 s em qualquer outra fase; sem cliente, os 3 s vêm de `perdeu_o_cliente` (T-CASCA-20) | R-18, RNF-4, B7 |
| T-CASCA-23 | `o_registro_comeca_do_zero_a_cada_abertura` | unidade | Com um `presage.log` da abertura anterior na pasta, `abrir_log` devolve o arquivo vazio | A14 |
| T-CASCA-24 | `a_casca_nunca_escreve_no_lcu_nem_em_api_nenhuma` | arquitetura | Nenhum fonte da casca tem `.post(`, `.put(`, `.patch(`, `.delete(` nem `Method::` de escrita | R-18, B10 |
| T-CASCA-25 | `a_casca_so_fala_com_a_riot_e_com_a_propria_maquina` | arquitetura | Todo texto `http(s)://` e `ws(s)://` da casca é do loopback ou de um domínio da Riot (`riotgames.com`, `leagueoflegends.com`), e o da Riot Web API está lá | C-2, B2 |
| T-CASCA-26 | `nada_le_memoria_do_jogo_nem_injeta_input` | arquitetura | Nenhum fonte da casca chama `ReadProcessMemory`, `SendInput`, `SetWindowsHookEx` e afins, e o `Cargo.lock` não tem `enigo`, `rdev`, `inputbot`, `read-process-memory` nem outra crate de input ou memória | C-2, C-3, B3 |
| T-CASCA-27 | `iniciar_com_o_windows_so_quando_o_jogador_liga` | arquitetura | `lib.rs` tem um `.enable()` só, e dentro de `set_autostart` | R-19, B11 |
| T-CASCA-28 | `fechar_a_janela_principal_esconde_em_vez_de_destruir` | arquitetura | O `on_window_event` da casca pega `CloseRequested` da `main`, chama `prevent_close` e esconde a janela | R-19, A16 |
| T-CASCA-29 | `cada_janela_tem_so_o_que_precisa` | arquitetura | Nas capabilities, o overlay não tem permissão de `updater:` nem de `process:`, e a janela principal tem o atualizador | A33 |
| T-CASCA-30 | `uma_carga_que_falha_diz_por_que_e_nao_apaga_o_indice` | unidade | Uma carga de campeões com erro `"Data Dragon e cache"` sobre `"sem rede"` deixa `ready` falso e `champions_error` = `"Data Dragon e cache: sem rede"`, sem tocar em `items_error`; uma falha depois de um índice da 16.21.1 mantém a versão, e uma carga que dá certo limpa o motivo | R-1, R-10, RNF-1, A32 |
| T-CASCA-31 | `o_overlay_arrasta_pela_cabeca_e_so_pega_clique_quando_mostra_algo` | arquitetura | `capabilities/overlay.json` tem `core:window:allow-start-dragging`; a tag de `className="hud-head"` em `src/overlay.tsx` tem `data-tauri-drag-region`; `overlay.rs` chama `atravessa_o_mouse(` e tem um `set_ignore_cursor_events(` só. Falhou antes da A15 pela permissão | R-15, A15 |
| T-CASCA-32 | `cada_janela_so_alcanca_os_comandos_que_usa` | arquitetura | Os comandos de `generate_handler!` em `lib.rs` são os de `COMANDOS` em `build.rs`, que vai para `.commands(COMANDOS)`; os `allow-…` de `main.json` são os mesmos 26; os de `overlay.json` são os comandos que `src/overlay.tsx` chama. Falhou antes da A52: o `build.rs` não tinha manifesto | A52 |

## 3. Rodando um teste

```powershell
cargo test -p presage-core popularidade_nao_vira_matchup   # um só, pelo nome
cargo test -p presage-core                                 # o core, em segundos
cargo test --workspace                                     # tudo, no Windows
```

O nome da linha do catálogo é o argumento: a coluna **Teste(s)** é o que se passa
para o `cargo test`.

## 4. O teste que consegue falhar

Um teste verde não prova nada se ele também passaria com o código errado. Para uma
linha do core que importa, o mutante prova que a asserção pega:

```bash
python3 tools/dev/mutant.py core/src/engine/stats.rs "OLD" "NEW" -- cargo test -p presage-core NOME
```

Ele aplica a troca, roda o teste, restaura o arquivo byte a byte e imprime
`MUTANT KILLED` ou `MUTANT SURVIVED`. Morto por erro de compilação não conta.
Os docs são provados do mesmo jeito: mudar uma assinatura no 11, um campo em
`src/lib/types.ts` ou um nome de teste aqui tem de fazer
`npm run doccheck -- -strict` falhar.
