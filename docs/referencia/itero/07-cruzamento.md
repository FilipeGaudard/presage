# 07 — Cruzamento com o Presage

Cada funcionalidade do [catálogo](05-funcionalidades.md) contra o que o Presage
tem em 2026-10-28 (branch `feat/draft-analise`).

| Situação | Significa |
|---|---|
| **tem** | o Presage faz o mesmo, ou o equivalente por outro caminho |
| **parcial** | faz uma parte; a coluna de nota diz qual |
| **sem dado** | o código está pronto e vale zero até a base de partidas existir ([P-1](../../produto/perguntas-em-aberto.md)) |
| **falta** | não existe e cabe nas regras |
| **regra** | a regra da Riot ([C-\*](../../produto/requisitos.md)) ou um princípio registrado do produto impede |
| **decisão** | a Riot permite, mas muda o que o Presage faz hoje; é escolha do dono do produto |
| **sem fonte** | o dado só existe pelo Overwolf (GEP) ou pela memória do jogo; nenhuma API oficial entrega ([C-2](../../produto/requisitos.md)) |
| **fora** | fora do produto: plataforma, monetização, ARAM |

## 1. Draft

| Id | Funcionalidade | Presage | Nota |
|---|---|---|---|
| I-1 | Recomendação por modelo | parcial · sem dado | encaixe de composição + winrate, matchup, contra e sinergia em log-odds ([ADR-0019](../../adr/0019-contra-em-qualquer-rota-e-chance-do-time.md)); sem modelo treinado, e sem maestria — o pessoal é o histórico contra o oponente direto ([ADR-0010](../../adr/0010-oponente-direto-como-grupo-de-controle.md)) |
| I-2 | Nota relativa 0–5 | regra | o Presage mostra probabilidade com `±` e diz o que é medição ou palpite ([visão geral](../../produto/visao-geral.md#o-diferencial)); nota relativa esconde a incerteza |
| I-3 | Nota da fase de rota | parcial | curva de nível 1–18 e vantagem de nível pela conta de atributos ([03 §8](../../arquitetura/03-motor-da-partida.md)); sem ouro previsto |
| I-4 | Por que este campeão | tem | motivo em texto por componente |
| I-5 | Matchups do seu campeão | parcial · sem dado | 2v2/3v3 por atributos na prévia; `versus` contra os cinco na estimativa do time |
| I-6 | Estilo de jogo por faixa (Safe/Poke/Bully…) | falta | precisa de troca de dano e abates por confronto ao longo do tempo: timeline de partida no coletor |
| I-7 | Estatísticas do time | tem | mistura de dano, linha de frente, alcance ([ADR-0018](../../adr/0018-o-que-a-condicao-de-vitoria-pode-afirmar.md)); força por fase sai do `length` (sem dado) |
| I-8 | Recomendação de ban | parcial | `suggest_bans` só por composição ([ADR-0013](../../adr/0013-ban-nao-personalizado-enquanto-aditivo.md)) |
| I-9 | Ban pool | **falta** | |
| I-10 | Pool priorizada | tem | [ADR-0016](../../adr/0016-pool-de-oito-tres-escolhidos-cinco-por-cobertura.md) |
| I-11 | Filtro de pick rate | parcial | o filtro de rota usa a fatia de jogo da rota; não há controle do usuário |
| I-12 | Só campeões que você tem | **falta** | leitura do LCU, permitida |
| I-13 | Hover dos aliados | tem | [L-26](../../arquitetura/09-licoes-do-prototipo.md) |
| I-14 | Pesos do modelo pelo usuário | falta | baixa prioridade: pesos à mão mexidos pelo usuário viram mais palpite |
| I-15 | Elo da estatística | parcial | uma base por arquivo; o coletor escolhe o tier |
| I-16 | Maestria agregada | fora | o Presage não usa maestria |
| I-17 | Override da rota do inimigo | **falta** | o Presage infere ([L-25](../../arquitetura/09-licoes-do-prototipo.md)) mas não deixa corrigir |
| I-18 | Build por composição | parcial | runas do confronto no draft; build adaptativa só na partida |
| I-19 | Enviar runas e itens ao cliente | decisão | hoje o Presage só **lê** o LCU (`src-tauri/src/lcu/client.rs` só tem `GET`); escrever é uma escolha nova |
| I-20 | Auto-hover | decisão | escreve no LCU, como I-19; e beira [C-5](../../produto/requisitos.md) |
| I-21 | Auto-lock | regra | [C-5](../../produto/requisitos.md): tira a decisão do jogador |
| I-22 | Auto-hover de ban | decisão | como I-20 |
| I-23 | Desligar automação ao mexer | — | só existe com I-20 |
| I-24 | Recomendação a cada fase | tem | refaz a cada mudança do champ select |
| I-25 | Personalizada, Practice Tool, torneio | tem | não filtra por fila |
| I-26 | ARAM | fora | o Presage é de Summoner's Rift |
| I-27 | Simulador de draft | **falta** | montar os dois times à mão e rodar o mesmo motor |
| I-28 | Construtor de pool | tem | 3 escolhidos + 5 por cobertura; sem checagem AP/AD explícita nem par do bot |
| I-29 | Tier list | sem dado | sai da base quando existir |
| I-30 | Quiz de personalidade | fora | |

## 2. Partida

| Id | Funcionalidade | Presage | Nota |
|---|---|---|---|
| I-31 | Abrir a tela da partida sozinho | tem | overlay em janela externa ([ADR-0003](../../adr/0003-overlay-como-janela-externa.md)) |
| I-32 | Scouting dos dez | falta | precisa de histórico de cada jogador: Riot Web API com chave de produção ([C-10](../../produto/requisitos.md)) |
| I-33 | Verificar o draft com as posições reais | tem | a análise da partida usa a posição da Live Client e ganha do draft (`analysis.rs`) |
| I-34 | Pairings na partida | parcial | chance por rota e escaramuça com nível e itens reais; sem par a par com os nove |
| I-35 | Força por faixa de duração | sem dado | `team.rs` + tabela `length` ([ADR-0019](../../adr/0019-contra-em-qualquer-rota-e-chance-do-time.md)) |
| I-36 | Estado da rota | parcial | ouro em itens dos dez; não diz "atrás/igual/à frente" contra o oponente direto |
| I-37 | Build da partida | tem | adaptativa contra os itens deles ([C-6](../../produto/requisitos.md)) |
| I-38 | Timer de inibidor e torre do nexo | **falta** | eventos `InhibKilled` e `TurretKilled` da Live Client |
| I-39 | Relíquias de vida | fora · sem fonte | ARAM, e o dado vem do GEP |
| I-40 | Buff de Barão e Ancião, com quem tem | **falta** | eventos `BaronKill`/`DragonKill` + `isDead` da Live Client; o Presage hoje só faz o respawn |
| I-41 | Valor em itens por jogador e time | tem | `gold.rs`, exato |
| I-42 | Suprema dos aliados | sem fonte | `team_frames` é do GEP; a Live Client não entrega recarga |
| I-43 | Dano por tipo, medido | sem fonte · parcial | a Live Client não entrega dano; o Presage estima a divisão do dano que vem pelos itens deles |
| I-44 | Próxima habilidade | falta | precisa de ordem de habilidade: timeline no coletor |
| I-45 | CS/min contra referência | **falta** | o iTero compara com a média do elo; dá para comparar ao vivo com o **oponente direto**, e só nas rotas que farmam (suporte: visão) |
| I-46 | Aprimoramentos do Mayhem | fora | |
| I-47 | Atalhos globais | tem | mostrar e interagir com o overlay |

## 3. Pós-jogo e conta

| Id | Funcionalidade | Presage | Nota |
|---|---|---|---|
| I-48 | Pós-partida automático | tem | gravação local + leitura ([03](../../arquitetura/03-motor-da-partida.md)) |
| I-49 | Detalhe da partida | parcial | leitura com tropas, visão, participação e mortes pela rota; sem nota de draft nem de macro |
| I-50 | Estatísticas da rota no gráfico | tem | suporte com visão no lugar de tropas ([L-27](../../arquitetura/09-licoes-do-prototipo.md)) |
| I-51 | Estatísticas da conta com percentil | parcial · regra | perfil de rota contra o oponente direto em cinco métricas ([04](../../arquitetura/04-perfil-de-rota.md)); percentil contra a população exige a população e beira [C-7](../../produto/requisitos.md) |
| I-52 | Títulos por estatística | falta | cosmético |
| I-53 | Elogio/crítica por IA | fora | |
| I-54 | Leaderboard de amigos | regra | [C-7](../../produto/requisitos.md): escada paralela |
| I-55 | Meu winrate × comunidade por campeão | parcial · sem dado | perfil por campeão; o da comunidade sai da base |
| I-56 | Coach de pontos de melhoria | parcial | uma partida por vez (`coach.rs`) e o perfil agregado; sem faixas de 7 min |
| I-57 | Gráfico de PDL | falta | a Riot não dá histórico de PDL; teria de ser gravado partida a partida |
| I-58 | Várias contas | falta | |

## 4. Plataforma

| Id | Funcionalidade | Presage | Nota |
|---|---|---|---|
| I-59 | Conta própria e nuvem | fora | o Presage não tem servidor |
| I-60 | Assinatura | fora | [C-8](../../produto/requisitos.md) |
| I-61 | Anúncios | fora | [C-9](../../produto/requisitos.md) proíbe dentro do jogo |
| I-62 | Idiomas | falta | só português |
| I-63 | Atualização automática | tem | [06](../../arquitetura/06-distribuicao-e-atualizador.md) |
| I-64 | Bandeja, autostart, instância única | falta | |
| I-65 | Tutorial de primeiro uso | falta | |

## 5. Contagem

| Situação | Quantas |
|---|---|
| tem | 15 |
| parcial | 13 |
| sem dado | 2 |
| falta | 17 |
| regra | 3 |
| decisão | 3 |
| sem fonte | 2 |
| fora | 9 |
| — | 1 |

Pela situação principal de cada linha (a primeira, antes do `·`); total 65.
O `conferir.py` recalcula e falha se esta tabela divergir das linhas.

## 6. As lacunas que cabem, por valor

Só o que está em **falta** e passa nas regras, ordenado por valor para o jogador
sobre custo. "Dado" diz se funciona hoje ou espera a base.

| # | Lacuna | Ids | Dado | Custo | Por quê |
|---|---|---|---|---|---|
| G-1 | Corrigir à mão a rota de cada inimigo no draft | I-17 | hoje | baixo | a inferência erra; um clique conserta o matchup de rota, a prévia e a condição de vitória — a rota é o fator principal |
| G-2 | Painel da sua rota ao vivo: contra o oponente direto, o que a rota cobra — tropas/min nas rotas que farmam, visão no suporte — e atrás/igual/à frente em ouro de itens | I-36, I-45 | hoje | médio | mesma regra do pós-jogo (`cobranca`), agora durante a partida |
| G-3 | Timer de inibidor e de torre do nexo | I-38 | hoje | baixo | evento oficial, já está na tela de quem viu |
| G-4 | Buff de Barão e Ancião: tempo restante e quem ainda tem | I-40 | hoje | baixo | idem |
| G-5 | Ban pool do jogador | I-9 | hoje | baixo | prioriza os seus bans sobre a sugestão por composição |
| G-6 | Mostrar só campeões que você tem | I-12 | hoje | baixo | leitura do LCU; evita recomendar o que não dá para pegar |
| G-7 | Simulador de draft | I-27 | hoje | médio | testar o motor e planejar sem champ select — e permite usar o Presage no Kali |
| G-8 | Bandeja e iniciar com o Windows | I-64 | hoje | médio | conforto |
| G-9 | Estilo de jogo da rota por faixa | I-6 | base + timeline | alto | precisa estender o coletor |
| G-10 | Próxima habilidade a subir | I-44 | base + timeline | médio | idem |
| G-11 | Scouting dos dez | I-32 | chave de produção | alto | |
| G-12 | Idiomas, tutorial, várias contas, títulos, PDL | I-52, I-57, I-58, I-62, I-65 | hoje | variado | baixo valor agora |

**Decisões que não são minhas** (mudam regra registrada):

- **Escrever no LCU** (I-19 enviar runas/itens, I-20 e I-22 auto-hover): é o
  que o iTero mais faz no champ select, e ele diz ter sido aprovado pela Riot
  com isso. O Presage hoje só lê; passar a escrever pede um ADR (o que pode ser
  escrito, quando, e como desligar).
- **Auto-lock** (I-21): fica de fora pela C-5 em qualquer leitura.
