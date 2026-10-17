use super::*;
use helix_core::chars::categorize_char;
use helix_view::keyboard::KeyModifiers;

pub(crate) fn checkpoint(cx: &mut Context) {
    if cx.editor.should_close() {
        return;
    }
    let (view, doc) = current!(cx.editor);
    doc.append_changes_to_history(view);
}

pub(super) fn standard_mode(cx: &mut Context) {
    if cx.editor.mode == Mode::Standard {
        return;
    }
    checkpoint(cx);
    cx.editor.set_mode(Mode::Standard);
}

pub(crate) fn text_char(event: KeyEvent) -> Option<char> {
    let ch = event.char()?;
    let modifiers = event.modifiers - KeyModifiers::SHIFT;
    let alt_gr =
        modifiers == (KeyModifiers::CONTROL | KeyModifiers::ALT) && !ch.is_ascii_alphanumeric();
    (modifiers.is_empty() || alt_gr)
        .then_some(ch)
        .filter(|ch| !ch.is_control())
}

#[derive(Clone, Copy)]
enum CaretMotion {
    Character(Direction),
    Word(Direction),
    Line(Direction),
    TenLines(Direction),
    Section(Direction),
    Page(Direction),
    LineStart,
    LineEnd,
    FileStart,
    FileEnd,
}

fn word_boundary(text: RopeSlice, mut pos: usize, direction: Direction) -> usize {
    match direction {
        Direction::Forward => {
            if pos == text.len_chars() {
                return pos;
            }
            let category = categorize_char(text.char(pos));
            while pos < text.len_chars() && categorize_char(text.char(pos)) == category {
                pos = graphemes::next_grapheme_boundary(text, pos);
            }
            while pos < text.len_chars() && text.char(pos).is_whitespace() {
                pos = graphemes::next_grapheme_boundary(text, pos);
            }
        }
        Direction::Backward => {
            while pos > 0 {
                let prev = graphemes::prev_grapheme_boundary(text, pos);
                if !text.char(prev).is_whitespace() {
                    break;
                }
                pos = prev;
            }
            if pos == 0 {
                return pos;
            }
            pos = graphemes::prev_grapheme_boundary(text, pos);
            let category = categorize_char(text.char(pos));
            while pos > 0 {
                let prev = graphemes::prev_grapheme_boundary(text, pos);
                if categorize_char(text.char(prev)) != category {
                    break;
                }
                pos = prev;
            }
        }
    }
    pos
}

fn syntax_boundary(
    text: RopeSlice,
    pos: usize,
    direction: Direction,
    syntax: &helix_core::Syntax,
    loader: &helix_core::syntax::Loader,
) -> Option<usize> {
    let byte = text.char_to_byte(pos);
    let layer = syntax.layer_for_byte_range(byte as u32, byte as u32);
    let root = syntax
        .tree_for_byte_range(byte as u32, byte as u32)
        .root_node();
    let query = loader.textobject_query(syntax.layer(layer).language)?;
    let boundaries = ["function", "class"]
        .into_iter()
        .flat_map(|object| {
            query
                .capture_nodes_any(
                    &[
                        &format!("{object}.movement"),
                        &format!("{object}.around"),
                        &format!("{object}.inside"),
                    ],
                    &root,
                    text,
                )
                .into_iter()
                .flatten()
        })
        .flat_map(|node| [node.start_byte(), node.end_byte()])
        .filter(|&boundary| boundary <= text.len_bytes());
    let boundary = match direction {
        Direction::Forward => boundaries.filter(|&boundary| boundary > byte).min(),
        Direction::Backward => boundaries.filter(|&boundary| boundary < byte).max(),
    }?;
    Some(text.byte_to_char(boundary))
}

fn section_boundary(text: RopeSlice, pos: usize, direction: Direction) -> usize {
    let is_blank = |line| text.line(line).chars().all(char::is_whitespace);
    let boundaries = |line| {
        if is_blank(line) {
            return [None, None];
        }
        [
            (line == 0 || is_blank(line - 1)).then(|| text.line_to_char(line)),
            (line + 1 == text.len_lines() || is_blank(line + 1))
                .then(|| line_end_char_index(&text, line)),
        ]
    };
    let line = text.char_to_line(pos);
    match direction {
        Direction::Forward => (line..text.len_lines())
            .flat_map(boundaries)
            .flatten()
            .find(|&boundary| boundary > pos)
            .unwrap_or(text.len_chars()),
        Direction::Backward => (0..=line)
            .rev()
            .flat_map(|line| boundaries(line).into_iter().rev())
            .flatten()
            .find(|&boundary| boundary < pos)
            .unwrap_or(0),
    }
}

