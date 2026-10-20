# Standard editing mode

Add `Mode::Standard` alongside Normal, Select, and Insert. In Standard mode,
characters insert text, modified keys run commands, and selections behave like
those in conventional desktop editors. The mode remains active while opening
files, switching buffers, changing splits, and dismissing prompts.

## Editing contract

| Input | Behavior |
| --- | --- |
| Printable text | Insert at the caret, replacing selected text |
| Left / Right | Move by grapheme; collapse an existing selection to its corresponding edge |
| Up / Down | Move by visual line, preserving the desired visual column |
| Shift + movement | Extend from a fixed anchor, including across the anchor |
| Ctrl + Left / Right | Move by word |
| Ctrl + Shift + Left / Right | Extend by word |
| Home / End | Move to line start / end |
| Ctrl + Home / End | Move to document start / end |
| Shift + Home / End | Extend to line start / end |
| Ctrl + Shift + Home / End | Extend to document start / end |
| Page Up / Down | Move by page |
| Backspace / Delete | Delete the selection, or the adjacent grapheme |
| Ctrl + Backspace / Delete | Delete the selection, or the adjacent word |
| Enter / Tab / Shift + Tab | Insert an indented newline / indent / unindent |
| Ctrl + A | Select the document |
| Ctrl + C / X / V | Copy / cut / paste using the system clipboard |
| Ctrl + Z / Y, Ctrl + Shift + Z | Undo / redo |
| Ctrl + S | Save; request a path for an unnamed buffer |
| Ctrl + N / O | Create an unnamed buffer / open a file by path |
| Ctrl + W | Close the buffer, offering save/discard/cancel for unsaved changes |
| Ctrl + Q | Exit, offering save/discard/cancel for unsaved changes |
| Ctrl + F | Search |
| Ctrl + P | Open the file picker |
| Ctrl + Shift + P | Open a command entry point |
| Escape | Dismiss the active popup first; otherwise clear the selection |

With no selection, Ctrl + C copies the current whole line and Ctrl + X removes it.
Whole-line copies paste below the current line when nothing is selected;
ordinary clipboard text and terminal paste insert at the caret. Clipboard paste and
terminal bracketed paste both replace the selection as one undoable edit.
Mouse clicks place a caret; dragging selects the exact intervening text.
Typing `hjkl`, digits, spaces, and punctuation always inserts those characters.

## 1. Register the mode and its lifecycle

Primary files:

- `helix-view/src/document.rs`: extend `Mode`, parsing, and serialization.
- `helix-view/src/editor.rs`: add a `default-mode` setting accepting `normal`
  or `standard`, initialize the editor, extend cursor-shape configuration and
  status-line mode labels, and review `switch`, `focus`, and `enter_normal_mode`.
- `helix-term/src/keymap/default.rs`: add the Standard keymap.
- `helix-term/src/keymap.rs`: verify map merging and configuration round trips.
- `helix-term/src/ui/statusline.rs`: render the Standard label and theme fallback.

Keep Normal as the configuration default and enable Standard explicitly:

```toml
[editor]
default-mode = "standard"

[keys.standard]
"C-s" = ":write"
```

The default save binding should use the unnamed-buffer behavior in the editing
contract rather than only invoking `:write`.

Separate an explicit request to enter Normal mode from internal resets during
file and view changes. Internal resets should return to the configured editing
mode and commit pending edits in the outgoing document. Centralize transition
notifications so hooks also run when a transition originates outside a keymap.

Completion state, pending keys, and selection conversion need explicit handling
when switching between Standard and modal editing.

Completion criterion: the editor starts in Standard, inserts literal characters,
and remains ready to type after opening another file or switching a split.

## 2. Implement caret movement and selections

Primary files: `helix-core/src/selection.rs`, `helix-core/src/movement.rs`,
`helix-term/src/commands.rs`, and a new `helix-term/src/commands/standard.rs`.

`Range` already stores gap-indexed anchor and head positions and supports
zero-width carets. However, `Range::cursor` and `Range::put_cursor` explicitly
implement block-cursor semantics: a forward selection's cursor sits one
grapheme before its head. Rebinding existing extend commands would therefore
produce incorrect conventional selections.

Use gap-indexed caret semantics in Standard: `head` is the caret, `anchor` is
fixed during extension, and selected text is `[min(anchor, head), max(...))`.
Introduce Standard movement adapters that reuse position calculations while
constructing ranges with these semantics. Keep block-cursor helpers unchanged
for modal editing. Audit caret consumers in scrolling, mouse handling,
completion, LSP requests, and rendering so they use the logical Standard caret.

Preserve desired columns across short lines and soft wraps. Treat grapheme
clusters and CRLF as units. Test crossing the anchor in both directions,
document boundaries, empty lines, tabs, emoji, and combining characters.

