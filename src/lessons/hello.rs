//! Lesson: Hello, Rust & formatting.

use crate::lesson::{Lesson, Question, Section};

pub static LESSON: Lesson = Lesson {
    id: "hello",
    title: "Hello, Rust & formatting",
    summary: "The anatomy of a Rust program, the printing macros, and the full format-string language.",
    source: include_str!("hello.rs"),
    sections: &[
        Section::new(
            "Your first program",
            r#"
            Every Rust executable starts at a function called `main`. It takes no
            arguments and, here, returns nothing:

            ```rust
            fn main() {
                println!("Hello, world!");
            }
            ```

            - `fn` declares a function. Bodies live in `{ }`.
            - `println!` ends in `!`, which means it is a **macro**, not a
              function. Macros are expanded at compile time; `println!` uses
              that to check your format string against its arguments before
              the program ever runs.
            - Statements end in `;`.

            To run it without Cargo, save it as `main.rs` and run
            `rustc --edition 2024 main.rs && ./main`. (Plain `rustc` defaults to
            the 2015 edition, so always pass `--edition`.) With Cargo:
            `cargo new hello && cd hello && cargo run`.

            Below is the body of a real function in this program. Every code
            sample marked like this is compiled into `rust_std` and executed
            when you view it — what you see is exactly what ran.
            "#,
        )
        .demo("hello", hello),
        Section::new(
            "The printing family",
            r#"
            Five macros share one formatting engine:

            - `print!` / `println!` write to **standard output** (stdout).
            - `eprint!` / `eprintln!` write to **standard error** (stderr). Use
              stderr for diagnostics so they don't mix with real output when
              someone pipes your program: `prog > out.txt` still shows errors.
            - `format!` builds a `String` instead of printing.

            Two more, `write!` and `writeln!`, format into anything that
            implements `std::fmt::Write` (like `String`) or `std::io::Write`
            (like files and sockets). They return a `Result`, because writing
            can fail.
            "#,
        )
        .demo("family", printing_family),
        Section::new(
            "Display {} and Debug {:?}",
            r#"
            The placeholder decides **which trait** formats the value. Think of a
            spectrum of audiences:

            ```text
            {}      Display   for end users      — must be implemented by hand
            {:?}    Debug     for developers     — usually #[derive(Debug)]
            {:#?}   Debug, pretty-printed across lines
            ```

            Numbers and strings implement both. Collections like `Vec` only
            implement `Debug`: there is no single "user-friendly" way to show a
            list, so Rust doesn't guess. Your own types get `Debug` from one
            line, `#[derive(Debug)]`. `Display` you write yourself (Lesson 17).
            "#,
        )
        .demo("display_debug", display_vs_debug),
        Section::new(
            "Positional, named and captured arguments",
            r#"
            Placeholders can refer to arguments by position or by name, and can
            reuse them. Since Rust 1.58 a placeholder can also **capture** a
            variable that is in scope: `{name}` looks up a local called `name`.

            Capturing only works with plain identifiers. `{user.name}` and
            `{items.len()}` are errors; pass expressions as arguments instead:
            `println!("{}", items.len())`.
            "#,
        )
        .demo("arguments", arguments),
        Section::new(
            "Width, alignment, fill, precision and sign",
            r#"
            After the colon comes a **format spec**. Its full grammar is:

            ```text
            {name:fill align sign # 0 width .precision type}
                  ─┬── ──┬── ─┬─ ┬ ┬ ──┬── ────┬───── ─┬──
                   │     │    │  │ │   │       │        └ ?, x, X, o, b, e, E
                   │     │    │  │ │   │       └ digits after the point, or max chars
                   │     │    │  │ │   └ minimum width
                   │     │    │  │ └ pad numbers with zeros
                   │     │    │  └ "alternate" form: 0x prefix, pretty Debug
                   │     │    └ + forces a sign
                   │     └ < left, ^ center, > right
                   └ any single character
            ```

            Every part is optional. Width and precision can come from arguments
            too: `{:>width$}` or `{:.*}` (the `*` takes precision from the next
            argument). This is how you line up tables without any crate.
            "#,
        )
        .demo("spec", format_spec),
        Section::new(
            "Number bases, exponents and pointers",
            r#"
            The trailing **type** letter picks a representation: `x`/`X` for
            hexadecimal, `o` octal, `b` binary, `e`/`E` scientific notation, and
            `p` for a pointer address. The `#` flag adds the conventional
            prefix (`0x`, `0o`, `0b`), and its width includes that prefix.

            Bases will matter a lot in the systems lessons, where you will read
            CAN frames and register values as hex and binary.
            "#,
        )
        .demo("bases", bases),
        Section::new(
            "Escapes and literal braces",
            r#"
            Inside format strings, `{{` and `}}` produce literal braces. Inside
            any string literal, backslash escapes work as in C: `\n`, `\t`,
            `\\`, `\"`, `\0`, plus `\u{1F980}` for any Unicode scalar value.
            A backslash at the end of a line skips the newline **and** the next
            line's leading whitespace, which is handy for long messages.
            "#,
        )
        .demo("escapes", escapes),
        Section::new(
            "Comments and documentation",
            r#"
            Rust has two kinds of comment, and the difference matters:

            - `//` and `/* ... */` are ordinary comments, ignored by the compiler.
            - `///` documents the item that follows; `//!` documents the
              enclosing item (a module or the crate). These are **doc comments**:
              `cargo doc` turns them into HTML, and code blocks inside them are
              compiled and run as tests by `cargo test`.

            Doc comments are Markdown. By convention they start with a
            one-line summary, then sections like `# Examples`, `# Errors`,
            `# Panics` and, for unsafe functions, `# Safety`.
            "#,
        )
        .demo("docs", call_documented),
        Section::new(
            "The toolchain",
            r#"
            You will use a small set of commands every day:

            ```text
            rustup update            install / update the toolchain
            cargo new my_app         new binary crate (--lib for a library)
            cargo check              type-check only — the fastest feedback loop
            cargo build [--release]  compile (debug by default; release optimizes)
            cargo run -- ARGS        build and run, passing ARGS to your program
            cargo test               unit, integration and doc tests
            cargo fmt                format the code (rustfmt)
            cargo clippy             lint for bugs and unidiomatic code
            cargo doc --open         build and open the API docs
            ```

            A crate's manifest is `Cargo.toml`. Its `edition` key picks the
            language edition (2015, 2018, 2021, 2024). Editions let Rust make
            small breaking changes, such as new keywords, without breaking old
            code, and crates of different editions link together freely.
            This course uses edition 2024.

            Debug and release builds sit on a spectrum of trade-offs. Debug
            compiles fast, keeps debug info, and **panics on integer
            overflow**. Release optimizes heavily and, by default, wraps on
            overflow instead (Lesson 3). Both are profiles in `Cargo.toml` that
            you can tune.
            "#,
        ),
    ],
    quiz: &[
        Question::new(
            "Why does `println!` end with an exclamation mark?",
            &[
                "It prints with emphasis",
                "It is a macro, expanded at compile time",
                "It can panic",
                "It is an unsafe function",
            ],
            1,
            "The `!` marks a macro invocation. `println!` checks your format string against the arguments at compile time.",
        ),
        Question::new(
            "Which placeholder works for a `Vec<i32>` without any extra code?",
            &["{}", "{:?}", "Both", "Neither"],
            1,
            "`Vec` implements `Debug` but not `Display`, so only `{:?}` (and `{:#?}`) work.",
        ),
        Question::new(
            "What does `format!(\"{:>6.2}\", 3.14159)` produce?",
            &["\"3.14  \"", "\"  3.14\"", "\"3.1416\"", "\"003.14\""],
            1,
            "Width 6, right-aligned (`>`), 2 digits after the point: two spaces then `3.14`.",
        ),
        Question::new(
            "Where should a program print error messages?",
            &[
                "stdout with println!",
                "stderr with eprintln!",
                "A String with format!",
                "It doesn't matter",
            ],
            1,
            "stderr keeps diagnostics separate from real output, so `prog > out.txt` still shows errors on screen.",
        ),
        Question::new(
            "Which line compiles?",
            &[
                "println!(\"{v.len()}\")",
                "println!(\"{}\", v.len())",
                "println!(\"{v}.len()\")  // prints the length",
                "println!(v.len())",
            ],
            1,
            "Inline capture only accepts plain identifiers. Pass expressions as arguments. (The third compiles but prints the vector followed by the text `.len()`.)",
        ),
        Question::new(
            "What does `format!(\"{:#06x}\", 255)` produce?",
            &["\"0x00ff\"", "\"0000ff\"", "\"0xff\"", "\"  0xff\""],
            0,
            "`#` adds the `0x` prefix, `0` pads with zeros, and the width (6) counts the prefix: `0x` plus `00ff`.",
        ),
    ],
    exercises: &[
        "Print a multiplication table for 1–9 with every column right-aligned to width 3, using only `print!` and `println!`.",
        "Print the number 3405691582 in decimal, hex (with prefix), octal and binary on one line each. Then work out why its hex form is memorable.",
        "Write a `format!` call that centers your name in a field of 20 asterisks, like `*****Ferris*******`. Then make the width a variable.",
        "Run a program that prints to both stdout and stderr, then redirect stdout to a file (`> out.txt`). Which messages stay on screen?",
    ],
};

