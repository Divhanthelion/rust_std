//! Lesson: Strings & UTF-8.

use std::borrow::Cow;
use std::fmt::Write as _;

use crate::lesson::{Lesson, Question, Section};

pub static LESSON: Lesson = Lesson {
    id: "strings",
    title: "Strings & UTF-8",
    summary: "String vs &str, UTF-8 and why you can't index by position, iteration, searching, splitting, building, and the other string types.",
    source: include_str!("strings.rs"),
    sections: &[
        Section::new(
            "String and &str",
            r#"
            Rust has two main string types, and they mirror `Vec<T>` and `&[T]`:

            ```text
            String        owned, growable, heap-allocated UTF-8 (pointer, len, capacity)
            &str          a borrowed view of UTF-8 bytes somewhere (pointer, len)
            &'static str  a &str that lives forever, e.g. a literal baked into the binary
            ```

            String literals like `"hello"` are `&'static str`. Turn a `&str`
            into an owned `String` with `String::from`, `.to_string()`, or
            `.to_owned()`; borrow a `String` as `&str` with `&s`, `&s[..]`, or
            `s.as_str()`.

            Both are **guaranteed valid UTF-8**. Every API that could break
            that, such as building a string from arbitrary bytes, either checks
            and returns a `Result`, or is `unsafe`.
            "#,
        )
        .demo("two_types", two_types),
        Section::new(
            "Building strings",
            r#"
            A `String` grows like a `Vec`: `push(char)`, `push_str(&str)`,
            `insert`, `extend`. For anything non-trivial, `format!` or
            `write!` is the clearest.

            The `+` operator takes ownership of its left side and borrows its
            right: `s1 + &s2` reuses `s1`'s buffer. It reads oddly when chained,
            which is one reason `format!` is preferred. When building a string
            in a loop, `String::with_capacity` avoids repeated reallocation.
            "#,
        )
        .demo("building", building),
        Section::new(
            "UTF-8: why s[0] doesn't compile",
            r#"
            UTF-8 stores each `char` in 1 to 4 bytes. ASCII takes 1, "é" takes
            2, and the katakana in "トヨタ" take 3 each. So "the character at
            position 3" can't be found in constant time, and "the byte at
            position 3" might be the middle of a character. Rust refuses to
            pick an answer for you:

            ```compile_fail,E0277
            let s = String::from("トヨタ");
            let c = s[0]; // error: the type `str` cannot be indexed by `{integer}`
            ```

            `len()` counts **bytes**. Range slicing like `&s[0..3]` works on
            byte offsets and **panics** if an offset isn't on a character
            boundary. The non-panicking alternative, `s.get(0..3)`, returns an
            `Option<&str>`. To get characters, iterate with `chars()`.
            "#,
        )
        .demo("utf8", utf8),
        Section::new(
            "Iterating: bytes, chars and char_indices",
            r#"
            Pick the view that matches what you mean:

            - `bytes()` gives the raw `u8`s, the right view for protocols and
              ASCII.
            - `chars()` gives Unicode scalar values (`char`).
            - `char_indices()` gives `(byte_offset, char)` pairs, which is how
              you find safe slice positions.

            Grapheme clusters (what a reader calls "a character", like a flag
            emoji or "e" plus a combining accent) are not in std. That's one of
            the few things that genuinely needs a crate.
            "#,
        )
        .demo("iterating", iterating),
        Section::new(
            "Searching, splitting and trimming",
            r#"
            Most string methods borrow and return `&str` slices of the
            original, so no copies are made. `split`, `split_whitespace`,
            `lines` and `matches` return **iterators** that are lazy and
            allocation-free; collect them only if you need a `Vec`.
            `split_once` and `strip_prefix` return `Option`s, so a parser that
            uses them can't panic.

            Patterns can be a `&str`, a `char`, a slice of chars, or a closure
            `|c: char| -> bool`.
            "#,
        )
        .demo("searching", searching),
        Section::new(
            "Case, comparison and conversion",
            r#"
            `to_uppercase`/`to_lowercase` are Unicode-aware and allocate a new
            `String`; the `_ascii_` variants are faster and only touch ASCII.
            Strings compare lexicographically **by bytes**, which isn't the
            same as dictionary order across languages. `parse::<T>()` converts
            text to any type implementing `FromStr`, and `to_string()` converts
            any `Display` type to text.
            "#,
        )
        .demo("case", case_and_conversion),
        Section::new(
            "Take &str, return String",
            r#"
            For parameters, take `&str`. Through **deref coercion**, a
            `&String` automatically becomes a `&str`, so one function serves
            both literals and owned strings. Return `String` when you create
            new text, and `&str` when you return part of an input.

            If a function *sometimes* needs to allocate, for example escaping
            text only when it contains special characters, `Cow<str>` ("clone
            on write") returns either a borrowed `&str` or an owned `String`
            behind one type.
            "#,
        )
        .demo("params", parameters),
        Section::new(
            "Raw, byte and C string literals",
            r##"
            - `r"C:\path"` is a **raw string**: backslashes aren't escapes.
              Add hashes to include quotes: `r#"say "hi""#`.
            - `b"bytes"` is a `&[u8; N]`, so it isn't UTF-8 checked. `b'x'` is a
              single `u8`.
            - `c"text"` (Rust 1.77+) is a `&CStr` with a trailing NUL, ready for
              C APIs (Lesson 36).
            - Multi-line literals keep their newlines and indentation, unless
              you end a line with `\`.
            "##,
        )
        .demo("literals", literals),
        Section::new(
            "The other string types",
            r#"
            Each string type exists because some boundary has different rules:

            ```text
            String / &str       UTF-8, Rust's own text
            OsString / &OsStr   whatever the OS uses (may not be UTF-8)
            PathBuf / &Path     file paths, built on OsString
            CString / &CStr     NUL-terminated, for C FFI
            Vec<u8> / &[u8]     arbitrary bytes, no promises at all
            Box<str>            an immutable String with no spare capacity
            Cow<'_, str>        borrowed or owned, decided at run time
            ```

            Converting toward UTF-8 can fail, and the API makes you handle
            it: `to_str()` returns an `Option`, `String::from_utf8` returns a
            `Result`, and `to_string_lossy()` substitutes `�` for invalid
            bytes.
            "#,
        )
        .demo("others", other_types),
    ],
    quiz: &[
        Question::new(
            "What is `\"トヨタ\".len()`?",
            &["3", "6", "9", "12"],
            2,
            "`len()` counts bytes. Each of these katakana takes 3 bytes in UTF-8. `chars().count()` would give 3.",
        ),
        Question::new(
            "What happens on `&\"héllo\"[0..2]`?",
            &[
                "Returns \"hé\"",
                "Returns \"h\"",
                "Panics: byte 2 is in the middle of 'é'",
                "Compile error",
            ],
            2,
            "'é' occupies bytes 1..3, so 2 is not a char boundary and slicing panics. Use `get(0..2)` for an `Option`.",
        ),
        Question::new(
            "Which parameter type accepts both `\"literal\"` and `&my_string`?",
            &["String", "&String", "&str", "&mut str"],
            2,
            "`&String` derefs to `&str` automatically. `&String` parameters would reject literals.",
        ),
        Question::new(
            "What does `s1 + &s2` do with `s1: String`?",
            &[
                "Copies both into a new String",
                "Moves s1, appends s2 into its buffer, returns it",
                "Borrows both",
                "Doesn't compile",
            ],
            1,
            "`Add for String` takes `self` by value and `&str` on the right, reusing the left buffer. `s1` can't be used afterwards.",
        ),
        Question::new(
            "Which type should hold a file name from the operating system?",
            &["String", "&'static str", "OsString / PathBuf", "Vec<char>"],
            2,
            "File names aren't guaranteed to be UTF-8 on every OS. `OsString` and `PathBuf` represent them faithfully.",
        ),
    ],
    exercises: &[
        "Write `fn initials(full_name: &str) -> String` that returns the first char of each word, uppercased: \"ada lovelace\" → \"AL\". Make it work for \"Émile zola\".",
        "Write `fn truncate_chars(s: &str, max: usize) -> &str` that never panics and never cuts a character in half. (Hint: `char_indices().nth(max)`.)",
        "Parse lines like `rpm=3200; temp=88.5; gear=D` into (key, value) pairs using `split(';')`, `split_once('=')` and `trim`.",
        "Write `fn escape_html(s: &str) -> Cow<'_, str>` that only allocates when the input contains `<`, `>` or `&`.",
    ],
};

fn two_types() {
    // ANCHOR: two_types
    let literal: &'static str = "Prius"; // baked into the binary
    let owned: String = String::from(literal); // a heap copy we own
    let also_owned = "Mirai".to_string();
    let view: &str = &owned; // borrow a String as &str
    let whole: &str = also_owned.as_str();
    println!("{literal} {owned} {view} {whole}");

    use std::mem::size_of;
    println!(
        "size_of::<&str>()   = {} (pointer + length)",
        size_of::<&str>()
    );
    println!(
        "size_of::<String>() = {} (pointer + length + capacity)",
        size_of::<String>()
    );
    // ANCHOR_END: two_types
}

fn building() {
    // ANCHOR: building
    let mut s = String::new();
    s.push_str("Hybrid");
    s.push(' ');
    s.push_str("Synergy");
    s.insert(0, '[');
    s.push(']');
    println!("{s}");

    let a = String::from("Toyota");
    let b = String::from("Way");
    let joined = a + " " + &b; // `a` is moved; `b` is borrowed
    println!("{joined}  (b is still usable: {b})");

    let formatted = format!("{}-{}-{:03}", "ECU", "brake", 7);
    println!("{formatted}");

    let mut report = String::with_capacity(64);
    for (i, temp) in [88.5, 90.25, 91.0].iter().enumerate() {
        // write! appends formatted text without temporary Strings
        write!(report, "{}{temp}", if i > 0 { ", " } else { "" }).unwrap();
    }
    println!(
        "{report}  (len {}, capacity {})",
        report.len(),
        report.capacity()
    );

    let words = ["just", "in", "time"];
    println!("{}", words.join(" "));
    println!("{}", words.concat());
    let from_chars: String = ['k', 'a', 'i', 'z', 'e', 'n'].iter().collect();
    println!("{from_chars}");
    // ANCHOR_END: building
}

fn utf8() {
    // ANCHOR: utf8
    let name = "トヨタ";
    println!(
        "{name}: len() = {} bytes, chars().count() = {}",
        name.len(),
        name.chars().count()
    );
    println!("bytes: {:02x?}", name.as_bytes());

    let s = "héllo";
    println!("&s[0..1] = {:?}", &s[0..1]); // 'h' is 1 byte: fine
    println!("s.get(0..2) = {:?}", s.get(0..2)); // mid-'é': None instead of a panic
    println!("s.get(0..3) = {:?}", s.get(0..3)); // "hé"
    println!("is_char_boundary(2)? {}", s.is_char_boundary(2));
    println!("third char: {:?}", s.chars().nth(2)); // O(n) walk, but correct
    // ANCHOR_END: utf8
}

fn iterating() {
    // ANCHOR: iterating
    let s = "aé🦀";
    for b in s.bytes() {
        print!("{b:#04x} ");
    }
    println!("← bytes");
    for c in s.chars() {
        print!("{c:?} ");
    }
    println!("← chars");
    for (i, c) in s.char_indices() {
        print!("{i}:{c} ");
    }
    println!("← char_indices (byte offsets)");

    let reversed: String = "stressed".chars().rev().collect();
    println!("reversed: {reversed}");
    // ANCHOR_END: iterating
}

fn searching() {
    // ANCHOR: searching
    let line = "  DTC P0301: cylinder 1 misfire detected  ";
    let t = line.trim();
    println!("trimmed: {t:?}");
    println!(
        "starts_with DTC? {}  contains misfire? {}",
        t.starts_with("DTC"),
        t.contains("misfire")
    );
    println!(
        "find ':' → {:?}   rfind ' ' → {:?}",
        t.find(':'),
        t.rfind(' ')
    );

    if let Some((code, description)) = t.split_once(": ") {
        println!("code {code:?}, description {description:?}");
        let code_number = code.strip_prefix("DTC P").unwrap_or("?");
        println!("code number: {code_number}");
    }

    let csv = "rpm,3200,,temp,88";
    let fields: Vec<&str> = csv.split(',').collect(); // keeps empty fields
    println!("split(',') → {fields:?}");
    let words: Vec<&str> = "  many   spaces here ".split_whitespace().collect();
    println!("split_whitespace → {words:?}");
    let parts: Vec<&str> = "a=b=c".splitn(2, '=').collect();
    println!("splitn(2) → {parts:?}");
    let digits: String = "v2.4.1-rc3".chars().filter(char::is_ascii_digit).collect();
    println!("only digits: {digits}");
    println!(
        "split on closure: {:?}",
        "one1two22three"
            .split(|c: char| c.is_ascii_digit())
            .collect::<Vec<_>>()
    );
    println!("count of 'e': {}", "excellence".matches('e').count());
    println!("replace: {}", "red light, red car".replace("red", "green"));

    for (n, l) in "first\nsecond\r\nthird".lines().enumerate() {
        println!("line {n}: {l:?}"); // \r\n handled too
    }
    // ANCHOR_END: searching
}

fn case_and_conversion() {
    // ANCHOR: case
    println!("{} {}", "Straße".to_uppercase(), "ÉCOLE".to_lowercase());
    println!("ascii only: {}", "Straße".to_ascii_uppercase());
    println!(
        "eq_ignore_ascii_case: {}",
        "GEAR".eq_ignore_ascii_case("gear")
    );

    println!("\"apple\" < \"banana\": {}", "apple" < "banana");
    println!(
        "\"Zebra\" < \"apple\": {} (uppercase bytes sort first)",
        "Zebra" < "apple"
    );

    let n: i32 = "-42".parse().unwrap();
    let f: f64 = "3.5".parse().unwrap();
    let b: bool = "true".parse().unwrap();
    let c: char = "x".parse().unwrap();
    println!("parsed: {n} {f} {b} {c}");
    println!(
        "to_string: {} {} {}",
        42.to_string(),
        2.5.to_string(),
        'z'.to_string()
    );
    // ANCHOR_END: case
}

fn parameters() {
    // ANCHOR: params
    fn shout(s: &str) -> String {
        s.to_uppercase()
    }
    let owned = String::from("quiet");
    println!("{}", shout("literal")); // &'static str
    println!("{}", shout(&owned)); //    &String → &str by deref coercion
    println!("{}", shout(&owned[1..3])); // a sub-slice

    fn escape(input: &str) -> Cow<'_, str> {
        if input.contains(['<', '>', '&']) {
            let mut out = String::with_capacity(input.len() + 8);
            for c in input.chars() {
                match c {
                    '<' => out.push_str("&lt;"),
                    '>' => out.push_str("&gt;"),
                    '&' => out.push_str("&amp;"),
                    _ => out.push(c),
                }
            }
            Cow::Owned(out) // had to allocate
        } else {
            Cow::Borrowed(input) // zero-copy
        }
    }
    for text in ["plain text", "a < b && c"] {
        let escaped = escape(text);
        let kind = match &escaped {
            Cow::Borrowed(_) => "borrowed",
            Cow::Owned(_) => "owned",
        };
        println!("{escaped:<24} ({kind})");
    }
    // ANCHOR_END: params
}

