# Presage

Assistente de draft e macro para League of Legends. Fase 1: conexão com o
cliente da Riot, leitura do champ select em tempo real e o motor que ranqueia
campeões conforme o draft acontece.

## De onde vem cada número

A pergunta mais importante do projeto, porque tudo que o app mostra é número.
A resposta honesta é que eles **não têm todos a mesma qualidade**, e a tabela
abaixo diz qual é qual.

| O que aparece | De onde sai | Confiança |
|---|---|---|
| Ouro em itens, CS, nível, mortes, timers | medição direta da partida | **exata** |
| Valor de item por 1000 de ouro | fórmulas de resistência e penetração do jogo | **alta na ordem**, índice no valor |
| Divisão do dano que vem (`81% físico`) | itens que eles compraram | alta depois dos primeiros itens |
| A mesma, antes de eles comprarem | prior de classe escrito à mão | **fraca** |
| Chance por rota e escaramuça | razão de poder de combate × peso escolhido à mão | **fraca como probabilidade** |
| Curva por nível (3, 6, 11, 16) | crescimento de atributos por nível | real, mas **não vê habilidades** |
| Runa do confronto | regras mecânicas escritas à mão | **regra, não estatística** |
| Último pick da pool | composição da pool + resistências | **regra, não estatística** |
| Score de draft (`+3.4`) | encaixe de composição, pesos à mão | **fraca** |
| Winrate, matchup, sinergia | — | **não existe ainda** |

### O que é medição

`gold.rs` soma o preço de cada item no inventário dos dez. Isso é exato, não
estimativa — o preço vem do Data Dragon e o inventário vem da API oficial. CS,
nível, abates e mortes idem. A leitura pós-partida é construída inteira sobre
esses números, e por isso é a parte mais confiável do produto.

### O que é matemática do jogo

`damage.rs` implementa a ordem de penetração (redução plana → redução
percentual → penetração percentual → penetração plana) e o multiplicador
`100 / (100 + resistência)`. `champion.rs` implementa o crescimento por nível
`base + ganho × n × (0,7025 + 0,0175n)`. Nada disso é estatística: é a
aritmética do próprio jogo, e um teste trava a ordem porque inverter dois
passos muda 47 para 53.

A ressalva: `offense_against` é um **proxy** de saída de dano — `AD × mult +
AP × mult`. Não simula rotações de habilidade, velocidade de ataque, crítico
nem cooldown. Por isso a *ordem* dos itens é confiável e o `6.8` é um índice
relativo, não um ganho medido de DPS.

### O que é regra escrita à mão

As runas e o último pick da pool saem de regras sobre a mecânica do confronto,
não de frequência de vitória. Uma regra acerta o caso comum e erra o campeão
excêntrico. A interface diz isso em texto, e `from_data` no resultado é `false`
justamente para a tela não escrever "segundo N partidas" quando não há N.

### O que ainda não existe

Winrate base, matchup e sinergia. O encaixe está pronto (`engine/dataset.rs`,
`tools/ingest`), os dados não. Enquanto isso, `shrunk_delta` sem amostra
devolve **exatamente zero** — por construção, não por acidente. Nenhum número
inventado entra no lugar.

### O ponto fraco a corrigir primeiro

A chance por rota é `0,5 + 0,60 × (duelo − 0,5) + matchup`. Hoje o termo de
matchup é zero, e o `0,60` é um peso que foi escolhido, não ajustado contra
dado nenhum. Então aquele `47%` é uma comparação de poder de combate vestida de
probabilidade. Ela ordena bem — quem está à frente aparece à frente — mas o
número em si não é uma frequência de vitória, e só vira uma quando a base de
partidas entrar na conta.

## Toolchain sem admin

O instalador oficial do Node e o Visual Studio Build Tools pedem elevação. Se a
TI não libera UAC, dá para montar tudo no perfil do usuário:

| O quê | Onde | Como |
|---|---|---|
| Node.js 24 | `%USERPROFILE%\tools\node-v24.22.1-win-x64` | zip oficial do nodejs.org, descompactado |
| MinGW-w64 | `%USERPROFILE%\tools\mingw64` | build WinLibs, serve de linker no lugar do MSVC |
| Rust | `%USERPROFILE%\.cargo` | `rustup-init.exe -y --default-host x86_64-pc-windows-gnu` |

### Até onde o alvo GNU chega

- **Compila.** O workspace inteiro, Tauri incluso.
- **Os testes do core passam.** 45 testes, nativo, sem contorno.
- **O app não abre.** Binário que linka o Tauri sai com
  `STATUS_ENTRYPOINT_NOT_FOUND`: importa `WebView2Loader.dll` e import libs
  geradas pelo `dlltool` do binutils, que não batem com o loader.

Ou seja: **`npm run tauri dev` não roda numa máquina só com MinGW.** Para abrir
o app e gerar instalador é preciso o Visual Studio Build Tools, que exige admin.
O contorno GNU serve para escrever código com o compilador e os testes atrás,
não para executar.

Duas armadilhas que custaram tempo:

- **Caminho de build curto.** Com o `target/` em caminho longo o `dlltool`
  falha. Resolve com `$env:CARGO_TARGET_DIR = "C:\t\presage"`.
- **`npm install` não roda postinstall** nas versões novas do npm, e o esbuild
  precisa do dele. `npm install-scripts approve esbuild` — já está no
  `package.json`.

## Gerando o instalador

`tauri build` precisa do MSVC, e instalar MSVC pede administrador. Três
caminhos, em ordem de recomendação:

1. **GitHub Actions** — `.github/workflows/instalador.yml`. As máquinas
   `windows-latest` já têm MSVC. Dispare manualmente pela aba Actions, ou
   empurre uma tag `v0.2.0` e a release sai como rascunho com o `.exe` e o
   `.msi` anexados. É de graça em repositório público e dá a URL que a página
   de download precisa.
2. **Qualquer máquina com administrador** — instale o Visual Studio Build
   Tools e rode `npm run tauri build`. Os instaladores saem em
   `target/release/bundle/`.
3. **MSVC portátil, sem administrador** — baixar os pacotes do compilador e do
   Windows SDK direto do manifesto da Microsoft e montar à mão. Funciona, mas é
   o caminho mais trabalhoso dos três.

Sem certificado de assinatura, o SmartScreen avisa quem baixar. O workflow já
tem as variáveis prontas, comentadas, para quando o certificado existir.

## O atualizador

Sem ele, cada correção vira "baixe e reinstale à mão" — e numa ferramenta de
LoL, que precisa acompanhar o patch, isso é defeito de produto.

```powershell
$env:TAURI_SIGNING_PRIVATE_KEY = "$env:USERPROFILE\.presage\updater.key"
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = ""
npm run tauri build
npm run manifesto -- --url https://presage.gg --notas "o que mudou"
npm run entrega
```

O build assinado produz o instalador e um `.sig` ao lado. O `latest.json`
carrega a versão, a URL e essa assinatura. O app baixa o manifesto, compara a
versão, baixa o instalador e **confere a assinatura com a chave pública embutida
nele** antes de executar qualquer coisa. Sem a chave privada ninguém empurra um
executável para a base instalada — nem quem invadir a hospedagem.

### A chave privada

Está em `%USERPROFILE%\.presage\updater.key`, fora do repositório de propósito.

**Faça backup dela.** Perder essa chave significa que nenhuma instalação
existente volta a aceitar atualização: todo mundo teria de desinstalar e
instalar de novo à mão. Ela nunca vai para o repositório, nunca entra no
instalador, e em CI vive como secret.

A pública fica em `tauri.conf.json`, em `plugins.updater.pubkey` — essa é para
ser vista.

### O primeiro salto é manual

Uma versão sem atualizador não sabe atualizar. A `0.3.0` é a primeira que tem;
quem está em `0.1.0` ou `0.2.0` precisa baixar e instalar essa uma vez. Daí em
diante é automático.

Vale repetir para quem for distribuir: **não existe atualizador que conserte o
passado**. Quem instalou antes da `0.3.0` só sai de lá com uma instalação
manual.

### O que o cliente vê

Uma faixa discreta no topo: *"Versão 0.4.0 disponível"*, com o que mudou, e dois
botões — atualizar agora ou depois. **Nunca instala sozinho**: trocar o
executável e reiniciar no meio de um draft de 30 segundos seria o pior momento
possível. E falha calada: sem internet, servidor fora do ar ou endpoint ainda
não configurado, o aviso simplesmente não aparece.

### Publicar uma versão nova

O endereço que o app consulta é a release do GitHub:

```
https://github.com/pedrojpgazzoni-coder/presage/releases/latest/download/latest.json
```

Para publicar, basta uma tag — o workflow faz o resto:

```powershell
# 1. subir a versão nos três arquivos (package.json, tauri.conf.json, Cargo.toml)
# 2. commitar
git tag v0.8.0
git push origin v0.8.0
```

O workflow compila, assina, gera o `latest.json` apontando para a tag e publica
a release. A partir daí todo Presage instalado vê a versão nova na próxima vez
que abrir.

**Dois secrets precisam existir no repositório**, senão a release sai sem
assinatura e nenhum app aceita o download:

| Secret | O que é |
|---|---|
| `UPDATER_PRIVATE_KEY` | o conteúdo de `%USERPROFILE%\.presage\updater.key` |
| `UPDATER_KEY_PASSWORD` | vazio, se a chave foi gerada sem senha |

E o repositório precisa ser **público**: asset de release privada exige
autenticação, e o app não tem como se autenticar.

### O endpoint é permanente. Escolha antes de distribuir.

`plugins.updater.endpoints` em `tauri.conf.json` é gravado **dentro do
executável**. Aquela instalação vai perguntar naquele endereço pelo resto da
vida dela, e não existe jeito de mudar isso depois — mudar o endereço exige uma
versão nova, que só chega pelo endereço antigo.

