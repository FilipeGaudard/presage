# 02 — Telas e fluxos

Cada aba diz o que mostra, de qual comando ou evento vem cada dado, o que cada
controle chama e os estados, com o texto exato do código entre aspas. O que a
tela precisa e a casca não entrega aparece como `GC-n`, descrito em
[07](07-lacunas-da-casca.md). Quando um dado é refeito está em
[03](03-estado-e-atualizacao.md); o overlay está em [04](04-overlay.md).

## 1. Navegação

```
┌──────────────────────────────────────────────────────────────────────────────┐
│ ⑂ PRESAGE  patch 16.21.1 · v0.7.0   Draft Simulador Rotas Partida• Pós-jogo  │
│                                     Pool Ajustes          ● Seleção de campeões│
├──────────────────────────────────────────────────────────────────────────────┤
│ Versão 0.8.0 disponível …                     [Atualizar e reiniciar] [Depois]│
├──────────────────────────────────────────────────────────────────────────────┤
│ conteúdo da aba                                                              │
└──────────────────────────────────────────────────────────────────────────────┘
```

As abas são a união `View` de `App.tsx:50`. A aba abre em `draft`
(`App.tsx:53`) e não é lembrada entre execuções. Só uma coisa muda de aba
sozinha: o evento `presage://review`, que leva para o Pós-jogo quando a leitura
da partida fica pronta (`App.tsx:125-128`), em qualquer aba em que o jogador
esteja. A bolinha ao lado
de "Partida" acende enquanto há `live` (`App.tsx:363`).

| Aba | `View` | Componente(s) | Requisitos |
|---|---|---|---|
| Draft | `draft` | `DraftBoard`, `RecommendationPanel` | R-1, R-2, R-3, R-4, R-5, R-6 |
| Simulador | `simulador` | `SimuladorDraft`, `RecommendationPanel` | R-8, R-6 |
| Rotas | `lanes` | `LaneAnalysis`, `Runas` | R-11, R-7 |
| Partida | `live` | `LiveOverview`, `GoldTracker`, `LiveBuild` | R-10, R-11, R-12, R-13, R-14 |
| Pós-jogo | `coach` | `PostGame` (com `GraficoPartida`), `PerfilDeRota` | R-16, R-17 |
| Pool | `pool` | `PoolBuilder` | R-9, R-3 |
| Ajustes | `overlay` | `OverlayPrefsPanel` e a seção "App" escrita em `App.tsx` | R-15, R-19 |

`demo.tsx:864` repete a união `View` e `demo.tsx:929-937` repete os rótulos das
abas: são duas cópias da navegação.

## 2. Draft

```
┌─────────────────────────────────────┬──────────────────────────────────────────┐
│ Seleção de campeões             21s │ Recomendações     [ ] incluir quem você  │
│ BAN_PICK · ESCOLHENDO · você: MID   │ melhores picks · 42 candidatos  não tem  │
│ ┌ É a sua vez de escolher. ───────┐ │ Sua rota: TOP JUNGLE MID ADC SUPORTE     │
│ Seu time         Inimigos           │ Menos expostos ao contra-pick            │
│ ▣ Ahri  MID·você ▣ Malphite         │  ▣ −1.2  ▣ −1.8  ▣ −2.0  ▣ −2.4  ▣ −3.1   │
│ ▣ Lee Sin JG       [TOP (presumida)▾]│ Sua pool       ✳▣ 3.4  ✳▣ 2.1            │
│ ▣ Aguardando     ▣ Zed  [MID ▾]     │ Melhores no geral ▣ 2.8 ▣ 1.9 ▣ 0.7 …    │
│ Bans aliados   ▣ ▣                  │ ┌ Ahri  52.3% ±4.1 · n=… ─────────────┐  │
│ Bans inimigos  nenhum               │ │ motivos · Melhor resposta deles custa│  │
│                                     │ │ curva 1–18 · 2v2 e 3v3 · condição    │  │
└─────────────────────────────────────┴──────────────────────────────────────────┘
```

