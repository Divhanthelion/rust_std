//! Terminal output: colors, word wrapping, a tiny Markdown renderer and a
//! small Rust syntax highlighter — all with nothing but `std`.

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

// Global settings. Atomics let us mutate them from anywhere without
// `unsafe` (`static mut`) and without passing a config struct everywhere.
static COLOR: AtomicBool = AtomicBool::new(false);
static WIDTH: AtomicUsize = AtomicUsize::new(80);

pub fn set_color(on: bool) {
    COLOR.store(on, Ordering::Relaxed);
}

pub fn color_enabled() -> bool {
    COLOR.load(Ordering::Relaxed)
}

pub fn set_width(columns: usize) {
    WIDTH.store(columns.clamp(40, 120), Ordering::Relaxed);
}

/// Width used for wrapping prose.
pub fn width() -> usize {
    WIDTH.load(Ordering::Relaxed)
}

/// Visual styles, mapped to ANSI SGR escape codes.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Style {
    Title,
    Heading,
    Bold,
    Dim,
    Inline,
    Keyword,
    Str,
    Comment,
    Number,
    Macro,
    Type,
    Lifetime,
    Good,
    Bad,
    Accent,
}

impl Style {
    fn sgr(self) -> &'static str {
        match self {
            Style::Title => "1;36",
            Style::Heading => "1;33",
            Style::Bold => "1",
            Style::Dim | Style::Comment => "90",
            Style::Inline | Style::Macro => "36",
            Style::Keyword => "35",
            Style::Str => "32",
            Style::Number | Style::Lifetime => "33",
            Style::Type => "94",
            Style::Good => "1;32",
            Style::Bad => "1;31",
            Style::Accent => "1;35",
        }
    }
}

/// Wraps `text` in the escape codes for `style` (if color is on).
pub fn paint(text: &str, style: Style) -> String {
    if color_enabled() && !text.is_empty() {
        format!("\x1b[{}m{}\x1b[0m", style.sgr(), text)
    } else {
        text.to_string()
    }
}

/// A horizontal line across the terminal.
pub fn rule() -> String {
    paint(&"─".repeat(width()), Style::Dim)
}

/// A banner for a lesson title.
pub fn title(number: usize, text: &str) -> String {
    let label = format!(" Lesson {number}: {text} ");
    let bar = "═".repeat(width().saturating_sub(label.chars().count() + 2));
    paint(&format!("══{label}{bar}"), Style::Title)
}

pub fn section_heading(text: &str) -> String {
    paint(&format!("▌ {text}"), Style::Heading)
}

// ─── Prose ────────────────────────────────────────────────────────────────

/// Renders lesson prose. The dialect is deliberately small:
///
/// - blank lines separate blocks
/// - a block whose lines start with `- ` is a bullet list
/// - a block whose lines start with `> ` is a callout
/// - ```` ```info ```` ... ```` ``` ```` is a fenced block; `info` may be
///   `rust`, `text`, or `compile_fail,E0xxx`
/// - inside text, `` `code` `` and `**bold**` are highlighted
pub fn render_prose(text: &str) -> String {
    let text = crate::snippet::dedent(text);
    let mut out = String::new();
    let mut lines = text.lines().peekable();

    while let Some(line) = lines.next() {
        if line.trim().is_empty() {
            continue;
        }

        if let Some(info) = line.trim_start().strip_prefix("```") {
            let mut code = Vec::new();
            for inner in lines.by_ref() {
                if inner.trim_start().starts_with("```") {
                    break;
                }
                code.push(inner);
            }
            out.push_str(&render_fenced(info.trim(), &code.join("\n")));
            out.push('\n');
            continue;
        }

        // Gather the rest of this block: consecutive non-blank lines that
        // don't start a fenced block.
        let mut block = vec![line];
        while let Some(next) = lines.peek() {
            if next.trim().is_empty() || next.trim_start().starts_with("```") {
                break;
            }
            block.push(lines.next().unwrap());
        }

        if block[0].starts_with("- ") {
            for item in split_items(&block, "- ") {
                out.push_str(&wrap(&item, "  • ", "    "));
            }
        } else if block[0].starts_with("> ") {
            let joined: Vec<&str> = block
                .iter()
                .map(|l| l.strip_prefix('>').unwrap_or(l).trim())
                .collect();
            let bar = paint("┃ ", Style::Accent);
            out.push_str(&wrap(&joined.join(" "), &bar, &bar));
        } else {
            let joined: Vec<&str> = block.iter().map(|l| l.trim()).collect();
            out.push_str(&wrap(&joined.join(" "), "", ""));
        }
        out.push('\n');
    }
    out
}

