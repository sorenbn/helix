# Dot-source this script before running Cargo in a development shell.
$helixRepoRoot = Split-Path -Parent $PSScriptRoot
$helixCargoBin = Join-Path $helixRepoRoot '.tools\cargo\bin'
if (-not (Test-Path -LiteralPath (Join-Path $helixCargoBin 'rustup.exe'))) {
    throw 'Repository-local Rust is missing. See docs/windows-development.md.'
}

$env:CARGO_HOME = Join-Path $helixRepoRoot '.tools\cargo'
$env:RUSTUP_HOME = Join-Path $helixRepoRoot '.tools\rustup'
if (($env:PATH -split ';') -notcontains $helixCargoBin) {
    $env:PATH = "$helixCargoBin;$env:PATH"
}
$env:HELIX_RUNTIME = Join-Path $helixRepoRoot 'runtime'
$env:HELIX_DISABLE_AUTO_GRAMMAR_BUILD = '1'

$helixGitBin = Join-Path $env:ProgramFiles 'Git\bin'
if ((Test-Path -LiteralPath (Join-Path $helixGitBin 'bash.exe')) -and (($env:PATH -split ';') -notcontains $helixGitBin)) {
    $env:PATH = "$helixGitBin;$env:PATH"
}

if (-not (Get-Command cl.exe -ErrorAction SilentlyContinue)) {
    $helixVswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
    if (-not (Test-Path -LiteralPath $helixVswhere)) {
        throw 'Install Visual Studio C++ build tools and a Windows SDK. See docs/windows-development.md.'
    }
    $helixVsPath = & $helixVswhere -latest -products '*' -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath
    if (-not $helixVsPath) {
        throw 'Visual Studio C++ build tools were not found. See docs/windows-development.md.'
    }
    Import-Module (Join-Path $helixVsPath 'Common7\Tools\Microsoft.VisualStudio.DevShell.dll') -ErrorAction Stop
    Enter-VsDevShell -VsInstallPath $helixVsPath -SkipAutomaticLocation -DevCmdArguments '-arch=x64 -host_arch=x64' -ErrorAction Stop | Out-Null
}