Distribuir com o placeholder, ou com um domínio que você não comprou, orfana
todo mundo que instalar: nunca mais recebem versão nova, e a única saída é
desinstalar e instalar à mão.

Por isso `entrega.ps1` reclama alto quando acha `TROCAR` na configuração, e
testa por HTTP os endereços que sobraram.

São dois endereços de propósito, e o app tenta na ordem. O segundo é um espelho
que não depende do domínio — GitHub Releases serve bem, é de graça e sobrevive
a troca de domínio:

```
https://github.com/<usuario>/<repo>/releases/latest/download/latest.json
```

Com o espelho no lugar, trocar de domínio depois deixa de ser catastrófico:
quem instalou continua achando atualização pelo segundo endereço.

O `latest.json` vai para a mesma pasta do site, então a hospedagem que serve a
página já serve a atualização. Não precisa de servidor separado.

### A pasta de entrega

```powershell
npm run tauri build   # instalador (assinado, se a chave estiver no ambiente)
npm run demo          # a interface num arquivo só
npm run site          # a página num arquivo só
npm run manifesto -- --url https://presage.gg
npm run entrega       # junta tudo em Presage-entrega\
```

`Presage-entrega\` fica dentro do projeto e é montada do zero a cada vez —
`instalador\` e `site-para-hospedar\` são esvaziados antes de encher, senão o
instalador da versão anterior fica ao lado do novo e ninguém sabe qual é qual.

Dentro de `site-para-hospedar\` o instalador se chama **`Presage-setup.exe`**,
sem número de versão: é o nome para onde o botão da página aponta, e ele não
pode mudar a cada release. A cópia versionada fica em `instalador\`.

### A versão

Está em três lugares que precisam andar juntos: `package.json`,
`src-tauri/tauri.conf.json` e o `version` do workspace em `Cargo.toml`. A
interface não repete o número — o Vite injeta `__APP_VERSION__` a partir do
`package.json`, porque a barra de cima já ficou dizendo `0.1.0` depois de um
bump, e versão que mente atrapalha mais que ajuda.

É por ela que se sabe qual build está instalada: o número aparece no canto
superior esquerdo, ao lado do patch.

## Rodando

```powershell
npm install
npm run tauri dev
```

Abra o cliente do League em seguida. O Presage procura a cada 3 segundos e
acorda sozinho quando um champ select começa.

```powershell
cargo test -p presage-core
```

### Olhar a interface sem o cliente aberto

Os componentes são puramente visuais — nenhum fala com o Tauri — então há uma
página que os renderiza com dados de exemplo:

```powershell
npm run dev
```

Depois abra `http://localhost:1420/demo.html`. Serve para trabalhar no visual
sem partida em andamento, e **funciona mesmo numa máquina sem MSVC**, já que é
só a parte web.

## Arquitetura

O workspace separa a lógica pura da casca.

```
core/src/                    presage-core — sem Tauri, sem rede, sem IO
  draft.rs                   tipos crus do LCU  ->  modelo de domínio
  live.rs                    estado da partida  ->  modelo de domínio
  lockfile.rs                parsing das credenciais do cliente
  pool.rs                    pool do jogador, por rota e tier
  prefs.rs                   o que o jogador quer ver no overlay
  riot.rs                    endereçamento da Riot API (região, Riot ID)
  engine/champion.rs         atributos, classes e stats base por nível
  engine/damage.rs           resistência efetiva, penetração, vida efetiva
  engine/items.rs            atributos de item e parser da descrição
  engine/build.rs            build adaptativa por valor/ouro
  engine/lanes.rs            chance por rota e escaramuça 2v2 / 3v3
  engine/gold.rs             ouro dos dez: investido exato, ganho estimado
  engine/coach.rs            leitura da partida a partir da gravação
  engine/dataset.rs          formato e carregador da base de partidas
  engine/stats.rs            shrinkage bayesiano e a interface de dados
  engine/comp.rs             perfil de composição de time
  engine/mod.rs              rank(): junta tudo e propaga incerteza

src-tauri/src/               a casca
  lcu/credentials.rs         acha o cliente (linha de comando, depois lockfile)
  lcu/client.rs              HTTP com Basic Auth sobre o TLS autoassinado
  live.rs                    Live Client Data API, 127.0.0.1:2999
  riot.rs                    Riot Web API, chave vinda do ambiente
  profile.rs                 histórico pessoal, incremental e cacheado
  ddragon.rs                 campeões e itens, com cache em disco
  analysis.rs                resolve as rotas dos dez e chama o motor
  recorder.rs                grava a partida e dispara a leitura no fim
  watcher.rs                 polling adaptativo, emite eventos
  lib.rs                     comandos expostos ao front

src/                         React
  lib/ddragon.ts             ícones e nomes
  components/                DraftBoard, RecommendationPanel, LiveBuild,
                             LaneAnalysis, GoldTracker, PostGame, Runas,
                             PoolBuilder, OverlayPrefsPanel, ConnectionStatus

tools/ingest/src/            o coletor — binário à parte, fora do instalador
  aggregate.rs               partida crua -> contagens (puro, testado)
  limiter.rs                 janela deslizante no formato do cabeçalho da Riot
  api.rs                     cliente com dois limites, 429 e retentativa
  main.rs                    CLI, semeadura, retomada e cobertura
```

