# 04 — Overlay

Uma segunda janela do app, por cima do jogo, com até seis linhas de informação.
Não há injeção em nada: é uma janela externa, transparente e sempre no topo
([ADR-0003](../adr/0003-overlay-como-janela-externa.md),
[03 §10](../arquitetura/03-motor-da-partida.md#10-overlay)). O código é
`src/overlay.tsx` e `src/overlay.css`, de um lado, e `src-tauri/src/overlay.rs`,
do outro.

```
┌ PRESAGE ─────────── 14:32 ┐
│ Ouro em itens       +1.240 │
│ Dano contra você  71% físico│
│ ████████████░░░░░          │
│ Sua vida efetiva     2.140 │
│ Pen. de armadura deles 18% │
│ Dragão                0:48 │
│ Barão           4:12   EST │
│ Prioridade             BOT │
│ duas torres caídas no bot  │
│ jg+bot                 61% │
│ vocês · Dragão em 0:48     │
│ Próximo item               │
│ Zhonya             3.250   │
│ armadura contra o Zed      │
│ Vazio Abissal      3.000   │
└────────────────────────────┘
```

## 1. A janela

`tauri.conf.json` declara a janela `overlay`: `overlay.html`, 300 × 340 em
(24, 120), `transparent`, sem `decorations`, `alwaysOnTop`, `skipTaskbar`, sem
sombra, sem redimensionar, `focus: false` e `visible: false`.

| Atalho | O quê | Código |
|---|---|---|
| `Ctrl+Shift+P` | mostra ou esconde; ao mostrar, reafirma "sempre no topo" sem roubar o foco | `overlay::toggle_visible` |
| `Ctrl+Shift+O` | alterna entre atravessável e clicável | `overlay::toggle_interactive` |

- **Nasce atravessável.** `overlay::prepare` chama
  `set_ignore_cursor_events(true)` antes de qualquer exibição (`lib.rs:658`), e o estado de clique é um `AtomicBool` que começa em `false`
  (`overlay.rs:20`). O overlay não come clique no meio da partida (R-15).
- **Atalho global.** Os dois são registrados no `setup` (`lib.rs:659-667`). Se outro app já tiver o
  atalho, a casca só escreve um aviso no log (`atalho global já em uso por outro
  app`); a tela não sabe ([GC-8](07-lacunas-da-casca.md)).
- **Janela sem bordas no jogo.** Em tela cheia exclusiva nenhuma janela externa
  aparece por cima. Não dá para contornar.
- **Arrasta pela cabeça.** Na partida, `Ctrl+Shift+O` deixa a janela clicável,
  e a cabeça (PRESAGE e o relógio) é região de arrasto (`data-tauri-drag-region`,
  com `core:window:allow-start-dragging` na capability do overlay). Fora da
  partida, ou com tudo desligado, a janela não desenha nada e continua
  atravessável mesmo com o atalho: quem decide é
  `OverlayPrefs::atravessa_o_mouse`, por `overlay::aplicar` (A15, T-CASCA-31,
  T-ENTRADA-50). Até a 0.7.0 ela não arrastava, e clicável sem nada na tela era
  um retângulo invisível que comia clique. A posição ainda não é gravada
  ([GC-8](07-lacunas-da-casca.md)).

## 2. As seis seções

A união `OverlaySection` (`types.ts:397-403`) espelha `prefs::Section`, com os
mesmos nomes em `camelCase`. A ordem na tela é a de `Section::ALL`.

| Seção | Mostra | Comando | Aparece quando |
|---|---|---|---|
| `gold` | "Ouro em itens", `+N` ou `−N`, verde ou vermelho | `gold_report` (`investedLead`) | há relatório |
| `damage` | "Dano contra você" em % físico com barra, "Sua vida efetiva" e, se houver, "Pen. de armadura deles" | `build_suggestions` (o `BuildContext`) | há contexto |
| `objectives` | Os três próximos objetivos com contagem; "pronto" no zero; `est` quando não observado; destaque no último minuto | `get_live`, `presage://live` (`timeline.upcoming`) | há objetivo |
| `priority` | "Prioridade" e a rota de maior valor, com o primeiro motivo | `get_live` (`priority.lanes[0]`) | há rota |
| `skirmish` | A escaramuça que importa agora, a chance do seu lado e "vocês · {por quê}" | `skirmish_focus` | há foco |
| `nextItem` | "Próximo item": dois itens com o preço, e o primeiro motivo do primeiro | `build_suggestions` com `{ limit: 2 }` | há sugestão |

A cabeça sempre aparece quando o overlay aparece: "PRESAGE" e o relógio da
partida.

## 3. Atualização

- **Na abertura:** `get_overlay_prefs` e `get_live` (`overlay.tsx:42-59`).
- **A cada `presage://live`:** troca o snapshot e chama, juntos,
  `build_suggestions` com `{ limit: 2 }`, `gold_report` e `skirmish_focus`
  (`overlay.tsx:61-90`). A casca só emite quando a partida mudou, no ritmo de
  1 s.
- **A cada `presage://prefs`:** troca as preferências. O painel grava e a casca
  avisa as duas janelas; mexer em Ajustes vale no meio da partida.
- **Sem snapshot:** zera itens, contexto, ouro e briga, e não desenha nada.

As preferências começam em `{ hidden: [] }`: até a casca responder, tudo é
visível, para o atraso de uma consulta não piscar um overlay vazio
(`overlay.tsx:36-38`).

## 4. Preferências

- **Guardam o que está escondido** (`OverlayPrefs.hidden`), não o que está
  ligado: uma seção nova nasce visível para quem já tem configuração salva
  (`prefs.rs:58-67`).
- **Persistem** em `overlay.json`; a casca lê o arquivo no `setup` (R-15: a
  escolha sobrevive a reiniciar o app).
- **Quem grava:** só a janela principal, por `set_overlay_prefs`, de forma
  otimista ([03 §6](03-estado-e-atualizacao.md#6-escritas-otimista-ou-confirmada)).
  Se o disco falhar, o painel mostra a escolha nova e o overlay continua na
  velha, sem aviso.
- **Tudo desligado:** `overlay.tsx:94` não desenha nada quando
  `prefs.hidden.length >= 6`. O 6 é escrito à mão: o painel usa
  `OVERLAY_SECTIONS.length` e o core tem `Section::ALL` e
  `OverlayPrefs::anything_visible`, que ninguém chama fora dos testes
  ([GC-11](07-lacunas-da-casca.md)).
- **"Não desenhar" não é "esconder".** Sem snapshot ou com tudo desligado, o
  componente devolve `null`, mas a janela continua aberta e no topo. O comentário
  de `overlay.tsx:92-93` diz que "a janela some"; quem some é o conteúdo. Com a
  janela clicável, um retângulo invisível de 300 × 340 continua recebendo os
  cliques.

## 5. Regras da Riot

| Regra | Como o overlay cumpre |
|---|---|
| C-4: nada que o jogador não saberia | Ouro em itens é a soma dos inventários que a aba de pontuação mostra; o overlay não mostra o ouro em mão de ninguém. Prioridade é valor, não posição |
| C-5: escolhas, não ordens | Dois itens, não um (`overlay.tsx:29-31`); a escaramuça diz a chance, não manda brigar |
| C-1: sem timer de habilidade inimiga | Só objetivos do mapa |
| C-9: sem anúncio por cima do jogo | O overlay não tem imagem, link nem texto que não seja dado da partida |
| P-2: o overlay com valor calculado ao vivo é aprovado? | Aberta. O default é que todo o produto funcione sem o overlay: nada do painel depende dele, e desligar as seis seções o apaga ([perguntas](../produto/perguntas-em-aberto.md)) |

## 6. Cores

`overlay.css` repete os tokens de `styles.css` à mão, com `--dim` diferente e o
vermelho renomeado para `--hot` ([01 §7](01-arquitetura.md#7-cor-e-estilo)).

- `up` e `down` (verde e vermelho) pintam o ouro e a escaramuça; o corte da
  escaramuça é 55% e 45% (`overlay.tsx:170`), o mesmo de `LaneAnalysis`.
- `hot` pinta o objetivo que nasce em menos de um minuto, com o vermelho de
  número ruim, e não com o âmbar de atenção da
  [identidade](../produto/identidade.md#paleta).
- `accent` pinta o nome da rota de prioridade com `--halo`, a cor da marca. É
  um rótulo, não um número, mas é a cor da marca destacando um dado (RNF-12).
