# Regras e pesos

Cada regra e cada peso escrito à mão que decide um resultado do Presage: o que
ele é, de onde veio, quanto vale a confiança no número que ele produz, onde está
no código e o que o substitui quando a base de partidas existir.

## 1. Por que este documento existe

O Presage promete dizer de onde vem cada número
([visão geral §2](visao-geral.md#2-de-onde-vem-cada-número)). As etiquetas
**fraca** e **regra** só são honestas se o peso por trás delas estiver à vista:
quem lê "chance por rota, fraca como probabilidade" tem de poder achar o `0,60`
que a produz, ver que foi escolhido e não ajustado ([P-7](perguntas-em-aberto.md)),
e saber o que vai tomar o lugar dele.

A pergunta que este documento responde, peso a peso, é a da RNF-1: o que aqui é
medição e o que é escolha. E a segunda, que decide o roteiro: quando a base
existir, o que ela substitui e o que fica porque é mecânica do jogo.

## 2. Como ler

Cada entrada é um `RN-n` com cinco linhas:

| Linha | Diz |
|---|---|
| **Regra** | o que o código faz, em uma ou duas frases |
| **Origem** | **mecânica do jogo** (fato do jogo, da Riot ou da matemática) ou **escolha à mão** |
| **Confiança** | a etiqueta do número que sai dela: exata, alta, fraca ou regra ([README](../README.md#confiança-do-número)) |
| **Onde** | o arquivo e a linha, e a entrada do 10 ou do 11 |
| **Quando a base existir** | o que a substitui, ou "fica: é mecânica" |

**Os valores.** Uma constante aparece aqui num bloco `rust`, escrita como no
código; o título da seção dá o módulo, e o `doccheck` confere cada valor contra
o código a cada push ([ADR-0020](../adr/0020-o-doc-e-o-contrato-e-o-doccheck-prova.md)).
Um valor que vive dentro de uma função ou de um `impl Default` não tem como ser
conferido: quando ele já tem um lugar nos docs, a entrada aponta para lá; quando
não tem, ele fica numa tabela marcada **valor no código, não conferido pelo
doccheck**.

Os defeitos que estes pesos causam hoje estão em `build/audit.md`, e as dúvidas
sobre eles nas [perguntas em aberto](perguntas-em-aberto.md).

## 3. Ranking do draft — `core::engine`

### RN-1 — A partida neutra vale 50%

```rust
const PRIOR_RATE: f64 = 0.50;
```

- **Regra:** toda taxa encolhida parte de 50%, e o `delta` da tela é a distância
  até ele.
- **Origem:** mecânica: toda partida tem um vencedor e um perdedor, então a média
  de qualquer população de partidas é 50%.
- **Confiança:** exata como média.
- **Onde:** `core/src/engine/mod.rs:34`; 11 §9. `wincon.rs:149` escreve `0.5` de
  novo em vez de usar a constante.
- **Quando a base existir:** fica: é a média por construção. A referência de cada
  efeito passa a vir da base (ADR-0008).

### RN-2 — A força da winrate base é escolhida, não derivada

```rust
const BASE_STRENGTH: f64 = 400.0;
```

- **Regra:** a winrate base do campeão na rota vale 400 partidas fictícias de
  50%. É a única força do motor que não sai de um espalhamento
  (`strength_for_sd`); equivale a um desvio de cerca de 2,5pp.
- **Origem:** escolha à mão.
- **Confiança:** fraca.
- **Onde:** `core/src/engine/mod.rs:36`, usada também por `team.rs` e `wincon.rs`
  (que a usa para encolher as faixas de duração); 02 §2.
- **Quando a base existir:** `strength_for_sd(τ)`, com o τ da winrate base medido
  por Bayes empírico sobre a própria base, como as outras
  ([ADR-0009](../adr/0009-forca-do-shrinkage-derivada-do-desvio.md)).

### RN-3 — O quanto cada efeito costuma se espalhar

```rust
const MATCHUP_PRIOR_SD: f64 = 0.030;
const SYNERGY_PRIOR_SD: f64 = 0.012;
const PERSONAL_PRIOR_SD: f64 = 0.035;
const VERSUS_PRIOR_SD: f64 = 0.010;
```

- **Regra:** o desvio típico de cada efeito na população. Dele sai a força do
  prior (`strength_for_sd`) e o `±` de quando não há amostra: 3pp dão ≈ 277
  partidas fictícias, 1,2pp ≈ 1735, 3,5pp ≈ 203, 1pp 2499.
- **Origem:** escolha à mão. O comentário do contra o chama de palpite.
- **Confiança:** fraca.
- **Onde:** `core/src/engine/mod.rs:41-46`; 02 §2 (a tabela de força e desvio).
  `lanes.rs` usa o do matchup; `team.rs`, os do matchup, da sinergia e do
  contra.
- **Quando a base existir:** o τ de cada tabela medido pelo `--verificar` do
  coletor (variância das taxas menos o ruído binomial médio). O do contra o
  coletor já mede ([05 §7](../arquitetura/05-coletor.md#7-o-que-esperar-da-base)).
  O pessoal não sai da base agregada: é por conta ([P-4](perguntas-em-aberto.md)).

### RN-4 — A composição tem sempre um desvio

```rust
const COMP_SD: f64 = 0.010;
```

- **Regra:** 1pp entra na variância de todo candidato, com ou sem base.
- **Origem:** escolha à mão.
- **Confiança:** fraca.
- **Onde:** `core/src/engine/mod.rs:47`; 02 §2.
- **Quando a base existir:** o desvio residual da regressão de composição sobre
  partidas reais.

### RN-5 — Os pesos da composição no ranking

```rust
const W_DAMAGE_BALANCE: f64 = 0.050;
const W_FRONTLINE: f64 = 0.016;
```

- **Regra:** o fit de composição é `W_DAMAGE_BALANCE × ganho de equilíbrio +
  W_FRONTLINE × necessidade de frontline`, em pontos de probabilidade, somado em
  log-odds a partir de 50%. O ganho é o quanto o candidato aproxima o time de 50%
  de dano físico; a necessidade, `1 − frontlines`, a partir do primeiro aliado.
  Os dois são multiplicados pela completude.
- **Origem:** escolha à mão.
- **Confiança:** fraca. É o score de draft (`+3.4`) de hoje.
- **Onde:** `core/src/engine/mod.rs:52-53`, aplicados em `mod.rs:239-245`; a
  forma em `comp.rs`; 02 §4, 11 §13.
- **Quando a base existir:** os coeficientes de uma regressão logística da
  vitória sobre o equilíbrio de dano e a frontline, em partidas reais. O mesmo
  efeito tem um segundo modelo no minimax (RN-11, [P-11](perguntas-em-aberto.md)).

Os números da forma (valor no código, não conferido pelo doccheck):

| O quê | Valor | Onde |
|---|---|---|
| completude | `min(known / 4, 1)`: os quatro outros do time | `comp.rs:15` |
| alvo do equilíbrio | 50% de dano físico | `comp.rs:48` |
| frontline que basta | uma (`1 − frontline`, nunca negativo) | `comp.rs:57` |
| time vazio | 50% físico, sem sinal | `comp.rs:25` |

### RN-6 — O corte da winrate e os limiares das razões

```rust
const MAX_REASONS: usize = 3;
```

- **Regra:** a winrate do ranking fica em [25%; 75%]; cada candidato leva até três
  razões, e cada razão só sai acima de um limiar.
- **Origem:** escolha à mão.
- **Confiança:** regra. O corte nunca age sem base: a composição move cerca de
  1,5pp.
- **Onde:** `core/src/engine/mod.rs:55`, `mod.rs:262`, `mod.rs:321`,
  `mod.rs:340`, `mod.rs:353`; 11 §9.
- **Quando a base existir:** o corte, pela calibração da soma contra resultados
  reais; os limiares, por significância (intervalo que não cruza o zero, como
  `Reading::is_clear` do perfil).

Valor no código, não conferido pelo doccheck:

| O quê | Valor | Onde |
|---|---|---|
| corte da winrate | [0,25; 0,75] | `mod.rs:262` |
| razão de matchup e de contra | `abs(δ) > 0,005` (0,5pp) e amostra | `mod.rs:277`, `mod.rs:321` |
| razão de equilíbrio | ganho `> 0,01` | `mod.rs:340` |
| razão do histórico pessoal | 5 partidas ou mais no campeão | `mod.rs:353` |
| `±` | `1,96 ×` o desvio: o quantil de 95% da normal | `mod.rs:299` |

## 4. Contexto do draft — `core::engine::context`

### RN-7 — O nível da prévia e a plausibilidade mínima

```rust
pub const PREGAME_LEVEL: u8 = 6;
const EPS: f64 = 0.01;
```

- **Regra:** antes da partida, a prévia compara as rotas no nível 6, igual para
  os dez. Na inferência de rota, cada campeão fora da rota custa `ln(0,01)`: uma
  atribuição improvável ainda ganha quando nenhuma é plausível.
- **Origem:** escolha à mão. O comentário diz que "é no 6 que as rotas se
  resolvem", sem fonte.
- **Confiança:** regra. Os números da prévia herdam a confiança da análise de
  rota (fraca).
- **Onde:** `core/src/engine/context.rs:32`, `context.rs:36`; 10 §10.1, 02 §0.
- **Quando a base existir:** o nível em que a rota mais decide a partida, medido.
  O `EPS` fica: continua necessário para fração de rota zero. O empate da
  inferência é a [P-20](perguntas-em-aberto.md).

### RN-8 — Quem joga a rota (`core::engine::roles`)

```rust
const ROLE_SHARE: f64 = 0.05;
```

- **Regra:** com a base, joga a rota quem tem nela 5% das próprias partidas. Sem a
  base, pela classe do Data Dragon: classe principal da rota vale 1, secundária
  0,5, nenhuma 0. Para os **seus** candidatos e a sua pool, o seu histórico vale
  antes de tudo (`plays`); para as respostas **deles**, não vale (`others_play`,
  A12).
- **Origem:** escolha à mão, nas classes de cada rota e no limiar.
- **Confiança:** regra.
- **Onde:** `core/src/engine/roles.rs:18`, classes em `roles.rs:28-37`, encaixe em
  `roles.rs:63-67`; 11 §11. A ordem de `ROLES` desempata a inferência (RN-7).
- **Quando a base existir:** `role_share` decide, e o limiar de 5% pode sair da
  cauda das partidas por rota. A classe fica para campeão novo que a base não
  viu.

Valor no código, não conferido pelo doccheck:

| Rota | Classes que jogam |
|---|---|
| Topo | `Fighter`, `Tank` |
| Selva | `Fighter`, `Tank`, `Assassin` |
| Meio | `Mage`, `Assassin` |
| Bot | `Marksman` |
| Suporte | `Support` (tanque sem a classe Suporte não entra) |

### RN-9 — O que um campeão é, pelo Data Dragon (`core::engine::champion`)

- **Regra:** três leituras do `champion.json` que decidem composição, pool,
  runas, wincon e build:
  - fração física: metade a razão `attack / (attack + magic)` do `info`, metade um
    prior pela classe principal;
  - à distância: alcance de ataque de 500 ou mais;
  - linha de frente: classe `Tank` ou defesa 7 ou mais no `info`.
- **Origem:** escolha à mão. O `info` é uma nota de 0 a 10 da Riot, grosseira.
- **Confiança:** fraca na fração física; regra nas outras duas.
- **Onde:** `core/src/engine/champion.rs:108-127`; 10 §23.3.
- **Quando a base existir:** a fração física sai do dano real por campeão na
  match-v5 (`physicalDamageDealtToChampions` contra
  `magicDamageDealtToChampions`), como o próprio comentário diz; a linha de
  frente, do dano recebido e mitigado (`totalDamageTaken`,
  `damageSelfMitigated`). O alcance fica: é mecânica.

Valor no código, não conferido pelo doccheck:

| O quê | Valor | Onde |
|---|---|---|
| prior de classe da fração física | `Marksman` 0,90, `Mage` 0,10, `Support` 0,40, `Assassin` 0,55, o resto 0,70 | `champion.rs:111-117` |
| sem `info` | 0,5 | `champion.rs:110` |
| à distância | alcance ≥ 500 | `champion.rs:122` |
| linha de frente | `Tank` ou defesa ≥ 7 | `champion.rs:126` |

O crescimento por nível (`grown`, `0,7025 + 0,0175n`) é mecânica do jogo, com a
confiança **alta**; não é escolha, e fica como está (`champion.rs:47-50`).

## 5. Resposta adversária e ban — `core::engine::minimax`

### RN-10 — A largura da busca e o número de bans

- **Regra:** a casca pede resposta adversária para os 12 melhores pelo guloso, e
  dez sugestões de ban.
- **Origem:** escolha à mão: orçamento de CPU dentro da leitura de 350 ms (RNF-4).
- **Confiança:** regra.
- **Onde:** `src-tauri/src/lib.rs:226` (largura 12) e `lib.rs:237` (10 bans),
  valores no código, não conferidos pelo doccheck; 12 §2.
- **Quando a base existir:** fica: é orçamento de tempo. Mas a poda usa o valor
  do time, que ignora a base, e com ela o primeiro do ranking pode ficar de fora
  dos 12 (defeito no `build/audit.md`).

### RN-11 — O segundo modelo da composição

```rust
const W_BALANCE: f64 = 0.06;
const W_FRONTLINE: f64 = 0.025;
const FRONTLINE_FROM: usize = 3;
```

- **Regra:** o valor do time é `0,5 + vantagem nossa − vantagem deles`, com a
  penalidade de cada lado `W_BALANCE × desequilíbrio`, mais `W_FRONTLINE` quando
  ninguém segura a frente a partir do terceiro pick. O desequilíbrio é
  `abs(fração física − 0,5) × 2`; a completude, `known / 5`.
- **Origem:** escolha à mão. Mede o mesmo efeito que a RN-5 com outra forma e
  outros pesos ([P-11](perguntas-em-aberto.md)).
- **Confiança:** fraca. A exposição aparece na tela em pp, sem etiqueta.
- **Onde:** `core/src/engine/minimax.rs:28-32`, a forma em `minimax.rs:64-86`;
  11 §12.1.
- **Quando a base existir:** o modelo de time (`team.rs`) levado ao minimax, como
  a [ADR-0019](../adr/0019-contra-em-qualquer-rota-e-chance-do-time.md) diz que é o
  passo seguinte.

Valor no código, não conferido pelo doccheck:

| O quê | Valor | Onde |
|---|---|---|
| completude | `min(known / 5, 1)` | `minimax.rs:76` |
| desequilíbrio | `abs(share − 0,5) × 2` | `minimax.rs:78` |
| corte do valor | [0,05; 0,95], que nunca age: a penalidade máxima de um lado é 0,085 | `minimax.rs:65` |

## 6. Chance do time e condição de vitória — `core::engine::team`

### RN-12 — A trava da chance do time

```rust
const FLOOR: f64 = 0.35;
const CEILING: f64 = 0.65;
const MAX_TERMS: usize = 4;
```

- **Regra:** a chance do time com os dez fica em [35%; 65%]; a tela mostra as
  quatro parcelas de maior efeito.
- **Origem:** escolha à mão; a ADR-0019 diz "escolhida, não medida".
- **Confiança:** fraca. Só existe com a base.
- **Onde:** `core/src/engine/team.rs:23-25`, aplicados em `team.rs:208`; 10 §14.
  O `±` ignora a trava ([P-41](perguntas-em-aberto.md)).
- **Quando a base existir:** calibração fora da amostra (chance prevista contra
  vitória observada por faixa). A trava some, ou passa a ser o ponto onde a curva
  de calibração achata.

### RN-13 — O que a condição de vitória afirma (`core::engine::wincon`)

```rust
const ONE_SIDED: f64 = 0.70;
const EDGE: f64 = 0.05;
const MIN_KNOWN: usize = 3;
const MAX_EDGES: usize = 5;
const LEAN: f64 = 0.08;
```

- **Regra:** fato de composição com 70% ou mais do dano de um tipo; vantagem de
  atributos com o duelo a 5 pontos ou mais de 50%; nada com menos de três
  definidos; até cinco vantagens; fase do jogo só com diferença de inclinação
  acima de 0,08 em log-odds (cerca de 2 pontos).
- **Origem:** escolha à mão.
- **Confiança:** regra; a fase, fraca (é o limiar de um número que só existe com a
  base).
- **Onde:** `core/src/engine/wincon.rs:24-36`;
  [ADR-0018](../adr/0018-o-que-a-condicao-de-vitoria-pode-afirmar.md), 11 §15.
- **Quando a base existir:** a fração de dano sai do dano real (RN-9) e os
  limiares ficam, como regra. A fase troca o limiar fixo por significância: a
  diferença fora de ±1,96 desvio.

Valor no código, não conferido pelo doccheck:

| Fato | Limiar | Onde |
|---|---|---|
| "N de M seguram a linha de frente" | 3 ou mais | `wincon.rs:110` |
| "N de M atacam à distância" | 4 ou mais | `wincon.rs:118` |
| "N de M são corpo a corpo" | 4 ou mais | `wincon.rs:123` |
| prior da força na rota, na fase | 0,5, com `BASE_STRENGTH` para as faixas | `wincon.rs:149-151` |

### RN-14 — As faixas de duração (`core::engine::stats`)

- **Regra:** curta abaixo de 1500 s (25:00), média até antes de 1950 s (32:30),
  longa a partir daí.
- **Origem:** escolha à mão ([P-33](perguntas-em-aberto.md)).
- **Confiança:** regra.
- **Onde:** `Length::of`, `core/src/engine/stats.rs:129-135`, valores no código,
  não conferidos pelo doccheck; 02 §8.
- **Quando a base existir:** os quantis de `gameDuration` da própria base.

## 7. Pool — `core::engine::poolgap`

### RN-15 — Oito por rota, três escolhidos

```rust
pub const POOL_SIZE: usize = 8;
pub const CORE_SIZE: usize = 3;
```

- **Regra:** a pool tem oito por rota: três que o jogador escolhe e cinco que a
  cobertura recomenda. A sua lista de bans tem até cinco (`BAN_POOL_MAX`, 10 §7.3).
- **Origem:** escolha à mão, de produto
  ([ADR-0016](../adr/0016-pool-de-oito-tres-escolhidos-cinco-por-cobertura.md));
  os cinco bans são mecânica do jogo.
- **Confiança:** regra.
- **Onde:** `core/src/engine/poolgap.rs:25-26`; 10 §17.1. O front copia os dois em
  `src/lib/types.ts`, e o limite de três core só vale lá.
- **Quando a base existir:** fica: é desenho do produto.

### RN-16 — As características da pool

```rust
const LEVEL: u8 = 11;
const ROBUST: f64 = 0.55;
const RELEVANT: f64 = 0.5;
const MAX_REASONS: usize = 3;
```

- **Regra:** a robustez é a menor vida efetiva (dano todo físico ou todo mágico)
  sobre a maior, no nível 11, sem itens; acima de 0,55 o campeão tem
  resistências equilibradas. Só característica com peso de 0,5 ou mais na rota
  vira ponto forte ou fraco. Até três razões por candidato.
- **Origem:** escolha à mão sobre a fórmula de resistência do jogo.
- **Confiança:** alta na robustez (a ordem é confiável, o valor é índice); regra
  nos limiares.
- **Onde:** `core/src/engine/poolgap.rs:29-38`; 10 §17.2, 11 §17.
- **Quando a base existir:** fica como regra; o que muda é a fração física por
  trás de físico e mágico (RN-9).

Valor no código, não conferido pelo doccheck:

| Característica | Tem quando | Onde |
|---|---|---|
| `Fisico` | fração física ≥ 0,65 | `poolgap.rs:95` |
| `Magico` | fração física ≤ 0,35 | `poolgap.rs:96` |
| `Robusto` | robustez ≥ `ROBUST` | `poolgap.rs:100` |

### RN-17 — O que cada rota valoriza, e o retorno decrescente

- **Regra:** cada característica tem um peso por rota (`weight`), e a cobertura
  dela cresce como `1 − ½ⁿ`: o candidato vale `Σ peso × ½ⁿ⁺¹` sobre o que ele
  tem, dividido pela soma de todas as doze. "Um estilo só" é ponto fraco com dois
  core ou mais da mesma classe principal ([P-35](perguntas-em-aberto.md)).
- **Origem:** escolha à mão (ADR-0016).
- **Confiança:** regra.
- **Onde:** `core/src/engine/poolgap.rs:183-198` (pesos), `poolgap.rs:311`
  (estilo), `poolgap.rs:357-358` (retorno e teto); 02 §7, 11 §17.
- **Quando a base existir:** a tabela de matchup diz contra quem os três perdem e
  quem ganha desses; o peso de cada característica pode sair do quanto ela muda
  a winrate da pool na rota.

Valor no código, não conferido pelo doccheck. Na ordem do `match`, a primeira
linha que casa vence:

| Rota | Característica | Peso |
|---|---|---|
| Bot | `Frontline`, `Alcance`, `CorpoACorpo` | 0,0 |
| Bot | `Fisico`, `Magico` | 0,6 |
| Meio | `Frontline` | 0,4 |
| Selva | `Alcance` | 0,3 |
| Suporte | `Fisico`, `Magico` | 0,5 |
| qualquer | `Fisico`, `Magico`, `Frontline` | 1,0 |
| qualquer | `Alcance` | 0,6 |
| qualquer | `CorpoACorpo` | 0,4 |
| qualquer | `Robusto` | 0,5 |
| qualquer | os seis estilos | 0,35 |

## 8. Runas — `core::engine::runes`

### RN-18 — As regras do confronto

```rust
const MAX_SUGGESTIONS: usize = 3;
```

- **Regra:** treze regras sobre você, ele e a rota; cada uma que dispara soma o
  peso dela à runa principal. Ficam as três de maior soma. A secundária é a
  primeira de três regras que casa, com `score` 1, e não olha a árvore da
  principal (defeito no `build/audit.md`). "Explode" é assassino, ou mago que não
  é suporte.
- **Origem:** escolha à mão. Cada linha é uma afirmação sobre a mecânica do
  confronto, nenhuma sobre frequência de vitória.
- **Confiança:** regra. `from_data` é sempre falso.
- **Onde:** `core/src/engine/runes.rs:26`, `REGRAS` em `runes.rs:124-203`,
  `SECUNDARIAS` em `runes.rs:212-228`, `explode` em `runes.rs:109-111`; 11 §18.2.
- **Quando a base existir:** a winrate da runa no confronto (campeão × oponente ×
  runa), encolhida para a winrate da runa no campeão, e `from_data` vira
  verdadeiro.

Valor no código, não conferido pelo doccheck (`REGRAS`, na ordem):

| Runa | Peso | Quando |
|---|---|---|
| `GraspOfTheUndying` | 1,0 | você corpo a corpo, tanque ou lutador; ele à distância |
| `Conqueror` | 1,0 | você lutador corpo a corpo; ele corpo a corpo |
| `Conqueror` | 0,8 | você lutador; ele linha de frente. Soma com a de cima. (A condição "ou Juggernaut" saiu: o Data Dragon não tem essa classe, e ela nunca disparava) |
| `LethalTempo` | 0,9 | você atirador; ele linha de frente |
| `PressTheAttack` | 0,85 | você atirador; ele não é linha de frente |
| `Electrocute` | 1,0 | você assassino; ele não é linha de frente |
| `ArcaneComet` | 0,9 | você mago que não é suporte; ele corpo a corpo |
| `PhaseRush` | 0,95 | você mago ou lutador; ele explode e não é linha de frente |
| `SummonAery` | 0,8 | você mago ou suporte; ele à distância |
| `Aftershock` | 0,95 | você tanque, na rota de suporte, selva ou topo |
| `Guardian` | 0,8 | você suporte que não é tanque, na rota de suporte |
| `FleetFootwork` | 0,75 | os dois à distância; você não é assassino |
| `DarkHarvest` | 0,7 | você assassino corpo a corpo; ele não é linha de frente |

`SECUNDARIAS`, a primeira que casa: `SecondWind` (ele à distância, você corpo a
corpo), `BonePlating` (ele explode), `TasteOfBlood` (os dois à distância).

## 9. Build adaptativa — `core::engine::build`

### RN-19 — O que entra na lista e como o item é pesado

```rust
const MIN_ITEM_COST: f64 = 1000.0;
```

- **Regra:** só item comprável de 1000 de ouro ou mais. Poder é vida efetiva ×
  dano efetivo, e o dano efetivo pesa AD e AP pelo jeito que o seu campeão bate:
  `attack / (attack + magic)` para o AD e `magic / (attack + magic)` para o AP,
  com as notas de 0 a 10 do Data Dragon (`info`); sem nota, metade de cada. Tudo
  passa pelas resistências do alvo. A penetração inimiga é o pior caso: o maior
  percentual e a maior letalidade entre eles, juntos. Na escaramuça (RN-21) o
  dano continua somando AD e AP um para um, porque lá os itens são os de verdade.
- **Origem:** o produto vida × dano e as resistências são mecânica; o corte de
  custo, o peso pela nota e o pior caso são escolha à mão.
- **Confiança:** alta na ordem entre itens do mesmo tipo de dano; **regra** entre
  AD e AP: a nota da Riot diz com o que o campeão bate, não quanto cada ponto
  rende nele. Até a 0.7.0 o peso era 1 para 1 e punha item de AP no topo para
  campeão de AD (A4, T-BUILD-10).
- **Onde:** `core/src/engine/build.rs:24`, `damage_weights` e `offense_weighted`
  (dano efetivo da build), `build.rs:219-222` (pior caso), `build.rs:266`
  (corte); 11 §26, 03 §2.
- **Quando a base existir:** o peso entre AD e AP sai da divisão de dano real do
  campeão (match-v5); o corte de custo, da árvore do item no Data Dragon
  (`into`, `depth`); a penetração, ponderada pela fatia de dano de cada inimigo.

Valor no código, não conferido pelo doccheck:

| O quê | Valor | Onde |
|---|---|---|
| divisão do dano que vem, com item | AD / (AD + AP) dos itens; o AD base entra só no peso | `build.rs:148-154` |
| divisão do dano, sem item | o prior de classe (RN-9) | `build.rs:151` |
| sem peso nenhum | 0,5 | `build.rs:160` |
| razão de armadura / resistência mágica | dano físico > 0,6 / < 0,4 | `build.rs:312`, `build.rs:315` |
| razões por item | até 2 | `build.rs:337` |
| chance de escaramuça | `nossa / (nossa + deles)`, razão de poderes | `build.rs:203-206` |

## 10. Rotas e escaramuças — `core::engine::lanes`

### RN-20 — O peso do duelo na chance da rota

```rust
const W_DUEL: f64 = 0.60;
const FLOOR: f64 = 0.28;
const CEILING: f64 = 0.72;
const MAX_REASONS: usize = 3;
pub const BREAKPOINTS: [u8; 4] = [3, 6, 11, 16];
```

- **Regra:** `chance = 0,5 + W_DUEL × (duelo − 0,5) + (estimativa − 0,5)`, cortada
  em [28%; 72%]: um duelo de 60/40 vira 56% de rota, e nenhuma rota está decidida
  antes de começar. A curva de nível olha os níveis 3, 6, 11 e 16. A soma é em
  probabilidade, não em log-odds como no draft.
- **Origem:** escolha à mão no peso e no corte; 6, 11 e 16 são mecânica (as cargas
  da suprema); 3 é escolha.
- **Confiança:** fraca como probabilidade ([P-7](perguntas-em-aberto.md)).
- **Onde:** `core/src/engine/lanes.rs:28-38`, a soma em `lanes.rs:307`; 03 §4,
  10 §27.
- **Quando a base existir:** o peso, pela regressão da vitória de rota sobre o
  duelo; o corte, pela calibração, onde as pontas reais aparecem. Os níveis-chave
  ficam: são exibição.

### RN-21 — Quem briga junto, e qual escaramuça destacar

```rust
const OBJECTIVE_WINDOW: f64 = 90.0;
```

- **Regra:** cinco agrupamentos fixos (selva com topo, selva com meio, bot, dragão,
  arauto). Barão, Arauto e Grubs puxam o grupo do arauto; Dragão e Ancião, o do
  dragão; um objetivo a até 90 s decide o destaque. Razão de duelo só a 2 pontos
  ou mais de 50%; "aguenta mais um golpe" com 15% de vida a mais.
- **Origem:** escolha à mão sobre o mapa do jogo.
- **Confiança:** regra.
- **Onde:** `core/src/engine/lanes.rs:456`, `GROUPS` em `lanes.rs:207-221`,
  `group_of` em `lanes.rs:468-474`, `my_duo` em `lanes.rs:477-485`, os limiares em
  `lanes.rs:376` e `lanes.rs:403`, valores no código, não conferidos pelo
  doccheck; 11 §27. A mesma janela de 90 s está de novo em `priority.rs` (RN-25).
- **Quando a base existir:** a timeline da match-v5 diria quem briga junto e onde
  ([P-3](perguntas-em-aberto.md)).

## 11. Ouro — `core::engine::gold`

### RN-22 — A renda reconstruída

```rust
const STARTING: f64 = 500.0;
const PASSIVE_START: f64 = 110.0;
const PASSIVE_PER_SECOND: f64 = 2.04;
const PER_CS: f64 = 21.0;
const PER_KILL: f64 = 300.0;
const PER_ASSIST: f64 = 150.0;
const UNSPENT_ALERT: f64 = 1500.0;
```

- **Regra:** ouro ganho = 500 iniciais + renda passiva a partir de 1:50 + 21 por
  tropa + 300 por abate + 150 por assistência, nunca abaixo do investido. Com 1500
  ou mais em mão, a sua, a nota "Você tem ~N de ouro para gastar".
- **Origem:** inicial e passiva são mecânica do jogo ([P-34](perguntas-em-aberto.md));
  tropa, abate e assistência são médias escolhidas à mão, sem primeiro abate,
  sequência, recompensa por cabeça, torre, objetivo nem placa; o alerta é
  escolha.
- **Confiança:** fraca. O comentário chama o número de piso, mas a assistência
  cheia para cada assistente pode passar do real numa briga de time.
- **Onde:** `core/src/engine/gold.rs:22-39`; 10 §28, 11 §28.
- **Quando a base existir:** o ouro por minuto da timeline da match-v5, por rota e
  minuto ([P-3](perguntas-em-aberto.md)); o seu, lido da Live Client
  ([P-18](perguntas-em-aberto.md)). O inicial e a passiva ficam: são mecânica.

Valor no código, não conferido pelo doccheck:

| O quê | Valor | Onde |
|---|---|---|
| nota de vantagem do time em itens | 1000 ou mais | `gold.rs:140` |
| piso do divisor do ouro por minuto | 1 s | `gold.rs:91` |
| `+N` de ouro em mão na tela | 800 ou mais, para os dez (contra a C-4) | `src/components/GoldTracker.tsx:70` |
| o mesmo fato no pós-jogo | 1300 (`IDLE_GOLD`, RN-27) | `coach.rs:31` |

## 12. Sua rota ao vivo — `core::engine::rival`

### RN-23 — A margem de ouro e o começo da partida

```rust
pub const MIN_SECONDS: f64 = 90.0;
pub const AHEAD_FLOOR: f64 = 300.0;
pub const AHEAD_SHARE: f64 = 0.10;
```

- **Regra:** antes de 1:30 não há painel: a taxa por minuto é ruído. À frente ou
  atrás quando a diferença de ouro em itens chega a `max(300, 10% da média dos
  dois)`. A métrica é a da `cobranca` (RN-26).
- **Origem:** escolha à mão.
- **Confiança:** regra; o ouro em itens comparado é exato.
- **Onde:** `core/src/engine/rival.rs:21-27`, `standing`; 10 §22.1. Até a 0.7.0
  a margem só olhava o ouro do oponente, e o mesmo par não era simétrico (A24).
- **Quando a base existir:** a chance de vitória por diferença de ouro no minuto
  calibraria a margem; nada a substitui hoje.

## 13. Timers — `core::engine::timeline`

### RN-24 — Os tempos dos objetivos e a leitura dos buffs

```rust
const BUFF_FRESH: f64 = 5.0;
```

- **Regra:** os tempos de spawn, respawn e buff vêm de `Timings::default()`. O
  buff só diz quem tem quando o abate foi visto até 5 s depois de acontecer.
  Arauto e Grubs não tomados somem 300 s depois de nascer; dois abates do mesmo
  buff a menos de 0,5 s são um só.
- **Origem:** os tempos são mecânica do jogo, conferidos à mão contra o patch 16;
  os 5 s, os 300 s e os 0,5 s são escolha.
- **Confiança:** exata no respawn observado; estimada no primeiro spawn; regra nas
  janelas.
- **Onde:** `core/src/engine/timeline.rs:233`; os valores de `Timings` estão no
  10 §29.1, o único lugar deles nos docs (vivem num `impl Default`, e o doccheck
  não os confere); os 300 s em `timeline.rs:385` e `timeline.rs:388`, valor no
  código, não conferido ([P-23](perguntas-em-aberto.md)).
- **Quando a base existir:** fica: são mecânica do jogo. O que as substitui é
  conferir no patch, ou ler o spawn da Live Client se um dia ela der.

## 14. Prioridade — `core::engine::priority`

### RN-25 — Os pesos de tela da prioridade

```rust
const DEAD_ENEMY_MAX: f64 = 25.0;
const DEAD_JUNGLER: f64 = 12.0;
const BEHIND_MAX: f64 = 10.0;
const OBJECTIVE_MAX: f64 = 15.0;
const OBJECTIVE_WINDOW: f64 = 90.0;
```

- **Regra:** a base do placar é a chance de escaramuça × 100. Somam: inimigo da
  rota morto, metade do tempo de respawn até 25; caçador inimigo morto, 12;
  tropas deles atrás, até 10, cheio com 20 de diferença; objetivo do lado da rota
  a até 90 s, até 15, linear na urgência. Placar em [0; 100]; uma razão por
  sinal que somou ponto, até cinco (A23: o corte em três deixava pontos sem razão).
- **Origem:** escolha à mão.
- **Confiança:** fraca. São pesos de exibição, e a prioridade não tem etiqueta na
  tela.
- **Onde:** `core/src/engine/priority.rs:24-30`, as somas e as razões em
  `score_lane`; 10 §30.
  As razões são cortadas depois de somar os pontos (defeito no `build/audit.md`).
- **Quando a base existir:** um modelo de valor por rota ajustado contra o
  resultado de escaramuça e objetivo.

Valor no código, não conferido pelo doccheck (`LANES`, `priority.rs:56-72`):

| Rota | Posições | Objetivos que puxam |
|---|---|---|
| `TOP` | `TOP` | Barão, Arauto, Grubs |
| `MID` | `MIDDLE` | nenhum |
| `BOT` | `BOTTOM`, `UTILITY` | Dragão, Ancião |

## 15. Pós-jogo — `core::engine::coach`

### RN-26 — A rota decide o que se cobra

- **Regra:** cada rota tem um trabalho: farm contra o oponente para topo, selva,
  meio e atirador; visão para o suporte, nunca tropas; e limiares de
  participação em abates próprios. A sua rota ao vivo usa a mesma tabela
  (RN-23).
- **Origem:** escolha à mão ([L-27](../arquitetura/09-licoes-do-prototipo.md)).
- **Confiança:** regra; as observações que saem dela são exatas.
- **Onde:** `cobranca`, `core/src/engine/coach.rs:51-59`. Os valores estão na
  tabela do [03 §8](../arquitetura/03-motor-da-partida.md#8-leitura-pós-partida)
  (valor no código, não conferido pelo doccheck).
- **Quando a base existir:** a distribuição da participação por rota e elo; o
  perfil de rota já mede `KillParticipation`.

### RN-27 — Os limiares das observações

```rust
const LEGENDARY_COST: f64 = 2500.0;
const CLUSTER_WINDOW: f64 = 240.0;
const CLUSTER_DEATHS: usize = 3;
const IDLE_GOLD: f64 = 1300.0;
const MIN_DURATION: f64 = 300.0;
const BUCKET: f64 = 30.0;
```

- **Regra:** item grande custa 2500 ou mais; três mortes em 240 s são uma
  sequência; ouro parado acima de 1300; partida de menos de 5 minutos não rende
  leitura; o gráfico reamostra a cada 30 s. Os demais limiares (tropas, visão,
  tempo morto, item tardio e cedo, vantagem devolvida, nível) e os pesos de cada
  observação estão no 11 §31.1.
- **Origem:** escolha à mão.
- **Confiança:** regra; o ouro parado é fraco, porque sai da renda reconstruída
  (RN-22).
- **Onde:** `core/src/engine/coach.rs:24-34`, `coach.rs:153`; a tabela completa no
  11 §31.1 (valor no código, não conferido pelo doccheck). O ouro de uma tropa é
  `21.0` escrito de novo em `coach.rs:370`, cópia de `PER_CS`.
- **Quando a base existir:** os percentis de diferença por rota e elo (tropas,
  visão, tempo morto), o tempo do primeiro item por campeão e rota, e a curva de
  vitória por vantagem de ouro.

### RN-28 — O teto e o piso da gravação (`casca::recorder`, `core::engine::coach`)

O teto é da casca:

```rust
const MAX_FRAMES: usize = 6000;
```

### O piso da RN-28, em `core::engine::coach`

```rust
const MIN_FRAMES: usize = 60;
```

- **Regra:** até 6000 quadros por partida. Para render leitura, a gravação
  precisa de ao menos 60 quadros **e** de uma partida de 300 s no relógio
  (`MIN_DURATION`, RN-27): os dois pisos moram juntos em `coach::worth_reviewing`.
- **Origem:** escolha à mão (disco, e o que dá para afirmar).
- **Confiança:** regra.
- **Onde:** `src-tauri/src/recorder.rs:23` (teto); `core/src/engine/coach.rs`
  (`MIN_FRAMES`, `worth_reviewing`); 11 §31.4, 12 §7. Até a 0.7.0 o piso de
  quadros vivia só na casca, e um remake de três minutos passava nele e apagava a
  última leitura (A6).
- **Quando a base existir:** fica: é orçamento de disco e honestidade.

## 16. Perfil de rota e histórico pessoal — `core::engine::profile`

### RN-29 — Quando uma leitura do perfil conta

```rust
const MIN_GAMES: usize = 5;
const EPS: f64 = 1e-9;
```

- **Regra:** com menos de cinco partidas a métrica fica de fora. Só ganha cor o
  que é claro a 95% (`abs(valor) > 1,96 × desvio`). Com menos de dois graus de
  liberdade dentro dos campeões, o ruído é a variância total, maior, e o perfil
  erra para a cautela.
- **Origem:** escolha à mão; o 1,96 é o quantil de 95% da normal.
- **Confiança:** regra; o perfil em si é medição, com `±`.
- **Onde:** `core/src/engine/profile.rs:32-35`, `is_clear` em `profile.rs:53`, o
  ruído em `profile.rs:210-215`; 04 §4. A tela repete o 1,96 em
  `src/components/PerfilDeRota.tsx:19`.
- **Quando a base existir:** fica: é a medição da conta, não da base. A base
  daria uma referência por rota e campeão
  ([04 §6](../arquitetura/04-perfil-de-rota.md#6-o-que-ele-não-separa)).

### RN-30 — Os cortes de partida do histórico (`core::history`)

```rust
const MIN_DURATION: i64 = 300;
```

- **Regra:** fora da Fenda (`CLASSIC`), terminada por erro, remake e menos de
  5 minutos não contam, nem para vitória. Rota em branco conta a vitória sem
  comparação de rota.
- **Origem:** os literais `CLASSIC` e `GameComplete` são da Riot; os 300 s, escolha
  à mão, acima da janela de remake.
- **Confiança:** regra.
- **Onde:** `core/src/history.rs:21`, regras em `history.rs:182-207`; 04 §3
  (Os cortes). O coletor tem as mesmas quatro escritas de novo (RN-32).
- **Quando a base existir:** a distribuição de duração diria onde cortar. Uma
  constante só para os dois.

### RN-31 — O histórico pessoal da casca (`casca::riot`)

```rust
const RANKED_SOLO_QUEUE: u32 = 420;
```

- **Regra:** o histórico pessoal pede só a solo/duo, até 60 partidas por
  atualização (`MATCHES_PER_REFRESH`, `src-tauri/src/profile.rs:25`).
- **Origem:** escolha à mão; o teto de 60 cabe no limite da chave de
  desenvolvimento.
- **Confiança:** regra.
- **Onde:** `src-tauri/src/riot.rs:26`; 12 §5. A flex é a
  [P-25](perguntas-em-aberto.md).
- **Quando a base existir:** fica. Com a chave de produção, o teto sobe.

## 17. Coletor — `coletor::aggregate`

### RN-32 — O que entra na base

```rust
const MIN_DURATION: i64 = 300;
const TEAM_SIZE: usize = 5;
const PARTICIPANTS: usize = 10;
const MIN_SPREAD_ENTRIES: usize = 30;
pub const USEFUL: u32 = 200;
```

- **Regra:** a partida entra inteira ou não entra: fila e patch do recorte, modo
  `CLASSIC`, terminada, dez participantes, sem remake, 5 minutos ou mais, toda rota
  preenchida e uma de cada lado. Uma célula com 200 partidas ou mais é útil; o
  espalhamento do contra só é medido com 30 pares úteis.
- **Origem:** dez participantes e cinco por time são mecânica; os 300 s, os 200 e
  os 30, escolha à mão.
- **Confiança:** regra.
- **Onde:** `tools/ingest/src/aggregate.rs:35-38`, `aggregate.rs:345`,
  `aggregate.rs:393`; 05 §3, 05 §5. O comentário de `USEFUL` ("mais de um ponto")
  depende de `MATCHUP_PRIOR_SD` (RN-3) e não o lê.
- **Quando a base existir:** `USEFUL` passa a sair de `strength_for_sd` do motor;
  os 300 s, da distribuição de duração (RN-30).

Valor no código, não conferido pelo doccheck (`Opcoes::default`,
`tools/ingest/src/main.rs:81-99`):

| Opção | Padrão | Decide |
|---|---|---|
| `--min-jogos` | 20 | a poda: o que fica na base |
| `--jogadores`, `--por-jogador` | 20000, 20 | o tamanho do crawl |
| `--tier`, `--fila` | `EMERALD`, 420 | o recorte |
| `--paralelo` | 8 | quantas partidas baixa ao mesmo tempo |
| fração única do `--plano` | 0,6, chute escrito à mão | só a estimativa de tempo (`main.rs:278`) |

## 18. Tela — `src/`

### RN-33 — Os cortes que pintam um número

- **Regra:** a tela escolhe a cor de um número por cortes fixos, longe do `±` que
  o motor calcula.
- **Origem:** escolha à mão.
- **Confiança:** fraca: a cor não carrega a incerteza.
- **Onde:** a tabela abaixo (TypeScript; o doccheck não lê constante do front).
- **Quando a base existir:** pintar só quando o intervalo não cruza 50%
  (`win ∓ confidence / 100` do mesmo lado), e o corte sai do front
  ([P-38](perguntas-em-aberto.md)).

Valor no código, não conferido pelo doccheck:

| Número | Verde / vermelho | Onde |
|---|---|---|
| delta do draft | ≥ 2 forte, ≥ 1 médio, < 1 fraco | `src/components/RecommendationPanel.tsx:36-40` |
| chance do time | ≥ 52% / ≤ 48% | `RecommendationPanel.tsx:363` |
| chance por rota e de escaramuça | ≥ 55% / ≤ 45% | `src/components/LaneAnalysis.tsx:15-19`, `src/overlay.tsx:170` |
| curva de nível | ≥ 53% / ≤ 47% | `src/components/CurvaDeNivel.tsx:31` |
| leitura do perfil | `abs(valor) > 1,96 × desvio`, cópia do core (RN-29) | `src/components/PerfilDeRota.tsx:19` |
| ouro em mão | `+N` a partir de 800 | `src/components/GoldTracker.tsx:70` |
| objetivo em destaque | a 60 s ou menos | `src/components/LiveOverview.tsx:50` |

## Resumo

| RN | Peso | Origem | Confiança | Substituto |
|---|---|---|---|---|
| RN-1 | `PRIOR_RATE` | mecânica | exata | fica |
| RN-2 | `BASE_STRENGTH` | escolha à mão | fraca | `strength_for_sd(τ)` da base |
| RN-3 | `*_PRIOR_SD` | escolha à mão | fraca | τ medido pelo `--verificar` |
| RN-4 | `COMP_SD` | escolha à mão | fraca | resíduo da regressão de composição |
| RN-5 | `W_DAMAGE_BALANCE`, `W_FRONTLINE` (ranking) | escolha à mão | fraca | regressão logística |
| RN-6 | corte [0,25; 0,75], limiares das razões | escolha à mão | regra | calibração; significância |
| RN-7 | `PREGAME_LEVEL`, `EPS` | escolha à mão | regra | nível medido; `EPS` fica |
| RN-8 | `ROLE_SHARE`, classes por rota | escolha à mão | regra | `role_share` |
| RN-9 | fração física, alcance, linha de frente | escolha à mão | fraca / regra | dano real da match-v5 |
| RN-10 | largura 12, 10 bans | escolha à mão | regra | fica |
| RN-11 | `W_BALANCE`, `W_FRONTLINE`, `FRONTLINE_FROM` (minimax) | escolha à mão | fraca | o modelo de time |
| RN-12 | `FLOOR`, `CEILING` da chance do time | escolha à mão | fraca | calibração |
| RN-13 | `ONE_SIDED`, `EDGE`, `MIN_KNOWN`, `LEAN` | escolha à mão | regra / fraca | significância na fase |
| RN-14 | faixas de duração | escolha à mão | regra | quantis da base |
| RN-15 | `POOL_SIZE`, `CORE_SIZE` | escolha à mão, de produto | regra | fica |
| RN-16 | `LEVEL`, `ROBUST`, `RELEVANT` | escolha à mão | alta / regra | fica |
| RN-17 | `weight` por rota, `½ⁿ` | escolha à mão | regra | matchup da pool |
| RN-18 | `REGRAS`, `SECUNDARIAS` | escolha à mão | regra | winrate da runa no confronto |
| RN-19 | `MIN_ITEM_COST`, AD e AP pela nota do Data Dragon | mecânica / escolha à mão | alta / regra | divisão de dano real; árvore do item |
| RN-20 | `W_DUEL`, `FLOOR`, `CEILING` (rota) | escolha à mão | fraca | regressão; calibração |
| RN-21 | `GROUPS`, `OBJECTIVE_WINDOW` (destaque) | escolha à mão | regra | timeline |
| RN-22 | renda reconstruída | mecânica / escolha à mão | fraca | timeline; `currentGold` |
| RN-23 | `MIN_SECONDS`, `AHEAD_FLOOR`, `AHEAD_SHARE` | escolha à mão | regra | nenhum hoje |
| RN-24 | `Timings`, `BUFF_FRESH` | mecânica / escolha à mão | exata / regra | fica |
| RN-25 | pesos da prioridade | escolha à mão | fraca | modelo de valor por rota |
| RN-26 | `cobranca` | escolha à mão | regra | participação por rota e elo |
| RN-27 | limiares do pós-jogo | escolha à mão | regra / fraca | percentis por rota e elo |
| RN-28 | `MIN_FRAMES`, `MAX_FRAMES` | escolha à mão | regra | fica |
| RN-29 | `MIN_GAMES`, 1,96 | escolha à mão | regra | fica |
| RN-30 | cortes do histórico | mecânica / escolha à mão | regra | distribuição de duração |
| RN-31 | `RANKED_SOLO_QUEUE`, 60 partidas | escolha à mão | regra | fica |
| RN-32 | cortes e limiares do coletor | mecânica / escolha à mão | regra | `strength_for_sd` |
| RN-33 | cortes de cor da tela | escolha à mão | fraca | o `±` do motor |
