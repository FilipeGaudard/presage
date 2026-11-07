# Requisitos

Três tipos. As regras da Riot (**C-\***) não se negociam: violar uma é perder a
chave de API, e sem chave não há produto. Os requisitos não funcionais
(**RNF-\***) são escolhas do produto, e cada um aponta a decisão que o garante.
Os requisitos funcionais (**R-\***) dizem o que o Presage faz, cada um com
critérios de aceite que um teste consegue provar. A matriz do §4 liga os três às
funções ([11](../arquitetura/11-catalogo-de-funcoes.md)) e aos testes
([13](../arquitetura/13-testes.md)) que os atendem.

## 1. Regras da Riot (C)

O Presage lê a LCU local e a Live Client Data API oficial. Fonte: a
[política de League](https://support-developer.riotgames.com/hc/en-us/articles/22698698001939-League-of-Legends)
e os anúncios citados.

| ID | Regra | O que significa aqui |
|---|---|---|
| C-1 | **Nada de timers de ultimate do inimigo** — proibido desde 13/03/2025, sob pena de desativação da chave. Inclusive os manuais | Nenhum timer de habilidade inimiga, nem com o jogador apertando o botão |
| C-2 | **Nada de leitura de memória do jogo** — bloqueado por padrão desde 06/10/2026 para apps não aprovados | Só APIs oficiais: LCU, Live Client, Riot Web API |
| C-3 | **Nada de injeção nem automação de input** | O overlay é janela externa transparente, compatível com o Vanguard ([ADR-0003](../adr/0003-overlay-como-janela-externa.md)) |
| C-4 | *"Products may not provide any game-session-specific information that would be previously unknown to the player"* | Tudo que aparece durante a partida deriva do que já está na tela: itens na aba de pontuação, níveis, abates, eventos. Nada de posição no mapa, cooldown inimigo ou ouro em mão alheia. Um teste varre as razões de prioridade procurando linguagem de posição |
| C-5 | *"Products should not remove game decisions, but may highlight decisions that are important and give multiple choices"*; "apps that dictate player decisions" está entre os **não** aprovados | O ranking mostra alternativas e motivo em texto, em vez de mandar num pick só |
| C-6 | *"Products should increase, and not decrease the diversity of game decisions"* | A build adaptativa varia com a composição inimiga, em vez de empurrar a mesma build do patch para todo mundo |
| C-7 | *"Products cannot create alternatives for official skill ranking systems... Prohibited alternatives include MMR or ELO calculators"* | A leitura pós-partida aponta erros concretos e **não** vira nota, rating ou escada paralela. O perfil de rota não tem nota única ([04 §6](../arquitetura/04-perfil-de-rota.md#6-o-que-ele-não-separa)) |
| C-8 | Cobrar exige produto registrado com status **Approved** ou **Acknowledged**, **camada grátis obrigatória** e conteúdo pago **transformativo** ("nova informação, nova estética, novos insights") | O plano pago é adicional, nunca o único. Formas aceitas: assinatura, doação, financiamento coletivo, inscrição de torneio. Proibido: aposta, jogo de azar, cripto, preço abusivo (quem julga é a Riot) |
| C-9 | Anúncio **dentro do jogo não é permitido**, e isso inclui overlay | Se a camada grátis for sustentada por anúncio, ele fica no painel ou no site, nunca por cima da partida |
| C-10 | Produto público **não** pode rodar com chave de desenvolvimento | Antes de distribuir, chave de produção ([00 §4](../arquitetura/00-contexto-e-limites.md#4-riot-web-api)) |

### O ponto de risco: o overlay

Entre os casos aprovados para chave de produção está *"game overlays that provide
static data that is available prior to the game"* — mais estreito do que o nosso
overlay, que mostra valor calculado ao vivo. Outras ferramentas fazem isso, mas
precedente não é permissão. Vale perguntar à Riot pelo Developer Portal antes de
cobrar, e vale ter o produto funcionando sem o overlay caso a resposta seja não
([P-2](perguntas-em-aberto.md)).

## 2. Requisitos não funcionais (RNF)

| ID | Requisito | Garantido por |
|---|---|---|
| RNF-1 | **Nenhum número inventado.** Componente sem amostra vale exatamente zero, e a interface diz de onde o número veio (`from_data`, "sem base estatística ainda") | [ADR-0007](../adr/0007-sem-dado-o-componente-vale-zero.md); `sem_amostra_o_desvio_e_zero`, `sem_base_de_partidas_o_numero_vem_so_do_duelo` |
| RNF-2 | **O `±` conta a mesma história que a estimativa**, e encolhe com a raiz da amostra | [ADR-0009](../adr/0009-forca-do-shrinkage-derivada-do-desvio.md); `forca_e_desvio_sao_a_mesma_crenca`, `desvio_posterior_cai_com_a_raiz_da_amostra` |
| RNF-3 | **Um efeito é medido contra o que as forças já explicam**, nunca contra a base de um lado só | [ADR-0008](../adr/0008-matchup-e-sinergia-contra-a-soma-das-forcas.md); `popularidade_nao_vira_matchup`, `sinergia_nao_herda_a_forca_do_aliado` |
| RNF-4 | **Tempo real para uma fase de 30 segundos.** Draft lido a cada 350 ms; minimax podado | [ADR-0002](../adr/0002-polling-em-vez-de-websocket.md); [02 §6](../arquitetura/02-motor-de-draft.md#6-draft-com-resposta) |
| RNF-5 | **Funciona sem chave.** Tudo menos histórico pessoal e base agregada | [00 §5](../arquitetura/00-contexto-e-limites.md#5-o-que-funciona-sem-chave) |
| RNF-6 | **A chave da Riot nunca vai para binário, repositório, argumento ou URL** | [ADR-0004](../adr/0004-chave-da-riot-so-no-backend.md); [07 §3](../arquitetura/07-seguranca.md#3-a-chave-da-riot) |
| RNF-7 | **A atualização nunca instala sozinha** e falha calada; só executa o que a chave privada assinou | [ADR-0005](../adr/0005-atualizador-assinado-com-endpoint-permanente.md) |
| RNF-8 | **Nada sai da máquina do jogador.** A gravação e a leitura pós-partida são locais | [03 §8](../arquitetura/03-motor-da-partida.md#8-leitura-pós-partida) |
| RNF-9 | **O core testa em segundos, sem MSVC e sem Tauri** | [ADR-0001](../adr/0001-core-puro-separado-da-casca.md) |
| RNF-10 | **O executável não depende do Visual C++ Redistributable** | [ADR-0014](../adr/0014-crt-estatica-no-windows.md) |
| RNF-11 | **O perfil e o pós-jogo apontam, não dão nota** | C-7; [ADR-0011](../adr/0011-perfil-de-rota-e-razao-nao-peso.md) |
| RNF-12 | **A cor da marca nunca carrega significado de dado** | [identidade](identidade.md) |

## 3. Requisitos funcionais (R)

De onde vieram: o que o Presage já faz ([visão geral §1](visao-geral.md#1-o-que-é)),
as lacunas fechadas frente ao iTero (G-1 a G-8) e as decisões do
[cruzamento](../referencia/itero/07-cruzamento.md). Cada critério é uma frase que
um teste prova ou desmente; o teste está na matriz.

### R-1 — Ranking do draft pela sua rota

O Presage ranqueia, a cada leitura do champ select, os campeões que você pode
escolher para a sua rota, com probabilidade, `±` e motivo
([02 §1](../arquitetura/02-motor-de-draft.md#1-rank)).

**Critérios de aceite:**
- Os candidatos jogam a sua rota ou estão na sua pool; quem não joga a rota e não
  está na pool fica de fora ([L-23](../arquitetura/09-licoes-do-prototipo.md)).
- Banido, escolhido e hover de aliado não aparecem; a sua escolha ainda não
  travada continua sua ([L-26](../arquitetura/09-licoes-do-prototipo.md)).
- A ordem é determinística: `winrate`, depois pool, depois id. A pool marca o
  campeão e só desempata: fora do empate exato, não muda a ordem.
- Cada candidato traz até três motivos em texto, e nenhum número sem origem
  (RNF-1).

### R-2 — Resposta adversária

Para cada candidato, o valor do time **depois** da melhor resposta deles, e a
diferença para o guloso: a exposição do pick
([02 §6](../arquitetura/02-motor-de-draft.md#6-draft-com-resposta)).

**Critérios de aceite:**
- A resposta varre todos os campeões que jogam uma rota inimiga ainda livre
  ([L-24](../arquitetura/09-licoes-do-prototipo.md)).
- O resultado vem ordenado pelo valor depois da resposta, com alternativas, nunca
  um pick só (C-5).

### R-3 — Sugestão de ban, com a sua lista na frente

**Critérios de aceite:**
- A sua lista de bans (até cinco, em ordem) vem na frente desde o primeiro ban,
  sem quem já saiu e sem quem um aliado quer pegar.
- O resto sai de quanto o campeão completa a composição deles, sem repetir a
  lista; sem nenhum pick deles e sem base, a sugestão é vazia
  ([ADR-0013](../adr/0013-ban-nao-personalizado-enquanto-aditivo.md)).
- O app nunca bane sozinho.

### R-4 — Rota do inimigo presumida e corrigível

**Critérios de aceite:**
- Sem posição informada, a rota do inimigo sai da atribuição mais plausível das
  rotas livres, uma vez, no contexto ([L-25](../arquitetura/09-licoes-do-prototipo.md)).
- A marca à mão vence a inferência e nunca a posição do cliente; duas marcas na
  mesma rota ficam com a do primeiro a escolher; as marcas morrem com o champ
  select.

### R-5 — Só quem a sua conta pode escolher

**Critérios de aceite:**
- Os candidatos se limitam à grade do champ select; a rotação grátis não vale na
  ranqueada, e fila desconhecida fica sem ela.
- Grade vazia ou indisponível não tira ninguém.
- A preferência "incluir quem você não tem" desliga o filtro, e o simulador o
  ignora.

### R-6 — Prévia do candidato

Clicar num candidato mostra a rota contra o oponente nível a nível, as
escaramuças 2v2 e 3v3 e a condição de vitória, com ele no seu lugar.

**Critérios de aceite:**
- A prévia é a mesma conta de quando ele estiver travado.
- A curva vai do nível 1 ao 18 pela fórmula de crescimento do jogo, e diz que não
  vê habilidades.
- A condição de vitória é descrição por regra, nunca ordem (C-5,
  [ADR-0018](../adr/0018-o-que-a-condicao-de-vitoria-pode-afirmar.md)).

### R-7 — Runas do confronto

**Critérios de aceite:**
- A runa principal sai de regras mecânicas do confronto e diz que não é
  estatística (`from_data = false`).
- Runa referenciada pela chave textual do Data Dragon, nunca pelo id
  ([L-17](../arquitetura/09-licoes-do-prototipo.md)).

### R-8 — Simulador de draft

**Critérios de aceite:**
- Enquanto a aba está aberta, ranking, resposta, bans e prévia leem o draft
  montado à mão no lugar do cliente; fechar a aba volta para o cliente.
- O motor refaz indisponíveis e a sua rota a partir do draft montado
  (`DraftState::normalized`), sem depender da interface.

### R-9 — Construtor de pool

**Critérios de aceite:**
- Por rota, três escolhidos pelo jogador, com pontos fortes e fracos.
- Os cinco lugares restantes são recomendados um por vez, cada um pelo quanto
  sobe a cobertura, recalculada a cada escolha; só entra quem joga a rota
  ([ADR-0016](../adr/0016-pool-de-oito-tres-escolhidos-cinco-por-cobertura.md)).
- Nada se preenche sozinho; com oito, não há mais sugestão.

### R-10 — Build adaptativa

**Critérios de aceite:**
- Cada item candidato vale o quanto sobe o seu poder de combate, por ouro, contra
  os itens que eles têm agora.
- Time inimigo de dano físico puxa armadura acima de resistência mágica; o
  inimigo comprar armadura sobe penetração na sua lista.
- A ordem de penetração e o multiplicador de resistência são os do jogo.

### R-11 — Rotas e escaramuças ao vivo

Você contra o oponente direto na métrica que a sua rota cobra, a chance por rota
e as escaramuças que importam para a sua rota.

**Critérios de aceite:**
- O suporte é comparado por visão por minuto, nunca por tropas; os outros, por
  tropas por minuto ([L-27](../arquitetura/09-licoes-do-prototipo.md)).
- Atrás, igual ou à frente sai do ouro em itens, que é exato; antes de 1:30 não há
  painel.
- Sem base, a chance por rota vem só do duelo e diz isso (RNF-1).

### R-12 — Ouro dos dez

**Critérios de aceite:**
- Ouro investido é a soma do preço de cada item no inventário: exato.
- Ouro em mão de outro jogador nunca aparece (C-4).

### R-13 — Timers de objetivo, estruturas e buffs

**Critérios de aceite:**
- Respawn observado é soma exata a partir do abate visto; primeiro spawn é
  estimado e a tela diferencia os dois.
- Inibidor e torre do nexo viram contagem até a volta, nos dois formatos de nome;
  nome desconhecido não vira timer.
- O buff de Barão e de Ancião é de quem estava vivo no abate e some de quem morre.
- Nenhum timer de habilidade inimiga (C-1).

### R-14 — Onde está o valor no mapa

**Critérios de aceite:**
- A saída é sempre "esta rota vale mais agora, por estes motivos", nunca onde o
  inimigo está; nenhuma razão usa linguagem de posição (C-4).

### R-15 — Overlay

**Critérios de aceite:**
- Janela externa, transparente, sempre no topo e atravessável pelo mouse até o
  jogador pedir ([ADR-0003](../adr/0003-overlay-como-janela-externa.md)).
- O jogador escolhe o que aparece; a escolha sobrevive a reiniciar o app.
- Nenhum anúncio (C-9).

### R-16 — Leitura pós-partida

**Critérios de aceite:**
- A partida é gravada localmente, um quadro por segundo, e lida no fim; nada sai
  da máquina (RNF-8).
- Cada observação é um número que o jogador pode conferir na aba de pontuação, e
  nada vira nota (C-7).
- A rota decide o que se cobra.

### R-17 — Perfil de rota contra o oponente direto

**Critérios de aceite:**
- Cada partida é comparada com o oponente direto em cinco métricas, encolhida por
  nível ([04](../arquitetura/04-perfil-de-rota.md)).
- Só ganha cor o que é claro a 95%; não há nota única (C-7).
- Remake e partida curta não contam ([L-5](../arquitetura/09-licoes-do-prototipo.md)).

### R-18 — Conexão com o cliente

**Critérios de aceite:**
- O app acha o cliente pelo lockfile, sem nada além da porta e do token locais.
  (`lockfile::parse_cmdline` existe no core, mas a casca não o usa.)
- O ritmo de leitura segue a fase: 350 ms no champ select, 1 s na partida,
  1,5 s com o cliente parado, 3 s sem cliente (RNF-4).
- O LCU é só lido, nunca escrito.

### R-19 — Instalação, atualização e início com o Windows

**Critérios de aceite:**
- A atualização só instala o que a chave privada assinou, e nunca sozinha (RNF-7).
- Iniciar com o Windows vem desligado até o jogador ligar.

### R-20 — Coletor da base de partidas

**Critérios de aceite:**
- Coleta por patch e partida inteira, sem bola de neve
  ([ADR-0006](../adr/0006-coletor-sem-bola-de-neve-um-patch-partida-inteira.md)).
- Retoma de onde parou, e a chave sai só de variável de ambiente (RNF-6).
- Os cortes de partida que o histórico pessoal também faz (modo, terminada,
  remake, curta) são os mesmos nos dois. Hoje eles estão escritos duas vezes e
  divergem na rota em branco ([05 §5](../arquitetura/05-coletor.md#5-a-duplicação-com-historyrs)).
  O defeito está registrado no `build/audit.md`.

### R-21 — A base de partidas no número

Com a base, acendem winrate base, matchup, contra, sinergia, chance do time e
fase do jogo.

**Critérios de aceite:**
- Cada efeito é medido contra o que as forças já explicam (RNF-3) e encolhido
  pela amostra (RNF-2).
- Sem a base, cada componente vale exatamente zero e a chance do time não
  aparece (RNF-1).

## 4. Matriz de rastreabilidade

Cada requisito, as funções principais que o atendem ([11](../arquitetura/11-catalogo-de-funcoes.md))
e os testes que o provam ([13](../arquitetura/13-testes.md)). O `doccheck` falha
se um requisito não tiver linha aqui, ou se a linha citar um teste que o catálogo
não tem.

| Requisito | Funções principais | Testes |
|---|---|---|
| R-1 — Ranking do draft pela sua rota | `to_draft_state`, `rank`, `rank_with`, `resolve_role`, `plays`, `profile`, `damage_balance_gain`, `frontline_need` e mais 17 | T-DRAFT-4, T-DRAFT-5, T-DRAFT-8, T-DRAFT-10, T-DRAFT-11, T-DRAFT-12, T-DRAFT-13, T-DRAFT-14, T-DRAFT-15, T-DRAFT-16, T-DRAFT-17, T-DRAFT-18, T-DRAFT-19, T-DRAFT-20, T-DRAFT-25, T-DRAFT-26, T-DRAFT-27, T-DRAFT-28, T-DRAFT-29, T-DRAFT-37, T-DRAFT-38, T-DRAFT-40, T-DRAFT-41, T-DRAFT-42, T-DRAFT-61, T-DRAFT-62, T-ENTRADA-6, T-ENTRADA-7, T-ENTRADA-8, T-ENTRADA-9, T-ENTRADA-35, T-ENTRADA-39, T-JOGO-1, T-JOGO-2, T-JOGO-5, T-PERFIL-8, T-STATS-10 |
| R-2 — Resposta adversária | `others_play`, `team_value`, `minimax`, `minimax_with`, `open_enemy_roles`, `enemy_ids`, `ids` | T-DRAFT-43, T-DRAFT-44, T-DRAFT-45, T-DRAFT-46, T-DRAFT-47, T-DRAFT-48, T-DRAFT-49, T-DRAFT-50, T-DRAFT-51, T-DRAFT-52, T-DRAFT-64 |
| R-3 — Sugestão de ban, com a sua lista na frente | `team_value`, `suggest_bans`, `bans_with_pool` | T-DRAFT-53, T-DRAFT-54, T-DRAFT-55, T-DRAFT-56, T-DRAFT-57, T-DRAFT-58, T-DRAFT-59, T-ENTRADA-6 |
| R-4 — Rota do inimigo presumida e corrigível | `role_share`, `role_fit`, `seats`, `new_with`, `from_lcu` | T-CASCA-3, T-DRAFT-6, T-DRAFT-13, T-DRAFT-22, T-DRAFT-23, T-DRAFT-24, T-DRAFT-30, T-DRAFT-31, T-DRAFT-32, T-DRAFT-33, T-DRAFT-39, T-PREVIA-5, T-PREVIA-13, TF-DRAFT-10 |
| R-5 — Só quem a sua conta pode escolher | `selectable`, `new_with` | T-DRAFT-34, T-DRAFT-35, T-ENTRADA-11, T-ENTRADA-12 |
| R-6 — Prévia do candidato | `profile`, `estimate`, `analyze`, `build`, `from_context`, `grown`, `lane_outlooks`, `skirmishes` e mais 8 | T-CASCA-1, T-CASCA-2, T-CASCA-4, T-CASCA-5, T-CASCA-6, T-CASCA-7, T-DRAFT-36, T-JOGO-3, T-JOGO-4, T-PREVIA-6, T-PREVIA-7, T-PREVIA-10, T-PREVIA-11, T-PREVIA-12, T-PREVIA-13, T-PREVIA-14, T-ROTA-19, T-ROTA-20, T-ROTA-21, T-ROTA-22, T-ROTA-23, T-ROTA-25, TF-DRAFT-10, TF-ROTAS-3 |
| R-7 — Runas do confronto | `plan`, `RuneIndex::get`, `RuneIndex::is_empty` | T-RUNAS-1, T-RUNAS-2, T-RUNAS-3, T-RUNAS-4, T-RUNAS-5, T-RUNAS-6, T-RUNAS-7, T-RUNAS-8, T-RUNAS-9, T-RUNAS-10, TF-ROTAS-2, TF-ROTAS-3 |
| R-8 — Simulador de draft | `normalized` | T-ENTRADA-13 |
| R-9 — Construtor de pool | `plays`, `robustness`, `analyze`, `suggest`, `weight`, `is_empty`, `ids`, `ids_for_role` e mais 5 | T-CASCA-21, T-DRAFT-38, T-DRAFT-42, T-ENTRADA-33, T-ENTRADA-34, T-ENTRADA-35, T-ENTRADA-36, T-ENTRADA-37, T-ENTRADA-38, T-ENTRADA-39, T-POOL-1, T-POOL-2, T-POOL-3, T-POOL-4, T-POOL-5, T-POOL-6, T-POOL-7, T-POOL-8, T-POOL-9, T-POOL-10, T-POOL-11, T-POOL-12, T-POOL-13 |
| R-10 — Build adaptativa | `to_live_game`, `grown`, `parse_stats`, `effective_resist`, `damage_multiplier`, `suggest`, `loadout`, `incoming_physical_share` e mais 15 | T-BUILD-1, T-BUILD-2, T-BUILD-3, T-BUILD-4, T-BUILD-5, T-BUILD-6, T-BUILD-7, T-BUILD-8, T-BUILD-9, T-BUILD-10, T-ENTRADA-20, T-JOGO-1, T-JOGO-3, T-JOGO-4, T-JOGO-6, T-JOGO-7, T-JOGO-8, T-JOGO-9, T-JOGO-10, T-JOGO-11, T-JOGO-12, T-JOGO-13, T-JOGO-14 |
| R-11 — Rotas e escaramuças ao vivo | `to_live_game`, `champion_key`, `analyze`, `build`, `report`, `standing`, `lane_outlooks`, `skirmishes` e mais 8 | T-CASCA-1, T-ENTRADA-14, T-ENTRADA-15, T-ENTRADA-16, T-ENTRADA-18, T-ENTRADA-47, T-JOGO-5, T-RIVAL-1, T-RIVAL-2, T-RIVAL-3, T-RIVAL-4, T-RIVAL-5, T-RIVAL-6, T-ROTA-1, T-ROTA-2, T-ROTA-3, T-ROTA-5, T-ROTA-6, T-ROTA-7, T-ROTA-8, T-ROTA-9, T-ROTA-10, T-ROTA-11, T-ROTA-12, T-ROTA-13, T-ROTA-19, T-ROTA-20, T-ROTA-23, T-ROTA-24, T-ROTA-25, T-ROTA-26 |
| R-12 — Ouro dos dez | `to_live_game`, `earned_at`, `report`, `gold_invested` | T-ENTRADA-15, T-ENTRADA-17, T-ENTRADA-19, T-ENTRADA-47, T-OURO-1, T-OURO-2, T-OURO-3, T-OURO-4, T-OURO-5, T-OURO-6, T-OURO-7, T-OURO-8, T-OURO-9, T-OURO-10, TF-PARTIDA-3 |
| R-13 — Timers de objetivo, estruturas e buffs | `build`, `update` | T-OBJETIVO-1, T-OBJETIVO-2, T-OBJETIVO-3, T-OBJETIVO-4, T-OBJETIVO-5, T-OBJETIVO-6, T-OBJETIVO-7, T-OBJETIVO-8, T-OBJETIVO-9, T-OBJETIVO-10, T-OBJETIVO-11, T-OBJETIVO-12, T-OBJETIVO-13, T-OBJETIVO-14, T-OBJETIVO-15, T-OBJETIVO-16, T-OBJETIVO-17, T-OBJETIVO-18, T-PRIORIDADE-8 |
| R-14 — Onde está o valor no mapa | `report` | T-PRIORIDADE-1, T-PRIORIDADE-2, T-PRIORIDADE-3, T-PRIORIDADE-4, T-PRIORIDADE-5, T-PRIORIDADE-6, T-PRIORIDADE-7, T-PRIORIDADE-8, T-PRIORIDADE-9, T-PRIORIDADE-10, T-PRIORIDADE-11 |
| R-15 — Overlay | `focus`, `label`, `detail`, `shows`, `set`, `anything_visible`, `visible` | T-CASCA-21, T-ENTRADA-40, T-ENTRADA-41, T-ENTRADA-42, T-ENTRADA-43, T-ENTRADA-44, T-ENTRADA-45, T-ROTA-13, T-ROTA-14, T-ROTA-15, T-ROTA-16, T-ROTA-17, T-ROTA-18 |
| R-16 — Leitura pós-partida | `review`, `worth_reviewing`, `outcome`, `push`, `duration`, `is_empty` | T-CASCA-15, T-CASCA-16, T-CASCA-17, T-CASCA-18, T-CASCA-19, T-POS-1, T-POS-2, T-POS-3, T-POS-4, T-POS-5, T-POS-6, T-POS-7, T-POS-8, T-POS-9, T-POS-10, T-POS-11, T-POS-12, T-POS-13, T-POS-14, T-POS-15, T-POS-16, T-POS-17, T-POS-18, T-POS-19, T-POS-20, T-POS-21, T-POS-22, T-POS-23, T-POS-24 |
| R-17 — Perfil de rota contra o oponente direto | `played`, `build`, `higher_is_better`, `phrase`, `from_lcu`, `split_riot_id`, `urlencode`, `add_played` e mais 2 | T-DRAFT-21, T-ENTRADA-21, T-ENTRADA-22, T-ENTRADA-23, T-ENTRADA-24, T-ENTRADA-25, T-ENTRADA-26, T-ENTRADA-27, T-ENTRADA-28, T-ENTRADA-29, T-ENTRADA-30, T-ENTRADA-31, T-PERFIL-1, T-PERFIL-2, T-PERFIL-3, T-PERFIL-4, T-PERFIL-5, T-PERFIL-6, T-PERFIL-7, T-STATS-11, T-STATS-12, T-STATS-13 |
| R-18 — Conexão com o cliente | `parse_lockfile`, `parse_cmdline`, `extract_arg`, `to_draft_state`, `base_url` | T-CASCA-8, T-CASCA-9, T-CASCA-10, T-CASCA-11, T-CASCA-12, T-CASCA-20, T-ENTRADA-1, T-ENTRADA-2, T-ENTRADA-3, T-ENTRADA-4, T-ENTRADA-5, T-ENTRADA-10, T-ENTRADA-46 |
| R-19 — Instalação, atualização e início com o Windows | — | T-CASCA-13, T-CASCA-14 |
| R-20 — Coletor da base de partidas | `played`, `from_platform`, `platforms` | T-COLETOR-1, T-COLETOR-2, T-COLETOR-3, T-COLETOR-4, T-COLETOR-5, T-COLETOR-6, T-COLETOR-7, T-COLETOR-8, T-COLETOR-9, T-COLETOR-10, T-COLETOR-12, T-COLETOR-13, T-COLETOR-14, T-COLETOR-20, T-COLETOR-21, T-COLETOR-22, T-COLETOR-23, T-COLETOR-24, T-COLETOR-25, T-COLETOR-27, T-COLETOR-30, T-COLETOR-31, T-COLETOR-32, T-COLETOR-33, T-COLETOR-34, T-COLETOR-35, T-COLETOR-36, T-COLETOR-37, T-ENTRADA-25, T-ENTRADA-32, T-STATS-19 |
| R-21 — A base de partidas no número | `rank_with`, `role_share`, `estimate`, `analyze`, `from_context`, `shrink`, `Length::of`, `Dataset::is_usable` e mais 8 | T-COLETOR-3, T-COLETOR-15, T-COLETOR-16, T-COLETOR-17, T-COLETOR-18, T-COLETOR-19, T-COLETOR-26, T-COLETOR-27, T-COLETOR-28, T-COLETOR-29, T-DRAFT-1, T-DRAFT-2, T-DRAFT-3, T-DRAFT-5, T-DRAFT-12, T-DRAFT-40, T-DRAFT-41, T-PREVIA-1, T-PREVIA-2, T-PREVIA-3, T-PREVIA-4, T-PREVIA-8, T-PREVIA-9, T-ROTA-3, T-ROTA-4, T-STATS-2, T-STATS-14, T-STATS-15, T-STATS-17, T-STATS-18, T-STATS-19 |
| C-1 — Nada de timer de ultimate inimiga | `build` | T-OBJETIVO-17 |
| C-2 — Nada de leitura de memória do jogo | — | — |
| C-3 — Nada de injeção nem automação de input | — | — |
| C-4 — Nada que o jogador não saberia | `report`, `build`, `gold_invested`, `update` | T-ENTRADA-17, T-OBJETIVO-4, T-OBJETIVO-11, T-OURO-10, T-PRIORIDADE-7, T-ROTA-12, TF-PARTIDA-3 |
| C-5 — Destacar decisões, nunca tirá-las | `minimax_with`, `bans_with_pool`, `analyze`, `suggest`, `plan`, `report` | T-DRAFT-46, T-POOL-12, T-PREVIA-11, T-PRIORIDADE-2, T-RUNAS-7 |
| C-6 — Aumentar a diversidade de decisões | `plan`, `suggest`, `incoming_physical_share` | T-BUILD-1, T-BUILD-2, T-BUILD-4, T-BUILD-5, T-RUNAS-3 |
| C-7 — Nada de MMR, nota ou escada paralela | `build`, `review` | T-PERFIL-9, T-POS-25 |
| C-8 — Cobrar só aprovado e com camada grátis | — | — |
| C-9 — Nada de anúncio dentro do jogo | — | TF-OVERLAY-6 |
| C-10 — Produto público só com chave de produção | — | — |
| RNF-1 — Nenhum número inventado | `rank_with`, `estimate`, `analyze`, `suggest`, `plan`, `shrink`, `shrunk_delta`, `Dataset::is_usable` e mais 4 | T-CASCA-5, T-DRAFT-8, T-DRAFT-10, T-DRAFT-11, T-ENTRADA-19, T-OBJETIVO-1, T-OBJETIVO-2, T-OBJETIVO-14, T-POOL-12, T-POS-2, T-POS-3, T-PREVIA-1, T-PREVIA-4, T-PREVIA-8, T-PREVIA-10, T-PREVIA-14, T-PRIORIDADE-8, T-ROTA-2, T-ROTA-3, T-ROTA-26, T-RUNAS-1, T-RUNAS-7, T-RUNAS-8, T-STATS-1, T-STATS-8, T-STATS-9, T-STATS-16, T-STATS-18, TF-DRAFT-7, TF-ROTAS-2 |
| RNF-2 — O `±` conta a mesma história que a estimativa | `rank_with`, `estimate`, `Estimate::se`, `strength_for_sd`, `posterior_sd` | T-DRAFT-7, T-DRAFT-9, T-PREVIA-3, T-STATS-3, T-STATS-5, T-STATS-6 |
| RNF-3 — Um efeito é medido contra o que as forças já explicam | `rank_with`, `estimate`, `shrunk_delta`, `lane_outlooks`, `expected_against`, `expected_together` | T-DRAFT-1, T-DRAFT-2, T-DRAFT-3, T-PREVIA-2, T-ROTA-4, T-STATS-7 |
| RNF-4 — Tempo real para uma fase de 30 segundos | `minimax_with` | T-CASCA-20, T-DRAFT-48 |
| RNF-5 — Funciona sem chave | `review`, `outcome` | T-STATS-8 |
| RNF-6 — A chave da Riot nunca vai para binário, repositório, argumento ou URL | — | T-COLETOR-11 |
| RNF-7 — A atualização nunca instala sozinha | — | T-CASCA-13 |
| RNF-8 — Nada sai da máquina do jogador | `review` | T-CASCA-17 |
| RNF-9 — O core testa em segundos, sem MSVC e sem Tauri | — | — |
| RNF-10 — O executável não depende do Visual C++ Redistributable | — | — |
| RNF-11 — O perfil e o pós-jogo apontam, não dão nota | `build`, `review`, `Reading::is_clear` | T-DRAFT-21, T-PERFIL-3, T-PERFIL-9, T-POS-25 |
| RNF-12 — A cor da marca nunca carrega significado de dado | — | TF-CASCA-7 |