fn hello() {
    // ANCHOR: hello
    println!("Hello, world!");
    println!("Hello from {}!", "Rust");
    // ANCHOR_END: hello
}

fn printing_family() {
    // ANCHOR: family
    use std::fmt::Write as _; // brings `write!` support for String into scope

    print!("print! adds no newline... ");
    println!("println! does.");
    eprintln!("eprintln! goes to stderr (you may see it out of order)");

    let greeting: String = format!("{}-{}", "format!", "returns a String");
    println!("{greeting}");

    let mut log = String::new();
    // write! returns fmt::Result; writing to a String cannot fail, so
    // unwrap is reasonable here.
    write!(log, "line {}", 1).unwrap();
    writeln!(log, ", then line {}", 2).unwrap();
    print!("{log}");
    // ANCHOR_END: family
}

// ANCHOR: display_debug
#[derive(Debug)]
struct Sensor {
    name: &'static str,
    reading: f64,
    ok: bool,
}

fn display_vs_debug() {
    let text = "tab\there";
    println!("Display: {}", text);
    println!("Debug:   {:?}", text); // quotes and escapes shown

    let readings = vec![20.5, 21.0, 19.75];
    println!("Debug vec: {:?}", readings);

    let s = Sensor {
        name: "coolant",
        reading: 88.5,
        ok: true,
    };
    println!("Debug struct: {:?}", s);
    println!("Pretty:\n{:#?}", s);
    println!("Fields still usable: {} {} {}", s.name, s.reading, s.ok);
}
// ANCHOR_END: display_debug

