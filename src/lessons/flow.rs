//! Lesson: Functions & control flow.

use crate::lesson::{Lesson, Question, Section};

pub static LESSON: Lesson = Lesson {
    id: "flow",
    title: "Functions & control flow",
    summary: "Functions, expressions vs statements, if/loop/while/for, labels, break values, ranges, the never type and fn pointers.",
    source: include_str!("flow.rs"),
    sections: &[
        Section::new(
            "Functions",
            r#"
            `fn name(param: Type, …) -> ReturnType { body }`. Every parameter
            must have a type annotation, and so must the return type unless
            it's `()`. Function names are `snake_case`. Functions can be defined
            in any order, and even inside other functions when they're helpers
            nobody else needs.

            Arguments are passed by value. What "by value" means for types
            that own heap memory, like `String`, is the subject of Lesson 6.
            "#,
        )
        .demo("functions", functions),
        Section::new(
            "Expressions and statements",
            r#"
            Rust is an **expression language**: almost everything produces a
            value, including blocks, `if`, `match` and `loop`. A **statement**
            is either a `let`, or an expression followed by `;`, which throws
            the value away and yields `()`.

            So the last expression of a function body, written without a
            semicolon, is its return value. Adding a stray semicolon is a
            classic first error:

            ```compile_fail,E0308
            fn plus_one(x: i32) -> i32 {
                x + 1; // the `;` turns the value into a statement → returns ()
            }
            ```

            The compiler will even suggest removing the semicolon. `return` is
            for **early** exits; at the end of a function, the bare expression
            is idiomatic.
            "#,
        )
        .demo("expressions", expressions),
        Section::new(
            "if is an expression",
            r#"
            `if` needs a `bool`; there's no truthiness. Because `if` is an
            expression, it replaces the ternary operator:
            `let x = if cond { a } else { b };`. Both branches must then have
            the same type:

            ```compile_fail,E0308
            let n = 3;
            let label = if n > 2 { "big" } else { 0 };
            ```

            An `if` without `else` has type `()`, so it can only be used as a
            statement.
            "#,
        )
        .demo("if", if_expression),
        Section::new(
            "loop, break values and labels",
            r#"
            `loop` repeats forever until a `break`. Like everything else, it's an
            expression: `break value` makes the loop evaluate to `value`, which
            is the clean way to write a retry loop or a search.

            `continue` skips to the next iteration. In nested loops, a
            **label** like `'outer:` lets `break 'outer` or `continue 'outer`
            target a specific loop. Labels also work on plain blocks, so
            `break 'block value` can exit a block early.
            "#,
        )
        .demo("loop", loops),
        Section::new(
            "while",
            r#"
            `while condition { … }` checks before each iteration. Use it when
            the number of iterations depends on state that changes inside the
            loop. When you are walking a collection or a range, `for` is
            better. (`while let`, which loops while a pattern matches, is
            covered in Lesson 12.)
            "#,
        )
        .demo("while", while_loop),
        Section::new(
            "for and ranges",
            r#"
            `for item in something` works with anything that can be turned into
            an iterator (Lesson 19), including ranges and collections:

            ```text
            0..5          0, 1, 2, 3, 4        (end excluded)
            0..=5         0, 1, 2, 3, 4, 5     (end included)
            (0..5).rev()  4, 3, 2, 1, 0
            (0..10).step_by(3)   0, 3, 6, 9
            ```

            Iterating a collection with `for` instead of indexing with a counter
            removes off-by-one errors and per-element bounds checks.
            `for x in &v` borrows each element, `for x in &mut v` lets you
            modify each one, and `for x in v` consumes the collection.
            "#,
        )
        .demo("for", for_loops),
        Section::new(
            "The never type !",
            r#"
            Some expressions never produce a value: `panic!`, `return`,
            `break`, `continue`, an endless `loop`, and
            `std::process::exit`. Their type is `!`, pronounced "never". Because
            a `!` can't actually occur, it coerces to any type. That's why this
            type-checks even though one arm has no number:

            ```rust
            let input = "12";
            let n: u32 = match input.parse() {
                Ok(n) => n,
                Err(_) => return, // type `!`, fits where a u32 is expected
            };
            ```

            A function can declare `-> !` to promise it never returns, which is
            useful for a fatal-error handler or a main loop on an embedded
            device.
            "#,
        )
        .demo("never", never_type),
        Section::new(
            "Functions as values, and recursion",
            r#"
            Functions are values too. The type `fn(i32) -> i32` is a **function
            pointer**, so you can store functions in variables, arrays and
            struct fields, or pass them as arguments. (Closures, Lesson 18,
            generalize this.)

            Recursion works as you'd expect, but Rust does **not** guarantee
            tail-call elimination. Deep recursion can overflow the stack, which
            aborts the process. Safety-critical coding standards usually ban
            recursion outright so that stack usage can be bounded statically.
            An explicit loop is the robust alternative.
            "#,
        )
        .demo("fn_values", function_values),
    ],
    quiz: &[
        Question::new(
            "What does this function return?\n\n```rust\nfn f() -> i32 { let x = 3; x * 2 }\n```",
            &[
                "Nothing — it needs `return`",
                "6",
                "()",
                "It doesn't compile",
            ],
            1,
            "The final expression without a semicolon is the return value.",
        ),
        Question::new(
            "What is the value of `loop { break 7; }`?",
            &["()", "7", "It never finishes", "A compile error"],
            1,
            "`break` can carry a value out of a `loop`, which makes the whole loop expression evaluate to it.",
        ),
        Question::new(
            "Which range yields 1, 2, 3, 4, 5?",
            &["1..5", "1..=5", "(1..5).rev()", "0..5"],
            1,
            "`..` excludes the end, `..=` includes it.",
        ),
        Question::new(
            "Why does `let n: u32 = match s.parse() { Ok(n) => n, Err(_) => return };` type-check?",
            &[
                "`return` evaluates to 0",
                "`return` has type `!`, which coerces to any type",
                "The compiler inserts a default",
                "It doesn't",
            ],
            1,
            "Diverging expressions have the never type `!`, which can stand in for any type.",
        ),
        Question::new(
            "Why do safety-critical standards often forbid recursion?",
            &[
                "It's slow",
                "Its stack usage can't be bounded statically, and Rust doesn't guarantee tail calls",
                "Rust doesn't support recursion",
                "It requires unsafe",
            ],
            1,
            "Without a bound on depth, you can't prove the stack won't overflow. Loops make the bound explicit.",
        ),
    ],
    exercises: &[
        "Write `fn collatz_steps(n: u64) -> u32` with a `loop` that uses `break` with a value. Print the steps for 1..=10 in a `for` loop.",
        "Find the first pair (i, j) with 1 ≤ i < j ≤ 50 where i² + j² is a perfect square, using labeled loops and `break 'outer`.",
        "Rewrite a recursive `fn fib(n: u32) -> u64` as an iterative one. Then use `checked_add` so it returns `None` instead of overflowing.",
        "Store three functions `fn(f64) -> f64` (e.g. `f64::sqrt`, a doubling fn, a negating fn) in an array, and apply each to 16.0.",
    ],
};

