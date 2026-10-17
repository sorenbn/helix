# Windows development

The repository uses Rust 1.90.0 from `rust-toolchain.toml`. Its local toolchain is
stored under `.tools/`, which is ignored by Git. Cargo dependencies also stay
under `.tools/`. Rustfmt, Clippy, Rust sources, and rust-analyzer are installed.
No global PATH change is required.

Visual Studio must provide the C++ x64/x86 build tools and a Windows SDK. The
development script discovers an installation through `vswhere` and loads its
x64 developer environment into the current PowerShell process.
It also makes Git Bash available when installed, for tests that use shell
commands.

## Open a development shell

Run from the repository root in PowerShell:

```powershell
. .\scripts\dev-env.ps1
rustc --version
cargo --version
```

The leading dot and space load the environment into the current shell. Repeat
this in each new shell. The script sets `CARGO_HOME`, `RUSTUP_HOME`,
`HELIX_RUNTIME`, and `HELIX_DISABLE_AUTO_GRAMMAR_BUILD` for that process.
Builds skip automatic grammar downloads; build grammars separately below.

## Build and run

```powershell
cargo build --locked -p helix-term
cargo run --locked -p helix-term -- --version
cargo run --locked -p helix-term -- --health
cargo run --locked -p helix-term -- README.md
```

To launch with conventional editing:

```powershell
cargo run --locked -p helix-term -- --config contrib/standard.toml README.md
```

Debug builds support the edit/build/run cycle. An optimized executable can be
built with `cargo build --locked --release -p helix-term`.
Launching through Cargo gives this checkout's runtime priority over any
previously installed Helix runtime.

For an optimized, distributable Windows ZIP with an installer, run:

```powershell
.\scripts\package-windows.ps1
```

See [Windows release installation](windows-release.md) for installation and PATH
setup. The package includes the runtime and compiled grammars; copying only
`hx.exe` leaves syntax features without their supporting files.

## Checks

```powershell
cargo check --locked --workspace
cargo fmt --all --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
cargo integration-test --locked
```

Some tests exercise syntax grammars. Prepare those before running the complete
test suite.

## Syntax grammars

Grammars provide tree-sitter highlighting and syntax-aware features. Their
sources and compiled libraries live in ignored `runtime/grammars/`.

```powershell
cargo run --locked -p helix-term -- --grammar fetch --strict
cargo run --locked -p helix-term -- --grammar build --strict
cargo run --locked -p helix-term -- --health rust
```

Fetching all grammars requires network access and can take a while. To restrict
them, use Helix's `use-grammars` setting in a `languages.toml` loaded by the
editor, as described in [the language configuration guide](../book/src/languages.md#choosing-grammars).
That file must be allowed by workspace trust when it is project-local.

Rust language-server support uses the installed rust-analyzer when the editor
is launched from the development shell. Other languages require their own
language servers.

## Install the local toolchain on another checkout

From the repository root, download the official x64 rustup installer and run:

```powershell
New-Item -ItemType Directory -Force work | Out-Null
Invoke-WebRequest https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe -OutFile work\rustup-init.exe
$env:CARGO_HOME = Join-Path $PWD '.tools\cargo'
$env:RUSTUP_HOME = Join-Path $PWD '.tools\rustup'
.\work\rustup-init.exe -y --no-modify-path --profile minimal --default-host x86_64-pc-windows-msvc --default-toolchain 1.90.0 --component rustfmt,clippy,rust-src,rust-analyzer
. .\scripts\dev-env.ps1
```

Install the Visual Studio C++ tools and Windows SDK if the development script
reports that they are missing. Rust's [MSVC prerequisites](https://rust-lang.github.io/rustup/installation/windows-msvc.html)
describe the required components.
