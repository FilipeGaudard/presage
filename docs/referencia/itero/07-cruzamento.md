# 07 — Cruzamento com o Presage

Cada funcionalidade do [catálogo](05-funcionalidades.md) contra o que o Presage
tem, e o que fazer com ela. A situação das linhas I-1 a I-65 foi lida em
2026-10-28 (branch `feat/draft-analise`), já com as lacunas G-1 a G-8
implementadas (§6). As linhas I-66 a I-83 e a coluna **Decisão** são de
2026-11-07.

| Situação | Significa |
|---|---|
| **tem** | o Presage faz o mesmo, ou o equivalente por outro caminho |
| **parcial** | faz uma parte; a coluna de nota diz qual |
| **sem dado** | o código está pronto e vale zero até a base de partidas existir ([P-1](../../produto/perguntas-em-aberto.md)) |
| **falta** | não existe e cabe nas regras |
| **regra** | a regra da Riot ([C-\*](../../produto/requisitos.md)) ou um princípio registrado do produto impede |
| **decisão** | a Riot permite, mas muda o que o Presage faz hoje; é escolha do dono do produto ([P-15](../../produto/perguntas-em-aberto.md)) |
| **sem fonte** | o dado só existe pelo Overwolf (GEP) ou pela memória do jogo; nenhuma API oficial entrega ([C-2](../../produto/requisitos.md)) |
| **fora** | fora do produto; a nota diz por quê |

| Decisão | Significa |
|---|---|
| **copiar** | o Presage faz, ou vai fazer, como o iTero |
| **mudar** | o Presage faz a mesma coisa do jeito dele; a nota diz qual |
| **rejeitar** | não entra: uma regra da Riot (C-n) ou um princípio registrado (RNF-n) impede |
| **adiar** | cabe, mas espera dado (a base, R-21; a timeline, [P-3](../../produto/perguntas-em-aberto.md); a chave de produção, C-10) ou uma pergunta em aberto |
| **fora** | fora do produto; a nota diz por quê |

