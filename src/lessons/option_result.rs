//! Lesson: Option & Result.

use std::collections::HashMap;
use std::num::ParseIntError;

use crate::lesson::{Lesson, Question, Section};

pub static LESSON: Lesson = Lesson {
    id: "option_result",
    title: "Option & Result",
    summary: "Absence and failure as values: extracting, transforming and combining Options and Results, the ? operator, and collecting.",
    source: include_str!("option_result.rs"),
    sections: &[
        Section::new(
            "No null, by design",
            r#"
            Rust has no null pointer for ordinary references. A value that
            might be absent has type `Option<T>`, and the compiler won't let
            you use it as a `T` until you've handled `None`. Tony Hoare called
            null his "billion-dollar mistake". In Rust the mistake becomes a
            type error.

            `Result<T, E>` does the same for operations that can **fail**: an
            `Ok(T)` value or an `Err(E)` explaining why. Both are plain enums
            (Lesson 11) with a large toolbox of methods.
            "#,
        )
        .demo("intro", intro),
        Section::new(
            "Getting the value out: a spectrum",
            r#"
            From most explicit to most abrupt:

            ```text
            match / if let / let-else       handle every case yourself
            unwrap_or(default)              fall back to a value (always evaluated)
            unwrap_or_else(|| compute())    fall back to a computed value (lazy)
            unwrap_or_default()             fall back to Default::default()
            expect("why this can't fail")   panic with a message if absent
            unwrap()                        panic with a generic message
            ```

            `expect` and `unwrap` belong in tests, prototypes, and places where
            absence really is a bug. In those last cases, `expect` with a
            message documenting the invariant is better: "config was validated
            at startup". Production code in safety-critical systems
            usually bans both (Lesson 35).
            "#,
        )
        .demo("extract", extract),
        Section::new(
            "Transforming Options",
            r#"
            Combinators let you work on the value inside without unwrapping it:

            - `map(f)` transforms a `Some`, and leaves `None` alone.
            - `and_then(f)` chains a step that can itself return `None` (a
              "flat map").
            - `filter(pred)` turns a `Some` into `None` unless `pred` holds.
            - `or(other)` and `or_else(f)` supply an alternative `Option`.
            - `zip`, `xor` and `flatten` combine or unwrap layers of options.
            - `is_some_and(pred)` and `is_none_or(pred)` (Rust 1.82+) test the
              contents.
            - `take`, `replace`, `get_or_insert_with` and `insert` modify an
              `Option` in place.
            "#,
        )
        .demo("transform", transform),
        Section::new(
            "Working with Results",
            r#"
            `Result` has mirror-image combinators. `map` touches `Ok`, and
            `map_err` touches `Err`. `and_then` chains fallible steps, and
            `unwrap_or_else` recovers from errors. `.ok()` converts to an
            `Option`, discarding the error, and `.err()` keeps only the error.
            "#,
        )
        .demo("results", results),
        Section::new(
            "Converting between Option and Result",
            r#"
            - `opt.ok_or(err)` or `opt.ok_or_else(|| err)` turns an `Option`
              into a `Result`, which you'll want when absence is an error worth
              explaining.
            - `res.ok()` turns a `Result` into an `Option`.
            - `transpose()` swaps `Option<Result<T, E>>` and `Result<Option<T>, E>`.
              That's handy for optional fields that must be valid when they're
              present.
            "#,
        )
        .demo("convert", convert),
        Section::new(
            "The ? operator",
            r#"
            `expr?` means "if this is `Err`/`None`, return it from the current
            function; otherwise unwrap it". For `Result` it is roughly:

            ```rust
            # fn demo() -> Result<i32, std::num::ParseIntError> {
            let n: i32 = match "42".parse::<i32>() {
                Ok(v) => v,
                Err(e) => return Err(From::from(e)), // note the From conversion
            };
            # Ok(n) }
            ```

            The `From::from` call means the error type can be **converted**
            on the way out. That's how `?` lets several error types funnel
            into one (Lesson 14). It works with `Option` in functions returning
            `Option`. It doesn't silently mix the two: use `ok_or` to cross
            over.
            "#,
        )
        .demo("question", question_mark),
        Section::new(
            "Borrowing the inside: as_ref, as_mut, as_deref",
            r#"
            Most combinators take `self` by value, which would move a
            `String` out of an `Option<String>` you only wanted to look at.
            `as_ref()` turns `&Option<T>` into `Option<&T>` first, and
            `as_mut()` does the same mutably. `as_deref()` goes one step
            further, turning `Option<String>` into `Option<&str>`. Going the
            other way, `copied()` and `cloned()` turn `Option<&T>` into
            `Option<T>`.
            "#,
        )
        .demo("borrow", borrow_inside),
        Section::new(
            "Collecting Options and Results",
            r#"
            An iterator of `Result<T, E>` can be collected into
            `Result<Vec<T>, E>`. It stops at the **first** error and returns
            it, or returns all the values. Options work the same way, and so do
            `sum` and `product`. This replaces a manual loop with an early
            return. To keep the successes and gather the failures instead, use
            `partition` or `filter_map(Result::ok)`.
            "#,
        )
        .demo("collect", collecting),
    ],
    quiz: &[
        Question::new(
            "What does `Some(4).filter(|x| x % 2 == 1)` return?",
            &["Some(4)", "None", "Some(true)", "false"],
            1,
            "The predicate is false for 4, so the Some becomes None.",
        ),
        Question::new(
            "Which is lazy: `unwrap_or(expensive())` or `unwrap_or_else(|| expensive())`?",
            &["unwrap_or", "unwrap_or_else", "Both", "Neither"],
            1,
            "Arguments are evaluated before the call, so `unwrap_or(expensive())` always runs `expensive()`. The closure only runs on None.",
        ),
        Question::new(
            "In a function returning `Result<T, MyError>`, `?` on a `Result<_, ParseIntError>` compiles if…",
            &[
                "Always",
                "MyError implements From<ParseIntError>",
                "ParseIntError implements From<MyError>",
                "Never",
            ],
            1,
            "`?` calls `From::from` on the error, so the function's error type must be convertible from the inner one.",
        ),
        Question::new(
            "What does collecting `[\"1\", \"x\", \"3\"].iter().map(|s| s.parse::<i32>())` into `Result<Vec<i32>, _>` give?",
            &[
                "Ok([1, 3])",
                "Err(the error for \"x\")",
                "Ok([1, 0, 3])",
                "A panic",
            ],
            1,
            "Collecting into Result short-circuits at the first Err.",
        ),
        Question::new(
            "How do you get an `Option<&str>` from `name: Option<String>` without moving it?",
            &[
                "name.unwrap()",
                "name.as_deref()",
                "name.map(|s| &s)",
                "&name",
            ],
            1,
            "`as_deref` borrows through the option and derefs `String` to `str`.",
        ),
    ],
    exercises: &[
        "Write `fn parse_pair(s: &str) -> Option<(i32, i32)>` for input like \"3,4\" using only combinators and `?`, with no `match`.",
        "Given `HashMap<&str, &str>` of settings, write `fn port(settings) -> Result<u16, String>` that distinguishes 'missing' from 'not a number' using `ok_or_else` and `map_err`.",
        "Parse a whitespace-separated line of numbers into `Result<Vec<u8>, ParseIntError>` with a single `collect`. Then change it to skip bad entries instead.",
        "Implement `fn first_even_square(v: &[i32]) -> Option<i32>` using `iter().find` and `map`.",
    ],
};

