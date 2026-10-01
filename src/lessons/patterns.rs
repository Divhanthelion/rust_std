//! Lesson: Patterns in depth.

use crate::lesson::{Lesson, Question, Section};

pub static LESSON: Lesson = Lesson {
    id: "patterns",
    title: "Patterns in depth",
    summary: "Every pattern form: literals, ranges, alternatives, destructuring, guards, @ bindings, slice patterns, binding modes, if let, let-else, while let and let chains.",
    source: include_str!("patterns.rs"),
    sections: &[
        Section::new(
            "Patterns are everywhere",
            r#"
            Patterns aren't just for `match`. They appear in:

            ```text
            let PATTERN = value;              irrefutable: must always match
            fn f(PATTERN: Type)               irrefutable
            |PATTERN| closure params          irrefutable
            for PATTERN in iter               irrefutable
            match value { PATTERN => … }      refutable arms; all together exhaustive
            if let PATTERN = value            refutable
            while let PATTERN = value         refutable
            let PATTERN = value else { … }    refutable, with a diverging else
            ```

            A **refutable** pattern might fail to match, like `Some(x)`. An
            **irrefutable** one can't, like `(a, b)` against a pair. A plain
            `let` needs an irrefutable pattern:

            ```compile_fail,E0005
            let maybe: Option<i32> = None;
            let Some(x) = maybe; // error: refutable pattern in local binding
            ```
            "#,
        )
        .demo("everywhere", everywhere),
        Section::new(
            "Literals, ranges and alternatives",
            r#"
            - **Literals** match exact values: `0`, `'q'`, `"stop"`, `true`.
            - **Ranges**: `1..=9` (inclusive), `10..20` (exclusive, Rust
              1.80+), `100..` (open-ended), `..=0`. They work for integers and
              chars.
            - **Alternatives**: `'y' | 'Y'` matches either side.

            The compiler checks ranges for exhaustiveness too. It knows
            whether `..=0 | 1..=9 | 10..` covers every `i32`.
            "#,
        )
        .demo("ranges", ranges),
        Section::new(
            "Destructuring",
            r#"
            Patterns mirror the way values are built, so you take things apart
            with the same syntax you used to construct them: tuples, structs,
            enums, arrays and references, nested as deep as you like. A
            `&(a, b)` pattern matches a reference to a tuple and binds the
            parts.
            "#,
        )
        .demo("destructuring", destructuring),
        Section::new(
            "Ignoring: _, .. and _name",
            r#"
            - `_` matches anything and binds nothing.
            - `..` skips the remaining fields of a struct or tuple, or the
              middle elements of a slice.
            - `_name` binds the value but suppresses the unused warning.

            Remember from Lesson 2 that `_` doesn't bind, so it doesn't move
            either. `let _ = s;` leaves `s` usable, whereas `let _t = s;` moves
            it.
            "#,
        )
        .demo("ignoring", ignoring),
        Section::new(
            "Match guards",
            r#"
            A **guard** adds an arbitrary boolean condition to an arm:
            `Some(t) if t > 100 => …`. The arm matches only when the pattern
            matches **and** the guard is true. If the arm has alternatives
            (`A | B if cond`), the guard applies to the whole arm.

            The compiler can't see inside guards when it checks
            exhaustiveness. Arms with guards don't count as covering their
            pattern, so you usually need a final arm without one.
            "#,
        )
        .demo("guards", guards),
        Section::new(
            "@ bindings",
            r#"
            `name @ pattern` tests a value against a pattern **and** binds the
            whole value. It's most useful with ranges (`code @ 0x7E8..=0x7EF`)
            and when you need both an enum variant and its contents. Since Rust
            1.56 you can also bind inside with `whole @ Some(inner)`.
            "#,
        )
        .demo("at", at_bindings),
        Section::new(
            "Slice patterns",
            r#"
            Slices and arrays can be matched by shape:

            ```text
            []                     empty
            [x]                    exactly one element
            [first, .., last]      at least two; bind both ends
            [head, rest @ ..]      at least one; rest is a sub-slice
            [a, b, c]              exactly three (arrays: checked statically)
            ```

            They make command parsers and protocol decoders read like the
            grammar they implement, and they replace length checks plus
            indexing, which can't panic now.
            "#,
        )
        .demo("slices", slice_patterns),
        Section::new(
            "Binding modes: matching through references",
            r#"
            When you match a **reference** against a non-reference pattern,
            Rust automatically switches to binding by reference. Matching
            `&Option<String>` with `Some(s)` gives `s: &String` and moves
            nothing. This is called "match ergonomics", and it's why matching
            on `&self` fields just works.

            The older explicit forms still exist and occasionally help:
            `ref x` binds by shared reference and `ref mut x` by mutable
            reference, even when matching an owned value.
            "#,
        )
        .demo("modes", binding_modes),
        Section::new(
            "if let, let-else and while let",
            r#"
            When you care about only one pattern, a full `match` is noise:

            - `if let Some(x) = opt { … } else { … }` handles one case and lets
              everything else fall through.
            - `let Some(x) = opt else { return; };` (let-else, Rust 1.65+)
              binds `x` for the **rest of the function**, and its `else` must
              diverge (`return`, `break`, `continue` or `panic!`). It flattens
              the "happy path" and is great for validation.
            - `while let Some(x) = stack.pop() { … }` loops until the pattern
              stops matching.
            "#,
        )
        .demo("iflet", if_let_family),
        Section::new(
            "let chains",
            r#"
            Since Rust 1.88, in the 2024 edition, you can chain `let`
            patterns and boolean conditions with `&&` in `if` and `while`:

            ```rust
            let a = Some(3);
            let b: Result<i32, ()> = Ok(4);
            if let Some(x) = a && x > 2 && let Ok(y) = b {
                println!("{}", x + y);
            }
            ```

            Before, this needed nested `if let`s or a tuple match. Each binding
            is visible to the conditions after it.
            "#,
        )
        .demo("chains", let_chains),
        Section::new(
            "matches!",
            r#"
            `matches!(value, pattern)` is a `match` that returns `true` or
            `false`. It supports guards too: `matches!(c, 'a'..='z' if c != 'q')`.
            Use it for quick classification or in `filter` closures.
            "#,
        )
        .demo("matches", matches_macro),
    ],
    quiz: &[
        Question::new(
            "Which pattern matches a slice with at least two elements, binding the first and the last?",
            &[
                "[first, last]",
                "[first, .., last]",
                "[first, last @ ..]",
                "(first, .., last)",
            ],
            1,
            "`..` skips any number (including zero) of middle elements. `[first, last]` matches exactly two.",
        ),
        Question::new(
            "What must the `else` block of a let-else statement do?",
            &[
                "Return a default value",
                "Diverge: return, break, continue or panic",
                "Nothing special",
                "Bind the same variables",
            ],
            1,
            "Code after the let-else uses the bound variables, so the else branch must not fall through.",
        ),
        Question::new(
            "Matching `&Option<String>` with the pattern `Some(s)` gives `s` what type?",
            &["String (moved)", "&String", "&str", "Option<&String>"],
            1,
            "Default binding modes: matching through a reference binds by reference, so nothing is moved out.",
        ),
        Question::new(
            "Why does `let Some(x) = opt;` not compile?",
            &[
                "Option can't be destructured",
                "`let` requires an irrefutable pattern; use `if let` or `let … else`",
                "x must be mut",
                "It compiles fine",
            ],
            1,
            "A plain `let` must always match. `Some(x)` is refutable (error E0005).",
        ),
        Question::new(
            "In `match n { x @ 1..=9 => …, _ => … }`, what is `x`?",
            &[
                "The range 1..=9",
                "The matched value of n",
                "A bool",
                "The index of the arm",
            ],
            1,
            "`@` binds the whole value that matched the subpattern.",
        ),
    ],
    exercises: &[
        "Write `fn classify(c: char) -> &'static str` with ranges and alternatives for vowels, consonants, digits, whitespace and other.",
        "Parse command lines like `set speed 80`, `get rpm`, `reset` using `split_whitespace().collect::<Vec<_>>()` and a match on slice patterns.",
        "Rewrite a function with three nested `if let`s using let-else, then again using a let chain. Which reads best?",
        "Use `while let Some(top) = stack.pop()` to evaluate a Reverse Polish expression like `3 4 + 2 *`.",
    ],
};

