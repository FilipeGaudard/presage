# Visão geral

## 1. O que é

O Presage acompanha o jogador em três momentos e fala só do que já está na
tela dele ([C-4](requisitos.md#1-regras-da-riot-c)):

| Momento | O que entrega | De onde lê |
|---|---|---|
| **Draft** | campeões da sua rota ranqueados conforme o draft acontece, com motivo em texto, intervalo de confiança e a resposta adversária (minimax); para cada candidato, a prévia: rota nível a nível, 2v2 e 3v3, condição de vitória; sugestão de ban; runas do confronto | LCU, Data Dragon, base de partidas quando existir |
| **Partida** | build adaptativa contra os itens que eles têm agora, chance por rota e escaramuça, ouro dos dez, timers de objetivo, onde está o valor no mapa | Live Client Data API |
| **Pós-jogo** | leitura da partida gravada; perfil de rota contra o oponente direto ao longo das suas partidas | gravação local, Riot Web API |

E fora deles, o **construtor de pool**: por rota, o jogador escolhe três
campeões, o Presage aponta os pontos fortes e fracos deles e recomenda, um lugar
por vez, os cinco que completam a pool de oito cobrindo o que faltava. No draft,
a pool aparece ranqueada antes do resto
([02 §7](../arquitetura/02-motor-de-draft.md#7-pool)).

### O diferencial

**Não depende da Riot Web API.** Build adaptativa, calculadora de dano, timers,
ouro por time, chance por rota e prioridade saem todos da Live Client Data API,
que é local e sem chave. O que precisa de chave — histórico pessoal e base
agregada — é a camada que entra por último de qualquer jeito.

**Honestidade sobre o número.** Ferramentas do gênero mostram `52,3%` sem dizer
de onde. O Presage diz qual número é medição, qual é matemática do jogo e qual é
palpite, e um componente sem dado vale exatamente zero
([ADR-0007](../adr/0007-sem-dado-o-componente-vale-zero.md)).

**Varia com a partida.** A build responde aos itens que o inimigo comprou, não à
build mais popular do patch ([C-6](requisitos.md#1-regras-da-riot-c)).

**Você contra o seu nível.** O perfil de rota compara cada partida com o
oponente direto, que o matchmaking escolheu para ter o seu nível
([ADR-0010](../adr/0010-oponente-direto-como-grupo-de-controle.md)).

### O que não é

Cada linha é uma regra da Riot ([requisitos](requisitos.md)), não uma falta de
tempo:

- não diz onde o inimigo está, nem cooldown, nem ouro em mão alheia;
- não mostra timer de ultimate inimiga, nem manual;
- não lê memória do jogo, não injeta nada, não automatiza input;
- não manda num pick só: ranqueia com alternativas e motivo;
- não dá nota, rating nem escada paralela ao ranqueado.

## 2. De onde vem cada número

A pergunta mais importante do projeto, porque tudo que o app mostra é número. A
resposta honesta é que eles **não têm todos a mesma qualidade**.

| O que aparece | De onde sai | Confiança |
|---|---|---|
| Ouro em itens, CS, nível, mortes, timers | medição direta da partida | **exata** |
| Valor de item por 1000 de ouro | fórmulas de resistência e penetração do jogo | **alta na ordem**, índice no valor |
| Divisão do dano que vem (`81% físico`) | itens que eles compraram | alta depois dos primeiros itens |
| A mesma, antes de eles comprarem | prior de classe escrito à mão | **fraca** |
| Chance por rota e escaramuça | razão de poder de combate × peso escolhido à mão | **fraca como probabilidade** |
| Curva por nível (1 a 18) e vantagem de nível | crescimento de atributos por nível | real, mas **não vê habilidades** — nem a suprema no 6, 11 e 16 |
| Condição de vitória | composição por regra + duelos pela conta de atributos | **regra**; sem fase do jogo até a base existir |
| Runa do confronto | regras mecânicas escritas à mão | **regra, não estatística** |
| Recomendação da pool (lugares 4 a 8) | composição da pool + resistências, pesos à mão por rota | **regra, não estatística** |
| Score de draft (`+3.4`) | encaixe de composição, pesos à mão | **fraca** |
| Perfil de rota (`+6 tropas aos 10`) | suas partidas contra o oponente direto, encolhidas por nível | **medição**, com `±`; mistura você e o campeão |
| Winrate, matchup, sinergia | — | **não existe ainda** |

### O que é medição

`gold.rs` soma o preço de cada item no inventário dos dez. Isso é exato, não
estimativa — o preço vem do Data Dragon e o inventário vem da API oficial. CS,
nível, abates e mortes idem. A leitura pós-partida é construída inteira sobre
esses números, e por isso é a parte mais confiável do produto.

O perfil de rota também é medição: a diferença para o oponente direto em cada
partida. O que ele não separa é o quanto daquilo é você e o quanto é o campeão
([04 §6](../arquitetura/04-perfil-de-rota.md#6-o-que-ele-não-separa)).

### O que é matemática do jogo

`damage.rs` implementa a ordem de penetração (redução plana → redução
percentual → penetração percentual → penetração plana) e o multiplicador
`100 / (100 + resistência)`. `champion.rs` implementa o crescimento por nível
`base + ganho × n × (0,7025 + 0,0175n)`. Nada disso é estatística: é a
aritmética do próprio jogo, e um teste trava a ordem porque inverter dois passos
muda 47 para 53.

A ressalva: `offense_against` é um **proxy** de saída de dano — `AD × mult +
AP × mult`. Não simula rotações de habilidade, velocidade de ataque, crítico nem
cooldown. Por isso a *ordem* dos itens é confiável e o `6.8` é um índice
relativo, não um ganho medido de DPS.

### O que é regra escrita à mão

As runas e a recomendação da pool saem de regras sobre a mecânica do confronto,
não de frequência de vitória. Uma regra acerta o caso comum e erra o campeão
excêntrico. A interface diz isso em texto, e `from_data` no resultado é `false`
justamente para a tela não escrever "segundo N partidas" quando não há N.

### O que ainda não existe

Winrate base, matchup e sinergia. O encaixe está pronto
(`engine/dataset.rs`, `tools/ingest`), os dados não. Enquanto isso, `shrunk_delta`
sem amostra devolve **exatamente zero** — por construção, não por acidente.
Nenhum número inventado entra no lugar.

### O ponto fraco a corrigir primeiro

A chance por rota é `0,5 + 0,60 × (duelo − 0,5) + matchup`. Hoje o termo de
matchup é zero, e o `0,60` é um peso que foi escolhido, não ajustado contra dado
nenhum. Então aquele `47%` é uma comparação de poder de combate vestida de
probabilidade. Ela ordena bem — quem está à frente aparece à frente — mas o
número em si não é uma frequência de vitória, e só vira uma quando a base de
partidas entrar na conta ([P-7](perguntas-em-aberto.md)).

## 3. O que já roda sem dado nenhum

Tudo que sai da matemática do jogo, e não de estatística:

| Função | De onde vem |
|---|---|
| Build adaptativa | itens deles + ordem de penetração |
| Chance por rota | duelo resolvido com resistências e dano reais |
| Escaramuça 2v2 / 3v3 | mesma conta, somada por grupo |
| Ouro em itens | preço de cada item, soma exata |
| Timers de objetivo | eventos da partida |
| Leitura pós-partida | gravação local, quadro a quadro |
| Ranking de draft | só composição, com `±` largo porque de fato não sabemos |

Com a chave da Riot acendem o componente pessoal do ranking e o perfil de rota.
Com a base de partidas, winrate base, matchup e sinergia.

## 4. Roteiro

| # | Etapa | Estado |
|---|---|---|
| 1 | Fase 2 in-game, sobre a Live Client Data API | feita (0.7.0) |
| 2 | Overlay e o resto da casca | feita (0.7.0) |
| 3 | Referência e força do shrinkage ([ADR-0008](../adr/0008-matchup-e-sinergia-contra-a-soma-das-forcas.md), [ADR-0009](../adr/0009-forca-do-shrinkage-derivada-do-desvio.md)) | feita |
| 4 | Perfil de rota ([04](../arquitetura/04-perfil-de-rota.md)) | feita; compila no Windows, nomes de campo não conferidos ([P-5](perguntas-em-aberto.md)) |
| 5 | Coletor com chave Personal e o teste do limite por região | espera chave ([P-1](perguntas-em-aberto.md), [P-6](perguntas-em-aberto.md)) |
| 6 | Chave de produção, perguntando à Riot sobre o overlay no mesmo pedido | espera produto rodando ([P-2](perguntas-em-aberto.md)) |
| 7 | Com a base: variância de matchup (destrava exposição a counter-pick de verdade), ban personalizado por pool, detector de fraqueza pessoal | — |

A ordem não é arbitrária. As etapas 1 e 2 são o diferencial e não precisam de
chave; a chave de produção leva de duas semanas a mais de um mês para sair e
exige produto funcionando; e a base só existe depois dela.
