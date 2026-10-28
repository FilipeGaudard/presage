# 00 — Contexto e limites

## 1. As fontes

```
máquina do jogador
  cliente da Riot ──► LCU          https://127.0.0.1:<porta>   Basic Auth, senha nova a cada execução
  partida ─────────► Live Client  https://127.0.0.1:2999      sem autenticação
  Presage ── lê as duas, grava em %LOCALAPPDATA%\Presage

internet
  Data Dragon        CDN público: campeões, itens, runas
  Riot Web API       chave: contas, partidas (em produção, via backend)
  GitHub Releases    manifesto e instalador do atualizador
```

| Fonte | Onde | Autenticação | O que dá | O que **não** dá |
|---|---|---|---|---|
| LCU | `127.0.0.1:<porta>` | Basic Auth, senha do lockfile | champ select (picks, bans, hovers do seu time), conta (puuid, região); aplica runas e item set | hover do inimigo |
| Live Client Data API | `127.0.0.1:2999` | nenhuma | itens, nível, KDA, CS e eventos dos dez durante a partida | posição no mapa, cooldowns, ouro em mão de ninguém além de você |
| Riot Web API | `*.api.riotgames.com` | `X-Riot-Token` | contas, ligas, partidas completas | nada em tempo real |
| Data Dragon | CDN público | nenhuma | campeões, itens, runas, ícones, stats base | penetração, letalidade e haste no bloco `stats` de item ([ADR-0015](../adr/0015-atributos-de-item-pela-descricao.md)) |

## 2. LCU

### Como achamos o cliente

O cliente sobe um servidor HTTPS local numa porta aleatória a cada execução, com
senha gerada na hora. Duas fontes, nesta ordem (`lcu/credentials.rs`):

1. a linha de comando do `LeagueClientUx.exe` (`--app-port` e
   `--remoting-auth-token`, `parse_cmdline`), que funciona mesmo com instalação
   fora do caminho padrão;
2. o `lockfile` na raiz da instalação, no formato
   `LeagueClient:<pid>:<porta>:<senha>:https` (`parse_lockfile`).

As credenciais mudam a cada reinício do cliente, então perder a conexão e
redescobrir é o caminho normal, não o excepcional. Sem cliente, o watcher procura
a cada 3 segundos.

### Por que a tradução fica isolada

O formato do LCU não é documentado e muda entre patches. Tudo passa por
`to_draft_state` (`core/src/draft.rs`), então quando a Riot mexer no JSON quebra
num lugar só ([01 §2](01-core-e-casca.md#2-fronteiras-entre-json-cru-e-domínio)).

A pegadinha que os testes cobrem: **o time inimigo vem com `championId`
zerado**. O campeão só aparece na ação concluída, nunca no objeto do jogador.

## 3. Live Client Data API

Local e sem autenticação: roda na máquina do jogador durante a partida e entrega
itens, nível, KDA, CS e eventos de todos os dez. Build adaptativa, calculadora de
dano, timers de objetivo, ouro por time, chance por rota e prioridade saem todos
dela ([03](03-motor-da-partida.md)). **O diferencial do produto não depende da
Riot Web API.**

A ausência de autenticação vale para todo mundo: qualquer processo da máquina lê
a porta 2999. Não é nosso para corrigir, mas é por isso que nada do Presage
confia nela como prova de identidade.

## 4. Riot Web API

### Os três tipos de chave

| Tipo | Expira | Limite | Para quê |
|---|---|---|---|
| Development | **24h** | 20 req/s, 100 req/2min | prototipagem |
| Personal | não | igual à de dev | projeto pessoal, não comercial |
| Production | não | ~500 req/10s, 30k req/10min | produto público |

A Riot proíbe manter um produto público com chave de desenvolvimento (C-10), e a
aprovação da chave de produção leva de duas semanas a mais de um mês, exigindo
produto funcionando e descrição do uso.

### Como isso fica em produção

**A chave é do desenvolvedor, não do usuário.** Ninguém que instalar o Presage
vai precisar de chave nenhuma — existe uma só, e ela nunca sai do servidor
([ADR-0004](../adr/0004-chave-da-riot-so-no-backend.md)).

```
app do usuário  ->  backend do Presage  ->  Riot API
                           ^
                    a chave vive aqui, e só aqui
```

O app pede ao backend "as estatísticas do puuid X"; o backend chama a Riot. Por
isso **não existe tela de cadastro de chave no app**, e não vai existir.

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
`PRESAGE_RIOT_ID` (`nome#tag`) e `PRESAGE_REGION`. O app espera até 2 minutos
pelo cliente antes de cair para essas variáveis.

O que a chave alimenta hoje: o histórico pessoal e o perfil de rota
([04](04-perfil-de-rota.md)). A base agregada vem do coletor
([05](05-coletor.md)), que roda à parte.

## 5. O que funciona sem chave

Quase tudo. A chave cobre uma camada só — a estatística — e ela é a última a
entrar de qualquer jeito.

| | Fonte | Precisa de chave |
|---|---|---|
| Champ select, picks, bans, hovers | LCU, `127.0.0.1:<porta>` | não |
| Aplicar runas e item set no cliente | LCU | não |
| Campeões, itens, runas, ícones | Data Dragon | não |
| Motor de composição | Data Dragon | não |
| Construtor de pool | local | não |
| **Toda a fase in-game** | Live Client Data API, `127.0.0.1:2999` | **não** |
| Leitura pós-partida | gravação local | não |
| Histórico pessoal e perfil de rota | Riot Web API | sim |
| Winrate base, matchup, sinergia | base de partidas (coletor) | sim, no coletor |

Enquanto não houver chave, `NoStats` cobre a camada estatística e o motor roda só
com composição. Nada quebra.

## 6. Limites conhecidos

- **Hover inimigo não existe.** O LCU não revela `championPickIntent` do time
  adversário; o campeão só aparece travado. É a API, não bug.
- **Só Windows.** A descoberta de credenciais usa CIM do Windows, e os caches
  moram em `%LOCALAPPDATA%\Presage`.
- **Verificação de TLS desligada** para falar com o loopback da Riot
  ([07 §2](07-seguranca.md#2-superfície), [P-8](../produto/perguntas-em-aberto.md)).
- **Cache do Data Dragon não expira.** Sem rede ele carrega o último patch
  baixado, sem avisar que está velho.