fn everywhere() {
    // ANCHOR: everywhere
    let (x, y) = (3, 4); //                          let
    let distance = |(dx, dy): (i32, i32)| dx.abs() + dy.abs(); // closure params
    fn swap((a, b): (i32, i32)) -> (i32, i32) {
        //   fn params
        (b, a)
    }
    for (i, word) in ["zero", "one"].iter().enumerate() {
        // for
        println!("{i}: {word}");
    }
    println!("{} {:?}", distance((x, y)), swap((x, y)));
    // ANCHOR_END: everywhere
}

fn ranges() {
    // ANCHOR: ranges
    fn describe(code: u16) -> &'static str {
        match code {
            0 => "no error",
            1..=99 => "warning",
            100..500 => "recoverable fault", // exclusive range pattern
            500 | 501 | 502 => "critical (known)",
            503.. => "critical (other)",
        }
    }
    for code in [0, 42, 100, 499, 501, 9000] {
        println!("{code:>5}: {}", describe(code));
    }

    fn kind(c: char) -> &'static str {
        match c {
            'a' | 'e' | 'i' | 'o' | 'u' => "vowel",
            'a'..='z' => "consonant",
            '0'..='9' => "digit",
            _ => "other",
        }
    }
    println!("{:?}", "ev 9!".chars().map(kind).collect::<Vec<_>>());
    // ANCHOR_END: ranges
}