/// Splits a list block into items. Lines that don't start with `marker`
/// continue the previous item.
fn split_items(block: &[&str], marker: &str) -> Vec<String> {
    let mut items: Vec<String> = Vec::new();
    for line in block {
        match line.strip_prefix(marker) {
            Some(start) => items.push(start.trim().to_string()),
            None => {
                if let Some(last) = items.last_mut() {
                    last.push(' ');
                    last.push_str(line.trim());
                }
            }
        }
    }
    items
}

/// Rustdoc convention: in Rust blocks, lines starting with `# ` are compiled
/// (by the prose tests) but hidden from the reader.
pub fn is_hidden_line(line: &str) -> bool {
    let t = line.trim_start();
    t == "#" || t.starts_with("# ")
}

fn render_fenced(info: &str, code: &str) -> String {
    let kind = info.split(',').next().unwrap_or("");
    if matches!(kind, "text" | "output") {
        return code_block(code, false);
    }
    let visible: Vec<&str> = code.lines().filter(|l| !is_hidden_line(l)).collect();
    let code = &visible.join("\n");
    match kind {
        "compile_fail" => {
            let codes: Vec<&str> = info
                .split(',')
                .map(str::trim)
                .filter(|c| c.starts_with('E'))
                .collect();
            let label = if codes.is_empty() {
                "✗ does not compile".to_string()
            } else {
                format!("✗ does not compile — error {}", codes.join(", "))
            };
            format!("{}\n{}", paint(&label, Style::Bad), code_block(code, true))
        }
        _ => code_block(code, true),
    }
}

/// Word-wraps `text` to the terminal width. `first` prefixes the first line,
/// `rest` prefixes continuation lines. Inline markup is styled per word.
pub fn wrap(text: &str, first: &str, rest: &str) -> String {
    let limit = width().min(100);
    let prefix_width = visible_width(first).max(visible_width(rest));
    let mut out = String::from(first);
    let mut line_width = prefix_width;
    let mut state = Markup::default();

    for (i, word) in units(text).iter().enumerate() {
        let (styled, w) = style_word(word, &mut state);
        if i > 0 {
            if line_width + 1 + w > limit {
                out.push('\n');
                out.push_str(rest);
                line_width = prefix_width;
            } else {
                out.push(' ');
                line_width += 1;
            }
        }
        out.push_str(&styled);
        line_width += w;
    }
    out.push('\n');
    out
}

/// Splits text into wrap units: words, except that a `code span` containing
/// spaces stays together so it is never broken across lines.
fn units(text: &str) -> Vec<String> {
    let mut units: Vec<String> = Vec::new();
    let mut open = false; // inside an unclosed code span?
    for word in text.split_whitespace() {
        match units.last_mut() {
            Some(last) if open => {
                last.push(' ');
                last.push_str(word);
            }
            _ => units.push(word.to_string()),
        }
        if word.matches('`').count() % 2 == 1 {
            open = !open;
        }
    }
    units
}

/// Inline-markup state carried from word to word, so a code span that
/// contains spaces (`` `let x = 5;` ``) is styled across all its words.
#[derive(Default)]
struct Markup {
    code: bool,
    bold: bool,
}