fn move_caret(cx: &mut Context, motion: CaretMotion, extend: bool) {
    let (view, doc) = current!(cx.editor);
    let text = doc.text().slice(..);
    let loader = cx.editor.syn_loader.load();
    let text_fmt = doc.text_format(view.inner_width(doc), None);
    let mut annotations = view.text_annotations(doc, None);
    let selection = doc.selection(view.id).clone().transform(|range| {
        if !extend && !range.is_empty() {
            if let CaretMotion::Character(direction) = motion {
                return Range::point(match direction {
                    Direction::Backward => range.from(),
                    Direction::Forward => range.to(),
                });
            }
        }
        let mut caret = Range::point(range.head);
        caret.old_visual_position = range.old_visual_position;
        let head = match motion {
            CaretMotion::Character(Direction::Backward) => {
                graphemes::prev_grapheme_boundary(text, range.head)
            }
            CaretMotion::Character(Direction::Forward) => {
                graphemes::next_grapheme_boundary(text, range.head)
            }
            CaretMotion::Word(direction) => word_boundary(text, range.head, direction),
            CaretMotion::TenLines(direction) => {
                caret = move_vertically(
                    text,
                    caret,
                    direction,
                    10,
                    Movement::Move,
                    &text_fmt,
                    &mut annotations,
                );
                caret.head
            }
            CaretMotion::Section(direction) => doc
                .syntax()
                .and_then(|syntax| syntax_boundary(text, range.head, direction, syntax, &loader))
                .unwrap_or_else(|| section_boundary(text, range.head, direction)),
            CaretMotion::Line(direction) | CaretMotion::Page(direction) => {
                let count = if matches!(motion, CaretMotion::Page(_)) {
                    view.inner_height().saturating_sub(2).max(1)
                } else {
                    1
                };
                caret = move_vertically_visual(
                    text,
                    caret,
                    direction,
                    count,
                    Movement::Move,
                    &text_fmt,
                    &mut annotations,
                );
                caret.head
            }
            CaretMotion::LineStart => text.line_to_char(text.char_to_line(range.head)),
            CaretMotion::LineEnd => line_end_char_index(&text, text.char_to_line(range.head)),
            CaretMotion::FileStart => 0,
            CaretMotion::FileEnd => text.len_chars(),
        };
        let mut result = Range::new(if extend { range.anchor } else { head }, head);
        if matches!(
            motion,
            CaretMotion::Line(_) | CaretMotion::TenLines(_) | CaretMotion::Page(_)
        ) {
            result.old_visual_position = caret.old_visual_position;
        }
        result
    });
    drop(annotations);
    doc.set_selection(view.id, selection);
}

macro_rules! movements {
    ($($name:ident, $motion:expr, $extend:expr;)*) => {$(
        pub(super) fn $name(cx: &mut Context) {
            move_caret(cx, $motion, $extend);
        }
    )*};
}

movements! {
    standard_left, CaretMotion::Character(Direction::Backward), false;
    standard_right, CaretMotion::Character(Direction::Forward), false;
    standard_up, CaretMotion::Line(Direction::Backward), false;
    standard_down, CaretMotion::Line(Direction::Forward), false;
    standard_ten_lines_up, CaretMotion::TenLines(Direction::Backward), false;
    standard_ten_lines_down, CaretMotion::TenLines(Direction::Forward), false;
    standard_select_ten_lines_up, CaretMotion::TenLines(Direction::Backward), true;
    standard_select_ten_lines_down, CaretMotion::TenLines(Direction::Forward), true;
    standard_section_up, CaretMotion::Section(Direction::Backward), false;
    standard_section_down, CaretMotion::Section(Direction::Forward), false;
    standard_select_section_up, CaretMotion::Section(Direction::Backward), true;
    standard_select_section_down, CaretMotion::Section(Direction::Forward), true;
    standard_select_left, CaretMotion::Character(Direction::Backward), true;
    standard_select_right, CaretMotion::Character(Direction::Forward), true;
    standard_select_up, CaretMotion::Line(Direction::Backward), true;
    standard_select_down, CaretMotion::Line(Direction::Forward), true;
    standard_word_left, CaretMotion::Word(Direction::Backward), false;
    standard_word_right, CaretMotion::Word(Direction::Forward), false;
    standard_select_word_left, CaretMotion::Word(Direction::Backward), true;
    standard_select_word_right, CaretMotion::Word(Direction::Forward), true;
    standard_home, CaretMotion::LineStart, false;
    standard_end, CaretMotion::LineEnd, false;
    standard_select_home, CaretMotion::LineStart, true;
    standard_select_end, CaretMotion::LineEnd, true;
    standard_file_start, CaretMotion::FileStart, false;
    standard_file_end, CaretMotion::FileEnd, false;
    standard_select_file_start, CaretMotion::FileStart, true;
    standard_select_file_end, CaretMotion::FileEnd, true;
    standard_page_up, CaretMotion::Page(Direction::Backward), false;
    standard_page_down, CaretMotion::Page(Direction::Forward), false;
    standard_select_page_up, CaretMotion::Page(Direction::Backward), true;
    standard_select_page_down, CaretMotion::Page(Direction::Forward), true;
}

