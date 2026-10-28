# ADR-0005 — Atualizador assinado, com endpoint permanente

- **Status:** aceito
- **Data:** anterior à 0.7.0 (registrada em 2026-10-28)
- **Decisores:** time do Presage

## Contexto

- Uma ferramenta de LoL precisa acompanhar o patch; "baixe e reinstale à mão" é
  defeito de produto.
- Quem controla a atualização executa código em toda instalação.
- O endereço do manifesto fica gravado no executável.

## Decisão

**Atualizador do Tauri com assinatura obrigatória e manifesto em GitHub
Releases.**

- O build assinado gera o instalador e um `.sig`; o `latest.json` leva versão,
  URL e assinatura. O app confere com a chave pública embutida antes de executar.
- A chave privada fica fora do repositório (`%USERPROFILE%\.presage\updater.key`)
  e no CI como secret.
- O desenho prevê dois endpoints, tentados em ordem: domínio próprio e o espelho
  no GitHub Releases. Hoje só o GitHub está configurado.
- **Nunca instala sozinho** e falha calada.

## Consequências

- **Positivas:** invadir a hospedagem não basta para empurrar executável; uma tag
  publica a versão.
- **Negativas:**
  - perder a chave privada orfana a base: ninguém volta a aceitar atualização;
  - o endpoint é para sempre — mudar exige uma versão que só chega pelo endereço
    antigo;
  - o repositório precisa ser público, porque o app não se autentica para baixar
    asset de release privada;
  - não existe atualizador que conserte o passado: quem instalou antes da 0.3.0
    precisa de uma instalação manual.
- **Obrigatório:** backup da chave; `entrega.ps1` recusa `TROCAR` na configuração
  e testa os endereços por HTTP; versão em três arquivos andando juntos.

## Alternativas rejeitadas

| Alternativa | Por que não |
|---|---|
| Sem atualizador | Cada correção vira reinstalação manual |
| Atualizador sem assinatura | Quem invadir a hospedagem distribui o que quiser |
| Instalar sozinho em segundo plano | Reiniciar no meio de um draft de 30 segundos é o pior momento possível |
| Só o domínio próprio, sem espelho | Perder o domínio orfana todo mundo |

## Referências

- [06 — Distribuição e atualizador](../arquitetura/06-distribuicao-e-atualizador.md); [07 §4](../arquitetura/07-seguranca.md#4-o-atualizador); RNF-7.
