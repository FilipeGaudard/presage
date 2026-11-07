## O que muda

<!-- O doc primeiro (seção e ID), depois o código. -->

## Prova

- T-IDs (linha no 13 escrita antes do teste):
- Teste que falhou antes da correção (para defeito, com a linha A-n do `build/audit.md`):
- Mutantes (mortos / equivalentes, com o motivo):

## Conferência

- [ ] `npm run doccheck -- -strict` verde, e o `build/ledger.md` deste commit é o gerado.
- [ ] Item público novo (tipo, função, comando, evento, arquivo) entrou no 10, 11 ou 12.
- [ ] Tipo novo na ponte tem título `↔` no 10 e espelho em `src/lib/types.ts`.
- [ ] Nenhum Riot ID, puuid ou chave no diff (`bash tools/dev/scan.sh`).
- [ ] Regra da Riot (C-*) afetada? Qual, e o teste que a guarda.