pub(super) fn standard_select_all(cx: &mut Context) {
    let (view, doc) = current!(cx.editor);
    doc.set_selection(view.id, Selection::single(0, doc.text().len_chars()));
}

pub(super) fn standard_escape(cx: &mut Context) {
    let (view, doc) = current!(cx.editor);
    let selection = Selection::point(doc.selection(view.id).primary().head);
    doc.set_selection(view.id, selection);
}

fn move_lines(cx: &mut Context, direction: Direction) {
    let (view, doc) = current!(cx.editor);
    let text = doc.text().slice(..);
    let selection = doc.selection(view.id);
    let mut changes = Vec::new();
    let mut moved_blocks = Vec::new();
    for (first, last) in selection.line_ranges(text) {
        let start = text.line_to_char(first);
        let end = line_end_char_index(&text, last);
        let (from, to, replacement, destination, after) = match direction {
            Direction::Backward if first > 0 => {
                let previous = text.line_to_char(first - 1);
                let previous_end = line_end_char_index(&text, first - 1);
                let separator = text.slice(previous_end..start);
                (
                    previous,
                    end,
                    format!(
                        "{}{}{}",
                        text.slice(start..end),
                        separator,
                        text.slice(previous..previous_end)
                    ),
                    previous,
                    previous + (end - start) + separator.len_chars(),
                )
            }
            Direction::Forward if last + 1 < text.len_lines() => {
                let next = text.line_to_char(last + 1);
                let next_end = line_end_char_index(&text, last + 1);
                let separator = text.slice(end..next);
                (
                    start,
                    next_end,
                    format!(
                        "{}{}{}",
                        text.slice(next..next_end),
                        separator,
                        text.slice(start..end)
                    ),
                    start + (next_end - next) + separator.len_chars(),
                    text.line_to_char((last + 2).min(text.len_lines())),
                )
            }
            _ => continue,
        };
        changes.push((from, to, Some(replacement.into())));
        moved_blocks.push((first, last, start, destination, after));
    }
    if changes.is_empty() {
        return;
    }
    let selection = selection.clone().transform(|range| {
        let (line, _) = range.line_range(text);
        let Some(&(_, last, start, destination, after)) = moved_blocks
            .iter()
            .find(|&&(first, last, ..)| first <= line && line <= last)
        else {
            return range;
        };
        let old_after = text.line_to_char((last + 1).min(text.len_lines()));
        let map = |pos| {
            if last + 1 < text.len_lines() && !range.is_empty() && pos == old_after {
                after
            } else {
                destination + (pos - start)
            }
        };
        Range::new(map(range.anchor), map(range.head))
    });
    let transaction =
        Transaction::change(doc.text(), changes.into_iter()).with_selection(selection);
    doc.apply(&transaction, view.id);
}

pub(super) fn standard_move_lines_up(cx: &mut Context) {
    move_lines(cx, Direction::Backward);
}

pub(super) fn standard_move_lines_down(cx: &mut Context) {
    move_lines(cx, Direction::Forward);
}

