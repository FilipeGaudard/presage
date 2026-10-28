# 04 — As 39 rotas da janela principal

Roteador TanStack em `desktop-BYOe2afW.js`; cada rota carrega o seu chunk sob
demanda. A coluna "Chunk" é o arquivo em `dist/assets/`. Rotas de layout
(`/home`, `/tools`, `/settings`, `/profile`) só desenham o menu da seção.

## 1. Mapa

| Rota | Chunk | Tela |
|---|---|---|
| `/home` | `DLwEA00z.js` | layout: User Guide, Latest Update |
| `/home/user-guide` | `DwiflMJy.js` | índice do guia |
| `/home/user-guide/getting-started/` | `CwLuwxt4.js` | guia: primeiros passos |
| `/home/user-guide/riot-approved/` | `CKyr18Pp.js` | guia: aprovação da Riot e Vanguard |
| `/home/user-guide/the-drafting-tool/` | `BoqA-43n.js` | guia: ferramenta de draft |
| `/home/user-guide/the-statistics/` | `CXtIfQYi.js` | guia: estatísticas |
| `/home/user-guide/the-ai-macro-coach/` | `CY6ApCfg.js` | guia: AI Macro Coach |
| `/home/latest-update/` | `x6jYFdwd.js` | notas da versão 3.11.0 |
| `/tier-list/` | `BDWwrFPa.js` | tier list geral e pessoal |
| `/ai-coach/` | `h1_fjSTU.js` | AI Macro Coach |
| `/premium/` | `CbpbJgP6.js` | planos e teste grátis |
| `/tools` | `D8raCtkB.js` | layout das ferramentas |
| `/tools/drafting-tool/` | `v6Gj80KT.js` | simulador de draft |
| `/tools/champion-pool-builder/` | `a3MS-J-6.js` | construtor de pool conversacional |
| `/tools/quiz/` | `dQuImjrX.js` | apresentação do quiz |
| `/tools/quiz/personality-quiz/` | `DolMdrYW.js` | quiz de personalidade |
| `/profile` | `gT4A5dz7.js` | layout: Match History, Account Statistics, Champion Statistics |
| `/profile/match-history/` | `DcliAquW.js` | histórico de partidas |
| `/profile/match-history/$matchId/` | `BUEq32Eu.js` | detalhe da partida (parâmetros `puuid`, `region`, `rankClass`) |
| `/profile/account-statistics/` | `DF5z8Cgx.js` | estatísticas da conta |
| `/profile/champion-statistics/` | `D7TV8dtA.js` | estatísticas por campeão |
| `/settings` | `e8rNRDGT.js` | layout das configurações |
| `/settings/general/` | `CJrYdSKV.js` | geral e "Danger Zone" |
| `/settings/screens/` | `DKCaA8kW.js` | janela principal, draft, in-game |
| `/settings/drafting/` | `CNLMi995.js` | draft: modelo, filtros, builds |
| `/settings/champion-pool/` | `DUYjc-bu.js` | pool e auto-lock |
| `/settings/champion-ban-pool/` | `BhyI5ZWd.js` | ban pool e auto-hover de ban |
| `/settings/overlays/` | `CY3CFoqT.js` | overlays |
| `/settings/riot-accounts/` | `DxKdBoQ3.js` | contas Riot e maestria agregada |
| `/settings/itero-account/` | `Cv7gw-uJ.js` | conta iTero e assinatura |
| `/settings/language/` | `mqInUVnT.js` | idioma |
| `/settings/privacy/` | `BgljJzuo.js` | anúncios e privacidade |
| `/login/` | `BACmdFok.js` | entrar (Firebase Auth) |
| `/register/` | `Bsin7zHw.js` | criar conta |
| `/verify-email/` | `BEgAYKgd.js` | verificar e-mail |
| `/forgot-password/` | `IBjHXOkN.js` | recuperar senha |
| `/change-password/` | `DLcgqHzW.js` | trocar senha |
| `/logout/` | `zRh33zZ_.js` | sair e voltar ao login |
| `/debug/` | `DZUxX0Gb.js` | tela de depuração, só em modo de desenvolvimento |

## 2. Guia do usuário

O próprio iTero descreve as funções; é a melhor fonte sobre o **modelo de
draft**:

- treinado em "milhões" de partidas de solo queue; recomenda o campeão que prevê
  render melhor naquele draft; "jogadores que seguem as sugestões ganham mais";
- termos, em ordem aproximada de importância: **maestria** (desligável),
  **winrate do campeão**, **contra de rota**, **ouro previsto aos 14 min**,
  **capacidade de jogar à frente, igual ou atrás**, **contra do time inteiro**,
  **sinergia do time inteiro**, **estatísticas do time** (divisão de dano
  mágico/físico);
