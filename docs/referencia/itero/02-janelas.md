# 02 — As 16 janelas do iTero

Cada janela é uma página própria em `dist/src/app/<nome>/index.html`, com o seu
chunk de entrada em `dist/assets/<nome>-<hash>.js`. Há dois tipos
(`main.js:14657-14737`):

- **Janela comum** do Electron: sem moldura, fundo preto, não redimensionável
  (exceto a principal), guarda posição e tamanho, recentraliza se abrir fora da
  tela.
- **Overlay injetado** pelo pacote `overlay` do Overwolf: desenhado **dentro**
  do processo do jogo, preso à área do jogo (`ensureInGameBound`). Alguns são
  *passthrough* (o clique atravessa para o jogo) e sem ajuste de DPI, porque se
  posicionam por cálculo sobre o HUD.

| # | Janela | Tipo | Tamanho | Abre quando | Chunk de entrada |
|---|---|---|---|---|---|
| 1 | `desktop` | comum, redimensionável | 1320×760 (mín. igual) | launcher detectado (`autoLaunchDesktop`), bandeja, segunda instância, fim da partida | `desktop-BYOe2afW.js` |
| 2 | `drafting` | comum | 460×832 (tamanho `lg`/`sm` alternável) | champ select de SR (`autoLaunchDraft`) | `drafting-sEhWYRpU.js` |
| 3 | `drafting-aram` | comum | 1280×720 | champ select de ARAM/ARAM Mayhem | `drafting-aram-DzmzYadY.js` |
| 4 | `ingame` | overlay | 1200×720 | partida de SR, um monitor (`autoLaunchInGame`) ou atalho | `ingame-DixYuRIr.js` |
| 5 | `ingame-second` | comum | 1200×720 | partida de SR havendo segundo monitor | `ingame-second-CyjCQBW0.js` |
| 6 | `ingame-aram` | overlay | 950×652 | partida de ARAM, um monitor | `ingame-aram-DPA4tIGI.js` |
| 7 | `ingame-aram-second` | comum | 950×652 | partida de ARAM, segundo monitor | `ingame-aram-second-DD2-329J.js` |
| 8 | `monster-buff-timers` | overlay | 240×60 | primeiro evento da partida, se habilitado (SR) | `monster-buff-timers-C39-tqn4.js` |
| 9 | `team-gold-tracker` | overlay | 300×330 | idem, se habilitado | `team-gold-tracker-BWcnDvnQ.js` |
| 10 | `ult-cooldown-timers` | overlay, passthrough | sobre os quadros do time | idem | `ult-cooldown-timers-DwTUIVLh.js` |
| 11 | `map-timers` | overlay, passthrough | sobre o minimapa | idem (SR ou ARAM) | `map-timers-C_5-vGjv.js` |
| 12 | `damage-tracker` | overlay | 200×200 (120 de altura com um lado só) | idem | `damage-tracker-B2L_oOQp.js` |
| 13 | `background` | comum, invisível (1×1) | — | sempre, no boot | `background-io78d1_K.js` |
| 14 | `skill-level-up-tracker` | overlay, passthrough | sobre a barra de habilidades | idem | `skill-level-up-tracker-Dd7icVnR.js` |
| 15 | `stat-tracker` | overlay | 135×165 | idem (SR, rotas escolhidas) | `stat-tracker-CpjXv1Kv.js` |
| 16 | `aram-mayhem-augments` | overlay, passthrough | tela inteira do jogo | aprimoramentos oferecidos no ARAM Mayhem | `aram-mayhem-augments-D1lsd3GA.js` |

Os overlays 8–16 (exceto 13) obedecem ao atalho de overlay (Tab, segurar ou
alternar): com ele ativo aparecem ou ficam interativos, conforme a janela.

## 1. `desktop` — a janela principal

Roteador TanStack com 39 rotas, descritas em [04](04-telas-da-janela-principal.md).
Barra lateral com Perfil, Ferramentas, Tier list, AI Coach, Premium, Guia,
Configurações; aviso de nova versão (consulta `app-status` a cada hora); para
quem não assina, coluna de anúncios de 440 px (banner 400×60, "high-impact"
400×600 via `owadview` do Overwolf, e 400×300 embaixo).

