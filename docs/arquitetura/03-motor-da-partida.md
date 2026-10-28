# 03 — Motor da partida

`core`, alimentado pela Live Client Data API a cada segundo. Nada aqui é
estatística de partidas: é a matemática do próprio jogo sobre o que está na aba
de pontuação, e por isso tudo funciona sem chave e sem base
([visão geral §3](../produto/visao-geral.md#3-o-que-já-roda-sem-dado-nenhum)).

## 1. Da Live Client ao domínio

`to_live_game` (`core/src/live.rs`) traduz `RawPlayer`, `RawItem` e
`RawScores` para `LiveGame`. `analysis.rs`, na casca, monta os dois lados da
partida e chama `engine::report`, a mesma montagem que o draft usa a partir do
`DraftContext`.

## 2. Build adaptativa

A parte do produto que **não é estatística**. Dado o item que o inimigo comprou,
o resultado é exato: não há amostra, prior nem intervalo de confiança.

Para cada item candidato a pergunta é sempre a mesma: quanto ele sobe o seu
poder de combate, por ouro, **contra os itens que eles têm agora** (`suggest`).

Poder é vida efetiva vezes dano efetivo (`side_power`). O produto é o que importa
— dobrar o dano e dobrar a vida valem o mesmo numa luta — e é o que deixa item
ofensivo e defensivo competirem na mesma régua, sem peso arbitrário entre os
dois.

O que o motor reage, com teste para cada caso:

- time inimigo AD puxa armadura acima de resistência mágica;
- o mago deles comprar poder de habilidade inverte a prioridade;
- penetração de armadura no inventário deles derruba o valor da sua armadura;
- contra letalidade, vida passa a render mais que armadura;
- o inimigo comprar armadura faz penetração subir na sua lista.

### De onde vêm os números

Stats base de campeão saem do Data Dragon, com a fórmula real de crescimento por
nível (`grown`):

```text
valor(n) = base + ganho * n * (0.7025 + 0.0175 * n),  n = nível - 1
```

Não é linear. No nível 18 o fator fecha em exatamente 17, o que serve de teste da
própria fórmula.

Atributos de item são o ponto delicado: o bloco `stats` do Data Dragon é legado e
não traz penetração, letalidade nem haste, então parseamos a `description`
(`parse_stats`), sempre em en_US ([ADR-0015](../adr/0015-atributos-de-item-pela-descricao.md)).

### Limites honestos

O lado ofensivo é um **proxy, não DPS**: soma dano de ataque e poder de
habilidade passados pelas resistências reais do alvo, sem razões de habilidade,
recarga ou crítico. Serve para ordenar itens, não para prever o dano de um combo.

Também não trata hoje: caminhos de construção (sugere o item final, não o
componente), itens únicos por categoria, botas, e passivas de item, que não estão
em nenhum dado estruturado.

## 3. Dano

Duas regras do jogo que o motor respeita (`damage.rs`) e que calculadora amadora
erra:

- **Ordem de penetração** (`effective_resist`): redução plana, redução
  percentual, penetração percentual, penetração plana. Com 100 de armadura, 35%
  de pen e 18 de letalidade dá 47 — na ordem trocada daria 53,3.
- **Penetração percentual é multiplicativa**: dois itens de 35% dão 57,75%, não
  70%.

E o multiplicador `100 / (100 + resistência)` (`damage_multiplier`), com a vida
efetiva saindo dele (`effective_hp`).

## 4. Chance por rota e escaramuça

`lane_outlooks` responde "você contra o seu oponente direto, os dois sozinhos";
`skirmishes`, "um grupo contra o outro, quando a briga acontece". Mesma base: a
matemática do duelo com resistências, dano e penetração reais dos dois lados
(`skirmish_odds`).

```text
chance da rota = 0,5 + W_DUEL × (duelo − 0,5) + Δmatchup     cortada em [0,28; 0,72]
```

- `W_DUEL = 0,60`: poder de combate não é vitória de rota — quem ganha o 1v1 em
  tela aberta ainda perde a rota por onda, por gank e por nível. Com 0,60 um
  duelo de 60/40 vira 56% de rota. Escolhido, não ajustado ([P-7](../produto/perguntas-em-aberto.md)).
- `Δmatchup` vem da base, encolhido para o que as duas forças já explicam
  (`expected_against`, a mesma referência do draft) com a mesma força de matchup.
  Sem base é exatamente zero, e `from_data = false` faz a interface escrever "só a
  conta do duelo"; com base, a força de cada um já entra no número e `from_data`
  fica verdadeiro.
- Nenhuma rota está decidida antes de começar: daí o piso e o teto.
- Rotas sem oponente identificado ficam de fora em vez de virarem 50%: um palpite
  com cara de medida é pior que silêncio.

### Curva por nível e vantagem de nível

Três leituras da mesma conta de duelo (`LaneOutlook`):

- `by_level`: os níveis 3, 6, 11 e 16 (`BREAKPOINTS`), com os dois nivelados — os
  quatro números compactos da linha.
- `curve`: do 1 ao 18, nivelados. Entre um nível e o seguinte a conta quase não
  muda: vale a forma da curva, quem cresce mais, não um pico num nível qualquer.
- `gaps`: no 3, 6, 11 e 16, você um nível à frente, nivelado e um atrás (`gap` ±1).
  No 6, 11 e 16 o nível a mais é justamente a suprema, que a conta não vê — a
  tabela diz "atributos, sem a suprema" nessas linhas.

Na partida, `duel`, `my_level` e `their_level` são o duelo nos níveis reais, e a
curva marca esse ponto: a vantagem ou o atraso de nível aparece fora da linha
nivelada.

**O que esta curva vê:** crescimento de vida, armadura, resistência mágica e dano
de ataque por nível, que diferem bastante entre campeões, mais os itens que cada
um já tem.

**O que ela não vê:** habilidades. O modelo não sabe que alguém ganhou a suprema
no 6, nem quanto ela bate. Então a curva mostra a diferença de *atributos* ao
longo do jogo — real, mas não é o powerspike de habilidade que a palavra costuma
sugerir. A interface diz isso.

### Condição de vitória

`engine/wincon.rs` lê os dois times e os duelos já calculados
([ADR-0018](../adr/0018-o-que-a-condicao-de-vitoria-pode-afirmar.md)):

- **fatos de composição, por regra:** dano de um tipo só (≥70%), ninguém na linha
  de frente, time de alcance ou corpo a corpo. Time com menos de três definidos
  fica sem fatos;
- **onde a conta de atributos pende:** rotas e escaramuças a mais de 5 pontos de
  50%, cada uma marcada como `atributos`.

**A fase do jogo só pelos dados.** A conta de atributos não vê habilidade e por
isso não vê quem escala: "seu time é de fim de jogo" a partir dela poderia sair ao
contrário. Com a base, a fase sai da tabela `length`: a winrate de cada um, na rota
dele, em partidas longas contra curtas, encolhida para a força dele na rota. A
diferença entre os dois times vira uma frase com o selo `dados` e o n. Sem a base,
não há linha de fase. Tudo é descrição, nunca ordem (C-5).

### A prévia no draft

No draft, clicar num candidato chama `lane_analysis` com ele no seu lugar
(`DraftContext::with_candidate`): a rota contra o oponente, as escaramuças de que
ele participa e a condição de vitória — a mesma conta de quando ele estiver
travado (`a_previa_e_a_mesma_conta_de_quando_ele_estiver_travado`).

## 5. Ouro

`gold.rs` soma o preço de cada item no inventário dos dez: **investido** é exato.
O **ganho** é estimado (`earned_at`) a partir de tempo, CS, abates, assistências
e o investido. Ouro em mão de outro jogador nunca aparece (C-4).

## 6. Timers

Dois tipos de número, e a diferença aparece na tela (`timeline.rs`):

- **Observado**: vimos o abate no fluxo de eventos, então o respawn é uma soma
  exata.
- **Estimado**: ninguém matou ainda, e o primeiro spawn vem de uma constante.

As constantes vivem num único `Timings` e **mudam a cada temporada** — na 16 o
Barão voltou de 25:00 para 20:00, o Arauto virou único às 15:00 e o Atakhan foi
removido. A lógica é testada contra valores arbitrários, então trocar uma
constante não quebra nada além dela mesma.

| Constante | Valor |
|---|---|
| primeiro dragão / respawn | 5:00 / 5:00 |
| respawn do Ancião | 6:00 |
| primeiro Barão / respawn | 20:00 / 6:00 |
| Arauto (único) | 15:00 |
| Grubs (únicos) | 8:00 |
| dragões para a alma | 4 |
| respawn do inibidor | 5:00 |
| respawn da torre do nexo | 3:00 |
| buff do Barão / do Ancião | 3:00 / 2:30 |

**Estruturas** (`Respawn`): `InhibKilled` e `TurretKilled` de torre do nexo viram
contagem até a volta, com o lado dono da estrutura. Há dois formatos de nome no
mundo — `Inhib_TOrder_L2`/`Turret_TOrder_L1_P5`, o que apps em produção leem
hoje, e `Barracks_T1_L1`/`Turret_T1_C_01_A`, o da documentação da Riot — e os
dois são aceitos; nome desconhecido não vira timer
([P-14](../produto/perguntas-em-aberto.md)).

**Buffs** (`BuffTracker`): o buff de Barão e de Ancião é de quem estava vivo no
abate, e some de quem morre. Saber quem ainda tem pede lembrar da leitura
anterior, então este é o único timer com estado: a casca guarda o rastreador
durante a partida e o zera fora dela. Quem morre perde o buff mesmo que renasça
antes da leitura seguinte (o `ChampionKill` conta); abate visto mais de 5 s
depois de acontecer mostra o tempo sem inventar quem tem; o buff some quando
acaba ou quando ninguém mais o tem.

Nenhum timer de habilidade inimiga, nem manual (C-1). Recarga de suprema, nem de
aliado: a Live Client não entrega, e o que entrega isso (o GEP do Overwolf) não
é API oficial (C-2).

## 6.1 Sua rota ao vivo

`rival.rs`: você contra o oponente direto, pela posição que a Live Client
informa para os dez. A rota decide a métrica pela mesma `cobranca` do pós-jogo
(§8): tropas por minuto para quem farma, **placar de visão por minuto para o
suporte, nunca tropas**. Atrás, igual ou à frente sai do ouro em itens (exato),
com a margem `max(300, 10% do que ele investiu)` — regra, não estatística.
Antes de 1:30 não há painel: a taxa por minuto ainda é ruído.

## 7. Prioridade

A API da Riot não entrega posição de campeão no mapa, e nenhuma via legal
entrega. Então o Presage **não diz onde o inimigo está** — diz onde está o
valor (`priority::report`). A formulação importa tanto quanto a conta: a saída é
sempre "esta rota vale mais agora, por estes motivos", e há um teste que varre as
razões geradas procurando linguagem de posição (C-4).

O que alimenta: quem está morto e por quanto tempo, CS e nível de cada um, os
itens dos dez, e os eventos de objetivo. Por cima roda a simulação de escaramuça
do motor de dano, com as resistências e a penetração reais — o mesmo motor da
build adaptativa. `focus` escolhe a escaramuça que importa para a sua rota diante
dos próximos objetivos.

## 8. Leitura pós-partida

Não depende da chave da Riot. Enquanto o jogo roda, `recorder.rs` grava um quadro
por segundo da Live Client Data API em `gravacao.json`; no fim, `coach::review`
lê a gravação e devolve o que foi bem e o que foi mal (`Report`, salvo em
`ultima-partida.json`).

Cada observação precisa ser verificável pela própria pessoa: um número que ela
poderia ter conferido na aba de pontuação. Nada de "jogue mais seguro". E nada
vira nota (C-7). A gravação fica na máquina do jogador; nada é enviado (RNF-8).

**A rota decide o que se cobra** (`cobranca`, escrito à mão — regra):

| Rota | Farm contra o oponente | Visão contra o oponente | Participação fraca / forte |
|---|---|---|---|
| Topo | sim | — | ≤30% / ≥60% |
| Selva | sim (campos contra campos) | — | ≤45% / ≥70% |
| Meio, Atirador | sim | — | ≤40% / ≥65% |
| Suporte | **não** | **sim** | ≤50% / ≥75% |
| Sem rota | — | — | ≤40% / ≥65% |

Cobrar tropas de um suporte contra o outro é mandar fazer o trabalho de outra
pessoa ([L-27](09-licoes-do-prototipo.md)). O suporte é cobrado pelo placar de
visão por minuto contra o outro suporte, e o gráfico do pós-jogo mostra a curva
de visão no lugar da de tropas.

O perfil de rota ([04](04-perfil-de-rota.md)) é o par desta leitura: uma partida
aqui, o histórico inteiro lá.

## 9. Runas

`runes::plan` sugere a runa principal para o confronto da sua rota a partir de
regras mecânicas — quem alcança quem, quem aguenta troca longa, quem morre de
explosão. **Não sai de winrate**, e `from_data` é `false` para a tela não
escrever "segundo N partidas".

As runas são referenciadas pela **chave textual** do Data Dragon (`Conqueror`,
`GraspOfTheUndying`), nunca pelo id numérico. Id muda de patch e vira ícone
errado em silêncio; a chave é estável, e uma chave que suma simplesmente deixa de
ser sugerida.

## 10. Overlay

Janela externa, transparente, sempre no topo. **Não há injeção em nada**: o
Vanguard bloqueia hook de render, e um overlay que dependesse disso estaria morto
antes de nascer ([ADR-0003](../adr/0003-overlay-como-janela-externa.md)).

| Atalho | O quê |
|---|---|
| `Ctrl+Shift+P` | mostra ou esconde |
| `Ctrl+Shift+O` | torna clicável (por padrão o mouse atravessa) |

Duas consequências de ser janela externa, e não dá para contornar: **o jogo
precisa rodar em janela sem bordas** — em tela cheia exclusiva nenhuma janela
externa aparece por cima — e o overlay nasce atravessável pelo mouse, senão
comeria cliques no meio da partida.

O que aparece nele o jogador escolhe (`OverlayPrefs`, `overlay.json`). Anúncio
nunca (C-9). Se o overlay é permitido com valor calculado ao vivo é a pergunta
[P-2](../produto/perguntas-em-aberto.md).
