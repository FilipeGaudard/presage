# 04 — Perfil de rota

`core` (`history.rs`, `engine/profile.rs`) com a busca na casca (`profile.rs`).
Responde: quanto você tira do oponente direto — no geral, por campeão e por
confronto.

## 1. Por que não winrate

Vitória e derrota sozinhas dizem quase nada sobre uma conta:

- cada partida é uma moeda de variância 0,25. Para ver 5pp de diferença entre
  dois campeões seus a 2σ: `0,05 ≥ 2 × 0,5 / √n`, logo **n ≥ 400 partidas em
  cada**;
- o matchmaking puxa toda conta para 50%, então a winrate geral não diz nada de
  nível, e a por campeão só diz algo relativo aos seus outros campeões.

A partida que a Riot devolve tem ~150 campos por participante, e a chamada já
foi paga — o histórico antigo guardava só campeão e resultado
([L-6](09-licoes-do-prototipo.md)).

## 2. O oponente direto

`history::played` lê cada partida contra quem jogou a mesma rota do outro lado
([ADR-0010](../adr/0010-oponente-direto-como-grupo-de-controle.md)). É o grupo
de controle que uma conta sozinha consegue ter: o matchmaking escolheu ele para o
seu nível, na mesma partida, na mesma rota, pelo mesmo tempo. E a diferença soma
exatamente zero na população — cada confronto entra uma vez com +d e outra com
−d —, então a referência existe sem base de partidas.

Sem oponente direto (posição em branco, dois na mesma rota do outro lado) a
partida conta para vitória e derrota mas não entra no perfil.

## 3. As métricas

| Métrica | De onde sai | Unidade | Por quê |
|---|---|---|---|
| `Cs10` | `laneMinionsFirst10Minutes`; `jungleCsBefore10Minutes` para o caçador; nada para o suporte | tropas | medida antes de a partida decidir |
| `KillParticipation` | `killParticipation` × 100 | pp | fração do próprio time: tira boa parte do resultado |
| `DamageShare` | `teamDamagePercentage` × 100 | pp | idem |
| `VisionPerMinute` | `visionScorePerMinute` | placar/min | a métrica de suporte e caçador |
| `Deaths` | `deaths` | mortes | carrega parte do resultado, mas é o erro mais acionável |

Ficaram de fora ouro e dano por minuto: no fim da partida medem quem ganhou,
porque quem ganhou farma o mapa inteiro. Suporte não compara tropa porque não
farma por desenho.

Uma métrica só entra numa partida se os **dois lados** a têm. Sem `challenges`
(partidas antigas, alguns modos), sobram as mortes.

### Os cortes

Os mesmos do coletor, e valem também para vitória e derrota (`Skip`): fora da
Fenda, terminada por erro, remake, menos de cinco minutos, jogador ausente. Uma
conta que soma remake mede a taxa de remake.

## 4. Três níveis

`profile::build` lê as partidas em três níveis, cada um encolhido para o de cima:

| Nível | Referência | Encolhe para |
|---|---|---|
| você | zero, por construção | — (é medida direta: média e erro padrão) |
| você com o campeão | — | a sua média |
| você com o campeão contra aquele oponente | — | o seu valor com o campeão |

Para um grupo com `n` partidas e média crua `x̄`, sob um pai de média crua `x̄ₚ`,
valor `vₚ` e variância `Vₚ`:

```text
amostragem  v  = σ² / n
puxão       B  = v / (v + τ²)          1 quando τ² = 0; perto de 0 com muita amostra
valor          = vₚ + (1 − B)(x̄ − x̄ₚ)
variância      = (1 − B)·v + B²·Vₚ
```

- `σ²`, o ruído de uma partida, é a variação dentro de cada campeão, somada. Com
  quase tudo em campeões de uma partida só não há como medir, e a variação total
  entra no lugar — maior, então o perfil erra para o lado da cautela.
