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
- **Dono:** Riot.

## P-2 — O overlay com valor calculado ao vivo é aprovado? 🟡

- **Por que importa:** a política aprova *"game overlays that provide static data
  that is available prior to the game"*; o nosso mostra valor calculado durante a
  partida ([requisitos §1](requisitos.md#o-ponto-de-risco-o-overlay)).
- **Default:** o overlay existe, e todo o produto funciona sem ele. Perguntar no
  Developer Portal **no mesmo pedido da chave de produção**, antes de cobrar.
- **Dono:** Riot.

## P-3 — A timeline vale a chamada extra? 🟡

- **Por que importa:** a timeline dá ouro e experiência aos 10 e aos 15 minutos,
  exatos. Custa uma chamada por partida e corta pela metade a vazão do histórico
  pessoal.
- **Default:** não. O perfil de rota usa só o payload da partida, onde
  `laneMinionsFirst10Minutes` já dá tropa aos 10 sem a timeline. Reabrir com
  dados reais na mão.
- **Dono:** dado.

## P-4 — De onde calibrar o termo de experiência? 🟡

- **Por que importa:** sem ele o componente pessoal carrega a fase da conta
  ([ADR-0012](../adr/0012-vazamento-da-fase-da-conta-mantido.md)), e corrigir só
  a referência empata o principal de quem só joga um campeão com campeões nunca
  jogados.
- **Candidatos:** winrate por número de partidas no campeão, se o coletor passar
  a registrar isso; `champion-mastery-v4`.
- **Default:** vazamento mantido e documentado no código.
- **Dono:** dado.

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
- **Dono:** Riot.

## P-6 — Que chave pedir primeiro? 🟡

- **Por que importa:** a de desenvolvimento expira em 24h e o crawl leva 87h
  nela ([05 §6](../arquitetura/05-coletor.md#6-quanto-tempo-leva-de-verdade)). A
  **Personal** tem o mesmo limite e **não expira**: o mesmo crawl termina em ~3,6
  dias. É não comercial, o que serve para o protótipo.
- **Default:** Personal para o coletor e o histórico pessoal; produção quando
  houver produto para descrever (C-10).
- **Dono:** Riot.

## P-7 — A chance por rota soma duas coisas que se sobrepõem? 🟡

- **Por que importa:** `0,5 + W_DUEL × (duelo − 0,5) + matchup`. O duelo vem dos
  atributos; o matchup, de partidas. Quando a base existir, parte do que o duelo
  mede já estará no matchup. E o `W_DUEL = 0,60` foi escolhido, não ajustado.
- **Resolvido em parte (2026-10-28):** `lanes.rs` encolhia o matchup para 50%;
  agora encolhe para o que as duas forças explicam, e `from_data` fica verdadeiro
  quando a base entra no número.
- **Default:** fórmula atual, com a força e a referência de matchup do draft.
  `W_DUEL` e a sobreposição entre duelo e matchup se ajustam contra dado quando a
  base existir.
- **Dono:** dado.

## P-8 — Pinning do root CA da Riot no loopback 🟡

- **Por que importa:** a verificação de TLS está desligada para o LCU e para a
  porta 2999 ([07 §2](../arquitetura/07-seguranca.md#2-superfície)).
- **Default:** desligada só no loopback. Antes de release público, pinning do
  root CA da Riot em `lcu/client.rs` e na Live Client (`live.rs`). Cada uma tem o
  seu `TODO(segurança)`, adiado aqui em `tools/doccheck/deferred.txt` (P4).
- **Dono:** técnica.

## P-9 — Separar o perfil de um campeão por rota? 🔵

- **Por que importa:** o perfil soma todas as rotas de um campeão. Para quase
  todo mundo é a mesma; para pick flexível, mistura duas coisas.
- **Default:** soma.
- **Dono:** produto.

## P-10 — Somar os componentes em log-odds? 🟢

- **Resposta (2026-10-28):** sim. O ranking soma em log-odds; cada efeito entra
  como `logit(referência + δ) − logit(referência)`, e a composição pela mesma
  conversão a partir de 50%, o que preserva a escala em pontos dos pesos dela.
  Os testes anteriores passaram sem mudança
  ([02 §1](../arquitetura/02-motor-de-draft.md#1-rank)).
- **Dono:** técnica.

## P-12 — O cliente preenche o seu `championId` antes de travar? 🟡

- **Por que importa:** o tradutor do LCU monta `champion_id` com o
  `championId` do jogador quando ele vem preenchido, e `locked` só com a ação
  concluída. Se o cliente preenche durante o seu turno, a sua escolha entrava em
  `unavailable` e sumia da sua própria lista.
- **Default:** o contexto não marca como indisponível a sua escolha não travada —
  certo nos dois casos. Conferir num champ select real o que o LCU manda.
- **Dono:** Riot.

## P-13 — Quanto a inferência de rota do inimigo erra sem a base? 🟡

- **Por que importa:** sem a base, a rota sai das classes. Um lutador puro empata
  entre topo e selva, e o empate vai para a ordem de escolha; um campeão fora de
  rota (suporte no meio) é posto onde a classe manda. A prévia mostra o oponente
  errado nesses casos.
- **Default:** inferência pelas classes, `roles_assumed` avisando na tela. Com a
  base, `role_share` decide e o empate quase some.
- **Agora corrigível na tela:** a rota marcada à mão vence a inferência
  ([02 §0](../arquitetura/02-motor-de-draft.md#0-o-contexto-do-draft)).
- **Dono:** dado.

## P-14 — Que nome a Live Client dá a inibidor e torre do nexo hoje? 🟡

- **Por que importa:** o timer de respawn lê a estrutura pelo nome no evento
  (`InhibKilled`, `TurretKilled`). A documentação da Riot mostra
  `Barracks_T1_L1` e `Turret_T1_C_01_A`; apps em produção em 2026 leem
  `Inhib_TOrder_L2` e `Turret_TOrder_L1_P5`. Nome que nenhum dos dois formatos
  reconhece não vira timer — falha calada.
- **Default:** os dois formatos aceitos, com teste para cada
  (`timeline.rs`). Conferir numa partida real o que chega, e o mesmo para os
  campos da grade do champ select (`owned`, `freeToPlay`, `xboxGPReward`) e a
  fila em `/lol-gameflow/v1/session` (`gameData.queue.id`).
- **Dono:** Riot.

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
  constante só. Não são só os pesos: a completude também difere (`known / 4` no
  ranking, `known / 5` no `team_value`), e um mede o ganho marginal enquanto o
  outro mede o nível do desequilíbrio. Com só o lutador aliado e o mago deles
  travados, o ranking põe o tanque acima do mago e o guloso do minimax põe o mago
  acima do tanque ([RN-5](regras-e-pesos.md), [RN-11](regras-e-pesos.md)).
- **Dono:** técnica.

## P-15 — Escrever no cliente (runas, itens, feitiços, hover, ban)? 🟡

- **Pergunta:** o Presage deve mandar a runa e a build sugeridas para o cliente,
  ou fazer hover, travar e banir, como o iTero faz (I-19 a I-23)?
- **Por que importa:**
  - copiar a runa à mão é o atrito que o iTero tira;
  - em troca, cada escrita é uma ação no cliente que o jogador não fez;
  - travar e banir sozinho tira a decisão do jogador (C-5). Mandar runa e item é
    permitido, mas muda o que o produto é: de assistente para automatizador.
- **Default:** LCU só leitura ([ADR-0026](../adr/0026-lcu-so-leitura-e-autostart-desligado.md)).
  Travar e banir sozinho fica fora de qualquer jeito.
- **Dono:** produto.

## P-16 — Que harness de teste o front ganha? 🟢

- **Pergunta:** o front não tem teste automatizado. Que camada vem primeiro,
  componente (Vitest + Testing Library sobre os componentes puros, com os dados do
  `demo.tsx`) ou ponta a ponta (Playwright sobre o build de demonstração)?
- **Por que importa:** o catálogo do front ([frontend 06](../frontend/06-testes.md))
  está escrito e todo pendente. Um erro de escrita em disco engolido na tela, por
  exemplo, só se prova com um teste do front.
- **Default:** componentes primeiro, porque já são puros e o `demo.tsx` já tem os
  dados. Os TF-* ficam adiados até lá.
- **Resposta (2026-11-07):** componentes primeiro, com Vitest, Testing Library e
  jsdom (`vitest.config.ts`, `npm test`, também no `docs.yml`). O ID da linha vai
  no título do teste. TF-PARTIDA-3 foi o primeiro; os outros ficam adiados um a
  um em `tools/doccheck/deferred.txt` até o teste existir. Ponta a ponta
  (TF-OVERLAY-5) continua sem ferramenta.
- **Dono:** técnica.

## P-17 — Uma partida real, anonimizada, para o passo a passo? 🔵

- **Pergunta:** o passo a passo ([14](../arquitetura/14-uma-partida-passo-a-passo.md))
  percorre uma partida com os dados de exemplo do `demo.tsx`. Vale gravar uma
  partida real, trocar Riot IDs por nomes fictícios e versioná-la como fixture?
- **Por que importa:**
  - os dados de exemplo são escritos à mão e provam que a tela monta, não que a
    leitura acerta uma partida de verdade;
  - uma gravação real também fecharia parte do [P-5](#p-5--os-nomes-de-campo-de-challenges-batem-com-uma-partida-real-)
    e do [P-14](#p-14--que-nome-a-live-client-dá-a-inibidor-e-torre-do-nexo-hoje-).
- **Default:** dados de exemplo, rotulados como exemplo.
- **Dono:** produto, com uma partida do dono.

## P-18 — Ler o ouro em mão do jogador local? 🟡

- **Pergunta:** a Live Client entrega o seu ouro em mão (`currentGold`, em
  `/liveclientdata/activeplayer`), e a casca não lê esse endpoint: `live.rs` lê
  `playerlist`, `activeplayername`, `gamestats` e `eventdata`. Vale ler?
- **Por que importa:** "Você tem ~N de ouro para gastar" ao vivo (`gold.rs`) e o
  ouro parado do pós-jogo (`coach.rs`) são estimativas (`gold::earned_at`) de um
  número que daria para medir. Com a leitura, os dois passam de fraca a exata.
  A estimativa dos outros nove continua proibida na tela (C-4).
- **Default:** estimativa por `gold::earned_at`, para os dez.
- **Dono:** técnica.

## P-19 — A Live Client conta os campos da selva em `creepScore`? 🟡

- **Pergunta:** o caçador é cobrado em farm (`cobranca`), e tanto o pós-jogo
  quanto a sua rota ao vivo comparam só `creepScore` com o do outro caçador. Os
  campos entram nesse número?
- **Por que importa:** se não entram, o caçador é comparado pelas tropas que
  roubou das rotas, e o "campos contra campos" do [03 §8](../arquitetura/03-motor-da-partida.md#8-leitura-pós-partida)
  não existe.
- **Default:** compara o que a Live Client manda, assumindo que inclui os campos.
  Conferir numa partida real.
- **Dono:** Riot.

## P-20 — Como a inferência de rota desempata? 🟡

- **Pergunta:** no empate, a inferência fica com a primeira atribuição da busca:
  inimigos na ordem de escolha, rotas na ordem de `ROLES` (topo, selva, meio,
  bot, suporte). Com dois magos sem posição, o primeiro a escolher vai para o
  topo e o segundo fica no meio. É o que se quer? E o peso de rota
  (`lane_share`) divide o encaixe do inimigo pelas cinco rotas, mesmo as já
  tomadas: um assassino inferido no meio com a selva já informada para outro
  fica com peso 0,5, embora entre as rotas livres só possa ser do meio.
- **Por que importa:** o oponente de rota e o peso dele decidem o matchup do
  ranking e a prévia ([P-13](#p-13--quanto-a-inferência-de-rota-do-inimigo-erra-sem-a-base-)).
  O teste `o_empate_da_sempre_a_mesma_resposta` trava só que a resposta é
  sempre a mesma, não qual ela é.
- **Default:** o primeiro máximo da busca, e a fração sobre as cinco rotas.
- **Dono:** produto.

## P-21 — O `±` do ranking conta a incerteza de quem ainda vai escolher? 🟡

- **Pergunta:** sem oponente de rota, o peso de rota é 1 e a variância ainda soma
  o espalhamento inteiro do matchup. O contra e a sinergia só entram para quem já
  travou. Sem base, o `±` cresce conforme o draft avança. A incerteza do matchup
  futuro deve contar e a do contra futuro não?
- **Por que importa:** o `±` tem de contar a mesma história que a estimativa
  (RNF-2). Hoje ele é mais largo no começo pelo matchup e mais largo no fim pelo
  contra, por razões diferentes.
- **Default:** como está.
- **Dono:** técnica.

## P-22 — O ban filtra por rota aberta? 🟡

- **Pergunta:** `suggest_bans` pontua todo campeão do índice que não está
  indisponível. A resposta do minimax, ao contrário, só vem de quem joga uma rota
  que eles ainda não fecharam. O ban deveria seguir a mesma regra?
- **Por que importa:** com quatro deles travados, sugerir banir quem só joga uma
  rota já fechada é ban desperdiçado.
- **Default:** todos os campeões disponíveis.
- **Dono:** produto.

## P-23 — Até quando Arauto e Grubs ficam na lista, e com que nome chegam? 🟡

- **Pergunta:** Arauto e Grubs não tomados somem da lista 300 s depois de nascer
  (literal em `timeline.rs`, fora de `Timings`). Esse prazo é do jogo ou é
  estimativa nossa? E o abate de Grubs chega como `VoidgrubKill` ou como
  `HordeKill`?
- **Por que importa:**
  - um objetivo nascido e não morto fica com `seconds_left` 0: a tela escreve
    "Dragão em 0s (estimado)" até alguém matar, e a prioridade dá os 15 pontos
    inteiros;
  - o Ancião conta como dragão: depois dele a tela mostra um dragão a mais;
  - nome de evento que não reconhecemos não vira timer, calado (o mesmo da
    [P-14](#p-14--que-nome-a-live-client-dá-a-inibidor-e-torre-do-nexo-hoje-)).
- **Default:** 300 s; os dois nomes aceitos; o Ancião conta como dragão.
- **Dono:** Riot.

## P-24 — O histórico pessoal se atualiza só quando o app abre? 🟡

- **Pergunta:** `profile::refresh` roda uma vez, na subida do app. Uma partida
  jogada com o app aberto só entra no perfil de rota e no componente pessoal na
  abertura seguinte. Deve atualizar depois de cada partida?
- **Por que importa:** quem deixa o app aberto a noite toda vê o perfil de antes.
- **Default:** uma vez por abertura.
- **Dono:** técnica.

## P-25 — O histórico pessoal conta a flex? 🟡

- **Pergunta:** o histórico pessoal pede só a fila 420, solo/duo
  (`RANKED_SOLO_QUEUE`). A flex (440) entra?
- **Por que importa:** quem joga muito flex tem metade das partidas fora do
  perfil. Em troca, a flex é outra fila, com outro adversário médio.
- **Default:** só solo/duo.
- **Dono:** produto.

## P-26 — Para que serve `gravacao.json`? 🟡

- **Pergunta:** a gravação é escrita inteira no fim da partida, depois de a
  leitura já ter sido feita em memória, e nada no app a relê. Fica para depurar o
  pós-jogo, ou sai?
- **Por que importa:** o arquivo guarda o `displayName` dos dez, quadro a quadro,
  na máquina do jogador. Se é para depurar, o doc tem de dizer; se não é, ele só
  guarda dado de outras pessoas.
- **Default:** escrito e sobrescrito a cada partida.
- **Dono:** produto.

## P-27 — Os comandos pesados devem ser `async`? 🟡

- **Pergunta:** os 27 comandos são `fn` síncronos. `minimax_draft` com todo
  candidato, `lane_analysis` e `rune_plan` rodam a cada mudança de draft, e na partida
  `lane_analysis` roda a cada segundo. Comando síncrono do Tauri roda fora da
  thread da janela?
- **Por que importa:** se rodar na thread principal, uma conta lenta trava a
  janela (RNF-4).
- **Default:** síncronos. Não medido.
- **Dono:** técnica.

## P-28 — `--retomar` confere o recorte inteiro? 🟡

- **Pergunta:** `Estado::combina` confere região, tier e fila. Trocar `--patch`,
  `--min-jogos` ou `--por-jogador` no meio de uma coleta retomada é aceito. Deve
  ser recusado como o resto do recorte?
- **Por que importa:** uma base com metade das partidas por um critério e metade
  por outro sai sob um cabeçalho só.
- **Default:** trocar no meio é aceito.
- **Dono:** técnica.

## P-29 — O `price` de um item empilhável é por unidade? 🟡

- **Pergunta:** o ouro em itens soma um `price` por espaço do inventário e não lê
  quantas unidades há nele. Em poção e sentinela de controle, o `price` da Live
  Client já vem multiplicado?
- **Por que importa:** o ouro em itens é o número exato do produto (R-12). Se o
  preço é por unidade, ele erra para baixo com consumível empilhado.
- **Default:** um preço por espaço.
- **Dono:** Riot.

## P-30 — Que região o LCU dá para o Oriente Médio? 🟡

- **Pergunta:** `Region::from_lcu` não reconhece nenhuma grafia de Oriente Médio
  e cai em `br1`, embora a tabela de plataformas tenha `me1`. Que texto
  `/riot/v1/region-locale` manda para essa conta?
- **Por que importa:** região errada busca o histórico no servidor errado, e o
  erro só aparece como "componente pessoal desligado" no log (defeito no
  `build/audit.md`).
- **Default:** cai em `br1`.
- **Dono:** Riot.

## P-31 — Títulos por estatística: lacuna ou recusa? 🟡

- **Pergunta:** o iTero dá títulos ("The Inter", "Blind") pelo desempenho (I-52).
  A G-12 os listava como lacuna que cabe; a decisão derivada do cruzamento é
  rejeitar, pela RNF-11. Qual vale?
- **Por que importa:** título é nota em forma de apelido, e o perfil e o
  pós-jogo apontam, não rotulam (RNF-11, C-7). Aceitar muda a RNF-11.
- **Default:** rejeitar. A G-12 já não lista os títulos
  ([07 §6](../referencia/itero/07-cruzamento.md#6-as-lacunas-que-cabem-por-valor)).
- **Dono:** produto.

## P-32 — Quais lacunas viram requisito? 🟡

- **Pergunta:** cinco decisões do cruzamento pedem um requisito que R-1 a R-21
  não têm, e estão marcadas "R novo":
  - scouting dos dez (I-32), que pede chave de produção e o backend;
  - gráfico de PDL (I-57), que teria de ser gravado pelo LCU partida a partida;
  - idiomas (I-62);
  - tutorial de primeiro uso (I-65);
  - link de Discord e de relatório de bug (I-78), que espera existir um canal.
- **Por que importa:** sem requisito, cada uma fica em adiar e nada a cobra.
- **Default:** nenhum R novo; as cinco ficam em adiar.
- **Dono:** produto.

## P-33 — De onde vêm os cortes de duração? 🟡

- **Pergunta:** `Length::of` corta em 25:00 e 32:30. De onde saíram?
- **Por que importa:** a fase do jogo, com a base, compara partidas curtas com
  longas. Um corte fora da distribuição real deixa uma faixa quase vazia.
- **Default:** os dois, escolhidos à mão. Com a base, os quantis de
  `gameDuration` dela ([RN-14](regras-e-pesos.md)).
- **Dono:** dado.

## P-34 — A renda passiva vale no patch atual? 🟡

- **Pergunta:** a estimativa de ouro ganho usa renda passiva a partir de 1:50, a
  20,4 a cada 10 s (`PASSIVE_START`, `PASSIVE_PER_SECOND`). Os dois valem no
  patch de hoje?
- **Por que importa:** são a maior parcela do ouro estimado no começo da partida.
- **Default:** 110 s e 2,04 por segundo.
- **Dono:** Riot.

## P-35 — "Um estilo só" com dois core, ou só com os três? 🟡

- **Pergunta:** o retrato da pool marca ponto fraco quando todos os core têm a
  mesma classe principal, a partir de dois. A ADR-0016 falava em "os três". Qual
  vale?
- **Por que importa:** com dois escolhidos, o aviso já aparece antes do terceiro,
  e pode empurrar o jogador para longe do campeão que ele ia escolher.
- **Default:** dois bastam.
- **Dono:** produto.

## P-36 — A rota escolhida na tela morre com o champ select? 🟡

- **Pergunta:** a rota que você escolhe na tela para blind e normal
  (`set_draft_role`) nunca é limpa: o fim do champ select apaga as marcas e a
  lista da conta, e não ela. Deve morrer junto?
- **Por que importa:** na ranqueada a posição do cliente vence, então só pesa em
  blind e normal; lá, o próximo champ select começa na rota do anterior.
- **Default:** vale até fechar o app.
- **Dono:** produto.

## P-37 — Os itens sugeridos filtram por mapa? 🟡

- **Pergunta:** a carga de itens do Data Dragon não lê `maps`, e a build só filtra
  `purchasable` e custo de 1000 ou mais. Cópias de item de outro modo marcadas
  como compráveis entram na lista da Fenda?
- **Por que importa:** um item que não existe na partida no topo da build.
- **Default:** todo item comprável de 1000 ou mais.
- **Dono:** técnica.

## P-38 — Que corte de cor vale para uma chance? 🟡

- **Pergunta:** três componentes pintam uma chance com três cortes: 55/45 na rota
  e na escaramuça, 53/47 na curva de nível, 52/48 na chance do time. Qual é a
  regra?
- **Por que importa:** o mesmo 54% sai verde num lugar e neutro no outro. Um corte
  fixo também ignora o `±`: o certo seria pintar só quando o intervalo não cruza
  50%.
- **Default:** cada componente com o seu ([RN-33](regras-e-pesos.md)).
- **Dono:** produto.

## P-39 — O que o instalador leva além do app? 🟡

- **Pergunta:** o build do Vite sai com source maps (`sourcemap: true`) e com a
  página de demonstração (`demo.html`) como entrada, e a CSP aceita
  `style-src 'unsafe-inline'`. Os três vão para o instalador?
- **Por que importa:** source map e demonstração não servem ao jogador; o
  `unsafe-inline` pode não ser necessário, porque o React aplica `style={{}}` pelo
  CSSOM, que a CSP não barra. Não está conferido se o Tauri ou o Vite injetam
  `<style>`.
- **Default:** vão os três.
- **Dono:** técnica.

## P-40 — O pós-jogo pode tirar o jogador do meio de uma edição? 🟡

- **Pergunta:** quando a leitura pós-partida fica pronta, a janela troca para a
  aba Pós-jogo, esteja o jogador onde estiver. Deve trocar com ele no meio de uma
  edição na Pool?
- **Por que importa:** a troca é a hora certa para quem acabou de jogar e a hora
  errada para quem está montando a pool com a janela aberta.
- **Default:** troca sempre.
- **Dono:** produto.

## P-41 — Como a chance do time trata a rota incerta e a trava? 🔵

- **Pergunta:** três escolhas da chance do time que só importam com a base:
  - o seu assento entra com certeza 1 mesmo quando a sua rota veio da tela ou da
    pool, e não do cliente;
  - a força de cada um na rota entra inteira, sem o peso da certeza: um inimigo
    inferido no topo com fração 0,3 conta a força dele no topo por inteiro (só o
    matchup e o contra são divididos);
  - o `±` ignora a trava: com a chance presa em 65%, o `±` é o da soma sem trava.
- **Por que importa:** os três deixam a chance mais confiante do que o draft
  justifica.
- **Default:** como está.
- **Dono:** dado.
