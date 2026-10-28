# Perguntas em aberto

O que depende da Riot, de dado que ainda não existe ou de uma decisão de
produto. **Nenhuma bloqueia o trabalho:** cada uma tem um **default adotado**,
que fica valendo até a resposta. Quando uma for respondida, atualize o status e
o documento afetado no mesmo commit.

| Status | Significado |
|---|---|
| 🟡 aberta | ainda sem resposta |
| 🟢 respondida | resposta registrada e docs atualizados |
| 🔵 adiada | fora do horizonte atual |

## P-1 — O limite de taxa da Riot conta por região de roteamento? 🟡

- **Por que importa:** se `americas`, `europe`, `asia` e `sea` forem baldes
  separados, o coletor multiplica a vazão por até 4 coletando em várias regiões
  ao mesmo tempo. Com a chave de desenvolvimento, isso muda o crawl de "não
  termina" para "termina".
- **Hipótese:** a Riot documenta o limite de aplicação como contado por
  roteamento. O `limiter.rs` já separa plataforma e região por host.
- **Teste:** uma rajada contra duas regiões, comparando `X-App-Rate-Limit-Count`
  de cada uma. O limitador já lê esse cabeçalho.
- **Custo de estar certo:** uma base por região. O matchup provavelmente se
  transfere entre regiões; a winrate de meta, não.
- **Default:** um balde só, uma região.

## P-2 — O overlay com valor calculado ao vivo é aprovado? 🟡