- Abaixo de cinco partidas (`MIN_GAMES`) a métrica não sai.

### Quanto encolhe: DerSimonian–Laird

`τ²` não é escolhido à mão. `between_variance` estima quanto as médias se
espalham além do que o ruído explica:

```text
w = n / σ²     Q = Σ w (x̄ − centro)²     gl = células − grupos     C = Σ_grupo (Σw − Σw² / Σw)
τ² = max(0, (Q − gl) / C)
```

No nível de campeão há um grupo só (todos os seus campeões); no de confronto, um
grupo por campeão, cada um com o próprio centro. Grupo de uma célula não diz nada
sobre espalhamento e fica de fora.

A consequência que importa: **se os seus campeões não se distinguem acima do
ruído, todos colapsam na sua média**, em vez de o perfil inventar um especialista
(`campeoes_que_nao_se_distinguem_colapsam_na_media`). O limiar é baixo — DL trata
como parcialmente real qualquer coisa acima de Q = gl, mais ou menos z > 1 —, e
por isso a tela tem a segunda trava, a de cor.

## 5. Na tela e no draft

- **Pós-jogo** (`PerfilDeRota.tsx`, comando `lane_profile`): uma tabela com você
  no geral e cada campeão, `valor ± 1,96·desvio`. Só ganha cor o que é
  **claro** — o intervalo de 95% não cruza o zero (`Reading::is_clear`). Mortes é a
  única métrica em que menos é melhor. Confrontos com uma partida só não aparecem.
- **Draft**: a leitura mais clara do candidato vira razão ("Sua rota com ele: +6
  tropas aos 10 contra o oponente (n=22)"), e só como texto
  ([ADR-0011](../adr/0011-perfil-de-rota-e-razao-nao-peso.md)). Converter tropa em
  vitória exige a base de partidas. Há teste de que o ranking é idêntico com e sem
  perfil (`rota_clara_vira_razao_sem_mexer_na_ordem`).

## 6. O que ele não separa

- **Você e o campeão.** Quem perde tropa aos 10 por desenho aparece como se fosse
  você. Separar exige a referência por campeão e por confronto que só a base de
  partidas dá — o mesmo erro de referência do draft
  ([ADR-0008](../adr/0008-matchup-e-sinergia-contra-a-soma-das-forcas.md)), um
  nível abaixo.
- **Rotas.** O perfil de um campeão soma todas as rotas em que você o jogou
  ([P-9](../produto/perguntas-em-aberto.md)).
- **O momento.** Só o payload da partida, sem timeline: tropa aos 10 sim, ouro aos
  15 não ([P-3](../produto/perguntas-em-aberto.md)).
- **Uma nota.** De propósito: uma média das métricas seria o substituto de ranking
  que a política da Riot proíbe (C-7).

## 7. O cache

`profile.rs` busca as últimas 60 ranqueadas solo a cada atualização, espaçadas
1,3s para respeitar o limite da chave; a primeira carga leva ~80s em segundo
plano, depois só as partidas novas. Partidas que não contam ficam marcadas como
vistas (`mark_seen`) para não serem buscadas de novo.

Tudo vai para `%LOCALAPPDATA%\Presage\personal-v2.json`. O `personal.json` antigo
fica para trás de propósito: ele marca partidas como vistas sem a comparação de
rota, e reaproveitá-lo deixaria essas partidas sem rota para sempre. O formato
novo ainda lê o antigo (`lanes` é `default`), o que um teste trava
(`cache_antigo_carrega_sem_rotas`).

## 8. O que ainda não foi conferido

- Os nomes de campo de `challenges` contra uma partida real
  ([P-5](../produto/perguntas-em-aberto.md)). Um nome errado zera a métrica em
  silêncio.
- A compilação da casca: `riot.rs` (`match_detail`), `profile.rs` e o comando
  `lane_profile` foram escritos numa máquina sem WebKitGTK.
