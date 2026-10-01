//! Lesson: Declarative macros.

use std::collections::HashMap;

use crate::lesson::{Lesson, Question, Section};

pub static LESSON: Lesson = Lesson {
    id: "macros",
    title: "Declarative macros",
    summary: "macro_rules!: matching, fragment specifiers, repetition, recursion, hygiene, generating items such as register tables, and the built-in macros.",
    source: include_str!("macros.rs"),
    sections: &[
        Section::new(
            "Why macros?",
            r#"
            Macros are **code that writes code** at compile time. They cover
            what functions can't express:

            - a **variable number of arguments**: `println!`, `vec!`;
            - **syntax checks at compile time**: `format!` validates its
              format string;
            - **declaring items**: types, impls or tables generated from a
              compact description;
            - capturing **source information**: `file!()`, `line!()`,
              `stringify!(expr)`.

            Rust has two kinds. **Declarative** macros (`macro_rules!`) match
            token patterns, and are covered here. **Procedural** macros, such as
            `#[derive(Debug)]` and attribute macros, are Rust functions that
            transform token streams; they need a dedicated `proc-macro` crate.
            The rule of thumb: prefer functions, then generics, then macros.
            Macros are harder to read, debug and document.
            "#,
        )
        .demo("why", why),
        Section::new(
            "macro_rules! basics",
            r#"
            A declarative macro is a list of **rules**: `(pattern) =>
            { expansion }`. The first rule whose pattern matches the input
            tokens wins. Captured fragments are named `$name:kind` and pasted
            into the expansion.

            A macro can be invoked with `()`, `[]` or `{}`. By convention you
            write `vec![…]` for collections, `name!(…)` for expressions and
            `name! { … }` for items. A macro must be defined **before** its
            use in the file (textual order), unlike functions.
            "#,
        )
        .demo("basics", basics),
        Section::new(
            "Fragment specifiers",
            r#"
            Each capture says what kind of syntax it accepts:

            ```text
            expr      an expression: 2 + 3, f(x), if c { a } else { b }
            ident     an identifier: speed, MyType
            ty        a type: u32, Vec<String>, &'a str
            pat       a pattern: Some(x), 1..=5, _
            literal   a literal: 42, "text", 'c', true
            block     a { … } block
            stmt      a statement, without its trailing semicolon
            item      an item: fn, struct, impl, …
            path      a path: std::collections::HashMap
            vis       a visibility, possibly empty: pub, pub(crate)
            lifetime  'a
            meta      the inside of an attribute: derive(Debug)
            tt        a single token tree: any token, or a (…), […], {…} group
            ```

            Edition 2024 note: `expr` now also matches `const { … }` blocks
            and `_`. The old behaviour is available as `expr_2021`.
            "#,
        )
        .demo("fragments", fragments),
        Section::new(
            "Repetition",
            r#"
            `$( … ),*` matches zero or more comma-separated repetitions, `+`
            means one or more, and `?` means zero or one. The separator (`,`,
            `;`, `=>`…) is optional. In the expansion, `$( … )*` repeats once
            per match, and each captured variable inside must have been
            captured the same number of times. This is how `vec!`,
            `HashMap`-literal macros and `assert!`-with-message work.
            "#,
        )
        .demo("repetition", repetition),
        Section::new(
            "Recursion and multiple rules",
            r#"
            Rules are tried top to bottom, so put specific patterns first. A
            macro can call itself to process its input piece by piece. This
            "token-tree muncher" style can implement surprisingly rich small
            languages. The compiler limits recursion depth
            (`#![recursion_limit]`, default 128), so keep inputs modest.
            "#,
        )
        .demo("recursion", recursion),
        Section::new(
            "Hygiene",
            r#"
            Local variables introduced **inside** a macro live in their own
            syntax context. They can't accidentally capture or shadow the
            caller's variables of the same name. That's **hygiene**, and C's
            preprocessor doesn't have it.

            Hygiene covers local variables and labels, but **not** items. A
            macro that defines `fn helper()` really defines `helper` in the
            caller's scope. For paths to your own crate, exported macros use
            `$crate::path`, so they work wherever they're invoked.
            "#,
        )
        .demo("hygiene", hygiene),
        Section::new(
            "Generating items: a register map",
            r#"
            Macros shine at removing boilerplate that must stay consistent.
            Below, one compact table of hardware registers produces a typed
            constant per register, a lookup from address to name, and a list
            of all registers. Adding a register is a one-line change, and the
            three artifacts can't drift apart. The same idea generates
            bitflag types, CAN signal definitions and diagnostic-code tables.
            "#,
        )
        .demo("registers", registers),
        Section::new(
            "Built-in macros worth knowing",
            r#"
            ```text
            stringify!(x + 1)      the tokens as a &'static str: "x + 1"
            concat!("a", 1, true)  a literal string at compile time: "a1true"
            file!(), line!(), column!(), module_path!()   source location
            env!("CARGO_PKG_VERSION")   a build-time environment variable
            option_env!("X")       the same, but Option instead of a compile error
            include_str!/include_bytes!  embed a file (this program embeds its lessons)
            cfg!(...)              a configuration check as a bool
            compile_error!("…")    fail the build with a message
            dbg!(expr)             prints file:line, the expression and its value to
                                   stderr, then returns the value
            todo!(), unimplemented!(), unreachable!()
            matches!, assert!, assert_eq!, debug_assert!, write!, format_args!
            ```
            "#,
        )
        .demo("builtins", builtins),
        Section::new(
            "Exporting and debugging macros",
            r#"
            Inside a crate, a `macro_rules!` is visible after its definition in
            textual order. To share it across modules, add `pub(crate) use
            my_macro;` after it, and then import it like any item.
            `#[macro_export]` publishes it at the root of the crate for other
            crates.

            To see what a macro expands to, use `cargo expand` (a separate
            tool), or `rustc -Zunpretty=expanded` on nightly. While developing a
            macro, write one test per rule, and add a `compile_error!` arm for
            input it should reject with a clear message.
            "#,
        ),
    ],
    quiz: &[
        Question::new(
            "Which fragment specifier matches any single token or a bracketed group?",
            &["expr", "ident", "tt", "item"],
            2,
            "`tt` (token tree) matches one token or one delimited group, which makes it the most flexible specifier.",
        ),
        Question::new(
            "What does `$( $x:expr ),+` match?",
            &[
                "Zero or more expressions",
                "One or more comma-separated expressions",
                "Exactly one expression",
                "Expressions separated by +",
            ],
            1,
            "`+` means one or more repetitions; the `,` before it is the separator.",
        ),
        Question::new(
            "A macro defines `let tmp = 1;` internally. The caller also has a `tmp`. What happens?",
            &[
                "The caller's tmp is overwritten",
                "Compile error",
                "Nothing: hygiene keeps them separate",
                "Undefined behaviour",
            ],
            2,
            "macro_rules! variables are hygienic: identifiers from the macro and from the call site don't collide.",
        ),
        Question::new(
            "Why do exported macros write `$crate::helper()` instead of `crate::helper()`?",
            &[
                "It's shorter",
                "`$crate` resolves to the defining crate wherever the macro is expanded",
                "crate:: is deprecated",
                "Hygiene requires it for locals",
            ],
            1,
            "When expanded in another crate, `crate::` would mean the caller's crate. `$crate` always points back to the macro's own crate.",
        ),
        Question::new(
            "What does `dbg!(x * 2)` return?",
            &[
                "()",
                "The value of x * 2",
                "A String",
                "Nothing; it only prints",
            ],
            1,
            "dbg! prints the location and value to stderr, and then returns the value, so you can wrap expressions in place.",
        ),
    ],
    exercises: &[
        "Write `min!(a, b, …)` that works for any number of arguments by recursion, using `std::cmp::min`.",
        "Write `newtype!(Meters, f64)` that generates a tuple struct with `Debug`, `Clone`, `Copy`, `PartialEq`, a `new` fn and a `value()` getter.",
        "Write `flags! { Abs = 0, Esc = 1, Tcs = 2 }` that generates a `u8` bit constant per name and `fn names(bits: u8) -> Vec<&'static str>`.",
        "Add a rule to `hashmap!` that accepts a trailing comma, and a `compile_error!` rule for an empty invocation.",
    ],
};

