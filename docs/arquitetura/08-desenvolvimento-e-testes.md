# 08 — Desenvolvimento e testes

## 1. Rodando

```powershell
npm install
npm run tauri dev
```

Abra o cliente do League em seguida. O Presage procura a cada 3 segundos e acorda
sozinho quando um champ select começa.

```powershell
cargo test -p presage-core      # o motor inteiro, em segundos
cargo test -p presage-ingest    # o coletor
npm run build                   # tsc --noEmit + vite build
```

## 2. Olhar a interface sem o cliente

Os componentes são puramente visuais — nenhum fala com o Tauri — então há uma
página que os renderiza com dados de exemplo:

```powershell
npm run dev
```

Depois abra `http://localhost:1420/demo.html`. Serve para trabalhar no visual sem
partida em andamento, e **funciona numa máquina sem MSVC e sem WebKitGTK**, já
que é só a parte web. Todo componente novo entra no `demo.tsx` com dados de
tamanho realista.

## 3. O que roda em cada máquina

| Máquina | Core e coletor | Front e demo | App Tauri | Instalador |
|---|---|---|---|---|
| Windows com MSVC | sim | sim | sim | sim |
| Windows só com MinGW | sim | sim | **não abre** | não |
| Linux | sim (o teste da saída padrão do coletor falha) | sim | só com `libwebkit2gtk-4.1-dev`; e não acha o cliente da Riot | não |
| GitHub Actions `windows-latest` | sim | sim | — | sim |

### Toolchain sem admin

O instalador oficial do Node e o Visual Studio Build Tools pedem elevação. Se a
TI não libera UAC, dá para montar tudo no perfil do usuário:

| O quê | Onde | Como |
|---|---|---|
| Node.js 24 | `%USERPROFILE%\tools\node-v24.22.1-win-x64` | zip oficial do nodejs.org, descompactado |
| MinGW-w64 | `%USERPROFILE%\tools\mingw64` | build WinLibs, serve de linker no lugar do MSVC |
| Rust | `%USERPROFILE%\.cargo` | `rustup-init.exe -y --default-host x86_64-pc-windows-gnu` |

Até onde o alvo GNU chega:

- **Compila.** O workspace inteiro, Tauri incluso.
- **Os testes do core passam.** Nativo, sem contorno.
- **O app não abre.** Binário que linka o Tauri sai com
  `STATUS_ENTRYPOINT_NOT_FOUND`: importa `WebView2Loader.dll` e import libs
  geradas pelo `dlltool` do binutils, que não batem com o loader.

Ou seja: **`npm run tauri dev` não roda numa máquina só com MinGW.** Para abrir o
app e gerar instalador é preciso o Visual Studio Build Tools, que exige admin. O
contorno GNU serve para escrever código com o compilador e os testes atrás, não
para executar.

Duas armadilhas que custaram tempo:

- **Caminho de build curto.** Com o `target/` em caminho longo o `dlltool` falha.
  Resolve com `$env:CARGO_TARGET_DIR = "C:\t\presage"`.
- **`npm install` não roda postinstall** nas versões novas do npm, e o esbuild
  precisa do dele. `npm install-scripts approve esbuild` — já está no
  `package.json` (`allowScripts`).

### Linux

Core, coletor e demo rodam. O app Tauri só compila com o WebKitGTK de
desenvolvimento instalado, e mesmo assim não acha o cliente da Riot, que é só
Windows. Para conferir a interface sem navegador aberto, o Vite em `:1420` e um
Chromium headless com `--remote-debugging-port` dirigido por um script CDP
fotografam qualquer aba da demo.

Ao derrubar esses processos, cuidado com `pkill -f <padrão>`: o padrão aparece na
linha de comando do próprio shell que o executa, e o shell se mata junto. Use
`pgrep -f "[v]ite"` — a classe de caracteres casa com o processo e não com a
linha que a contém.

## 4. Os testes

| Pacote | Estado esperado |
|---|---|
| `presage-core` | tudo passa, em qualquer máquina |
| `presage-ingest` | tudo passa no Windows; fora dele, `a_saida_padrao_e_onde_o_app_procura` falha ([L-18](09-licoes-do-prototipo.md)) |
| `src-tauri` | sem testes próprios; a lógica fica no core de propósito |
| front | `tsc --noEmit` no `npm run build` |

A contagem de testes não fica escrita aqui de propósito: ela muda a cada commit, e
um número que envelhece é o defeito de [L-7](09-licoes-do-prototipo.md).

O CI (`instalador.yml`) roda os testes antes de empacotar.

### Como os testes são escritos

- **Nomes em português, como frases**: `popularidade_nao_vira_matchup`. O nome é a
  regra que o teste prova.
- **Números conhecidos, não sorteio.** Os testes de shrinkage montam bases à mão
  (`Montada` em `engine/mod.rs`) e ruído fixo de média zero (`RUIDO` em
  `engine/profile.rs`).
- **Teste com dado, não só sem dado.** Sem base, todo componente estatístico sai
  multiplicado por zero, e um erro de referência passa invisível. Os testes de
  invariante sem dado estão certos, mas não bastam ([L-8](09-licoes-do-prototipo.md)).
- **Teste primeiro, contra o código velho.** Os três testes do
  [ADR-0008](../adr/0008-matchup-e-sinergia-contra-a-soma-das-forcas.md) e do
  [ADR-0009](../adr/0009-forca-do-shrinkage-derivada-do-desvio.md) foram rodados
  antes da correção e falharam com +2,59pp, +0,86pp e +8,33pp — o tamanho
  previsto do defeito.
- **Teste que trava uma limitação.** O teste do ban não pessoal deve falhar
  quando o matchup entrar: é o aviso de que a limitação acabou
  ([ADR-0013](../adr/0013-ban-nao-personalizado-enquanto-aditivo.md)).
- **Teste que trava o formato de fora.** Cada fronteira de JSON cru tem um teste
  com os nomes de campo da Riot (`le_os_nomes_de_campo_da_riot`) — o que trava o
  que assumimos, não o que a Riot manda ([P-5](../produto/perguntas-em-aberto.md)).