**Mostra:** os dois times com retrato, nome e rota; o relógio; os bans; à
direita, o ranking em duas grades (a sua pool e os melhores no geral), os menos
expostos ao contra-pick, a sugestão de ban na fase de ban e o detalhe do
candidato clicado, com a prévia.

| Dado | Comando ou evento |
|---|---|
| Draft e conexão na abertura | `get_snapshot` |
| Draft ao vivo | `presage://draft` |
| Fase do cliente (texto do vazio) | `presage://status` |
| Motor pronto, patch, base global e pessoal | `engine_status`, `presage://engine` |
| Ranking | `rank_draft` |
| Menos expostos, melhor resposta | `minimax_draft` |
| Sugestão de ban | `ban_suggestions` |
| Rota presumida de cada inimigo | `lane_analysis` com `candidate: null`, quando `source` é `"draft"` (`App.tsx:287-293`) |
| Prévia do candidato | `lane_analysis` com `candidate: id` |
| "incluir quem você não tem" | `get_draft_prefs` |
| Nomes e retratos | Data Dragon pelo front (`lib/ddragon.ts`) |

**Interações:**
- **Rota de um inimigo** (`DraftBoard.tsx:156-183`): só para inimigo travado e
  sem `position`. O seletor mostra a presumida como "`TOP (presumida)`" e lista as
  cinco rotas. Escolher chama `set_enemy_role` com
  `{ championId, role: "middle" }` (a rota passa por `ROLE_TO_LCU`); voltar à
  primeira opção manda `role: null`. A marca só aparece depois que o comando
  resolve (`App.tsx:272-284`). O `title` diz "Rota marcada por você" ou "Rota
  presumida pelo Presage; corrija se estiver errada".
- **Sua rota** (`RecommendationPanel.tsx:128-145`): as abas aparecem só quando o
  cliente não informa a sua posição (`localPosition` vazio, blind e normal).
  Clicar chama `set_draft_role` com `{ role: "middle" }`; a aba acende depois
  que o comando resolve (`App.tsx:264-269`).
- **incluir quem você não tem**: chama `set_draft_prefs` com
  `{ prefs: { showAllChampions } }`. A caixa muda antes da resposta. O `title`
  diz "Desligado, a lista só traz quem a sua conta pode escolher nesta fila,
  quando o cliente informa".
- **Clicar um retrato** abre o detalhe; clicar de novo fecha. O candidato sai da
  seleção sozinho quando some das duas listas (`RecommendationPanel.tsx:78-86`).
- O app nunca bane nem trava (R-3): nenhum controle desta aba escreve no
  cliente.

**Detalhe e prévia:** `score% ±confidence · n=sample`, os motivos de
`ScoredChampion.reasons` e "Melhor resposta deles tira x.x pontos da composição
(Nome)". A prévia
tem a curva 1–18 e a tabela de vantagem de nível contra o oponente da rota
("Rota contra {nome}: o 1v1 de atributos, nível a nível"), as brigas "2v2 e 3v3
com ele", a "Chance do time com ele, pela base de partidas" (só com a base) e a
condição de vitória.

**Estados:**
- Sem draft, conectado: "Sem draft ativo" e "Cliente conectado (Lobby)." — a fase
  sai crua do LCU, sem o rótulo da barra.
- Sem draft, sem cliente: "Sem draft ativo" e "Abra o League of Legends."
- Painel da direita sem draft: "Recomendações" e "O ranking aparece aqui durante
  a seleção de campeões."
- Sua vez: faixa "É a sua vez de banir." ou "É a sua vez de escolher." e relógio
  em violeta.
- Slot vazio: "Aguardando"; campeão sem dado estático: `#103`; hover de aliado:
  "· hover".
- Motor carregando: "Carregando dados de campeões…"; se a casca não conseguiu os
  campeões, "Não consegui os dados de campeão: {motivo}. O ranking volta quando o
  Presage abrir de novo com internet." (A32, [GC-7](07-lacunas-da-casca.md)).