Completion criterion: keyboard and mouse selections cover exactly the text
shown, and movement never splits a grapheme or CRLF pair.

## 3. Implement replacement editing and clipboard commands

Primary files: the Standard command module, `commands.rs`, and
`helix-term/src/ui/editor.rs`.

Use transactions for selection replacement, deletion, and paste. Reuse auto
pairs and indentation where appropriate, with deliberate behavior when text is
selected. Both selection directions must produce the same replacement result.
Backspace/Delete remove selected text before applying single-caret behavior.
Normalize pasted line endings to the document and leave the caret after the
inserted text. Avoid modal linewise paste behavior for clipboard text ending
in a newline. Dispatch existing text-change and insertion hooks as needed.

Keep the implementation compatible with multiple selections even before adding
shortcuts to create multiple carets. Apply simultaneous changes in one
transaction with correctly mapped resulting carets.

Completion criterion: type, delete, cut, and both paste paths replace a selected
range correctly and undo in a single step.

## 4. Route input and integrate editor services

Primary files:

- `helix-term/src/ui/editor.rs`: text fallback, popup precedence, selection
  rendering, mouse input, and terminal paste.
- `helix-term/src/handlers/completion.rs` and `completion/request.rs`.
- `helix-term/src/handlers/signature_help.rs`, `auto_save.rs`, and `diagnostics.rs`.
- `helix-term/src/ui/completion.rs`, `picker.rs`, and `prompt.rs`.

Standard must route unmapped printable input to insertion without command
counts or repeat recording. Unbound modified keys must not insert their base
character. Popup handlers get first refusal; Escape closes a popup without
leaving Standard. Search acceptance and cancellation must restore the intended
caret or selection and editing mode.

Audit each `Mode::Insert` comparison by purpose. Text input, automatic completion,
and signature help should support Standard; modal repeat recording should not.
Use small capability helpers where the same policy is shared. Check command-name
filters in completion hooks as well as mode checks, since new Standard commands
can otherwise cancel completion unexpectedly.

Provide theme fallbacks for Standard and a bar caret. Highlight the full selected
range without the inward block-cursor adjustment. Verify that caret placement,
scrolling, and LSP requests agree while a forward selection is active.

Completion criterion: completion, search, and file picking work without modal
commands, and the caret is correct after each interaction.

## 5. Define undo boundaries

Primary files: the Standard command module, `helix-term/src/ui/editor.rs`, and
the existing document history API in `helix-view/src/document.rs`.

The input loop currently commits pending changes outside Insert mode. Leaving
that condition unchanged would create one undo step per Standard keystroke.
Treat Standard as a grouped-edit mode and use explicit checkpoints:

- Group consecutive ordinary typing.
- End the group on navigation, selection changes, newline, or switching edit
  kind between insertion and deletion.
- Put paste, cut, selection replacement, and indentation in separate groups.
- Commit before save, undo, buffer changes, and split changes.
- Ensure redo remains available until a new edit occurs.

An idle-time boundary can be added after the deterministic command boundaries
work. History already exposes `append_changes_to_history`; use it rather than
replacing the undo tree.

Completion criterion: a sequence of typing, navigation, replacement, and paste
can be undone and redone in predictable steps while staying in Standard.

## 6. Validate and document the mode

Extend `helix-term/tests/integration.rs` with
`helix-term/tests/test/standard.rs`. Existing helpers accept key sequences and
assert text and selection state. Add startup-mode configuration and adapt the
helper's `<esc>:q!<ret>` cleanup, which assumes Escape reaches Normal mode.

Cover the editing contract through user-visible outcomes, plus mode transitions,
buffer/split changes, prompt cancellation, undo/redo, and regression coverage for
Normal/Select/Insert. Test clipboard behavior through an isolated provider or
test abstraction so automated tests do not depend on the user's clipboard.

Run the workspace unit tests and integration suite. Add the keymap and settings
to `book/src/keymap.md`, `book/src/configuration.md`, and `book/src/remapping.md`.
Regenerate command documentation with `cargo xtask docgen` when static commands
change. Retain the existing cross-platform CI matrix.

Manually check Windows Terminal first, then another terminal and a Unix host.
Confirm Ctrl/Shift combinations, bracketed paste, Unicode input, clipboard,
completion, and mouse dragging. Offer fallback bindings for combinations a
terminal intercepts.

## Delivery order

1. Mode registration, startup configuration, literal typing, and lifecycle tests.
2. Caret movement, selection, replacement, clipboard, and undo boundaries.
3. Search, file prompts, close/exit prompts, pickers, completion, and mouse behavior.
4. Cross-platform checks, configuration documentation, and modal regressions.

The first usable checkpoint is step 2. The highest-risk work is reconciling
caret semantics with shared cursor consumers and preserving mode across editor
services. Mode registration and keybinding definitions are comparatively small.