fn why() {
    // ANCHOR: why
    let v = vec![1, 2, 3]; //                 variadic construction
    let s = format!("{} + {} = {}", 1, 2, 3); // format string checked at compile time
    println!("{v:?} {s}");
    println!("stringify!: {}", stringify!(speed * 3.6 + offset));
    println!(
        "we are at {}:{}",
        file!().rsplit('/').next().unwrap_or("?"),
        line!()
    );
    // ANCHOR_END: why
}

fn basics() {
    // ANCHOR: basics
    macro_rules! square {
        ($x:expr) => {
            $x * $x
        };
    }

    macro_rules! greet {
        () => {
            println!("hello, nobody")
        };
        ($name:expr) => {
            println!("hello, {}", $name)
        };
    }

    println!("square!(7) = {}", square!(7));
    println!("square!(2 + 3) = {}", square!(2 + 3)); // 25: $x is one expression, not text
    greet!();
    greet!("Ferris");
    let list = [square!(2), square!(3)]; // [] or {} delimiters work too: square![2]
    println!("{list:?}");
    // ANCHOR_END: basics
}

fn fragments() {
    // ANCHOR: fragments
    // ident + ty + expr: generate a getter function.
    macro_rules! constant_fn {
        ($vis:vis $name:ident -> $t:ty = $value:expr) => {
            $vis fn $name() -> $t {
                $value
            }
        };
    }
    constant_fn!(max_rpm -> u32 = 6_500);
    constant_fn!(pub(crate) model -> &'static str = "Corolla");
    println!("{} {}", max_rpm(), model());

    // pat: build a classifier from patterns.
    macro_rules! is_any {
        ($value:expr, $($p:pat_param)|+) => {
            matches!($value, $($p)|+)
        };
    }
    println!(
        "is 'R' a gear letter? {}",
        is_any!('R', 'P' | 'R' | 'N' | 'D')
    );

    // block + literal:
    macro_rules! timed {
        ($label:literal, $body:block) => {{
            let start = std::time::Instant::now();
            let result = $body;
            let _elapsed = start.elapsed(); // a real version would log this
            println!("{} finished", $label);
            result
        }};
    }
    let sum = timed!("summing", { (1..=1000).sum::<u32>() });
    println!("sum = {sum}");
    // ANCHOR_END: fragments
}

fn repetition() {
    // ANCHOR: repetition
    macro_rules! hashmap {
        ($($key:expr => $value:expr),* $(,)?) => {{
            let mut map = HashMap::new();
            $( map.insert($key, $value); )*
            map
        }};
    }

    let limits = hashmap! {
        "speed" => 180,
        "rpm" => 6500,
        "temp" => 110, // trailing comma accepted by $(,)?
    };
    let mut keys: Vec<_> = limits.keys().collect();
    keys.sort();
    println!("{} entries: {keys:?}", limits.len());

    macro_rules! sum_all {
        ($($x:expr),+) => {
            0 $( + $x )+
        };
    }
    println!("sum_all!(1, 2, 3, 4) = {}", sum_all!(1, 2, 3, 4));
    // ANCHOR_END: repetition
}

fn recursion() {
    // ANCHOR: recursion
    macro_rules! count {
        () => { 0usize };
        ($head:tt $($tail:tt)*) => { 1usize + count!($($tail)*) };
    }

    macro_rules! max {
        ($x:expr) => { $x };
        ($x:expr, $($rest:expr),+) => {{
            let a = $x;
            let b = max!($($rest),+);
            if a > b { a } else { b }
        }};
    }

    println!("count!(a b c d) = {}", count!(a b c d));
    println!("max!(3, 9, 4, 1) = {}", max!(3, 9, 4, 1));
    const N: usize = count!(x y z); // usable in const context
    println!("N = {N}");
    // ANCHOR_END: recursion
}

fn hygiene() {
    // ANCHOR: hygiene
    macro_rules! double_it {
        ($e:expr) => {{
            let value = $e; // this `value` is the macro's own
            value * 2
        }};
    }

    let value = 10;
    let result = double_it!(value + 1); // the caller's `value`: no collision
    println!("double_it!(value + 1) = {result}, caller's value still {value}");

    macro_rules! make_helper {
        () => {
            fn helper() -> &'static str {
                "items are not hygienic: helper() is now in scope here"
            }
        };
    }
    make_helper!();
    println!("{}", helper());
    // ANCHOR_END: hygiene
}