- Sem base global: "Sem base estatística: o ranking sai da composição dos times
  e do seu histórico (N partidas, Riot ID). Winrate base, matchup e sinergia
  precisam do pipeline de partidas — nenhum dos três sai de uma conta só."
- Base de outro patch: "A base é do patch X e o jogo está no Y. Os números são
  de outro patch."; do mesmo patch: "Base do patch X, N partidas."
- Sem rota e sem escolha: "sem rota, a lista inclui todos". Mente quando a pool
  é de uma rota só: a casca usa a rota da pool ([GC-5](07-lacunas-da-casca.md)).
- Ban sem sugestão: "Nada travado do lado deles ainda e nenhum ban na sua lista
  (aba Pool). Sem pick e sem base estatística não há o que ranquear — e ordenar
  ruído seria pior que não dizer nada."
- Pool sem ninguém disponível: "Nenhum campeão da sua pool está disponível. Monte
  uma no Construtor de Pool." — a aba se chama "Pool".
- Geral vazio: "Sem candidatos."
- Prévia: "Calculando a prévia…"; sem resposta: "A prévia de rota aparece quando
  o outro time travar alguém."; sem oponente na rota: "O oponente da sua rota
  ainda não travou."; rotas inferidas: "As rotas deles foram inferidas pelo que
  cada campeão costuma jogar."

## 3. Simulador

**Mostra:** o mesmo painel de recomendação do Draft, alimentado por um draft
montado à mão: cinco vagas de cada lado, a fase (escolha ou ban) e os bans.

| Dado | Comando ou evento |
|---|---|
| O draft montado passa a ser o que o motor lê | `set_simulated_draft` com `{ draft }` |
| Ranking, menos expostos, bans | `rank_draft`, `minimax_draft`, `ban_suggestions` |
| Prévia | `lane_analysis` com `candidate: id` |
| "Copiar o champ select" | o draft do cliente que `App` já tem (`presage://draft`) |

**Interações:**
- **Campeão**: um campo com `datalist` dos nomes. No `blur`, nome que não existe
  é ignorado; nome válido grava `championId` e `locked: true`
  (`SimuladorDraft.tsx:69-73`).
- **Rota do aliado**: seletor com as cinco posições do LCU; **você**: o rádio
  move `isLocal`, e a sua rota é a da sua vaga.
- **Rota do inimigo**: "rota presumida" (vazio) ou uma das cinco, já na palavra
  do LCU.
- **Fase**: "ESCOLHA" ou "BAN" troca `currentAction`.
- **Bans**: digitar e Enter acrescenta, até dez; clicar num ban tira.
- **Copiar o champ select** (só com draft do cliente) e **Limpar**
  (`draftVazio()`).
- Cada mudança manda `set_simulated_draft`; o ranking só é pedido depois que a
  casca guardou (`App.tsx:150-165`). Sair da aba manda `{ draft: null }`.

**Estados:**
- Antes da primeira resposta: o painel da direita mostra "O ranking aparece aqui
  durante a seleção de campeões.", o texto do Draft.
- Subtítulo: "Monte os times; a recomendação ao lado é a mesma do champ select."
- Vaga sua vazia: placeholder "você (vazio = candidatos)".
- Não há abas de rota nem "incluir quem você não tem": o simulador ignora a
  lista da conta (R-5).

## 4. Rotas

```
┌──────────────────────────────────────────────────────────────────────────┐
│ Chance de você ganhar cada rota            projeção no nível 6, itens iguais│
│ ! O cliente não informou as rotas deles. Elas foram inferidas …           │
│ TOP  ▣ x ▣  ███████░░░░  58%  · motivo · sem base de partidas: só a conta │
│             nv 3 61%  nv 6 58%  nv 11 52%  nv 16 49%   [curva 1–18]        │
│ JG   ▣ x ▣  ████░░░░░░░  41%  …                                           │
│ Se a briga acontecer                                                     │
│ jg+mid  ▣▣ x ▣▣  ██████░░  Vocês 63%  Eles 37%                            │
│ Condição de vitória   Vocês: [regra] …   Eles: [atributos] …              │
│ Runa principal · Ahri x Zed   ◉ Eletrocutar  Também servem · Árvore secundária│
└──────────────────────────────────────────────────────────────────────────┘
```