fn literals() {
    // ANCHOR: literals
    let path = r"C:\vehicles\ecu.bin"; // raw: backslashes are literal
    let quoted = r#"He said "hi""#; // hashes allow quotes inside
    let bytes = b"CAN\x01"; // &[u8; 4]
    let one = b'A'; // u8
    let multi = "line one
line two";
    let c_string = c"for C"; // &CStr, NUL-terminated
    println!("{path}\n{quoted}\n{bytes:?} {one}\n{multi}");
    println!(
        "{c_string:?} has {} bytes including NUL",
        c_string.to_bytes_with_nul().len()
    );
    // ANCHOR_END: literals
}

fn other_types() {
    // ANCHOR: others
    use std::ffi::{CString, OsStr};
    use std::path::Path;

    let p = Path::new("/var/log/can0.log");
    println!(
        "file_name {:?}, extension {:?}, parent {:?}",
        p.file_name(),
        p.extension(),
        p.parent()
    );

    let os: &OsStr = OsStr::new("config.toml");
    println!("OsStr → &str: {:?}", os.to_str());

    let good = String::from_utf8(vec![0x52, 0x75, 0x73, 0x74]);
    let bad = String::from_utf8(vec![0x52, 0xFF]);
    println!("from_utf8 good: {good:?}");
    println!(
        "from_utf8 bad:  {:?}",
        bad.map_err(|e| e.utf8_error().to_string())
    );
    println!("lossy: {}", String::from_utf8_lossy(&[0x52, 0xFF, 0x53]));

    let c = CString::new("hello").unwrap();
    println!("CString bytes with NUL: {:?}", c.as_bytes_with_nul());
    println!(
        "interior NUL is rejected: {:?}",
        CString::new("he\0llo").is_err()
    );

    let frozen: Box<str> = String::from("read-only").into_boxed_str();
    println!("Box<str>: {frozen}");
    // ANCHOR_END: others
}
