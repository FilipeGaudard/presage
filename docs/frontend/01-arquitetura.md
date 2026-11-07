# 01 — Arquitetura do frontend

React 18 e TypeScript, empacotados pelo Vite e servidos pela webview do Tauri 2.
Não há roteador, biblioteca de estado nem cliente HTTP: o estado é `useState`
em `App.tsx`, e a única rede do front é o Data Dragon.

## 1. Três páginas, um código

| Página | Entrada | Monta | Fala com a casca | Onde abre |
|---|---|---|---|---|
| `index.html` | `src/main.tsx` | `App` em `StrictMode` | sim: `invoke` e `listen` em `App.tsx` | janela `main` (`tauri.conf.json`) |
| `overlay.html` | `src/overlay.tsx` | `Overlay` | sim: `invoke` e `listen` no próprio arquivo | janela `overlay` ([04](04-overlay.md)) |
| `demo.html` | `src/demo.tsx` | `Demo`, com dados de exemplo | não | `http://localhost:1420/demo.html` no `npm run dev`, ou `Presage-demo.html` ([08 §2](../arquitetura/08-desenvolvimento-e-testes.md#2-olhar-a-interface-sem-o-cliente)) |

As três importam os mesmos componentes. `main.tsx` e `demo.tsx` carregam
`styles.css`; `overlay.tsx` carrega só `overlay.css`, que repete à mão os tokens
de cor ([§7](#7-cor-e-estilo)).

## 2. Estrutura de pastas

```
index.html  overlay.html  demo.html       as três entradas
vite.config.ts                            build do app: main, overlay e demo
vite.demo.config.ts                       build da demonstração num arquivo só
public/favicon.svg
src/
  main.tsx            monta App
  App.tsx             estado, todos os invoke/listen da janela principal, as sete abas
  overlay.tsx         o overlay inteiro: estado, invoke/listen e desenho
  demo.tsx            dados de exemplo e a mesma navegação de App, sem casca
  globals.d.ts        declara __APP_VERSION__
  styles.css          tokens de cor e todo o estilo da janela principal
  overlay.css         tokens copiados e o estilo do overlay
  lib/
    types.ts          espelho à mão dos tipos que a casca serializa, e constantes
    ddragon.ts        nomes e ícones de campeão, direto do CDN
  components/
    Atualizacao.tsx        aviso de versão nova (plugin updater)
    ConnectionStatus.tsx   ponto de conexão, fase do cliente, Riot ID
    Logo.tsx               a marca em SVG
    DraftBoard.tsx         os dois times, bans, relógio, rota presumida
    RecommendationPanel.tsx  ranking, menos expostos, bans, detalhe e prévia
    SimuladorDraft.tsx     o draft montado à mão
    LaneAnalysis.tsx       chance por rota e escaramuças (exporta SkirmishRow)
    CurvaDeNivel.tsx       curva 1–18 e a tabela de vantagem de nível
    CondicaoDeVitoria.tsx  as frases com a fonte ao lado
    Runas.tsx              runa principal, alternativas, árvore secundária
    LiveOverview.tsx       timers, buffs, placar, sua rota ao vivo, prioridade
    GoldTracker.tsx        ouro dos dez
    LiveBuild.tsx          divisão do dano, vida efetiva, próximo item, inimigos
    PostGame.tsx           a leitura pós-partida
    GraficoPartida.tsx     as curvas de ouro, tropas ou visão, e nível
    PerfilDeRota.tsx       a tabela do perfil de rota
    PoolBuilder.tsx        o assistente da pool e a lista de bans
    OverlayPrefsPanel.tsx  as seções do overlay
```

## 3. Quem fala com a casca

**`App.tsx` faz todo `invoke` e todo `listen` da janela principal.** Os
componentes recebem dado e callback por props e nunca importam
`@tauri-apps/api` (confira: só `App.tsx:2-3` e `overlay.tsx:3-4` importam). É
por isso que `demo.tsx` renderiza os mesmos componentes sem casca: troca o
callback que chama `invoke` por um que devolve um objeto fixo
(`onSuggest={async () => planoDemo}`, `onPreview={async (id) => previaDemo(id)}`).

Três ressalvas ao "puro":

- **Estado de tela.** `RecommendationPanel` guarda o candidato aberto e a
  prévia; `PoolBuilder` guarda o passo do assistente, a rota e a busca. É
  estado de interface, não dado do motor.
- **Callbacks que chamam a casca.** `onPreview` e `onSuggest` são `invoke`
  embrulhados em `App.tsx:246-260`. O componente chama a função e espera a
  promessa; não sabe que existe Tauri.
- **`Atualizacao.tsx` não é puro.** Ele importa `check` de
  `@tauri-apps/plugin-updater` e `relaunch` de `@tauri-apps/plugin-process`. Não
  usa `invoke`, mas fala com plugins do Tauri, e por isso `demo.tsx` não o monta
  ([01 §4](../arquitetura/01-core-e-casca.md#4-o-front) registra a exceção; a
  frase "nenhum fala com o Tauri" de
  [08 §2](../arquitetura/08-desenvolvimento-e-testes.md#2-olhar-a-interface-sem-o-cliente)
  vale para os outros dezessete).

O overlay é a exceção de desenho: `overlay.tsx` tem estado, chamadas e marcação
no mesmo arquivo e não usa nenhum componente de `components/`.

Os nomes de evento são constantes repetidas nos dois lados:
`App.tsx:44-48` e `overlay.tsx:17-18` contra `watcher.rs:35-41` e
`lib.rs:37`. A lista completa de comandos e eventos é a do
[12](../arquitetura/12-contrato-da-casca.md).

## 4. O espelho dos tipos

`lib/types.ts` descreve à mão o JSON que cada comando devolve: 55 interfaces e
uniões, em `camelCase`, como o `serde` da casca escreve. Nada liga os dois lados
na compilação: um campo renomeado no Rust compila dos dois lados e quebra em
tempo de execução, com `undefined` na tela.

Quem pega a divergência é o doccheck
([ADR-0022](../adr/0022-tipos-da-ponte-conferidos-campo-a-campo.md)). Cada tipo
que cruza a ponte tem um título com `↔` no 10 (por exemplo `Pool ↔ Pool`), e o
doccheck exige campos do doc = campos do Rust e nomes serializados do Rust =
campos da interface TS.

O arquivo também guarda o que não é espelho:

| Constante | Valor | Cópia de | Conferida? |
|---|---|---|---|
| `POOL_SIZE`, `CORE_SIZE` | 8, 3 | `poolgap::POOL_SIZE`, `poolgap::CORE_SIZE` | não |
| `BAN_POOL_MAX` | 5 | `pool::BAN_POOL_MAX`, que nenhum código Rust usa: o limite só existe na tela | não |
| `OVERLAY_SECTIONS` | rótulo e detalhe das seis seções | `prefs::Section::label` e `detail`, com o mesmo texto | não |
| `ROLES` | `TOP`, `JUNGLE`, `MID`, `ADC`, `SUPORTE` | nenhuma: `Role::short` diz `JG` e `SUP` | — |
| `POSITION_LABEL` | `top` → `TOP` … `utility` → `SUP` | `Role::short`, pela chave do LCU | não |
| `ROLE_TO_LCU` | `Mid` → `middle` … | `Role::from_lcu` ao contrário | não |
| `TIERS` | `Core`, `Situacionais`, `Extras` | nenhuma: `Tier` não tem rótulo no Rust | — |

`ROLE_TO_LCU` existe porque a ponte usa dois vocabulários de rota: os comandos
que recebem rota leem a palavra do cliente (`middle`), e os tipos que voltam
trazem o nome do enum (`Mid`) ([GC-6](07-lacunas-da-casca.md)). As cópias sem
conferência são as [GC-9](07-lacunas-da-casca.md) e [GC-10](07-lacunas-da-casca.md).

## 5. Data Dragon no front

`lib/ddragon.ts` baixa, ao abrir, `api/versions.json` e o `champion.json` da
versão mais nova em `pt_BR`, com `en_US` de reserva (`ddragon.ts:22-49`). Monta um
`Map` de id numérico para nome e URL do retrato, e `itemIcon` monta a URL do
ícone de item. Não há cache: sem rede, `App.tsx:88` guarda o erro e a faixa
"Não consegui carregar os dados do Data Dragon" aparece; retratos viram `?` e
nomes viram `#id`.

A casca baixa o mesmo Data Dragon por conta própria, com cache em disco
(`ddragon.rs:137-150`). Os dois podem divergir de versão, e offline a casca
continua com nomes enquanto a tela não tem nenhum ([GC-12](07-lacunas-da-casca.md)).

As runas não passam por aqui: o ícone vem da casca como caminho relativo e
`Runas.tsx:9` prefixa `https://ddragon.leagueoflegends.com/cdn/img/`.

## 6. Build e versão

| Comando | Config | Sai |
|---|---|---|
| `npm run dev` | `vite.config.ts` | servidor na porta fixa 1420 (`strictPort`), que é o `devUrl` do Tauri |
| `npm run build` | `tsc --noEmit && vite build` | `dist/` com `index.html`, `overlay.html`, `demo.html`, alvo `chrome105`, com source maps |
| `npm run demo` | `vite.demo.config.ts` + `scripts/standalone.mjs` | `dist-demo/` com um JS e um CSS, costurados em `Presage-demo.html`, que abre com dois cliques |

- **Versão.** `vite.config.ts:12-14` lê `version` do `package.json` (tirando o
  BOM que o PowerShell 5.1 grava) e injeta `__APP_VERSION__`; `globals.d.ts`
  declara o tipo; `App.tsx:332` mostra `v0.7.0` na barra. A interface não
  escreve o número em lugar nenhum. É a prevenção do
  [L-12](../arquitetura/09-licoes-do-prototipo.md), descrita em
  [06 §6](../arquitetura/06-distribuicao-e-atualizador.md#6-a-versão).
  `vite.demo.config.ts` repete a mesma leitura, mas a demonstração não mostra a
  versão.
- **A demonstração vai no instalador.** `demo.html` é entrada do build do app
  (`vite.config.ts:40`), então os dados de exemplo e os `.map` de todo o front
  entram em `dist/`, que o Tauri empacota.
- **O que o `tsc` prova.** Só que o front é coerente com `lib/types.ts`. Não
  prova que `lib/types.ts` é coerente com o Rust (§4).

## 7. Cor e estilo

`styles.css` define os tokens da [identidade](../produto/identidade.md#paleta)
em `:root` e todo o estilo da janela principal, sem pré-processador nem CSS por
componente. O número ganha cor por `good`, `bad` e `even`, e pelos `delta-strong`,
`delta-ok` e `delta-weak` do draft, que reusam `--mint`, `--ember` e `--coral`.

`overlay.css` copia os tokens à mão, com diferenças: `--dim` é `#8b94a7` no
overlay e `#7c86a6` no painel, e o vermelho se chama `--hot`. São duas cópias da
paleta, o padrão do [L-7](../arquitetura/09-licoes-do-prototipo.md).

Os limiares de cor também moram em cada componente, e não concordam:

| Onde | Bom | Ruim |
|---|---|---|
| `LaneAnalysis.tsx:15-19`, overlay (escaramuça) | ≥ 55% | ≤ 45% |
| `CurvaDeNivel.tsx:31`, `:89` | ≥ 53% | ≤ 47% |
| chance do time na prévia (`RecommendationPanel.tsx:363`) | ≥ 52% | ≤ 48% |
| delta do draft (`RecommendationPanel.tsx:36-40`) | ≥ 2 | < 1 (âmbar entre 1 e 2) |
| perfil de rota (`PerfilDeRota.tsx:19`) | fora de ±1,96·sd | idem |

## 8. Desempenho

- `App.tsx:139` dispara um `setInterval` de 100 ms que nunca para. Ele só serve
  ao relógio do draft, mas re-renderiza a árvore inteira dez vezes por segundo,
  em qualquer aba, com ou sem draft. Nenhum componente usa `memo`.
- Cada evento `presage://live` (no máximo um por segundo, e só quando algo
  mudou) dispara dois comandos na janela principal e três no overlay.
- Cada mudança de draft dispara cinco comandos, seis com uma prévia aberta
  ([03 §4](03-estado-e-atualizacao.md#4-o-que-refaz-cada-comando)).
