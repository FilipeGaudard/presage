# Referência — iTero AI Coach

O que o iTero faz, como faz, e o que disso o Presage tem. Serve para decidir o
que construir; não é especificação a seguir.

| | |
|---|---|
| Produto | "The iTero AI Coach" (ITERO GAMING LTD), app Overwolf-Electron |
| Versão analisada | 3.11.7 (Electron 42.7.1), notas de versão de 15/09/2026 |
| Lido em | 2026-10-28 |

## Documentos

| Doc | Conteúdo |
|---|---|
| [01 — Processo principal](01-processo-principal.md) | inicialização, pacotes Overwolf, stores, máquina de fases, champ select e draft, pós-partida, dados de configuração, configurações do usuário, eventos ao vivo, atalhos |
| [02 — Janelas](02-janelas.md) | as 16 janelas: tipo, tamanho, quando abrem, o que mostram |
| [03 — IPC e ponte](03-ipc-e-ponte.md) | o que o preload expõe, todos os canais, as stores por IPC |
| [04 — Telas da janela principal](04-telas-da-janela-principal.md) | as 39 rotas e cada tela |
| [05 — Funcionalidades](05-funcionalidades.md) | catálogo `I-1`…`I-65` e o scouting de jogadores |
| [06 — Dados e backend](06-dados-e-backend.md) | fontes, endpoints e corpos, esquemas, LCU, Firestore, o que fica na máquina |
| [07 — Cruzamento](07-cruzamento.md) | cada funcionalidade contra o Presage, e as lacunas que cabem, por valor |

## Método

1. O pacote `app.asar` foi listado e extraído fora do repositório. **Nada do
   bundle foi executado**: só lido como texto, por scripts próprios rodando com
   `python3 -I`.
2. **Processo principal** (`dist-electron/main.js`): vem formatado, com nomes
   minificados e rótulos de log legíveis. Os blocos do próprio iTero foram
   separados das bibliotecas (Electron, Zod, Firebase…) e lidos inteiros.
3. **Interface** (`dist/assets/*.js`, 155 chunks minificados): o mapa de rotas
   sai do roteador; o texto de cada tela, das chaves de tradução resolvidas pelo
   recurso em inglês embutido; a lógica dos overlays, lendo cada chunk de
   entrada.
4. **Não foi observado em execução**: nenhuma chamada ao backend do iTero foi
   feita. O que o servidor calcula (o modelo de draft, o coach, o scouting) está
   descrito pelo que o cliente envia, recebe e exibe, e pelo que o próprio guia
   do iTero afirma.

O que **não** está aqui, de propósito: código do iTero, hosts e ids de
distribuição, o token público da loja, e qualquer chave. Os números citados
(limiares, durações) são comportamento observável do produto.

## Prova de cobertura

`conferir.py` tira do bundle as listas que estes docs prometem cobrir e falha se
algum item não aparece nos `.md`; também confere que todo `I-n` do catálogo
aparece uma vez no cruzamento e que a contagem por situação bate.

```
python3 -I docs/referencia/itero/conferir.py <diretório-extraído>
```

Resultado em 2026-10-28:

| Lista | Itens |
|---|---|
| janelas | 16/16 |
| rotas da janela principal | 39/39 |
| canais do preload | 46/46 |
| canais tratados no processo principal | 42/42 |
| canais enviados às janelas | 10/10 |
| stores | 16/16 |
| recursos do GEP (launcher + jogo) | 21/21 |
| endpoints do backend | 10/10 |
| caminhos do LCU | 8/8 |
| configurações do usuário | 53/53 |
| chaves da store `config` | 44/44 |
| eventos ao vivo tratados | 33/33 |
| cruzamento | 65 funcionalidades, 65 linhas |

Um teste de mutação (apagar uma janela, um endpoint e uma linha do cruzamento
numa cópia) fez as três listas falharem.

O que o script **não** prova: que a descrição de cada item está certa. Isso
depende da leitura, e as citações `main.js:<linha>` e `<chunk>.js` existem para
que qualquer item possa ser conferido no bundle.
