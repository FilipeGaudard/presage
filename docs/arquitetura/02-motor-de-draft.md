# 02 — Motor de draft

`core`. Entrada: o `DraftState` que o LCU dá, o `ChampionIndex` do Data Dragon, a
loja de estatísticas e a pool. Saída: campeões ranqueados com probabilidade,
intervalo e motivo.

## 0. O contexto do draft

Ranking, minimax e análise de rota leem o champ select de um jeito só
(`DraftContext`, [ADR-0017](../adr/0017-uma-leitura-do-champ-select.md)):

- **Sua rota:** a do cliente; em blind e normal, a escolhida no seletor do painel
  (`set_draft_role`); sem escolha, a da pool, se ela for de uma rota só
  (`resolve_role`).
- **Candidatos:** disponíveis, e que jogam a sua rota (`roles::plays`) **ou** estão
  na sua pool. A pool é escolha do jogador e passa por cima da regra de rota.
  Rota desconhecida não filtra.
- **Quem joga a rota** (`engine/roles.rs`): com a base, quem tem nela 5% das
  próprias partidas; sem ela, as classes do Data Dragon (suporte exige a classe
  Suporte); e sempre quem o próprio jogador já jogou ali (`played_in`).
- **Rota do inimigo:** o cliente não informa na ranqueada. Ela é inferida pela
  atribuição mais plausível das rotas livres (`role_fit`, no máximo 120
  combinações); empate fica com a ordem de escolha. Posição informada nunca é
  contestada, e `roles_assumed` avisa a interface ([P-13](../produto/perguntas-em-aberto.md)).
- **Indisponíveis:** banidos, escolhidos e em hover de aliado. A sua escolha ainda
  não travada **não**: o cliente preenche o seu `championId` antes de travar
  ([P-12](../produto/perguntas-em-aberto.md)).

## 1. `rank()`

`rank()` devolve, para cada candidato do contexto, uma probabilidade de vitória
com intervalo de confiança. Ela sai da soma de cinco componentes:

| Componente | Estado | Fonte |
|---|---|---|
| Winrate base do campeão na rota | **sem dado** | base de partidas |
| Matchup contra o laner inimigo | **sem dado** | base de partidas |
| Sinergia com os picks aliados | **sem dado** | base de partidas |
| Seu histórico pessoal | ativo com a chave da Riot | sua conta |
| Fit de composição | **ativo** | Data Dragon |

```text
winrate = base + Δmatchup + Σ Δsinergia + Δpessoal + Δcomposição     cortada em [0,25; 0,75]
```

A soma é em probabilidade, não em log-odds ([P-10](../produto/perguntas-em-aberto.md)).
A ordem é por `winrate`, depois pool, depois id — determinística, e há teste.

O que `Scored` leva para a tela:

| Campo | O quê |
|---|---|
| `winrate`, `score` | a probabilidade, e ela em porcentagem |
| `delta` | desvio em pontos percentuais contra uma partida neutra: o número embaixo do retrato |
| `confidence` | `1,96 × desvio`, em pontos percentuais: o `±` |
| `sample` | partidas por trás da winrate base |
| `in_pool` | o campeão está na pool do jogador |
| `reasons` | até três motivos em texto |

Ligar a conta acende **um** componente, o pessoal. Os outros três não saem de uma
conta sozinha: winrate base, matchup e sinergia são médias sobre milhões de
partidas de milhares de jogadores. Com 100 partidas suas você teria menos de uma
observação por matchup — por isso `PersonalStats` devolve `None` nos três, em vez
de fingir cobertura.

## 2. Shrinkage

Cada componente estatístico passa por `shrink()`, média posterior de uma Beta.
Sem amostra o componente devolve exatamente o prior e **não mexe no ranking**;
com amostra grande converge para a taxa observada. É isso que deixa o motor
honesto enquanto a base não existe: hoje a ordenação vem inteira da composição, e
os intervalos são largos porque de fato não sabemos
([ADR-0007](../adr/0007-sem-dado-o-componente-vale-zero.md)).

