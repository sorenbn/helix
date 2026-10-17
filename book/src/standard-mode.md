# Standard mode

Standard mode provides conventional caret movement and selection. Typing inserts
text and replaces any selected text. It stays active when switching files or
splits.

Enable it in `config.toml`:

```toml
[editor]
default-mode = "standard"
```

Press `F6` to switch between Standard and Normal mode. Escape clears the
selection and extra carets while keeping Standard mode active. Popups handle
Escape before the editor does.

| Keys | Action |
| --- | --- |
| Arrows | Move the caret |
| Shift + arrows | Extend the selection |
| Alt + Up / Down | Move the current line or selected lines up / down |
| Ctrl + Left / Right | Move by word |
| Ctrl + Shift + Left / Right | Extend by word |
| Ctrl + Up / Down | Move ten source lines |
| Ctrl + Shift + Up / Down | Extend by ten source lines |
| Ctrl + Alt + Up / Down | Go to previous / next function, class or section boundary |
| Ctrl + Alt + Shift + Up / Down | Extend to previous / next boundary |
| Home / End | Go to line start / end |
| Ctrl + Home / End | Go to document start / end |
| Shift + Home / End | Extend to line start / end |
| Ctrl + Shift + Home / End | Extend to document start / end |
| Page Up / Down | Move by a page |
| Shift + Page Up / Down | Extend by a page |
| Ctrl + A | Select the entire document |
| Backspace / Delete | Delete selected text or an adjacent grapheme |
| Ctrl + Backspace / Delete | Delete selected text or an adjacent word |
| Enter | Replace the selection with an indented newline |
| Tab | Insert indentation, or indent selected lines |
| Shift + Tab | Unindent selected lines |
| Ctrl + C / X / V | Copy / cut / paste using the system clipboard |
| Ctrl + K, Ctrl + C | Comment the current line or selected lines |
| Ctrl + K, Ctrl + U | Uncomment the current line or selected lines |
| Ctrl + Z | Undo |
| Ctrl + Y or Ctrl + Shift + Z | Redo |
| Ctrl + S | Save the current named buffer |
| Ctrl + P | Open the file picker |
| Ctrl + Shift + P | Open the command prompt |
| Ctrl + Space | Request completion |
| F1 | Open the command prompt |
| F2 | Rename the symbol (LSP) |
| F12 | Go to definition (LSP) |
| Shift + F12 | Find references (LSP) |
| Ctrl + F12 | Go to implementation (LSP) |
| Ctrl + Alt + Left / Right | Navigate back / forward |
| Ctrl + . | Show code actions (LSP) |
| Ctrl + Backslash | Split the editor side by side |
| Ctrl + F6 / Ctrl + Shift + F6 | Focus the next / previous editor split |
| Ctrl + Q | Quit, checking for unsaved changes |

With no selection, Ctrl + X cuts the whole current source line, including its line
ending when present. Multiple carets cut each affected line once. Cutting the last
line removes its preceding line break so an empty last line is not left behind.
Ctrl + C leaves the clipboard untouched when no text is selected. Pasting replaces
selected text, including when the clipboard contains complete lines. Consecutive
typing is grouped into one undo step. Movement, selection changes, switching
between insertion and deletion, and standalone edits create undo boundaries.

Comment shortcuts are two-step chords: press Ctrl + K, then Ctrl + C or Ctrl + U.
The second key also works after releasing Ctrl. Comments use the language's line
comment token, such as `//` for C# and Odin or `#` for Python. Languages with only
block comments, such as HTML and CSS, wrap each affected line. Blank lines are
skipped. Commenting leaves existing comments in place; uncommenting leaves plain
code in place. The caret and selection follow the text, and each chord is one
undo step. A selection ending at the start of the next line leaves that next
line unchanged.

Ten-line jumps preserve the preferred column across short lines and count source
lines even with soft wrapping enabled. Contextual jumps stop at function and class
starts and ends using the language's syntax queries. When no syntax boundary is
available in that direction, they stop at the start or end of a block of text
separated by blank lines.

Alt + Up / Down moves whole source lines and keeps the caret and selection with
the text. Selected lines move together, and a selection ending at the start of
the next line leaves that next line in place. Each move is one undo step.

LSP navigation uses the language server for the current project. Place the caret
on a function call and press F12 to open its definition. Multiple definitions or
references open a location picker; Enter opens the selected result. A single
result opens directly. Standard mode places a caret at the destination without
selecting the returned symbol range. Ctrl + Alt + Left returns to the caller, and
Ctrl + Alt + Right moves forward through the jump history.

The LSP client is built in; language servers are separate programs. Odin uses
`ols`. C# supports `roslyn-language-server`, `OmniSharp`, and `csharp-ls`. Check
what is available with `hx --health odin` or `hx --health c-sharp`. Open files
within their project so the server can resolve symbols across files.

For two files inside Helix, press Ctrl + Backslash, then Ctrl + P to open another
file in the focused split. Click a split or use Ctrl + F6 to focus it. F1 also
accepts `vsplit <path>` for side-by-side files and `hsplit <path>` for files above
and below each other. These are editor views; a shell alongside the editor uses
a terminal pane.

In Windows Terminal, Alt + Shift + Plus creates a pane to the right, and Alt +
Shift + Minus creates one below. Keep Helix in one pane and use PowerShell in the
other. Click a pane to focus it. Windows Terminal's default Alt + arrow shortcuts
switch terminal panes and can intercept the editor's line-moving shortcuts when
multiple panes are open. Remap terminal pane navigation or use editor splits for
two files. See [Windows Terminal panes](https://learn.microsoft.com/en-us/windows/terminal/panes).

For an unnamed buffer, open the command prompt and run `write <path>`. The
command prompt also provides replacement and buffer commands. Keybindings
can be customized in `[keys.standard]`:

```toml
[keys.standard]
"C-S-s" = ":write-all"
```

To use contextual jumps on Ctrl + Up / Down instead:

```toml
[keys.standard]
"C-up" = "standard_section_up"
"C-down" = "standard_section_down"
"C-S-up" = "standard_select_section_up"
"C-S-down" = "standard_select_section_down"
```

Standard mode uses a bar caret by default. Customize it with
`[editor.cursor-shape] standard = "bar"`. Themes can define `ui.cursor.standard`,
`ui.cursor.primary.standard`, and `ui.statusline.standard`; insert-mode styles
provide fallbacks.

Some terminals reserve shortcuts or do not distinguish all Ctrl/Shift
combinations. Adjust terminal shortcuts or use a different binding when a key
does not reach the editor.

Windows Terminal reserves Ctrl + Shift + Up / Down for scrolling and Ctrl + V for
terminal paste. The Windows input backend receives terminal paste as individual
keystrokes, which can be slow and trigger indentation or auto-pairing. Let Ctrl + V
reach Standard mode to read the clipboard and insert the text in one edit.

In Windows Terminal's `settings.json`, set these entries in `keybindings`, replacing
any existing entries for the same keys:

```json
{ "id": "unbound", "keys": "ctrl+shift+up" },
{ "id": "unbound", "keys": "ctrl+shift+down" },
{ "id": "unbound", "keys": "ctrl+v" }
```

See [Windows Terminal keybindings](https://learn.microsoft.com/en-us/windows/terminal/customize-settings/actions#unbind-keys-disable-keybindings).
