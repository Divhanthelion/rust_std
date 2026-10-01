//! Lesson: Panic freedom & determinism.

use std::collections::{BTreeMap, HashMap};
use std::panic;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use crate::lesson::{Lesson, Question, Section};

pub static LESSON: Lesson = Lesson {
    id: "panic_free",
    title: "Panic freedom & determinism",
    summary: "Where panics hide, how to write total functions, lint-enforced panic policies, overflow and panic settings, panic hooks, and sources of non-determinism.",
    source: include_str!("panic_free.rs"),
    sections: &[
        Section::new(
            "Why panics matter in a vehicle",
            r#"
            A panic is Rust's controlled reaction to a bug, and it's far better
            than memory corruption. In a brake or steering controller, though,
            "controlled stop of this thread" can still mean **loss of
            function**. Safety engineering (ISO 26262) reasons in a chain:

            ```text
            fault (a bug)  →  error (wrong internal state)  →  failure (wrong behaviour)
            ```

            Rust's type system removes many faults. A panic-free policy keeps
            the remaining ones from turning into failures. There's a spectrum
            of strategies, and real systems combine them:

            ```text
            avoid     write total functions; make panics impossible by construction
            contain   catch at task boundaries; restart the task or degrade
            abort     treat any panic as a fault; reset into a safe state
            ```
            "#,
        ),
        Section::new(
            "Where panics hide",
            r#"
            Some panics are explicit (`panic!`, `unwrap`, `expect`,
            `unreachable!`, `todo!`). Many are **implicit** in innocent-looking
            code:

            ```text
            v[i], &s[a..b]           index or range out of bounds
            &text[0..n]              slicing a str in the middle of a UTF-8 character
            a + b, a * b             integer overflow (debug builds, or overflow-checks)
            a / b, a % b             division by zero (always), MIN / -1
            x << n                   shift amount ≥ bit width (debug builds)
            refcell.borrow_mut()     already borrowed
            dst.copy_from_slice(src) length mismatch
            v.chunks(0)              zero chunk size
            Duration - Duration      underflow
            vec![0; huge]            allocation failure (abort)
            ```

            The demo triggers each one under `catch_unwind` with a silent hook
            and prints the message. Every line below would have taken down a
            thread.
            "#,
        )
        .demo("hidden", hidden_panics),
        Section::new(
            "Total functions: every input gets an answer",
            r#"
            A **total** function returns a value for **every** possible input.
            Panic-free code is built from total pieces:

            ```text
            v[i]                 →  v.get(i)                     Option
            &v[a..b]             →  v.get(a..b)                  Option
            a + b                →  a.checked_add(b)             Option (or saturating_/wrapping_)
            a / b                →  a.checked_div(b)             Option
            s.parse().unwrap()   →  s.parse()?                   Result
            arr.try_into().unwrap() → split_first_chunk / try_into()?
            ```

            Push the `Option`s and `Result`s up to a level that can **decide**
            what to do: use a default, keep the last good value, report a
            diagnostic trouble code, or enter a degraded mode. The function
            below can't panic, whatever bytes it's given.
            "#,
        )
        .demo("total", total_functions),
        Section::new(
            "Enforce the policy with lints",
            r#"
            Discipline doesn't scale, lints do. Clippy's **restriction**
            group has lints that ban panicking constructs. Enable them for
            safety-relevant crates or modules, and CI enforces the policy:

            ```rust,nowrap
            // At the top of lib.rs (or a module) of a safety-relevant crate:
            #![forbid(unsafe_code)]
            #![deny(
                clippy::unwrap_used,
                clippy::expect_used,
                clippy::panic,
                clippy::indexing_slicing,
                clippy::arithmetic_side_effects,
                clippy::unreachable,
                clippy::todo,
                clippy::as_conversions
            )]
            ```

            `forbid` can't be overridden further down, while `deny` can be
            relaxed locally with a justified `#[allow(...)]`. That gives an
            auditable list of exceptions. (This course's lessons demonstrate
            panics on purpose, so they don't enable these.)
            "#,
        ),
        Section::new(
            "Build settings: overflow checks and panic strategy",
            r#"
            Two `Cargo.toml` profile settings change panic behaviour without
            touching code:

            ```text
            [profile.release]
            overflow-checks = true   # keep overflow detection in release builds
            panic = "abort"          # no unwinding: smaller binaries, immediate stop
            ```

            - `overflow-checks = true` turns silent wrap-around into a detected
              fault. That's slower, but only where you use plain operators,
              and you should prefer the explicit methods anyway.
            - `panic = "abort"` removes unwinding, which most embedded targets
              require anyway. Destructors don't run, and `catch_unwind` can't
              catch anything. Your **panic handler** (Lesson 37) or hook must
              put the system into a safe state.
            "#,
        ),
        Section::new(
            "Panic hooks: record the fault before going down",
            r#"
            `std::panic::set_hook` runs **before** unwinding or aborting. That's
            the place to record a fault: increment a counter, store a
            diagnostic code in non-volatile memory, or tell a watchdog that the
            reset is intentional. Keep hooks tiny and allocation-free; the
            system is already in trouble. `catch_unwind` at a **task boundary**
            then lets a supervisor restart just that task, which works when
            unwinding is enabled.
            "#,
        )
        .demo("hook", hooks),
        Section::new(
            "Determinism: same input, same output",
            r#"
            Safety arguments and reproducible bug reports both need
            deterministic behaviour. Classic sources of non-determinism:

            - **HashMap iteration order**, which is randomized per map. Use
              `BTreeMap`, or sort.
            - **Floating-point summation order**: `(a + b) + c != a + (b + c)`.
              Fix the order, and beware of parallel reductions.
            - **Wall-clock time**: inject a clock (Lesson 28), and use
              `Instant` for intervals.
            - **Unbounded work**: loops without an iteration limit, recursion,
              and allocation.
            - **Thread scheduling**: anything that depends on which thread runs
              first.
            "#,
        )
        .demo("determinism", determinism),
        Section::new(
            "Bounded loops",
            r#"
            Every loop in a control path should have an **explicit, provable
            bound**. A loop that is supposed to end ("until the sensor answers")
            becomes a `for` over a fixed number of attempts, followed by an
            explicit failure path. Hangs become errors, and the worst-case
            execution time can be computed.
            "#,
        )
        .demo("bounded", bounded_loops),
        Section::new(
            "Going further: proving it",
            r#"
            Tests show panics are absent **on the inputs you tried**. Stronger
            assurance comes from tools:

            - **Kani**, a model checker for Rust, can prove that a function
              can't panic or overflow for **all** inputs of bounded size.
            - **Miri** interprets code to detect undefined behaviour in
              `unsafe`.
            - **Fuzzing** (`cargo fuzz`) explores inputs guided by coverage.
            - Linker tricks like the `no-panic` crate fail the **build** if a
              function could panic.

            In ASIL-rated projects, these join coverage measurement and code
            review in the safety case (Lesson 39).
            "#,
        ),
    ],
    quiz: &[
        Question::new(
            "Which expression can panic in a release build with default settings?",
            &[
                "a.wrapping_add(b)",
                "v.get(10)",
                "x / y where y may be 0",
                "a + b on u32 (with overflow-checks off)",
            ],
            2,
            "Division by zero panics in every build. Plain + wraps in release by default; get and wrapping_add never panic.",
        ),
        Question::new(
            "What's the main effect of `panic = \"abort\"`?",
            &[
                "Panics are ignored",
                "The process stops immediately: no unwinding, no destructors, catch_unwind can't catch",
                "Panics become Results",
                "Only debug builds abort",
            ],
            1,
            "Abort removes unwinding machinery. It's smaller and simpler, but cleanup must happen in the hook or panic handler.",
        ),
        Question::new(
            "Why prefer BTreeMap over HashMap in code whose output must be reproducible?",
            &[
                "BTreeMap is faster",
                "HashMap iteration order is randomized per map",
                "HashMap can panic",
                "BTreeMap uses less memory",
            ],
            1,
            "Iteration order of a HashMap changes between runs. BTreeMap iterates in key order.",
        ),
        Question::new(
            "`#![forbid(unsafe_code)]` vs `#![deny(unsafe_code)]`?",
            &[
                "Identical",
                "forbid can't be overridden by inner #[allow]; deny can",
                "deny is stronger",
                "forbid only warns",
            ],
            1,
            "forbid is a hard ceiling. deny allows local, visible exceptions.",
        ),
        Question::new(
            "What is a 'total function'?",
            &[
                "A function that uses all its parameters",
                "A function that returns a value for every possible input, with no panics or infinite loops",
                "A function with no side effects",
                "A const fn",
            ],
            1,
            "Total functions are defined for their entire input domain; partiality is expressed in the return type (Option/Result).",
        ),
    ],
    exercises: &[
        "Take a parser you wrote in an earlier lesson and enable `#![deny(clippy::indexing_slicing, clippy::unwrap_used, clippy::arithmetic_side_effects)]` on its module. Fix every finding.",
        "Write `fn average(samples: &[u16]) -> Option<u16>` that is total: no overflow, no division by zero, no indexing.",
        "Install a panic hook that increments a static `AtomicU32` fault counter and records the panic location into a fixed-size buffer, without allocating.",
        "Show that `[0.1f64; 10].iter().sum()` and the same sum computed pairwise differ in the last bits, and decide which order your code must specify.",
    ],
};

fn hidden_panics() {
    // ANCHOR: hidden
    use std::hint::black_box; // keeps the compiler from rejecting obvious cases

    let v = vec![1u8, 2, 3];
    let text = "héllo";
    let zero = black_box(0u32);
    let cases: Vec<(&str, Box<dyn Fn() + panic::RefUnwindSafe>)> = vec![
        (
            "index",
            Box::new(|| {
                black_box(black_box(&v)[black_box(5)]);
            }),
        ),
        (
            "slice range",
            Box::new(|| {
                black_box(&black_box(&v)[1..black_box(9)]);
            }),
        ),
        (
            "str boundary",
            Box::new(|| {
                black_box(&black_box(text)[0..black_box(2)]);
            }),
        ),
        (
            "division",
            Box::new(move || {
                black_box(black_box(10u32) / zero);
            }),
        ),
        (
            "overflow (debug)",
            Box::new(|| {
                black_box(black_box(u8::MAX) + black_box(1));
            }),
        ),
        (
            "unwrap None",
            Box::new(|| {
                black_box(black_box(None::<u8>).unwrap());
            }),
        ),
        (
            "copy_from_slice",
            Box::new(|| {
                [0u8; 2].copy_from_slice(black_box(&[1, 2, 3]));
            }),
        ),
        (
            "Duration sub",
            Box::new(|| {
                black_box(black_box(Duration::from_secs(1)) - Duration::from_secs(2));
            }),
        ),
    ];

    let previous = panic::take_hook();
    panic::set_hook(Box::new(|_| {})); // silence the default report
    for (name, f) in &cases {
        let outcome = panic::catch_unwind(f);
        let message = match outcome {
            Ok(()) => "no panic in this build".to_string(),
            Err(payload) => payload
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| payload.downcast_ref::<&str>().map(|s| s.to_string()))
                .unwrap_or_default(),
        };
        println!("{name:<17} → {message}");
    }
    panic::set_hook(previous);
    // ANCHOR_END: hidden
}

