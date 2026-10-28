# ADR-0003 — Overlay como janela externa, sem injeção

- **Status:** aceito
- **Data:** anterior à 0.7.0 (registrada em 2026-10-28)
- **Decisores:** time do Presage

## Contexto

- O Vanguard bloqueia hook de render. Injeção e automação de input são proibidas
  (C-2, C-3).
- Um overlay precisa aparecer por cima do jogo sem comer cliques.

## Decisão

**O overlay é uma janela do próprio app: transparente, sem bordas, sempre no
topo, fora da barra de tarefas, atravessável pelo mouse por padrão.**
`Ctrl+Shift+P` mostra ou esconde; `Ctrl+Shift+O` torna clicável.

## Consequências

- **Positivas:** compatível com o Vanguard; nada toca no processo do jogo; a mesma
  pilha React do app principal.
- **Negativas:** o jogo precisa rodar em **janela sem bordas** — em tela cheia
  exclusiva nenhuma janela externa aparece por cima. Não dá para contornar.
- **Obrigatório:** o overlay nasce atravessável; nada de anúncio nele (C-9). Se o
  conteúdo dele é permitido continua aberto ([P-2](../produto/perguntas-em-aberto.md)).

## Alternativas rejeitadas

| Alternativa | Por que não |
|---|---|
| Hook de render (DirectX) | Bloqueado pelo Vanguard e proibido: estaria morto antes de nascer |
| Overlay clicável por padrão | Comeria cliques no meio da partida |

## Referências

- [03 §10](../arquitetura/03-motor-da-partida.md#10-overlay); [07 §5](../arquitetura/07-seguranca.md#5-compliance-é-fronteira-de-segurança).