- **Por que importa:** a política aprova *"game overlays that provide static data
  that is available prior to the game"*; o nosso mostra valor calculado durante a
  partida ([requisitos §1](requisitos.md#o-ponto-de-risco-o-overlay)).
- **Default:** o overlay existe, e todo o produto funciona sem ele. Perguntar no
  Developer Portal **no mesmo pedido da chave de produção**, antes de cobrar.

## P-3 — A timeline vale a chamada extra? 🟡

- **Por que importa:** a timeline dá ouro e experiência aos 10 e aos 15 minutos,
  exatos. Custa uma chamada por partida e corta pela metade a vazão do histórico
  pessoal.
- **Default:** não. O perfil de rota usa só o payload da partida, onde
  `laneMinionsFirst10Minutes` já dá tropa aos 10 sem a timeline. Reabrir com
  dados reais na mão.

## P-4 — De onde calibrar o termo de experiência? 🟡

- **Por que importa:** sem ele o componente pessoal carrega a fase da conta
  ([ADR-0012](../adr/0012-vazamento-da-fase-da-conta-mantido.md)), e corrigir só
  a referência empata o principal de quem só joga um campeão com campeões nunca
  jogados.
- **Candidatos:** winrate por número de partidas no campeão, se o coletor passar
  a registrar isso; `champion-mastery-v4`.
- **Default:** vazamento mantido e documentado no código.

## P-5 — Os nomes de campo de `challenges` batem com uma partida real? 🟡

- **Por que importa:** todo campo é `default`. Um nome errado não dá erro — zera
  a métrica em silêncio.
- **Estado:** o teste `le_os_nomes_de_campo_da_riot` trava os nomes que
  assumimos, não os que a Riot manda. A casca do perfil de rota já compila no
  Windows (CI, `cargo test --workspace`).
- **Tentativa (2026-10-28):** a chave de desenvolvimento recebida já não existia
  para a Riot (`401 Unknown apikey`) — chave de desenvolvimento morre em 24h ou
  ao ser regenerada. No caminho apareceu uma armadilha que imita chave errada
  ([L-21](../arquitetura/09-licoes-do-prototipo.md)).
- **Default:** os nomes do teste. **Primeira coisa a fazer com uma chave viva:**
  baixar uma partida real e passá-la por `history::played`.

## P-6 — Que chave pedir primeiro? 🟡

- **Por que importa:** a de desenvolvimento expira em 24h e o crawl leva 87h
  nela ([05 §6](../arquitetura/05-coletor.md#6-quanto-tempo-leva-de-verdade)). A
  **Personal** tem o mesmo limite e **não expira**: o mesmo crawl termina em ~3,6
  dias. É não comercial, o que serve para o protótipo.
- **Default:** Personal para o coletor e o histórico pessoal; produção quando
  houver produto para descrever (C-10).

## P-7 — A chance por rota soma duas coisas que se sobrepõem? 🟡

- **Por que importa:** `0,5 + W_DUEL × (duelo − 0,5) + matchup`. O duelo vem dos
  atributos; o matchup, de partidas. Quando a base existir, parte do que o duelo
  mede já estará no matchup. E o `W_DUEL = 0,60` foi escolhido, não ajustado.
- **Também:** `lanes.rs` encolhe o matchup para 50%, não para o que as bases dos
  dois preveem. Corrigir faz a base sozinha mexer no número, e aí `from_data =
  false` ("só a conta do duelo") deixa de ser verdade — é decisão de interface,
  não só de conta.
- **Default:** fórmula atual, com a força de matchup compartilhada com o draft.
  Ajustar contra dado quando a base existir.

## P-8 — Pinning do root CA da Riot no loopback 🟡

- **Por que importa:** a verificação de TLS está desligada para o LCU e para a
  porta 2999 ([07 §2](../arquitetura/07-seguranca.md#2-superfície)).
- **Default:** desligada só no loopback. Antes de release público, pinning do
  root CA da Riot em `lcu/client.rs`.

## P-9 — Separar o perfil de um campeão por rota? 🔵

- **Por que importa:** o perfil soma todas as rotas de um campeão. Para quase
  todo mundo é a mesma; para pick flexível, mistura duas coisas.
- **Default:** soma.

## P-10 — Somar os componentes em log-odds? 🟡

- **Por que importa:** o ranking soma base, matchup, sinergia, pessoal e
  composição em probabilidade e corta em [0,25; 0,75]. O certo seria log-odds,
  como a referência já é ([ADR-0008](../adr/0008-matchup-e-sinergia-contra-a-soma-das-forcas.md)).
- **Por que não ainda:** os pesos de composição estão calibrados em pontos
  percentuais (~1,5pp numa correção forte). Mudar a soma muda a escala deles.
- **Default:** probabilidade, enquanto a composição for o único componente com
  valor.

## P-12 — O cliente preenche o seu `championId` antes de travar? 🟡

- **Por que importa:** o tradutor do LCU monta `champion_id` com o
  `championId` do jogador quando ele vem preenchido, e `locked` só com a ação
  concluída. Se o cliente preenche durante o seu turno, a sua escolha entrava em
  `unavailable` e sumia da sua própria lista.
- **Default:** o contexto não marca como indisponível a sua escolha não travada —
  certo nos dois casos. Conferir num champ select real o que o LCU manda.

## P-13 — Quanto a inferência de rota do inimigo erra sem a base? 🟡

- **Por que importa:** sem a base, a rota sai das classes. Um lutador puro empata
  entre topo e selva, e o empate vai para a ordem de escolha; um campeão fora de
  rota (suporte no meio) é posto onde a classe manda. A prévia mostra o oponente
  errado nesses casos.
- **Default:** inferência pelas classes, `roles_assumed` avisando na tela. Com a
  base, `role_share` decide e o empate quase some.

## P-11 — Dois conjuntos de pesos de composição? 🟡

- **Por que importa:** `rank()` usa `W_DAMAGE_BALANCE = 0,050` e
  `W_FRONTLINE = 0,016`; `minimax.rs` usa `W_BALANCE = 0,06` e
  `W_FRONTLINE = 0,025`. Os dois são priors escritos à mão para o mesmo efeito, e
  a ordem do guloso e a do minimax podem discordar por isso, não por causa da
  resposta adversária. É a classe de defeito de [L-4](../arquitetura/09-licoes-do-prototipo.md).
- **Por que pode ser intencional:** `rank()` mede o encaixe marginal de um
  candidato; `team_value` mede o valor de um time inteiro, e cobra frontline só a
  partir do terceiro pick.
- **Default:** os dois como estão. Decidir se são a mesma crença; se forem, uma
  constante só.
