# 05 — Catálogo de funcionalidades

Uma linha por funcionalidade, com um id (`I-n`) que o
[cruzamento](07-cruzamento.md) usa. O detalhe fica onde já está descrito; aqui
entra só o que não cabe em outro lugar (o scouting, §2).

## 1. O catálogo

### Draft

| Id | Funcionalidade | Onde está descrita |
|---|---|---|
| I-1 | Recomendação de pick por modelo treinado (maestria, winrate, contra de rota, ouro aos 14, jogar à frente/atrás, contra e sinergia do time, estatísticas do time) | [04 §2](04-telas-da-janela-principal.md#2-guia-do-usuário), [01 §5](01-processo-principal.md#5-champ-select-e-draft) |
| I-2 | Nota relativa 0,0–5,0 (Overall Score) | [02 §2](02-janelas.md#2-drafting--o-companheiro-do-champ-select) |
| I-3 | Laning Phase Score; ordenar e filtrar por ela | idem |
| I-4 | Por que este campeão: "works well with / doesn't", "strong vs / bad vs" | idem |
| I-5 | Matchups do seu campeão contra cada inimigo e com cada aliado | idem |
| I-6 | Estilo de jogo do confronto por faixa de tempo (Neutral/Safe/Sustain/Poke/Bully; selva: Farm and Crossmap / Gank and Invade), com Trading e Kill Threat | idem |
| I-7 | Team Stats: AD/AP, dano total, dano recebido, força no início/meio/fim | idem |
| I-8 | Recomendação de ban: contra o seu hover, por lado, geral | [01 §5](01-processo-principal.md#automação-do-champ-select) |
| I-9 | Ban pool (até 5, em ordem) | idem |
| I-10 | Pool de campeões priorizada na recomendação | idem |
| I-11 | Filtro de pick rate (5 níveis, Non-Meta ↔ Meta) | [01 §8](01-processo-principal.md#8-configurações-do-usuário-settings) |
| I-12 | Mostrar só campeões que você tem, ou todos | idem |
| I-13 | Considerar o hover dos aliados | [04 §2](04-telas-da-janela-principal.md#2-guia-do-usuário) |
| I-14 | Customização do modelo: peso por termo, por rota | [01 §5](01-processo-principal.md#as-duas-chamadas) |
| I-15 | Elo da recomendação: o seu ou um fixo | idem |
| I-16 | Maestria agregada de várias contas | idem |
| I-17 | Override da posição dos inimigos | [02 §2](02-janelas.md#2-drafting--o-companheiro-do-champ-select) |
| I-18 | Build por composição: runas, feitiços, fragmentos, itens, ordem de habilidade | idem |
| I-19 | **Enviar build ao cliente** (página de runas e conjunto de itens), manual e automático | [03 §2](03-ipc-e-ponte.md#registrados-pelo-helper-api-mainjs26233-26354) |
| I-20 | **Auto-hover** do melhor pick | [01 §5](01-processo-principal.md#automação-do-champ-select) |
| I-21 | **Auto-lock** no fim do cronômetro | idem |
| I-22 | **Auto-hover de ban** | idem |
| I-23 | Desliga a automação quando você mexe no cliente | [01 §5](01-processo-principal.md#leitura-do-champ-select) |
| I-24 | Recomendação contínua a cada fase (assinante) ou só na sua vez | [04 §7](04-telas-da-janela-principal.md#7-premium-premium) |
| I-25 | Suporte a personalizada, Practice Tool e torneio (assinante) | [01 §4](01-processo-principal.md#4-a-máquina-de-fases) |
| I-26 | ARAM: grade, campeão aleatório, builds, build aleatória por raridade, aprimoramentos do Mayhem | [02 §3](02-janelas.md#3-drafting-aram) |
| I-27 | Simulador de draft com importação de partida e sorteio | [04 §6](04-telas-da-janela-principal.md#simulador-de-draft-toolsdrafting-tool) |
| I-28 | Construtor de pool conversacional (2 blinds, 3 situacionais, par do bot, pocket, AP/AD) | [04 §6](04-telas-da-janela-principal.md#construtor-de-pool-toolschampion-pool-builder) |
| I-29 | Tier list geral e pessoal | [04 §5](04-telas-da-janela-principal.md#5-tier-list-tier-list) |
| I-30 | Quiz de personalidade | [04 §6](04-telas-da-janela-principal.md#quiz-de-personalidade-toolsquiz) |

### Partida

| Id | Funcionalidade | Onde está descrita |
|---|---|---|
| I-31 | Abrir a tela da partida sozinho; overlay num monitor, janela solta com dois | [01 §4](01-processo-principal.md#4-a-máquina-de-fases) |
| I-32 | Scouting dos dez jogadores (rótulos, §2) | §2 |
| I-33 | Verificar o draft com as posições reais da partida e refazer a análise | [01 §5](01-processo-principal.md#verificação-do-draft-no-jogo) |
| I-34 | Pairings: você com e contra cada um dos nove; corrigir o draft | [02 §4–5](02-janelas.md#45-ingame-e-ingame-second) |
| I-35 | Team Scaling Advantage em três faixas de duração (15–26, 27–32, 33+) | idem |
| I-36 | Estado da rota (atrás/igual/à frente pelo ouro, limiares por rota) e dica/build por estado | [01 §9](01-processo-principal.md#9-eventos-ao-vivo) |
| I-37 | Build de partida: iniciais, ordem, alternativas | [02 §4–5](02-janelas.md#45-ingame-e-ingame-second) |
| I-38 | Timers de inibidor e torre do nexo | [02 §11](02-janelas.md#11-map-timers) |
| I-39 | Timers de relíquia de vida (ARAM) | idem |
| I-40 | Timer de buff de Barão e Ancião, com quem tem | [02 §8](02-janelas.md#8-monster-buff-timers) |
| I-41 | Valor em itens por jogador e por time | [02 §9](02-janelas.md#9-team-gold-tracker) |
| I-42 | Recarga da suprema dos **aliados** | [02 §10](02-janelas.md#10-ult-cooldown-timers) |
| I-43 | Dano causado e recebido por tipo | [02 §12](02-janelas.md#12-damage-tracker) |
| I-44 | Próxima habilidade a subir | [02 §14](02-janelas.md#14-skill-level-up-tracker) |
| I-45 | CS por minuto contra a média do elo **no seu confronto** | [02 §15](02-janelas.md#15-stat-tracker--cs-por-minuto) |
| I-46 | Nota de tier dos aprimoramentos oferecidos (ARAM Mayhem) | [02 §16](02-janelas.md#16-aram-mayhem-augments) |
| I-47 | Atalhos globais (tela in-game, troca de tela, overlay) | [01 §10](01-processo-principal.md#10-atalhos-e-janelas) |

### Pós-jogo e conta

| Id | Funcionalidade | Onde está descrita |
|---|---|---|
| I-48 | Análise pós-partida no servidor, abrindo a partida sozinha | [01 §6](01-processo-principal.md#6-pós-partida) |
| I-49 | Detalhe da partida: AI Draft Score, AI Macro Score, sinergia e contra, estatísticas com média da conta, do elo e percentil | [04 §3](04-telas-da-janela-principal.md#detalhe-da-partida-profilematch-historymatchid) |
| I-50 | Gráfico minuto a minuto com as **seis estatísticas da rota** (suporte: visão e sentinelas no lugar de CS) | idem |
| I-51 | Estatísticas da conta: ~500, faixas de 7 min, ajuste pela pool, percentil no elo, média móvel de 50 | [04 §3](04-telas-da-janela-principal.md#estatísticas-da-conta-profileaccount-statistics) |
| I-52 | Títulos por estatística ("The Farmer", "Blind") | idem |
| I-53 | Elogio / crítica gerados por IA | idem |
| I-54 | Leaderboard de amigos, por estatística | idem |
| I-55 | Estatísticas por campeão: meu winrate × comunidade, Draft Score | [04 §3](04-telas-da-janela-principal.md#estatísticas-por-campeão-profilechampion-statistics) |
| I-56 | AI Macro Coach: forças e pontos de melhoria por faixa de 7 min | [04 §4](04-telas-da-janela-principal.md#4-ai-macro-coach-ai-coach) |
| I-57 | Gráfico de PDL da ranqueada solo | [04 §3](04-telas-da-janela-principal.md#3-perfil) |
| I-58 | Várias contas Riot | [04 §8](04-telas-da-janela-principal.md#8-configurações) |

### Plataforma

| Id | Funcionalidade | Onde está descrita |
|---|---|---|
| I-59 | Conta própria (Firebase Auth) e configurações na nuvem | [02 §13](02-janelas.md#13-background) |
| I-60 | Assinatura, teste grátis, cupom | [04 §7](04-telas-da-janela-principal.md#7-premium-premium) |
| I-61 | Anúncios na janela principal | [02 §1](02-janelas.md#1-desktop--a-janela-principal) |
| I-62 | 12 idiomas | [04 §8](04-telas-da-janela-principal.md#8-configurações) |
| I-63 | Atualização automática silenciosa, reabrindo como estava | [01 §1](01-processo-principal.md#1-inicialização) |
| I-64 | Bandeja, autostart, instância única, reinício como admin | idem |
| I-65 | Tutorial de primeiro uso no draft e na janela principal | [02 §2](02-janelas.md#2-drafting--o-companheiro-do-champ-select) |

## 2. Scouting de jogadores

`POST {região}/lake-team-scouting` com a composição por posição e o Riot ID de
cada um (`main.js:36862`). A resposta traz, para cada um dos dez: Riot ID,
campeão, elo e PDL, estatísticas **no campeão** e **no geral** — participação em
abates, KDA, mortes, mortes cedo, assistências, monstros épicos, grubs, dragões,
torres, invasões, visão/min, participação na rota, participação fora da rota
(roam), CS/min, CS/min na rota, vitórias, partidas, partidas na rota — e uma
lista de **rótulos** (`main.js:36809`).

Os rótulos (`bloco3` do recurso de tradução) seguem um padrão: uma estatística,
seis graus (`ultra_low`, `very_low`, `low`, `high`, `very_high`, `ultra_high`),
com nome e frase. As estatísticas:

| Família | Graus de exemplo | O que mede |
|---|---|---|
| campeão | Monster, Underperformer, No Recent Games, Noob, One-Trick Player, Main, Player | winrate e partidas recentes no campeão |
| conta | High Performer, Poor Performer | winrate na conta |
| rota | Never Plays, Off-Role | quanto joga a rota |
| `csm` | Zero Farm … Exceptional Farmer | CS/min |
| `kp` | Zero Fighting … Always Fighting | participação em abates |
| `towers` | Never Split-Pushes … Always Split-Pushing | dano a torre |
| `monsters` | Zero Epic Monsters … Epic Monster Slayer | participação em objetivo |
| `vpm` | Zero Vision Score … All-Seeing Vision | visão/min |
| `lane_kp` | AFK Early Game … Early Game Menace | participação cedo |
| `early_deaths` | Zero Deaths Early Game … Total Inter Early Game | mortes cedo |
| `roam_kp` | Never Ganks … Non-Stop Ganking | abates em outras rotas cedo |

As frases dão a leitura tática ("ele não vai aparecer para lutar", "puxe a
vantagem cedo, ele nunca jogou o confronto"). O modo avançado troca os cartões
por comparações de fase de rota e meio de jogo.

Nota: os rótulos de farm (`csm_*`) existem para todo jogador, inclusive o
suporte; o recorte por rota, se houver, é do servidor e não aparece no cliente.