fn set_comments(cx: &mut Context, uncomment: bool) {
    checkpoint(cx);
    let loader = cx.editor.syn_loader.load();
    let (view, doc) = current!(cx.editor);
    let text = doc.text().slice(..);
    let selection = doc.selection(view.id);
    let (line_tokens, block_tokens) = comment_tokens(doc, &loader, selection.primary().head);
    let lines: Vec<_> = selection
        .line_ranges(text)
        .flat_map(|(first, last)| first..=last)
        .filter_map(|line| Some((line, text.line(line).first_non_whitespace_char()?)))
        .collect();

    let transaction = if line_tokens.is_none_or(|tokens| tokens.is_empty())
        && block_tokens.is_some_and(|tokens| !tokens.is_empty())
    {
        if lines.is_empty() {
            return;
        }
        let line_selection = Selection::new(
            lines
                .iter()
                .map(|&(line, _)| {
                    Range::new(text.line_to_char(line), line_end_char_index(&text, line))
                })
                .collect(),
            0,
        );
        let (_, changes) =
            comment::find_block_comments(block_tokens.unwrap(), text, &line_selection);
        comment::create_block_comment_transaction(doc.text(), &line_selection, uncomment, changes).0
    } else {
        let default_tokens = [comment::DEFAULT_COMMENT_TOKEN.to_owned()];
        let tokens = line_tokens
            .filter(|tokens| !tokens.is_empty())
            .unwrap_or(&default_tokens);
        let indent = lines.iter().map(|&(_, indent)| indent).min().unwrap_or(0);
        let changes = lines.into_iter().filter_map(|(line, offset)| {
            let existing = comment::get_comment_token(text, tokens, line);
            let start = text.line_to_char(line);
            if uncomment {
                let token = existing?;
                let from = start + offset;
                let end = from + token.chars().count();
                let margin = usize::from(text.get_char(end) == Some(' '));
                Some((from, end + margin, None))
            } else if existing.is_none() {
                let pos = start + indent;
                Some((pos, pos, Some(format!("{} ", tokens[0]).into())))
            } else {
                None
            }
        });
        Transaction::change(doc.text(), changes)
    };
    let selection = selection.clone().map(transaction.changes());
    doc.apply(&transaction.with_selection(selection), view.id);
    doc.append_changes_to_history(view);
}

pub(super) fn standard_comment(cx: &mut Context) {
    set_comments(cx, false);
}

pub(super) fn standard_uncomment(cx: &mut Context) {
    set_comments(cx, true);
}

pub(crate) fn insert_character(cx: &mut Context, ch: char) {
    let (view, doc) = current_ref!(cx.editor);
    let loader = cx.editor.syn_loader.load();
    let pairs = doc.auto_pairs(cx.editor, &loader, view);
    let transaction =
        Transaction::change_by_and_with_selection(doc.text(), doc.selection(view.id), |range| {
            if range.is_empty() {
                if let Some(result) = pairs.and_then(|pairs| {
                    helix_core::auto_pairs::hook_insert(
                        doc.text(),
                        &range.min_width_1(doc.text().slice(..)),
                        ch,
                        pairs,
                    )
                }) {
                    return (result.0, Some(Range::point(result.1.from())));
                }
            }
            let from = range.from();
            (
                (from, range.to(), Some(Tendril::from_iter([ch]))),
                Some(Range::point(from + 1)),
            )
        });
    let view_id = view.id;
    let doc_id = doc.id();
    drop(loader);
    doc_mut!(cx.editor, &doc_id).apply(&transaction, view_id);
    helix_event::dispatch(crate::events::PostInsertChar { c: ch, cx });
}

fn delete(cx: &mut Context, direction: Direction, word: bool) {
    let (view, doc) = current_ref!(cx.editor);
    if !word
        && direction == Direction::Backward
        && doc.selection(view.id).iter().all(|r| r.is_empty())
    {
        insert::delete_char_backward(cx);
        return;
    }
    let text = doc.text().slice(..);
    let transaction =
        Transaction::delete_by_and_with_selection(doc.text(), doc.selection(view.id), |range| {
            let (from, to) = if !range.is_empty() {
                (range.from(), range.to())
            } else {
                let other = if word {
                    word_boundary(text, range.head, direction)
                } else if direction == Direction::Backward {
                    graphemes::prev_grapheme_boundary(text, range.head)
                } else {
                    graphemes::next_grapheme_boundary(text, range.head)
                };
                (other.min(range.head), other.max(range.head))
            };
            ((from, to), Some(Range::point(from)))
        });
    let view_id = view.id;
    let doc_id = doc.id();
    doc_mut!(cx.editor, &doc_id).apply(&transaction, view_id);
}

pub(super) fn standard_delete_backward(cx: &mut Context) {
    delete(cx, Direction::Backward, false);
}
pub(super) fn standard_delete_forward(cx: &mut Context) {
    delete(cx, Direction::Forward, false);
}
pub(super) fn standard_delete_word_backward(cx: &mut Context) {
    delete(cx, Direction::Backward, true);
}
pub(super) fn standard_delete_word_forward(cx: &mut Context) {
    delete(cx, Direction::Forward, true);
}