A separação não é cosmética: `cargo test -p presage-core` roda em 2 segundos e
funciona mesmo nesta máquina sem MSVC, porque o core não puxa a árvore do Tauri.

### Como achamos o cliente

O cliente sobe um servidor HTTPS local numa porta aleatória a cada execução, com
senha gerada na hora. Duas fontes, nesta ordem: a linha de comando do
`LeagueClientUx.exe` (`--app-port` e `--remoting-auth-token`), que funciona mesmo
com instalação fora do caminho padrão; e o `lockfile` na raiz da instalação, no
formato `LeagueClient:<pid>:<porta>:<senha>:https`.

As credenciais mudam a cada reinício do cliente, então perder a conexão e
redescobrir é o caminho normal, não o excepcional.

### Por que a tradução fica isolada

O formato do LCU não é documentado e muda entre patches. Tudo passa por
`to_draft_state`, então quando a Riot mexer no JSON quebra num lugar só.

A pegadinha que os testes cobrem: **o time inimigo vem com `championId`
zerado**. O campeão só aparece na ação concluída, nunca no objeto do jogador.

### Por que polling e não WebSocket

350ms durante o champ select é tempo real para uma fase de 30 segundos, e é bem
mais simples de acertar. O WS (`wss://127.0.0.1:<porta>/`, mesma Basic Auth,
evento `OnJsonApiEvent_lol-champ-select_v1_session`) vira ganho real quando
formos reagir a hover de aliado.

## O motor

`rank()` devolve, para cada candidato, uma probabilidade de vitória com
intervalo de confiança. Ela sai da soma de cinco componentes:

| Componente | Estado | Fonte |
|---|---|---|
| Winrate base do campeão na rota | **sem dado** | pipeline de partidas |
| Matchup contra o laner inimigo | **sem dado** | pipeline de partidas |
| Sinergia com os picks aliados | **sem dado** | pipeline de partidas |
| Seu histórico pessoal | ativo com a chave da Riot | sua conta |
| Fit de composição | **ativo** | Data Dragon |

Ligar a conta acende **um** componente, o pessoal. Os outros três não saem de
uma conta sozinha: winrate base, matchup e sinergia são médias sobre milhões de
partidas de milhares de jogadores. Com 100 partidas suas você teria menos de uma
observação por matchup — por isso o `PersonalStats` devolve `None` nos três, em
vez de fingir cobertura.

Cada componente estatístico passa por `shrink()`, média posterior de uma Beta.
Sem amostra o componente devolve exatamente o prior e **não mexe no ranking**;
com amostra grande converge para a taxa observada. É isso que deixa o motor
honesto enquanto a base não existe: hoje a ordenação vem inteira da composição,
e os intervalos de confiança são largos porque de fato não sabemos.

A incerteza de cada componente não é zero quando falta dado — é o espalhamento
típico daquele efeito na população (`MATCHUP_PRIOR_SD` e companhia), encolhendo
conforme a amostra cresce. Por isso o `±` que aparece na tela fica em torno de
11pp sem base estatística.

### O que o fit de composição mede

Dois sinais, ambos derivados do Data Dragon:

- **Equilíbrio de dano.** O time fica mais ou menos unilateral com este pick? Um
  time 100% físico deixa o inimigo comprar só armadura.
- **Frontline.** O time tem alguém que segura a linha de frente?

Os dois são ponderados pelo quanto do draft já aconteceu: um sinal lido com um
pick travado vale menos que o mesmo sinal com quatro.

A fração de dano físico de cada campeão mistura dois sinais fracos — a razão
`attack`/`magic` do campo `info`, que é grosseira, e um prior pela classe, que
acerta melhor os extremos. Sai de cena quando o pipeline entregar o split real,
que vem de `physicalDamageDealtToChampions` vs `magicDamageDealtToChampions`.

Os pesos (`W_DAMAGE_BALANCE`, `W_FRONTLINE`) são escolhidos à mão, calibrados
para que uma correção forte valha ~1,5pp. São priors, não coeficientes
ajustados.

### O que falta

Dados. O encaixe está pronto: `engine/dataset.rs` define o formato, implementa
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

Quem produz esse arquivo é o coletor em `tools/ingest`, documentado abaixo.
Enquanto ele não rodar, matchup e sinergia contribuem exatamente zero — por
construção, não por acidente: `shrunk_delta` sem amostra devolve 0.

Depois dos dados entram, nesta ordem: variância de matchup (destrava exposição a
counter-pick de verdade), ban personalizado por pool, detector de fraqueza
pessoal sobre a série de partidas gravadas.

## O coletor

`tools/ingest` é um binário à parte. Não vai no instalador e o app não depende
dele — é a ferramenta que roda na sua máquina (ou num servidor) para produzir a
base que o app lê.

