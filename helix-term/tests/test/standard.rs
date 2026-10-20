use super::helpers::*;
use anyhow::Context as _;
use helix_core::Selection;
use helix_term::config::Config;
use helix_view::{clipboard::ClipboardProvider, document::Mode, editor::DefaultMode};

fn config() -> Config {
    let mut config = test_config();
    config.editor.default_mode = DefaultMode::Standard;
    config.editor.clipboard_provider = ClipboardProvider::None;
    config.editor.auto_pairs = helix_core::syntax::config::AutoPairConfig::Enable(false);
    config
}

async fn standard(input: &str, keys: &str, output: &str) -> anyhow::Result<()> {
    let native = |text: &str| text.replace('\n', helix_core::NATIVE_LINE_ENDING.as_str());
    test_with_config(
        AppBuilder::new().with_config(config()),
        (native(input), keys, native(output), LineFeedHandling::AsIs),
    )
    .await
}

async fn standard_vertical(
    input: &str,
    keys: &str,
    output: &str,
    column: u32,
) -> anyhow::Result<()> {
    let native = |text: &str| text.replace('\n', helix_core::NATIVE_LINE_ENDING.as_str());
    let mut case = TestCase::from((native(input), keys, native(output), LineFeedHandling::AsIs));
    case.out_selection = case.out_selection.transform(|mut range| {
        range.old_visual_position = Some((0, column));
        range
    });
    test_with_config(AppBuilder::new().with_config(config()), case).await
}

#[tokio::test(flavor = "multi_thread")]
async fn literal_typing_and_unbound_modifiers() -> anyhow::Result<()> {
    standard("#[|]#", "hjkl 123<C-g><A-g>!", "hjkl 123!#[|]#").await
}

#[tokio::test(flavor = "multi_thread")]
async fn selection_replacement_and_collapse() -> anyhow::Result<()> {
    for (keys, output) in [
        ("<S-right><S-right>x", "ax#[|]#d"),
        ("<S-right><S-right><left>", "a#[|]#bcd"),
        ("<S-right><S-right><right>", "abc#[|]#d"),
        ("<S-right><S-left><S-left>x", "x#[|]#bcd"),
        ("<S-right><S-right><backspace>", "a#[|]#d"),
        ("<S-right><S-right><del>", "a#[|]#d"),
        ("<S-right><S-right><esc>", "abc#[|]#d"),
    ] {
        standard("a#[|]#bcd", keys, output).await?;
    }
    standard("abc#[|]#d", "<S-left><S-left>x", "ax#[|]#d").await
}

#[tokio::test(flavor = "multi_thread")]
async fn word_and_line_navigation() -> anyhow::Result<()> {
    standard(
        "#[|]#one two three",
        "<C-right><C-S-right>x",
        "one x#[|]#three",
    )
    .await?;
    standard("one two three#[|]#", "<C-backspace>", "one two #[|]#").await?;
    standard("#[|]#one two", "<C-del>", "#[|]#two").await?;
    standard("ab#[|]#cd", "<home><S-end>x", "x#[|]#").await?;
    standard("ab#[|]#cd", "<C-end><C-S-home>x", "x#[|]#").await
}

#[tokio::test(flavor = "multi_thread")]
async fn grapheme_and_crlf_boundaries() -> anyhow::Result<()> {
    standard("#[|]#e\u{301}👨‍👩‍👧‍👦z", "<right><del>", "e\u{301}#[|]#z").await?;
    standard("e\u{301}👨‍👩‍👧‍👦#[|]#z", "<backspace><backspace>", "#[|]#z").await?;
    standard("ab#[|]#\ncd", "<del>", "ab#[|]#cd").await?;
    standard("ab\n#[|]#cd", "<backspace>", "ab#[|]#cd").await
}

#[tokio::test(flavor = "multi_thread")]
async fn clipboard_and_undo() -> anyhow::Result<()> {
    standard(
        "#[|]#abcd",
        "<S-right><S-right><C-c><end><C-v>",
        "abcdab#[|]#",
    )
    .await?;
    standard(
        "#[|]#abcd",
        "<S-right><S-right><C-x><end><C-v>",
        "cdab#[|]#",
    )
    .await?;
    standard("#[|]#abcd", "<S-right><S-right><C-c><C-a><C-v>", "ab#[|]#").await?;
    standard("#[|]#abcd", "<S-right><S-right><C-x><C-z>", "#[ab|]#cd").await?;
    standard("#[|]#", "hello<left>X<C-z>", "hell#[|]#o").await?;
    standard("#[|]#", "hello<backspace><backspace><C-z>", "hello#[|]#").await?;
    standard("#[|]#", "hello<C-z><C-y>", "hello#[|]#").await?;
    standard("#[|]#", "hello<C-z><C-S-z>", "hello#[|]#").await
}

