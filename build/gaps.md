# Lacunas G-1 a G-12: recibos

O recibo de cada lacuna do Presage frente ao iTero
([07 §6](../docs/referencia/itero/07-cruzamento.md#6-as-lacunas-que-cabem-por-valor)):
o commit, o código, o teste que a prova e o T-ID dele no
[catálogo de testes](../docs/arquitetura/13-testes.md).

Os commits foram lidos com `git log --oneline` e `git show --stat`. As oito
lacunas feitas entraram em três commits de 2026-10-28:

| Commit | O que traz |
|---|---|
| `2223e23` | o core do draft: rota marcada (`context.rs`), lista da conta no contexto (`context.rs`), sua lista de bans (`minimax.rs`, `pool.rs`) |
| `33150d7` | o core da partida: sua rota ao vivo (`rival.rs`), estruturas e buffs (`timeline.rs`) |
| `7e50a94` | a casca e a tela das oito, com os docs: `selectable` e `DraftState::normalized` (`draft.rs`), os comandos novos (`lib.rs`), o simulador, a bandeja e o iniciar com o Windows |

**Mutantes (2026-11-07, WSL, `bash tools/dev/core-test.sh <teste>`).** Cada
lacuna com teste tem uma linha do core trocada, e o teste dela tem de falhar por
asserção, não por compilação:

| # | Troca | Teste | Resultado |
|---|---|---|---|
| G-1 | `enemies[i].role = role` → `Role::Unknown` (`context.rs`) | `a_rota_marcada_vence_a_inferida` | morto (`context.rs:530`) |
| G-2 | `Some(Metric::Vision)` → `Some(Metric::Farm)` (`rival.rs`) | `suporte_ao_vivo_e_comparado_pela_visao_e_nunca_por_tropas` | morto (`rival.rs:167`) |
| G-3 | `timings.inhibitor_respawn` → `+ 1.0` (`timeline.rs`) | `inibidor_volta_em_cinco_minutos_nos_dois_formatos_de_nome` | morto (`timeline.rs:624`) |
| G-4 | `!p.is_dead && !morreu` → `!p.is_dead` (`timeline.rs`) | `quem_morre_perde_o_buff_mesmo_depois_de_renascer` | morto (`timeline.rs:716`) |
| G-5 | sem `!do_time.contains(&id)` (`minimax.rs`) | `nao_bane_o_que_o_time_quer_pegar_nem_o_que_ja_saiu` | morto (`minimax.rs:565`) |
| G-6 | sem `!RANKED_QUEUES.contains(&q)` (`draft.rs`) | `na_ranqueada_a_rotacao_gratis_nao_vale` | morto (`draft.rs:387`) |
| G-7 | `normalized` sem refazer `unavailable` (`draft.rs`) | `draft_montado_a_mao_ganha_indisponiveis_e_rota` | morto (`draft.rs:421`) |
| G-8 | — | nenhum teste | não há o que matar: a casca não compila no WSL, e a bandeja não tem teste |

A primeira rodada deu seis sobreviventes falsos: o cargo rodava o binário velho
([L-28](../docs/arquitetura/09-licoes-do-prototipo.md)).

## As feitas

| # | Lacuna | Commit | Código | Teste | T-ID |
|---|---|---|---|---|---|
| G-1 | Corrigir à mão a rota de cada inimigo | `2223e23` (core), `7e50a94` (casca e tela) | `core/src/engine/context.rs` (`Leitura::enemy_roles`), `set_enemy_role` em `src-tauri/src/lib.rs`, seletor em `src/components/DraftBoard.tsx` | `a_rota_marcada_vence_a_inferida`, `a_rota_marcada_tira_a_rota_dos_outros` | T-DRAFT-30, T-DRAFT-31 |
| G-2 | Sua rota ao vivo: o que a rota cobra contra o oponente direto, e atrás, igual ou à frente em ouro de itens | `33150d7` (core), `7e50a94` (tela) | `core/src/engine/rival.rs`, `src/components/LiveOverview.tsx` | `suporte_ao_vivo_e_comparado_pela_visao_e_nunca_por_tropas` | T-RIVAL-1 |
| G-3 | Timer de inibidor e de torre do nexo | `33150d7` | `core/src/engine/timeline.rs` (`Respawn`) | `inibidor_volta_em_cinco_minutos_nos_dois_formatos_de_nome` | T-OBJETIVO-8 |
| G-4 | Buff de Barão e Ancião: tempo restante e quem ainda tem | `33150d7` | `core/src/engine/timeline.rs` (`Buff`, `BuffTracker`) | `quem_morre_perde_o_buff_mesmo_depois_de_renascer` | T-OBJETIVO-12 |
| G-5 | Sua lista de bans | `2223e23` (core), `7e50a94` (tela) | `bans_with_pool` em `core/src/engine/minimax.rs`, `Pool::bans` e `BAN_POOL_MAX` em `core/src/pool.rs`, "Seus bans" em `src/components/PoolBuilder.tsx` | `nao_bane_o_que_o_time_quer_pegar_nem_o_que_ja_saiu` | T-DRAFT-59 |
| G-6 | Mostrar só os campeões que a conta pode escolher | `2223e23` (contexto), `7e50a94` (`selectable`, casca e tela) | `selectable` em `core/src/draft.rs`, `DraftContext` em `core/src/engine/context.rs`, `set_draft_prefs` em `src-tauri/src/lib.rs` | `na_ranqueada_a_rotacao_gratis_nao_vale`, `so_entra_quem_a_sua_conta_pode_escolher` | T-ENTRADA-11, T-DRAFT-34 |
| G-7 | Simulador de draft | `7e50a94` | `set_simulated_draft` em `src-tauri/src/lib.rs`, `DraftState::normalized` em `core/src/draft.rs`, `src/components/SimuladorDraft.tsx` | `draft_montado_a_mao_ganha_indisponiveis_e_rota` | T-ENTRADA-13 |
| G-8 | Bandeja e iniciar com o Windows | `7e50a94` | `bandeja`, `get_autostart` e `set_autostart` em `src-tauri/src/lib.rs` | nenhum: a casca só compila no CI do Windows, e a bandeja não tem teste | — |

O que estes testes não pegam, e está aberto no `build/audit.md`:
- G-4: o buff que sumiu volta como "sem saber quem tem" na leitura seguinte (A8);
- G-2: a margem de ouro não é simétrica (A24);
- G-1: a marca vive em dois lugares, e `get_enemy_roles` nunca é chamado (A2);
- G-8: fechar a janela principal provavelmente não tem volta pela bandeja (A16).

## As que ficam

G-9 a G-12 estão abertas. O porquê e o custo de cada uma estão no
[07 §6](../docs/referencia/itero/07-cruzamento.md#6-as-lacunas-que-cabem-por-valor).

| # | Lacuna | Espera |
|---|---|---|
| G-9 | Estilo de jogo da rota por faixa | base e timeline ([P-3](../docs/produto/perguntas-em-aberto.md)) |
| G-10 | Próxima habilidade a subir | base e timeline |
| G-11 | Scouting dos dez | chave de produção e o backend (C-10, ADR-0004); a P-32 pergunta se vira requisito |
| G-12 | Idiomas, tutorial, várias contas, PDL, instância única | decisão de produto (P-32); os títulos saíram daqui (P-31) |
