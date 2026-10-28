# Requisitos

Dois tipos. As regras da Riot (**C-\***) não se negociam: violar uma é perder a
chave de API, e sem chave não há produto. Os requisitos não funcionais
(**RNF-\***) são escolhas do produto, e cada um aponta a decisão que o garante.

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