fn destructuring() {
    // ANCHOR: destructuring
    struct Point {
        x: i32,
        y: i32,
    }
    enum Shape {
        Circle { center: Point, radius: u32 },
        Line(Point, Point),
    }

    let shapes = [
        Shape::Circle {
            center: Point { x: 0, y: 0 },
            radius: 5,
        },
        Shape::Line(Point { x: 1, y: 2 }, Point { x: 1, y: 9 }),
        Shape::Line(Point { x: 0, y: 0 }, Point { x: 3, y: 4 }),
    ];
    for shape in &shapes {
        match shape {
            Shape::Circle {
                center: Point { x: 0, y: 0 },
                radius,
            } => {
                println!("circle at the origin, r = {radius}")
            }
            Shape::Circle { center, radius } => {
                println!("circle at ({}, {}), r = {radius}", center.x, center.y)
            }
            Shape::Line(Point { x: x1, .. }, Point { x: x2, .. }) if x1 == x2 => {
                println!("vertical line at x = {x1}")
            }
            Shape::Line(a, b) => println!("line ({},{})→({},{})", a.x, a.y, b.x, b.y),
        }
    }

    let pair = &(10, 'k');
    let &(n, c) = pair; // reference pattern: copy the parts out
    println!("{n} {c}");
    let [r, g, b] = [255u8, 128, 0]; // arrays destructure too
    println!("rgb {r} {g} {b}");
    // ANCHOR_END: destructuring
}

fn ignoring() {
    // ANCHOR: ignoring
    struct Frame {
        id: u32,
        dlc: u8,
        data: [u8; 8],
        timestamp: u64,
    }
    let f = Frame {
        id: 0x7E8,
        dlc: 3,
        data: [2, 0x41, 0x0D, 0, 0, 0, 0, 0],
        timestamp: 99,
    };

    let Frame { id, dlc, .. } = f; // skip the rest
    println!("id {id:#x}, dlc {dlc}");

    let (first, _, third) = (1, 2, 3);
    let [a, .., z] = f.data; // first and last of the array
    println!("{first} {third} | {a} {z} | ts {}", f.timestamp);

    let s = String::from("kept");
    let _ = s; //   `_` doesn't bind: no move
    println!("still have {s}");
    let _moved = s; // `_moved` *is* a binding: s moves
    // ANCHOR_END: ignoring
}

fn guards() {
    // ANCHOR: guards
    fn coolant_status(reading: Option<f64>) -> &'static str {
        match reading {
            Some(t) if t.is_nan() => "sensor fault",
            Some(t) if t > 110.0 => "overheating",
            Some(t) if t < 60.0 => "warming up",
            Some(_) => "normal",
            None => "no data",
        }
    }
    for r in [Some(115.0), Some(45.0), Some(88.0), Some(f64::NAN), None] {
        println!("{r:?} → {}", coolant_status(r));
    }

    // A guard applies to *all* alternatives of an arm:
    let n = 4;
    match n {
        2 | 4 | 6 if n > 3 => println!("{n}: even and > 3"),
        _ => println!("{n}: something else"),
    }
    // ANCHOR_END: guards
}

fn at_bindings() {
    // ANCHOR: at
    fn route(can_id: u16) -> String {
        match can_id {
            id @ 0x7E0..=0x7E7 => format!("{id:#x}: diagnostic request"),
            id @ 0x7E8..=0x7EF => format!("{id:#x}: diagnostic response from ECU {}", id - 0x7E8),
            0x000 => "broadcast".to_string(),
            other => format!("{other:#x}: application message"),
        }
    }
    for id in [0x7DF, 0x7E0, 0x7EA, 0x000] {
        println!("{}", route(id));
    }

    let reply = Some(42);
    if let whole @ Some(inner) = reply {
        println!("whole = {whole:?}, inner = {inner}");
    }
    // ANCHOR_END: at
}