fn copy_selection(cx: &mut Context) -> bool {
    let (view, doc) = current_ref!(cx.editor);
    let text = doc.text().slice(..);
    let values: Vec<_> = doc
        .selection(view.id)
        .iter()
        .filter(|r| !r.is_empty())
        .map(|r| text.slice(r.from()..r.to()).to_string())
        .collect();
    if values.is_empty() {
        return false;
    }
    if let Err(err) = cx.editor.registers.write('+', values) {
        cx.editor
            .set_error(format!("Cannot copy to clipboard: {err}"));
        return false;
    }
    true
}

pub(super) fn standard_copy(cx: &mut Context) {
    copy_selection(cx);
}

pub(super) fn standard_cut(cx: &mut Context) {
    let (view, doc) = current_ref!(cx.editor);
    if doc.selection(view.id).iter().all(|range| range.is_empty()) {
        cut_lines(cx);
        return;
    }
    if copy_selection(cx) {
        let (view, doc) = current!(cx.editor);
        let transaction =
            Transaction::change_by_and_with_selection(doc.text(), doc.selection(view.id), |r| {
                ((r.from(), r.to(), None), Some(Range::point(r.from())))
            });
        doc.apply(&transaction, view.id);
    }
}

fn cut_lines(cx: &mut Context) {
    let (view, doc) = current_ref!(cx.editor);
    let text = doc.text().slice(..);
    if text.len_chars() == 0 {
        return;
    }
    let selection = doc.selection(view.id);
    let mut deletions = Vec::new();
    let mut values = Vec::new();
    for (first, last) in selection.line_ranges(text) {
        let start = text.line_to_char(first);
        let end = text.line_to_char(last + 1);
        let from = if last + 1 == text.len_lines() && first > 0 {
            line_end_char_index(&text, first - 1)
        } else {
            start
        };
        let clipboard_start = if start == end { from } else { start };
        values.push(text.slice(clipboard_start..end).to_string());
        deletions.push((from, end));
    }
    let transaction = Transaction::delete(doc.text(), deletions.into_iter());
    let selection = selection.clone().map(transaction.changes());
    let transaction = transaction.with_selection(selection);
    let view_id = view.id;
    let doc_id = doc.id();
    if let Err(err) = cx.editor.registers.write('+', values) {
        cx.editor
            .set_error(format!("Cannot copy to clipboard: {err}"));
        return;
    }
    doc_mut!(cx.editor, &doc_id).apply(&transaction, view_id);
}

fn replace_values(cx: &mut Context, values: &[String]) {
    if values.is_empty() {
        return;
    }
    let (view, doc) = current!(cx.editor);
    let line_ending = doc.line_ending.as_str();
    let values: Vec<_> = values
        .iter()
        .map(|s| LINE_ENDING_REGEX.replace_all(s, line_ending).into_owned())
        .collect();
    let joined = values.join(line_ending);
    let distribute = values.len() == doc.selection(view.id).len();
    let mut index = 0;
    let transaction =
        Transaction::change_by_and_with_selection(doc.text(), doc.selection(view.id), |r| {
            let value = if distribute { &values[index] } else { &joined };
            index += 1;
            (
                (r.from(), r.to(), Some(value.as_str().into())),
                Some(Range::point(r.from() + value.chars().count())),
            )
        });
    doc.apply(&transaction, view.id);
}

pub(crate) fn paste_text(cx: &mut Context, text: &str) {
    checkpoint(cx);
    replace_values(cx, &[text.to_owned()]);
    checkpoint(cx);
}

pub(super) fn standard_paste(cx: &mut Context) {
    let values: Vec<_> = cx
        .editor
        .registers
        .read('+', cx.editor)
        .into_iter()
        .flatten()
        .map(Cow::into_owned)
        .collect();
    replace_values(cx, &values);
}

fn remove_selected_text(cx: &mut Context) {
    replace_values(cx, &[String::new()]);
}

pub(super) fn standard_newline(cx: &mut Context) {
    remove_selected_text(cx);
    insert::insert_newline(cx);
}

pub(super) fn standard_tab(cx: &mut Context) {
    let (view, doc) = current_ref!(cx.editor);
    if doc.selection(view.id).iter().any(|r| !r.is_empty()) {
        indent(cx);
    } else {
        insert::insert_tab(cx);
    }
}

pub(super) fn standard_unindent(cx: &mut Context) {
    unindent(cx);
}