`shrunk_delta` é a mesma conta expressa como desvio da referência: sem amostra,
exatamente zero.

### Força e desvio

A força de cada Beta sai do espalhamento do efeito na população
([ADR-0009](../adr/0009-forca-do-shrinkage-derivada-do-desvio.md)). Uma Beta de
força `s` em torno de 50% tem desvio `√(0,25 / (s + 1))`, então:

```text
strength_for_sd(sd) = 0,25 / sd² − 1
```

| Constante | Espalhamento | Força derivada |
|---|---|---|
| `MATCHUP_PRIOR_SD` | 3,0pp | ≈ 277 |
| `SYNERGY_PRIOR_SD` | 1,2pp | ≈ 1735 |
| `PERSONAL_PRIOR_SD` | 3,5pp | ≈ 203 |
| `BASE_STRENGTH` | — | 400 (o `±` da base sai de `Estimate::se`, já coerente com ela) |
| `COMP_SD` | 1,0pp | — |

Os espalhamentos são escolhidos à mão. Quando a base existir, saem dela por Bayes
empírico: `τ² = Var(taxas observadas) − média(p(1 − p) / n)`.

### Incerteza

A incerteza de cada componente não é zero quando falta dado — é o espalhamento
típico daquele efeito na população, encolhendo conforme a amostra cresce:

```text
posterior_sd(n, sd) = √(0,25 / (strength_for_sd(sd) + 1 + n))
```

Sem amostra é exatamente `sd`; depois cai com a raiz da amostra, como na Beta. Por
isso o `±` na tela fica em torno de 11pp sem base estatística. A variância total
soma a da base, a da composição e a de cada componente.

## 3. A referência

Matchup e sinergia são medidos contra o que as duas forças já explicam, somadas
em log-odds — e não contra a base do candidato
([ADR-0008](../adr/0008-matchup-e-sinergia-contra-a-soma-das-forcas.md)):

```text
expected_against(você, rival)   = σ(logit p_você − logit p_rival)
expected_together(você, aliado) = σ(logit p_você + logit p_aliado)
```

A taxa crua de A contra B carrega a fraqueza geral de B; medida contra a base de
A, essa fraqueza vira "matchup" na proporção da amostra, e o pick popular sobe
por ser popular. Com o rival em 47%, 5000 partidas sem nada de especial no
confronto valiam +2,6pp; 100 partidas, +0,3pp — a diferença tem o tamanho do
próprio efeito que se quer medir. A base de cada aliado é lida na rota dele; rota
desconhecida cai em 50%, e a sinergia volta a ser medida só contra a sua base.

Testes: `popularidade_nao_vira_matchup`, `sinergia_nao_herda_a_forca_do_aliado`.

## 4. Fit de composição

Dois sinais, ambos derivados do Data Dragon (`comp.rs`):

- **Equilíbrio de dano** (`damage_balance_gain`). O time fica mais ou menos
  unilateral com este pick? Um time 100% físico deixa o inimigo comprar só
  armadura.
- **Frontline** (`frontline_need`). O time tem alguém que segura a linha de
  frente?

Os dois são ponderados pelo quanto do draft já aconteceu: um sinal lido com um
pick travado vale menos que o mesmo sinal com quatro.

A fração de dano físico de cada campeão mistura dois sinais fracos — a razão
`attack`/`magic` do campo `info`, que é grosseira, e um prior pela classe, que
acerta melhor os extremos. Sai de cena quando o coletor entregar o split real,
que vem de `physicalDamageDealtToChampions` contra `magicDamageDealtToChampions`.

Os pesos (`W_DAMAGE_BALANCE = 0,050`, `W_FRONTLINE = 0,016`) são escolhidos à
mão, calibrados para que uma correção forte valha ~1,5pp. São priors, não
coeficientes ajustados. O `minimax.rs` tem os seus próprios (`W_BALANCE = 0,06`,
`W_FRONTLINE = 0,025`) para o valor de um time inteiro
([P-11](../produto/perguntas-em-aberto.md)).

