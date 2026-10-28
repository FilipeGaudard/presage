# 09 — Lições do protótipo

Cada defeito encontrado, ligado ao que o **criou** e à prevenção estrutural. A
meta é tornar o erro difícil de cometer, não só corrigir o caso. O padrão que
mais aparece: **a mesma crença escrita em dois lugares**.

## 1. No motor

| # | Defeito | O que o causou | Prevenção | Onde |
|---|---|---|---|---|
| L-1 | Matchup e sinergia favoreciam o pick popular: com o rival em 47%, 5000 partidas sem efeito valiam +2,6pp, 100 partidas +0,3pp | O efeito era medido contra a base do candidato, mas a taxa crua já carrega a força do rival ou do aliado. O viés cresce com a amostra | Referência é a soma das forças em log-odds | [ADR-0008](../adr/0008-matchup-e-sinergia-contra-a-soma-das-forcas.md), `popularidade_nao_vira_matchup`, `sinergia_nao_herda_a_forca_do_aliado` |
| L-2 | 15V-5D em 20 partidas movia o score em +8,3pp, enquanto o `±` assumia 3,5pp de espalhamento | Força da Beta e desvio do prior eram constantes separadas — duas crenças sobre o mesmo efeito | A força sai do desvio (`strength_for_sd`) | [ADR-0009](../adr/0009-forca-do-shrinkage-derivada-do-desvio.md), `historico_pessoal_curto_nao_domina`, `forca_e_desvio_sao_a_mesma_crenca` |
| L-3 | Com muita amostra o `±` prometia certeza demais (0,41pp em 5000 partidas, contra 0,69pp da Beta) | O desvio do componente caía com 1/n, não com 1/√n | `posterior_sd`, derivado da mesma Beta | `desvio_posterior_cai_com_a_raiz_da_amostra` |
| L-4 | `lanes.rs` tinha a própria `MATCHUP_STRENGTH = 800`, comentada como "a mesma do draft" | Cópia de constante: nada obrigava as duas a andarem juntas | `lanes.rs` usa `MATCHUP_PRIOR_SD` do motor. Ainda há duas cópias do mesmo tipo: os pesos de composição de `rank()` e de `minimax` ([P-11](../produto/perguntas-em-aberto.md)), e as regras de descarte do coletor e de `history.rs` ([05 §5](05-coletor.md#5-a-duplicação-com-historyrs)) | [02 §4](02-motor-de-draft.md#4-fit-de-composição) |
| L-22 | A sugestão da pool ignorava a rota (a pool de atirador sem frente recebia tanque) e só via quatro buracos sim/não — uma pool equilibrada de três não recebia sugestão nenhuma | O candidato vinha de todos os campeões do jogo, e "buraco" era booleano: coberto uma vez, sumia | Filtro de rota (`plays`) e cobertura com retorno decrescente (`1 − ½ⁿ`), recalculada a cada lugar | [ADR-0016](../adr/0016-pool-de-oito-tres-escolhidos-cinco-por-cobertura.md), `so_entra_quem_joga_a_rota`, `pool_equilibrada_ainda_recebe_sugestao_ate_oito` |
| L-23 | O draft não recomendava pela rota: sem base, suporte e topo apareciam para o meio | `rank()` pontuava todos os campeões do jogo; a rota só entrava na busca da base, que dava 50% para todos | Candidatos do `DraftContext`: jogam a rota ou estão na pool | [ADR-0017](../adr/0017-uma-leitura-do-champ-select.md), `quem_nao_joga_a_rota_fica_de_fora` |
| L-24 | O "melhor contra-pick" do minimax saía sempre dos campeões de id baixo | A busca de resposta fazia `take(depth × 8)` sobre os ids em ordem crescente: 32 campeões, sempre os mesmos | Varre todas as respostas que jogam uma rota inimiga livre | `a_resposta_varre_todos_os_campeoes` |
| L-25 | O matchup de rota nunca disparava na ranqueada, e a análise de rota pareava contra o inimigo errado | O ranking exigia a posição do inimigo, que a ranqueada não informa; a análise pareava pela ordem de escolha, que não é rota | Rota inimiga inferida pela atribuição mais plausível (`role_fit`), uma vez, no contexto | `o_oponente_de_rota_e_achado_sem_posicao`, `a_rota_do_inimigo_sai_de_quem_ele_e_nao_da_ordem` |
| L-26 | O campeão que um aliado estava em hover podia ser recomendado; e a sua própria escolha ainda não travada sumia da sua lista | Três leituras do champ select: o ranking contava o hover, a indisponibilidade não; e `unavailable` incluía o seu `championId`, que o cliente preenche antes de travar | Uma leitura só: hover de aliado indisponível, a sua escolha não | `hover_de_aliado_nao_e_recomendado`, `a_sua_escolha_nao_travada_continua_sua` |
| L-8 | Nenhum teste pegou L-1 nem L-2 | Sem base, todo componente estatístico sai multiplicado por zero. Os testes provavam os invariantes sem dado, que estão certos, e nenhum montava uma base em que a referência importasse | Bases montadas à mão (`Montada`) com números conhecidos; teste rodado contra o código velho antes da correção | [08 §4](08-desenvolvimento-e-testes.md#como-os-testes-são-escritos) |
| L-19 | Um teste esperava que dois campeões com 2 pontos de diferença colapsassem na média, e o perfil puxou só 53% | A premissa do teste: com aquele ruído a diferença era z ≈ 1,46, e DerSimonian–Laird trata como parcialmente real qualquer coisa acima de z ≈ 1 | O estimador ficou; a tela ganhou a segunda trava — só ganha cor o que é claro a 95% | [04 §4](04-perfil-de-rota.md#quanto-encolhe-dersimonianlaird) |

## 2. No histórico pessoal

| # | Defeito | O que o causou | Prevenção | Onde |
|---|---|---|---|---|
| L-5 | O histórico pessoal contava remake e partida curta como vitória ou derrota | A busca lia só campeão e resultado, sem os cortes que o coletor já fazia | `history::played` aplica os mesmos cortes, para vitória e derrota também | `remake_nao_conta_nem_para_vitoria` |
| L-6 | Uma chamada limitada pela taxa baixava ~150 campos por participante e guardava dois | O componente pessoal foi desenhado em cima de vitória e derrota, o sinal mais pobre da partida | Comparação com o oponente direto em cinco métricas | [ADR-0010](../adr/0010-oponente-direto-como-grupo-de-controle.md), [04](04-perfil-de-rota.md) |
| L-9 | O cache antigo marcaria como vistas partidas que nunca tiveram comparação de rota | `seen` registra "já busquei", não "já tenho tudo o que preciso dela" | Arquivo novo, `personal-v2.json`; a primeira atualização refaz o histórico | [04 §7](04-perfil-de-rota.md#7-o-cache) |

## 3. Na documentação

| # | Defeito | O que o causou | Prevenção |
|---|---|---|---|
| L-7 | O README dizia que um matchup de dez partidas move a estimativa "em menos de meio ponto" — verdade com força 800, falso com 277 | O número estava escrito em dois lugares (o README e um comentário do coletor), longe da constante que o produz | Um fato num lugar só: o README da raiz aponta para `docs/`, e cada doc aponta para o código. Uma verificação automática doc↔código, como a de outros projetos, é o passo seguinte |

## 4. Armadilhas da API e do ambiente

| # | Armadilha | Prevenção | Onde |
|---|---|---|---|
| L-10 | O time inimigo vem do LCU com `championId` zerado: o campeão só aparece na ação concluída | `to_draft_state` lê das ações; teste cobre | [00 §2](00-contexto-e-limites.md#por-que-a-tradução-fica-isolada) |
| L-11 | O bloco `stats` de item do Data Dragon não tem penetração, letalidade nem haste | Parsear a descrição em en_US | [ADR-0015](../adr/0015-atributos-de-item-pela-descricao.md) |
| L-12 | A barra de cima ficou dizendo `0.1.0` depois de um bump de versão | `__APP_VERSION__` injetado pelo Vite a partir do `package.json` | [06 §6](06-distribuicao-e-atualizador.md#6-a-versão) |
| L-13 | Com o `target/` em caminho longo, o `dlltool` falha | `CARGO_TARGET_DIR` curto | [08 §3](08-desenvolvimento-e-testes.md#toolchain-sem-admin) |
| L-14 | O npm novo não roda o postinstall do esbuild | `allowScripts` no `package.json` | [08 §3](08-desenvolvimento-e-testes.md#toolchain-sem-admin) |
| L-15 | O alvo GNU compila o app Tauri, mas o binário não abre (`STATUS_ENTRYPOINT_NOT_FOUND`) | MSVC para abrir e empacotar; GNU só para compilar e testar | [08 §3](08-desenvolvimento-e-testes.md#toolchain-sem-admin) |
| L-16 | O retrato de campeão esticava conforme o texto ao lado | `.face` com quadrado fixo em CSS | `src/styles.css` |
| L-17 | Id numérico de runa muda de patch e vira ícone errado em silêncio | Runas pela chave textual do Data Dragon | [03 §9](03-motor-da-partida.md#9-runas) |
| L-18 | O teste da saída padrão do coletor falha fora do Windows | **Aberto.** O caminho sai de `LOCALAPPDATA`; o teste assume Windows | [05 §1](05-coletor.md#1-uso) |
| L-20 | O workflow dizia que sem a chave "o build sai sem o `.sig`". O Tauri 2.12 falha: a chave pública está no `tauri.conf.json` e a privada não veio. O workflow nunca tinha rodado, então ninguém viu | Uma afirmação sobre ferramenta de fora, escrita sem rodar a ferramenta | Tag sem a chave falha no primeiro passo; disparo manual sem a chave compila com `--no-sign` (em bash, porque o `npm.ps1` engole o `--`) | [06 §1](06-distribuicao-e-atualizador.md#1-gerando-o-instalador), `instalador.yml` |
| L-21 | Script em Python contra a Riot recebe `403` com o corpo `error code: 1010` — parece chave recusada e não é | O Cloudflare na frente da Riot bloqueia a assinatura padrão do `urllib` (`Python-urllib/3.x`). Com `User-Agent` próprio, vazio ou ausente a requisição chega à Riot. O `reqwest` do app, que não manda `User-Agent`, passa; o coletor manda o seu | Script de conferência sempre com `User-Agent` próprio. Erro da Riot de verdade vem em JSON (`{"status": …}`); `error code: 1010` é o Cloudflare | [P-5](../produto/perguntas-em-aberto.md) |
