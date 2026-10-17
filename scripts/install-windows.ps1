[CmdletBinding()]
param(
    [string]$PackageDirectory,
    [string]$InstallDirectory = (Join-Path ([Environment]::GetFolderPath('LocalApplicationData')) 'Programs\HelixButBetter'),
    [switch]$KeepConfig
)

$ErrorActionPreference = 'Stop'
if ([string]::IsNullOrWhiteSpace($PackageDirectory)) { $PackageDirectory = $PSScriptRoot }
$PackageDirectory = [IO.Path]::GetFullPath($PackageDirectory)
$InstallDirectory = [IO.Path]::GetFullPath($InstallDirectory)
if (-not (Test-Path -LiteralPath (Join-Path $PackageDirectory 'hx.exe'))) {
    throw 'Point -PackageDirectory at the extracted release folder containing hx.exe.'
}
if (-not (Test-Path -LiteralPath (Join-Path $PackageDirectory 'runtime\queries'))) {
    throw 'The release runtime folder is missing.'
}
if ($PackageDirectory.TrimEnd('\') -eq $InstallDirectory.TrimEnd('\') -or
    $PackageDirectory.StartsWith($InstallDirectory.TrimEnd('\') + '\', [StringComparison]::OrdinalIgnoreCase) -or
    $InstallDirectory.StartsWith($PackageDirectory.TrimEnd('\') + '\', [StringComparison]::OrdinalIgnoreCase)) {
    throw 'Choose an install directory outside the extracted release folder.'
}
if ($InstallDirectory.TrimEnd('\') -eq [IO.Path]::GetPathRoot($InstallDirectory).TrimEnd('\')) {
    throw 'Choose a program folder rather than a drive root.'
}
& (Join-Path $PackageDirectory 'hx.exe') --version
if ($LASTEXITCODE -ne 0) {
    throw 'The executable cannot run. Install the Microsoft Visual C++ x64 runtime and try again.'
}

$configPath = Join-Path ([Environment]::GetFolderPath('ApplicationData')) 'helix\config.toml'
$configText = if (Test-Path -LiteralPath $configPath) { [IO.File]::ReadAllText($configPath) } else { '' }
$updatedConfig = $configText
if (-not $KeepConfig) {
    $newline = if ($configText.Contains("`r`n")) { "`r`n" } else { "`n" }
    $editorTable = [regex]'(?ms)^\[editor\][^\r\n]*(?:\r?\n|$)(?<body>.*?)(?=^\s*\[|\z)'
    $modeSetting = [regex]'(?m)^([ \t]*default-mode[ \t]*=[ \t]*)(?:"[^"]*"|''[^'']*'')'
    if ($editorTable.IsMatch($configText)) {
        $updatedConfig = $editorTable.Replace($configText, [Text.RegularExpressions.MatchEvaluator]{
            param($match)
            $body = $match.Groups['body'].Value
            $header = $match.Value.Substring(0, $match.Value.Length - $body.Length)
            if ($modeSetting.IsMatch($body)) {
                $header + $modeSetting.Replace($body, '${1}"standard"', 1)
            } else {
                $header.TrimEnd("`r", "`n") + $newline + 'default-mode = "standard"' + $newline + $body
            }
        }, 1)
    } elseif ($configText -match '(?m)^\s*editor\s*(\.|=)') {
        throw 'This config uses root-level editor assignments. Set editor.default-mode to standard manually and rerun with -KeepConfig.'
    } else {
        $editorSettings = '[editor]' + $newline + 'default-mode = "standard"' + $newline + $newline
        $firstTable = [regex]::Match($configText, '(?m)^\s*\[')
        $index = if ($firstTable.Success) { $firstTable.Index } else { $configText.Length }
        $prefix = $configText.Substring(0, $index)
        if ($prefix.Length -gt 0 -and -not $prefix.EndsWith("`n")) { $prefix += $newline }
        $updatedConfig = $prefix + $editorSettings + $configText.Substring($index)
    }
}

$stamp = Get-Date -Format 'yyyyMMdd-HHmmss-ffff'
if (Test-Path -LiteralPath $InstallDirectory) {
    $installRoot = $InstallDirectory.TrimEnd('\') + '\'
    $backup = [IO.Path]::GetFullPath((Join-Path $InstallDirectory ".backups\$stamp"))
    if (-not $backup.StartsWith($installRoot, [StringComparison]::OrdinalIgnoreCase)) {
        throw 'Backup directory must stay inside the install directory.'
    }
    $previousFiles = @(Get-ChildItem -LiteralPath $InstallDirectory -Force | Where-Object Name -ne '.backups')
    New-Item -ItemType Directory -Path $backup | Out-Null
    foreach ($entry in $previousFiles) {
        $sourcePath = [IO.Path]::GetFullPath($entry.FullName)
        $backupPath = [IO.Path]::GetFullPath((Join-Path $backup $entry.Name))
        if (-not $sourcePath.StartsWith($installRoot, [StringComparison]::OrdinalIgnoreCase) -or
            -not $backupPath.StartsWith($backup + '\', [StringComparison]::OrdinalIgnoreCase)) {
            throw 'Backup paths must stay inside the install directory.'
        }
        Move-Item -LiteralPath $sourcePath -Destination $backupPath
    }
    Write-Host "Previous installation backed up to $backup"
}
New-Item -ItemType Directory -Force -Path $InstallDirectory | Out-Null
foreach ($name in @('hx.exe', 'runtime', 'LICENSE', 'README.md', 'BUILD-INFO.txt', 'source.zip')) {
    $source = Join-Path $PackageDirectory $name
    if (Test-Path -LiteralPath $source) {
        Copy-Item -LiteralPath $source -Destination $InstallDirectory -Recurse -Force
    }
}
if (-not $KeepConfig -and $updatedConfig -cne $configText) {
    New-Item -ItemType Directory -Force -Path (Split-Path -Parent $configPath) | Out-Null
    if (Test-Path -LiteralPath $configPath) {
        Copy-Item -LiteralPath $configPath -Destination "$configPath.backup-$stamp"
    }
    [IO.File]::WriteAllText($configPath, $updatedConfig, [Text.UTF8Encoding]::new($false))
}

$userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
if (($env:Path -split ';').TrimEnd('\') -notcontains $InstallDirectory.TrimEnd('\')) {
    if (($userPath -split ';').TrimEnd('\') -notcontains $InstallDirectory.TrimEnd('\')) {
        [Environment]::SetEnvironmentVariable('Path', "$InstallDirectory;$userPath", 'User')
    }
}
$env:Path = "$InstallDirectory;$env:Path"
Write-Host "Installed to $InstallDirectory"
Write-Host 'Open a new terminal and run: hx'
& (Join-Path $InstallDirectory 'hx.exe') --version
if ($LASTEXITCODE -ne 0) { throw 'The installed executable failed its version check.' }