/// Styles one word; returns the styled text and its visible width.
fn style_word(word: &str, state: &mut Markup) -> (String, usize) {
    let mut out = String::new();
    let mut run = String::new();
    let mut width = 0;
    let mut chars = word.chars().peekable();

    let flush = |run: &mut String, out: &mut String, state: &Markup| {
        let style = if state.code {
            Some(Style::Inline)
        } else if state.bold {
            Some(Style::Bold)
        } else {
            None
        };
        match style {
            Some(s) => out.push_str(&paint(run, s)),
            None => out.push_str(run),
        }
        run.clear();
    };

    while let Some(c) = chars.next() {
        if c == '`' {
            flush(&mut run, &mut out, state);
            state.code = !state.code;
        } else if c == '*' && !state.code && chars.peek() == Some(&'*') {
            chars.next();
            flush(&mut run, &mut out, state);
            state.bold = !state.bold;
        } else {
            run.push(c);
            width += 1;
        }
    }
    flush(&mut run, &mut out, state);
    (out, width)
}

/// Number of columns `s` occupies, ignoring ANSI escape sequences.
pub fn visible_width(s: &str) -> usize {
    let mut width = 0;
    let mut in_escape = false;
    for c in s.chars() {
        match (in_escape, c) {
            (false, '\x1b') => in_escape = true,
            (true, 'm') => in_escape = false,
            (true, _) => {}
            (false, _) => width += 1,
        }
    }
    width
}

// ─── Code ─────────────────────────────────────────────────────────────────

/// Renders code with a left gutter, optionally syntax highlighted.
pub fn code_block(code: &str, highlight: bool) -> String {
    let body = if highlight {
        highlight_rust(code)
    } else {
        code.to_string()
    };
    let gutter = paint("  │ ", Style::Dim);
    let mut out = String::new();
    for line in body.lines() {
        out.push_str(&gutter);
        out.push_str(line);
        out.push('\n');
    }
    out
}

const KEYWORDS: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern",
    "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub",
    "ref", "return", "self", "Self", "static", "struct", "super", "trait", "true", "type",
    "unsafe", "use", "where", "while", "union",
];

const PRIMITIVES: &[&str] = &[
    "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize", "f32",
    "f64", "bool", "char", "str",
];