**Mostra:** a chance de cada rota com barra, motivos, os quatro níveis-chave e a
curva; as escaramuças; a condição de vitória; a runa do confronto.

| Dado | Comando ou evento |
|---|---|
| Rotas, escaramuças, condição de vitória | `lane_analysis` com `candidate: null` |
| Runa principal, alternativas, árvore secundária | `rune_plan` |
| Patch do texto da runa | `engine_status` (`version`) |

A mesma chamada serve o draft e a partida: com partida, `source` é `"partida"`,
o subtítulo vira "níveis e itens reais da partida" e a curva ganha o ponto de
agora ("você nv 9 · ele nv 8").

**Interações:** nenhuma. A aba só lê.

**Estados:**
- Sem rotas: "Sem rotas para comparar" e "A análise aparece quando os dois lados
  tiverem campeões escolhidos."
- Rota sem base: "sem base de partidas: só a conta do duelo".
- Rotas inferidas: "O cliente não informou as rotas deles. Elas foram inferidas
  pelo que cada campeão costuma jogar — pode errar com escolha fora de rota."
- Condição de vitória vazia: "Nada a ler ainda: as composições estão incompletas
  e nenhum duelo pende para um lado."; um lado vazio: "Nada que se destaque."
- Runa sem base: "Isto sai de regras sobre a mecânica do confronto — quem alcança
  quem, quem aguenta troca longa, quem morre de explosão. Não sai de winrate:
  essa base ainda não existe. Uma regra acerta o caso comum e erra o campeão
  excêntrico."; com base: "Base de partidas do patch {versão}." — a versão é a do
  Data Dragon da casca, não a da base.
