# ADR-0020 — O doc é o contrato, e o doccheck prova

- **Status:** aceito
- **Data:** 2026-11-07
- **Decisores:** time do Presage

## Contexto

- O defeito de documentação que mais aparece é o mesmo fato escrito em dois
  lugares, e um deles envelhecendo calado ([L-7](../arquitetura/09-licoes-do-prototipo.md)).
  O L-7 já pedia uma verificação automática doc↔código "como a de outros
  projetos" como o passo seguinte.
- Os docs explicavam bem o porquê do motor, mas não listavam o que existe:
  - metade dos tipos e das funções públicas do `presage-core` não aparecia em doc
    nenhum;
  - os 26 comandos e 6 eventos da casca apareciam só pelo nome;
  - nenhum catálogo ligava requisito a teste.
- `src/lib/types.ts` espelha à mão os tipos que a casca serializa. Mudar um lado
  sem o outro não quebra a compilação de nenhum dos dois: quebra em tempo de
  execução ([01 §4](../arquitetura/01-core-e-casca.md#4-o-front)).
- Não há cargo em toda máquina que mexe nos docs (o WSL de desenvolvimento não
  tem), mas há Node em todas.

## Decisão

**Os docs de referência são o contrato, e `tools/doccheck` prova a cada execução
que o código o cumpre, nos dois sentidos.** Do doc para o código, cada item tem
de existir. Do código para o doc, cada item público tem de estar documentado.

O que ele confere:

| Doc | O que vira item | Conferido contra |
|---|---|---|
| 10 e 11, blocos `rust` | tipo, função, método, constante | existe no módulo do título; mesma assinatura, mesmos campos `nome:tipo`, mesmas variantes, mesmo valor |
| 10, título com `↔` | tipo da ponte | campos do doc = campos do Rust; nomes serializados (serde) = campos da interface TS |
| `src/` (TS), comentário `Espelha \`core::…\`` | constante do front que copia o core | mesmo valor do `const` do core; numa lista, os `id` na ordem das variantes de `Tipo::ALL` (P3) |
| 12 | comando, evento, arquivo | `#[tauri::command]` registrado e chamado pelo front; evento emitido e ouvido; literal do arquivo no código |
| 05 | opção do coletor | o coletor lê |
| 13 | linha `T-…` | cada teste nomeado existe; todo `#[test]` está numa linha |
| requisitos §4 | linha da matriz | todo R/C/RNF tem linha, e ela cita T-IDs do catálogo |
| cruzamento do iTero | `I-n` | tem decisão (copiar, mudar, rejeitar, adiar, fora) e motivo |
| todos | link e ID citado | o caminho e a âncora existem; o ID está definido |

O que não resolve está **pendente**, a não ser que `tools/doccheck/deferred.txt`
o adie com motivo. O resultado vai para `build/ledger.md`. `-strict` falha com
qualquer pendente ou aviso, e roda em todo push (`docs.yml`).

O doccheck é Node sem dependência. Lê Rust e TypeScript como texto, com um
scanner, não com um compilador.

## Consequências

- **Positivas:**
  - um fato duplicado entre doc e código deixa de poder envelhecer calado:
    assinatura, campo, variante e valor de constante são comparados;
  - o espelho TS deixa de quebrar só em tempo de execução;
  - comportamento novo começa pela linha do teste no 13, e o doccheck mostra
    pendente até o código chegar.
- **Negativas e custos:**
  - toda função pública nova pede a entrada no 11 no mesmo commit;
  - um scanner erra onde um compilador não erraria: macro que declara item,
    item dentro de corpo de função. Os testes do próprio doccheck travam o que ele
    entende, e as contagens mínimas sobre o repositório real pegam quando ele
    perde itens;
  - o doccheck não prova comportamento. Comportamento é o teste, e o teste que
    consegue falhar é o mutante.
- **O que passa a ser obrigatório:**
  - `npm run doccheck -- -strict` verde antes de todo commit;
  - `node --test tools/doccheck/doccheck.test.mjs` verde ao mexer no doccheck;
  - adiamento só com motivo, e motivo que aponte um ID (P-n, A-n, ADR).

## Alternativas rejeitadas

| Alternativa | Por que não |
|---|---|
| Gerar a referência com `rustdoc` | Documenta o que existe, não o que foi combinado. Não diz unidade, origem, confiança nem requisito, não liga teste a requisito, e não confere o espelho TS |
| Doccheck em Rust com `syn` | Exato, mas só roda onde há cargo; o WSL de desenvolvimento e o runner Linux barato ficariam de fora |
| Conferir só nomes, como no roteirizador | Assinatura, campo e valor de constante são exatamente os fatos que envelhecem |
| Gerar os tipos TS a partir do Rust (ts-rs, specta) | Resolve o espelho, mas não o resto, e põe uma dependência no build da casca. Fica como opção se o espelho doer |

## Referências

- `tools/doccheck/doccheck.mjs`, `tools/doccheck/doccheck.test.mjs`,
  `tools/doccheck/deferred.txt`, `build/ledger.md`.
- [ADR-0021](0021-o-id-do-teste-nomeia-a-funcao.md),
  [ADR-0022](0022-tipos-da-ponte-conferidos-campo-a-campo.md).