A célula de decisão traz a palavra, o requisito ([R-n](../../produto/requisitos.md#3-requisitos-funcionais-r))
ou a regra que ela toca, e **`a revisar`** quando a decisão foi derivada aqui e
o dono do produto ainda não confirmou. Sem `a revisar`, a decisão já está
escrita no requisito, no RNF, na regra ou no ADR citado. `R novo` quer dizer que
nenhum R-1 a R-21 cobre: aceitar a decisão pede um requisito novo.

## 1. Draft

| Id | Funcionalidade | Presage | Decisão | Nota |
|---|---|---|---|---|
| I-1 | Recomendação por modelo | parcial · sem dado | adiar · R-1, R-21 · a revisar | encaixe de composição + winrate, matchup, contra e sinergia em log-odds ([ADR-0019](../../adr/0019-contra-em-qualquer-rota-e-chance-do-time.md)); sem modelo treinado, e sem maestria — o pessoal é o histórico contra o oponente direto ([ADR-0010](../../adr/0010-oponente-direto-como-grupo-de-controle.md)). O resto são os termos da base, que já estão no encaixe e valem zero sem ela |
| I-2 | Nota relativa 0–5 | regra | rejeitar · RNF-1, RNF-2 | o Presage mostra probabilidade com `±` e diz o que é medição ou palpite ([visão geral](../../produto/visao-geral.md#o-diferencial)); nota relativa esconde a incerteza |
| I-3 | Nota da fase de rota | parcial | adiar · R-6, R-21 · a revisar | curva de nível 1–18 e vantagem de nível pela conta de atributos ([03 §8](../../arquitetura/03-motor-da-partida.md)); sem ouro previsto, que o iTero tira do modelo (ouro aos 14) e o Presage só teria com a base |
| I-4 | Por que este campeão | tem | copiar · R-1 | motivo em texto por componente, até três por candidato |
| I-5 | Matchups do seu campeão | parcial · sem dado | adiar · R-6, R-21 · a revisar | 2v2/3v3 por atributos na prévia; `versus` contra os cinco na estimativa do time. A nota par a par com cada inimigo e cada aliado é matchup e sinergia da base |
| I-6 | Estilo de jogo por faixa (Safe/Poke/Bully…) | falta | adiar · R-20, P-3 · a revisar | precisa de troca de dano e abates por confronto ao longo do tempo: timeline de partida no coletor (G-9) |
| I-7 | Estatísticas do time | tem | copiar · R-6, R-21 | mistura de dano, linha de frente, alcance ([ADR-0018](../../adr/0018-o-que-a-condicao-de-vitoria-pode-afirmar.md)); força por fase sai do `length` quando a base existir |
| I-8 | Recomendação de ban | parcial | adiar · R-3, R-21 | `suggest_bans` só por composição ([ADR-0013](../../adr/0013-ban-nao-personalizado-enquanto-aditivo.md)); ban contra o seu hover e por lado espera a variância de matchup da base, como o ADR decide |
| I-9 | Ban pool | tem | copiar · R-3 | até 5, em ordem, na frente desde o primeiro ban, sem banir hover de aliado (`bans_with_pool`, G-5) |
| I-10 | Pool priorizada | tem | copiar · R-9 | a pool sai numa seção antes do resto, marcada e sem mudar a ordem ([ADR-0016](../../adr/0016-pool-de-oito-tres-escolhidos-cinco-por-cobertura.md)) |
| I-11 | Filtro de pick rate | parcial | adiar · R-1, R-21 · a revisar | o filtro de rota usa a fatia de jogo da rota; não há controle do usuário. Um controle "meta ↔ fora do meta" pede pick rate por rota, que é da base |
| I-12 | Só campeões que você tem | tem | copiar · R-5 | grade do champ select + fila, só leitura; rotação grátis fora da ranqueada (`draft::selectable`, G-6) |
| I-13 | Hover dos aliados | tem | copiar · R-1 | [L-26](../../arquitetura/09-licoes-do-prototipo.md) |
| I-14 | Pesos do modelo pelo usuário | falta | rejeitar · RNF-1 · a revisar | pesos à mão mexidos pelo usuário viram mais palpite, e o número na tela deixa de dizer de onde veio |
| I-15 | Elo da estatística | parcial | adiar · R-20, R-21 · a revisar | uma base por arquivo; o coletor escolhe o tier. Escolher o elo na tela pede uma base por tier |
| I-16 | Maestria agregada | fora | fora · ADR-0010 | o Presage não usa maestria: o componente pessoal é o histórico contra o oponente direto ([ADR-0010](../../adr/0010-oponente-direto-como-grupo-de-controle.md)), e a soma de várias contas não tem onde entrar |
| I-17 | Override da rota do inimigo | tem | copiar · R-4 | a marca vence a inferência ([L-25](../../arquitetura/09-licoes-do-prototipo.md)) e nunca a posição do cliente (`Leitura::enemy_roles`, G-1) |
| I-18 | Build por composição | parcial | mudar · R-7, R-10 · a revisar | runas do confronto no draft; build adaptativa só na partida. O Presage escolhe itens contra os itens deles ([C-6](../../produto/requisitos.md)), que só existem na partida; antes dela sobra o prior de classe, fraco. Feitiços e ordem de habilidade pedem a base e a timeline |
| I-19 | Enviar runas, itens e feitiços ao cliente | decisão | adiar · R-18, P-15 · a revisar | hoje o Presage só **lê** o LCU (`src-tauri/src/lcu/client.rs` só tem `GET`); escrever é escolha nova, e o default da [P-15](../../produto/perguntas-em-aberto.md) é "LCU só leitura" |
| I-20 | Auto-hover | decisão | adiar · R-18, P-15 · a revisar | escreve no LCU, como I-19; e beira [C-5](../../produto/requisitos.md) |
| I-21 | Auto-lock | regra | rejeitar · C-5 | [C-5](../../produto/requisitos.md): tira a decisão do jogador; a P-15 também o deixa de fora |
| I-22 | Auto-hover de ban | decisão | adiar · R-18, P-15 · a revisar | como I-20; e o Presage nunca bane sozinho (R-3) |
| I-23 | Desligar automação ao mexer | decisão | adiar · R-18, P-15 · a revisar | só existe com I-20 e I-22, e cai junto com eles |
| I-24 | Recomendação a cada fase | tem | copiar · R-1 | refaz a cada mudança do champ select, lido a cada 350 ms (RNF-4) |
| I-25 | Personalizada, Practice Tool, torneio | tem | copiar · R-1 | não filtra por fila. Swiftplay e Quickplay não têm champ select; a partida vem pela Live Client como em qualquer fila |
| I-26 | ARAM | fora | fora · Summoner's Rift | o Presage é de Summoner's Rift: o histórico e o coletor descartam o ARAM (R-17, R-20), e rotas, timers e o motor são do mapa de cinco rotas |
| I-27 | Simulador de draft | tem | copiar · R-8 · a revisar | aba Simulador: o mesmo motor lê o draft montado à mão (`set_simulated_draft`, G-7). O resto: importar uma partida espera o histórico (I-72); sortear o draft não ajuda a decidir e não entra |
| I-28 | Construtor de pool | tem | copiar · R-9 · a revisar | 3 escolhidos + 5 por cobertura. O resto: dano físico e mágico já são características da cobertura ([02 §7](../../arquitetura/02-motor-de-draft.md#7-pool)), então a checagem AP/AD está dentro dela; o par do bot pede a sinergia da base (R-21) |
| I-29 | Tier list | sem dado | adiar · R-21 | sai da base quando existir |
| I-30 | Quiz de personalidade | fora | fora · RNF-1 | campeão por afinidade de personalidade não sai de partida nenhuma; o Presage não recomenda sem origem (RNF-1), e o quiz não ajuda a decidir um draft |
| I-66 | Hover, travar e banir à mão | decisão | adiar · R-18, P-15 · a revisar | o botão é do jogador e não tira decisão (a C-5 permite), mas é escrita no LCU como I-19; com o LCU só leitura, o jogador faz isso no cliente |
| I-67 | Botão "Open Draft" | falta | mudar · R-1 · a revisar | no Presage o draft é uma aba da janela única (`App.tsx`), não uma janela; o equivalente é trazer a aba Draft para a frente quando o champ select começa, como a leitura pós-partida já faz com a aba dela |
| I-68 | Dois tamanhos da janela do draft | parcial | adiar · R-1 · a revisar | uma janela redimensionável de 1320×840, com mínimo de 1024 px de largura (`src-tauri/tauri.conf.json`); não cabe estreita ao lado do cliente. Um modo estreito pede outro layout do draft |
| I-83 | Build de ARAM com perfil de dano e etiquetas | fora | fora · Summoner's Rift | ARAM, como I-26 |

## 2. Partida

| Id | Funcionalidade | Presage | Decisão | Nota |
|---|---|---|---|---|
| I-31 | Abrir a tela da partida sozinho | tem | mudar · R-15, C-3 | overlay em janela externa ([ADR-0003](../../adr/0003-overlay-como-janela-externa.md)), não injetado no processo do jogo como o do Overwolf |
| I-32 | Scouting dos dez | falta | adiar · R novo, C-10 · a revisar | precisa de histórico de cada jogador: Riot Web API com chave de produção ([C-10](../../produto/requisitos.md)) e o backend da [ADR-0004](../../adr/0004-chave-da-riot-so-no-backend.md) |
| I-33 | Verificar o draft com as posições reais | tem | copiar · R-11 | a análise da partida usa a posição da Live Client e ganha do draft (`analysis.rs`) |
| I-34 | Pairings na partida | parcial | adiar · R-11, R-21 · a revisar | chance por rota e escaramuça com nível e itens reais; sem par a par com os nove, que é matchup e sinergia da base |
| I-35 | Força por faixa de duração | sem dado | adiar · R-21 | `team.rs` + tabela `length` ([ADR-0019](../../adr/0019-contra-em-qualquer-rota-e-chance-do-time.md)) |
| I-36 | Estado da rota | tem | mudar · R-11 | atrás/igual/à frente pelo ouro em itens contra o oponente direto, não contra limiares fixos por rota (`rival.rs`, G-2) |
| I-37 | Build da partida | tem | copiar · R-10 | adaptativa contra os itens deles ([C-6](../../produto/requisitos.md)) |
| I-38 | Timer de inibidor e torre do nexo | tem | copiar · R-13 | `Respawn` a partir de `InhibKilled`/`TurretKilled`, nos dois formatos de nome ([P-14](../../produto/perguntas-em-aberto.md), G-3) |
| I-39 | Relíquias de vida | fora · sem fonte | fora · C-2 | ARAM, e o dado vem do GEP |
| I-40 | Buff de Barão e Ancião, com quem tem | tem | copiar · R-13 | `BuffTracker`: quem estava vivo no abate, perde quem morre (G-4) |
| I-41 | Valor em itens por jogador e time | tem | copiar · R-12 | `gold.rs`, exato |
| I-42 | Suprema dos aliados | sem fonte | rejeitar · C-2 | `team_frames` é do GEP; a Live Client não entrega recarga, e ler a memória do jogo é proibido ([C-2](../../produto/requisitos.md)) |
| I-43 | Dano por tipo, medido | sem fonte · parcial | mudar · R-10, C-2 | a Live Client não entrega dano; o Presage estima a divisão do dano que vem pelos itens deles |
| I-44 | Próxima habilidade | falta | adiar · R-20, P-3 · a revisar | precisa de ordem de habilidade: timeline no coletor (G-10) |
| I-45 | CS/min contra referência | tem | mudar · R-11 | contra o **oponente direto** ao vivo, só nas rotas que farmam; o suporte é medido em visão (`rival.rs`, G-2). O iTero compara com a média do elo, que exige a base |
| I-46 | Aprimoramentos do Mayhem | fora | fora · C-2 | ARAM Mayhem, e os aprimoramentos oferecidos só chegam pelo GEP (`me.augments`) |
| I-47 | Atalhos globais | tem | copiar · R-15 | mostrar e interagir com o overlay (Ctrl+Shift+P e Ctrl+Shift+O, `overlay.rs`) |
| I-69 | Modo streamer do cliente | tem | copiar · R-12 | o Presage não mostra nome de jogador, só campeão; o nome (`display_name`) serve de chave e não aparece na tela |
| I-70 | Atalhos remapeáveis | falta | copiar · R-15 · a revisar | os dois atalhos são fixos (`overlay.rs`); se outro app ocupa a tecla, o registro falha e o atalho some com um aviso só no log (`lib.rs`), que no build de release não aparece. Remapear resolve isso |

## 3. Pós-jogo e conta

| Id | Funcionalidade | Presage | Decisão | Nota |
|---|---|---|---|---|
| I-48 | Pós-partida automático | tem | copiar · R-16 | gravação local + leitura ([03](../../arquitetura/03-motor-da-partida.md)); a aba vem para a frente quando a leitura fica pronta |
| I-49 | Detalhe da partida | parcial | rejeitar · R-16, RNF-11 | leitura com tropas, visão, participação e mortes pela rota; o que falta são a nota de draft e a de macro, e o pós-jogo aponta sem dar nota (RNF-11) |
| I-50 | Estatísticas da rota no gráfico | tem | copiar · R-16 | suporte com visão no lugar de tropas ([L-27](../../arquitetura/09-licoes-do-prototipo.md)) |
| I-51 | Estatísticas da conta com percentil | parcial · regra | adiar · R-17, P-3 · a revisar | perfil de rota contra o oponente direto em cinco métricas ([04](../../arquitetura/04-perfil-de-rota.md)); as faixas de 7 min pedem a timeline (P-3); percentil contra a população pede a base e fica por métrica, nunca nota única ([C-7](../../produto/requisitos.md)) |
| I-52 | Títulos por estatística | falta | rejeitar · RNF-11 · a revisar | cosmético, e "The Inter" ou "Blind" é nota em forma de apelido; o perfil aponta, não rotula (RNF-11). A G-12 a listava como lacuna; saiu de lá, e o conflito ficou para o dono na [P-31](../../produto/perguntas-em-aberto.md) |
| I-53 | Elogio/crítica por IA | fora | fora · RNF-8 | texto gerado no servidor do iTero a partir das suas estatísticas: sai da máquina (RNF-8) e não aponta um número que o jogador confira (R-16) |
| I-54 | Leaderboard de amigos | regra | rejeitar · C-7 | [C-7](../../produto/requisitos.md): escada paralela |
| I-55 | Meu winrate × comunidade por campeão | parcial · sem dado | adiar · R-17, R-21 · a revisar | perfil por campeão; o da comunidade sai da base |
| I-56 | Coach de pontos de melhoria | parcial | adiar · R-16, R-17, P-3 · a revisar | uma partida por vez (`coach.rs`) e o perfil agregado; as faixas de 7 min pedem a timeline (P-3) |
| I-57 | Gráfico de PDL | falta | adiar · R novo · a revisar | a Riot não dá histórico de PDL; teria de ser gravado partida a partida, lido do LCU e guardado na máquina. Baixo valor para o draft (G-12) |
| I-58 | Várias contas | falta | adiar · R-17 · a revisar | o perfil é da conta que está no cliente; várias contas pedem guardar o histórico por conta (G-12) |
| I-71 | Busca de jogador | falta | adiar · R-17, C-10 · a revisar | o perfil é só da sua conta. Buscar qualquer conta é histórico de outro jogador pela Riot Web API, que pede chave de produção (C-10) e o backend da [ADR-0004](../../adr/0004-chave-da-riot-so-no-backend.md) |
| I-72 | Histórico de partidas | falta | copiar · R-16 · a revisar | só a última leitura fica guardada (`recorder.rs`, `ultima-partida.json`); guardar as leituras dá a lista, local (RNF-8), com filtro por fila, rota e campeão |
| I-73 | Modos de comparação no detalhe | parcial | mudar · R-16 · a revisar | as curvas do pós-jogo são fixas pela rota: ouro em itens time contra time, tropas ou visão e nível contra o oponente direto (`GraficoPartida.tsx`). A rota decide o que se compara (R-16), sem seletor; XP e dano não vêm da Live Client |
| I-74 | Filtros das estatísticas da conta | falta | adiar · R-17, P-3 · a revisar | o perfil de rota não tem filtro. A faixa de tempo e o à frente/igual/atrás pedem a timeline de cada partida (P-3); o tipo de estatística caberia hoje, mas são só cinco métricas |
| I-75 | Winrate por faixa de Draft Score | falta | mudar · R-21, RNF-1 · a revisar | o "Community Win Rate" do iTero é constante no cliente ([04 §3](04-telas-da-janela-principal.md#estatísticas-por-campeão-profilechampion-statistics)): número sem origem, que o RNF-1 proíbe. O jeito do Presage: gravar a chance do time no draft e mostrar a calibração — quantas vezes você ganhou quando ele disse X%, com a amostra —, só com a base, porque sem ela a chance do time não aparece (R-21) |

## 4. Plataforma

| Id | Funcionalidade | Presage | Decisão | Nota |
|---|---|---|---|---|
| I-59 | Conta própria e nuvem | fora | fora · RNF-8 | o Presage não tem conta nem nuvem, e nada do jogador sai da máquina (RNF-8). O backend que a [ADR-0004](../../adr/0004-chave-da-riot-so-no-backend.md) prevê para produção guarda a chave da Riot, não conta nem configuração |
| I-60 | Assinatura | fora | fora · C-8 | a [C-8](../../produto/requisitos.md) não proíbe: permite cobrar com camada grátis obrigatória e conteúdo pago transformativo. O Presage não tem plano pago; cobrar é decisão que vem depois da chave de produção (C-10) |
| I-61 | Anúncios | fora | fora · C-9 | a [C-9](../../produto/requisitos.md) proíbe anúncio dentro do jogo, overlay incluso; painel e site podem ter. O iTero põe anúncio na principal, no draft e na segunda tela da partida, nunca no overlay. O Presage não tem anúncio porque não tem monetização (I-60) |
| I-62 | Idiomas | falta | adiar · R novo · a revisar | só português (G-12) |
| I-63 | Atualização automática | tem | mudar · R-19, RNF-7 | o Presage avisa e instala quando o jogador pede ([06](../../arquitetura/06-distribuicao-e-atualizador.md)); o iTero instala calado e reabre |
| I-64 | Bandeja, autostart, instância única | parcial | copiar · R-19 · a revisar | bandeja e iniciar com o Windows, desligado por padrão (G-8); falta a trava de instância única (G-12). Reiniciar como admin não se aplica: o Presage não injeta nada no jogo |
| I-65 | Tutorial de primeiro uso | falta | adiar · R novo · a revisar | sem primeiro uso guiado; hoje a única explicação são as etiquetas de origem de cada número (RNF-1) (G-12) |
| I-76 | Aviso remoto na barra de título | falta | adiar · R-19 · a revisar | o Presage não tem servidor; um JSON estático no mesmo endereço do manifesto de atualização ([ADR-0005](../../adr/0005-atualizador-assinado-com-endpoint-permanente.md)) daria o aviso sem mandar nada do jogador (RNF-8) |
| I-77 | Pop-up de novidades | fora | fora · Summoner's Rift | o conteúdo é o ARAM (I-26); as notas de versão o Presage já mostra no aviso de atualização (`Atualizacao.tsx`) |
| I-78 | Discord e relatório de bug | falta | adiar · R novo · a revisar | não há canal com o jogador; o canal (issues do repositório, um servidor) tem de existir antes do botão |
| I-79 | Manutenção (cache, padrões, logs, desinstalar) | parcial | copiar · R-19 · a revisar | o aviso de versão checa na abertura e mostra as notas (`Atualizacao.tsx`); o log vai para a saída padrão (`tracing_subscriber`), sem arquivo, e o build de release não tem console (`main.rs`), então o jogador não tem log nenhum; não há limpar dados nem restaurar padrões. Abrir a pasta de dados e restaurar padrões cabem sem mudar regra |
| I-80 | Relatório de erro e telemetria | regra | rejeitar · RNF-8 | nada sai da máquina, telemetria inclusive ([visão geral §7](../../produto/visao-geral.md#7-como-medimos)) |
| I-81 | Canal Overwolf (beta) | fora | fora · ADR-0003 | o Presage não usa Overwolf: só APIs oficiais ([C-2](../../produto/requisitos.md)) e overlay em janela externa ([ADR-0003](../../adr/0003-overlay-como-janela-externa.md)); no iTero a troca nem é do usuário, é da tela de depuração |
| I-82 | Extras do assinante | fora | fora · C-8 | pressupõem plano pago e servidor de análise, e o Presage não tem nenhum dos dois (I-59, I-60) |

## 5. Contagem

### Por situação

| Situação | Quantas |
|---|---|
| tem | 24 |
| parcial | 16 |
| sem dado | 2 |
| falta | 17 |
| regra | 4 |
| decisão | 5 |
| sem fonte | 2 |
| fora | 13 |

Pela situação principal de cada linha (a primeira, antes do `·`); total 83.

### Por decisão

| Decisão | Quantas | A revisar |
|---|---|---|
| copiar | 24 | 6 |
| mudar | 9 | 4 |
| rejeitar | 8 | 2 |
| adiar | 29 | 26 |
| fora | 13 | 0 |

Total 83, 38 a revisar. O `conferir.py` recalcula as duas tabelas pela coluna de
cada linha e falha se divergirem, ou se alguma linha não tiver uma decisão entre
copiar, mudar, rejeitar, adiar e fora, ou ficar sem nota.

## 6. As lacunas que cabem, por valor

Só o que estava em **falta** e passa nas regras, ordenado por valor para o
jogador sobre custo. "Dado" diz se funciona hoje ou espera a base. G-1 a G-8
foram escolhidas e implementadas em 2026-10-28; o "Onde" aponta o código e o
teste que prova. Os recibos de cada uma (commit, código, teste e o T-ID dele no
[13](../../arquitetura/13-testes.md)) estão em [build/gaps.md](../../../build/gaps.md).

| # | Lacuna | Ids | Situação | Onde |
|---|---|---|---|---|
| G-1 | Corrigir à mão a rota de cada inimigo no draft | I-17 | **feita** | `context.rs` (`Leitura`), seletor no quadro do draft; `a_rota_marcada_vence_a_inferida`, `a_rota_marcada_tira_a_rota_dos_outros` |
| G-2 | Painel da sua rota ao vivo: o que a rota cobra contra o oponente direto, e atrás/igual/à frente em ouro de itens | I-36, I-45 | **feita** | `rival.rs`; `suporte_ao_vivo_e_comparado_pela_visao_e_nunca_por_tropas` |
| G-3 | Timer de inibidor e de torre do nexo | I-38 | **feita** | `timeline.rs` (`Respawn`); `inibidor_volta_em_cinco_minutos_nos_dois_formatos_de_nome` |
| G-4 | Buff de Barão e Ancião: tempo restante e quem ainda tem | I-40 | **feita** | `timeline.rs` (`BuffTracker`); `quem_morre_perde_o_buff_mesmo_depois_de_renascer` |
| G-5 | Ban pool do jogador | I-9 | **feita** | `minimax.rs` (`bans_with_pool`), seção "Seus bans" na aba Pool; `nao_bane_o_que_o_time_quer_pegar_nem_o_que_ja_saiu` |
| G-6 | Mostrar só campeões que você tem | I-12 | **feita** | `draft.rs` (`selectable`), `context.rs`; `na_ranqueada_a_rotacao_gratis_nao_vale`, `so_entra_quem_a_sua_conta_pode_escolher` |
| G-7 | Simulador de draft | I-27 | **feita** | aba Simulador, `set_simulated_draft`, `DraftState::normalized`; `draft_montado_a_mao_ganha_indisponiveis_e_rota`. Roda no app (Windows): o motor está na casca Tauri, que não compila no Kali; lá só o demo, com dados fixos |
| G-8 | Bandeja e iniciar com o Windows | I-64 | **feita** | `lib.rs` (`bandeja`, `set_autostart`); compila só no CI do Windows |

As que ficam:

| # | Lacuna | Ids | Dado | Custo | Por quê |
|---|---|---|---|---|---|
| G-9 | Estilo de jogo da rota por faixa | I-6 | base + timeline | alto | precisa estender o coletor |
| G-10 | Próxima habilidade a subir | I-44 | base + timeline | médio | idem |
| G-11 | Scouting dos dez | I-32 | chave de produção | alto | |
| G-12 | Idiomas, tutorial, várias contas, PDL, instância única | I-57, I-58, I-62, I-64, I-65 | hoje | variado | baixo valor agora. Os títulos (I-52) saíram daqui: a decisão é rejeitar pela RNF-11, e a [P-31](../../produto/perguntas-em-aberto.md) leva o conflito ao dono |

As linhas I-66 a I-83 entraram depois e não ganharam número de lacuna; o que
fazer com cada uma está na coluna Decisão.

**Decisões que não são minhas** (mudam regra registrada):

- **Escrever no LCU** (I-19 enviar runas, itens e feitiços; I-20 e I-22
  auto-hover; I-23; I-66 hover, travar e banir à mão): é o que o iTero mais faz
  no champ select, e ele diz ter sido aprovado pela Riot com isso. O Presage hoje
  só lê (R-18); a pergunta é a [P-15](../../produto/perguntas-em-aberto.md), com o
  default "LCU só leitura", e escrever pede um ADR (o que pode ser escrito,
  quando, e como desligar).
- **Auto-lock** (I-21): fica de fora pela C-5 em qualquer leitura.

## O que o Presage tem e o iTero não

O caminho inverso: o que o Presage faz e nenhuma linha do catálogo cobre, ou
cobre de um jeito que o iTero não tem. Cada linha aponta o requisito que a
garante ([requisitos](../../produto/requisitos.md)).

| Presage | R-n | Por quê |
|---|---|---|
| Etiqueta de origem em cada número, e componente sem dado valendo exatamente zero | RNF-1 | o iTero mostra a nota sem dizer de onde ela vem; o Presage diz se é medição, matemática do jogo, regra ou palpite ([visão geral §2](../../produto/visao-geral.md#2-de-onde-vem-cada-número)), e o que não tem amostra vale zero ([ADR-0007](../../adr/0007-sem-dado-o-componente-vale-zero.md)) |
| `±` que encolhe com a amostra | RNF-2 | a nota relativa 0–5 do iTero (I-2) não tem incerteza nenhuma à vista |
| Resposta adversária: o valor do pick depois da melhor resposta deles | R-2 | o iTero ranqueia pelo modelo para o draft de agora; não mostra quanto o pick fica exposto a um counter-pick |
| Prévia do candidato: rota nível a nível do 1 ao 18, 2v2 e 3v3, condição de vitória | R-6 | o iTero dá a nota da fase de rota e as estatísticas do time; não tem curva de nível nem condição de vitória em texto |
| Rota do inimigo presumida e corrigível | R-4 | o iTero não infere: manda o inimigo sem posição e deixa o modelo lidar, ou usa a marca à mão ([01 §5](01-processo-principal.md#composição-enviada-ao-backend)) |
| Pool de três escolhidos e cinco por cobertura, calculada na máquina | R-9 | o construtor do iTero é uma conversa com o servidor (`cpc_*`); o do Presage recalcula a cobertura a cada escolha, sem rede, e diz que é regra |
| Runa do confronto que diz ser regra | R-7 | `from_data = false`: a tela não escreve "segundo N partidas" quando não há N |
| Build contra os itens que eles têm agora | R-10 | a build do iTero vem do servidor por composição e troca pelo estado da rota; a do Presage reage a cada item comprado (C-6) |
| Timers de objetivo, com respawn observado e primeiro spawn estimado separados | R-13 | o iTero tem inibidor, torre do nexo e buffs; não tem timer de objetivo neutro |
| Onde está o valor no mapa | R-14 | não há equivalente no iTero |
| Perfil de rota contra o oponente direto | R-17 | o iTero compara com a média e o percentil do elo; o oponente direto é o grupo de controle que o matchmaking escolheu ([ADR-0010](../../adr/0010-oponente-direto-como-grupo-de-controle.md)) |
| Pós-jogo lido na máquina | R-16, RNF-8 | a análise do iTero roda no servidor dele e exige rede; a do Presage é a gravação local, quadro a quadro |
| Funciona sem chave e sem servidor | RNF-5 | sem o backend do iTero, o draft dele não tem lista; o Presage ranqueia, presume a rota e monta a build só com LCU, Live Client e Data Dragon. A [ADR-0004](../../adr/0004-chave-da-riot-so-no-backend.md) prevê um backend em produção, mas só para a chave da Riot |
| Nenhuma ação no cliente que o jogador não fez | R-18 | o iTero escreve no LCU (hover, lock, ban, runas, itens, feitiços); o Presage só lê |
| Atualização que nunca instala sozinha e só executa o que foi assinado | RNF-7 | o iTero instala calado e reabre |
| Nada sai da máquina, telemetria inclusive | RNF-8 | o relatório de erro do iTero leva o corpo das requisições, com puuid e Riot ID ([06 §6](06-dados-e-backend.md#6-telemetria-e-anúncios)) |
| Sem Overwolf: só APIs oficiais e overlay em janela externa | C-2, C-3 | o iTero depende do GEP para quase todo dado ao vivo e injeta o overlay no processo do jogo ([ADR-0003](../../adr/0003-overlay-como-janela-externa.md)) |
| Aponta, não dá nota | RNF-11 | o iTero dá AI Draft Score, AI Macro Score, percentil e títulos; o Presage mostra o número que o jogador confere na aba de pontuação (C-7) |
