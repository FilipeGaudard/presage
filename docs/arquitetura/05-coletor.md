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

Outras opções: `--jogadores`, `--por-jogador`, `--fila`, `--patch`,
`--min-jogos`, `--paralelo`, `--limite`, `--saida`, `--estado` (`--ajuda` lista
todas). A saída padrão é onde o app procura — o teste
`a_saida_padrao_e_onde_o_app_procura` trava isso, e falha fora do Windows porque
monta o caminho a partir de `LOCALAPPDATA`.

A chave sai de `PRESAGE_RIOT_API_KEY` e de lugar nenhum mais — nunca de
argumento, porque argumento aparece na lista de processos, e nunca da URL, porque
URL aparece em log de proxy. Ela viaja no cabeçalho `X-Riot-Token` (RNF-6).

## 2. Como ele coleta

1. **Semeia** jogadores pelo `league-v4`, de um tier mínimo para cima. Até
   mestre, paginando as quatro divisões; de mestre para cima, a lista única.
2. **Lista** as últimas partidas ranqueadas de cada jogador semeado pelo
   `match-v5`.
3. **Baixa** cada partida ainda não vista, em lotes concorrentes.
4. **Agrega** em memória (`aggregate.rs`) e grava estado e base a cada 500
   partidas.

O que entra na conta de cada partida aceita:

- **base**: uma linha por participante, `campeão:rota`;
- **matchup**: o par de cada rota, nas duas direções — `A:B:rota` conta a vitória
  de A, `B:A:rota` a de B. É direcional de propósito, porque é assim que a chave é
  lida no app;
- **sinergia**: todo par de companheiros, sem direção (o menor id primeiro), dez
  por time, vinte por partida.

## 3. Três decisões que mudam o resultado

Detalhe e alternativas em [ADR-0006](../adr/0006-coletor-sem-bola-de-neve-um-patch-partida-inteira.md).

**Não há bola de neve.** Dava para pegar os dez jogadores de cada partida baixada
e expandir a partir deles — cresce muito mais rápido. Mas aí a amostra escorrega
para fora do elo semeado, e o `tier` do cabeçalho passa a mentir sobre de quem
são aquelas partidas. Só os jogadores semeados são expandidos.

**Um patch por base.** A primeira partida aceita trava o patch; as de outro são
recusadas e contadas no relatório. Misturar patch é o jeito mais fácil de uma
base parecer grande e recomendar item que mudou de preço.

**Partida ou entra inteira ou não entra.** Remake, partida terminada por erro,
modo que não é a Fenda, rota em branco, rotas que não fecham cinco por time, menos
de cinco minutos: tudo descartado com motivo (`Skip`), antes de qualquer
contagem. Meia partida contada é pior que partida nenhuma, e uma base que engole
remake mede taxa de remake.

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

O coletor tem os próprios tipos crus da match-v5 e as próprias regras de
descarte; o histórico pessoal (`core/src/history.rs`) tem os seus. As regras são
as mesmas por desenho — mas são duas cópias. Até unificar, uma mudança numa
precisa da outra no mesmo commit ([L-4](09-licoes-do-prototipo.md)).

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