fn total_functions() {
    // ANCHOR: total
    #[derive(Debug, PartialEq)]
    enum Reject {
        TooShort,
        BadChecksum,
        Overflow,
    }

    /// Sum of `count` big-endian u16 readings after a 1-byte count header,
    /// followed by a 1-byte XOR checksum. Total: never panics, for any input.
    fn checked_total(frame: &[u8]) -> Result<u32, Reject> {
        let (&count, rest) = frame.split_first().ok_or(Reject::TooShort)?;
        let body_len = usize::from(count).checked_mul(2).ok_or(Reject::Overflow)?;
        let body = rest.get(..body_len).ok_or(Reject::TooShort)?;
        let &checksum = rest.get(body_len).ok_or(Reject::TooShort)?;
        let xor = frame
            .iter()
            .take(body_len.saturating_add(1))
            .fold(0u8, |acc, b| acc ^ b);
        if xor != checksum {
            return Err(Reject::BadChecksum);
        }
        let (pairs, _) = body.as_chunks::<2>(); // &[[u8; 2]]: no indexing needed
        pairs
            .iter()
            .map(|&pair| u32::from(u16::from_be_bytes(pair)))
            .try_fold(0u32, |acc, x| acc.checked_add(x))
            .ok_or(Reject::Overflow)
    }

    let mut good = vec![2, 0x01, 0x00, 0x00, 0x20];
    let xor = good.iter().fold(0u8, |a, b| a ^ b);
    good.push(xor);
    let frames: [&[u8]; 4] = [&good, &good[..3], &[], &[2, 1, 0, 0, 0x20, 0x00]];
    for frame in frames {
        println!("{frame:02X?} → {:?}", checked_total(frame));
    }
    // ANCHOR_END: total
}

