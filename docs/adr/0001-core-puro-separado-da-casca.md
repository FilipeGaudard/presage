# ADR-0001 — Core puro separado da casca

- **Status:** aceito
- **Data:** anterior à 0.7.0 (registrada em 2026-10-28)
- **Decisores:** time do Presage

## Contexto

- O Tauri puxa uma árvore de dependências que só compila e abre com MSVC e
  WebView2 no Windows ([08 §3](../arquitetura/08-desenvolvimento-e-testes.md#3-o-que-roda-em-cada-máquina)).
  A máquina de desenvolvimento não tinha administrador para instalar o MSVC.
- O motor — ranking, dano, build, timers, shrinkage — é a parte que mais precisa
  de teste, e não depende de nada disso.
- Os formatos da Riot mudam sem aviso, e uma mudança precisa quebrar num lugar só.

## Decisão

**`presage-core` não depende de Tauri, rede, disco nem relógio.** Recebe dados já
lidos e devolve resultados; a casca (`src-tauri`) faz todo o I/O e chama o core.

- Os tipos crus da Riot e a tradução para o domínio moram no core, numa função de
  fronteira por fonte (`to_draft_state`, `to_live_game`, `history::played`).
- A casca não tem lógica de produto: lê, chama o core, emite eventos.
- O coletor é um terceiro crate e reaproveita o core.

## Consequências

- **Positivas:** `cargo test -p presage-core` roda em segundos numa máquina sem
  MSVC, e numa Linux sem WebKitGTK; o motor inteiro é testável com números
  conhecidos; o front pode ser visto pela demo sem casca.
- **Negativas:** a casca não tem testes próprios, e um erro de fiação só aparece
  rodando o app; tipos da casca para o front são espelhados à mão em
  `src/lib/types.ts`.
- **Obrigatório:** nada de `tauri`, `reqwest` ou `std::fs` em `core/`; lógica nova
  vai para o core, não para `lib.rs`.

## Alternativas rejeitadas

| Alternativa | Por que não |
|---|---|
| Um crate só | Cada `cargo test` puxaria o Tauri: sem MSVC, nada testa |
| Lógica no front, em TypeScript | Duas linguagens para a mesma conta, e o overlay e o pós-jogo precisariam dela também |

## Referências

- [01 — Core e casca](../arquitetura/01-core-e-casca.md); RNF-9.
