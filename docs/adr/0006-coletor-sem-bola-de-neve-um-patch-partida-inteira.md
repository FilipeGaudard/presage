# ADR-0006 — Coletor: sem bola de neve, um patch por base, partida inteira

- **Status:** aceito
- **Data:** anterior à 0.7.0 (registrada em 2026-10-28)
- **Decisores:** time do Presage

## Contexto

- A base de partidas declara no cabeçalho `patch`, `region` e `tier`, e o app
  confia neles.
- Crescer rápido é tentador: a chave de desenvolvimento não termina o crawl.
- Uma partida mal contada contamina as três tabelas ao mesmo tempo.

## Decisão

Três regras em `tools/ingest`:

1. **Sem bola de neve.** Só os jogadores semeados pelo `league-v4` são expandidos;
   os dez de cada partida baixada não viram semente.
2. **Um patch por base.** A primeira partida aceita trava o patch; as de outro são
   recusadas e contadas.
3. **Partida inteira ou nada.** Remake, terminada por erro, fora da Fenda, rota em
   branco, rotas que não fecham cinco por time, menos de cinco minutos: descartada
   com motivo antes de qualquer contagem.

## Consequências

- **Positivas:** o `tier` do cabeçalho diz a verdade sobre de quem são as
  partidas; nenhum item recomendado a partir de preço velho; a base mede vitória,
  não taxa de remake.
- **Negativas:** o crawl cresce mais devagar; uma base por patch, refeita a cada
  patch.
- **Obrigatório:** os motivos de descarte sobrevivem a uma retomada; o histórico
  pessoal (`history::played`) aplica os mesmos cortes ([L-5](../arquitetura/09-licoes-do-prototipo.md)).

## Alternativas rejeitadas

| Alternativa | Por que não |
|---|---|
| Bola de neve a partir dos dez | A amostra escorrega para fora do elo semeado, e o cabeçalho passa a mentir |
| Misturar patches | O jeito mais fácil de uma base parecer grande e recomendar item que mudou |
| Contar o que der de uma partida com campo faltando | Meia partida contada é pior que partida nenhuma |

## Referências

- [05 §3](../arquitetura/05-coletor.md#3-três-decisões-que-mudam-o-resultado); testes `remake_fica_de_fora`, `rota_em_branco_derruba_a_partida_inteira`, `rotas_que_nao_fecham_cinco_por_time_ficam_de_fora`.