Já existe um binário pronto em `ferramentas\presage-ingest.exe`. Para
recompilar: `cargo build --release -p presage-ingest`.

```powershell
# o que ele faria, sem rede e sem chave
.\ferramentas\presage-ingest.exe --plano

# a coleta de verdade
$env:PRESAGE_RIOT_API_KEY = "RGAPI-..."
.\ferramentas\presage-ingest.exe --regiao br1 --tier emerald

# continuar depois de uma interrupção
.\ferramentas\presage-ingest.exe --retomar

# relatório de cobertura de uma base já pronta
.\ferramentas\presage-ingest.exe --verificar "$env:LOCALAPPDATA\Presage\base-de-partidas.json"
```

A chave sai de `PRESAGE_RIOT_API_KEY` e de lugar nenhum mais — nunca de
argumento, porque argumento aparece na lista de processos, e nunca da URL,
porque URL aparece em log de proxy. Ela viaja no cabeçalho `X-Riot-Token`.

### Como ele coleta

1. **Semeia** jogadores pelo `league-v4`, de um tier mínimo para cima. Até
   mestre, paginando as quatro divisões; de mestre para cima, a lista única.
2. **Lista** as últimas partidas ranqueadas de cada jogador semeado pelo
   `match-v5`.
3. **Baixa** cada partida ainda não vista, em lotes concorrentes.
4. **Agrega** em memória e grava estado e base a cada 500 partidas.

### Três decisões que mudam o resultado

**Não há bola de neve.** Dava para pegar os dez jogadores de cada partida
baixada e expandir a partir deles — cresce muito mais rápido. Mas aí a amostra
escorrega para fora do elo semeado, e o `tier` do cabeçalho passa a mentir sobre
de quem são aquelas partidas. Só os jogadores semeados são expandidos.

**Um patch por base.** A primeira partida aceita trava o patch; as de outro são
recusadas e contadas no relatório. Misturar patch é o jeito mais fácil de uma
base parecer grande e recomendar item que mudou de preço.

**Partida ou entra inteira ou não entra.** Remake, partida terminada por erro,
modo que não é a Fenda, rota em branco, menos de cinco minutos: tudo descartado
com motivo, antes de qualquer contagem. Meia partida contada é pior que partida
nenhuma, e uma base que engole remake mede taxa de remake.

### O limite de taxa

Vem do cabeçalho `X-App-Rate-Limit` da própria resposta, não de constante
nossa — a primeira chamada revela o limite real da chave e o limitador adota,
mantendo o histórico do que já saiu. 429 para o limitador inteiro pelo
`Retry-After` que o servidor mandar; insistir depois de um 429 é como se perde a
chave.

Plataforma e região têm limitadores separados, porque a Riot conta as janelas
por host: `league-v4` fala com `br1.api.riotgames.com` e `match-v5` com
`americas.api.riotgames.com`.

### Quanto tempo leva, de verdade

| Chave | Taxa sustentável | ~260 mil chamadas |
|---|---|---|
| Development | 0,83/s (100 a cada 2 min) | 87 horas — e ela expira em 24 |
| Production | 50/s (30.000 a cada 10 min) | ~1,4 hora |

A linha de cima é o ponto: com chave de desenvolvedor o crawl **não termina**,
porque a chave morre antes. O coletor roda com ela para você testar o caminho,
não para produzir a base.

### O que esperar da base

`--verificar` responde isso com números. O resumo:

- **Winrate base por rota** fica sólida com algumas centenas de milhares de
  partidas. São ~170 campeões × 5 rotas.
- **Matchup** é outra ordem de grandeza. Milhares de pares por rota, e para a
  maioria passar de n=200 são necessários alguns milhões de partidas.
- **Nada disso é tudo ou nada.** O motor encolhe o que tem amostra pequena até
  o ponto de não mexer no ranking, então uma base parcial recomenda tão bem
  quanto a parte dela que já tem amostra.

`--min-jogos` (padrão 20) corta as entradas magras na hora de gravar. Com a
força de prior do motor, um matchup de dez partidas desloca a estimativa em
menos de meio ponto percentual: ocupa espaço sem dizer nada.

### O que já roda sem dado nenhum

Tudo que sai da matemática do jogo, e não de estatística:

| Função | De onde vem |
|---|---|
| Build adaptativa | itens deles + ordem de penetração |
| Chance por rota | duelo resolvido com resistências e dano reais |
| Escaramuça 2v2 / 3v3 | mesma conta, somada por grupo |
| Ouro em itens | preço de cada item, soma exata |
| Timers de objetivo | eventos da partida |
| Leitura pós-partida | gravação local, quadro a quadro |

## A chave da Riot

**A chave é do desenvolvedor, não do usuário.** Ninguém que instalar o Presage
vai precisar de chave nenhuma — existe uma só, sua, e ela nunca sai do servidor.

### Os três tipos

| Tipo | Expira | Limite | Para quê |
|---|---|---|---|
| Development | **24h** | 20 req/s, 100 req/2min | prototipagem |
| Personal | não | igual à de dev | projeto pessoal, não comercial |
| Production | não | ~500 req/10s, 30k req/10min | produto público |

