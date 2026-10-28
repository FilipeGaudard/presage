# 06 — Distribuição e atualizador

## 1. Gerando o instalador

`tauri build` precisa do MSVC, e instalar MSVC pede administrador. Três caminhos,
em ordem de recomendação:

1. **GitHub Actions** — `.github/workflows/instalador.yml`. As máquinas
   `windows-latest` já têm MSVC. Dispare manualmente pela aba Actions, ou empurre
   uma tag `v*` e a release sai com o `.exe` e o `.msi` anexados. É de graça em
   repositório público e dá a URL que a página de download precisa.
2. **Qualquer máquina com administrador** — instale o Visual Studio Build Tools e
   rode `npm run tauri build`. Os instaladores saem em `target/release/bundle/`.
3. **MSVC portátil, sem administrador** — baixar os pacotes do compilador e do
   Windows SDK direto do manifesto da Microsoft e montar à mão
   (`scripts/msvc-portatil.mjs`, `scripts/ambiente-msvc.ps1`). Funciona, mas é o
   caminho mais trabalhoso dos três.

Sem certificado de assinatura de código, o SmartScreen avisa quem baixar. O
workflow já tem as variáveis prontas, comentadas, para quando o certificado
existir.

O executável linka a CRT estática ([ADR-0014](../adr/0014-crt-estatica-no-windows.md)),
então não depende do Visual C++ Redistributable instalado.

## 2. O atualizador

Sem ele, cada correção vira "baixe e reinstale à mão" — e numa ferramenta de LoL,
que precisa acompanhar o patch, isso é defeito de produto
([ADR-0005](../adr/0005-atualizador-assinado-com-endpoint-permanente.md)).

```powershell
$env:TAURI_SIGNING_PRIVATE_KEY = "$env:USERPROFILE\.presage\updater.key"
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = ""
npm run tauri build
npm run manifesto -- --url https://presage.gg --notas "o que mudou"
npm run entrega
```

O build assinado produz o instalador e um `.sig` ao lado. O `latest.json` carrega
a versão, a URL e essa assinatura. O app baixa o manifesto, compara a versão,
baixa o instalador e **confere a assinatura com a chave pública embutida nele**
antes de executar qualquer coisa. Sem a chave privada ninguém empurra um
executável para a base instalada — nem quem invadir a hospedagem.

### A chave privada

Está em `%USERPROFILE%\.presage\updater.key`, fora do repositório de propósito.

**Faça backup dela.** Perder essa chave significa que nenhuma instalação existente
volta a aceitar atualização: todo mundo teria de desinstalar e instalar de novo à
mão. Ela nunca vai para o repositório, nunca entra no instalador, e em CI vive
como secret. A pública fica em `tauri.conf.json`, em `plugins.updater.pubkey` —
essa é para ser vista.

Quem segura a chave privada executa código em toda instalação. É o ativo mais
sensível do projeto ([07 §1](07-seguranca.md#1-ativos)).

### O primeiro salto é manual

Uma versão sem atualizador não sabe atualizar. A 0.3.0 é a primeira que tem; quem
está em 0.1.0 ou 0.2.0 precisa baixar e instalar essa uma vez. Daí em diante é
automático.

Vale repetir para quem for distribuir: **não existe atualizador que conserte o
passado**. Quem instalou antes da 0.3.0 só sai de lá com uma instalação manual.

### O que o cliente vê

Uma faixa discreta no topo: *"Versão 0.4.0 disponível"*, com o que mudou, e dois
botões — atualizar agora ou depois. **Nunca instala sozinho**: trocar o
executável e reiniciar no meio de um draft de 30 segundos seria o pior momento
possível. E falha calada: sem internet, servidor fora do ar ou endpoint ainda não
configurado, o aviso simplesmente não aparece (RNF-7). No Windows a instalação
roda em modo `passive`.

## 3. Publicar uma versão nova

O endereço que o app consulta é a release do GitHub:

```
https://github.com/pedrojpgazzoni-coder/presage/releases/latest/download/latest.json
```

Para publicar, basta uma tag — o workflow faz o resto:

```powershell
# 1. subir a versão nos três arquivos (package.json, tauri.conf.json, Cargo.toml)
# 2. commitar
git tag v0.8.0
git push origin v0.8.0
```

O workflow compila, roda os testes, assina, gera o `latest.json` apontando para a
tag e publica a release. A partir daí todo Presage instalado vê a versão nova na
próxima vez que abrir.

**Dois secrets precisam existir no repositório**, senão a release sai sem
assinatura e nenhum app aceita o download:

| Secret | O que é |
|---|---|
| `UPDATER_PRIVATE_KEY` | o conteúdo de `%USERPROFILE%\.presage\updater.key` |
| `UPDATER_KEY_PASSWORD` | vazio, se a chave foi gerada sem senha |

E o repositório precisa ser **público**: asset de release privada exige
autenticação, e o app não tem como se autenticar.

Um fork que compile sem mudar `tauri.conf.json` gera um app que consulta a
release do repositório acima e só aceita a assinatura da chave dele. Isso é o
desenho funcionando, não um defeito: a instalação obedece a quem segura a chave.

## 4. O endpoint é permanente

`plugins.updater.endpoints` em `tauri.conf.json` é gravado **dentro do
executável**. Aquela instalação vai perguntar naquele endereço pelo resto da vida
dela, e não existe jeito de mudar isso depois — mudar o endereço exige uma versão
nova, que só chega pelo endereço antigo.

Distribuir com o placeholder, ou com um domínio que você não comprou, orfana todo
mundo que instalar: nunca mais recebem versão nova, e a única saída é desinstalar
e instalar à mão. Por isso `entrega.ps1` reclama alto quando acha `TROCAR` na
configuração, e testa por HTTP os endereços que sobraram.

O desenho prevê dois endereços, e o app tenta na ordem: o domínio próprio e um
espelho que não depende dele — GitHub Releases serve bem, é de graça e sobrevive a
troca de domínio:

```
https://github.com/<usuario>/<repo>/releases/latest/download/latest.json
```

Com o espelho no lugar, trocar de domínio depois deixa de ser catastrófico. Hoje
só o GitHub está configurado. O `latest.json` vai para a mesma pasta do site,
então a hospedagem que serve a página já serve a atualização.

## 5. A pasta de entrega

```powershell
npm run tauri build   # instalador (assinado, se a chave estiver no ambiente)
npm run demo          # a interface num arquivo só
npm run site          # a página num arquivo só
npm run manifesto -- --url https://presage.gg
npm run entrega       # junta tudo em Presage-entrega\
```

`Presage-entrega\` fica dentro do projeto e é montada do zero a cada vez —
`instalador\` e `site-para-hospedar\` são esvaziados antes de encher, senão o
instalador da versão anterior fica ao lado do novo e ninguém sabe qual é qual.

Dentro de `site-para-hospedar\` o instalador se chama **`Presage-setup.exe`**, sem
número de versão: é o nome para onde o botão da página aponta, e ele não pode
mudar a cada release. A cópia versionada fica em `instalador\`.

## 6. A versão

Está em três lugares que precisam andar juntos: `package.json`,
`src-tauri/tauri.conf.json` e o `version` do workspace em `Cargo.toml`. A
interface não repete o número — o Vite injeta `__APP_VERSION__` a partir do
`package.json`, porque a barra de cima já ficou dizendo `0.1.0` depois de um bump,
e versão que mente atrapalha mais que ajuda ([L-12](09-licoes-do-prototipo.md)).

É por ela que se sabe qual build está instalada: o número aparece no canto
superior esquerdo, ao lado do patch.