## 5. Componente pessoal e razões

O componente pessoal encolhe o seu registro no campeão para a base dele, com a
força do `PERSONAL_PRIOR_SD`: 15V-5D em 20 partidas move ~2pp
(`historico_pessoal_curto_nao_domina`). Ele carrega a fase da conta, e isso fica
de propósito ([ADR-0012](../adr/0012-vazamento-da-fase-da-conta-mantido.md)).

As razões, na ordem em que entram (`build_reasons`, no máximo três):

1. matchup com amostra e mais de 0,5pp: "Vence Zed na rota (+4.8pp, n=5000)";
2. equilíbrio de dano: "Equilibra o dano: 74% → 59% físico";
3. frontline: "Entra como a frontline que o time não tem";
4. histórico pessoal com cinco partidas ou mais: "Seus jogos: 15V-5D (+2.2pp)";
5. perfil de rota, só a leitura mais clara: "Sua rota com ele: +6 tropas aos 10
   contra o oponente (n=22)" — razão, nunca peso
   ([ADR-0011](../adr/0011-perfil-de-rota-e-razao-nao-peso.md));
6. nada disso e sem base: "Sem base estatística ainda — ordenado só por
   composição".

## 6. Draft com resposta

O ranking guloso responde "qual o melhor pick agora", que não é a pergunta do
draft: depois de você travar, eles escolhem contra você. `minimax` olha uma
camada à frente e ordena pelo valor **depois** da melhor resposta deles. A
diferença entre os dois números é a **exposição** do pick.

Os candidatos são os do contexto, podados pelo guloso (`breadth`). As respostas
deles são **todas** as que jogam uma rota que eles ainda não fecharam — varrer
tudo é barato, porque `team_value` é aritmética pura. Antes a busca olhava só os
32 primeiros ids, e o "melhor contra-pick" saía sempre dos campeões de id baixo
([L-24](09-licoes-do-prototipo.md)).

### Por que o ban ainda não é pessoal

`team_value` é **aditiva**: a composição deles e a sua não interagem. Logo
"quanto o campeão X bate no seu pool" é, na álgebra, idêntico a "quanto X ajuda
eles" — o pool não muda nada na conta. Separar os dois exige o termo de
interação, que é exatamente a tabela de matchup que a base vai trazer
([ADR-0013](../adr/0013-ban-nao-personalizado-enquanto-aditivo.md)).

Até lá `suggest_bans` ordena por quanto o campeão completa a composição deles, e
a função não finge que faz mais. Há um teste travando essa limitação; quando o
matchup entrar, ele deve falhar — e aí o ban vira pessoal.

O ban também **devolve vazio enquanto o outro lado não travou nada**. Sem pick e
sem base estatística todo campeão pontua igual, e ordenar isso seria apresentar
ruído como conselho.

## 7. Pool

A pool é por rota e tem oito: **três que o jogador escolhe** (core) e **cinco
que o Presage recomenda** para cobrir o que os três não cobrem (situacionais)
([ADR-0016](../adr/0016-pool-de-oito-tres-escolhidos-cinco-por-cobertura.md)). Nada
trava: na revisão o jogador põe e tira quem quiser, e um campeão sem rota vale em
todas. A pool fica em `%LOCALAPPDATA%\Presage\pool.json`.

No draft, as recomendações saem em duas seções: primeiro a pool ranqueada entre
si, depois os melhores no geral. A pool marca o campeão (`in_pool`) mas **não
mexe na ordem** (`pool_e_marcado_sem_alterar_a_ordem`).

### Os três e o retrato deles