- "Rating" é nota **relativa** de 0,0 a 5,0 contra as alternativas (5,0 de
  winrate = entre os mais fortes do patch);
- dados: patch atual, partidas do **seu elo** nas regiões maiores;
- pool: "a melhor forma de subir é uma pool de 1 a 6 campeões" — a pool tem
  prioridade nas recomendações;
- "Consider Team Hover": conta a sinergia com o hover de quem ainda não travou,
  porque "a maioria acaba jogando o que deixou em hover";
- filtro de meta: do "Non-Meta" (qualquer campeão em qualquer rota) ao "Meta"
  (só os mais comuns da rota).

Riot: afirma ter sido **permitido pela Riot**, que todo dado vem da API oficial
e que interage com o cliente só pelo LCU — "incluindo auto-hover de campeão e
envio de runas".

Estatísticas: ~500 por conta, quebradas em **faixas de 7 minutos**, **ajustadas
pela pool** (compara você com outros jogadores do mesmo campeão), comparadas ao
**seu elo** em percentis, e acompanhadas por **média móvel de 50 partidas**.

## 3. Perfil

### Histórico (`/profile/match-history/`)

Últimas partidas com filtro por fila, rota e campeão, "Last 10 Games" (winrate),
**Refresh**. Paginação limitada no servidor (`match_history_window`).

### Detalhe da partida (`/profile/match-history/$matchId/`)

Abas **Summary**, **Macro Coach**, **Draft**, **Statistics**:

- cabeçalho: Victory/Defeat, lado, **AI Draft Score** (com e sem maestria),
  **AI Macro Score** (assinantes; senão "Upgrade to iTero Plus" ou "This match
  is still being analysed"), elo médio do lobby;
- **Draft**: Synergy Score e Counter Score do seu pick;
- **Macro Coach**: Key Improvement Areas e Strengths **da partida**, por faixa
  de tempo, com "Coaching Review Objectives" do último review da conta;
- **Statistics**: valor na partida, valor ajustado pelo campeão, média da conta,
  média do elo, percentil; gráfico **Statistics Breakdown** minuto a minuto (as
  seis estatísticas configuráveis por rota — padrão do suporte com visão e
  sentinelas, [01 §8](01-processo-principal.md#8-configurações-do-usuário-settings));
  linha do tempo de objetivos (Void Grubs, Herald, Dragon, Baron) dos dois
  times; comparação com o oponente.

### Estatísticas da conta (`/profile/account-statistics/`)

Visões **Simplified** (gráficos) e **Detailed** (tabela com busca), por fila e
rota, com aviso de amostra baixa (mínimo 5 partidas):

- grupos: Resource (Lane CS, XP, Jungle CS, Gold), Combat (participação total,
  KDA, dano feito e recebido, first bloods, invasões nível 1, participação antes
  dos 7), Objectives (dragões, grubs e arauto, torres internas e externas),
  Vision (sentinelas comuns, de controle, trinket azul, do item de suporte),
  Share (dano, ouro e XP do time);
- gráficos: Statistical Summary; What are you Destroying? (torres × jogadores ×
  monstros); Your Favourite Monsters; Who are you killing? (abates por rota
  inimiga); Team Percentages; Early Game Aggression; Wallet Size Over Time (ouro
  por faixa); dano feito × recebido; Kills vs Deaths solo × assistidos;
- **Compliment Me / Flame Me**: texto gerado por IA sobre as suas estatísticas
  (`lake-flame-me`), com limite por sessão e aviso de que é IA;
- cada estatística abre o painel individual: média da conta, do elo, percentil,
  partidas recentes, faixa de tempo e o **Leaderboard** (a mesma estatística dos
  amigos adicionados).

**Títulos** por estatística (`Q1` no recurso de tradução): cada uma tem rótulo
para "alto" e "baixo" — "MAXIMUM DAMAGE"/"The Tickler", "The Tank"/"The Ghost",
"The Inter"/"Invincible!", "The Farmer"/"A Minion's Friend", "The
Oracle"/"Blind", "Bullied by Top", "The Duelist" etc. (37 estatísticas).

### Estatísticas por campeão (`/profile/champion-statistics/`)

Por campeão: partidas, **meu winrate** contra o **winrate da comunidade**,
maestria, **Draft Score** médio (com "Consider Mastery"), "Top 3 Most Played",
marca "4 or Less Games".

## 4. AI Macro Coach (`/ai-coach/`)

- Treinado com partidas do seu elo; avalia como as suas estatísticas mexeram na
  chance prevista de vitória de cada partida recente; as que mais **derrubam** a
  chance média viram **Key Improvement Areas**, as que mais sobem viram
  **Strengths** (`POST {região}/lake-coach-{rota}`, `request_type`, puuid,
  fila).
- Por faixa de 7 minutos; mínimo 10 partidas (no geral e por faixa); indica a
  faixa recomendada para a sessão; mostra "Last Coach Review: N games ago" e
  quando o próximo fica disponível.
- Tabela por estatística: média da conta, média do elo, percentil; "Related
  Statistics"; partidas recentes; winrate das últimas 10; estatísticas
  parecidas (`similar_stats`).

## 5. Tier list (`/tier-list/`)

**General**: por rota e faixa de elo (até "MASTER+"), winrate, ban rate, pick
rate (CloudFront, `champion_tier_list.json`). **Personal**: pesa a força do
campeão no patch pela **sua experiência** com ele (`personal-tier-list`, pelo
seu elo e rota).

## 6. Ferramentas

### Simulador de draft (`/tools/drafting-tool/`)

Monte os dois times à mão (busca, Blue/Red), **Randomize Draft** (pela
distribuição de campeões por rota), **Import Draft from Match** (puxa a
composição de uma partida do seu histórico, com o resultado), **Clear Draft**,
**Get Recommendations** — a mesma análise do champ select (AI Draft Score,
Matchups, Laning Phase, Team Stats, recomendações) com customização própria do
modelo.

### Construtor de pool (`/tools/champion-pool-builder/`)

Conversa guiada por um personagem ("I am the iTero AI Champion Pool Builder"),
com perguntas de sim/não:

1. explicar o modelo? (fatores: maestria, winrate, contra do time, sinergia,
   ouro aos 14, estatísticas do time);
2. usar a maestria? ("o maior fator é a experiência no campeão");
3. elo da estatística; rota;
4. **Blind pick 1**: forte no patch, pouco contrado pelos picks comuns, de
   preferência com experiência (`cpc_blinds`);
5. **Blind pick 2**: idem, **sem** opções parecidas com o primeiro;
6. **Situational picks 1–3**: lista os "Uncountered Champions" — campeões
   comuns da rota contra os quais a pool não tem resposta — e sugere quem os
   responde; repete com a pool atualizada (`cpc_recs`);
7. no bot: **pairing pick** — parceiros comuns sem bom par na pool;
8. **pocket pick**: se sobrou uma ameaça muito popular sem resposta;
9. **damage check**: garante pelo menos um AP e um AD (`cpc_damage_check`);
10. resultado com "Good vs" de cada um (`cpc_summary`) e **Add selection to
    champion pool**.

### Quiz de personalidade (`/tools/quiz/…`)

18 afirmações (concordância; "Neutro" não conta) → campeões de cada rota que
combinam (`api.itero.gg/champ_quiz`).

## 7. Premium (`/premium/`)

iTero Plus (padrão USD 7,99, preço regional do backend; pagamento Tebex):
remove anúncios, **AI Macro Coach em toda partida**, análise na frente da fila,
**análise contínua do draft** (recomendação refeita a cada fase, não só na sua
vez), cargo no Discord, criar dados de **qualquer** conta (pelo leaderboard),
customização do modelo, maestria agregada, suporte a personalizada. **Teste de 7
dias** sem cartão; ao terminar, cupom de 50%.

## 8. Configurações

| Página | Itens |
|---|---|
| General | abrir com o Windows, aceleração por hardware, verificar atualização, abrir logs, reiniciar, **Danger Zone**: limpar cache e reiniciar, desinstalar, restaurar padrões |
| Screens | janela principal (abrir com o League, esconder durante a partida), draft (abrir no champ select, tamanho), in-game (abrir na partida, aba padrão, atalhos Alt+I/Alt+O) |
| Drafting | usar o meu elo ou um fixo; considerar maestria (peso); considerar hover do time; pick rate mínimo (5 níveis); mostrar todos os campeões; mostrar/ordenar/nota mínima de fase de rota; modo avançado; **Model Customization** (peso por termo, por rota); envio de build (itens, runas, feitiços; manual e automático); tecla do Flash (D/F) e ordem dos feitiços |
| Champion Pool | pool por rota; auto-hover; auto-lock (aviso: "usa a API do cliente, separada do Overwolf… use por sua conta e risco") |
| Champion Ban Pool | até 5 bans em ordem; auto-hover de ban por rota ou geral |
| Overlays | habilitar todos; timers de mapa (SR e ARAM), habilidade, aprimoramentos, ouro do time, dano, buffs, suprema dos aliados, CS por minuto (elo, rotas, mostrar só com atalho); atalho de overlay (segurar/alternar); scouting avançado |
| Riot Accounts | conta atual e alternativas (região, nome, tag); **Aggregate Mastery** (assinantes) |
| iTero Account | e-mail, verificação, senha, assinatura (status, início, fim/renovação, gerenciar), e-mail com hash para anúncio |
| Language | idiomas: inglês, português, alemão, espanhol, francês, japonês, coreano, polonês, turco, vietnamita, chinês, árabe |
| Privacy | CMP de anúncios, política de privacidade, termos |