/// A deliberately small Rust highlighter. It understands comments, strings
/// (including raw and byte strings), char literals vs. lifetimes, numbers,
/// keywords, macros and capitalised type names. That covers what lessons show.
pub fn highlight_rust(code: &str) -> String {
    if !color_enabled() {
        return code.to_string();
    }
    let chars: Vec<char> = code.chars().collect();
    let mut out = String::with_capacity(code.len() * 2);
    let mut i = 0;

    // Paints chars[start..end], splitting at newlines so that every line is
    // self-contained (the gutter is added between lines later).
    let emit = |out: &mut String, start: usize, end: usize, style: Option<Style>| {
        let text: String = chars[start..end].iter().collect();
        match style {
            None => out.push_str(&text),
            Some(style) => {
                let parts: Vec<String> = text.split('\n').map(|p| paint(p, style)).collect();
                out.push_str(&parts.join("\n"));
            }
        }
    };

    while i < chars.len() {
        let c = chars[i];
        let next = chars.get(i + 1).copied();
        let start = i;

        if c == '/' && next == Some('/') {
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
            emit(&mut out, start, i, Some(Style::Comment));
        } else if c == '/' && next == Some('*') {
            i += 2;
            while i < chars.len() && !(chars[i] == '*' && chars.get(i + 1) == Some(&'/')) {
                i += 1;
            }
            i = (i + 2).min(chars.len());
            emit(&mut out, start, i, Some(Style::Comment));
        } else if let Some(end) = raw_string_end(&chars, i) {
            i = end;
            emit(&mut out, start, i, Some(Style::Str));
        } else if c == '"' || (c == 'b' && next == Some('"')) {
            i += if c == 'b' { 2 } else { 1 };
            while i < chars.len() && chars[i] != '"' {
                i += if chars[i] == '\\' { 2 } else { 1 };
            }
            i = (i + 1).min(chars.len());
            emit(&mut out, start, i, Some(Style::Str));
        } else if c == '\'' || (c == 'b' && next == Some('\'')) {
            let quote = if c == 'b' { i + 1 } else { i };
            match char_literal_end(&chars, quote) {
                Some(end) => {
                    i = end;
                    emit(&mut out, start, i, Some(Style::Str));
                }
                None => {
                    // A lifetime or loop label: 'a, 'static, 'outer
                    i = quote + 1;
                    while i < chars.len() && is_ident_char(chars[i]) {
                        i += 1;
                    }
                    let style = if c == 'b' {
                        None
                    } else {
                        Some(Style::Lifetime)
                    };
                    emit(&mut out, start, i, style);
                }
            }
        } else if c.is_ascii_digit() {
            while i < chars.len() {
                let d = chars[i];
                let decimal_point =
                    d == '.' && chars.get(i + 1).is_some_and(|n| n.is_ascii_digit());
                if is_ident_char(d) || decimal_point {
                    i += 1;
                } else {
                    break;
                }
            }
            emit(&mut out, start, i, Some(Style::Number));
        } else if is_ident_start(c) {
            while i < chars.len() && is_ident_char(chars[i]) {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            let is_macro = chars.get(i) == Some(&'!') && chars.get(i + 1) != Some(&'=');
            let style = if is_macro {
                i += 1;
                Some(Style::Macro)
            } else if KEYWORDS.contains(&word.as_str()) {
                Some(Style::Keyword)
            } else if PRIMITIVES.contains(&word.as_str()) || c.is_uppercase() {
                Some(Style::Type)
            } else {
                None
            };
            emit(&mut out, start, i, style);
        } else {
            i += 1;
            emit(&mut out, start, i, None);
        }
    }
    out
}

fn is_ident_start(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}

fn is_ident_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// If a raw string (`r"…"`, `r#"…"#`, `br"…"`) starts at `i`, returns the
/// index just past its end.
fn raw_string_end(chars: &[char], i: usize) -> Option<usize> {
    let mut j = i;
    if chars.get(j) == Some(&'b') {
        j += 1;
    }
    if chars.get(j) != Some(&'r') {
        return None;
    }
    // `r` must not be the tail of an identifier like `var"`.
    if i > 0 && is_ident_char(chars[i - 1]) {
        return None;
    }
    j += 1;
    let mut hashes = 0;
    while chars.get(j) == Some(&'#') {
        hashes += 1;
        j += 1;
    }
    if chars.get(j) != Some(&'"') {
        return None;
    }
    j += 1;
    while j < chars.len() {
        if chars[j] == '"' && (1..=hashes).all(|k| chars.get(j + k) == Some(&'#')) {
            return Some(j + 1 + hashes);
        }
        j += 1;
    }
    Some(chars.len())
}

/// If a char literal starts at the quote at `i`, returns the index past it.
/// Returns `None` for lifetimes and labels.
fn char_literal_end(chars: &[char], i: usize) -> Option<usize> {
    match chars.get(i + 1)? {
        '\\' => {
            // Escape: '\n', '\'', '\u{1F600}'
            let mut j = i + 3;
            while j < chars.len() && chars[j] != '\'' {
                j += 1;
            }
            Some((j + 1).min(chars.len()))
        }
        _ if chars.get(i + 2) == Some(&'\'') => Some(i + 3),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visible_width_skips_escapes() {
        assert_eq!(visible_width("\x1b[1;36mhello\x1b[0m"), 5);
    }

    #[test]
    fn style_word_strips_markup() {
        let mut state = Markup::default();
        let (text, width) = style_word("`Vec<T>`,", &mut state);
        assert_eq!(width, "Vec<T>,".len());
        assert!(!state.code);
        // Without color, styling is just markup removal.
        if !color_enabled() {
            assert_eq!(text, "Vec<T>,");
        }
    }

    #[test]
    fn code_spans_stay_together() {
        assert_eq!(
            units("run `cargo test -q` now"),
            ["run", "`cargo test -q`", "now"]
        );
        assert_eq!(units("a `b` c"), ["a", "`b`", "c"]);
    }

    #[test]
    fn char_literals_and_lifetimes() {
        let chars: Vec<char> = "'a' 'static '\\n'".chars().collect();
        assert_eq!(char_literal_end(&chars, 0), Some(3));
        assert_eq!(char_literal_end(&chars, 4), None);
        assert_eq!(char_literal_end(&chars, 12), Some(16));
    }

    #[test]
    fn raw_strings() {
        let chars: Vec<char> = r##"r#"a "quoted" b"# x"##.chars().collect();
        assert_eq!(raw_string_end(&chars, 0), Some(17));
    }
}