fn slice_patterns() {
    // ANCHOR: slices
    fn run(words: &[&str]) -> String {
        match words {
            [] => "empty command".into(),
            ["help"] => "commands: get <key>, set <key> <value>, reset".into(),
            ["get", key] => format!("reading {key}"),
            ["set", key, value] => format!("setting {key} = {value}"),
            ["set", ..] => "usage: set <key> <value>".into(),
            [cmd, args @ ..] => format!("unknown {cmd:?} with {} argument(s)", args.len()),
        }
    }
    for line in ["", "help", "get rpm", "set limit 120", "set", "frob a b c"] {
        let words: Vec<&str> = line.split_whitespace().collect();
        println!("{line:<16} → {}", run(&words));
    }

    fn checksum_ok(frame: &[u8]) -> bool {
        match frame {
            [0x7E, body @ .., checksum, 0x7E] => {
                body.iter().fold(0u8, |acc, b| acc.wrapping_add(*b)) == *checksum
            }
            _ => false,
        }
    }
    println!("valid frame: {}", checksum_ok(&[0x7E, 1, 2, 3, 6, 0x7E]));
    println!("bad checksum: {}", checksum_ok(&[0x7E, 1, 2, 3, 7, 0x7E]));
    println!("too short:    {}", checksum_ok(&[0x7E]));
    // ANCHOR_END: slices
}

fn binding_modes() {
    // ANCHOR: modes
    let name: Option<String> = Some(String::from("Corolla"));

    match &name {
        Some(s) => println!("borrowed: {s} (len {})", s.len()), // s: &String
        None => println!("none"),
    }
    println!("still own it: {name:?}");

    let mut counter = Some(0);
    if let Some(c) = &mut counter {
        *c += 1; // c: &mut i32
    }
    println!("counter = {counter:?}");

    // `ref` borrows even when matching an owned value:
    let pair = (String::from("left"), String::from("right"));
    let (ref l, ref r) = pair;
    println!("{l} {r} and the pair is intact: {pair:?}");
    // ANCHOR_END: modes
}

fn if_let_family() {
    // ANCHOR: iflet
    // if let: one interesting case
    let config_value: Option<&str> = Some("250000");
    if let Some(text) = config_value {
        println!("configured: {text}");
    } else {
        println!("using default");
    }

    // let-else: validate, then continue on the happy path
    fn parse_speed(input: &str) -> Result<u32, String> {
        let Some(number) = input.strip_suffix("kmh") else {
            return Err(format!("{input:?}: missing unit"));
        };
        let Ok(value) = number.trim().parse::<u32>() else {
            return Err(format!("{input:?}: not a number"));
        };
        Ok(value) // `number` and `value` are in scope here
    }
    for input in ["80 kmh", "fast", "x kmh"] {
        println!("{input:?} → {:?}", parse_speed(input));
    }

    // while let: loop while the pattern matches
    let mut stack = vec![1, 2, 3];
    while let Some(top) = stack.pop() {
        print!("{top} ");
    }
    println!("← popped");
    // ANCHOR_END: iflet
}

fn let_chains() {
    // ANCHOR: chains
    struct Sensor {
        name: &'static str,
        value: Option<f64>,
        calibrated: bool,
    }
    let sensors = [
        Sensor {
            name: "oil",
            value: Some(97.0),
            calibrated: true,
        },
        Sensor {
            name: "fuel",
            value: None,
            calibrated: true,
        },
        Sensor {
            name: "tire",
            value: Some(31.0),
            calibrated: false,
        },
    ];
    for s in &sensors {
        if let Some(v) = s.value
            && s.calibrated
            && v > 50.0
        {
            println!("{}: {v} needs attention", s.name);
        } else {
            println!("{}: nothing to report", s.name);
        }
    }
    // ANCHOR_END: chains
}

fn matches_macro() {
    // ANCHOR: matches
    let codes = ["P0301", "U0100", "B1234", "P0420", "C0035"];
    let powertrain: Vec<&&str> = codes
        .iter()
        .filter(|c| matches!(c.as_bytes(), [b'P', ..]))
        .collect();
    println!("powertrain DTCs: {powertrain:?}");

    let c = 'k';
    println!(
        "lowercase but not q? {}",
        matches!(c, 'a'..='z' if c != 'q')
    );
    // ANCHOR_END: matches
}