fn intro() {
    // ANCHOR: intro
    let readings: [Option<f64>; 3] = [Some(88.0), None, Some(92.5)];
    for (i, r) in readings.iter().enumerate() {
        match r {
            Some(v) => println!("sensor {i}: {v}"),
            None => println!("sensor {i}: no data"),
        }
    }

    let parsed: Result<u8, _> = "300".parse::<u8>();
    match parsed {
        Ok(v) => println!("parsed {v}"),
        Err(e) => println!("failed: {e}"),
    }
    // ANCHOR_END: intro
}

fn extract() {
    // ANCHOR: extract
    let present: Option<u32> = Some(7);
    let absent: Option<u32> = None;

    println!(
        "unwrap_or:         {} {}",
        present.unwrap_or(0),
        absent.unwrap_or(0)
    );
    println!("unwrap_or_else:    {}", absent.unwrap_or_else(|| 40 + 2));
    println!("unwrap_or_default: {}", absent.unwrap_or_default());
    println!(
        "expect:            {}",
        present.expect("sensor 0 is always fitted")
    );

    let fallback = || {
        println!("  (computing fallback)");
        99
    };
    println!("lazy on Some:  {}", present.unwrap_or_else(fallback)); // closure never runs
    println!("lazy on None:  {}", absent.unwrap_or_else(fallback));

    let Some(v) = present else {
        println!("no value");
        return;
    };
    println!("let-else bound {v}");
    // ANCHOR_END: extract
}

fn transform() {
    // ANCHOR: transform
    let speed: Option<u32> = Some(72);
    println!("map:        {:?}", speed.map(|kmh| kmh as f64 / 1.609)); // mph
    println!("filter:     {:?}", speed.filter(|&s| s > 100));
    println!(
        "and_then:   {:?}",
        speed.and_then(|s| s.checked_mul(100_000_000))
    );
    println!("or:         {:?}", None.or(Some(50)).or(speed));
    println!("zip:        {:?}", speed.zip(Some("km/h")));
    println!("xor:        {:?}", speed.xor(None::<u32>));
    println!("flatten:    {:?}", Some(Some(3)).flatten());
    println!("is_some_and {}", speed.is_some_and(|s| s > 60));
    println!("is_none_or  {}", None::<u32>.is_none_or(|s| s > 60));

    let mut cache: Option<Vec<u8>> = None;
    cache.get_or_insert_with(Vec::new).push(1); // create on first use
    cache.get_or_insert_with(Vec::new).push(2);
    println!("cache:      {cache:?}");
    let old = cache.replace(vec![9]);
    println!("replace:    old {old:?}, new {cache:?}");
    let taken = cache.take();
    println!("take:       {taken:?}, left {cache:?}");
    // ANCHOR_END: transform
}