A Riot proíbe explicitamente manter um produto público com chave de
desenvolvimento, e a aprovação da chave de produção leva de duas semanas a mais
de um mês, exigindo produto funcionando e descrição do uso.

### Como isso fica em produção

```
app do usuário  ->  backend do Presage  ->  Riot API
                           ^
                    a chave vive aqui, e só aqui
```

O app pede ao backend "as estatísticas do puuid X"; o backend chama a Riot. Três
razões, e nenhuma é opcional:

1. **Chave embutida em binário vaza.** Qualquer um extrai de um `.exe`. A Riot
   revoga e todo mundo perde acesso de uma vez.
2. **O rate limit é por chave, não por usuário.** Com mil usuários batendo
   direto na Riot com a mesma chave, o limite estoura no primeiro minuto. No
   backend você enfileira e distribui.
3. **Dados de partida são imutáveis.** Buscar uma vez e servir para todos é o
   que torna o pipeline viável — e é o mesmo pipeline que preenche winrate base,
   matchup e sinergia.

Por isso **não existe tela de cadastro de chave no app**, e não vai existir: na
produção o usuário não tem chave.

### O que funciona sem chave nenhuma

Quase tudo. A chave cobre uma camada só — a estatística — e ela é a última a
entrar de qualquer jeito, porque depende da chave de produção, que leva semanas
para ser aprovada.

**Sem chave, e já testável hoje:**

| | Fonte | Precisa de chave |
|---|---|---|
| Champ select, picks, bans, hovers | LCU, `127.0.0.1:<porta>` | não |
| Aplicar runas e item set no cliente | LCU | não |
| Campeões, itens, runas, ícones | Data Dragon, CDN público | não |
| Motor de composição | Data Dragon | não |
| Construtor de Pool | local | não |
| **Toda a Fase 2 in-game** | Live Client Data API, `127.0.0.1:2999` | **não** |

A Live Client Data API é local e sem autenticação: roda na máquina do jogador
durante a partida e entrega itens, nível, KDA, CS e eventos de todos os dez.
Build adaptativa, calculadora de dano, timers de objetivo, gold por time, win
probability e ranking de prioridade saem toda dela. **O diferencial do produto
não depende da Riot Web API.**

**Só isto precisa de chave:** histórico pessoal e a base agregada (winrate,
matchup, sinergia).

### Ordem recomendada

1. Fase 2 in-game, que é o diferencial e não precisa de chave
2. Overlay e o resto da casca
3. Pedir a chave de produção assim que houver produto para descrever
4. Pipeline de ingestão, quando a chave sair

Enquanto não houver chave, `NoStats` cobre a camada estatística e o motor roda
só com composição. Nada quebra.

### Enquanto é protótipo

Chave de desenvolvimento em variável de ambiente, que nunca entra no
repositório:

```powershell
[Environment]::SetEnvironmentVariable("PRESAGE_RIOT_API_KEY", "<sua chave>", "User")
```

Pegue em developer.riotgames.com, logando com a conta da Riot — o dashboard
mostra a chave e um botão para regenerar. Quando expirar, o Presage registra o
erro e segue sem o componente pessoal, sem quebrar.

A conta sai do próprio cliente — o LCU entrega puuid e região, então não há Riot
ID digitado em lugar nenhum. Com o cliente fechado dá para forçar por
`PRESAGE_RIOT_ID` (`nome#tag`) e `PRESAGE_REGION`.

A busca é incremental: 60 partidas de ranqueada solo por atualização, espaçadas
1,3s para respeitar o limite da chave, e o que já foi contado fica em
`%LOCALAPPDATA%\Presage\personal.json`. A primeira carga leva ~80s em segundo
plano; depois só as partidas novas.

## Construtor de Pool

Nada é obrigatório e não há cota por tier — os limites do iTero (3 core, 3
situacionais, até 4 extras) viram sugestão, não trava. O jogador escolhe por
rota, e um campeão sem rota vale em todas.

No draft, as recomendações saem em duas seções: primeiro o seu pool ranqueado
entre si, depois os melhores no geral. O número embaixo do retrato é o desvio em
pontos percentuais contra uma partida neutra, não a winrate absoluta.

O pool fica em `%LOCALAPPDATA%\Presage\pool.json`.

## Build adaptativa

A parte do produto que **não é estatística**. Dado o item que o inimigo
comprou, o resultado é exato: não há amostra, prior nem intervalo de confiança.
Por isso ela funciona hoje, sem chave e sem pipeline.

Para cada item candidato a pergunta é sempre a mesma: quanto ele sobe o seu
poder de combate, por ouro, **contra os itens que eles têm agora**.

Poder é vida efetiva vezes dano efetivo. O produto é o que importa — dobrar o
dano e dobrar a vida valem o mesmo numa luta — e é o que deixa item ofensivo e
defensivo competirem na mesma régua, sem peso arbitrário entre os dois.

O que o motor reage, com teste para cada caso:

