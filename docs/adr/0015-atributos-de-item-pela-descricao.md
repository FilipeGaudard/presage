# ADR-0015 — Atributos de item lidos da descrição

- **Status:** aceito
- **Data:** anterior à 0.7.0 (registrada em 2026-10-28)
- **Decisores:** time do Presage

## Contexto

- A build adaptativa depende de penetração, letalidade e haste.
- **O bloco `stats` do Data Dragon é legado e não traz nenhum dos três** — o Lord
  Dominik's aparece lá só com dano de ataque e crítico, e o Cajado do Vazio sem
  penetração mágica. O CommunityDragon também não traz.
- O que está completo é a `description`:

  ```text
  <stats><attention>35</attention> Attack Damage<br><attention>35%</attention> Armor Penetration</stats>
  ```

## Decisão

**Os atributos de item saem do parse da descrição (`parse_stats`), sempre em
en_US**, porque os rótulos são a chave do parser. O nome exibido continua vindo no
idioma do usuário.

## Consequências

- **Positivas:** penetração e letalidade reais na conta; a ordem de penetração do
  jogo vale de verdade ([03 §3](../arquitetura/03-motor-da-partida.md#3-dano)).
- **Negativas:** o parser depende do texto da Riot; um rótulo que mude de nome
  deixa o atributo em zero.
- **Obrigatório:** dois downloads de item, en_US para a conta e o idioma do
  usuário para a tela (`STAT_LOCALE` em `ddragon.rs`); testes do parser no formato
  real da descrição, incluindo bloco truncado e rótulo desconhecido.

## Alternativas rejeitadas

| Alternativa | Por que não |
|---|---|
| Bloco `stats` do Data Dragon | Não tem os atributos que mais mudam a build |
| CommunityDragon | Também não tem |
| Tabela de atributos escrita à mão | Envelhece a cada patch, em silêncio |

## Referências

- [03 §2](../arquitetura/03-motor-da-partida.md#de-onde-vêm-os-números); [L-11](../arquitetura/09-licoes-do-prototipo.md).