fn results() {
    // ANCHOR: results
    fn parse_rpm(s: &str) -> Result<u32, ParseIntError> {
        s.trim().parse()
    }

    let good = parse_rpm(" 3200 ");
    let bad = parse_rpm("fast");

    println!("map:       {:?}", good.clone().map(|rpm| rpm / 60)); // rev/s
    println!(
        "map_err:   {:?}",
        bad.clone().map_err(|e| format!("bad rpm: {e}"))
    );
    println!(
        "and_then:  {:?}",
        good.clone()
            .and_then(|rpm| parse_rpm(&(rpm * 2).to_string()))
    );
    println!("recover:   {}", bad.clone().unwrap_or_else(|_| 800)); // idle
    println!("ok():      {:?} {:?}", good.clone().ok(), bad.clone().ok());
    println!("err():     {:?}", bad.as_ref().err().map(|e| e.to_string()));
    println!("is_ok_and: {}", good.is_ok_and(|rpm| rpm < 6500));
    // ANCHOR_END: results
}

fn convert() {
    // ANCHOR: convert
    let settings: HashMap<&str, &str> = HashMap::from([("bitrate", "500000"), ("mode", "normal")]);

    fn bitrate(settings: &HashMap<&str, &str>) -> Result<u32, String> {
        let text = settings.get("bitrate").ok_or("bitrate is not set")?;
        text.parse().map_err(|e| format!("bitrate {text:?}: {e}"))
    }
    println!("bitrate → {:?}", bitrate(&settings));
    println!("empty   → {:?}", bitrate(&HashMap::new()));

    // An optional setting that must be valid if present:
    let timeout: Option<Result<u32, ParseIntError>> = settings.get("timeout").map(|t| t.parse());
    let timeout: Result<Option<u32>, ParseIntError> = timeout.transpose();
    println!("timeout → {timeout:?}");
    // ANCHOR_END: convert
}

fn question_mark() {
    // ANCHOR: question
    // With Result: each `?` returns early on Err.
    fn total_mass(lines: &[&str]) -> Result<u32, ParseIntError> {
        let mut total = 0;
        for line in lines {
            let kg: u32 = line.parse()?;
            total += kg;
        }
        Ok(total)
    }
    println!("{:?}", total_mass(&["1500", "75", "80"]));
    println!("{:?}", total_mass(&["1500", "heavy"]));

    // With Option: `?` returns None early.
    fn second_word_len(text: &str) -> Option<usize> {
        let word = text.split_whitespace().nth(1)?;
        Some(word.len())
    }
    println!(
        "{:?} {:?}",
        second_word_len("adaptive cruise"),
        second_word_len("lane")
    );
    // ANCHOR_END: question
}

fn borrow_inside() {
    // ANCHOR: borrow
    let mut driver: Option<String> = Some(String::from("Akio"));

    let len = driver.as_ref().map(|s| s.len()); // doesn't move the String
    let name: Option<&str> = driver.as_deref(); // Option<String> → Option<&str>
    println!("{len:?} {name:?}");

    if let Some(s) = driver.as_mut() {
        s.push_str(" T.");
    }
    println!("{driver:?}");

    let numbers = [10, 20, 30];
    let first: Option<&i32> = numbers.first();
    let owned: Option<i32> = first.copied(); // Option<&i32> → Option<i32>
    println!("{first:?} {owned:?}");
    // ANCHOR_END: borrow
}

fn collecting() {
    // ANCHOR: collect
    let inputs = ["12", "7", "x", "40"];

    let all: Result<Vec<u8>, _> = inputs.iter().map(|s| s.parse::<u8>()).collect();
    println!("all or nothing: {all:?}");

    let good_only: Vec<u8> = inputs.iter().filter_map(|s| s.parse().ok()).collect();
    println!("skip failures:  {good_only:?}");

    let (oks, errs): (Vec<_>, Vec<_>) = inputs
        .iter()
        .map(|s| s.parse::<u8>())
        .partition(Result::is_ok);
    println!("partition:      {} ok, {} errors", oks.len(), errs.len());

    let sum: Option<u32> = [Some(1), Some(2), Some(3)].into_iter().sum();
    let sum_missing: Option<u32> = [Some(1), None, Some(3)].into_iter().sum();
    println!("sum of options: {sum:?} {sum_missing:?}");
    // ANCHOR_END: collect
}
