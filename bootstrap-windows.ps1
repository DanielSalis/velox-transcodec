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
    $environment = & $env:ComSpec /d /c "call `"$devcmd`" -no_logo -arch=x64 -host_arch=x64 >nul && set"
    if ($LASTEXITCODE -ne 0) { throw 'Não foi possível configurar o ambiente de compilação MSVC.' }
    foreach ($line in $environment) {
        if ($line -match '^([^=]+)=(.*)$') { [Environment]::SetEnvironmentVariable($Matches[1], $Matches[2], 'Process') }
    }
}
function Find-UsablePython {
    $candidates = @()
    $candidates += @(Get-Command python.exe -All -CommandType Application -ErrorAction SilentlyContinue | ForEach-Object { $_.Source })
    $candidates += (Join-Path $env:LOCALAPPDATA 'Programs\Python\Python312\python.exe')
    $candidates += (Join-Path $env:ProgramFiles 'Python312\python.exe')
    foreach ($root in @('HKCU:\Software\Python\PythonCore', 'HKLM:\Software\Python\PythonCore', 'HKLM:\Software\WOW6432Node\Python\PythonCore')) {
        if (Test-Path -LiteralPath $root) {
            foreach ($version in @(Get-ChildItem -LiteralPath $root -ErrorAction SilentlyContinue)) {
                $key = Join-Path $version.PSPath 'InstallPath'
                if (Test-Path -LiteralPath $key) {
                    $directory = (Get-Item -LiteralPath $key).GetValue('')
                    if ($directory) { $candidates += (Join-Path $directory 'python.exe') }
                }
            }
        }
    }
    foreach ($candidate in ($candidates | Select-Object -Unique)) {
        # WindowsApps pode conter apenas o alias da Microsoft Store, que não é Python.
        if ($candidate -like (Join-Path $env:LOCALAPPDATA 'Microsoft\WindowsApps\*')) { continue }
        if (-not (Test-Path -LiteralPath $candidate -PathType Leaf)) { continue }
        try {
            & $candidate -c 'import sys; sys.exit(0 if sys.version_info >= (3, 9) else 1)' 2>$null
            if ($LASTEXITCODE -eq 0) { return $candidate }
        } catch { continue }
    }
    return $null
}
function Test-QtQMake([string]$Path) {
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        return @{ Valid = $false; Detail = "qmake não encontrado: $Path" }
    }
    $bin = Split-Path -Parent $Path
    if (-not (Test-Path -LiteralPath (Join-Path $bin 'windeployqt.exe') -PathType Leaf)) {
        return @{ Valid = $false; Detail = "windeployqt.exe não encontrado em $bin" }
    }
    $savedPath = $env:PATH
    $savedPreference = $ErrorActionPreference
    $output = ''
    $exitCode = -1
    try {
        $env:PATH = "$bin;$savedPath"
        $ErrorActionPreference = 'Continue'
        $output = (& $Path -query QT_VERSION 2>&1 | Out-String).Trim()
        $exitCode = $LASTEXITCODE
    } catch {
        $output = $_.Exception.Message
    } finally {
        $env:PATH = $savedPath
        $ErrorActionPreference = $savedPreference
    }
    if ($exitCode -eq 0 -and $output -match '^6\.\d+(\.\d+)?$') {
        return @{ Valid = $true; Detail = $output }
    }
    return @{ Valid = $false; Detail = "qmake: $Path; código: $exitCode; saída: $output" }
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
if (-not $QMake) {
    $qtRoot = Join-Path $env:LOCALAPPDATA 'VeloxTranscode\Qt'
    $qtQMake = Join-Path $qtRoot '6.8.3\msvc2022_64\bin\qmake.exe'
    $candidates = @($qtQMake) + @(Get-Command qmake.exe -All -CommandType Application -ErrorAction SilentlyContinue | ForEach-Object { $_.Source })
    foreach ($candidate in ($candidates | Select-Object -Unique)) {
        if ((Test-QtQMake $candidate).Valid) { $QMake = $candidate; break }
    }
    if (-not $QMake) {
        $python = Find-UsablePython
        if (-not $python) {
            Install-WingetPackage 'Python.Python.3.12'
            $python = Find-UsablePython
        }
        if (-not $python) { throw 'Python 3.9 ou posterior não foi encontrado após a instalação. Verifique a instalação do Python e execute novamente.' }
        $venv = Join-Path $env:LOCALAPPDATA 'VeloxTranscode\aqt-venv'
        if (-not (Test-Path -LiteralPath (Join-Path $venv 'Scripts\python.exe'))) {
            Invoke-Checked $python @('-m', 'venv', $venv)
        }
        $venvPython = Join-Path $venv 'Scripts\python.exe'
        Invoke-Checked $venvPython @('-m', 'pip', 'install', 'aqtinstall==3.3.0')
        Invoke-Checked $venvPython @('-m', 'aqt', 'install-qt', 'windows', 'desktop', '6.8.3', 'win64_msvc2022_64', '-O', $qtRoot)
        $QMake = $qtQMake
    }
}
