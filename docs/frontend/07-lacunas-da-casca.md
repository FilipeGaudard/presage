# 07 — Lacunas da casca

Descrever as telas mostrou o que o contrato da casca
([12](../arquitetura/12-contrato-da-casca.md)) ainda não entrega, ou entrega de
um jeito que faz a tela errar. Cada lacuna diz por quê, a proposta e o que vale
enquanto isso. A prioridade:
- **P0** faz a tela mentir ou contraria uma regra da Riot;
- **P1** faz a tela mostrar dado velho ou incompleto num caso comum;
- **P2** é duplicação ou folga que ainda não errou.

Uma lacuna fecha com o teste do [06](06-testes.md) que a prova.

O princípio que gera a maioria delas é o do [00 §4](00-visao-e-escopo.md#4-princípios-de-interface):
a casca calcula e a tela mostra. Quando a tela precisa saber de algo, isso tem
de vir da casca, e não ser adivinhado ou copiado no TypeScript.

## Resumo

| # | Lacuna | Prioridade | Estado |
|---|---|---|---|
| GC-1 | A falha de um comando não chega à tela | P0 | parcial: o front fechou (TF-POOL-4, TF-CASCA-4); a forma do erro e o evento de fundo ficam (A14) |
| GC-2 | O ouro em mão dos outros nove cruza a ponte e aparece | P0 | fechada (TF-PARTIDA-3, T-OURO-10) |
| GC-3 | Perder o cliente não limpa o draft na tela | P1 | fechada (T-CASCA-20) |
| GC-4 | Marcas e rota escolhida vivem em dois lugares; `get_enemy_roles` sem uso | P1 | parcial: as marcas voltam depois de recarregar (TF-DRAFT-11); a rota escolhida, não |
| GC-5 | A rota que o motor usou não volta para a tela | P1 | aberta |
| GC-6 | Dois vocabulários de rota na mesma ponte | P1 | parcial: os comandos aceitam os dois (T-JOGO-15); unificar a ponte fica |
| GC-7 | Motor sem campeões fica "carregando" para sempre | P1 | parcial: campeões e itens dizem o motivo (A32); runas ainda falham só no log |
| GC-8 | O overlay não arrasta e não diz o próprio estado | P1 | parcial: arrasta pela cabeça e não come clique vazio (A15); posição, estado e atalhos ficam |
| GC-9 | Regras do core copiadas no front | P1 | aberta |
| GC-10 | Rótulos escritos dos dois lados | P2 | aberta |
| GC-11 | O número de seções do overlay escrito à mão | P2 | aberta |
| GC-12 | O Data Dragon baixado duas vezes | P2 | aberta |
| GC-13 | Uma capability para as duas janelas | P2 | parcial: uma capability por janela (T-CASCA-29); o manifesto de comandos fica (A52) |

## GC-1 — A falha de um comando não chega à tela (P0)

**Por quê:**
- `App.tsx` tem 22 `.catch(() => void 0)` e `overlay.tsx` tem 5
  ([03 §7](03-estado-e-atualizacao.md#7-erros-hoje)).
- `set_pool`, `set_overlay_prefs` e `set_draft_prefs` mudam a tela antes da
  resposta. A casca grava em disco primeiro e só então troca a memória
  (`lib.rs:420-424`), então uma falha de disco deixa a tela com o valor novo e o
  motor com o velho. A casca também não registra o erro: ele volta como `String`
  e o front o descarta.
- Os três comandos do ranking vão num `Promise.all`; um que rejeita mantém na
  tela as listas da leitura anterior.
- Erro de comando é `Result<(), String>`: texto livre, sem código para a tela
  decidir o que dizer (a forma de hoje está no 12 §1, "Erros").
- O que falha fora de comando (Data Dragon, histórico pessoal, registro de
  atalho) só vai para o log.

**Proposta:**
- Casca: erro de comando com forma fixa, `{ code, message }`, com códigos como
  `disco` ou `sem-motor`; e um evento `presage://erro` para as falhas de fundo.
- Front: um único `chamar(comando, args, rótulo)` em `App.tsx` que guarda a
  última falha por rótulo e mostra uma faixa ("Não consegui gravar a pool: …").
  Uma escrita otimista que rejeita volta ao valor anterior.
- Prova: TF-CASCA-4, TF-POOL-4.

> **O front fechou.** Como proposto: `chamar(rótulo, comando, args)` em
> `App.tsx`, uma faixa por rótulo, escrita otimista que volta ao valor anterior,
> e cada leitura que falha sai da tela em vez de ficar velha. O overlay ganhou o
> mesmo cuidado sem faixa (cada seção por si). Prova: TF-POOL-4 e TF-CASCA-4,
> vermelhos antes. **Fica:** o erro com forma fixa (`{ code, message }`) e o
> evento `presage://erro` para as falhas de fundo, com o log que o jogador
> consegue abrir (A14).

## GC-2 — O ouro em mão dos outros nove cruza a ponte e aparece (P0)

**Por quê:**
- C-4 e R-12: "Ouro em mão de outro jogador nunca aparece".
- `GoldReport.players` traz `earned`, `unspent` e `goldPerMinute` estimados para
  os dez (`gold.rs:97-117`).
- `GoldTracker.tsx:70-74` mostra `+N` ao lado de qualquer jogador com
  `unspent ≥ 800`, inimigos inclusive, com o `title` "Estimativa do ouro em mão",
  e o rodapé explica que "o dos outros nove é reconstruído".

**Proposta:**
- Casca: `GoldPlayer` só traz `earned`, `unspent` e `goldPerMinute` do jogador
  local (`Option`, `None` para os outros). O dado proibido não cruza a ponte, e a
  regra não depende de a tela lembrar dela.
- Front: o `+N` e a frase do rodapé saem para os outros nove.
- Prova: TF-PARTIDA-3.

> **Fechada.** Como proposto, nos dois lados: `GoldPlayer::unspent` virou
> `Option<f64>`, `Some` só para o jogador local, e `GoldTracker` só desenha o
> `+N` na linha `isLocal`. O rodapé foi reescrito. `earned` e `goldPerMinute`
> continuam saindo para os dez, porque os totais por time precisam deles, e
> nenhuma tela os mostra por jogador. Prova: T-OURO-10 e TF-PARTIDA-3, os dois
> vermelhos antes da correção.

## GC-3 — Perder o cliente não limpa o draft na tela (P1)

**Por quê:** quando `gameflow_phase` falha, o watcher apaga `Shared::draft` e
volta ao começo do laço sem emitir `presage://draft` e sem limpar as marcas
(`watcher.rs:218-225`). O fim normal do champ select passa por `clear_draft`,
que faz as duas coisas. A tela continua mostrando o draft, o relógio desce até
zero, e o ranking, pedido de novo, volta vazio.

**Proposta:** o ramo de erro chama `clear_draft`, como os outros dois ramos.

> **Fechada.** Perder o cliente passa pelo mesmo `clear_draft` do fim do champ
> select (emite `presage://draft` nulo, limpa marcas e lista da conta), publica
> `Disconnected` com o erro e espera os 3 s de sem cliente (`perdeu_o_cliente`,
> T-CASCA-20, A5 e A11).

## GC-4 — Marcas e rota escolhida vivem em dois lugares; `get_enemy_roles` sem uso (P1)

**Por quê:**
- `set_enemy_role` e `set_draft_role` guardam na casca, e `App.tsx` guarda uma
  cópia (`enemyRoles`, `draftRole`) que só muda quando o comando resolve.
- `get_enemy_roles` existe (`lib.rs:178-189`) e nenhum arquivo de `src/` o chama.
  Para a rota escolhida não há comando de leitura.
- Recarregar a webview no meio do champ select zera as cópias e mantém as marcas
  na casca: o seletor mostra "(presumida)" e o ranking usa a marca.
- A casca esquece as marcas no fim do champ select, mas não a rota escolhida
  (`clear_draft` não toca `draft_role`).

**Proposta:** o draft que a casca emite traz as marcas e a rota escolhida (ou
`get_snapshot` as devolve), e a tela passa a ler em vez de copiar. Se
`get_enemy_roles` continuar, o front o chama na montagem.

> **Parcial.** A tela chama `get_enemy_roles` na montagem, e as marcas voltam
> depois de recarregar no meio do champ select (A2, TF-DRAFT-11). A rota escolhida
> ainda não tem comando de leitura.

## GC-5 — A rota que o motor usou não volta para a tela (P1)

**Por quê:** sem posição do cliente e sem escolha, a casca usa a rota da pool
quando ela é de uma rota só (`context::resolve_role`). O ranking não diz qual
rota foi usada, e a tela escreve "sem rota, a lista inclui todos"
(`RecommendationPanel.tsx:143`), que nesse caso é falso.

**Proposta:** `rank_draft` (ou o draft emitido) traz a rota resolvida e a origem
dela: `cliente`, `escolhida`, `pool` ou nenhuma. A aba acende a rota usada e o
texto diz de onde ela veio.

- Prova: TF-DRAFT-5.

**Enquanto isso:** o texto mente para quem tem pool de uma rota só e joga blind.

## GC-6 — Dois vocabulários de rota na mesma ponte (P1)

**Por quê:**
- Os comandos que recebem rota (`set_draft_role`, `set_enemy_role`,
  `pool_suggestions`) recebem `Option<String>` e passam por `Role::from_lcu`, que
  espera a palavra do cliente: `middle`, `bottom`, `utility`.
- Os tipos que cruzam a ponte nos dois sentidos (`Pool`, `LaneOutlook`, `Rival`,
  `CoachReport`) usam o nome do enum: `Mid`, `Bot`, `Support`.
- `from_lcu` transforma palavra desconhecida em `Unknown`, e os três comandos
  tratam `Unknown` como "sem rota". Mandar `"Mid"` para `set_draft_role` não dá
  erro: apaga a escolha. `ROLE_TO_LCU` em `types.ts` existe para isso não
  acontecer.

A tabela completa está no 12 §1, "Rota: dois vocabulários".

**Proposta:** os três comandos recebem `Option<Role>` com o `serde` do enum, e
`ROLE_TO_LCU` sai do front. A palavra do LCU fica só na fronteira com o LCU.

> **Parcial.** `Role::from_lcu` passou a aceitar os dois vocabulários, então um
> `"Mid"` num comando escolhe o meio em vez de apagar a escolha (A30, T-JOGO-15).
> A proposta inteira, um vocabulário só na ponte e `ROLE_TO_LCU` fora do front,
> continua aberta.

## GC-7 — Motor sem campeões fica "carregando" para sempre (P1)

**Por quê:** `EngineStatus.ready` é `!champions.is_empty()` (`lib.rs:67`). Se o
Data Dragon e o cache falharem, a casca só registra no log ("sem dados de
campeão, motor fica inativo") e emite `presage://engine` com `ready: false`. A
tela mostra "Carregando dados de campeões…" sem fim. Item e runa que falham
somem da mesma forma, sem estado próprio.

**Proposta:** `EngineStatus` ganha o estado de cada carga (`carregando`,
`pronto`, `falhou` com o motivo) para campeões, itens e runas. A tela troca
"Carregando" por "Não consegui os dados de campeão" quando falhou.

**Feito (A32):** `EngineStatus` ganhou `champions_error` e `items_error`, que
`guardar_carga` preenche; a faixa do draft e a build dizem o motivo
(T-CASCA-30, TF-DRAFT-14, TF-PARTIDA-5). Falta: as runas, e um "carregando" de
item e de runa separado do de campeão.

## GC-8 — O overlay não arrasta e não diz o próprio estado (P1)

**Por quê:**
- Ajustes diz que `Ctrl+Shift+O` deixa a janela clicável "quando você quiser
  arrastá-la". A janela não tem decoração, o front não marca região de arrasto e
  a capability não dá `core:window:allow-start-dragging`: ela não se move.
- A posição é a do `tauri.conf.json`, (24, 120), e não é gravada.
- O modo clicável é um `AtomicBool` na casca (`overlay.rs:20`); nem o overlay
  nem o painel sabem em que modo a janela está.
- Se outro app já tem o atalho, o registro falha só com um aviso no log
  (`lib.rs:663-665`), e o painel continua prometendo `Ctrl+Shift+P`.

**Proposta:**
- `data-tauri-drag-region` na cabeça do overlay e a permissão de arrasto na
  capability da janela `overlay`; a posição gravada em `overlay.json` ao soltar.
- Um evento `presage://overlay` com `{ visible, interactive }`, para o overlay
  mostrar uma borda quando estiver clicável.
- Um comando que diga quais atalhos foram registrados, para Ajustes avisar.
- Prova: TF-OVERLAY-5.

**Feito (A15):** a cabeça é região de arrasto, com a permissão na capability do
overlay; e a janela só pega clique com o atalho ligado, na partida e com alguma
seção à mostra (`OverlayPrefs::atravessa_o_mouse`, aplicada por
`overlay::aplicar`). Falta: gravar a posição, o evento de estado e o comando dos
atalhos.

## GC-9 — Regras do core copiadas no front (P1)

**Por quê:** a mesma crença escrita em dois lugares é o padrão do
[L-7](../arquitetura/09-licoes-do-prototipo.md). Hoje:

| Regra | No core | No front |
|---|---|---|
| Leitura clara: `abs(value) > 1,96·sd` | `Reading::is_clear` (`profile.rs:52-53`) | `PerfilDeRota.tsx:19`, e o `±` mostrado é `1,96·sd` calculado na tela (`:55`) |
| À frente a partir de 300 ou de 10% | `rival::AHEAD_FLOOR`, `AHEAD_SHARE` | o texto de `LiveOverview.tsx:175` |
| Até cinco bans | `pool::BAN_POOL_MAX`, que nenhum código Rust usa | `types.ts:455`, o único limite que vale |
| Oito na pool, três core | `poolgap::POOL_SIZE`, `CORE_SIZE` | `types.ts:333-334` |
| Quando o ouro em mão importa | `gold.rs:39`, 1.500 | `GoldTracker.tsx:70`, 800 |

**Proposta:**
- `LaneReading` ganha `clear: bool` e `interval: f64`, calculados por
  `is_clear`; a tela só lê.
- `Rival` ganha o limite usado (`aheadAt`), e o texto o mostra.
- A casca recusa em `set_pool` uma lista com mais de `BAN_POOL_MAX` bans.
- As constantes que o TS precisa vêm de um comando (`limits`) ou entram na
  comparação do doccheck.

**Enquanto isso:** mudar um destes números no Rust exige mudar o front à mão.

## GC-10 — Rótulos escritos dos dois lados (P2)

**Por quê:**
- `OVERLAY_SECTIONS` (`types.ts:411-438`) repete letra por letra
  `Section::label` e `Section::detail` (`prefs.rs:35-55`), que no Rust só os
  testes usam.
- A rota tem três tabelas de rótulo: `Role::short` (`JG`, `SUP`), que chega à
  tela em `LaneOutlook.lane`; `POSITION_LABEL` no TS, com os mesmos `JG` e
  `SUP`; e `ROLES` no TS, com `JUNGLE` e `SUPORTE`. A mesma tela mostra
  "JUNGLE" na aba de rota e "JG" na linha da rota.
- A fase do timer do champ select (`BAN_PICK`) aparece crua no `DraftBoard`, e a
  fase do cliente aparece crua em "Cliente conectado (Lobby)." enquanto a barra
  usa `PHASE_LABEL`.

**Proposta:** um lado só escreve rótulo. Para as seções, a casca manda
`{ id, label, detail }` em `get_overlay_prefs`; para a rota, o front usa um
rótulo só, e `Role::short` vira o de todos.

**Enquanto isso:** mudar um texto exige achar as cópias.

## GC-11 — O número de seções do overlay escrito à mão (P2)

**Por quê:** `overlay.tsx:94` apaga o overlay quando `prefs.hidden.length >= 6`.
O painel usa `OVERLAY_SECTIONS.length`, e o core tem `Section::ALL` e
`OverlayPrefs::anything_visible`, que ninguém chama fora dos testes. Uma
sétima seção faria o overlay sumir com seis desligadas e uma ligada.

**Proposta:** `overlay.tsx` usa `OVERLAY_SECTIONS.length`, ou a casca manda
`anythingVisible` e esconde a janela de verdade quando é falso, em vez de a tela
desenhar `null` numa janela que continua aberta.

- Prova: TF-OVERLAY-4.

**Enquanto isso:** vale enquanto houver seis seções.

## GC-12 — O Data Dragon baixado duas vezes (P2)

**Por quê:** a casca baixa campeões, itens e runas e guarda cache em disco
(`ddragon.rs:137-150`). O front baixa os campeões de novo, sem cache
(`ddragon.ts:22-49`). Sem rede, o motor funciona pelo cache e a tela fica sem
nomes e sem retratos. As duas cópias podem estar em versões diferentes: o texto da
runa cita a versão da casca (`engine.version`), e o ícone de item usa a do
front (`staticData.version`).

**Proposta:** um comando `static_data` que devolve, do `ChampionIndex` da
casca, a versão e, por campeão, o id, a chave e o nome. O front monta as URLs a
partir disso e para de buscar `versions.json` e `champion.json`.

**Enquanto isso:** a faixa "Não consegui carregar os dados do Data Dragon" avisa
quando o front falha.

## GC-13 — Uma capability para as duas janelas (P2)

**Por quê:** `capabilities/default.json` vale para `main` e `overlay`, com
`updater:default` e `process:allow-restart`. Sem manifesto de comandos do app
(`build.rs` é só `tauri_build::build()`), as duas janelas alcançam os 26
comandos. O overlay usa cinco ([05 §3](05-seguranca.md#3-o-que-a-webview-pode-chamar)).

**Proposta:** duas capabilities. A do overlay com `core:event:default` e os cinco
comandos que ele usa, declarados num manifesto do app
(`tauri_build::Attributes::app_manifest`); a principal com o resto.

> **Parcial.** Duas capabilities: `main.json` com o atualizador e o reinício,
> `overlay.json` só com `core:default` (A33, T-CASCA-29). O manifesto de comandos do
> app, que limitaria o overlay aos cinco comandos que ele usa, continua aberto
> (A52).