fn registers() {
    // ANCHOR: registers
    macro_rules! registers {
        ($( $name:ident = $addr:literal : $doc:literal ),+ $(,)?) => {
            #[allow(non_snake_case, dead_code)]
            mod reg {
                $( #[doc = $doc] pub const $name: u16 = $addr; )+

                pub fn name_of(addr: u16) -> Option<&'static str> {
                    match addr {
                        $( $addr => Some(stringify!($name)), )+
                        _ => None,
                    }
                }

                pub const ALL: &[(&str, u16)] = &[ $( (stringify!($name), $addr) ),+ ];
            }
        };
    }

    registers! {
        CTRL    = 0x00 : "control: enable, reset, loopback",
        STATUS  = 0x04 : "status: bus-off, error-passive",
        BITRATE = 0x08 : "bit timing configuration",
        TX_BUF  = 0x10 : "transmit buffer",
    }

    println!("STATUS register at {:#04x}", reg::STATUS);
    println!(
        "0x08 is {:?}, 0x0C is {:?}",
        reg::name_of(0x08),
        reg::name_of(0x0C)
    );
    for (name, addr) in reg::ALL {
        println!("  {addr:#04x}  {name}");
    }
    // ANCHOR_END: registers
}

fn builtins() {
    // ANCHOR: builtins
    println!("stringify!: {}", stringify!(a + b * c));
    println!("concat!:    {}", concat!("ecu-", 7, "-", true));
    println!(
        "location:   {}:{}:{}",
        file!().rsplit('/').next().unwrap_or("?"),
        line!(),
        column!()
    );
    println!("module:     {}", module_path!());
    println!("version:    {}", env!("CARGO_PKG_VERSION"));
    println!("option_env: {:?}", option_env!("RUST_STD_SURELY_UNSET"));
    let source_lines = include_str!("macros.rs").lines().count();
    println!("this lesson's source has {source_lines} lines (via include_str!)");

    let doubled = dbg!(21 * 2); // prints to stderr, returns 42
    println!("dbg! returned {doubled}");
    // ANCHOR_END: builtins
}
