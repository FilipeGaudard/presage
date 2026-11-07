# 14 — Uma partida, passo a passo

> **O que é este documento.** O caminho de uma partida, do app abrindo até a
> leitura pós-jogo, na ordem em que o código o percorre. Os outros documentos
> detalham cada peça (02 e 03 o desenho, 10 os tipos, 11 as funções, 12 a casca,
> 13 os testes); este conta a história inteira e, em cada passo, **por que** ele
> existe.
>
> Escrito em 2026-11-07, conferido contra o commit `4d76a91`. As linhas citadas
> (`watcher.rs:240`) são desse commit. Os números são as constantes do código. A
> partida do §13 é a dos **dados de exemplo** de `src/demo.tsx`, não uma partida
> real.

## 1. Como ler

Cada passo tem quatro partes:

- **O que acontece:** a sequência, como o código a executa.
- **Por quê:** o que cria o passo: a decisão (ADR), o requisito (R), a regra da
  Riot (C), o requisito não funcional (RNF), a lição do protótipo (L) ou a
  pergunta em aberto (P). Os testes que o provam vêm pelo T-ID do
  [13](13-testes.md).
- **Onde:** a função e o arquivo, em `monoespaçado`, e a seção do
  [modelo de dados](10-modelo-de-dados.md), do
  [catálogo de funções](11-catalogo-de-funcoes.md) ou do
  [contrato da casca](12-contrato-da-casca.md), escrita como "11 §12.3".
- **O que pode dar errado:** o que vira erro, silêncio ou número errado nesse
  passo. **Defeito conhecido** marca o que o código faz hoje contra o que um
  requisito pede.

**Caminhos.** Sem prefixo é a casca, em `src-tauri/src/` (`lib.rs`,
`watcher.rs`, `recorder.rs`, `lcu/credentials.rs`, `live.rs`). O core é
`core/src/…` e a interface é `src/…`. Há dois `live.rs`: o da casca lê a Live
Client, o do core (`core/src/live.rs`) traduz.

