# Helix But Better for Windows x64

This package includes the optimized editor, syntax grammars, queries, themes,
and source code. Rust and Visual Studio build tools are not required to run it.
If the machine does not already have the Microsoft Visual C++ x64 runtime,
install it from [Microsoft's redistributable downloads](https://learn.microsoft.com/en-us/cpp/windows/latest-supported-vc-redist).

## Install

Extract the ZIP, open PowerShell in the extracted folder, and run:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\install.ps1
```

The installer copies the editor to your local Programs folder, adds it to your
user PATH, and enables Standard mode in your Helix configuration. Existing
installations and changed configuration files are backed up. Open a new terminal:

```powershell
hx
hx C:\path\to\file.rs
```

Previous program files are retained under `.backups` in the install directory.
Configuration backups are saved beside `config.toml` with a timestamp suffix.

Close editors running from the destination directory before updating. To update
an installation already on PATH, pass its directory:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\install.ps1 -InstallDirectory C:\SDKs\helix
```

Use `-KeepConfig` to keep your current editor mode. If another `hx` takes
precedence, check `Get-Command hx -All` and update that installation or adjust
PATH. Keep the `runtime` directory beside `hx.exe`.

## Run without installing

```powershell
.\hx.exe --config .\standard.toml
```

## Windows Terminal shortcuts

In Windows Terminal's `settings.json`, set these entries in `keybindings`,
replacing existing entries for the same keys:

```json
{ "id": "unbound", "keys": "ctrl+v" },
{ "id": "unbound", "keys": "ctrl+shift+up" },
{ "id": "unbound", "keys": "ctrl+shift+down" }
```

This lets the editor handle clipboard paste and selection during ten-line jumps.
See [Windows Terminal keybindings](https://learn.microsoft.com/en-us/windows/terminal/customize-settings/actions#unbind-keys-disable-keybindings).

## Build from source

Extract `source.zip`, install Rust 1.90.0 and the Visual Studio C++ tools, and run
from the source root:

```powershell
cargo build --locked --profile opt -p helix-term
.\target\opt\hx.exe --grammar fetch --strict
.\target\opt\hx.exe --grammar build --strict
```

In a checkout with the repository-local toolchain, create the ZIP with:

```powershell
.\scripts\package-windows.ps1
```

Pass `-BuildGrammars` when compiled grammars are not already available. The ZIP
is written to `dist\helix-but-better-windows-x64.zip`. The `opt` profile uses
full link-time optimization and strips symbols. The ordinary `--release` profile
uses thin link-time optimization and builds faster.

Language servers and formatters are separate programs. Install the ones needed
for your languages and check them with `hx --health`. This ZIP targets Windows
x64; Linux, macOS, and Windows ARM64 require builds for their own targets.

The editor is based on [Helix](https://github.com/helix-editor/helix). See LICENSE
for its Mozilla Public License 2.0 and BUILD-INFO.txt for the build details.
