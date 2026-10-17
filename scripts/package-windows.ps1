[CmdletBinding()]
param(
    [switch]$SkipBuild,
    [switch]$BuildGrammars,
    [string]$OutputDirectory
)

$ErrorActionPreference = 'Stop'
$repo = Split-Path -Parent $PSScriptRoot
if ([string]::IsNullOrWhiteSpace($OutputDirectory)) { $OutputDirectory = Join-Path $repo 'dist' }
Push-Location $repo
try {
    . (Join-Path $PSScriptRoot 'dev-env.ps1')
    if (-not $SkipBuild) {
        cargo build --locked --profile opt -p helix-term
        if ($LASTEXITCODE -ne 0) { throw 'Release build failed.' }
    }
    $executable = Join-Path $repo 'target\opt\hx.exe'
    if (-not (Test-Path -LiteralPath $executable)) { throw 'Build target/opt/hx.exe first.' }
    if ($BuildGrammars) {
        & $executable --grammar fetch --strict
        if ($LASTEXITCODE -ne 0) { throw 'Grammar fetch failed.' }
        & $executable --grammar build --strict
        if ($LASTEXITCODE -ne 0) { throw 'Grammar build failed.' }
    }
    $grammars = @(Get-ChildItem -LiteralPath (Join-Path $repo 'runtime\grammars') -File | Where-Object Extension -eq '.dll')
    if ($grammars.Count -eq 0) { throw 'No compiled grammars found. Rerun with -BuildGrammars.' }
    $stamp = Get-Date -Format 'yyyyMMdd-HHmmss-ffff'
    $stage = Join-Path $repo "work\package-windows\$stamp"
    $package = Join-Path $stage 'helix-but-better-windows-x64'
    New-Item -ItemType Directory -Force -Path (Join-Path $package 'runtime\grammars') | Out-Null
    Copy-Item -LiteralPath $executable -Destination (Join-Path $package 'hx.exe')
    foreach ($name in @('queries', 'themes', 'tutor')) {
        Copy-Item -LiteralPath (Join-Path $repo "runtime\$name") -Destination (Join-Path $package 'runtime') -Recurse
    }
    foreach ($grammar in $grammars) {
        Copy-Item -LiteralPath $grammar.FullName -Destination (Join-Path $package 'runtime\grammars')
    }
    Copy-Item -LiteralPath (Join-Path $repo 'LICENSE') -Destination $package
    Copy-Item -LiteralPath (Join-Path $repo 'contrib\standard.toml') -Destination (Join-Path $package 'standard.toml')
    Copy-Item -LiteralPath (Join-Path $PSScriptRoot 'install-windows.ps1') -Destination (Join-Path $package 'install.ps1')
    Copy-Item -LiteralPath (Join-Path $repo 'docs\windows-release.md') -Destination (Join-Path $package 'README.md')
    $version = & $executable --version
    $commit = git rev-parse HEAD
    @(
        "Version: $version"
        'Target: x86_64-pc-windows-msvc'
        'Profile: opt (fat LTO, one codegen unit, stripped)'
        "Built: $([DateTime]::UtcNow.ToString('o'))"
        "Upstream commit: $commit"
        'Source: source.zip (includes local modifications)'
        "Compiled grammars: $($grammars.Count)"
    ) | Set-Content -LiteralPath (Join-Path $package 'BUILD-INFO.txt') -Encoding UTF8

    $sourceStage = Join-Path $stage 'source'
    New-Item -ItemType Directory -Path $sourceStage | Out-Null
    $sourceFiles = git -c core.quotepath=false ls-files --cached --others --exclude-standard
    if ($LASTEXITCODE -ne 0) { throw 'Cannot enumerate source files.' }
    foreach ($file in $sourceFiles) {
        $source = Join-Path $repo $file
        if (-not (Test-Path -LiteralPath $source -PathType Leaf)) { continue }
        $destination = Join-Path $sourceStage $file
        New-Item -ItemType Directory -Force -Path (Split-Path -Parent $destination) | Out-Null
        Copy-Item -LiteralPath $source -Destination $destination
    }
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    [IO.Compression.ZipFile]::CreateFromDirectory($sourceStage, (Join-Path $package 'source.zip'))
    New-Item -ItemType Directory -Force -Path $OutputDirectory | Out-Null
    $archive = Join-Path ([IO.Path]::GetFullPath($OutputDirectory)) 'helix-but-better-windows-x64.zip'
    $stagedArchive = Join-Path $stage 'helix-but-better-windows-x64.zip'
    [IO.Compression.ZipFile]::CreateFromDirectory($package, $stagedArchive)
    Move-Item -LiteralPath $stagedArchive -Destination $archive -Force
    $hash = (Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLowerInvariant()
    "$hash  $([IO.Path]::GetFileName($archive))" | Set-Content -LiteralPath "$archive.sha256" -Encoding ASCII
    Write-Host "Archive: $archive"
    Write-Host "Size: $((Get-Item -LiteralPath $archive).Length) bytes"
    Write-Host "SHA256: $hash"
} finally {
    Pop-Location
}