fn arguments() {
    // ANCHOR: arguments
    // Positional: {0} is the first argument, {1} the second.
    println!("{0} sees {1}; {1} sees {0}", "Alice", "Bob");

    // Named arguments.
    println!(
        "{city} is in {country}",
        city = "Toyota City",
        country = "Japan"
    );

    // Captured identifiers (Rust 1.58+).
    let model = "RAV4";
    let year = 2026;
    println!("{year} {model}");

    // Mix freely; expressions must be passed as arguments.
    let parts = ["engine", "battery", "brakes"];
    println!("{} parts, first is {first}", parts.len(), first = parts[0]);
    // ANCHOR_END: arguments
}

fn format_spec() {
    // ANCHOR: spec
    let pi = std::f64::consts::PI;
    println!("[{:8}]", 42); //   numbers right-align by default
    println!("[{:8}]", "hi"); // strings left-align by default
    println!("[{:<8}] [{:^8}] [{:>8}]", "left", "mid", "right");
    println!("[{:*^11}]", " fill ");
    println!("[{:08.3}]", pi); // zero-pad, width 8, 3 decimals
    println!("[{:+}] [{:+}]", 5, -5); // always show sign
    println!("[{:.2}] [{:.0}] [{:.0}]", pi, 2.5, 3.5); // ties round to even
    println!("[{:.3}]", "truncate me"); // precision truncates strings

    // Width and precision from variables or arguments:
    let width = 10;
    let decimals = 4;
    println!("[{:>width$.decimals$}]", pi);
    println!("[{:>1$}]", "dyn", 6); // width from argument 1
    println!("[{:.*}]", 2, pi); // precision from the next argument

    // A little table:
    for (part, qty, price) in [("bolt", 120, 0.15), ("gasket", 8, 4.5)] {
        println!("{:<8}|{:>5}|{:>8.2}", part, qty, price);
    }
    // ANCHOR_END: spec
}

fn bases() {
    // ANCHOR: bases
    let id = 0x1A5_u32; // hex literal
    println!("dec {id}  hex {id:x}  HEX {id:X}  prefixed {id:#x}");
    println!("oct {id:o} {id:#o}");
    println!("bin {id:b}");
    println!("byte  {:08b}", 5u8); // zero-padded to 8 digits
    println!("pfx   {:#010b}", 5u8); // width 10 includes the "0b"
    println!("sci   {:e}  {:E}", 1234.5, 0.00042);

    let value = 7;
    let address = &value;
    // The address changes from run to run; we print only its shape.
    let shown = format!("{address:p}");
    println!("pointer looks like 0x… ({} chars long)", shown.len());
    // ANCHOR_END: bases
}

fn escapes() {
    // ANCHOR: escapes
    println!("Braces: {{}} and {{{}}}", 42);
    println!("Tab\tseparated, \"quoted\", back\\slash");
    println!("Unicode: \u{1F980} \u{263A} \u{00E9}");
    let long = "This message is long, \
                so it continues on the next line \
                without extra spaces.";
    println!("{long}");
    // ANCHOR_END: escapes
}

// ANCHOR: docs
/// Converts a temperature from Celsius to Fahrenheit.
///
/// # Examples
///
/// ```
/// // `cargo test` compiles and runs this block as a "doc test".
/// assert_eq!(9.0 / 5.0 * 100.0 + 32.0, 212.0);
/// ```
fn celsius_to_fahrenheit(celsius: f64) -> f64 {
    celsius * 9.0 / 5.0 + 32.0 // a normal comment, ignored by rustdoc
}

/* A block comment: rarely used in practice,
but it can span lines. */
fn call_documented() {
    for c in [-40.0, 0.0, 37.0, 100.0] {
        println!("{c:>6.1} °C = {:>6.1} °F", celsius_to_fahrenheit(c));
    }
}
// ANCHOR_END: docs