fn hooks() {
    // ANCHOR: hook
    static FAULTS: AtomicU32 = AtomicU32::new(0);
    static LAST_FAULT_LINE: AtomicU32 = AtomicU32::new(0);

    let previous = panic::take_hook();
    panic::set_hook(Box::new(|info| {
        // Tiny, allocation-free bookkeeping: count and remember where.
        FAULTS.fetch_add(1, Ordering::Relaxed);
        if let Some(loc) = info.location() {
            LAST_FAULT_LINE.store(loc.line(), Ordering::Relaxed);
        }
    }));

    let before = FAULTS.load(Ordering::Relaxed);
    // A supervisor restarting a task that fails twice, then succeeds:
    let mut attempts = 0;
    let result = loop {
        attempts += 1;
        let outcome = panic::catch_unwind(|| {
            if attempts < 3 {
                panic!("sensor task crashed");
            }
            "sensor task healthy"
        });
        match outcome {
            Ok(status) => break Some(status),
            Err(_) if attempts < 5 => continue, // bounded restarts
            Err(_) => break None,
        }
    };
    panic::set_hook(previous);

    println!("result {result:?} after {attempts} attempts");
    println!(
        "faults recorded by the hook: {}",
        FAULTS.load(Ordering::Relaxed) - before
    );
    println!(
        "last fault on line {}",
        LAST_FAULT_LINE.load(Ordering::Relaxed)
    );
    // ANCHOR_END: hook
}