fn functions() {
    // ANCHOR: functions
    fn area(width: u32, height: u32) -> u32 {
        width * height
    }

    fn greet(name: &str) {
        // no `->`: returns ()
        println!("Hello, {name}!");
    }

    println!("area = {}", area(3, 4));
    greet("Ferris");
    println!("later-defined fn: {}", double(21)); // order doesn't matter
    // ANCHOR_END: functions
}

fn double(x: i32) -> i32 {
    x * 2
}

fn expressions() {
    // ANCHOR: expressions
    fn plus_one(x: i32) -> i32 {
        x + 1 // no semicolon: this value is returned
    }

    fn classify(temp: i32) -> &'static str {
        if temp < 0 {
            return "freezing"; // early return
        }
        if temp > 100 { "boiling" } else { "liquid" }
    }

    println!("plus_one(41) = {}", plus_one(41));
    for t in [-5, 20, 120] {
        println!("{t}°C is {}", classify(t));
    }

    let unit = {
        let _ = 5; // a statement
    }; //            a block ending in a statement has value ()
    println!("block with only statements = {unit:?}");
    // ANCHOR_END: expressions
}

fn if_expression() {
    // ANCHOR: if
    let fuel_percent = 12;
    let status = if fuel_percent < 10 {
        "critical"
    } else if fuel_percent < 25 {
        "low"
    } else {
        "ok"
    };
    println!("fuel {fuel_percent}% → {status}");

    let count = 0;
    if count != 0 {
        // `if count {` would be a type error: no truthiness
        println!("non-zero");
    }
    let parity = if count % 2 == 0 { "even" } else { "odd" };
    println!("{count} is {parity}");
    // ANCHOR_END: if
}