- time inimigo AD puxa armadura acima de resistência mágica
- o mago deles comprar poder de habilidade inverte a prioridade
- penetração de armadura no inventário deles derruba o valor da sua armadura
- contra letalidade, vida passa a render mais que armadura
- o inimigo comprar armadura faz penetração subir na sua lista

### De onde vêm os números

Stats base de campeão saem do Data Dragon, com a fórmula real de crescimento
por nível:

```text
valor(n) = base + ganho * n * (0.7025 + 0.0175 * n),  n = nível - 1
```

Não é linear. No nível 18 o fator fecha em exatamente 17, o que serve de teste
da própria fórmula.

Atributos de item são o ponto delicado. **O bloco `stats` do Data Dragon é
legado e não traz penetração, letalidade nem haste** — o Lord Dominik's aparece
lá só com dano de ataque e crítico, e o Cajado do Vazio sem penetração mágica.
O CommunityDragon também não traz. O que está completo é a `description`:

```text
<stats><attention>35</attention> Attack Damage<br><attention>35%</attention> Armor Penetration</stats>
```

Então parseamos a descrição, sempre em en_US, porque os rótulos são a chave do
parser. O nome exibido continua vindo no idioma do usuário.

Duas regras do jogo que o motor respeita e que calculadora amadora erra:

- **Ordem de penetração**: redução plana, redução percentual, penetração
  percentual, penetração plana. Com 100 de armadura, 35% de pen e 18 de
  letalidade dá 47 — na ordem trocada daria 53,3.
- **Penetração percentual é multiplicativa**: dois itens de 35% dão 57,75%, não
  70%.

### Limites honestos

O lado ofensivo é um **proxy, não DPS**: soma dano de ataque e poder de
habilidade passados pelas resistências reais do alvo, sem razões de habilidade,
recarga ou crítico. Serve para ordenar itens, não para prever o dano de um
combo.

Também não trata hoje: caminhos de construção (sugere o item final, não o
componente), itens únicos por categoria, botas, e passivas de item, que não
estão em nenhum dado estruturado.

## Timers e prioridade

Dois tipos de número, e a diferença aparece na tela:

- **Observado**: vimos o abate no fluxo de eventos, então o respawn é uma soma
  exata.
- **Estimado**: ninguém matou ainda, e o primeiro spawn vem de uma constante.

As constantes vivem num único `Timings` e **mudam a cada temporada** — na 16 o
Barão voltou de 25:00 para 20:00, o Arauto virou único às 15:00 e o Atakhan foi
removido. A lógica é testada contra valores arbitrários, então trocar uma
constante não quebra nada além dela mesma.

### Ranking de prioridade

A API da Riot não entrega posição de campeão no mapa, e nenhuma via legal
entrega. Então o Presage **não diz onde o inimigo está** — diz onde está o
valor. A formulação importa tanto quanto a conta, e há um teste que varre as
razões geradas procurando linguagem de posição.

O que alimenta: quem está morto e por quanto tempo, CS e nível de cada um, os
itens dos dez, e os eventos de objetivo. Por cima roda a simulação de
escaramuça do motor de dano, com as resistências e a penetração reais — o mesmo
motor da build adaptativa.

## Draft com resposta

O ranking guloso responde "qual o melhor pick agora", que não é a pergunta do
draft: depois de você travar, eles escolhem contra você. O `minimax` olha uma
camada à frente e ordena pelo valor **depois** da melhor resposta deles. A
diferença entre os dois números é a exposição do pick.

A busca é podada — só os melhores candidatos, só as melhores respostas — porque
a árvore cheia é 170 × 170 e a fase dura 30 segundos.

### Por que o ban ainda não é pessoal

`team_value` é **aditiva**: a composição deles e a sua não interagem. Logo
"quanto o campeão X bate no seu pool" é, na álgebra, idêntico a "quanto X ajuda
eles" — o pool não muda nada na conta. Separar os dois exige o termo de
interação, que é exatamente a tabela de matchup que o pipeline vai trazer.

Até lá o ban ordena por quanto o campeão completa a composição deles, e a função
não finge que faz mais. Há um teste travando essa limitação; quando o matchup
entrar, ele deve falhar — e aí o ban vira pessoal.

O ban também **devolve vazio enquanto o outro lado não travou nada**. Sem pick e
sem base estatística todo campeão pontua igual, e ordenar isso seria apresentar
ruído como conselho.

## Overlay

Janela externa, transparente, sempre no topo. **Não há injeção em nada**: o
Vanguard bloqueia hook de render, e um overlay que dependesse disso estaria
morto antes de nascer.

| Atalho | O quê |
|---|---|
| `Ctrl+Shift+P` | mostra ou esconde |
| `Ctrl+Shift+O` | torna clicável (por padrão o mouse atravessa) |

Duas consequências de ser janela externa, e não dá para contornar: **o jogo
precisa rodar em janela sem bordas** — em tela cheia exclusiva nenhuma janela
externa aparece por cima — e o overlay nasce atravessável pelo mouse, senão
comeria cliques no meio da partida.

