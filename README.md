# Presage

Assistente de draft, partida e pós-jogo para League of Legends. Ranqueia campeões
conforme o draft acontece, adapta a build aos itens que o inimigo comprou, conta
timers e ouro, lê a partida quando ela acaba e compara cada partida sua com o
oponente direto. Diz de onde vem cada número — medição, matemática do jogo ou
palpite — e um componente sem dado vale exatamente zero.

App de desktop para Windows: Tauri, Rust e React. O design está em
[`docs/`](docs/README.md); este arquivo só diz como rodar e onde olhar.

## Layout

```
core/            presage-core: o motor inteiro, sem Tauri, rede nem disco
src-tauri/       a casca: LCU, Live Client, Riot Web API, Data Dragon, disco
src/             a interface em React; demo.tsx a mostra com dados de exemplo
tools/ingest/    o coletor que produz a base de partidas, fora do instalador
scripts/         entrega, manifesto do atualizador, toolchain sem admin
docs/            produto, arquitetura, decisões (adr/), perguntas em aberto
```

## Rodando

```powershell
npm install
npm run tauri dev
```

Abra o cliente do League em seguida. O Presage procura a cada 3 segundos e acorda
sozinho quando um champ select começa.

```powershell
cargo test -p presage-core      # o motor, em segundos, em qualquer máquina
npm run dev                     # http://localhost:1420/demo.html, sem cliente nem Tauri
```

Sem MSVC, ou sem administrador para instalá-lo: [toolchain sem admin](docs/arquitetura/08-desenvolvimento-e-testes.md#toolchain-sem-admin).

## A chave da Riot

Nada exige chave, exceto o histórico pessoal e o perfil de rota. Para eles, no
protótipo:

```powershell
[Environment]::SetEnvironmentVariable("PRESAGE_RIOT_API_KEY", "<sua chave>", "User")
```

A chave nunca entra no repositório, em argumento ou em URL, e em produção vive só
num backend — ver [00 §4](docs/arquitetura/00-contexto-e-limites.md#4-riot-web-api).

## Publicando uma versão

```powershell
# subir a versão em package.json, src-tauri/tauri.conf.json e Cargo.toml; commitar
git tag v0.8.0
git push origin v0.8.0
```

O workflow `instalador.yml` compila, testa, assina e publica a release que todo
Presage instalado consulta. Os secrets, a chave privada e por que o endpoint é
permanente: [06 — Distribuição e atualizador](docs/arquitetura/06-distribuicao-e-atualizador.md).

## Onde olhar

| Para | Leia |
|---|---|
| Entender o produto e a confiança de cada número | [visão geral](docs/produto/visao-geral.md) |
| Saber o que a Riot proíbe | [requisitos](docs/produto/requisitos.md) |
| Mexer no motor de draft | [02](docs/arquitetura/02-motor-de-draft.md) e as ADRs 0007–0009 |
| Mexer no perfil de rota | [04](docs/arquitetura/04-perfil-de-rota.md) e as ADRs 0010–0012 |
| Saber por que algo é do jeito que é | [adr/](docs/adr/) |
| Saber o que ainda está em aberto | [perguntas em aberto](docs/produto/perguntas-em-aberto.md) |

Doc e código andam no mesmo commit, e um fato vive num lugar só.
