# ADR-0026 — O LCU é só lido, e iniciar com o Windows vem desligado

- **Status:** aceito
- **Data:** 2026-11-07 (registra as decisões da lacuna G-8 e do LCU)
- **Decisores:** time do Presage

## Contexto

- O iTero escreve no cliente: manda runas, itens e feitiços, faz hover, trava e
  bane sozinho (I-19 a I-23). A política da Riot proíbe tirar a decisão do
  jogador (C-5), e "apps que ditam decisões" estão entre os não aprovados.
- Escrever no LCU é a fronteira entre um assistente e um automatizador. Cada
  escrita aumenta o que pode dar errado num champ select de verdade.
- O iTero abre com o Windows por padrão (I-64). Um app que abre sozinho sem o
  jogador pedir é invasivo, e um app de jogo vive disso.

## Decisão

1. **O LCU é só lido.** `lcu/client.rs` só faz `GET`. Nada de enviar runas,
   itens ou feitiços, hover, travar ou banir. Se escrever no cliente um dia valer
   a pena, é decisão de produto ([P-15](../produto/perguntas-em-aberto.md)) e
   pede ADR nova.
2. **Iniciar com o Windows vem desligado** até o jogador ligar em Ajustes
   (`set_autostart`, a entrada `Run` do HKCU, pelo plugin de autostart do Tauri).
   A bandeja fica como está. Instância única ainda não existe: é lacuna aberta
   (G-12 no [cruzamento](../referencia/itero/07-cruzamento.md)).

## Consequências

- **Positivas:**
  - nenhuma ação no cliente que o jogador não fez;
  - o app só fica residente se o jogador quiser.
- **Negativas:**
  - o jogador copia a runa e o item à mão;
  - quem não liga o autostart abre o Presage antes do League.
- **O que passa a ser obrigatório:** `lcu/client.rs` sem `post`, `put`, `patch`
  nem `delete`. Uma escrita nova entra por ADR.

## Alternativas rejeitadas

| Alternativa | Por que não |
|---|---|
| Enviar runas e itens, sem travar nem banir | Fica em aberto como P-15: é permitido, mas muda o que o produto é |
| Autostart ligado por padrão | Residente sem pedir |

## Referências

- [cruzamento](../referencia/itero/07-cruzamento.md); [07](../arquitetura/07-seguranca.md).
