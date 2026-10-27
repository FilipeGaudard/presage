# Monta a pasta de entrega dentro do projeto.
#
# Junta num lugar só o que vai para outra pessoa: o instalador, a página num
# arquivo único, a interface num arquivo único e a pasta pronta para hospedar.
#
# O instalador entra também dentro de `site-para-hospedar`, com nome fixo,
# porque o botão de baixar da página aponta para ele por caminho relativo —
# hospedado, o botão funciona sem configurar nada.
#
# Uso:  .\scripts\entrega.ps1
# Pressupõe que `npm run tauri build`, `npm run demo` e `npm run site` já
# rodaram.

$ErrorActionPreference = "Stop"

$raiz = Split-Path -Parent $PSScriptRoot
$bundle = "C:\t\presage-msvc\release\bundle"
# Dentro do projeto, nao na Area de Trabalho: assim a pasta abre junto com o
# resto do codigo e nao fica solta no desktop.
$destino = Join-Path $raiz "Presage-entrega"

# Acha pelo padrao, nao pelo nome exato: o numero da versao muda a cada
# release, e um caminho fixo quebraria em silencio no proximo bump.
function Achar([string]$pasta, [string]$padrao) {
  $achado = Get-ChildItem (Join-Path $bundle $pasta) -Filter $padrao -ErrorAction SilentlyContinue |
    Sort-Object LastWriteTime -Descending |
    Select-Object -First 1
  if (-not $achado) {
    throw "nao achei $padrao em $pasta. Rode antes: npm run tauri build"
  }
  $achado.FullName
}

$setup = Achar "nsis" "Presage_*_x64-setup.exe"
$msi = Achar "msi" "Presage_*_x64_en-US.msi"

# O endereco de atualizacao fica gravado dentro do executavel e nao muda mais
# naquela instalacao. Distribuir com um placeholder significa orfanar todo
# mundo que instalar: eles nunca mais recebem versao nova, e a unica saida e
# reinstalar a mao. Por isso aqui e erro, nao aviso.
$conf = Get-Content (Join-Path $raiz "src-tauri\tauri.conf.json") -Raw | ConvertFrom-Json
$endpoints = @($conf.plugins.updater.endpoints)
$placeholders = $endpoints | Where-Object { $_ -match "TROCAR" }

if ($placeholders) {
  Write-Warning "ATUALIZADOR SEM ENDERECO REAL"
  Write-Warning "  $($placeholders -join "`n  ")"
  Write-Warning "Quem instalar este build nunca vai receber atualizacao."
  Write-Warning "Troque em src-tauri\tauri.conf.json e refaca o build antes de distribuir."
  Write-Warning "A pasta vai ser montada mesmo assim, para voce testar na sua maquina."
}

foreach ($url in $endpoints | Where-Object { $_ -notmatch "TROCAR" }) {
  try {
    Invoke-WebRequest -Uri $url -Method Head -TimeoutSec 8 -UseBasicParsing | Out-Null
    Write-Output "endpoint OK: $url"
  } catch {
    Write-Warning "endpoint NAO responde: $url"
  }
}

New-Item -ItemType Directory -Force -Path $destino | Out-Null

# Esvazia antes de encher. Sem isto, o instalador da versao anterior fica ao
# lado do novo e ninguem sabe qual e qual - que foi exatamente o problema que
# apareceu na primeira entrega.
foreach ($sub in "instalador", "site-para-hospedar") {
  $caminho = Join-Path $destino $sub
  if (Test-Path $caminho) {
    Remove-Item "$caminho\*" -Recurse -Force
  }
  New-Item -ItemType Directory -Force -Path $caminho | Out-Null
}

Copy-Item $setup "$destino\instalador\" -Force
Copy-Item $msi "$destino\instalador\" -Force

Copy-Item (Join-Path $raiz "Presage-site.html") $destino -Force
Copy-Item (Join-Path $raiz "Presage-demo.html") $destino -Force

Copy-Item (Join-Path $raiz "site\*") "$destino\site-para-hospedar\" -Force

# Duas copias do mesmo instalador, para dois leitores diferentes.
#
# `Presage-setup.exe` e para gente: o botao da pagina aponta para ele e o nome
# nao muda de release para release, entao o link nunca quebra.
#
# A versionada e para o atualizador: a URL dentro do latest.json carrega o
# numero, entao o arquivo e imutavel e nenhum cache entrega um binario velho
# para quem esta atualizando.
Copy-Item $setup "$destino\site-para-hospedar\Presage-setup.exe" -Force
Copy-Item $setup "$destino\site-para-hospedar\" -Force

# O manifesto que o app consulta. Sem ele a hospedagem serve o site mas
# ninguem atualiza, entao vale avisar alto.
$manifesto = Join-Path $raiz "latest.json"
if (Test-Path $manifesto) {
  # Um manifesto de teste aponta para localhost e anuncia uma versao que nao
  # existe. Publicar isso e pior que nao publicar nada: quem abrir o app fica
  # com um aviso de atualizacao que nunca baixa.
  $bruto = Get-Content $manifesto -Raw
  $dados = $bruto | ConvertFrom-Json
  $url = $dados.platforms.'windows-x86_64'.url

  if ($url -match "localhost|127\.0\.0\.1") {
    Write-Warning "latest.json aponta para $url - manifesto de teste, nao copiado."
  } elseif ($dados.version -ne $conf.version) {
    Write-Warning "latest.json anuncia $($dados.version) mas o build e $($conf.version) - nao copiado."
  } else {
    Copy-Item $manifesto "$destino\site-para-hospedar\" -Force
  }
} else {
  Write-Warning "latest.json nao existe: ninguem vai receber atualizacao. Rode 'npm run manifesto -- --url https://seu-dominio'"
}
# E a interface, que o botao "Ver a interface" procura ao lado do index.
Copy-Item (Join-Path $raiz "Presage-demo.html") "$destino\site-para-hospedar\demo.html" -Force
# O site em arquivo unico tambem, para quem quiser so pegar e mandar.
Copy-Item (Join-Path $raiz "Presage-site.html") "$destino\site-para-hospedar\" -Force

Get-ChildItem $destino -Recurse -File |
  Select-Object @{n = "arquivo"; e = { $_.FullName.Replace("$destino\", "") } },
                @{n = "KB"; e = { [math]::Round($_.Length / 1KB) } } |
  Sort-Object arquivo |
  Format-Table -AutoSize

Write-Output "pronto em $destino"