**Confiança.** Todo número na tela é exato, alto, fraco ou regra
([README](../README.md#confiança-do-número)). O passo diz qual, quando importa.

## 2. O caminho numa página

```
 [§3] O app abre
      run → montar(true) → setup: pool, base, preferências e última leitura do disco
      bandeja · overlay atravessável · Ctrl+Shift+P / Ctrl+Shift+O
      ├── tarefa 1: watcher::run ─────────────────────────────────────────┐
      ├── tarefa 2: Data Dragon (campeões, itens, runas) → presage://engine│
      └── tarefa 3: profile::refresh (só com a chave) ─────────────┐       │
                                                                   │       │
 [§4] Achar o cliente ◀────────────────────────────────────────────┼───────┘
      lockfile → Credentials → conta (puuid, Riot ID, região)      │
      a cada volta: fase do fluxo → pausa 350 ms · 1 s · 1,5 s · 3 s
         │                                                         │
         ├── ChampSelect                                           │
         │   [§5] grade + fila → selectable · sessão → to_draft_state → presage://draft
         │   [§6] DraftContext: sua rota, as rotas deles, os candidatos
         │   [§7] rank_with · minimax_with (largura 12) · bans_with_pool (10)
         │   [§8] prévia: lane_analysis(candidato) · runes::plan
         │                                                         │
         ├── InProgress                                            │
         │   [§9]  Live Client a cada 1 s → to_live_game → LiveSnapshot → presage://live
         │   [§10] timers, buffs, prioridade, sua rota · build, rotas, ouro · overlay
         │   [§11] recorder::observe: um quadro por volta          │
         │                                                         │
         └── qualquer outra fase                                   │
             [§11] close_recording → coach::review → ultima-partida.json
                   → presage://review → aba Pós-jogo               │
                                                                   ▼
 [§12] match-v5 → history::played → personal-v2.json → perfil de rota · o que fica em disco
```

A ideia que atravessa tudo: **a casca lê, o core calcula, a tela mostra.** Nada
escreve no cliente ([ADR-0026](../adr/0026-lcu-so-leitura-e-autostart-desligado.md)),
nada sai da máquina (RNF-8), e um componente sem dado vale exatamente zero
(RNF-1).

## 3. O app abre

### 3.1 O processo e o atualizador

**O que acontece.**
- `run` liga o log (`RUST_LOG`, senão `presage_lib=debug,warn`) e sobe o Tauri
  com `montar(true)`.
- `montar(true)` registra o atualizador e o reinício (`tauri-plugin-updater`,
  `tauri-plugin-process`), o início com o Windows e os atalhos globais.
- Se a inicialização falha e o texto do erro cita `updater`
  (`erro_do_atualizador`), o app sobe de novo com `montar(false)`, sem os dois
  plugins. Qualquer outro erro derruba o app com pânico.
- A janela monta e `Atualizacao.tsx` chama `check()` **uma vez**. Achou versão
  nova, mostra o aviso; instalar é clique do jogador.

**Por quê.**
- **Um endpoint errado não pode deixar todo mundo sem app.** O Tauri recusa
  configuração inválida do atualizador na inicialização, e o processo morreria
  junto ([ADR-0005](../adr/0005-atualizador-assinado-com-endpoint-permanente.md)).
- **Nunca instala sozinho, falha calado** (RNF-7, R-19): trocar o executável no
  meio de um draft de 30 s é o pior momento.
- Testes: T-CASCA-13, T-CASCA-14.

**Onde.** `run` (`lib.rs:496`), `montar` (`lib.rs:588`), `erro_do_atualizador`
(`lib.rs:539`); `check()` em `src/components/Atualizacao.tsx:27`. 12 §7
(Atualizador), [06 §2](06-distribuicao-e-atualizador.md#2-o-atualizador).

**O que pode dar errado.**
- O reconhecimento é casamento de texto. Se a mensagem do Tauri mudar, o app
  volta a morrer na inicialização. O teste prende a mensagem real.
- Sem atualizador, `check()` rejeita e a rejeição é engolida
  (`Atualizacao.tsx:31`). Ninguém sabe que o app parou de se atualizar.

### 3.2 O que volta do disco

**O que acontece.** O `setup` (`lib.rs:612`) cria o `Shared` vazio e o enche,
nesta ordem:
1. `pool.json` → `Shared::pool` (`lib.rs:614`).
2. `base-de-partidas.json`, **só se** `Dataset::is_usable` (`lib.rs:622`).
3. `rebuild_stats` com um `PersonalStats` vazio (`lib.rs:634`): a loja vira
   `Layered`, com a base (ou `None`) e sem histórico pessoal.
4. `ultima-partida.json` → `last_review` (`lib.rs:636`).
5. `draft.json` e `overlay.json` → as preferências (`lib.rs:640`, `lib.rs:644`).
6. `app.manage` (`lib.rs:652`): o mesmo `Arc<Shared>` serve os 26 comandos e as
   três tarefas de fundo.

**Por quê.**
- **Um estado só** (12 §1, "O estado compartilhado"): todo comando lê do
  `Shared`, nenhum comando fala com a rede.
- **Base truncada é recusada:** melhor sem base do que recomendar de meia dúzia
  de linhas. Sem base, cada componente estatístico vale zero
  ([ADR-0007](../adr/0007-sem-dado-o-componente-vale-zero.md), RNF-1).
- **Hoje não há base.** O coletor existe; a base espera uma chave da Riot que
  dure (P-1, P-6).

**Onde.** O fecho de `.setup` em `montar`; `storage::read_json`. 10 §32.5
(`Shared`), 10 §33 (os arquivos), 12 §6.

**O que pode dar errado.**
- `read_json` devolve `None` para arquivo que falta, não abre ou não
  desserializa, sem log (12 §6). Uma pool corrompida abre como pool vazia.
- Nenhum arquivo tem versão de esquema (10 §33).
- O histórico pessoal **não** sai do disco aqui. Mesmo com `personal-v2.json`
  salvo, o componente pessoal só liga quando `profile::refresh` termina (§3.5).

### 3.3 Bandeja, overlay e atalhos

**O que acontece.**
- `bandeja` (`lib.rs:472`, chamada em `lib.rs:654`): o ícone com "Mostrar o
  Presage" e "Sair".
- `overlay::prepare` (`lib.rs:658`) liga `set_ignore_cursor_events(true)` antes
  de qualquer exibição. A janela `overlay` já existe, escondida
  (`tauri.conf.json`).
- `Ctrl+Shift+P` e `Ctrl+Shift+O` são registrados como atalhos globais
  (`lib.rs:662`).

**Por quê.**
- **O overlay é janela externa, nunca injeção** (C-3,
  [ADR-0003](../adr/0003-overlay-como-janela-externa.md)). Ele nasce
  atravessável para não comer clique no meio da partida (R-15).

**Onde.** `bandeja` e o `setup` (`lib.rs`), `overlay.rs`. 12 §7 (Overlay e
atalhos, Bandeja).

**O que pode dar errado.**
- Ícone que não sobe ou atalho já registrado por outro app viram só um aviso no
  log. Ajustes continua prometendo `Ctrl+Shift+P`
  ([GC-8](../frontend/07-lacunas-da-casca.md)).
- O overlay não arrasta: a janela fica em (24, 120) (GC-8).

### 3.4 As três tarefas de fundo

**O que acontece.** O `setup` lança três tarefas e retorna:
1. `watcher::run` (`lib.rs:669`): o laço do cliente e da partida (§4).
2. A carga do Data Dragon (`lib.rs:673`): `load_champions`, `load_items` e
   `load_runes`, em sequência, em `pt_BR`. No fim, `presage://engine` com o
   `EngineStatus` (`lib.rs:702`).
3. `profile::refresh` (`lib.rs:707`): o histórico pessoal (§3.5 e §12.1). No
   fim, `presage://engine` de novo (`lib.rs:715`), com ou sem sucesso.

**Por quê.**
- **Atributos de item vêm da descrição em `en_US`:** o bloco `stats` do Data
  Dragon não tem penetração, letalidade nem haste
  ([ADR-0015](../adr/0015-atributos-de-item-pela-descricao.md), L-11).
- **Runas pela chave textual,** nunca pelo id (L-17, R-7).
- **Motor pronto é ter campeões:** `EngineStatus.ready = !champions.is_empty()`
  (`lib.rs:67`). A interface só pede ranking com o motor pronto
  (`src/App.tsx:168`).

**Onde.** `ddragon.rs` (`load_champions`, `ddragon.rs:137`); 12 §5 (Data
Dragon), 12 §6 (os caches), 10 §32.1 (`EngineStatus`).

**O que pode dar errado.**
- Sem rede, cada carga cai para o cache (`champions.json`, `items.json`,
  `runas.json`) e serve o último patch baixado sem avisar
  ([00 §6](00-contexto-e-limites.md#6-limites-conhecidos)).
- Sem rede e sem cache, `ready` fica `false` e a tela diz "Carregando dados de
  campeões…" para sempre ([GC-7](../frontend/07-lacunas-da-casca.md)).
- A interface baixa o Data Dragon de novo, por conta própria, para nomes e
  ícones (`src/lib/ddragon.ts`, GC-12).

### 3.5 O histórico pessoal começa a esperar

**O que acontece.**
- Sem `PRESAGE_RIOT_API_KEY`, `profile::refresh` devolve `Err` na primeira linha
  (`profile.rs:33`). O log diz "componente pessoal desligado" e o resto do app
  segue.
- Com a chave, `resolve_identity` (`profile.rs:106`) espera a conta que o watcher
  lê do cliente (§4.2), olhando a cada 2 s, por até 120 s (`profile.rs:29`).
- Passados os 120 s, tenta `PRESAGE_RIOT_ID` pela account-v1
  (`identity_from_env`, `profile.rs:122`). O resto está em §12.1.

**Por quê.**
- **Funciona sem chave:** tudo menos histórico pessoal e base agregada (RNF-5,
  [00 §5](00-contexto-e-limites.md#5-o-que-funciona-sem-chave)).
- **A chave sai só do ambiente,** nunca de binário, argumento ou URL (RNF-6,
  [ADR-0004](../adr/0004-chave-da-riot-so-no-backend.md)).
- **A conta sai do cliente:** o jogador não digita o Riot ID em lugar nenhum.

**Onde.** `refresh` e `resolve_identity` (`profile.rs`). 12 §5 (Riot Web API),
12 §7 (Variáveis de ambiente), 10 §32.4 (`Identity`).

**O que pode dar errado.**
- Abrir o app mais de 120 s antes do cliente, sem `PRESAGE_RIOT_ID`, desliga o
  histórico pessoal até a próxima abertura.
- O histórico só se atualiza quando o app abre. Uma partida jogada com o app
  aberto entra no perfil na abertura seguinte
  ([04 §7](04-perfil-de-rota.md#7-o-cache)).

### 3.6 A janela monta

**O que acontece.**
- `App.tsx` pede o Data Dragon do front (`src/App.tsx:85`) e faz nove `invoke`
  de uma vez (`src/App.tsx:94-112`): `get_snapshot`, `engine_status`, `get_pool`,
  `get_live`, `last_review`, `lane_profile`, `get_overlay_prefs`,
  `get_draft_prefs` e `get_autostart`.
- Depois ouve cinco eventos (`src/App.tsx:114-137`): `presage://status`,
  `presage://engine`, `presage://live`, `presage://review` e `presage://draft`.
- A interface não faz polling. Ela chama comandos quando um evento muda o
  estado dela (12 §1, "Ritmo").

**Por quê.** `get_snapshot` existe para a tela não esperar a próxima volta do
watcher ao abrir: o último status e o último draft já estão no `Shared`.

**Onde.** `App` (`src/App.tsx`); 12 §2 (comandos), 12 §3 (eventos);
[frontend 03](../frontend/03-estado-e-atualizacao.md).

**O que pode dar errado.**
- **Defeito corrigido (A1).** Até a 0.7.0, a falha de qualquer comando sumia
  (22 `.catch(() => void 0)`). Agora toda chamada passa por `chamar`, e a falha
  vira uma faixa que diz o que não deu ([GC-1](../frontend/07-lacunas-da-casca.md)).
- `set_pool`, `set_overlay_prefs` e `set_draft_prefs` mudam a tela antes da
  resposta e voltam ao valor de antes se a casca recusar (TF-POOL-4).

## 4. Achar o cliente e o ritmo

### 4.1 O lockfile

**O que acontece.** Na volta sem cliente, `credentials::discover` procura o
`lockfile` do League, do mais barato para o mais caro (`candidate_dirs`):
1. as três pastas de `KNOWN_DIRS` (`lcu/credentials.rs:36`);
2. todo caminho `X:\…` citado na configuração da própria Riot
   (`dirs_from_riot_config`, `lcu/credentials.rs:79`), mais a subpasta `League of
   Legends` de cada um;
3. só se nada disso tiver lockfile, a pasta do executável de `LeagueClientUx.exe`
   (`dir_from_running_client`, `lcu/credentials.rs:154`), pelo caminho da imagem.

O primeiro lockfile que `parse_lockfile` (`core/src/lockfile.rs:17`) aceita vence.
`LcuClient::new` monta `https://127.0.0.1:<porta>`, Basic Auth `riot:<senha>`,
timeout de 5 s.

**Por quê.**
- **Um falso positivo de antivírus.** A primeira versão lançava PowerShell
  escondido para ler a linha de comando do cliente, e foi marcada como trojan. A
  ordem vai do silencioso para o invasivo (`lcu/credentials.rs:6-26`).
- **Só APIs oficiais, nada de memória** (C-2). **Só leitura** (R-18,
  [ADR-0026](../adr/0026-lcu-so-leitura-e-autostart-desligado.md)).
- Testes: T-ENTRADA-1, T-ENTRADA-2, T-CASCA-8 a T-CASCA-12.

**Onde.** `discover` (`lcu/credentials.rs:42`), `parse_lockfile`; 11 §2.2, 12 §5
(Descoberta do cliente, LCU).

**O que pode dar errado.**
- Sem lockfile, `discover` devolve um erro que diz o que fazer. O watcher publica
  `connected: false`, `phase: "Disconnected"` com esse texto em `detail`, e dorme
  3 s (`watcher.rs:197-209`).
- A verificação de TLS do loopback está desligada (P-8).
- `parse_cmdline` existe no core e a casca não o usa (R-18).

### 4.2 A conta

**O que acontece.**
- Uma vez por conexão, `current_summoner` (`watcher.rs:178`) dá `gameName`,
  `tagLine` e `puuid`. O Riot ID vira `nome#tag` e vai para a barra.
- Com `puuid`, `region` lê `/riot/v1/region-locale`; erro vira `"BR"`
  (`watcher.rs:187`). As três coisas viram `Identity` no `Shared`
  (`watcher.rs:188`), que §3.5 está esperando.

**Por quê.** O perfil de rota precisa do puuid (R-17), e o histórico da match-v5
mora no host regional da conta (`Region::from_lcu`, 11 §6.1).

**Onde.** `watcher::run` (`watcher.rs:175-212`); 10 §32.4.

**O que pode dar errado.** A leitura da conta não se repete na mesma conexão. Se
a primeira chamada falha (o cliente ainda subindo), a barra fica sem Riot ID e a
`Identity` não existe até o cliente cair e voltar. O histórico pessoal espera
os 120 s e cai para `PRESAGE_RIOT_ID`.

### 4.3 O laço e o ritmo por fase

**O que acontece.** Cada volta pede a fase do fluxo (`gameflow_phase`,
`watcher.rs:216`), publica o status só se mudou (`publish_status`,
`watcher.rs:358`) e escolhe um ramo:

| Fase | Ramo | Pausa | Constante |
|---|---|---|---|
| `ChampSelect` | grade, fila e sessão (§5) | 350 ms | `POLL_CHAMP_SELECT` (`watcher.rs:43`) |
| `InProgress` | Live Client, gravação (§9, §11.1) | 1 s | `POLL_LIVE` (`watcher.rs:44`) |
| qualquer outra, com cliente | limpa o draft e a partida, fecha a gravação (§9.1, §11.2) | 1,5 s | `POLL_IDLE` (`watcher.rs:45`) |
| sem cliente | `discover` de novo (§4.1) | 3 s | `POLL_DISCONNECTED` (`watcher.rs:46`) |

**Por quê.**
- **Polling, não WebSocket:** mais simples de acertar, e o custo é irrelevante
  nesta escala ([ADR-0002](../adr/0002-polling-em-vez-de-websocket.md)).
- **Tempo real para uma fase de 30 s** (RNF-4): 350 ms no champ select. O ritmo
  segue a fase (R-18).
- **Reconectar é o caminho normal:** as credenciais mudam a cada reinício do
  cliente (`watcher.rs:173`).

**Onde.** `watcher::run` (`watcher.rs:158`); 12 §1 ("Ritmo").

**O que pode dar errado.**
- A pausa vem **depois** das chamadas: o período real é a pausa mais o tempo das
  requisições, com até 5 s de timeout por chamada ao LCU.
- **Perder o cliente** (a leitura da fase falha): o watcher encerra o draft como
  no fim do champ select (emite `presage://draft` nulo e limpa marcas e lista da
  conta), publica `Disconnected` com o erro e espera os 3 s de sem cliente antes
  de procurar de novo (`perdeu_o_cliente`, T-CASCA-20). Até a 0.7.0 ele apagava o
  draft calado e voltava sem pausa: a tela ficava com o draft velho, e um
  lockfile velho fazia o laço girar (A5, A11,
  [GC-3](../frontend/07-lacunas-da-casca.md)).
- Pela leitura do código: o ramo "qualquer outra" pega toda fase que não é
  `ChampSelect` nem `InProgress`, inclusive a de reconexão, que a barra traduz
  como "Reconectando" (`src/components/ConnectionStatus.tsx:13`). Cair da partida
  e voltar fecha a gravação no meio (§11.2) e zera os buffs.

## 5. O champ select

### 5.1 A grade e a fila

**O que acontece.** Enquanto `Shared::owned` é `None` (`watcher.rs:243`):
- `grid_champions` lê `/lol-champ-select/v1/all-grid-champions`, e `queue_id`, a
  fila em `/lol-gameflow/v1/session`.
- `selectable` (`core/src/draft.rs:280`) fica com quem a conta tem ou o Game Pass
  libera. A rotação grátis só entra com fila conhecida e fora de
  `RANKED_QUEUES` (420, 440).
- Lista não vazia vai para `owned` (`watcher.rs:250`). Vazia, a próxima volta
  tenta de novo.

**Por quê.**
- **Recomendar quem você não tem é ruído que parece conselho**
  ([ADR-0025](../adr/0025-so-quem-a-conta-pode-escolher-por-padrao.md), R-5).
- **Fila desconhecida fica do lado seguro,** sem a rotação.
- Testes: T-ENTRADA-11, T-ENTRADA-12.

**Onde.** `selectable`; 11 §3.3, 10 §3.9 a 10 §3.13.

**O que pode dar errado.** Grade indisponível não tira ninguém (T-DRAFT-35): o
ranking volta a ter todo mundo, calado. "Incluir quem você não tem"
(`draft.json`) e o simulador desligam o filtro de propósito (`lib.rs:132`).

### 5.2 A sessão

**O que acontece.**
- A cada 350 ms, `champ_select_session` (`watcher.rs:257`) e `to_draft_state`
  (`core/src/draft.rs:129`):
  - os bans vão para cada lado por `is_ally_action`;
  - o campeão do time deles sai do `pick` concluído, porque no jogador ele vem
    zerado;
  - o `type` da primeira ação em andamento vira `current_action`;
  - `unavailable` junta bans e campeões dos slots, ordenado, sem repetição e
    sem 0.
- `same_draft` (`watcher.rs:348`) compara sem o relógio. Só a mudança vira
  `presage://draft` (`watcher.rs:269`).
- A tela guarda o draft e uma âncora do relógio (`src/App.tsx:129-136`), e
  interpola o relógio a cada 100 ms (`src/App.tsx:139`).

**Por quê.**
- **O time inimigo vem do LCU com `championId` zerado** (L-10).
- **Uma leitura do champ select** para ranking, resposta e prévia
  ([ADR-0017](../adr/0017-uma-leitura-do-champ-select.md)).
- **O relógio fora da comparação:** senão todo tick viraria evento e todo evento
  refaria o ranking.
- Testes: T-ENTRADA-6, T-ENTRADA-7, T-ENTRADA-9, T-ENTRADA-10.

**Onde.** `to_draft_state`; 11 §3.1, 10 §3.1 a 10 §3.7.

**O que pode dar errado.** Falha ao ler a sessão vira log de depuração
(`watcher.rs:273`), e a tela fica com a leitura anterior. O hover do inimigo não
existe na API: o campeão deles só aparece travado
([00 §6](00-contexto-e-limites.md#6-limites-conhecidos)).

### 5.3 O simulador

**O que acontece.** Com a aba Simulador aberta, a tela manda o draft montado à
mão (`set_simulated_draft`, `src/App.tsx:157`), que passa por
`DraftState::normalized` (`lib.rs:193`). `current_draft` (`lib.rs:109`) prefere
o simulado ao do cliente. Sair da aba manda `null` (`src/App.tsx:153`).

**Por quê.** Treinar sem champ select e mostrar o app sem cliente (R-8,
[ADR-0023](../adr/0023-o-simulador-passa-por-cima-do-cliente.md)). O motor refaz
indisponíveis e a sua rota sozinho (T-ENTRADA-13).

**Onde.** `set_simulated_draft`, `current_draft` (`lib.rs`); 11 §3.2.

## 6. O contexto do draft

`draft_context` (`lib.rs:119`) monta o mesmo `DraftContext` para `rank_draft`,
`minimax_draft`, `lane_analysis` e `rune_plan`. `ban_suggestions` lê o draft
direto (§7.3).

### 6.1 Sua rota

**O que acontece.** `resolve_role` (`core/src/engine/context.rs:84`) devolve, na
ordem:
1. a posição que o cliente informa;
2. a escolhida no seletor do painel (`set_draft_role`);
3. a rota da pool, quando todas as entradas com rota têm a mesma;
4. `Role::Unknown`, e aí a rota não filtra ninguém.

**Por quê.** Em blind e normal o cliente não informa a posição, e o ranking
precisa da rota (R-1, [ADR-0017](../adr/0017-uma-leitura-do-champ-select.md),
[02 §0](02-motor-de-draft.md#0-o-contexto-do-draft)).

**Onde.** `resolve_role`; 11 §10.1.

**O que pode dar errado.**
- `draft_role` nunca é limpa (12 §1, tabela do `Shared`): a rota escolhida num
  blind vale no blind seguinte, até você trocar.
- A rota que o motor usou não volta para a tela. O painel diz "sem rota, a lista
  inclui todos" quando a casca usou a rota da pool
  ([GC-5](../frontend/07-lacunas-da-casca.md)).

### 6.2 Quem está em cada rota

**O que acontece.** `DraftContext::new_with` (`core/src/engine/context.rs:187`):
1. `me`: o campeão do seu slot ou, sem ele, o hover.
2. `allies`: os outros slots do seu time com campeão ou hover.
3. `enemies`: os slots deles com campeão.
4. As marcas (`set_enemy_role`) entram primeiro, na ordem dos slots, em quem está
   sem posição e numa rota ainda livre.
5. Quem sobrou ganha rota pela atribuição mais plausível das rotas livres: busca
   exaustiva (no máximo 120 combinações), soma de `ln(role_fit + EPS)`, a primeira
   melhor em ordem. `roles_assumed` avisa a tela.

**Por quê.**
- **A ranqueada não informa a rota do inimigo.** Pela ordem de escolha, a análise
  pareava contra o inimigo errado (L-25, R-4).
- **A marca vence a inferência e nunca a posição do cliente**
  ([ADR-0024](../adr/0024-a-marca-de-rota-vence-a-inferencia.md)).
- Testes: T-DRAFT-22, T-DRAFT-30, T-CASCA-3.

**Onde.** `new_with`, `roles::role_fit`; 11 §10.2, 11 §11.2, 10 §10.4.

**O que pode dar errado.**
- Sem a base, `role_fit` sai das classes do Data Dragon e erra lutador puro e
  escolha fora de rota (P-13).
- As marcas vivem em dois lugares: no `Shared` e no estado da tela
  (`src/App.tsx:74`). `get_enemy_roles` não tem quem chame
  ([GC-4](../frontend/07-lacunas-da-casca.md)).

### 6.3 Indisponíveis e candidatos

**O que acontece.**
- `unavailable`: bans, aliados (hover inclusive), inimigos e o seu campeão **só
  travado**.
- `candidates`: os ids do índice fora de `unavailable`, dentro de `owned` quando
  ele não é vazio, e que jogam a sua rota (`roles::plays`) **ou** estão na sua
  pool.

**Por quê.**
- **Suporte e topo apareciam para o meio:** o ranking pontuava o jogo inteiro
  (L-23).
- **Hover de aliado não é recomendado; a sua escolha não travada continua sua**
  (L-26). O cliente preenche o seu `championId` antes de travar (P-12).
- **A pool passa por cima da rota, não da conta** (R-5).
- Testes: T-DRAFT-14, T-DRAFT-25, T-DRAFT-26, T-DRAFT-34.

**Onde.** `new_with`, `roles::plays`; 11 §10.2, 11 §11.3.

## 7. Ranking, resposta e bans

A cada mudança de draft, pool, rota, marca ou preferência, a tela pede os três
juntos num `Promise.all` (`src/App.tsx:167-192`), só com o motor pronto.

### 7.1 O ranking

**O que acontece.** `rank_with` (`core/src/engine/mod.rs:86`), para cada
candidato:
1. a winrate base na sua rota, encolhida para 50%;
2. o matchup contra o oponente de rota, com peso `w` (1 com posição informada
   ou marcada, a fração do encaixe quando inferida);
3. o contra de cada inimigo travado, em qualquer rota;
4. a sinergia com cada aliado, hover inclusive;
5. o componente pessoal;
6. a composição: equilíbrio de dano e linha de frente.

Soma tudo em log-odds, corta em [0,25; 0,75], monta até três razões e ordena por
`winrate`, depois pool, depois id.

**Por quê.**
- **Sem dado, o componente vale exatamente zero** (RNF-1,
  [ADR-0007](../adr/0007-sem-dado-o-componente-vale-zero.md)). Sem base, a ordem
  sai só da composição e do histórico pessoal.
- **Efeito medido contra o que as forças já explicam** (RNF-3, L-1,
  [ADR-0008](../adr/0008-matchup-e-sinergia-contra-a-soma-das-forcas.md)); a força
  do shrinkage sai do desvio (RNF-2, L-2,
  [ADR-0009](../adr/0009-forca-do-shrinkage-derivada-do-desvio.md)); o contra em
  qualquer rota ([ADR-0019](../adr/0019-contra-em-qualquer-rota-e-chance-do-time.md)).
- **Alternativas com motivo, nunca um pick só** (C-5, R-1).
- Testes: T-DRAFT-10, T-DRAFT-18.

**Onde.** `rank_with`; 11 §9.2, 10 §9.1 (`Scored`),
[02 §1](02-motor-de-draft.md#1-rank).

**O que pode dar errado.**
- Confiança **fraca:** os pesos de composição são escritos à mão, e há dois
  conjuntos deles, o do ranking e o do minimax (P-11).
- O componente pessoal carrega a fase da conta
  ([ADR-0012](../adr/0012-vazamento-da-fase-da-conta-mantido.md)).
- Se um dos três comandos falha, a lista dele sai da tela e a faixa diz qual; as
  outras duas continuam valendo (TF-CASCA-4).

### 7.2 A resposta deles

**O que acontece.** `minimax_with` (`core/src/engine/minimax.rs:114`), com
largura 12 fixa na casca (`lib.rs:226`):
1. as respostas possíveis: todo campeão fora de `unavailable` que joga uma rota
   inimiga ainda livre (`open_enemy_roles`);
2. o guloso: `team_value` com cada candidato, cortado nos 12 melhores;
3. para cada um, a pior resposta deles e quem a deu;
4. ordena pelo valor depois da resposta.

A tela mostra os cinco **menos expostos**, reordenados por `exposure`
(`src/components/RecommendationPanel.tsx:102`), e no detalhe "Melhor resposta
deles custa x.xpp (Nome)".

**Por quê.**
- **Um pick bom no guloso pode ser o mais fácil de punir** (R-2,
  [02 §6](02-motor-de-draft.md#6-draft-com-resposta)).
- **A resposta varre todo mundo:** a busca antiga olhava sempre os 32 campeões de
  id mais baixo (L-24).
- **Podado para caber em 30 s** (RNF-4).
- Testes: T-DRAFT-46, T-DRAFT-48, T-DRAFT-51.

**Onde.** `minimax_with`, `team_value`; 11 §12.1, 11 §12.3, 10 §12.1.

**O que pode dar errado.** `team_value` não lê estatística: o valor da resposta
é índice de composição, confiança **fraca**. A loja só decide quem joga a rota.

### 7.3 Os bans

**O que acontece.** `bans_with_pool` (`core/src/engine/minimax.rs:265`), com
limite 10 (`lib.rs:237`):
1. a sua lista de bans (até cinco, em ordem), sem quem já saiu e sem quem o seu
   time quer pegar;
2. depois, `suggest_bans`: quanto cada campeão completa a composição deles
   (`threat`), sem repetir a lista.

A seção só aparece quando `current_action` é `ban`
(`src/components/RecommendationPanel.tsx:97`).

**Por quê.**
- **A sua lista vem na frente desde o primeiro ban** (R-3).
- **O ban não é pessoal enquanto o valor é aditivo:** banir pela sua fraqueza e
  banir pela composição deles dariam a mesma ordem
  ([ADR-0013](../adr/0013-ban-nao-personalizado-enquanto-aditivo.md), T-DRAFT-57).
- **O app nunca bane sozinho** ([ADR-0026](../adr/0026-lcu-so-leitura-e-autostart-desligado.md)).
- Testes: T-DRAFT-53, T-DRAFT-58, T-DRAFT-59.

**Onde.** `bans_with_pool`, `suggest_bans`; 11 §12.4, 11 §12.5, 10 §12.2.

**O que pode dar errado.** Sem pick deles e sem lista, a sugestão é vazia, e a
tela explica por quê em vez de ordenar ruído.

## 8. A prévia e as runas

### 8.1 A rota presumida na tela

**O que acontece.** Durante o draft, a tela pede `lane_analysis` com
`candidate: null` a cada draft, partida, motor ou marca
(`src/App.tsx:224-243`). Sem partida, `analysis::report` (`analysis.rs:56`) cai
para `report::from_context`, com `source = "draft"`. A tela lê dali a rota que o
motor presumiu para cada inimigo (`src/App.tsx:287-293`) e a mostra como
"`TOP (presumida)`" no quadro.

**Por quê.** A inferência acontece uma vez, no contexto (L-25); a tela só mostra
o que o motor usou.

**Onde.** `lane_analysis` (`lib.rs:280`), `analysis::report`; 12 §2.

### 8.2 A prévia do candidato

**O que acontece.** Clicar num retrato chama `lane_analysis` com o id
(`src/App.tsx:256-260`):
1. `DraftContext::with_candidate` (`core/src/engine/context.rs:285`) põe o
   candidato no seu lugar; o resto fica igual.
2. `report::from_context` (`core/src/engine/report.rs:62`): os dois lados no
   nível 6 (`PREGAME_LEVEL`), sem itens.
3. `lanes::lane_outlooks` (`core/src/engine/lanes.rs:258`): o duelo 1v1 com
   resistências e penetração reais, `0,5 + 0,60 × (duelo − 0,5) + Δmatchup`,
   cortado em [0,28; 0,72]; os níveis 3, 6, 11 e 16; a curva de 1 a 18; a
   vantagem de um nível para cada lado.
4. `lanes::skirmishes` (`core/src/engine/lanes.rs:415`): os 2v2 e 3v3 de que ele
   participa.
5. `wincon::analyze` (`core/src/engine/wincon.rs:179`): fatos de composição, por
   regra; onde a conta de atributos pende; a fase do jogo só com a base.
6. Com a base, a chance do time (`team::seats`, `team::estimate`).

**Por quê.**
- **A prévia é a mesma conta de quando ele estiver travado** (R-6, T-PREVIA-12).
- **Antes da partida ninguém tem vantagem de nível inventada** (T-CASCA-6).
- **A condição de vitória descreve, nunca manda** (C-5,
  [ADR-0018](../adr/0018-o-que-a-condicao-de-vitoria-pode-afirmar.md)). Toda
  afirmação diz de onde sai (T-PREVIA-10, T-PREVIA-11).
- Testes: T-PREVIA-13, T-PREVIA-14.

**Onde.** `from_context`, `lane_outlooks`, `skirmishes`, `analyze`; 11 §15.1,
11 §16.2, 11 §27.1, 11 §27.2,
[03 §4](03-motor-da-partida.md#4-chance-por-rota-e-escaramuça).

**O que pode dar errado.**
- A curva vê atributos e itens, **não habilidades**: a suprema do nível 6 não
  aparece. A tela diz isso.
- O peso `W_DUEL = 0,60` foi escolhido, não ajustado (P-7).
- Sem inimigo travado, não há prévia. Com a rota dele inferida errado, a prévia é
  contra o oponente errado até você marcar (§6.2).

### 8.3 As runas

**O que acontece.** A tela pede `rune_plan` junto com a análise de rota
(`src/App.tsx:232`). `rune_plan` (`lib.rs:382`) refaz `analysis::report`, acha a
linha da sua rota e chama `runes::plan` (`core/src/engine/runes.rs:246`):
- 13 regras sobre (você, ele, rota), cada uma somando peso e motivo à sua runa;
- até 3 principais, por soma;
- a secundária é a primeira de `SECUNDARIAS` que casa;
- `from_data = false`.

**Por quê.**
- **Sai de regras mecânicas, não de winrate:** a base não existe, e a tela não
  pode escrever "segundo N partidas" (R-7, RNF-1).
- **Chave textual, nunca id:** id muda de patch e vira ícone errado (L-17).
- **Alternativas, não ordem** (C-5, C-6). Testes: T-RUNAS-1, T-RUNAS-7.

**Onde.** `rune_plan` (`lib.rs:382`), `plan`; 11 §18.2, 10 §18.4,
[03 §9](03-motor-da-partida.md#9-runas).

**O que pode dar errado.**
- Confiança **regra:** acerta o caso comum e erra o campeão excêntrico.
- A secundária não olha a árvore da principal.
- O app não aplica a runa no cliente (P-15).

## 9. Travar, carregar e a Live Client

### 9.1 O fim do champ select

**O que acontece.** Quem trava é o jogador, no cliente. Quando a fase sai de
`ChampSelect`, a próxima volta passa por `clear_draft` (`watcher.rs:148`):
- esvazia as marcas e a lista da conta (`watcher.rs:151-152`);
- tira o draft e, se havia um, emite `presage://draft` com `null`
  (`watcher.rs:154`).

A tela apaga as marcas dela ao receber o `null` (`src/App.tsx:132`).

**Por quê.** A marca e a lista da conta são daquele champ select; no próximo,
campeões e fila são outros. O app nunca trava (R-3, ADR-0026).

**Onde.** `clear_draft` (`watcher.rs:148`); 12 §3.

### 9.2 Carregando

**O que acontece.** `InProgress` começa na tela de carregamento. Enquanto a Live
Client não responde, cada volta falha em `snapshot` e vira só log de rastreio
(`watcher.rs:326`): nada é emitido. O ramo chama `clear_draft` de novo a cada
volta (`watcher.rs:277`), sem efeito depois da primeira.

**Por quê.** Carregar não é erro. A tela de "Sem partida em andamento" segue até
a primeira leitura.

### 9.3 A leitura de 1 s

**O que acontece.**
- `LiveClient::snapshot` (`live.rs:67`) faz quatro `GET` em sequência em
  `https://127.0.0.1:2999/liveclientdata`, com 3 s de timeout:
  - `/playerlist` e `/activeplayername`: se um falha, a volta inteira falha;
  - `/gamestats`: falha vira `gameTime = 0`;
  - `/eventdata`: falha vira lista vazia, sem timers nem resultado.
- `to_live_game` (`core/src/live.rs:140`): tira os bots, acha o id pelo nome do
  campeão, soma o `price` de cada item em `gold_invested` e marca o jogador local
  pelo Riot ID ou pelo nome de invocador.
- O watcher grava o quadro (§11.1) e monta a `LiveSnapshot` com `timeline`,
  `priority`, `buffs` e `rival` da mesma leitura (`watcher.rs:281-310`).
- Só a mudança vira `presage://live` (`watcher.rs:312-321`). O relógio muda
  sempre, então na prática sai um evento por segundo.

**Por quê.**
- **Toda a fase in-game sem chave:** a Live Client é local e oficial (C-2,
  RNF-5). Os caminhos estreitos são escolha: `allgamedata` a Riot marca como só
  para teste.
- **Os cinco campos da mesma leitura,** para não divergirem na tela (10 §32.3).
- **Ouro em itens é soma exata** (R-12). Testes: T-ENTRADA-14, T-ENTRADA-15,
  T-ENTRADA-17, T-ENTRADA-18, T-ENTRADA-19.

**Onde.** `snapshot` (`live.rs`), `to_live_game`; 11 §4.1, 10 §4, 10 §32.3,
12 §5 (Live Client Data API).

**O que pode dar errado.**
- Sem jogador local (o nome de `/activeplayername` não casa), `me()` é `None`:
  a build não sai, a tela diz "Carregando os dados de item…", não há "sua rota"
  nem notas de ouro.
- Campeão que o índice não conhece fica com id 0.

## 10. A partida na tela

`timeline`, `priority`, `buffs` e `rival` chegam prontos na `LiveSnapshot`. O
resto a tela pede a cada `presage://live`: `build_suggestions` e `gold_report`
(`src/App.tsx:194-220`), `lane_analysis` e `rune_plan` (`src/App.tsx:224-243`).
São quatro `invoke` por segundo na janela principal e três no overlay.

### 10.1 Timers e buffs

**O que acontece.**
- `timeline::build` (`core/src/engine/timeline.rs:319`) passa pela lista inteira
  de eventos: dragões por lado e tipo, Barão, Arauto, Grubs, torres e abates.
  Abate visto vira `at = abate + respawn` e `observed = true`; sem abate, `at` é
  a constante de `Timings` e `observed = false`.
- Inibidor e torre do nexo caídos viram contagem até a volta, nos dois formatos
  de nome.
- `BuffTracker::update` (`core/src/engine/timeline.rs:242`) cria o buff de Barão
  ou Ancião para o time de quem matou. Lido até 5 s depois do abate, `holders`
  são os vivos; depois disso, `None`. Quem morre sai de `holders`.

**Por quê.**
- **Observado é soma exata; estimado é constante, e a tela diferencia** (R-13,
  RNF-1, [03 §6](03-motor-da-partida.md#6-timers)).
- **Nenhum timer de habilidade inimiga** (C-1). O buff é de quem estava vivo no
  abate: nada que o jogador não saberia (C-4).
- **Nome desconhecido não vira timer** (P-14).
- Testes: T-OBJETIVO-10, T-OBJETIVO-11, T-OBJETIVO-12, T-OBJETIVO-13,
  T-OBJETIVO-14.

**Onde.** `build`, `BuffTracker::update`; 11 §29.1, 11 §29.3, 10 §29.

**O que pode dar errado.**
- **Defeito corrigido (A8).** Até a 0.7.0 o buff podia ressuscitar: quando todos
  que o tinham morriam, ele saía da lista, e na leitura seguinte o mesmo abate, que
  continua em `/eventdata`, o recriava com `holders = None`. Agora o rastreador
  lembra os buffs que acabaram assim (`BuffTracker::encerrados`) e não os recria
  (T-OBJETIVO-13).
- As constantes mudam a cada temporada. A 16 trouxe o Barão de volta para 20:00.

### 10.2 Prioridade

**O que acontece.** `priority::report` (`core/src/engine/priority.rs:74`) pontua
`TOP`, `MID` e `BOT`:
1. a escaramuça dos vivos de cada lado, com você somado se não for da rota,
   `score = chance × 100`;
2. `+ min(respawn / 2, 25)` pelo primeiro inimigo morto da rota;
3. `+ 12` em todas, com o jungler deles morto;
4. até `+ 10` pela vantagem de tropas do seu lado na rota;
5. até `+ 15` pelo objetivo da rota a até 90 s.

Corta em [0, 100], guarda três razões e escolhe o foco: o primeiro objetivo a
até 90 s.

**Por quê.** A API não entrega posição, e nenhuma via legal entrega. O Presage
diz **onde está o valor**, nunca onde o inimigo está (R-14, C-4). Um teste varre
as razões procurando linguagem de posição (T-PRIORIDADE-7).

**Onde.** `report`; 11 §30.1, 10 §30,
[03 §7](03-motor-da-partida.md#7-prioridade).

**O que pode dar errado.** O `score` é de exibição: pesos à mão, confiança
**fraca**. Se o overlay com valor calculado ao vivo é aprovado é a P-2.

### 10.3 Sua rota

**O que acontece.** `rival::report` (`core/src/engine/rival.rs:80`): antes de
1:30 não há painel. O oponente é o inimigo da mesma posição. A métrica sai de
`cobranca`: tropas por minuto para quem farma, visão por minuto para o suporte.
`standing` (`core/src/engine/rival.rs:69`) põe você à frente ou atrás pelo ouro
em itens, com margem `max(300, 10% da média dos dois)`.

**Por quê.**
- **O suporte nunca é cobrado por tropas** (L-27, R-11).
- **Atrás ou à frente sai do ouro em itens,** que é exato. A margem é regra.
- Testes: T-RIVAL-1 a T-RIVAL-5.

**Onde.** `report`, `standing`; 11 §22, 10 §22.

**O que pode dar errado.** O texto da tela copia o 300 e o 10% à mão
([GC-9](../frontend/07-lacunas-da-casca.md)).

### 10.4 Build

**O que acontece.** `build_suggestions` (`lib.rs:247`) vira você e os inimigos em
`Combatant` e chama `build::suggest` (`core/src/engine/build.rs:228`):
1. o seu `Loadout`: atributos base no nível, pela curva do jogo, mais os itens;
2. o time deles: a divisão física do dano que vem, a maior penetração de cada
   tipo, os `Loadout` dos alvos;
3. para cada item que você não tem: quanto ele sobe vida efetiva × dano
   efetivo, por 1000 de ouro, com até duas razões;
4. a lista em ordem, cortada em 8 na janela e 2 no overlay.

**Por quê.**
- **A parte do produto que não é estatística:** dado o item que eles compraram,
  a conta é exata (R-10).
- **A build varia com a composição deles** (C-6): time físico puxa armadura, o
  deles comprar armadura sobe penetração na sua lista. Teste: T-BUILD-1.
- **Penetração na ordem do jogo** ([03 §2](03-motor-da-partida.md#2-build-adaptativa)).

**Onde.** `suggest`, `loadout`, `incoming_physical_share`; 11 §26, 10 §26.

**O que pode dar errado.** O lado ofensivo é proxy, não DPS: sem razões de
habilidade, recarga ou crítico. Não trata caminho de construção, botas nem
passiva de item.

### 10.5 Rotas e escaramuças

**O que acontece.**
- Com partida, `lane_analysis` ignora o draft: `from_live` (`analysis.rs:23`)
  monta os dois lados pela posição que a Live Client dá para os dez, com níveis e
  itens reais, e chama `report::build` com `source = "partida"`.
- Para o overlay, `skirmish_focus` (`lib.rs:311`) chama `lanes::focus`
  (`core/src/engine/lanes.rs:497`): o objetivo a até 90 s escolhe o grupo (Dragão
  puxa `dragao`, Barão puxa `arauto`); senão, a dupla da sua rota; senão, a
  escaramuça mais desequilibrada.

**Por quê.** A partida ganha do draft: lá os itens e os níveis são reais
(T-CASCA-1). Rota sem oponente fica de fora em vez de virar 50% (T-ROTA-1). Sem
base, o número diz que vem só do duelo (T-ROTA-2). Por cima do jogo cabe uma
escaramuça, e ela tem de ser a certa (R-15, T-ROTA-13, T-ROTA-14).

**Onde.** `analysis::report`, `focus`; 11 §16.1, 11 §27.3, 10 §27.

**O que pode dar errado.** Posição em branco (ARAM, partida sem posição) fica de
fora e liga `roles_assumed`.

### 10.6 Ouro

**O que acontece.** `gold_report` (`lib.rs:333`) chama `gold::report`
(`core/src/engine/gold.rs:89`):
- **investido**, a soma do preço dos itens: exato;
- **ganho**, estimado por `earned_at` (`core/src/engine/gold.rs:76`): 500 de
  início, renda passiva desde 1:50, 21 por tropa, 300 por abate, 150 por
  assistência, nunca abaixo do investido;
- `unspent = max(0, ganho − investido)`, só para você; `None` para os outros nove;
- até três notas: a diferença de itens a partir de 1000, o **seu** ouro parado a
  partir de 1500, o inimigo mais adiantado.

**Por quê.** A API só entrega o ouro em mão do jogador local. Ouro em mão de outro
jogador nunca aparece (R-12, C-4). Testes: T-OURO-1 a T-OURO-10, e TF-PARTIDA-3 na
tela.

**Onde.** `report`, `earned_at`; 11 §28, 10 §28,
[03 §5](03-motor-da-partida.md#5-ouro).

**O que pode dar errado.**
- **Defeito corrigido (A3).** Até a 0.7.0, `unspent` saía para os dez e a tela
  mostrava o `+N` de qualquer jogador com 800 ou mais, inimigos inclusive. Agora o
  core manda `None` para os outros nove e a tela só desenha a sua linha
  ([GC-2](../frontend/07-lacunas-da-casca.md)). O overlay nunca mostrou a coluna.
- A estimativa é um piso: torres, placas e objetivos ficam de fora.

### 10.7 O overlay

**O que acontece.** `src/overlay.tsx` pede `get_overlay_prefs` e `get_live` ao
abrir. A cada `presage://live`, pede juntos `build_suggestions` (limite 2),
`gold_report` e `skirmish_focus`. Mostra até seis seções: ouro em itens, dano
contra você, objetivos, prioridade, a escaramuça e o próximo item. Com as seis
escondidas, não desenha nada (`src/overlay.tsx:94`).

**Por quê.**
- **Janela externa, transparente, sempre no topo, atravessável até você pedir**
  (R-15, C-3, [ADR-0003](../adr/0003-overlay-como-janela-externa.md)).
- **Você escolhe o que aparece,** e a escolha sobrevive a reiniciar
  (`overlay.json`). `set_overlay_prefs` avisa as duas janelas por
  `presage://prefs`. Anúncio nunca (C-9).

**Onde.** `src/overlay.tsx`, `overlay.rs`; 12 §7, 10 §8.2,
[frontend 04](../frontend/04-overlay.md),
[03 §10](03-motor-da-partida.md#10-overlay).

**O que pode dar errado.**
- Em tela cheia exclusiva nenhuma janela externa aparece por cima: o jogo
  precisa estar em janela sem bordas.
- "Não desenhar" não é "esconder": com a janela clicável, um retângulo invisível
  de 300 × 340 continua recebendo cliques.
- O 6 de `hidden.length >= 6` é escrito à mão
  ([GC-11](../frontend/07-lacunas-da-casca.md)).

## 11. Gravar, fechar e ler

### 11.1 A gravação

**O que acontece.** A cada volta de `InProgress`, **antes** de montar o resto,
`recorder::observe` (`recorder.rs:29`, chamado em `watcher.rs:291`):
- guarda o `LiveGame` enquanto houver menos de 6000 quadros (`MAX_FRAMES`,
  `recorder.rs:23`). `Recording::push` descarta o quadro cujo relógio não andou
  (`core/src/engine/coach.rs:81`);
- se o resultado ainda não é conhecido, procura um `GameEnd` (`Win` ou `Lose`)
  nos eventos. O primeiro vence.

**Por quê.**
- **O pós-jogo só tem o que foi gravado enquanto o jogo acontecia** (R-16). A
  timeline da match-v5 só viria com chave de produção.
- **O teto** existe para uma sessão esquecida aberta não encher o disco.
- **Nada sai da máquina** (RNF-8).
- Testes: T-CASCA-15, T-CASCA-16, T-CASCA-17, T-POS-1.

**Onde.** `observe` (`recorder.rs`), `Recording::push`; 11 §31.2, 10 §31.1,
12 §7 (Gravação da partida).

**O que pode dar errado.**
- Abrir o app no meio da partida começa a gravação tarde. O pós-jogo divide pelo
  relógio da partida, não pelo tempo gravado, e não comenta o item que já estava
  comprado no primeiro quadro (A9, corrigido; 11 §31.1). O que a gravação não
  viu, como mortes antes dela, fica fora das observações de morte.
- Se a Live Client parar antes do `GameEnd`, `won` fica `None` e a tela não diz
  vitória nem derrota.

### 11.2 O fim da partida

**O que acontece.** A fase sai de `InProgress` (para `WaitingForStats`,
`EndOfGame` ou qualquer outra) e a volta cai no ramo ocioso (`watcher.rs:330`):
1. zera o `BuffTracker` (`watcher.rs:332`);
2. tira a partida do `Shared` e emite `presage://live` com `null`
   (`watcher.rs:333-334`);
3. `close_recording` (`watcher.rs:129`): sem quadros, sai cedo; senão,
   `recorder::finish` (`recorder.rs:44`):
   - tira a gravação do `Shared`, deixando-a vazia em todos os casos;
   - se a gravação não vale leitura (`coach::worth_reviewing`: menos de 60
     quadros ou menos de 5 minutos de partida), devolve `None` e não grava nem
     emite nada;
   - senão, roda `coach::review` (§11.3), grava `gravacao.json` e
     `ultima-partida.json` (`recorder.rs:53`, `recorder.rs:56`);
4. emite `presage://review` (`watcher.rs:143`) e guarda em `last_review`.

**Por quê.** Restos de uma partida contaminariam a próxima (T-CASCA-18). Remake,
custom de teste ou gravação que começou no fim não valem uma leitura.

**Onde.** `close_recording` (`watcher.rs`), `finish` (`recorder.rs`); 12 §3,
12 §6, 10 §31.6.

**O que pode dar errado.**
- **Defeito corrigido (A6).** Até a 0.7.0 o piso da gravação eram 60 quadros e o
  do coach 300 segundos: um remake de 3:30 passava no primeiro, a leitura vazia
  "curta demais" substituía a anterior e a aba Pós-jogo vinha para a frente.
  Agora os dois pisos moram em `coach::worth_reviewing` (T-CASCA-19).
- Falha de disco ao gravar os dois arquivos vira só aviso no log
  (`recorder.rs:53-58`).
- A queda e volta do cliente no meio da partida passa por aqui (§4.3): a leitura
  sai da metade gravada, e a outra metade começa uma gravação nova.

### 11.3 A leitura

**O que acontece.** `coach::review` (`core/src/engine/coach.rs:215`):
1. o jogador local e a rota saem do último quadro. Sem ele, "Sem gravação desta
   partida."; com menos de 300 s gravados, "curta demais".
2. as curvas (`Series`): ouro em itens do time, tropas (visão para o suporte),
   nível, reamostrados a cada 30 s; mortes e abates do time com o instante exato.
3. as observações que a `cobranca` da rota pede (`core/src/engine/coach.rs:51`):
   farm ou visão contra o oponente direto, participação, tempo morto, mortes em
   sequência, primeiro item grande, ouro parado, vantagem que sumiu, nível.
4. "A corrigir" e "A manter", cada lista da mais pesada para a menos, e o resumo.

**Por quê.**
- **Cada observação é um número que você pode conferir na aba de pontuação,** e
  nada vira nota (R-16, C-7, RNF-11).
- **A rota decide o que se cobra:** o pós-jogo mandava o suporte farmar (L-27).
- Testes: T-POS-2, T-POS-3, T-POS-13, T-POS-15.

**Onde.** `review`, `cobranca`; 11 §31.1 (a tabela de observações e limiares),
10 §31, [03 §8](03-motor-da-partida.md#8-leitura-pós-partida).

**O que pode dar errado.** Todo limiar é regra. O ouro parado vem da renda
reconstruída por `earned_at`: confiança **fraca**, e a frase diz que é
estimativa.

### 11.4 A aba Pós-jogo

**O que acontece.** O ouvinte de `presage://review` guarda a leitura e troca a
aba para Pós-jogo (`src/App.tsx:125-128`). `PostGame` mostra resumo, observações
e o gráfico; `PerfilDeRota`, embaixo, o perfil (§12.1). Na próxima abertura, o
`setup` relê `ultima-partida.json` (`lib.rs:636`) e a tela a pede com
`last_review` (`src/App.tsx:108`).

**Por quê.** É exatamente a hora em que a leitura interessa.

**Onde.** `src/App.tsx`, `src/components/PostGame.tsx`,
`src/components/GraficoPartida.tsx`; 12 §3;
[frontend 02](../frontend/02-telas-e-fluxos.md).

**O que pode dar errado.** A aba pula para a frente também na leitura vazia do
remake (§11.2).

## 12. Depois da partida

### 12.1 Histórico pessoal e perfil de rota

Roda na tarefa 3 do §3.4, uma vez por abertura do app, e não depois de cada
partida.

**O que acontece.** Com a chave e a conta (§3.5), `profile::refresh`
(`profile.rs:32`):
1. lê `personal-v2.json`; um `puuid` diferente zera o histórico
   (`profile.rs:48`);
2. `ranked_match_ids`: as últimas 60 ranqueadas solo (fila 420,
   `MATCHES_PER_REFRESH`, `profile.rs:25`);
3. para cada uma que `seen` não tem, `match_detail`, com 1,3 s antes de cada
   chamada (`CALL_SPACING`, `riot.rs:24`);
4. `history::played` (`core/src/history.rs:182`): corta modo que não é
   `CLASSIC`, partida não terminada, remake e menos de 300 s; acha o oponente
   direto pelo `team_position`; calcula você menos ele em cinco métricas;
5. `add_played` soma; a partida que não conta é `mark_seen`, para não ser
   buscada de novo;
6. um erro da Riot para o laço e guarda o que deu (`profile.rs:89-93`);
7. grava `personal-v2.json` se algo entrou ou foi descartado (`profile.rs:97`).

Na volta, `rebuild_stats` (`lib.rs:89`) troca a loja inteira e
`presage://engine` faz a tela pedir `lane_profile` de novo
(`src/App.tsx:118-121`). `lane_profile` recalcula o perfil a cada chamada:
`profile::build` (`core/src/engine/profile.rs:103`) junta as diferenças por
métrica, encolhe cada campeão para o geral e cada confronto para o campeão
(DerSimonian–Laird), e só ganha cor o que `Reading::is_clear` aceita: o valor
acima de `1,96 × sd`.

**Por quê.**
- **O oponente direto é o grupo de controle:** vitória e derrota são o sinal mais
  pobre da partida (L-6, R-17,
  [ADR-0010](../adr/0010-oponente-direto-como-grupo-de-controle.md)).
- **Remake e partida curta não contam,** nem como vitória (L-5).
- **Arquivo novo:** o `personal.json` antigo marcava partidas como vistas sem a
  comparação de rota (L-9).
- **Razão, não peso; nada de nota única** (C-7, RNF-11,
  [ADR-0011](../adr/0011-perfil-de-rota-e-razao-nao-peso.md)); um efeito de z ≈ 1
  ainda não ganha cor (L-19).
- **A chave vai no cabeçalho, nunca na URL** (RNF-6).
- Testes: T-STATS-11, T-STATS-12, T-STATS-13, T-PERFIL-1, T-PERFIL-3.

**Onde.** `refresh` (`profile.rs`), `played`, `PersonalStats::add_played`,
`profile::build`; 11 §5.1, 11 §19.9, 11 §21, 10 §5, 10 §21,
[04](04-perfil-de-rota.md).

**O que pode dar errado.**
- Os nomes de campo de `challenges` ainda não foram conferidos contra uma partida
  real (P-5).
- A chave de desenvolvimento expira; 401 e 403 param a atualização com o que deu
  (P-6).
- Os cortes do histórico e do coletor estão escritos duas vezes e divergem na
  rota em branco (R-20, [05 §5](05-coletor.md#5-a-duplicação-com-historyrs)).
- O `±` que dá a cor é recalculado na tela
  ([frontend 03](../frontend/03-estado-e-atualizacao.md), §9).

### 12.2 O que fica em disco

Tudo em `%LOCALAPPDATA%\Presage`. A lista completa, com quem lê cada arquivo, está
em 12 §6; a forma de cada um, em 10 §33.

| Passo | Arquivo | Quem grava |
|---|---|---|
| Ajustes e pool, a qualquer hora | `pool.json`, `draft.json`, `overlay.json` | `set_pool`, `set_draft_prefs`, `set_overlay_prefs`, primeiro o disco, depois a memória |
| §3.4, a cada abertura com rede | `champions.json`, `items.json`, `runas.json` | `load_champions`, `load_items`, `load_runes` |
| §12.1, quando algo entrou | `personal-v2.json` | `profile::refresh` |
| §11.2, partida com 60 quadros ou mais | `gravacao.json`, `ultima-partida.json` | `recorder::finish` |
| fora do app | `base-de-partidas.json` | o coletor, nunca o app |

O que sai do processo: o LCU e a Live Client no loopback, o Data Dragon, a Riot
Web API (só com a chave) e o endpoint do atualizador. Nenhuma gravação nem
leitura sai da máquina (RNF-8).

**O que pode dar errado.**
- `write_json` grava em `.parcial` e renomeia: um corte no meio deixa o arquivo
  anterior inteiro (A7; antes, deixava JSON quebrado, lido como arquivo que falta).
- `gravacao.json` guarda o `displayName` dos dez em cada quadro (10 §33).

## 13. Uma partida do começo ao fim

**Estes são dados de exemplo.** A partida abaixo é a de `src/demo.tsx`, escrita
à mão para mostrar a interface sem o cliente da Riot. A própria página diz isso
numa faixa ("Dados de exemplo…", `src/demo.tsx:981-984`). Os números têm tamanho
realista, não são a partida de ninguém, e vários não fecham com as funções que
os produziriam (§13.8). Uma partida real, gravada e anonimizada, é a P-17.

O exemplo mostra o app **com a base de partidas carregada** (`hasGlobal: true`,
patch 16.21, 1.240.000 partidas; `src/demo.tsx:90-102`). O app real ainda não
tem base (P-1, P-6). Sem ela, todo motivo com "n=…" some, o ranking sai da
composição e do histórico pessoal, e a tela diz "Sem base estatística".

### 13.1 Antes do draft

- **A barra:** patch 16.21.1, 171 campeões, motor pronto. É o `EngineStatus` que
  `engine_status_of` monta (`lib.rs:61`) depois da carga do Data Dragon (§3.4).
- **A conta:** 47 partidas no histórico pessoal. Só existem com
  `PRESAGE_RIOT_API_KEY` (§3.5).
- **A pool:** vazia de propósito (`src/demo.tsx:740`), para a aba Pool abrir no
  construtor. O plano de exemplo (`src/demo.tsx:746-828`) retrata três lutadores
  de dano físico no topo: fortes em "3 de 3 com dano físico", fracos em dano
  mágico, linha de frente, alcance e estilo. Os candidatos são Malphite (0,71),
  Gragas (0,66), Ornn (0,63), Kennen, Mordekaiser, Sion, Teemo e Sylas. No app,
  isso é `pool_suggestions` → `poolgap::analyze` e `poolgap::suggest` (R-9,
  [ADR-0016](../adr/0016-pool-de-oito-tres-escolhidos-cinco-por-cobertura.md)).

### 13.2 O draft, com 21 s no relógio

A ranqueada está em `BAN_PICK`, na sua vez de escolher, e você é o meio
(`src/demo.tsx:104-130`).

| Na tela | No exemplo | Quem produz |
|---|---|---|
| Relógio | 21 s de 30 | `timeLeftMs` de `to_draft_state`, interpolado pela tela (`src/App.tsx:306-310`) |
| Seu time | Garen no topo, Lee Sin na selva, você no meio com Ahri em hover, Ashe e Thresh | `to_draft_state`; o hover é `intentId` |
| Inimigos | Malphite, Viego e Zed travados, sem posição; dois slots vazios | `to_draft_state`: o campeão deles sai do `pick` concluído (L-10) |
| Rotas presumidas | Malphite no topo, Viego na selva, Zed no meio | a inferência de `new_with` (§6.2), lida de `lane_analysis` (§8.1) |
| Bans | Yasuo do seu lado; Lux e LeBlanc do deles | `my_bans`, `their_bans` |
| Sua pool | Ahri 53,4% ±4,1 (n=3120), Orianna 52,1%, Syndra 50,9% | `rank_with`, separado por `inPool` (`RecommendationPanel.tsx:98`) |
| Melhores no geral | Annie 54,2%, Veigar 53,0%, Zoe 51,5%, Sylas 50,4% | `rank_with`, até 15 (`OVERALL_COUNT`) |
| Menos expostos | Orianna −0,7, Annie −1,9, Ahri −2,2, Syndra −2,8, Zoe −3,7 | `minimax_with`, reordenado por `exposure` (§7.2) |
| Base | "Base do patch 16.21, 1.240.000 partidas." | `EngineStatus.globalPatch` e `globalMatches` |

Por que a lista é essa:
- **Ahri está em hover e continua candidata:** a sua escolha não travada
  continua sua (L-26). Garen, Lee Sin, Ashe, Thresh e os três deles estão em
  `unavailable`.
- **O número embaixo do retrato** é o `delta`: 53,4% − 50% = +3,4 pontos.
- **Os três motivos da Ahri saem de três componentes:**
  - "Vence Zed na rota (+2.8pp, n=1840)" é o matchup contra o oponente de rota,
    com o Zed **presumido** no meio, então com peso `w` menor que 1. Só existe
    com a base.
  - "Seus jogos: 18V-9D (+2.1pp)" é o componente pessoal, encolhido para a base
    da Ahri ([ADR-0012](../adr/0012-vazamento-da-fase-da-conta-mantido.md)).
  - "Equilibra o dano: 78% → 62% físico", da Orianna, é a composição: com
    Garen, Lee Sin e Ashe, o seu time é quase todo físico.
- **Os bans** estão nos dados (`src/demo.tsx:248-253`), mas não aparecem: a ação
  atual é `pick`, e a seção só abre em `ban` (§7.3). Na fase de ban viriam Sylas,
  "Da sua lista de bans", e depois Katarina (3,1 pontos de ameaça), Fizz e Ekko,
  "Completa o que falta na composição deles".
- **O detalhe da Ahri** diz "Melhor resposta deles custa 2.2pp (Katarina)": o
  valor do time cai de 54,0% para 51,8% depois da pior resposta.

### 13.3 A prévia de um candidato

Clicar na Orianna abre o detalhe e pede a prévia. No exemplo, `previaDemo`
(`src/demo.tsx:623-634`) troca a Ahri pela candidata no meio, marca
`source = "draft"` e `rolesAssumed = true`, e a tela avisa que as rotas deles
foram inferidas.

No app, a prévia é `lane_analysis` com o id dela (§8.2): o mesmo contexto, com a
Orianna em `me`, todos no nível 6 e sem itens. A curva de 1 a 18 contra o Zed, as
brigas "2v2 e 3v3 com ela" e a condição de vitória são a mesma conta de quando ela
estiver travada (T-PREVIA-12). A "Chance do time com ele" só aparece com a base;
no exemplo, `team` é `null`.

### 13.4 Rotas e runas

O relatório de rotas do exemplo (`src/demo.tsx:445-619`) é o da partida
(`source = "partida"`), com níveis e itens reais.

| Rota | Confronto | Chance | Por quê, no exemplo |
|---|---|---|---|
| TOP | Garen × Malphite | 43% | "Perde para Malphite na rota (−3.1pp, n=2410)", "Perde o 1v1 com os itens de agora (45%)" |
| JG | Lee Sin × Viego | 56% | "Vence Viego na rota (+2.4pp, n=1920)", 1v1 a 58% |
| MID | Ahri × Zed | 47% | o matchup a favor (+2.8pp), o 1v1 contra (41%), "Ahri tem alcance; Zed precisa chegar perto" |
| ADC | Ashe × Jhin | 41% | o matchup e o 1v1 contra |
| SUP | Thresh × Lulu | 62% | o matchup a favor (+4.0pp), 1v1 a 66% |

- **A chance da rota** é `lane_outlooks`: o duelo pesa 0,60, o matchup da base
  entra encolhido, e o corte em [0,28; 0,72] diz que nenhuma rota está decidida
  (§8.2).
- **As escaramuças** são `skirmishes`: topo 48%, meio 46%, bot 53%, dragão 61%,
  arauto 45%.
- **A condição de vitória** é `wincon::analyze`. O fato "Eles: 72% do dano é
  físico — armadura responde a quase tudo" é regra: 70% ou mais de um tipo. As
  vantagens (Thresh sobre Lulu, o topo de Malphite sobre Garen) são a conta de
  atributos, marcadas como tal. `phase` é `null`, e o aviso fixo diz que a fase
  só entra com a base.
- **A runa** é `runes::plan` para "Ahri contra Zed": Correria Fásica (0,95,
  "contra quem explode, sair da briga vale mais que somar dano"), Cometa Arcano
  (0,90) e, na secundária, Revestimento Ósseo. `fromData = false`: a tela escreve
  que sai de regras, não de winrate (§8.3).

### 13.5 A partida aos 19:07

`gameTime = 1147` (`src/demo.tsx:279-385`). Você é a Ahri, nível 13, 5/2/6, 168
tropas.

| Na tela | No exemplo | Quem produz |
|---|---|---|
| Sua rota | MID contra Zed: 7,4 contra 8,1 tropas por minuto, −1.950 em itens, atrás | `rival::report`; a margem é `max(300, 10% da média de 4.150 e 6.100)` = 512 |
| Objetivos | Dragão em 0:43 e Barão em 5:13, os dois observados | `timeline::build`: dragão morto aos 14:50 + 5:00; Barão aos 18:20 + 6:00 |
| Estrutura | Inibidor do meio, deles, volta em 2:23 | `Respawn`: caído aos 16:30 + 5:00 |
| Buff | Barão do seu time, 2:13, com Ahri, Garen e Ashe | `BuffTracker::update`: 18:20 + 3:00; Lee Sin e Thresh já não estão em `holders` |
| Placar | dragões 2–1 (Fire, Ocean, Cloud), torres 3–1, abates 11–8 | `timeline::build` |
| Prioridade | BOT 78,4 ("3v2: 61% para o seu lado", "Jungler inimigo morto por 23s", "Dragão em 43s"), TOP 61,2, MID 54,8 | `priority::report` |
| Ouro | −2.950 em itens: 19.200 contra 22.150 | `gold::report` |
| Build | 81% do dano que vem é físico; vida efetiva 3.240; 35% de penetração de armadura e 18 de letalidade deles | `build::suggest`, o `BuildContext` |
| Próximo item | Zhonya 6,8 por 1000 de ouro, Rylai 5,4, Cajado do Vazio 4,9, Rabadon 3,6 | `build::suggest`, a lista |

Por que a tela mostra isso:
- **Os timers observados são somas exatas:** o abate visto mais a constante de
  `Timings` (§10.1). Nenhum é estimado aos 19 minutos.
- **O BOT vale mais agora:** o Viego está morto por 23 s, o dragão nasce em 43 s,
  e três dos seus (Ashe, Thresh e você) contra dois deles dão 61%. As razões
  dizem o valor, nunca onde o inimigo está (C-4).
- **A build reage aos itens deles:** "81% do dano deles é físico", "Contra 18 de
  letalidade, vida rende mais que armadura", "Eles têm 35% de penetração de
  armadura — armadura rende menos".
- **O ouro em itens é exato:** Garen 4.100 + Lee Sin 4.100 + Ahri 4.150 + Ashe
  4.650 + Thresh 2.200 = 19.200; Malphite 4.400 + Viego 4.400 + Zed 6.100 + Jhin
  6.750 + Lulu 500 = 22.150. As notas: "Seu time está 2950 atrás em itens", "Você
  tem ~1800 de ouro para gastar", "O mais adiantado deles é Jhin (6750 em
  itens)".
- **O ouro em mão é só o seu:** "+1.800" ao lado da Ahri. O exemplo manda
  `unspent: null` para os outros nove, como a casca (§10.6, GC-2).

### 13.6 O overlay

A demonstração não desenha o overlay; Ajustes só mostra as preferências. Com esta
mesma leitura e tudo visível, ele mostraria:
- **ouro:** −2.950, em vermelho, sem a coluna de ouro em mão;
- **dano:** 81% físico, vida efetiva 3.240, 35% de penetração de armadura deles;
- **objetivos:** Dragão 0:43 e Barão 5:13;
- **prioridade:** BOT, com "3v2: 61% para o seu lado";
- **escaramuça:** `lanes::focus` acha o Dragão a 43 s, dentro dos 90 s, e escolhe
  o grupo `dragao`, JG+ADC+SUP, a 61%, porque "Dragão em 0:43";
- **próximo item:** Zhonya (3.250) e Rylai (2.600), com o primeiro motivo.

### 13.7 O fim, aos 36:24

A partida acabou em derrota (`won: false`, de um `GameEnd` com `Lose`). A aba
Pós-jogo vem para a frente com a leitura (`src/demo.tsx:692-735`): "36:24 de
Ahri · 3 a corrigir, 2 a manter."

| Observação | No exemplo | A regra de `coach::review` |
|---|---|---|
| A corrigir | "4 mortes entre 21:40 e 24:10": 4 das 7 | 3 ou mais mortes em 240 s; peso 4/7 = 0,57 |
| A corrigir | "5,4 tropas por minuto contra 7,1": 62 tropas, ~1.302 de ouro | meio cobra farm; ele 0,8/min ou mais à frente; 21 de ouro por tropa |
| A corrigir | "3:10 com ouro parado", chegou a ~2.400 | renda estimada menos ouro em itens ≥ 1300, vivo, por 120 s ou mais; peso 190/600 = 0,31 |
| A manter | "71% dos abates do time": 17 de 24 | o meio é forte a partir de 65% |
| A manter | "Primeiro item grande aos 10:50" | item de 2500 ou mais até 11:00 |

O gráfico conta a mesma história: a vantagem de ouro do time sobe até perto de
+3.350, por volta dos 21:30, e cai; as sete mortes (10:12, 21:40, 22:20, 23:22,
24:10, 26:52 e 31:20) são marcas na linha do tempo. Cada frase do relatório pode
ser conferida na curva ou na aba de pontuação, e nada vira nota (C-7).

Embaixo, o perfil de rota (`src/demo.tsx:876-927`), com 40 partidas comparadas ao
oponente direto:
- **No geral:** tropas aos 10 +3,1 ±1,4 (34 partidas) é claro, porque 3,1 passa de
  1,96 × 1,4 = 2,7, e ganha cor. Participação +2,0 ±1,6 não é. Visão −0,2 ±0,06
  é, para o lado ruim.
- **Ahri, 22 partidas:** tropas aos 10 a +6,2, depois de encolher a média crua de
  +6,9 para o geral.
- **Ahri contra Zed, 4 partidas:** tropas aos 10 a −4,0 ±3,5, encolhido de −9,5.
  Não é claro e fica sem cor. Contra Fizz, com uma partida, o confronto nem
  aparece: a tela só mostra confronto com duas ou mais.

### 13.8 Onde o exemplo não fecha com o código

Os dados de exemplo provam que a tela monta, não que a leitura acerta.

| Tela | O exemplo diz | O código daria |
|---|---|---|
| Draft, detalhe | a melhor resposta à Ahri é Katarina; à Orianna, Zed; à Zoe, Lee Sin | `minimax_with` só responde com quem está fora de `unavailable` e joga uma rota deles ainda livre. Com Malphite, Viego e Zed no topo, na selva e no meio, sobram bot e suporte. Zed e Lee Sin já estão no draft |
| Partida, build | Zhonya em primeiro | a Ahri já tem o 3157, e `build::suggest` só mede item que você não tem (11 §26.1) |
| Partida, sua rota | 7,4 contra 8,1 tropas por minuto | 168 tropas aos 19:07 dão 8,8; as 142 do Zed dão 7,4 (`rival::report`) |
| Partida, prioridade | BOT 78,4, MID 54,8 | BOT: 61 + 12 (jungler morto) + 7,8 (dragão em 43 s) = 80,8. MID: 43 + 12 + 10, pelas 26 tropas a mais, = 65, com a razão "Eles estão 26 de CS atrás na rota" |
| Partida e Pós-jogo | −2.950 em itens aos 19:07 | a curva de ouro do mesmo jogo passa de +2.900 nesse instante |
| Pós-jogo | nenhuma observação de vantagem perdida | a curva vai a ~+3.350 e fecha negativa: pico ≥ 3000 e fim ≤ pico − 3000 é a observação ruim de `lead_swing` (T-POS-15) |

Uma gravação real, com Riot IDs trocados por nomes fictícios, viraria fixture e
fecharia esta tabela. A decisão é a P-17.

## 14. Para continuar

| Para saber mais sobre | Leia |
|---|---|
| Cada tipo, campo e unidade | [10](10-modelo-de-dados.md) |
| Cada função: assinatura, erros, confiança, requisito | [11](11-catalogo-de-funcoes.md) |
| Comandos, eventos, fontes e arquivos da casca | [12](12-contrato-da-casca.md) |
| Os testes que provam cada passo, por T-ID | [13](13-testes.md) |
| As fontes de dado e o que cada uma não dá | [00](00-contexto-e-limites.md) |
| Core e casca, e as fronteiras entre JSON cru e domínio | [01](01-core-e-casca.md) |
| Ranking, shrinkage, resposta, bans e pool | [02](02-motor-de-draft.md) |
| Build, dano, rotas, ouro, timers, prioridade, pós-jogo, overlay | [03](03-motor-da-partida.md) |
| O perfil de rota | [04](04-perfil-de-rota.md) |
| O coletor e a base de partidas | [05](05-coletor.md) |
| Instalador e atualizador | [06](06-distribuicao-e-atualizador.md) |
| Segurança e entradas não confiáveis | [07](07-seguranca.md) |
| Cada defeito do protótipo e o que o criou | [09](09-licoes-do-prototipo.md) |
| O que cada tela mostra e quando | [frontend 02](../frontend/02-telas-e-fluxos.md) |
| O que a casca ainda não entrega à tela | [frontend 07](../frontend/07-lacunas-da-casca.md) |
| Os requisitos e a matriz de rastreabilidade | [requisitos](../produto/requisitos.md) |
| O que ainda depende da Riot ou de dado | [perguntas em aberto](../produto/perguntas-em-aberto.md) |