fn determinism() {
    // ANCHOR: determinism
    let readings = [
        ("rear_left", 31.0),
        ("front_left", 32.5),
        ("rear_right", 30.5),
        ("front_right", 32.0),
    ];

    let hashed: HashMap<_, _> = readings.iter().copied().collect();
    let sorted: BTreeMap<_, _> = readings.iter().copied().collect();
    println!(
        "HashMap order:  {:?}  (may differ next run)",
        hashed.keys().collect::<Vec<_>>()
    );
    println!(
        "BTreeMap order: {:?}  (always)",
        sorted.keys().collect::<Vec<_>>()
    );

    // Floating-point addition is not associative:
    let (a, b, c) = (1e16, -1e16, 1.0);
    println!(
        "(a + b) + c = {}   a + (b + c) = {}",
        (a + b) + c,
        a + (b + c)
    );

    let small = [0.1f64; 10];
    let forward: f64 = small.iter().sum();
    let pairwise = small.chunks(2).map(|p| p.iter().sum::<f64>()).sum::<f64>();
    println!("sum of ten 0.1: forward {forward:.17}, pairwise {pairwise:.17}");
    // ANCHOR_END: determinism
}

fn bounded_loops() {
    // ANCHOR: bounded
    #[derive(Debug)]
    enum PollError {
        NoAnswer { attempts: u32 },
    }

    /// Polls a sensor at most `MAX_ATTEMPTS` times. Never hangs.
    fn wait_for_ready(mut is_ready: impl FnMut() -> bool) -> Result<u32, PollError> {
        const MAX_ATTEMPTS: u32 = 5;
        for attempt in 1..=MAX_ATTEMPTS {
            if is_ready() {
                return Ok(attempt);
            }
            // a real driver would wait one tick here
        }
        Err(PollError::NoAnswer {
            attempts: MAX_ATTEMPTS,
        })
    }

    let mut calls = 0;
    println!(
        "slow sensor   → {:?}",
        wait_for_ready(|| {
            calls += 1;
            calls >= 3
        })
    );
    println!("dead sensor   → {:?}", wait_for_ready(|| false));
    // ANCHOR_END: bounded
}