## 2. `drafting` — o companheiro do champ select

Abas **Recommendations**, **Laning Phase** e **Overview**, mais a de ban.

- **Recomendações**: lista de campeões com **Overall Score** (força prevista do
  seu campeão no draft, em nota relativa 0,0–5,0) e, opcionalmente, **Laning
  Phase Score** (como deve ir a rota). Ordena por chance de vencer ou pela nota
  de rota. Cada linha abre o porquê: "Works well with" / "Doesn't work well
  with" (aliados), "Is strong vs" / "Is bad vs" (inimigos). Pool marcada.
  Avisos de estado: "Waiting for your turn to update", "Waiting for final pick
  to update", "Waiting for lane assignment for…", "Waiting for everyone to lock
  in", e o de hover manual detectado.
- **Matchups**: para o **seu** campeão, a nota contra cada inimigo e com cada
  aliado (quanto maior, melhor para você). Sem o oponente travado: "This
  match-up is a blind pick and therefore is based on averages", com textos de
  reserva por rota ("most enemy Top Laners"…).
- **Laning Phase** (modo avançado): por faixa de tempo do confronto, um **estilo
  de jogo** — Neutral, Safe, Sustain, Poke, Bully; para a selva, Farm and
  Crossmap ou Gank and Invade — com duas razões: **Trading** (dano que você faz
  sobre o que recebe no confronto) e **Kill Threat** (abates seus sobre os
  dele). Sem dado: "Not enough data for this matchup."
- **Team Stats** (modo avançado): razão AD/AP, dano total esperado, dano
  recebido esperado, vantagem de força no início, meio e fim, sinergia e contra
  agregados, força de rota (ouro gerado no confronto médio) e impacto dos
  confrontos na rota.
- **Builds**: depois que todos travam, runas, feitiços, fragmentos, itens e
  ordem de habilidade para aquela composição; **Push Build** grava no cliente
  (página de runas e conjunto de itens "iTero"), e o auto-push faz isso sozinho.
- **Ban**: "Ban Recommendations for <campeão em hover>" (contra o seu hover),
  "Ban Recommendations Overall", e a sua **Champion Ban Pool**, com Pick % e Win
  Rate; alerta "You are banning a champion a teammate wants to pick".
- **Draft Override**: atribuir a posição de cada inimigo à mão.
- **Configuração rápida**: General, Model Customization (pesos), Display,
  Recommendation Filters, Build Suggestions, Matchups, Team Stats.
- Tutorial de primeiro uso (`ftue*`): seção de visão geral, as duas notas, a
  ordem por chance de vitória, "maior número = melhor para você".

## 3. `drafting-aram`

Grade "Available Champions" (com "Held by a teammate" e **Random Champ**), e,
para o campeão escolhido, **Builds** (Core Items, Runes, Summoners, Skill
Order, Win Rate, Patch, **Push Build**, **Random Build** dentro da faixa de
raridade configurada) e **Augments** (recomendações de aprimoramento para ARAM
Mayhem). Busca via `api:get-aram-champion-builds` e
`api:get-aram-champion-augments`.

## 4–5. `ingame` e `ingame-second`

Mesmo componente; a `-second` é a versão em janela solta para o segundo monitor.
Duas telas alternáveis (Alt+O) e abas:

- **Player Scouting** (padrão): os dez cartões de jogador
  ([05](05-funcionalidades.md#2-scouting-de-jogadores)); modo avançado com
  comparações de fase de rota e meio de jogo. Respeita o modo streamer do
  cliente ("Streamer mode enabled" esconde nomes).
- **Game Data / Builds**: Starter Items, Build Order, Alternatives — e troca
  pela dica e build do **estado da rota** (atrás/igual/à frente,
  `ingame.laneStateTooltip`).
- **Pairings**: como o seu campeão, no seu elo, se sai com e contra cada um dos
  outros nove ("baixo é sempre ruim para você"); **Change Draft** ("Wrong
  draft? Edit here") corrige a composição.
- **Team Scaling Advantage**: força do seu time contra o deles em três fases —
  Early Game **15–26 min**, Mid Game **27–32 min**, Late Game **33+ min** (são
  faixas de duração da partida).
- **Lane State (<faixa>)**: como você está contra o time inimigo, considerando
  os campeões.
- Atalhos para ligar cada overlay.

Estatísticas exibidas no scouting: Overall/Champion, Percentile, Laning Phase,
Lvl 1 Invades, Aggression, Survivability, Roaming, Farming, Mid Game, Fighting,
Towers, Epic Monsters, Vision (`DGXGLUPr.js`).

## 6–7. `ingame-aram` e `ingame-aram-second`

Starter Items, Build Order, Runes e Patch do campeão; atalhos para os overlays
que valem no ARAM (timers de mapa, habilidade, aprimoramentos, ouro, dano,
suprema) (`DG7zzsI2.js`).

## 8. `monster-buff-timers`

Dois blocos, Barão e Ancião: ícone, tempo restante do buff e os retratos de
quem ainda o tem. Visível enquanto alguém tem o buff, ou com o atalho.

## 9. `team-gold-tracker`

Para cada par de jogadores na **ordem da Live Client** (1º do time 1 contra 1º
do time 2, e assim por diante), o **valor em itens** de cada um (soma de
`priceTotal × quantidade` pelo dado de itens; 500 quando vazio) e a diferença
com seta; no topo, o total dos dois times e a diferença, com corte de 1.000
para a seta dupla (500 nas linhas).

## 10. `ult-cooldown-timers`

Quatro números, um por quadro de aliado, com a recarga restante da suprema
(`team_frames` do GEP), posicionados conforme minimapa invertido e quadros à
esquerda. **Só aliados.**

## 11. `map-timers`

Sobre o minimapa, em posição percentual de cada estrutura:

- SR: respawn dos **seis inibidores** e das **quatro torres do nexo**;
- ARAM: as quatro **relíquias de vida** e os dois inibidores.

Tamanho do texto ajustável, multiplicado pela escala do monitor.

## 12. `damage-tracker`

Barras de proporção físico/mágico/verdadeiro e os totais (PHYS, MAGIC, TRUE)
para **dano recebido** e/ou **dano causado a campeões**, acumulados na partida
(eventos `damage` do GEP).

## 13. `background`

Invisível. Faz o que precisa do SDK web do Firebase:

- autenticação: ao logar, escuta `users/{uid}/subscription/current` no
  Firestore e calcula as flags de assinatura (ativa por status `Active`,
  validade futura ou teste vigente), que manda ao processo principal pela store
  `q7z2k`;
- configurações: puxa `users/{uid}` ao logar e grava com `merge` quando o
  principal pede (`save-settings-to-db`, `before-quit-cleanup`);
- relatório de erro: função na nuvem `reporterror` (`send-error-report`);
- avisa o principal de online/offline.

## 14. `skill-level-up-tracker`

Sobre a barra Q/W/E/R, marca a habilidade a subir quando há ponto sobrando
(pontos gastos < nível), seguindo a ordem da build ativa do iTero (no ARAM, a
build do ARAM). A posição é calculada da resolução e da escala global do HUD.
Uma runa específica (id 8313) troca os níveis 11 e 12 da ordem.

## 15. `stat-tracker` — CS por minuto

`CS/m` atual (tropas + monstros sobre minutos) contra a **média do elo para o
seu confronto**, minuto a minuto: a curva vem do backend em `cs_tracker` (28
pontos `1-28` + o ritmo depois de 28), interpolada a cada 10 s nos primeiros 5
minutos; depois de 28 min, média ponderada. Mostra também a diferença
percentual. Com o atalho (ou sempre, conforme a opção), mostra **de onde veio a
média**: o seu campeão, o inimigo de rota e, no bot, o parceiro e o par inimigo
— com aviso "Not enough data" quando algum caiu para a média do elo. Habilitado
por rota.

## 16. `aram-mayhem-augments`

Quando o GEP informa os três aprimoramentos oferecidos, desenha sobre cada carta
a **nota do tier** (S/A verde, B/C amarelo, D vermelho) para o seu campeão
(`api:get-aram-champion-augments`). Casa nome localizado → id → nome em inglês,
porque o GEP só dá o nome no idioma do cliente. Some quando você escolhe.
