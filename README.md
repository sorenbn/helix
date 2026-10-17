# Helix But Better

An independent fork of [Helix](https://github.com/helix-editor/helix) with
conventional text editing in the terminal. **Standard mode** lets you type,
select with Shift, and use familiar editor shortcuts without learning modal
motions. It keeps Helix's syntax highlighting, multiple selections, language
server support, and split editor views.

## Fork features

- Conventional caret movement and selection; typing replaces selected text.
  Standard mode stays active when switching files or editor splits.
- Ten-line jumps, syntax-aware jumps to function and class boundaries, and
  selection while jumping. Contextual jumps fall back to sections separated by
  blank lines when a syntax boundary is unavailable.
- Move the current line or selected lines up and down with the caret attached.
- Comment and uncomment the current line or selected lines using the language's
  comment syntax.
- System clipboard copy, cut, and bulk paste. Cut removes the whole current line
  when nothing is selected; paste inserts clipboard text in one edit.
- Familiar shortcuts for definitions, references, renaming, completion, and code
  actions through Helix's built-in LSP client.
- Windows x64 build, packaging, and installation scripts with bundled runtime
  files and compiled syntax grammars.

### Standard mode shortcuts

| Shortcut | Action |
| --- | --- |
| Arrows / Shift + arrows | Move / select text |
| Ctrl + Left / Right | Move by word; add Shift to select |
| Ctrl + Up / Down | Jump ten source lines; add Shift to select |
| Ctrl + Alt + Up / Down | Jump to a function, class, or section boundary; add Shift to select |
| Alt + Up / Down | Move the current line or selected lines |
| Home / End | Go to line start / end; add Shift to select |
| Ctrl + Home / End | Go to document start / end; add Shift to select |
| Page Up / Down | Move by a page; add Shift to select |
| Ctrl + A | Select all |
| Ctrl + C / X / V | Copy / cut / paste; cut the current line when nothing is selected |
| Ctrl + K, Ctrl + C / Ctrl + K, Ctrl + U | Comment / uncomment the current line or selected lines |
| Backspace / Delete | Delete text; add Ctrl to delete by word |
| Enter | Insert an indented newline |
| Tab / Shift + Tab | Indent / unindent |
| Ctrl + Z | Undo |
| Ctrl + Y or Ctrl + Shift + Z | Redo |
| Ctrl + S | Save the current named buffer |
| Ctrl + P | Open the file picker |
| F1 or Ctrl + Shift + P | Open the command prompt |
| Ctrl + Space | Request completion |
| F2 | Rename the symbol |
| F12 / Shift + F12 | Go to definition / find references |
| Ctrl + F12 | Go to implementation |
| Ctrl + . | Show code actions |
| Ctrl + Alt + Left / Right | Navigate back / forward through jump history |
| Ctrl + Backslash | Split the editor side by side |
| Ctrl + F6 / Ctrl + Shift + F6 | Focus the next / previous editor split |
| Escape | Clear the selection and extra carets |
| F6 | Switch between Standard and Normal mode |
| Ctrl + Q | Quit, checking for unsaved changes |

Language features require a language server installed for your language. Check
availability with `hx --health odin`, `hx --health c-sharp`, or `hx --health`.
For an unnamed buffer, use F1 and enter `write <path>` to save it.

Enable Standard mode in your Helix `config.toml`:

```toml
[editor]
default-mode = "standard"
```

The Windows package installer enables it automatically. See the
[Standard mode guide](book/src/standard-mode.md) for customization, split views,
and Windows Terminal settings needed to pass Ctrl + V and Ctrl + Shift + Up / Down
through to the editor. A shell beside Helix uses a terminal pane; Helix splits
display files.

## Build and package on Windows x64

Prerequisites: Git, Rust 1.90.0, and Visual Studio C++ x64 build tools with a
Windows SDK. The scripts use a repository-local Rust toolchain under `.tools/`;
follow the [toolchain setup instructions](docs/windows-development.md#install-the-local-toolchain-on-another-checkout)
once on a fresh checkout.

From the repository root in PowerShell, build an optimized release and package
the runtime, syntax grammars, installer, and source:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\package-windows.ps1 -BuildGrammars
```

Omit `-BuildGrammars` on subsequent builds when the compiled grammars are already
available. The script loads the development environment and builds with the
`opt` profile (full link-time optimization and stripped symbols). Allow several
minutes for compilation. Output:

```text
dist\helix-but-better-windows-x64.zip
dist\helix-but-better-windows-x64.zip.sha256
```

Extract the ZIP, open PowerShell in the extracted folder, and install:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\install.ps1
```

Open a new terminal and run `hx`. To update an existing installation, close its
running editors and pass `-InstallDirectory C:\path\to\helix`. The installer
backs up existing program files and changed configuration files. Keep the
bundled `runtime` directory beside `hx.exe`; Rust is only needed to build.
See [Windows release instructions](docs/windows-release.md) for installation
options and runtime prerequisites.

For development, load the environment and run a debug build with Standard mode:

```powershell
. .\scripts\dev-env.ps1
cargo run --locked -p helix-term -- --config contrib/standard.toml README.md
```

See [Windows development](docs/windows-development.md) for checks and grammar
build commands. For other platforms, see the
[Helix source build guide](book/src/building-from-source.md).

---

<div align="center">


[![Build status](https://github.com/helix-editor/helix/actions/workflows/build.yml/badge.svg)](https://github.com/helix-editor/helix/actions)
[![GitHub Release](https://img.shields.io/github/v/release/helix-editor/helix)](https://github.com/helix-editor/helix/releases/latest)
[![Documentation](https://shields.io/badge/-documentation-452859)](https://docs.helix-editor.com/)
[![GitHub contributors](https://img.shields.io/github/contributors/helix-editor/helix)](https://github.com/helix-editor/helix/graphs/contributors)
[![Matrix Space](https://img.shields.io/matrix/helix-community:matrix.org)](https://matrix.to/#/#helix-community:matrix.org)

!


A [Kakoune](https://github.com/mawww/kakoune) / [Neovim](https://github.com/neovim/neovim) inspired editor, written in Rust.

The editing model is very heavily based on Kakoune; during development I found
myself agreeing with most of Kakoune's design decisions.

For more information, see the [website](https://helix-editor.com) or
[documentation](https://docs.helix-editor.com/).

All shortcuts/keymaps can be found [in the documentation on the website](https://docs.helix-editor.com/keymap.html).

[Troubleshooting](https://github.com/helix-editor/helix/wiki/Troubleshooting)

# Features

- Vim-like modal editing
- Multiple selections
- Built-in language server support
- Smart, incremental syntax highlighting and code editing via tree-sitter

Although it's primarily a terminal-based editor, I am interested in exploring
a custom renderer (similar to Emacs) using wgpu.

Note: Only certain languages have indentation definitions at the moment. Check
`runtime/queries/<lang>/` for `indents.scm`.

# Installation

[Installation documentation](https://docs.helix-editor.com/install.html).

[![Packaging status](https://repology.org/badge/vertical-allrepos/helix-editor.svg?exclude_unsupported=1)](https://repology.org/project/helix-editor/versions)

# Contributing

Contributing guidelines can be found [here](./docs/CONTRIBUTING.md).

# Getting help

Your question might already be answered on the [FAQ](https://github.com/helix-editor/helix/wiki/FAQ).

Discuss the project on the community [Matrix Space](https://matrix.to/#/#helix-community:matrix.org) (make sure to join `#helix-editor:matrix.org` if you're on a client that doesn't support Matrix Spaces yet).

# Credits

Thanks to [@jakenvac](https://github.com/jakenvac) for designing the logo!


</div>
  <h1>
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="logo_dark.svg">
      <source media="(prefers-color-scheme: light)" srcset="logo_light.svg">
        <img alt="Helix" height="128" src="logo_light.svg">
    </picture>
  </h1>
![Screenshot](./screenshot.png)
