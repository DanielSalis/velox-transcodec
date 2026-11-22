# Instalação por usuário a partir do código-fonte no Windows 11.
[CmdletBinding()]
param(
    [string]$QMake = $env:QMAKE,
    [string]$FFmpegDirectory = ''
)
$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

function Find-Executable([string]$Name) {
    $command = Get-Command $Name -CommandType Application -ErrorAction SilentlyContinue | Select-Object -First 1
    if (-not $command) { throw "Não foi encontrado $Name no PATH." }
    return $command.Source
}
function Invoke-Checked([string]$File, [string[]]$Arguments) {
    & $File @Arguments
    if ($LASTEXITCODE -ne 0) { throw "$File falhou (código $LASTEXITCODE)." }
}

if ([Environment]::OSVersion.Version.Build -lt 22000) { throw 'Este instalador requer Windows 11.' }
if (-not $env:LOCALAPPDATA -or -not $env:APPDATA) { throw 'As pastas da conta do usuário não estão disponíveis.' }
if ($env:SLINT_NO_QT) { throw 'Remova SLINT_NO_QT do ambiente para compilar com o backend Qt.' }
if ($PSScriptRoot -like '\\*') { throw 'O código está em um caminho de rede/WSL. Copie o repositório para uma pasta local do Windows e execute o instalador nessa cópia; Cargo e ferramentas MSVC podem não funcionar nesse caminho.' }
. (Join-Path $PSScriptRoot 'bootstrap-windows.ps1')
$missing = @()
if (-not (Get-Command 'cargo.exe' -CommandType Application -ErrorAction SilentlyContinue)) { $missing += 'Rust e Cargo (cargo.exe no PATH)' }
if (-not $QMake -and -not (Get-Command 'qmake.exe' -CommandType Application -ErrorAction SilentlyContinue)) { $missing += 'Qt 6 para MSVC (qmake.exe no PATH ou -QMake)' }
if (-not $FFmpegDirectory -and -not (Get-Command 'ffmpeg.exe' -CommandType Application -ErrorAction SilentlyContinue)) { $missing += 'FFmpeg e ffprobe (no PATH ou -FFmpegDirectory)' }
if (-not (Get-Command 'cl.exe' -CommandType Application -ErrorAction SilentlyContinue)) { $missing += 'compilador C++ MSVC (abra o terminal de desenvolvimento do Visual Studio)' }
if ($missing.Count) { throw "Dependências ausentes:`n - $($missing -join "`n - ")`nConsulte a seção Windows 11 do README e execute novamente." }
$cargo = Find-Executable 'cargo.exe'
if (-not $QMake) { $QMake = Find-Executable 'qmake.exe' }
$QMake = (Resolve-Path -LiteralPath $QMake).Path
$qtBin = Split-Path -Parent $QMake
$deploy = Join-Path $qtBin 'windeployqt.exe'
if (-not (Test-Path -LiteralPath $deploy -PathType Leaf)) { throw "windeployqt.exe não encontrado em $qtBin." }
$qtVersion = (& $QMake -query QT_VERSION | Select-Object -First 1)
if ($LASTEXITCODE -ne 0 -or $qtVersion -notmatch '^6\.') { throw 'É necessário um Qt 6 com qmake e windeployqt.' }
if (-not $FFmpegDirectory) { $FFmpegDirectory = Split-Path -Parent (Find-Executable 'ffmpeg.exe') }
$ffmpeg = Join-Path $FFmpegDirectory 'ffmpeg.exe'
$ffprobe = Join-Path $FFmpegDirectory 'ffprobe.exe'
if (-not (Test-Path -LiteralPath $ffmpeg -PathType Leaf) -or -not (Test-Path -LiteralPath $ffprobe -PathType Leaf)) { throw 'ffmpeg.exe e ffprobe.exe devem estar na mesma pasta.' }
Invoke-Checked $ffmpeg @('-version')
Invoke-Checked $ffprobe @('-version')
$previousPreference = $ErrorActionPreference
try {
    $ErrorActionPreference = 'Continue'
    $encoders = & $ffmpeg -hide_banner -encoders 2>&1 | Out-String
} finally { $ErrorActionPreference = $previousPreference }
if ($LASTEXITCODE -ne 0 -or $encoders -notmatch 'libx264|h264_nvenc|prores_ks') { throw 'O FFmpeg deve oferecer libx264, h264_nvenc ou prores_ks.' }

$root = $PSScriptRoot
$base = Join-Path $env:LOCALAPPDATA 'Programs'
$destination = Join-Path $base 'Velox Transcode'
$stage = Join-Path $base ('.velox-stage-' + [guid]::NewGuid().ToString('N'))
$backup = Join-Path $base ('.velox-backup-' + [guid]::NewGuid().ToString('N'))
$shortcut = Join-Path $env:APPDATA 'Microsoft\Windows\Start Menu\Programs\Velox Transcode.lnk'
$oldQMake = $env:QMAKE
$oldPath = $env:PATH
$installed = $false
try {
    $env:QMAKE = $QMake
    $env:PATH = "$qtBin;$oldPath"
    Push-Location $root
    try { Invoke-Checked $cargo @('build', '--release', '--locked') } finally { Pop-Location }
    New-Item -ItemType Directory -Path $stage -Force | Out-Null
    Copy-Item -LiteralPath (Join-Path $root 'target\release\velox-transcode.exe') -Destination $stage
    Invoke-Checked $deploy @('--release', '--no-translations', (Join-Path $stage 'velox-transcode.exe'))
    $mediaDir = Join-Path $stage 'ffmpeg'
    New-Item -ItemType Directory -Path $mediaDir | Out-Null
    Copy-Item -LiteralPath $ffmpeg, $ffprobe -Destination $mediaDir
    Get-ChildItem -LiteralPath $FFmpegDirectory -Filter '*.dll' -File | Copy-Item -Destination $mediaDir
    if (Test-Path -LiteralPath $destination) { Move-Item -LiteralPath $destination -Destination $backup }
    try { Move-Item -LiteralPath $stage -Destination $destination } catch {
        if (Test-Path -LiteralPath $backup) { Move-Item -LiteralPath $backup -Destination $destination }
        throw
    }
    try {
        $shell = New-Object -ComObject WScript.Shell
        $link = $shell.CreateShortcut($shortcut)
        $link.TargetPath = Join-Path $destination 'velox-transcode.exe'
        $link.WorkingDirectory = $destination
        $link.Description = 'Velox Transcode'
        $link.Save()
        $installed = $true
    } catch {
        Remove-Item -LiteralPath $destination -Recurse -Force
        if (Test-Path -LiteralPath $backup) { Move-Item -LiteralPath $backup -Destination $destination }
        throw
    }
    if ($installed -and (Test-Path -LiteralPath $backup)) { Remove-Item -LiteralPath $backup -Recurse -Force }
    Write-Host "Instalado em $destination. Abra Velox Transcode pelo menu Iniciar."
} finally {
    $env:QMAKE = $oldQMake
    $env:PATH = $oldPath
    if (Test-Path -LiteralPath $stage) { Remove-Item -LiteralPath $stage -Recurse -Force }
}