fn loops() {
    // ANCHOR: loop
    // A search loop that produces a value:
    let mut n = 1u32;
    let first_square_over_200 = loop {
        if n * n > 200 {
            break n * n;
        }
        n += 1;
    };
    println!("first square over 200: {first_square_over_200}");

    // Labeled loops: exit the outer loop from inside the inner one.
    let target = 42;
    let mut found = None;
    'outer: for a in 1..10 {
        for b in 1..10 {
            if a * b == target {
                found = Some((a, b));
                break 'outer;
            }
        }
    }
    println!("{target} = {:?}", found);

    // continue skips the rest of this iteration:
    for i in 0..8 {
        if i % 3 == 0 {
            continue;
        }
        print!("{i} ");
    }
    println!();

    // A labeled block: an early exit without a function.
    let config = Some("eco");
    let mode = 'pick: {
        let Some(name) = config else {
            break 'pick "default";
        };
        if name.is_empty() {
            break 'pick "default";
        }
        name
    };
    println!("mode = {mode}");
    // ANCHOR_END: loop
}

fn while_loop() {
    // ANCHOR: while
    let mut battery = 100.0_f64;
    let mut minutes = 0;
    while battery > 20.0 {
        battery *= 0.9; // drain 10% per minute
        minutes += 1;
    }
    println!("reached {battery:.1}% after {minutes} minutes");
    // ANCHOR_END: while
}

fn for_loops() {
    // ANCHOR: for
    for i in 0..3 {
        print!("{i} ");
    }
    println!("← 0..3");
    for i in (1..=10).rev().step_by(3) {
        print!("{i} ");
    }
    println!("← (1..=10).rev().step_by(3)");

    let mut speeds = vec![30, 50, 70];
    for s in &speeds {
        print!("{s} "); // borrow each element
    }
    println!();
    for s in &mut speeds {
        *s += 5; // modify in place
    }
    for (i, s) in speeds.iter().enumerate() {
        println!("lane {i}: {s} km/h");
    }
    for s in speeds {
        // consumes the Vec: `speeds` can't be used after this loop
        let _ = s;
    }
    // ANCHOR_END: for
}

// ANCHOR: never
fn fatal(message: &str) -> ! {
    // In a real program this might log and then call std::process::exit(1).
    panic!("fatal: {message}");
}

fn parse_or_default(text: &str) -> u32 {
    let n: u32 = match text.parse() {
        Ok(n) => n,
        Err(_) => return 0, // `return` has type `!`
    };
    n * 10
}

fn never_type() {
    println!("parse_or_default(\"7\") = {}", parse_or_default("7"));
    println!("parse_or_default(\"x\") = {}", parse_or_default("x"));

    let ok = true;
    let value: i32 = if ok { 5 } else { fatal("not ok") }; // `!` fits as i32
    println!("value = {value}");
}
// ANCHOR_END: never

fn function_values() {
    // ANCHOR: fn_values
    fn add_one(x: i32) -> i32 {
        x + 1
    }
    fn square(x: i32) -> i32 {
        x * x
    }
    fn apply_twice(f: fn(i32) -> i32, x: i32) -> i32 {
        f(f(x))
    }

    let ops: [(&str, fn(i32) -> i32); 3] =
        [("add_one", add_one), ("square", square), ("abs", i32::abs)];
    for (name, op) in ops {
        println!(
            "{name}(-3) = {:>3}   twice = {}",
            op(-3),
            apply_twice(op, -3)
        );
    }

    fn factorial(n: u64) -> u64 {
        if n == 0 { 1 } else { n * factorial(n - 1) } // recursion
    }
    fn factorial_iter(n: u64) -> Option<u64> {
        (1..=n).try_fold(1u64, |acc, k| acc.checked_mul(k)) // bounded, overflow-safe
    }
    println!("10! = {} = {:?}", factorial(10), factorial_iter(10));
    println!("25! overflows u64: {:?}", factorial_iter(25));
    // ANCHOR_END: fn_values
}
