# 05 — Segurança do frontend

O front roda numa webview do Tauri com código nosso, empacotado no instalador.
Ele não tem login, não guarda segredo e não fala com a Riot. O risco que sobra é
texto de fora virar código na webview, e a webview alcançar mais do processo
nativo do que precisa. O lado da casca, com ativos, chave da Riot e atualizador,
está em [arquitetura 07](../arquitetura/07-seguranca.md).

## 1. Controles

| Controle | Como | Causa que previne |
|---|---|---|
| Escape por padrão | Todo texto entra como filho JSX ou atributo, e o React escapa. Nenhum `dangerouslySetInnerHTML`, `innerHTML`, `eval` ou `new Function` em `src/` | Um nome de item, um motivo ou as notas de uma versão viram script |
| Nenhuma rede própria além do Data Dragon | O único `fetch` é `lib/ddragon.ts`, para `ddragon.leagueoflegends.com` | O front vazar dado ou depender de um terceiro a mais |
| CSP fechada | `tauri.conf.json`: script e tudo o mais só da própria origem; imagem e conexão também do Data Dragon (§4) | Script de fora; exfiltração para outro host |
| URL de imagem sobre host fixo | O retrato e o ícone de item são `https://ddragon.leagueoflegends.com/cdn/{versão}/img/…` (`ddragon.ts:40`, `:52`); a runa é o mesmo host mais o caminho que a casca manda (`Runas.tsx:9`) | Uma resposta estranha apontar a imagem para outro servidor; a CSP recusa de qualquer jeito |
| Sem link para fora | Nenhum `<a>`, `window.open` ou plugin de shell. As notas da versão (`update.body`) aparecem como texto | Notas de versão com link enganoso; navegar a webview para fora do app |
| Comandos fechados | A webview só alcança os 26 comandos de `generate_handler!`, cada um com tipo de argumento ([12](../arquitetura/12-contrato-da-casca.md)) | Um canal genérico para o processo nativo |
| Permissões mínimas de plugin | `capabilities/default.json`: `core:default`, `updater:default`, `process:allow-restart`. Nada de shell, sistema de arquivos ou HTTP pelo front | O front ler disco ou executar programa |
| Nenhum dado no navegador | Nenhum `localStorage`, `sessionStorage` ou IndexedDB; o que persiste vai por `set_*` para `%LOCALAPPDATA%\Presage` | Dado do jogador num segundo lugar, fora do que a casca controla |
| Riot ID de terceiros fora da tela | O `displayName` dos outros nove só vira `key` do React | Expor terceiros numa captura de tela ou numa transmissão |
| O simulador não confia na tela | O `DraftState` montado à mão passa por `DraftState::normalized` na casca, que refaz indisponíveis e a sua rota | Um draft incoerente chegar ao motor |
| Atualização só no clique | `downloadAndInstall` só roda em "Atualizar e reiniciar"; quem confere a assinatura é a casca ([arquitetura 07 §4](../arquitetura/07-seguranca.md#4-o-atualizador)) | Trocar o executável no meio de um draft (RNF-7) |
| Fontes locais | `font-family` cai em Inter, Segoe UI ou `system-ui`; nenhuma fonte web | Um CDN de fonte vendo quem abre o app |

## 2. Texto que não controlamos

O complemento da tabela de [arquitetura 07](../arquitetura/07-seguranca.md#entradas-que-não-controlamos),
do ponto de vista da tela.

| Origem | Exemplo | Onde aparece | O que segura |
|---|---|---|---|
| LCU | o seu Riot ID, a fase do timer (`BAN_PICK`) | barra, faixa "Sem base estatística", `DraftBoard` | escape do React |
| Live Client | `championKey` dos dez | `LiveOverview`, `LiveBuild`, `GoldTracker` (sem dado estático) | escape do React |
| Live Client | `displayName` dos dez | só `key` | não é desenhado |
| Data Dragon, pela casca | nome de item, nome e árvore de runa, caminho do ícone da runa | Partida, overlay, Rotas | escape; o caminho vai depois de um host fixo |
| Data Dragon, pelo front | nome e `id` do campeão | todo retrato e todo nome | escape; o `id` vai no caminho de uma URL de host fixo |
| `latest.json` do GitHub | `version` e `body` da versão nova | aviso de atualização | escape; nenhum link |
| Erro de conexão (`ConnStatus.detail`) | o texto do erro ao achar o cliente | `title` do indicador de conexão | escape; fica na máquina |
| Texto do core | motivos, notas, resumo, `because` | todas as abas | escape |

## 3. O que a webview pode chamar

- **Duas janelas, uma capability.** `default.json` vale para `main` e para
  `overlay`. O overlay ganha, sem precisar, `updater:default` e
  `process:allow-restart`.
- **Os comandos do app valem para as duas.** O projeto não declara manifesto de
  comandos (`build.rs` é só `tauri_build::build()`), então o Tauri deixa qualquer
  janela chamar os 26. O overlay usa cinco: `get_overlay_prefs`, `get_live`,
  `build_suggestions`, `gold_report`, `skirmish_focus`. Os outros, `set_pool` e
  `set_autostart` inclusive, também estão ao alcance dele.
- **Não há arrasto de janela.** `core:default` não inclui
  `core:window:allow-start-dragging`, e é por isso que, mesmo clicável, o overlay
  não se move ([04 §1](04-overlay.md#1-a-janela)).

As duas janelas carregam só o nosso código, então o risco é baixo; o defeito é
de mínimo privilégio ([GC-13](07-lacunas-da-casca.md)).

## 4. CSP

```
default-src 'self';
img-src 'self' https://ddragon.leagueoflegends.com data:;
connect-src 'self' ipc: http://ipc.localhost https://ddragon.leagueoflegends.com;
style-src 'self' 'unsafe-inline'
```

| Diretiva | Por quê | Observação |
|---|---|---|
| `default-src 'self'` | script, fonte e tudo o mais só do bundle | sem `unsafe-eval`, sem script de fora |
| `img-src` | retratos, ícones de item e de runa vêm do CDN | `data:` não é usado por nenhum arquivo de `src/`; o favicon `data:` é só da demonstração em arquivo único |
| `connect-src` | `ipc:` e `http://ipc.localhost` são o canal do Tauri; o Data Dragon é o `fetch` de `ddragon.ts` | — |
| `style-src 'unsafe-inline'` | estilo em linha | o React aplica `style={{ width: … }}` pelo CSSOM, que a CSP não barra, e nada em `src/` escreve `<style>` nem atributo `style` em HTML. Não foi conferido se algo mais precisa dela |

Fora da CSP, dois artefatos vão no instalador sem necessidade: os source maps
(`sourcemap: true` em `vite.config.ts`) e a página de demonstração com os dados
de exemplo ([01 §6](01-arquitetura.md#6-build-e-versão)).

## 5. Checklist de PR

- [ ] Nenhum `dangerouslySetInnerHTML`, `innerHTML`, `outerHTML`,
      `insertAdjacentHTML`, `eval` ou `new Function`.
- [ ] Nenhum `fetch` novo. Se for preciso, o host entra na CSP, em §1 e na
      tabela do [arquitetura 07](../arquitetura/07-seguranca.md#a-webview).
- [ ] Nenhuma URL montada com texto de fora sem host fixo na frente.
- [ ] Nenhum `<a href>` com dado, `window.open` ou plugin de shell.
- [ ] Componente novo não importa `@tauri-apps/*`; chamada nova à casca passa por
      `App.tsx` ou `overlay.tsx`.
- [ ] Comando novo entra no [12](../arquitetura/12-contrato-da-casca.md) e no
      `lib/types.ts`, com argumento tipado.
- [ ] Permissão nova em `capabilities/` vem com o motivo no PR e em §3.
- [ ] Nenhum dado de outro jogador desenhado: Riot ID, ouro em mão, posição
      (C-4).
- [ ] Número novo na tela tem etiqueta de origem ([00 §5](00-visao-e-escopo.md#5-cada-número-e-a-etiqueta-dele)).
- [ ] Nada em `localStorage`, `sessionStorage` ou IndexedDB.
- [ ] `demo.tsx` sem Riot ID, puuid ou partida de pessoa real.