- Sem runa (`keystones` vazio ou `null`): a seção some.
- A análise não se refaz quando você escolhe a sua rota ou muda a pool; só com o
  draft, a partida, o motor ou as marcas ([03 §4](03-estado-e-atualizacao.md#4-o-que-refaz-cada-comando)).

## 5. Partida

```
┌───────────────────────────────────────┬──────────────────────────────────────┐
│ Partida                         14:32 │ Build adaptativa                     │
│ Ahri · nível 11                       │ reage aos itens que eles têm agora   │
│ ┌ (foco da prioridade) ─────────────┐ │ Dano que vem contra você             │
│ Sua rota (MID) contra Zed             │ █████████░░░  71% físico  29% mágico │
│  tropas/min 7.1 – 6.4  ouro +420·igual│ Sua vida efetiva 2140  Letalidade 18 │
│ Objetivos  Dragão 1:12  Barão estimado│ Próximo item                         │
│ Dragões 2–1  Torres 3–2  Abates 9–7   │ 1 ▣ Zhonya  3250 ouro  6.8 poder/1000g│
│ Prioridade agora  BOT ████ 72  …      │ Inimigos  Zed nv 11 ▣▣▣  morto 12s   │
│ Ouro em itens +1.240  ████████░░      │                                      │
│ Seu time / Inimigos (os dez)          │                                      │
└───────────────────────────────────────┴──────────────────────────────────────┘
```

**Mostra:** relógio, campeão e nível; o foco da prioridade; você contra o
oponente direto; timers de objetivo, de estrutura caída e de buff; placar de
dragões, torres, abates e alma; prioridade por rota; ouro em itens dos dez; à
direita, a divisão do dano, a vida efetiva, a penetração e a letalidade deles,
os próximos itens e os itens de cada inimigo.

| Dado | Comando ou evento |
|---|---|
| Partida, timeline, prioridade, buffs, sua rota | `get_live`, `presage://live` |
| Divisão do dano, vida efetiva, próximos itens | `build_suggestions` com `{ limit: 8 }` |
| Ouro dos dez, notas | `gold_report` |
| Ícones de item e retratos | Data Dragon pelo front |

**Interações:** nenhuma.

**Estados:**
- Sem partida, conectado: "Sem partida em andamento" e "Entre numa partida e os
  timers e a build adaptativa ligam sozinhos."; sem cliente: "Abra o cliente do
  League of Legends."
- Sem jogador local: "jogador não identificado".
- Sem objetivo: "Nenhum objetivo na fila."; primeiro spawn: "estimado"; nada
  disponível: "disponível"; estrutura: "seu, caído" ou "deles, caído"; buff de
  abate anterior ao app: "quem tem: abate antes de o app olhar".
- Sem prioridade: "Sem dados de rota."; sempre: "Ranking de valor, não de
  posição. A API da Riot não entrega onde os campeões estão, e o Presage não
  tenta adivinhar."
- Sua rota: `Ouro em itens é exato. "À frente" a partir de 300 ou de 10% do que
  ele investiu, o maior — regra, não estatística.` O 300 e o 10% são cópia em
  texto de `rival::AHEAD_FLOOR` e `AHEAD_SHARE` ([GC-9](07-lacunas-da-casca.md)).
- Build sem contexto (`build_suggestions` devolve `None` só sem partida ou sem o
  jogador local): "A build precisa do seu campeão, e a partida ainda não disse
  qual é". Itens que falharam: "Não consegui os dados de item: {motivo}" (A32).
  Sem item: "Sem dados de item carregados."; inimigo sem item: "sem
  itens"; morto: "morto 12s".
- Ouro: "A coluna em negrito é exata: sai do preço de cada item no inventário. O
  valor claro ao lado do seu nome é estimativa do seu ouro em mão. O dos outros
  nove não aparece: não é algo que você saberia olhando o jogo." O valor claro só
  aparece na linha do jogador local, com `unspent ≥ 800` (C-4, R-12;
  [GC-2](07-lacunas-da-casca.md), fechada).

## 6. Pós-jogo

**Mostra:** o campeão e o resultado da última partida, o resumo, as observações
"A corrigir" e "A manter", as curvas de ouro em itens, de tropas (ou de visão,
para o suporte) e de nível, com as suas mortes marcadas; embaixo, o perfil de
rota.

| Dado | Comando ou evento |
|---|---|
| A leitura da última partida | `last_review`, `presage://review` |
| Perfil de rota | `lane_profile`, de novo a cada `presage://engine` |

**Interações:** nenhuma. A aba abre sozinha quando chega `presage://review`.

**Estados:**
- Sem leitura: "Nenhuma partida lida ainda" e "O Presage grava a partida enquanto
  ela acontece e faz a leitura quando ela acaba."
- Sem campeão: "Última partida"; resultado: "vitória" ou "derrota", ou nada
  quando `won` é `null`.
- Sem observação: "Nada a apontar nesta partida."
- Rodapé: "Tudo aqui sai da gravação da própria partida, feita na sua máquina.
  Nada é enviado para lugar nenhum."
- Leitura do gráfico: com pico de pelo menos 1.000 e queda de mais de 1.500
  até o fim, "Pico de X aos m:ss, fim em Y."; senão "Fechou em Y depois de m:ss."
  O corte é regra da tela (`GraficoPartida.tsx:133`).
- Perfil vazio: "Sem histórico de rota ainda" e "Com a chave da Riot definida, o
  Presage lê suas ranqueadas e compara cada partida com o seu oponente direto."
- Perfil: "Você, no geral" e uma linha por campeão; confrontos só com duas
  partidas ou mais, até quatro; célula sem leitura: "—"; o `title` diz "média
  crua X em N partidas". Só ganha cor o que sai de ±1,96·sd, e esse `±` é
  calculado na tela ([03 §9](03-estado-e-atualizacao.md#9-o-que-a-tela-ainda-calcula)).

## 7. Pool

```
┌────────────────────────────────────────────────────────────────────────┐
│ MID · passo 3 de 3                                                     │
│ Completar a pool: lugar 5 de 8                                         │
│ [▣][▣][▣] | [▣][ 5 ][ 6 ][ 7 ][ 8 ]                         4 de 8      │
│ Os seus três: pontos fortes        Pontos fracos                       │
│ ┃ dano mágico garantido             ┃ sem linha de frente               │
│ Para o lugar 5                                                         │
│ ▣ Malphite  traz frontline   ▣ Orianna …   ▣ …                         │
│ Esta lista sai da composição da sua pool … regra, não estatística.     │
│ [Voltar]                                              [Fechar com 4]   │
└────────────────────────────────────────────────────────────────────────┘
```

**Mostra:** um assistente em três passos por rota (a rota; os três core; os
cinco sugeridos um por vez) e uma revisão sem limite, com os três grupos e a
lista de bans.

| Dado | Comando ou evento |
|---|---|
| A pool e os bans | `get_pool` |
| Gravar | `set_pool` com `{ pool }` |
| Retrato dos três e candidatos do próximo lugar | `pool_suggestions` com `{ role: "middle", limit: 6 }` |
| Catálogo para buscar | Data Dragon pelo front |

**Interações:**
- **Passo 1, a rota:** cinco cartões, cada um com "N escolhidos" ou "vazia".
  Rota com entradas abre a revisão; sem entradas, o passo 2.
- **Passo 2, os três:** busca e grade; cada clique entra como `core`. Com três,
  a grade desliga ("Já escolheu 3") e "Analisar os três" leva ao passo 3; antes,
  o botão diz "Faltam N".
- **Passo 3, os cinco:** a cada campeão que entra ou sai, `pool_suggestions` é
  chamado de novo (`PoolBuilder.tsx:38-48`); clicar num candidato o acrescenta
  como `situational`. "Fechar com N" ou "Ver a pool" vão para a revisão.
- **Revisão:** abas por rota; em cada campeão, `C`, `S` e `E` trocam o grupo e
  `×` tira; "Completar até 8" (com core e menos de oito) e "Outra rota".
  "Adicionar" acrescenta como `extra`, sem limite.
- **Seus bans:** até cinco (`BAN_POOL_MAX`), em ordem; "↑" sobe um, `×` tira; a
  busca mostra até oito nomes.
- Toda mudança chama `set_pool` com a pool inteira. A tela muda antes da
  resposta, e uma falha não volta nada ([GC-1](07-lacunas-da-casca.md)).

**Estados:**
- Passo 1: "Em que rota você joga?" e "A pool é por rota. Dá para montar as
  outras depois."
- Passo 2: "Escolha seus 3 campeões" e "Os que você mais joga. A pool vai ser
  montada em volta deles."
- Passo 3: "Completar a pool: lugar N de 8" ou "Pool completa"; esperando:
  "Calculando…"; sem candidato: "Nada a recomendar para este lugar." — também
  quando o comando falha, porque o erro vira uma lista vazia
  (`PoolBuilder.tsx:44`); com oito: "Oito na pool. Dá para trocar qualquer um na
  revisão."; retrato: "Nada que a rota valorize." e "Nenhum buraco no que a rota
  cobra."
- Rodapé do passo 3: "Esta lista sai da composição da sua pool — tipo de dano,
  linha de frente, alcance, estilo — e da matemática de resistências do jogo. O
  que conta em cada rota são pesos escritos à mão: regra, não estatística. Contra
  quem os seus três perdem só a base de partidas vai dizer, e ela ainda não
  existe."
- Revisão: "Sua pool" e "Aqui não há limite. Ajuste o que quiser."; grupo vazio:
  "Vazio."; bans vazios: "Nenhum. Os bans que você sempre faz entram aqui."
- O limite de cinco bans só existe aqui: `pool::BAN_POOL_MAX` não é usado no
  Rust ([GC-9](07-lacunas-da-casca.md)).

## 8. Ajustes

**Mostra:** as seis seções do overlay, cada uma com rótulo e detalhe, e a seção
"App" com o início junto com o Windows.

| Dado | Comando ou evento |
|---|---|
| Seções escondidas | `get_overlay_prefs` |
| Gravar seções | `set_overlay_prefs` com `{ prefs: { hidden } }` |
| Iniciar com o Windows | `get_autostart` |
| Gravar | `set_autostart` com `{ enabled }` |

**Interações:**
- Cada caixa liga ou desliga uma seção. A tela muda na hora e grava depois
  (`App.tsx:317-322`); a casca emite `presage://prefs` e o overlay muda no meio da
  partida. "Mostrar tudo" manda `hidden: []`.
- "Iniciar o Presage com o Windows": a caixa só muda depois que `set_autostart`
  resolve (`App.tsx:300-304`).

**Estados:**
- Cabeçalho: "O que aparece no overlay" e "Vale na hora, mesmo com a partida
  rodando. O painel continua mostrando tudo."
- Tudo desligado: "Tudo desligado: o overlay não vai aparecer. O painel continua
  funcionando normalmente."
- Rodapé: "O overlay abre e fecha com Ctrl+Shift+P. Ele nasce atravessável pelo
  mouse, então não come clique; Ctrl+Shift+O deixa a janela clicável quando você
  quiser arrastá-la." Arrastar não funciona: a janela não tem região de arrasto
  ([GC-8](07-lacunas-da-casca.md)).
- Autostart: "Desligado até você ligar. O ícone na bandeja traz a janela de
  volta."

## Cabeçalho e avisos

| Elemento | Origem | O que mostra |
|---|---|---|
| Logo | `Logo.tsx` | A marca em `currentColor`, 22 px |
| Nome | `App.tsx:329` | "PRESAGE" em `--halo` |
| Versão | `engine_status` e `__APP_VERSION__` | "patch 16.21.1 · v0.7.0" com o motor pronto; "draft engine · v0.7.0" antes |
| Conexão | `get_snapshot`, `presage://status` | Ponto verde ou apagado, a fase traduzida (`PHASE_LABEL`, `ConnectionStatus.tsx:3-15`: "Seleção de campeões", "Cliente fechado"…; fase desconhecida aparece crua) e o seu Riot ID. O erro de conexão vai no `title` (`ConnStatus.detail`) |
| Versão nova | `check()` do plugin updater, uma vez na abertura | "Versão X disponível", as notas da versão, "Atualizar e reiniciar" e "Depois" (`title` "Volta a avisar na próxima vez que abrir"); baixando: "Baixando N%"; no fim, "Reiniciando…" |
| Data Dragon | `loadStaticData()` | "Não consegui carregar os dados do Data Dragon ({erro}). Nomes e ícones ficam indisponíveis." |
| Falha de comando | `chamar` em `App.tsx` ([03 §7](03-estado-e-atualizacao.md#7-erros-hoje)) | Uma faixa por coisa que falhou: "Não consegui {o que o app tentava}: {o que a casca disse}", com "×" para fechar. Ex.: "Não consegui gravar a pool: …", "Não consegui ler o ranking: …". Some sozinha quando a mesma chamada volta a dar certo |

- **Versão nova nunca instala sozinha** (RNF-7). Falha na checagem, no download
  ou na instalação esconde o aviso, que volta na próxima abertura
  (`Atualizacao.tsx:56-60`). Essa é a única falha calada por decisão.
- **"Depois"** vale para esta execução: o estado não é gravado.
- Todo comando que falha aparece numa faixa com o texto da casca (A1). A única
  falha calada por decisão continua sendo a da versão nova (RNF-7).
