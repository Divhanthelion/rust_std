//! Cutting named regions out of source text.
//!
//! Lesson files mark regions like this:
//!
//! ```text
//! // ANCHOR: greet
//! fn greet() { println!("hi"); }
//! // ANCHOR_END: greet
//! ```
//!
//! Regions may nest or overlap; marker lines for *other* regions are
//! dropped from the output so learners never see them.

/// Returns the text between `// ANCHOR: name` and `// ANCHOR_END: name`,
/// with other anchor markers removed and common indentation stripped.
///
/// ```
/// let src = "fn main() {\n    // ANCHOR: body\n    let x = 1;\n    // ANCHOR_END: body\n}\n";
/// assert_eq!(rust_std::snippet::extract(src, "body").as_deref(), Some("let x = 1;"));
/// assert_eq!(rust_std::snippet::extract(src, "missing"), None);
/// ```
pub fn extract(source: &str, name: &str) -> Option<String> {
    let mut lines = source.lines();
    // Advance the iterator past the opening marker. `find` consumes items
    // up to and including the match, which is exactly what we want.
    lines.find(|line| marker(line, "ANCHOR:") == Some(name))?;

    let mut body = Vec::new();
    for line in lines {
        if marker(line, "ANCHOR_END:") == Some(name) {
            return Some(dedent(&body.join("\n")));
        }
        let is_other_marker =
            marker(line, "ANCHOR:").is_some() || marker(line, "ANCHOR_END:").is_some();
        if !is_other_marker {
            body.push(line);
        }
    }
    None // opening marker without a closing one
}

/// Names of every anchor region opened in `source`, in order.
pub fn anchors(source: &str) -> Vec<&str> {
    source
        .lines()
        .filter_map(|line| marker(line, "ANCHOR:"))
        .collect()
}

/// If `line` is `// <kind> <name>` (ignoring surrounding whitespace),
/// returns `name`.
fn marker<'a>(line: &'a str, kind: &str) -> Option<&'a str> {
    let rest = line.trim().strip_prefix("//")?.trim_start();
    let name = rest.strip_prefix(kind)?.trim();
    (!name.is_empty()).then_some(name)
}

/// Removes the longest common leading whitespace from every non-blank line,
/// and drops leading/trailing blank lines.
///
/// ```
/// assert_eq!(rust_std::snippet::dedent("\n    a\n      b\n"), "a\n  b");
/// ```
pub fn dedent(text: &str) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let first = lines.iter().position(|l| !l.trim().is_empty());
    let last = lines.iter().rposition(|l| !l.trim().is_empty());
    let (Some(first), Some(last)) = (first, last) else {
        return String::new();
    };
    let lines = &lines[first..=last];

    let indent = lines
        .iter()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.len() - l.trim_start().len())
        .min()
        .unwrap_or(0);

    lines
        .iter()
        // `get` returns None instead of panicking if `indent` would split a
        // multi-byte character; blank lines shorter than `indent` also land there.
        .map(|l| l.get(indent..).unwrap_or_else(|| l.trim_start()))
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    const SRC: &str = "
        // ANCHOR: outer
        struct A;
        // ANCHOR: inner
        impl A {
            fn f(&self) {}
        }
        // ANCHOR_END: inner
        // ANCHOR_END: outer
    ";

    #[test]
    fn nested_markers_are_hidden() {
        let outer = extract(SRC, "outer").unwrap();
        assert!(!outer.contains("ANCHOR"));
        assert!(outer.starts_with("struct A;"));
        assert!(outer.ends_with('}'));
    }

    #[test]
    fn inner_region_is_dedented() {
        assert_eq!(
            extract(SRC, "inner").unwrap(),
            "impl A {\n    fn f(&self) {}\n}"
        );
    }

    #[test]
    fn unterminated_region_is_none() {
        assert_eq!(extract("// ANCHOR: x\nlet a = 1;", "x"), None);
    }

    #[test]
    fn lists_anchor_names() {
        assert_eq!(anchors(SRC), vec!["outer", "inner"]);
    }
}
