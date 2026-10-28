# 07 — Segurança

O Presage roda na máquina do jogador, ao lado de um anticheat, lendo duas APIs
locais e falando com três serviços na internet. O que pode dar errado é mais
"alguém executa código em toda a base instalada" do que "alguém lê o draft".

## 1. Ativos

| Ativo | O que acontece se vazar | Onde vive |
|---|---|---|
| **Chave privada do atualizador** | quem a tiver publica um executável que **toda instalação** baixa, confere e executa | `%USERPROFILE%\.presage\updater.key`, secret `UPDATER_PRIVATE_KEY` no CI ([06 §2](06-distribuicao-e-atualizador.md#a-chave-privada)) |
| **Chave da Riot** | a Riot revoga e todo mundo perde a camada estatística de uma vez | variável de ambiente; em produção, só no backend ([ADR-0004](../adr/0004-chave-da-riot-so-no-backend.md)) |
| Histórico do jogador | puuid e partidas: dado da própria pessoa, público na Riot | `%LOCALAPPDATA%\Presage\personal-v2.json` |
| Gravação da partida | idem | `gravacao.json`, `ultima-partida.json` — nunca sai da máquina (RNF-8) |

A ordem da tabela é a ordem de cuidado. A chave do atualizador vale mais que a
chave da Riot: uma derruba uma função, a outra entrega a máquina de cada usuário.

## 2. Superfície

### A webview

- **CSP** em `tauri.conf.json`:
  `default-src 'self'; img-src 'self' https://ddragon.leagueoflegends.com data:; connect-src 'self' ipc: http://ipc.localhost https://ddragon.leagueoflegends.com; style-src 'self' 'unsafe-inline'`.
  Sem `unsafe-eval`, sem script de fora. `'unsafe-inline'` só em estilo.
- **Permissões** em `capabilities/default.json`: `core:default`, `updater:default`,
  `process:allow-restart`. Nada de shell, nada de sistema de arquivos pelo front.
- **Nenhum `dangerouslySetInnerHTML`** no front. Todo texto passa pelo escape do
  React.
- **Comandos fechados.** A interface só alcança os comandos registrados em
  `generate_handler!`, cada um com tipo de argumento; não existe canal genérico
  para o processo nativo. O `DraftState` que o simulador manda é normalizado na
  casca antes de chegar ao motor.

### O cliente do League: só leitura

O cliente do LCU do Presage só tem `GET` (`lcu/client.rs`): sessão do champ
select, fase, invocador, região, grade de campeões e sessão do fluxo de jogo.
Nada de hover, travar, banir, página de runa ou conjunto de itens — o iTero faz
tudo isso ([referência](../referencia/itero/07-cruzamento.md)); passar a escrever
pede um ADR.

### Iniciar com o Windows

`set_autostart` grava a entrada de inicialização do usuário (plugin
`tauri-plugin-autostart`, chave `Run` do `HKCU`). É persistência na máquina, e
por isso só acontece quando o jogador liga — o padrão é desligado, ao contrário
do iTero, que liga na primeira execução.

### Entradas que não controlamos

| Entrada | Quem controla | Caminho | O que segura |
|---|---|---|---|
| Riot ID de outros jogadores | qualquer jogador da partida | Live Client, LCU → tela | escape do React; a Riot restringe os caracteres |
| Texto do Data Dragon (nome e descrição de item) | o CDN da Riot | `parse_stats` em Rust; nomes → tela | o parser só extrai números; a tela escapa |
| Payload da match-v5 | a Riot | `history::played` | tipos `default`: campo estranho derruba a métrica, não o app |
| Respostas do LCU e da porta 2999 | qualquer processo local que se passe por eles | casca | ver TLS abaixo |

### TLS no loopback

`danger_accept_invalid_certs(true)` em `lcu/client.rs` e em `live.rs`. O LCU e a
Live Client usam certificado autoassinado da Riot no `127.0.0.1`. Quem consegue
interceptar o loopback já está dentro da máquina e não precisa do Presage para
nada — o risco é baixo, mas é dívida: antes de release público vira pinning do
root CA da Riot ([P-8](../produto/perguntas-em-aberto.md)).

A Live Client não tem autenticação nenhuma: qualquer processo local lê a porta
2999. Não é nosso para corrigir; é por isso que nada do Presage trata o que vem
dela como prova de identidade.

## 3. A chave da Riot

- Vem de `PRESAGE_RIOT_API_KEY` e de lugar nenhum mais. Nunca de argumento
  (aparece na lista de processos), nunca de URL (aparece em log de proxy). Viaja
  no cabeçalho `X-Riot-Token`.
- Erro da Riot não carrega a URL: o path tem o puuid
  (`Riot API respondeu {status}`).
- 401 e 403 viram uma mensagem que diz que a chave de desenvolvimento expira em
  24h; 429 respeita o `Retry-After`.
- Em produção ela não existe no app. O app pergunta ao backend, o backend chama a
  Riot ([00 §4](00-contexto-e-limites.md#4-riot-web-api)). Chave embutida em
  binário vaza: qualquer um extrai de um `.exe`.

## 4. O atualizador

- O app só executa o que confere com a chave pública embutida
  (`plugins.updater.pubkey`). Invadir a hospedagem não basta.
- O endpoint é permanente e gravado no executável
  ([06 §4](06-distribuicao-e-atualizador.md#4-o-endpoint-é-permanente)). Um
  domínio que expire e seja registrado por outra pessoa não consegue empurrar
  código — a assinatura não bate —, mas consegue deixar a base sem atualização.
- Nunca instala sozinho; o usuário escolhe a hora (RNF-7).
- Sem certificado de assinatura de código, o Windows não sabe quem publicou o
  instalador. O SmartScreen avisa; a cadeia de confiança da primeira instalação é
  a página de download.

## 5. Compliance é fronteira de segurança

As regras C-1 a C-4 ([requisitos](../produto/requisitos.md#1-regras-da-riot-c))
também desenham a superfície: sem leitura de memória, sem injeção, sem automação
de input. O Presage não toca no processo do jogo, e por isso o Vanguard não tem
motivo para tocar no Presage. Um overlay por hook de render seria ao mesmo tempo
violação e superfície nova ([ADR-0003](../adr/0003-overlay-como-janela-externa.md)).
