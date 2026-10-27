# Gera o icone do Presage em todos os tamanhos que o Windows pede.
#
# Por que gerar em vez de redimensionar um PNG: o logo e geometria simples
# (tres circulos e tres tracos), e desenhar direto em cada tamanho sai nitido.
# Reduzir um 256x256 para 16x16 borra os tracos ate virarem cinza.
#
# O icon.ico anterior tinha uma imagem so, de 256x256. O Windows aceita, mas
# precisa inventar as versoes pequenas - e em lista, barra de tarefas e no
# proprio instalador o resultado fica sujo ou cai no icone generico.
#
# Uso:  .\scripts\icone.ps1

$ErrorActionPreference = "Stop"
Add-Type -AssemblyName System.Drawing

$raiz = Split-Path -Parent $PSScriptRoot
$destino = Join-Path $raiz "src-tauri\icons"
New-Item -ItemType Directory -Force -Path $destino | Out-Null

# Os tamanhos que o Explorer, a barra de tarefas e o NSIS consultam.
$TAMANHOS = @(16, 24, 32, 48, 64, 128, 256)

$FUNDO = [System.Drawing.Color]::FromArgb(255, 6, 8, 15)      # --ink
$MARCA = [System.Drawing.Color]::FromArgb(255, 123, 108, 255) # --halo

# A mesma geometria do logo na interface e no site, em coordenadas de 64.
function Desenhar([System.Drawing.Graphics]$g, [double]$lado) {
  $e = $lado / 64.0

  $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
  $g.Clear($FUNDO)

  # Os dois ramos nao escolhidos ficam esmaecidos: e o minimax desenhado.
  $fraco = [System.Drawing.Color]::FromArgb(128, $MARCA.R, $MARCA.G, $MARCA.B)
  $canetaFraca = New-Object System.Drawing.Pen($fraco, [float](5 * $e))
  $canetaFraca.StartCap = [System.Drawing.Drawing2D.LineCap]::Round
  $canetaFraca.EndCap = [System.Drawing.Drawing2D.LineCap]::Round

  $canetaForte = New-Object System.Drawing.Pen($MARCA, [float](5 * $e))
  $canetaForte.StartCap = [System.Drawing.Drawing2D.LineCap]::Round
  $canetaForte.EndCap = [System.Drawing.Drawing2D.LineCap]::Round

  $g.DrawLine($canetaFraca, [float](32*$e), [float](38*$e), [float](16.5*$e), [float](25*$e))
  $g.DrawLine($canetaFraca, [float](32*$e), [float](38*$e), [float](47.5*$e), [float](25*$e))
  $g.DrawLine($canetaForte, [float](32*$e), [float](57*$e), [float](32*$e), [float](17*$e))

  $pincelFraco = New-Object System.Drawing.SolidBrush($fraco)
  $pincelForte = New-Object System.Drawing.SolidBrush($MARCA)

  function Bolinha($pincel, $cx, $cy, $r) {
    $g.FillEllipse($pincel, [float](($cx - $r) * $e), [float](($cy - $r) * $e), [float](2 * $r * $e), [float](2 * $r * $e))
  }

  Bolinha $pincelFraco 14 22 4.5
  Bolinha $pincelFraco 50 22 4.5
  Bolinha $pincelForte 32 11 6

  $canetaFraca.Dispose(); $canetaForte.Dispose()
  $pincelFraco.Dispose(); $pincelForte.Dispose()
}

function PngBytes([int]$lado) {
  $bmp = New-Object System.Drawing.Bitmap($lado, $lado, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  Desenhar $g $lado
  $g.Dispose()

  $ms = New-Object System.IO.MemoryStream
  $bmp.Save($ms, [System.Drawing.Imaging.ImageFormat]::Png)
  $bmp.Dispose()
  $bytes = $ms.ToArray()
  $ms.Dispose()
  return ,$bytes
}

# ---- os PNGs soltos que o tauri.conf tambem usa ----

foreach ($lado in 32, 128) {
  [System.IO.File]::WriteAllBytes((Join-Path $destino "${lado}x${lado}.png"), (PngBytes $lado))
}
[System.IO.File]::WriteAllBytes((Join-Path $destino "icon.png"), (PngBytes 256))

# ---- o .ico, com todos os tamanhos dentro ----
#
# Estrutura: cabecalho de 6 bytes, uma entrada de 16 bytes por imagem, e os
# dados no fim. Do Vista em diante cada imagem pode ser um PNG inteiro, que e
# o que fazemos - evita montar DIB com mascara de transparencia a mao.

$imagens = @{}
foreach ($lado in $TAMANHOS) { $imagens[$lado] = PngBytes $lado }

$saida = New-Object System.IO.MemoryStream
$w = New-Object System.IO.BinaryWriter($saida)

$w.Write([uint16]0)                 # reservado
$w.Write([uint16]1)                 # 1 = icone
$w.Write([uint16]$TAMANHOS.Count)

$offset = 6 + (16 * $TAMANHOS.Count)
foreach ($lado in $TAMANHOS) {
  $dados = $imagens[$lado]
  # 256 e gravado como 0: o campo tem um byte so.
  $w.Write([byte]($(if ($lado -eq 256) { 0 } else { $lado })))
  $w.Write([byte]($(if ($lado -eq 256) { 0 } else { $lado })))
  $w.Write([byte]0)                 # cores na paleta
  $w.Write([byte]0)                 # reservado
  $w.Write([uint16]1)               # planos
  $w.Write([uint16]32)              # bits por pixel
  $w.Write([uint32]$dados.Length)
  $w.Write([uint32]$offset)
  $offset += $dados.Length
}

foreach ($lado in $TAMANHOS) { $w.Write($imagens[$lado]) }

$w.Flush()
[System.IO.File]::WriteAllBytes((Join-Path $destino "icon.ico"), $saida.ToArray())
$w.Dispose(); $saida.Dispose()

Get-ChildItem $destino | Select-Object Name, @{n = "KB"; e = { [math]::Round($_.Length / 1KB, 1) } } | Format-Table -AutoSize
Write-Output "icone gerado com $($TAMANHOS.Count) tamanhos: $($TAMANHOS -join ', ')"
