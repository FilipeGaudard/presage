# 05 — Coletor

`coletor`. `tools/ingest` é um binário à parte. Não vai no instalador e o app não
depende dele — é a ferramenta que roda numa máquina (ou num servidor) para
produzir a base de partidas que o app lê ([02 §8](02-motor-de-draft.md#8-a-base-de-partidas)).

## 1. Uso

Para compilar: `cargo build --release -p presage-ingest`. A pasta `ferramentas\`,
onde o binário costuma ficar, está no `.gitignore`.

```powershell
# o que ele faria, sem rede e sem chave
.\ferramentas\presage-ingest.exe --plano

# a coleta de verdade
$env:PRESAGE_RIOT_API_KEY = "RGAPI-..."
.\ferramentas\presage-ingest.exe --regiao br1 --tier emerald

# continuar depois de uma interrupção
.\ferramentas\presage-ingest.exe --retomar

# relatório de cobertura de uma base já pronta
.\ferramentas\presage-ingest.exe --verificar "$env:LOCALAPPDATA\Presage\base-de-partidas.json"
```

As opções, como `ler_opcoes` as lê (`tools/ingest/src/main.rs`). Uma opção que
ele não conhece é erro, e a mensagem traz a ajuda.

| Opção | Padrão | Faz |
|---|---|---|
| `--regiao` | `br1` | Host de plataforma. Fora da lista de `Region::platforms`, o coletor para e mostra as válidas |
| `--tier` | `EMERALD` | Elo mínimo; tudo acima entra junto (`tiers_from`). Lido em maiúsculas. Tier desconhecido é erro |
| `--jogadores` | `20000` | Quantos jogadores semear. A semeadura para ao bater a cota, e pode parar ainda no primeiro tier |
| `--por-jogador` | `20` | Quantas partidas pedir a cada jogador semeado (`count` do `match-v5`) |
| `--fila` | `420` | A fila da base: 420 (solo/duo) ou 440 (flex). Escolhe a liga semeada (`liga_da_fila`: `RANKED_SOLO_5x5` ou `RANKED_FLEX_SR`), vai na busca (`queue=`, com `type=ranked`) e no descarte (`Skip::Fila`). Outra fila é erro na partida, antes de coletar (A28) |
| `--patch` | o da 1ª partida aceita | Trava o patch (`15.21`). Partida de outro patch é descartada (`Skip::Patch`) |
| `--dias` | desligado | Janela em dias: a lista de partidas de cada jogador pede só as que começaram nos últimos N dias (`startTime`, `ids_path`), e a base aceita qualquer patch, guardando todos em `Dataset::patches` e o mais novo em `patch`. Vence `--patch`. Zero é erro |
| `--min-jogos` | `20` | Na hora de gravar, corta as entradas com menos partidas que isto (§7). O total de partidas do cabeçalho não é podado |
| `--paralelo` | `8` | Quantas partidas baixar ao mesmo tempo, por lote. Abaixo de 1 vira 1 |
| `--limite` | `20:1,100:120` | Limite inicial, no formato do `X-App-Rate-Limit`; a primeira resposta da Riot o troca pelo real (§4). Limite ilegível é erro antes da primeira chamada |
| `--saida` | `%LOCALAPPDATA%\Presage\base-de-partidas.json` | Onde gravar a base. Sem `LOCALAPPDATA`, `.\base-de-partidas.json` |
| `--estado` | `%LOCALAPPDATA%\Presage\coleta-em-andamento.json` | Onde gravar o progresso (o arquivo de estado, §2) |
| `--retomar` | desligado | Continua do arquivo de estado, se ele for do mesmo recorte |
| `--plano` | desligado | Mostra semeadura, chamadas estimadas e horas pela taxa sustentável do `--limite`, sem rede e sem chave (§6) |
| `--verificar` | — | Lê uma base pronta e imprime a cobertura (§7). Vence as outras opções: com ela o coletor não coleta |
| `--ajuda` | — | Imprime a lista de opções e sai |
| `--help` | — | O mesmo que `--ajuda`, como `-h` |

A saída padrão é onde o app procura — o teste
`a_saida_padrao_e_onde_o_app_procura` trava isso, e falha fora do Windows porque
monta o caminho a partir de `LOCALAPPDATA` ([L-18](09-licoes-do-prototipo.md)).

A chave sai de `PRESAGE_RIOT_API_KEY` e de lugar nenhum mais — nunca de
argumento, porque argumento aparece na lista de processos, e nunca da URL, porque
URL aparece em log de proxy. Ela viaja no cabeçalho `X-Riot-Token` (RNF-6).

## 2. Como ele coleta

1. **Semeia** jogadores pelo `league-v4`, de um tier mínimo para cima. Até
   mestre, paginando as quatro divisões; de mestre para cima, a lista única.
2. **Lista** as últimas partidas ranqueadas de cada jogador semeado pelo
   `match-v5`.
3. **Baixa** cada partida ainda não vista, em lotes concorrentes.
4. **Agrega** em memória (`aggregate.rs`) e grava estado e base a cada 2000
   partidas baixadas, aceitas ou descartadas (`SALVAR_A_CADA`). A conta é
   conferida ao fim de cada jogador, então a gravação pode passar um pouco das
   2000. O estado carrega todo id já visto e chega a dezenas de megabytes: de 500
   em 500, no ritmo da chave de produção, seria uma gravação a cada dez
   segundos. Perder 2000 partidas numa queda custa minutos.

O que entra na conta de cada partida aceita:

- **base**: uma linha por participante, `campeão:rota`;
- **matchup**: o par de cada rota, nas duas direções — `A:B:rota` conta a vitória
  de A, `B:A:rota` a de B. É direcional de propósito, porque é assim que a chave é
  lida no app;
- **sinergia**: todo par de companheiros, sem direção (o menor id primeiro), dez
  por time, vinte por partida;
- **contra** (`versus`): todo par de adversários, em qualquer rota, nas duas
  direções — `A:B` conta a vitória de A com B no time inimigo. Vinte e cinco
  pares, cinquenta entradas por partida. É o que pesa o candidato contra o time
  inimigo inteiro, já que na ranqueada a rota do inimigo não se sabe;
- **duração** (`length`): uma linha por participante, `campeão:rota:faixa`, pela
  duração da partida (até 25:00, até 32:30, depois). É a única fonte honesta para
  a fase do jogo em que um campeão rende
  ([ADR-0018](../adr/0018-o-que-a-condicao-de-vitoria-pode-afirmar.md)).

### O arquivo de estado

`coleta-em-andamento.json` (`Estado`, `tools/ingest/src/main.rs`) é o que
sobrevive a uma interrupção:

| Campo | O que guarda |
|---|---|
| `regiao`, `tier`, `fila` | O recorte. `--retomar` só aceita o mesmo (`Estado::combina`) |
| `jogadores` | O puuid de cada jogador semeado, na ordem |
| `tiers_semeados` | Os tiers que de fato deram jogadores. O rótulo de elo do cabeçalho sai daqui: `EMERALD+` só quando a semeadura chegou ao topo |
| `proximo` | Até onde a lista de jogadores já foi consumida |
| `vistas` | Todo id de partida com resposta definitiva: contada, ou recusada pela Riot (404). Insistir numa partida que a Riot não entrega trava o crawl; já erro passageiro (rede, 5xx depois das tentativas, 429 seguidos) não marca, e a partida volta a ser pedida (A26) |
| `contagens` | As tabelas em memória (`Counts`): base, matchup, sinergia, contra, duração, itens (o inventário final, `item_key`), o total de partidas, `versus_matches`, `items_matches`, o patch travado e os descartes por motivo |

**Quando é gravado:** logo depois da semeadura, para uma queda no começo não
refazer a semeadura; a cada `SALVAR_A_CADA`, junto com a base; e no fim da
coleta. Sempre num `.parcial` ao lado e depois renomeado: um Ctrl-C no meio da
escrita não deixa arquivo meio escrito no lugar de um bom.

**Quando sai:** nunca. O coletor não o apaga. No fim da coleta ele fica com
`proximo` no fim da lista, e um `--retomar` sobre ele não baixa nada e só regrava
a base. Sem `--retomar`, a coleta começa do zero e o sobrescreve depois da
semeadura. Para recomeçar o mesmo recorte, apague o arquivo ou aponte `--estado`
para outro.

**Quando é recusado:** `--retomar` com estado de outro recorte para o coletor, e
estado de uma versão sem as tabelas de contra e duração também (§3). Estado que
não existe vira começo do zero; estado que existe e não desserializa para a
coleta com o motivo, em vez de virar começo do zero e ser sobrescrito depois da
semeadura (`ler_estado`, A27).

## 3. Três decisões que mudam o resultado

Detalhe e alternativas em [ADR-0006](../adr/0006-coletor-sem-bola-de-neve-um-patch-partida-inteira.md).

**Não há bola de neve.** Dava para pegar os dez jogadores de cada partida baixada
e expandir a partir deles — cresce muito mais rápido. Mas aí a amostra escorrega
para fora do elo semeado, e o `tier` do cabeçalho passa a mentir sobre de quem
são aquelas partidas. Só os jogadores semeados são expandidos.

**Um patch por base, a não ser com `--dias`.** A primeira partida aceita trava o
patch; as de outro são recusadas e contadas no relatório. Misturar patch é o
jeito mais fácil de uma base parecer grande e recomendar item que mudou de preço.
Mas no começo de um patch quase nada é dele: em 2026-11-08, travada no 16.22,
a coleta descartou 1.786 de 2.002 partidas (89%) por serem do 16.21, e a chave de
desenvolvimento acabaria com ~3 mil partidas. Com `--dias 30` a base junta os
patches da janela, diz quais são (`Dataset::patches`), e a tela mostra todos
("Base dos patches 16.21 e 16.22").

**Partida ou entra inteira ou não entra.** Fila diferente, modo que não é a
Fenda, partida terminada por erro, menos de dez participantes ou times que não
fecham cinco, remake, menos de cinco minutos, outro patch ou versão ilegível,
rota em branco, rotas que não fecham cinco por time: tudo descartado com motivo
(`Skip`, nove motivos), antes de qualquer contagem. Meia partida contada é pior que partida nenhuma, e uma base que engole
remake mede taxa de remake.

Um estado salvo por uma versão do coletor anterior às tabelas de contra,
duração e itens não é retomado: as tabelas novas ficariam com menos partidas do que o
cabeçalho diz (`Counts::resumable`, `versus_matches`). O coletor recusa e pede
para recomeçar.

Os motivos de descarte sobrevivem a uma retomada: um crawl de horas que recomeça
não pode perder a conta do que recusou.

## 4. O limite de taxa

Vem do cabeçalho `X-App-Rate-Limit` da própria resposta, não de constante nossa —
a primeira chamada revela o limite real da chave e o limitador (`limiter.rs`)
adota, mantendo o histórico do que já saiu. 429 para o limitador inteiro pelo
`Retry-After` que o servidor mandar; insistir depois de um 429 é como se perde a
chave.

Plataforma e região têm limitadores separados, porque a Riot conta as janelas por
host: `league-v4` fala com `br1.api.riotgames.com` e `match-v5` com
`americas.api.riotgames.com`.

Se regiões diferentes (`americas`, `europe`, `asia`, `sea`) são baldes separados,
coletar em várias multiplica a vazão. Ainda não testado
([P-1](../produto/perguntas-em-aberto.md)).

## 5. A duplicação com `history.rs`

O coletor tem os próprios tipos crus da match-v5 (`Match`, `Info`,
`Participant`) e o próprio `Skip`, em `aggregate.rs`; o histórico pessoal
(`core/src/history.rs`) tem os seus. Os cortes que valem para os dois saem de
uma função só, `history::corte_comum` (11 §5.3); os outros são de cada um:

| Regra | Coletor | Histórico pessoal |
|---|---|---|
| Modo `CLASSIC` | `Skip::Modo` | `Skip::Modo` |
| `endOfGameResult` vazio ou `GameComplete` | `Skip::Terminada` | `Skip::Terminada` |
| Ninguém com `gameEndedInEarlySurrender` | `Skip::Remake` | `Skip::Remake` |
| Pelo menos 300 s (`history::MIN_DURATION`) | `Skip::Curta` | `Skip::Curta` |
| A fila pedida | `Skip::Fila` | não confere: a casca já pede só a solo/duo |
| Um patch só | `Skip::Patch` | não há |
| Dez participantes, cinco por time | `Skip::Incompleta` | não confere |
| Rota em branco | descarta a partida (`Skip::SemRota`) | conta para vitória e derrota, sem comparação de rota |
| Rotas que fecham cinco por time | `Skip::RotaRepetida` | dois na sua rota do outro lado só tiram a comparação |

As quatro primeiras são `corte_comum`, chamada pelos dois, nesta ordem; o
coletor troca o motivo pelo seu `Skip` de mesmo nome, e só confere os dez
participantes depois delas. Mudar uma das quatro muda as duas bases, e
`os_cortes_comuns_sao_os_do_historico` (T-COLETOR-41) passa a mesma partida
pelos dois e confere o motivo de cada lado (A18). As outras diferem de
propósito: a base mede a partida inteira, o histórico mede você. Os tipos crus
continuam duas cópias ([L-4](09-licoes-do-prototipo.md)).

## 6. Quanto tempo leva, de verdade

| Chave | Taxa sustentável | ~260 mil chamadas |
|---|---|---|
| Development | 0,83/s (100 a cada 2 min) | 87 horas — e ela expira em 24 |
| Personal | 0,83/s, igual | 87 horas, ~3,6 dias — e ela **não** expira |
| Production | 50/s (30.000 a cada 10 min) | ~1,4 hora |

Com chave de desenvolvimento o crawl **não termina**, porque a chave morre antes.
O coletor roda com ela para testar o caminho, não para produzir a base. A
Personal resolve isso para o protótipo, por ser não comercial e não expirar
([P-6](../produto/perguntas-em-aberto.md)).

## 7. O que esperar da base

`--verificar` responde isso com números. O resumo:

- **Winrate base por rota** fica sólida com algumas centenas de milhares de
  partidas. São ~170 campeões × 5 rotas.
- **Matchup** é outra ordem de grandeza. Milhares de pares por rota, e para a
  maioria passar de n=200 são necessários alguns milhões de partidas.
- **Nada disso é tudo ou nada.** O motor encolhe o que tem amostra pequena até o
  ponto de não mexer no ranking, então uma base parcial recomenda tão bem quanto a
  parte dela que já tem amostra.

- **Contra** é a tabela mais larga — ~170 × 170 pares — e a mais rala: com ~200
  mil partidas sai uma média de ~170 jogos por par, e cada termo de contra move o
  ranking em no máximo ~0,3pp. O efeito aparece somado sobre os cinco inimigos.

O `--verificar` também **mede o espalhamento do contra**: o resíduo de cada par
contra o que as duas forças explicam, menos o ruído de amostragem (`τ² =
Var(resíduos) − média(p(1 − p)/n)`). É o número que deve ir para o
`VERSUS_PRIOR_SD` do motor — medido, não chutado. Com menos de 30 pares úteis ele
diz que ainda não dá para medir.

`--min-jogos` (padrão 20) corta as entradas magras na hora de gravar. Com a força
de prior do motor, um matchup de dez partidas pesa ~3,5% contra o prior: ocupa
espaço sem dizer nada. Cortar não perde informação; só deixa de carregar ruído.

## 8. O que a base destrava

Na ordem: variância de matchup (exposição a counter-pick de verdade no minimax),
ban personalizado por pool ([ADR-0013](../adr/0013-ban-nao-personalizado-enquanto-aditivo.md)),
detector de fraqueza pessoal, a rota de cada campeão e a cobertura de
confronto na pool ([02 §7](02-motor-de-draft.md#7-pool)), a referência por
campeão do perfil de rota
([04 §6](04-perfil-de-rota.md#6-o-que-ele-não-separa)), os espalhamentos do
shrinkage medidos em vez de escolhidos
([02 §2](02-motor-de-draft.md#força-e-desvio)) e o split real de dano físico
([02 §4](02-motor-de-draft.md#4-fit-de-composição)).
