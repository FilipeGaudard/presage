# Prepara o terminal para compilar com o MSVC portátil montado por
# scripts/msvc-portatil.mjs. Nada aqui mexe no registro nem pede elevação:
# são só variáveis de ambiente deste processo.
#
#   . .\scripts\ambiente-msvc.ps1
#   npm run tauri build

$raiz = "$env:USERPROFILE\tools\msvc"
if (-not (Test-Path $raiz)) {
  throw "MSVC portátil não encontrado. Rode: node scripts/msvc-portatil.mjs"
}

$msvc = (Get-ChildItem "$raiz\VC\Tools\MSVC" -Directory | Sort-Object Name | Select-Object -Last 1).Name
$sdk = (Get-ChildItem "$raiz\Windows Kits\10\Include" -Directory | Sort-Object Name | Select-Object -Last 1).Name

$vcBin = "$raiz\VC\Tools\MSVC\$msvc\bin\Hostx64\x64"
$sdkBin = "$raiz\Windows Kits\10\bin\$sdk\x64"

$env:INCLUDE = @(
  "$raiz\VC\Tools\MSVC\$msvc\include"
  "$raiz\Windows Kits\10\Include\$sdk\ucrt"
  "$raiz\Windows Kits\10\Include\$sdk\shared"
  "$raiz\Windows Kits\10\Include\$sdk\um"
  "$raiz\Windows Kits\10\Include\$sdk\winrt"
  "$raiz\Windows Kits\10\Include\$sdk\cppwinrt"
) -join ";"

$env:LIB = @(
  "$raiz\VC\Tools\MSVC\$msvc\lib\x64"
  "$raiz\Windows Kits\10\Lib\$sdk\ucrt\x64"
  "$raiz\Windows Kits\10\Lib\$sdk\um\x64"
) -join ";"

$node = "$env:USERPROFILE\tools\node-v24.22.1-win-x64"
$cargo = "$env:USERPROFILE\.cargo\bin"
$env:Path = "$vcBin;$sdkBin;$cargo;$node;$env:Path"

Write-Output "MSVC $msvc / SDK $sdk prontos"
Write-Output "  link.exe: $((Get-Command link.exe -ErrorAction SilentlyContinue).Source)"