#[tokio::test(flavor = "multi_thread")]
async fn copy_whole_line_without_selection() -> anyhow::Result<()> {
    for (input, output) in [
        (
            "first\nse#[|]#cond\nlast",
            "first\nsecond\nsecond#[|]#\nlast",
        ),
        ("on#[|]#ly", "only\nonly#[|]#"),
        ("on#[|]#ly\n", "only\nonly#[|]#\n"),
        ("last\n#[|]#", "last\n\n#[|]#"),
        ("#[|]#", "\n#[|]#"),
        ("  é👩‍💻#[|]#z", "  é👩‍💻z\n  é👩‍💻z#[|]#"),
    ] {
        standard(input, "<C-c><C-v>", output).await?;
    }
    standard(
        "fi#[|]#rst\nsecond\nlast",
        "<C-c><down><C-v>",
        "first\nsecond\nfirst#[|]#\nlast",
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn line_copy_undo_and_redo() -> anyhow::Result<()> {
    let input = "a#[|]#bc\nnext";
    standard(input, "<C-c>", input).await?;
    standard(input, "<C-c><C-v><C-z>", input).await?;
    standard(input, "<C-c><C-v><C-z><C-y>", "abc\nabc#[|]#\nnext").await?;
    standard(input, "<C-c><C-v><C-v>", "abc\nabc\nabc#[|]#\nnext").await
}

#[tokio::test(flavor = "multi_thread")]
async fn line_copy_with_multiple_carets() -> anyhow::Result<()> {
    for (input, output) in [
        ("a#[|]#b\nc#(|)#d", "ab\nab#[|]#\ncd\ncd#(|)#"),
        ("a#[|]#\nb\nc#(|)#", "a\na#[|]#\nb\nc\nc#(|)#"),
        ("a#(|)#b#[|]#c", "abc\nabc#[|]#"),
        ("a#(|)#\nc#[|]#", "a\na#(|)#\nc\nc#[|]#"),
    ] {
        standard(input, "<C-c><C-v>", output).await?;
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn line_copy_preserves_selected_paste_and_character_copy() -> anyhow::Result<()> {
    standard("a#[|]#bc\nnext", "<C-c><C-a><C-v>", "abc\n#[|]#").await?;
    standard(
        "a#[|]#bc\nnext",
        "<C-c><home><S-right><C-c><end><C-v>",
        "abca#[|]#\nnext",
    )
    .await?;
    standard(
        "a#[|]#bc\nnext",
        "<C-c><home><S-end><S-right><C-c><end><C-v>",
        "abc\nnextabc\n#[|]#",
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn line_copy_and_paste_normalize_line_endings() -> anyhow::Result<()> {
    for eol in ["\n", "\r\n"] {
        let mut config = config();
        config.editor.default_line_ending = if eol == "\n" {
            helix_view::editor::LineEndingConfig::LF
        } else {
            helix_view::editor::LineEndingConfig::Crlf
        };
        test_with_config(
            AppBuilder::new().with_config(config.clone()),
            (
                format!("a#[|]#b{eol}next"),
                "<C-c><C-v>",
                format!("ab{eol}ab#[|]#{eol}next"),
                LineFeedHandling::AsIs,
            ),
        )
        .await?;
        // Explicit positions avoid the annotation helper collapsing adjacent
        // newlines around a zero-width selection.
        test_with_config(
            AppBuilder::new().with_config(config),
            TestCase {
                in_text: format!("{eol}next"),
                in_selection: Selection::point(0),
                in_keys: "<C-c><C-v>".to_owned(),
                out_text: format!("{eol}{eol}next"),
                out_selection: Selection::point(eol.chars().count()),
                line_feed_handling: LineFeedHandling::AsIs,
            },
        )
        .await?;
    }
    Ok(())
}

async fn comment_case(file: &str, input: &str, keys: &str, output: &str) -> anyhow::Result<()> {
    test_with_config(
        AppBuilder::new()
            .with_config(config())
            .with_file(file, None),
        (input, keys, output, LineFeedHandling::AsIs),
    )
    .await
    .with_context(|| format!("{file}: {keys}"))
}

#[tokio::test(flavor = "multi_thread")]
async fn comment_chords_use_language_tokens_and_preserve_carets() -> anyhow::Result<()> {
    for (file, token) in [
        ("standard-comments.cs", "//"),
        ("standard-comments.odin", "//"),
        ("standard-comments.py", "#"),
        ("standard-comments.txt", "#"),
    ] {
        let input = "    é👩‍💻#[|]#value";
        let output = format!("    {token} é👩‍💻#[|]#value");
        comment_case(file, input, "<C-k><C-c>", &output).await?;
        comment_case(file, &output, "<C-k><C-u>", input).await?;
    }
    comment_case("standard-comments.cs", "a#[|]#b", "<C-k>c", "// a#[|]#b").await?;
    comment_case("standard-comments.cs", "// a#[|]#b", "<C-k>u", "a#[|]#b").await?;
    comment_case(
        "standard-comments.cs",
        "#[|]#",
        "<C-k><C-c><C-k><C-u>",
        "#[|]#",
    )
    .await?;
    comment_case(
        "standard-comments.cs",
        "  #[|]#\r\n",
        "<C-k><C-c>",
        "  #[|]#\r\n",
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn comment_chords_preserve_selection_direction_and_line_boundaries() -> anyhow::Result<()> {
    for (input, output) in [
        (
            "    #[first();\n        second();|]#\nthird();",
            "    // #[first();\n    //     second();|]#\nthird();",
        ),
        ("#[|first();\nsecond();]#", "// #[|first();\n// second();]#"),
        ("#[a\n|]#b", "// #[a\n|]#b"),
        ("#[|a\n]#b", "// #[|a\n]#b"),
        ("#[a\r\nb\nc|]#", "// #[a\r\n// b\n// c|]#"),
        ("#[|]#a#(|)#b\n  c#(|)#d", "// #[|]#a#(|)#b\n//   c#(|)#d"),
    ] {
        comment_case("standard-comments.cs", input, "<C-k><C-c>", output).await?;
        comment_case("standard-comments.cs", output, "<C-k><C-u>", input).await?;
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn comment_chords_handle_mixed_comments_and_are_idempotent() -> anyhow::Result<()> {
    comment_case(
        "standard-comments.cs",
        "#[    // old();\n      new();|]#",
        "<C-k><C-c><C-k><C-c>",
        "#[    // old();\n    //   new();|]#",
    )
    .await?;
    comment_case(
        "standard-comments.cs",
        "#[    // a\n        //b\n  plain\n    /// doc|]#",
        "<C-k><C-u><C-k><C-u>",
        "#[    a\n        b\n  plain\n    doc|]#",
    )
    .await?;
    comment_case("standard-comments.cs", "#[ab|]#", "<C-k><C-u>", "#[ab|]#").await
}

#[tokio::test(flavor = "multi_thread")]
async fn comment_chords_support_block_only_languages() -> anyhow::Result<()> {
    for (file, input, output) in [
        (
            "standard-comments.css",
            "  a#[|]# { color: red; }",
            "  /* a#[|]# { color: red; } */",
        ),
        (
            "standard-comments.html",
            "  <#[|]#p>x</p>",
            "  <!-- <#[|]#p>x</p> -->",
        ),
        (
            "standard-comments.css",
            "#[a {}\r\n  b {}|]#",
            "/* #[a {} */\r\n  /* b {}|]# */",
        ),
    ] {
        comment_case(file, input, "<C-k><C-c><C-k><C-c>", output).await?;
        comment_case(file, output, "<C-k><C-u><C-k><C-u>", input).await?;
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn comment_chords_are_undoable_and_keep_standard_mode() -> anyhow::Result<()> {
    for (keys, output) in [
        ("<C-k><C-c><C-z>", "a#[|]#b"),
        ("<C-k><C-c><C-z><C-y>", "// a#[|]#b"),
        ("<C-k><C-c>X<C-z>", "// a#[|]#b"),
        ("<C-k><C-c>X<C-z><C-z>", "a#[|]#b"),
        ("<C-k><C-c>X", "// aX#[|]#b"),
        ("<C-k><C-c><C-k><C-u><C-z>", "// a#[|]#b"),
        ("<C-k><esc>X", "aX#[|]#b"),
        ("<C-k>x", "ax#[|]#b"),
        ("<C-k><C-k><esc>X", "aX#[|]#b"),
    ] {
        comment_case("standard-comments.cs", "a#[|]#b", keys, output).await?;
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn cut_whole_line_without_selection() -> anyhow::Result<()> {
    for (input, output) in [
        ("fi#[|]#rst\nsecond\nlast", "#[|]#second\nlast"),
        ("first\nse#[|]#cond\nlast", "first\n#[|]#last"),
        ("first\nla#[|]#st", "first#[|]#"),
        ("first\nla#[|]#st\n", "first\n#[|]#"),
        ("on#[|]#ly", "#[|]#"),
        ("#[|]#", "#[|]#"),
        ("first\né👩‍💻#[|]#z\nlast", "first\n#[|]#last"),
    ] {
        standard(input, "<C-x>", output).await?;
    }
    standard(
        "first\nse#[|]#cond\nlast",
        "<C-x><C-v>",
        "first\nsecond\n#[|]#last",
    )
    .await?;
    standard("#[ab|]#", "<C-x><C-x><C-v>", "ab#[|]#").await
}

#[tokio::test(flavor = "multi_thread")]
async fn cut_line_clipboard_excludes_preceding_separator() -> anyhow::Result<()> {
    let mut app = AppBuilder::new()
        .with_config(config())
        .with_input_text("first\nse#[|]#cond")
        .build()?;
    test_key_sequence(
        &mut app,
        Some("<C-x>"),
        Some(&|app| {
            let values: Vec<_> = app
                .editor
                .registers
                .read('+', &app.editor)
                .unwrap()
                .collect();
            assert_eq!(values, ["second"]);
        }),
        false,
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn cut_lines_with_multiple_carets() -> anyhow::Result<()> {
    for (input, output) in [
        ("a\nb#[|]#b #(|)#c\nd", "a\n#[|]#d"),
        ("a\nb#[|]#\nc#(|)#\nd", "a\n#[|]#d"),
        ("a\nb#[|]#\nc#(|)#", "a#[|]#"),
        ("a\nb#[|]#\nc\nd#(|)#\ne", "a\n#[|]#c\n#(|)#e"),
        ("a\nb#(|)#\nc\nd#[|]#\ne", "a\n#(|)#c\n#[|]#e"),
    ] {
        standard(input, "<C-x>", output).await?;
    }
    standard("a\nb#[|]#b #(|)#c\nd", "<C-x><C-v>", "a\nbb c\n#[|]#d").await
}

#[tokio::test(flavor = "multi_thread")]
async fn cut_line_undo_restores_original_caret() -> anyhow::Result<()> {
    let input = "first\nse#[|]#cond\nlast";
    standard(input, "<C-x><C-z>", input).await?;
    standard(input, "<C-x><C-z><C-y>", "first\n#[|]#last").await?;
    standard(input, "x<C-x><C-z>", "first\nsex#[|]#cond\nlast").await?;
    standard(input, "x<C-x><C-z><C-z>", input).await?;
    standard("a\nb#[|]#\nc#(|)#", "<C-x><C-z>", "a\nb#[|]#\nc#(|)#").await
}

#[tokio::test(flavor = "multi_thread")]
async fn cut_empty_lines_including_eof() -> anyhow::Result<()> {
    for eol in ["\n", "\r\n"] {
        let first = format!("a{eol}");
        let middle = format!("{first}{eol}b");
        let end = format!("{first}b{eol}");
        for (input, pos, output, expected_pos) in [
            (&middle, first.len(), format!("{first}b"), first.len()),
            (&end, end.len(), format!("{first}b"), first.len() + 1),
        ] {
            test_with_config(
                AppBuilder::new().with_config(config()),
                TestCase {
                    in_text: input.clone(),
                    in_selection: Selection::point(pos),
                    in_keys: "<C-x>".to_owned(),
                    out_text: output,
                    out_selection: Selection::point(expected_pos),
                    line_feed_handling: LineFeedHandling::AsIs,
                },
            )
            .await?;
        }
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn newline_tab_and_multiple_carets() -> anyhow::Result<()> {
    standard("ab#[cd|]#ef", "<ret>x", "ab\nx#[|]#ef").await?;
    standard("#[|]#", "<tab>x<backspace><backspace>", "#[|]#").await?;
    standard("#[|]#ab #(|)#cd", "X", "X#[|]#ab X#(|)#cd").await
}

#[tokio::test(flavor = "multi_thread")]
async fn mode_survives_buffer_and_split_changes() -> anyhow::Result<()> {
    let mut app = AppBuilder::new()
        .with_config(config())
        .with_input_text("#[|]#hello\n")
        .build()?;
    assert_eq!(app.editor.mode(), Mode::Standard);
    let first = app.editor.tree.focus;
    app.editor
        .new_file(helix_view::editor::Action::VerticalSplit);
    assert_eq!(app.editor.mode(), Mode::Standard);
    app.editor.focus(first);
    assert_eq!(app.editor.mode(), Mode::Standard);
    test_key_sequence(
        &mut app,
        Some("x"),
        Some(&|app| {
            let (view, doc) = helix_view::current_ref!(app.editor);
            assert!(doc.text().to_string().starts_with("xhello"));
            assert_eq!(doc.selection(view.id), &Selection::point(1));
            assert_eq!(app.editor.mode(), Mode::Standard);
        }),
        false,
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn vertical_movement_preserves_column_and_caret_rendering() -> anyhow::Result<()> {
    let mut app = AppBuilder::new()
        .with_config(config())
        .with_input_text("abc#[|]#def\nx\nabcdef\n")
        .build()?;
    let start = app.editor.cursor().0.unwrap();
    test_key_sequence(
        &mut app,
        Some("<S-down><S-down>"),
        Some(&|app| {
            let (view, doc) = helix_view::current_ref!(app.editor);
            let range = doc.selection(view.id).primary();
            assert_eq!(range.anchor, 3);
            assert_eq!(range.head, 12);
            let caret = app.editor.cursor().0.unwrap();
            assert_eq!(caret.col, start.col);
            assert_eq!(caret.row, start.row + 2);
        }),
        false,
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn ten_line_jumps_preserve_column_and_clamp() -> anyhow::Result<()> {
    let fixture = |line: usize, col: usize| {
        (0..25)
            .map(|i| {
                let mut text = if i == 10 { "x" } else { "abcdef" }.to_owned();
                if i == line {
                    text.insert_str(col, "#[|]#");
                }
                text
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    for (keys, line, col) in [
        ("<C-down>", 10, 1),
        ("<C-down><C-down>", 20, 3),
        ("<C-down><C-down><C-up><C-up>", 0, 3),
        ("<C-up><C-up>", 0, 3),
        ("<C-down><C-down><C-down><C-down>", 24, 3),
    ] {
        standard_vertical(&fixture(0, 3), keys, &fixture(line, col), 3).await?;
    }
    standard_vertical("#[|]#", "<C-up><C-down>", "#[|]#", 0).await?;
    standard_vertical("abc#[|]#\n", "<C-down>", "abc\n#[|]#", 3).await
}

#[tokio::test(flavor = "multi_thread")]
async fn ten_line_jumps_extend_and_reverse_selection() -> anyhow::Result<()> {
    let middle = "abcdef\n".repeat(9);
    let input = format!("abc#[|]#def\n{middle}abcdef");
    let selected = format!("abc#[def\n{middle}abc|]#def");
    standard_vertical(&input, "<C-S-down>", &selected, 3).await?;
    standard_vertical(&input, "<C-S-down><C-S-up>", &input, 3).await?;
    standard_vertical(&selected, "<C-up>", &input, 3).await?;
    standard_vertical(
        &format!("abcdef\n{middle}abc#[|]#def"),
        "<C-S-up>",
        &format!("abc#[|def\n{middle}abc]#def"),
        3,
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn ten_line_jumps_count_source_lines_with_soft_wrap() -> anyhow::Result<()> {
    let mut config = config();
    config.editor.soft_wrap.enable = Some(true);
    let line = "x".repeat(200);
    let mut app = AppBuilder::new()
        .with_config(config)
        .with_input_text(format!(
            "abc#[|]#def\n{}abcdef",
            format!("{line}\n").repeat(9)
        ))
        .build()?;
    test_key_sequence(
        &mut app,
        Some("<C-down>"),
        Some(&|app| {
            let (view, doc) = helix_view::current_ref!(app.editor);
            let range = doc.selection(view.id).primary();
            assert!(range.is_empty());
            assert_eq!(range.head, doc.text().line_to_char(10) + 3);
        }),
        false,
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn contextual_jumps_follow_blank_line_sections() -> anyhow::Result<()> {
    let fixture = "alpha\nbeta\n \t\ngamma\ndelta";
    for (keys, pos) in [
        ("<C-A-down>", fixture.find("beta").unwrap() + 4),
        ("<C-A-down><C-A-down>", fixture.find("gamma").unwrap()),
        ("<C-A-down><C-A-down><C-A-down>", fixture.len()),
        (
            "<C-A-down><C-A-down><C-A-up>",
            fixture.find("beta").unwrap() + 4,
        ),
        ("<C-A-up>", 0),
    ] {
        let mut output = fixture.to_owned();
        output.insert_str(pos, "#[|]#");
        standard(&format!("#[|]#{fixture}"), keys, &output).await?;
    }
    standard(
        "alpha\nbeta\n\ngamma#[|]#\ndelta",
        "<C-A-up>",
        "alpha\nbeta\n\n#[|]#gamma\ndelta",
    )
    .await?;
    standard("#[|]#a\n\nb", "<C-A-S-down>", "#[a|]#\n\nb").await?;
    standard("a\n\nb#[|]#", "<C-A-S-up>", "a\n\n#[|b]#").await?;
    standard("#[|]#", "<C-A-up><C-A-down>", "#[|]#").await
}

#[tokio::test(flavor = "multi_thread")]
async fn contextual_jumps_follow_syntax_boundaries() -> anyhow::Result<()> {
    let fixture =
        "fn first() {\n    let s = \"é👩‍💻\";\n\n    work();\n}\nstruct Item {\n    field: u8,\n}\nfn last() {}";
    let first_end = fixture.find('}').unwrap() + 1;
    let class_start = fixture.find("struct").unwrap();
    let class_end = fixture[class_start..].find('}').unwrap() + class_start + 1;
    let last_start = fixture.find("fn last").unwrap();
    for (keys, pos) in [
        ("<C-A-down>", first_end),
        ("<C-A-down><C-A-down>", class_start),
        ("<C-A-down><C-A-down><C-A-down>", class_end),
        ("<C-A-down><C-A-down><C-A-down><C-A-down>", last_start),
        ("<C-A-down><C-A-down><C-A-up>", first_end),
        ("<C-A-down><C-A-up>", 0),
    ] {
        let mut output = fixture.to_owned();
        output.insert_str(pos, "#[|]#");
        test_with_config(
            AppBuilder::new()
                .with_config(config())
                .with_file("standard-navigation.rs", None),
            (
                format!("#[|]#{fixture}"),
                keys,
                output,
                LineFeedHandling::AsIs,
            ),
        )
        .await?;
    }
    for (input, keys, output) in [
        (
            "fn f() { #[|]#body(); }",
            "<C-A-down>",
            "fn f() { body(); }#[|]#",
        ),
        (
            "fn f() { #[|]#body(); }",
            "<C-A-up>",
            "#[|]#fn f() { body(); }",
        ),
        ("#[|]#fn f() {}", "<C-A-S-down>", "#[fn f() {}|]#"),
        ("fn f() {}#[|]#", "<C-A-S-up>", "#[|fn f() {}]#"),
    ] {
        test_with_config(
            AppBuilder::new()
                .with_config(config())
                .with_file("standard-navigation.rs", None),
            (input, keys, output, LineFeedHandling::AsIs),
        )
        .await?;
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn move_current_line_keeps_caret_and_supports_repetition() -> anyhow::Result<()> {
    for (input, keys, output) in [
        ("a\nbb#[|]#b\ncccc", "<A-up>", "bb#[|]#b\na\ncccc"),
        ("a\nbb#[|]#b\ncccc", "<A-down>", "a\ncccc\nbb#[|]#b"),
        ("aa\nb#[|]#b\nc\nd", "<A-down><A-down>", "aa\nc\nd\nb#[|]#b"),
        (
            "aa\nb#[|]#b\nc\nd",
            "<A-down><A-down><A-up><A-up>",
            "aa\nb#[|]#b\nc\nd",
        ),
        ("#[|]#a\nb", "<A-up>", "#[|]#a\nb"),
        ("a\nb#[|]#", "<A-down>", "a\nb#[|]#"),
        ("#[|]#", "<A-up><A-down>", "#[|]#"),
        ("single#[|]#", "<A-up><A-down>", "single#[|]#"),
        ("a\nb#[|]#\n", "<A-up>", "b#[|]#\na\n"),
        ("a\nb#[|]#\n", "<A-down>", "a\n\nb#[|]#"),
        ("a\né👩‍💻#[|]#z\nc", "<A-down>", "a\nc\né👩‍💻#[|]#z"),
    ] {
        standard(input, keys, output).await?;
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn move_empty_line_to_and_from_eof() -> anyhow::Result<()> {
    for eol in ["\n", "\r\n"] {
        let middle = format!("a{eol}{eol}b");
        let end = format!("a{eol}b{eol}");
        let middle_caret = 1 + eol.len();
        let end_caret = end.len();
        for (input, pos, keys, output, expected_pos) in [
            (&middle, middle_caret, "<A-down>", &end, end_caret),
            (&end, end_caret, "<A-up>", &middle, middle_caret),
        ] {
            test_with_config(
                AppBuilder::new().with_config(config()),
                TestCase {
                    in_text: input.clone(),
                    in_selection: Selection::point(pos),
                    in_keys: keys.to_owned(),
                    out_text: output.clone(),
                    out_selection: Selection::point(expected_pos),
                    line_feed_handling: LineFeedHandling::AsIs,
                },
            )
            .await?;
        }
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn move_lines_uses_whole_source_lines_with_soft_wrap() -> anyhow::Result<()> {
    let mut config = config();
    config.editor.soft_wrap.enable = Some(true);
    let line = "x".repeat(200);
    test_with_config(
        AppBuilder::new().with_config(config),
        (
            format!("first\n{line}#[|]#\nlast"),
            "<A-down>",
            format!("first\nlast\n{line}#[|]#"),
            LineFeedHandling::AsIs,
        ),
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn move_selected_lines_keeps_selection_direction_and_extent() -> anyhow::Result<()> {
    for (input, keys, output) in [
        ("a\nb#[bb\ncc|]#c\ndddd", "<A-up>", "b#[bb\ncc|]#c\na\ndddd"),
        (
            "a\nb#[bb\ncc|]#c\ndddd",
            "<A-down>",
            "a\ndddd\nb#[bb\ncc|]#c",
        ),
        (
            "a\nb#[|bb\ncc]#c\ndddd",
            "<A-down>",
            "a\ndddd\nb#[|bb\ncc]#c",
        ),
        ("a\n#[bbb\n|]#cccc\nd", "<A-down>", "a\ncccc\n#[bbb\n|]#d"),
        ("a\n#[bbb\n|]#cccc\nd", "<A-up>", "#[bbb\n|]#a\ncccc\nd"),
        ("a\n#[|bbb\n]#cccc\nd", "<A-down>", "a\ncccc\n#[|bbb\n]#d"),
        ("a\n#[bbb\n|]#cccc", "<A-down>", "a\ncccc\n#[bbb|]#"),
        ("a\n#[bbb|]#", "<A-up>", "#[bbb|]#\na"),
        ("#[a\nb|]#\nc", "<A-up>", "#[a\nb|]#\nc"),
        ("a\n#[b\nc|]#", "<A-down>", "a\n#[b\nc|]#"),
        ("#[a\nb\nc|]#", "<A-down><A-up>", "#[a\nb\nc|]#"),
    ] {
        standard(input, keys, output).await?;
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn move_lines_handles_multiple_carets_and_overlapping_line_ranges() -> anyhow::Result<()> {
    for (input, keys, output) in [
        (
            "a\nb#[|]#b\nc\nd#(|)#d\ne",
            "<A-up>",
            "b#[|]#b\na\nd#(|)#d\nc\ne",
        ),
        (
            "a\nb#[|]#b\nc\nd#(|)#d\ne",
            "<A-down>",
            "a\nc\nb#[|]#b\ne\nd#(|)#d",
        ),
        (
            "a\nb#[|]#b\nc#(|)#c\nd",
            "<A-down>",
            "a\nd\nb#[|]#b\nc#(|)#c",
        ),
        ("a\nb#[|]#b #(|)#c\nd", "<A-up>", "b#[|]#b #(|)#c\na\nd"),
        ("a#[|]#\nb\nc#(|)#\nd", "<A-up>", "a#[|]#\nc#(|)#\nb\nd"),
    ] {
        standard(input, keys, output).await?;
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn move_lines_has_independent_undo_steps() -> anyhow::Result<()> {
    for (keys, output) in [
        ("<A-down><C-z>", "a\nb#[|]#\nc\nd"),
        ("<A-down><A-down><C-z>", "a\nc\nb#[|]#\nd"),
        ("<A-down><C-z><C-y>", "a\nc\nb#[|]#\nd"),
        ("x<A-down><C-z>", "a\nbx#[|]#\nc\nd"),
        ("x<A-down><C-z><C-z>", "a\nb#[|]#\nc\nd"),
    ] {
        standard("a\nb#[|]#\nc\nd", keys, output).await?;
    }
    standard("a\nb#[bb\nc|]#c\nd", "<A-down><C-z>", "a\nb#[bb\nc|]#c\nd").await
}

#[tokio::test(flavor = "multi_thread")]
async fn move_lines_preserves_mixed_line_endings() -> anyhow::Result<()> {
    for (input, keys, output) in [
        ("a\r\n#[b\n|]#c\r\nd", "<A-up>", "#[b\r\n|]#a\nc\r\nd"),
        ("a\r\n#[b\n|]#c\r\nd", "<A-down>", "a\r\nc\n#[b\r\n|]#d"),
        (
            "a\r\n#[b\n|]#c\r\nd",
            "<A-down><A-up>",
            "a\r\n#[b\n|]#c\r\nd",
        ),
    ] {
        test_with_config(
            AppBuilder::new().with_config(config()),
            (input, keys, output, LineFeedHandling::AsIs),
        )
        .await?;
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn terminal_paste_replaces_selection_in_one_undo_step() -> anyhow::Result<()> {
    #[cfg(windows)]
    use crossterm::event::Event;
    #[cfg(not(windows))]
    use termina::event::Event;
    use tokio_stream::wrappers::UnboundedReceiverStream;

    let mut app = AppBuilder::new()
        .with_config(config())
        .with_input_text("a#[bc|]#d")
        .build()?;
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    tx.send(Ok(Event::Paste("x\ny".into())))?;
    let mut events = UnboundedReceiverStream::new(rx);
    assert!(app.event_loop_until_idle(&mut events).await);
    let doc = helix_view::doc!(app.editor);
    let expected = format!("ax{}yd", doc.line_ending.as_str());
    assert_eq!(doc.text().to_string(), expected);
    test_key_sequence(
        &mut app,
        Some("<C-z>"),
        Some(&|app| {
            let (view, doc) = helix_view::current_ref!(app.editor);
            assert_eq!(doc.text().to_string(), "abcd");
            assert_eq!(doc.selection(view.id), &Selection::single(1, 3));
        }),
        false,
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn auto_pairs_and_mode_toggle() -> anyhow::Result<()> {
    let mut pairs = config();
    pairs.editor.auto_pairs = helix_core::syntax::config::AutoPairConfig::Enable(true);
    test_with_config(
        AppBuilder::new().with_config(pairs.clone()),
        ("#[|]#", "()<F6><F6>x", "()x#[|]#", LineFeedHandling::AsIs),
    )
    .await?;
    test_with_config(
        AppBuilder::new().with_config(pairs.clone()),
        ("#[|]#", "(<backspace>", "#[|]#", LineFeedHandling::AsIs),
    )
    .await?;
    test_with_config(
        AppBuilder::new().with_config(pairs),
        (
            "é👩‍💻#[|]# text",
            "(",
            "é👩‍💻(#[|]#) text",
            LineFeedHandling::AsIs,
        ),
    )
    .await?;
    standard("#[|]#", "<C-A-@>x", "@x#[|]#").await
}

#[test]
fn standard_configuration_round_trip() -> anyhow::Result<()> {
    let config: helix_view::editor::Config = toml::from_str("default-mode = 'standard'")?;
    assert_eq!(config.default_mode, DefaultMode::Standard);
    assert_eq!(
        config.cursor_shape.from_mode(Mode::Standard),
        helix_view::graphics::CursorKind::Bar
    );
    let serialized = toml::to_string(&config)?;
    assert_eq!(
        toml::from_str::<helix_view::editor::Config>(&serialized)?,
        config
    );
    assert!(toml::from_str::<helix_view::editor::Config>("default-mode = 'insert'").is_err());
    let key_config = "[keys.standard]\nC-s = ':write'".to_owned();
    let remapping = helix_term::config::Config::load(
        Ok(&key_config),
        Err(helix_term::config::ConfigLoadError::Error(
            std::io::Error::from(std::io::ErrorKind::NotFound),
        )),
    )
    .unwrap();
    assert!(remapping.keys.contains_key(&Mode::Standard));
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn open_and_save_named_file() -> anyhow::Result<()> {
    let mut file = temp_file_with_contents("hello")?;
    let mut config = config();
    config.editor.insert_final_newline = false;
    let mut app = AppBuilder::new()
        .with_config(config)
        .with_file(file.path(), None)
        .build()?;
    let (view, doc) = helix_view::current_ref!(app.editor);
    assert_eq!(doc.selection(view.id), &Selection::point(0));
    test_key_sequence(
        &mut app,
        Some("x<C-s>"),
        Some(&|app| {
            assert_eq!(app.editor.mode(), Mode::Standard);
            assert_status_not_error(&app.editor);
        }),
        false,
    )
    .await?;
    assert_file_has_content(&mut file, "xhello")
}

#[tokio::test(flavor = "multi_thread")]
async fn quit_checks_unsaved_changes() -> anyhow::Result<()> {
    let mut app = AppBuilder::new().with_config(config()).build()?;
    test_key_sequence(&mut app, Some("<C-q>"), None, true).await?;
    let mut app = AppBuilder::new().with_config(config()).build()?;
    test_key_sequence(
        &mut app,
        Some("x<C-q>"),
        Some(&|app| {
            assert!(!app.editor.should_close());
            assert!(app.editor.is_err());
            assert_eq!(app.editor.mode(), Mode::Standard);
        }),
        false,
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn split_shortcuts_keep_standard_mode_and_independent_files() -> anyhow::Result<()> {
    let first = temp_file_with_contents("first")?;
    let second = temp_file_with_contents("second")?;
    let mut app = AppBuilder::new()
        .with_config(config())
        .with_file(first.path(), None)
        .build()?;
    let first_view = app.editor.tree.focus;
    test_key_sequence(
        &mut app,
        Some(&format!(
            "<C-\\><F1>open {}<ret><C-F6>x<C-S-F6>y",
            second.path().display()
        )),
        Some(&|app| {
            assert_eq!(app.editor.mode(), Mode::Standard);
            assert_eq!(app.editor.tree.views().count(), 2);
            assert_ne!(app.editor.tree.focus, first_view);
            let docs: Vec<_> = app
                .editor
                .documents()
                .map(|doc| doc.text().to_string())
                .collect();
            assert!(docs.contains(&"xfirst".to_owned()));
            assert!(docs.contains(&"ysecond".to_owned()));
            assert_status_not_error(&app.editor);
        }),
        false,
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn command_prompt_opens_vertical_and_horizontal_splits() -> anyhow::Result<()> {
    for command in ["vsplit", "hsplit"] {
        let file = temp_file_with_contents("second")?;
        let mut app = AppBuilder::new().with_config(config()).build()?;
        test_key_sequence(
            &mut app,
            Some(&format!("<F1>{command} {}<ret>x", file.path().display())),
            Some(&|app| {
                assert_eq!(app.editor.mode(), Mode::Standard);
                assert_eq!(app.editor.tree.views().count(), 2);
                assert_eq!(helix_view::doc!(app.editor).text().to_string(), "xsecond");
                assert_status_not_error(&app.editor);
            }),
            false,
        )
        .await?;
    }
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn cancelled_prefix_keeps_navigation() -> anyhow::Result<()> {
    let bindings = "[keys.standard]\nj = { k = 'standard_escape' }".to_owned();
    let remapped = Config::load(
        Ok(&bindings),
        Err(helix_term::config::ConfigLoadError::Error(
            std::io::ErrorKind::NotFound.into(),
        )),
    )
    .unwrap();
    let mut config = config();
    config.keys = remapped.keys;
    test_with_config(
        AppBuilder::new().with_config(config),
        ("#[|]#", "j<left>x", "x#[|]#j", LineFeedHandling::AsIs),
    )
    .await
}