`poolgap::analyze` lê os três core em características que saem do Data Dragon
(`Trait`): dano físico, dano mágico, linha de frente, alcance, corpo a corpo,
resistências equilibradas, e os seis estilos (as classes). **Ponto forte** é o
que a rota valoriza e ao menos um deles tem; **ponto fraco** é o que a rota
valoriza e nenhum tem — ou os três terem o mesmo estilo, porque aí a mesma
resposta serve contra todos.

O que a rota valoriza vem de pesos escritos à mão (`weight`): todo atirador tem
alcance e nenhum segura a frente, então nada disso conta para a pool de bot.

### Os cinco, um lugar por vez

`poolgap::suggest` recomenda o próximo lugar, e o front chama de novo a cada
campeão aceito. A cobertura de cada característica é `1 − ½ⁿ`, com n quantos da
pool a têm; o candidato vale o quanto sobe a cobertura:

```text
ganho = Σ peso(rota, característica) × ½ⁿ⁺¹     sobre o que o candidato tem
```

A primeira ocorrência de algo vale metade do peso, a segunda um quarto. Então o
quarto lugar tapa o buraco maior, o quinto o que sobrou, e assim por diante —
recalcular a cada escolha é o que impede cinco respostas ao mesmo buraco
(`cada_escolha_muda_a_proxima`). O `score` é o ganho dividido pelo maior ganho
possível: índice de encaixe de 0 a 1, não winrate. Com oito, a pool está completa
e não há mais sugestão.

O jogador escolhe entre as alternativas de cada lugar; nada se preenche sozinho
(C-5).

### Quem joga a rota

Sem a base, a rota sai das classes do Data Dragon (`plays`): topo é lutador ou
tanque, selva lutador, tanque ou assassino, meio mago ou assassino, bot atirador,
suporte suporte ou tanque. É grosseiro — classe não é rota, e o topo de alcance
fica de fora —, mas é o que impede sugerir tanque para a pool de atirador
([L-22](09-licoes-do-prototipo.md)). Com a base, joga a rota quem tem nela ao
menos 5% das próprias partidas. A escolha manual não passa por aqui.

### O que não é

Daqui não sai winrate. Sai a composição da pool e a matemática de resistências:
um campeão "robusto" (`robustness`) é um campeão cuja vida efetiva varia pouco
conforme o dano que vem contra ele — aritmética sobre armadura e resistência
mágica no nível 11. É **regra, não estatística**, e a interface diz isso. O que
esta conta não vê — contra quem os seus três perdem, e quem ganha desses — é
tabela de matchup, e entra com a base ([05 §8](05-coletor.md#8-o-que-a-base-destrava)).

## 8. A base de partidas

O encaixe está pronto: `engine/dataset.rs` define o formato, implementa
`StatsStore` e `Layered` junta a base agregada com o histórico da conta. O app
procura `%LOCALAPPDATA%\Presage\base-de-partidas.json` ao subir; não achando,
segue sem — e `has_global` fica `false`, que é o que a interface mostra.

O formato é um JSON com cabeçalho e três mapas:

```json
{
  "patch": "16.21",
  "region": "BR1",
  "tier": "EMERALD+",
  "generatedAt": "2026-10-27T00:00:00Z",
  "matches": 1200000,
  "base":    { "103:Mid": { "games": 40000, "wins": 21000 } },
  "matchup": { "103:238:Mid": { "games": 1840, "wins": 1050 } },
  "synergy": { "64:103": { "games": 900, "wins": 480 } }
}
```

A chave de sinergia guarda sempre o menor id primeiro, para Lee+Ahri e Ahri+Lee
caírem na mesma entrada. `is_usable()` recusa arquivo truncado: sem `matches` ou
sem `base`, a camada global fica desligada em vez de recomendar a partir de meia
dúzia de linhas.

`Layered` mantém as duas camadas independentes de propósito: ter a conta ligada
não produz winrate de matchup, e ter a base não diz nada sobre como *você* joga.
Quem produz o arquivo é o coletor ([05](05-coletor.md)).
