# Chamado pelo instalador no mesmo processo para preparar o ambiente de compilação.
function Update-InstallPath {
    $parts = @($env:PATH, [Environment]::GetEnvironmentVariable('Path', 'Machine'),
        [Environment]::GetEnvironmentVariable('Path', 'User'),
        (Join-Path $env:USERPROFILE '.cargo\bin'),
        (Join-Path $env:LOCALAPPDATA 'Microsoft\WinGet\Links'))
    $env:PATH = ($parts | Where-Object { $_ }) -join ';'
}
function Install-WingetPackage([string]$Id, [string[]]$ExtraArguments = @()) {
    if (-not (Get-Command winget.exe -CommandType Application -ErrorAction SilentlyContinue)) {
        throw 'O winget não foi encontrado. Instale ou atualize o App Installer da Microsoft Store e execute novamente.'
    }
    Write-Host "Instalando $Id com winget; o Windows pode solicitar autorização."
    & winget.exe install --id $Id --exact --source winget --accept-package-agreements --accept-source-agreements @ExtraArguments
    if ($LASTEXITCODE -ne 0) { throw "A instalação de $Id falhou (código $LASTEXITCODE). Verifique a saída do winget e execute novamente." }
    Update-InstallPath
}
function Import-MsvcEnvironment {
    if (Get-Command cl.exe -CommandType Application -ErrorAction SilentlyContinue) { return }
    $vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
    if (-not (Test-Path -LiteralPath $vswhere)) { return }
    $installation = & $vswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
    if (-not $installation) { return }
    $devcmd = Join-Path ($installation | Select-Object -First 1) 'Common7\Tools\VsDevCmd.bat'
    if (-not (Test-Path -LiteralPath $devcmd)) { return }
    $environment = & $env:ComSpec /d /s /c "`"`"$devcmd`" -no_logo -arch=x64 -host_arch=x64 >nul && set`""
    if ($LASTEXITCODE -ne 0) { throw 'Não foi possível configurar o ambiente de compilação MSVC.' }
    foreach ($line in $environment) {
        if ($line -match '^([^=]+)=(.*)$') { [Environment]::SetEnvironmentVariable($Matches[1], $Matches[2], 'Process') }
    }
}

if ($env:PROCESSOR_ARCHITECTURE -ne 'AMD64') { throw 'A instalação automática requer Windows x64; configure manualmente as dependências para outra arquitetura.' }
Update-InstallPath
if (-not (Get-Command cargo.exe -CommandType Application -ErrorAction SilentlyContinue)) {
    Install-WingetPackage 'Rustlang.Rustup'
}
Import-MsvcEnvironment
if (-not (Get-Command cl.exe -CommandType Application -ErrorAction SilentlyContinue)) {
    Install-WingetPackage 'Microsoft.VisualStudio.2022.BuildTools' @('--override', '--wait --quiet --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended')
    Import-MsvcEnvironment
}
if (-not $FFmpegDirectory -and -not (Get-Command ffmpeg.exe -CommandType Application -ErrorAction SilentlyContinue)) {
    Install-WingetPackage 'Gyan.FFmpeg'
}
if (-not $QMake -and -not (Get-Command qmake.exe -CommandType Application -ErrorAction SilentlyContinue)) {
    $qtRoot = Join-Path $env:LOCALAPPDATA 'VeloxTranscode\Qt'
    $qtQMake = Join-Path $qtRoot '6.8.3\msvc2022_64\bin\qmake.exe'
    if (-not (Test-Path -LiteralPath $qtQMake)) {
        if (-not (Get-Command python.exe -CommandType Application -ErrorAction SilentlyContinue)) {
            Install-WingetPackage 'Python.Python.3.12'
        }
        $python = (Get-Command python.exe -CommandType Application -ErrorAction Stop | Select-Object -First 1).Source
        $venv = Join-Path $env:LOCALAPPDATA 'VeloxTranscode\aqt-venv'
        if (-not (Test-Path -LiteralPath (Join-Path $venv 'Scripts\python.exe'))) {
            Invoke-Checked $python @('-m', 'venv', $venv)
        }
        $venvPython = Join-Path $venv 'Scripts\python.exe'
        Invoke-Checked $venvPython @('-m', 'pip', 'install', 'aqtinstall==3.3.0')
        Invoke-Checked $venvPython @('-m', 'aqt', 'install-qt', 'windows', 'desktop', '6.8.3', 'win64_msvc2022_64', '-O', $qtRoot)
    }
    $QMake = $qtQMake
}