## Identidade

A marca é um **caminho que se ramifica**, com a ponta escolhida preenchida e
acima das alternativas, que ficam esmaecidas. É o minimax do draft desenhado:
prever qual galho sobrevive à resposta do adversário. Lê como sigilo a 16px e
como árvore de decisão a 128px.

Vive em `src/components/Logo.tsx` (usa `currentColor`, então herda a cor do
contexto) e em `public/favicon.svg`. Os ícones do executável saem da mesma
geometria.

### Paleta

Uma regra organiza tudo: **a cor da marca nunca carrega significado de dado.**

| Token | Valor | Papel |
|---|---|---|
| `--ink` | `#06080f` | fundo |
| `--veil` | `#0c1020` | painéis |
| `--rise` | `#141a30` | superfícies elevadas |
| `--line` | `#222b47` | divisórias |
| `--bone` | `#e6e9f2` | texto |
| `--dim` | `#7c86a6` | texto secundário |
| `--halo` | `#7b6cff` | **marca**: seleção, foco, identidade |
| `--mint` | `#3ddc97` | número bom |
| `--coral` | `#ff5c7a` | número ruim |
| `--ember` | `#ffb454` | atenção, urgência |

Não há dourado aqui, e isso é deliberado: o dourado é a cor do próprio League,
e um app que copia a paleta do jogo não tem cara de nada. O violeta também sai
da rota de Blitz e Porofessor, que são azuis.

## Limites conhecidos

- **Hover inimigo não existe.** O LCU não revela `championPickIntent` do time
  adversário; o campeão só aparece travado. É a API, não bug.
- **Só Windows.** A descoberta de credenciais usa CIM do Windows.
- **Verificação de TLS desligada** para falar com o loopback da Riot. Antes de
  release público vira pinning do root CA da Riot, em `lcu/client.rs`.
- **Cache do Data Dragon não expira.** Sem rede ele carrega o último patch
  baixado, sem avisar que está velho.

## Compliance

O Presage lê apenas a LCU API local e, na Fase 2, a Live Client Data API
oficial. Três regras que não se negociam:

1. **Nada de ultimate timers do inimigo** — proibido pela Riot desde 13/03/2025,
   sob pena de desativação da chave de API. Inclusive os manuais.
2. **Nada de leitura de memória do jogo** — bloqueado por padrão desde
   06/10/2026 para apps não aprovados.
3. **Nada de injeção ou automação de input.** O overlay da Fase 2 é janela
   externa transparente, compatível com o Vanguard.

### Vender o produto

A política da Riot permite cobrar, e é específica sobre como. Resumo do que
está escrito na [política de League](https://support-developer.riotgames.com/hc/en-us/articles/22698698001939-League-of-Legends):

| Regra | O que significa aqui |
|---|---|
| Produto registrado, status **Approved** ou **Acknowledged** | sem isso não se cobra nada |
| **Camada grátis obrigatória** | o plano pago é adicional, nunca o único |
| Conteúdo pago tem de ser **transformativo** | "nova informação, nova estética, novos insights" |
| Formas aceitas | assinatura, doação, financiamento coletivo, inscrição de torneio |
| Proibido | aposta, jogo de azar, cripto, preço abusivo (quem julga é a Riot) |

Anúncio dentro do jogo **não é permitido**, e isso inclui overlay. Se a camada
grátis for sustentada por anúncio, ele fica no painel ou no site — nunca por
cima da partida.

### As regras de integridade que tocam o desenho

Estas não são sobre dinheiro, mas mudam o que o produto pode ser:

- *"Products should not remove game decisions, but may highlight decisions that
  are important and give multiple choices"* — é por isso que o Presage ranqueia
  com alternativas e motivo em texto, em vez de mandar num pick só. "Apps that
  dictate player decisions" está na lista de casos **não** aprovados.
- *"Products should increase, and not decrease the diversity of game decisions"*
  — a build adaptativa ajuda aqui: ela varia com a composição inimiga, em vez de
  empurrar a mesma build do patch para todo mundo.
- *"Products cannot create alternatives for official skill ranking systems...
  Prohibited alternatives include MMR or ELO calculators"* — a leitura
  pós-partida aponta erros concretos e **não** pode virar uma nota, um rating ou
  uma escada paralela.
- *"Products may not provide any game-session-specific information that would be
  previously unknown to the player"* — tudo que o Presage mostra durante a
  partida é derivado do que já está na tela do jogador: itens na aba de
  pontuação, níveis, abates, eventos. Nada de posição no mapa, cooldown inimigo
  ou ouro em mão alheio.

**O ponto de risco é o overlay.** Entre os casos aprovados para chave de
produção está *"game overlays that provide static data that is available prior
to the game"* — mais estreito do que o nosso overlay, que mostra valor
calculado ao vivo. Outras ferramentas fazem isso, mas precedente não é
permissão. Vale perguntar à Riot pelo Developer Portal antes de cobrar, e vale
ter o produto funcionando sem o overlay caso a resposta seja não.
