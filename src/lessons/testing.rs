//! Lesson: Testing.

use std::cell::Cell;
use std::hint::black_box;
use std::panic;
use std::time::{Duration, Instant};

use crate::lesson::{Lesson, Question, Section};
use crate::rng::Rng;

pub static LESSON: Lesson = Lesson {
    id: "testing",
    title: "Testing",
    summary: "Unit, integration and doc tests; should_panic and Result tests; table-driven golden vectors; property tests with a seeded PRNG; test doubles; benchmarks without crates.",
    source: include_str!("testing.rs"),
    sections: &[
        Section::new(
            "Unit tests",
            r#"
            Tests are ordinary functions marked `#[test]`. They pass if they
            return normally and fail if they panic. By convention unit tests
            live next to the code, in a module compiled only for testing:

            ```rust,nowrap
            pub fn add(a: u32, b: u32) -> u32 {
                a + b
            }

            #[cfg(test)]
            mod tests {
                use super::*; // the private items of the parent are visible

                #[test]
                fn adds() {
                    assert_eq!(add(2, 2), 4);
                    assert_ne!(add(2, 2), 5, "custom message with {}", "formatting");
                    assert!(add(0, 0) == 0);
                }
            }
            ```

            `cargo test` builds a special binary with a test **harness**
            (libtest) that runs every `#[test]` function, in parallel by
            default, and catches panics. The demo below is a ten-line harness
            of our own, so you can see the mechanism.
            "#,
        )
        .demo("harness", harness),
        Section::new(
            "should_panic, Result tests and ignore",
            r#"
            ```rust,nowrap
            #[cfg(test)]
            mod tests {
                #[test]
                #[should_panic(expected = "divide by zero")]
                fn rejects_zero() {
                    let d = 0;
                    if d == 0 { panic!("divide by zero"); }
                }

                #[test]
                fn parses() -> Result<(), std::num::ParseIntError> {
                    let n: u32 = "42".parse()?; // `?` instead of unwrap
                    assert_eq!(n, 42);
                    Ok(())
                }

                #[test]
                #[ignore = "needs a CAN interface"]
                fn hardware_in_the_loop() {}
            }
            ```

            `expected` matches a **substring** of the panic message, so the
            test fails if it panics for a different reason. Run ignored tests
            with `cargo test -- --ignored`.
            "#,
        ),
        Section::new(
            "Integration and doc tests",
            r#"
            - **Integration tests** live in `tests/*.rs`. Each file is a
              separate crate that can only use your **public** API, exactly like
              a user. For binaries, Cargo exposes the built executable's path as
              `env!("CARGO_BIN_EXE_<name>")`.
            - **Doc tests** are the code blocks in `///` comments. They're
              compiled and run, so examples can't rot. Attributes on the fence
              tune them: `no_run`, `ignore`, `should_panic`, and `compile_fail`.
              Lines starting with `# ` are hidden from the docs but still
              compiled.

            This repository uses all three. `tests/cli.rs` drives the real
            binary. `tests/prose_examples.rs` compiles every code block in these
            lessons and checks that each "does not compile" example really
            fails, with the stated error code. `src/snippet.rs` has doc tests.
            Run `cargo test` in the repo to watch them.
            "#,
        ),
        Section::new(
            "Running tests",
            r#"
            ```text
            cargo test                      everything: unit, integration, doc
            cargo test parse                only tests whose name contains "parse"
            cargo test --test cli           one integration test file
            cargo test --doc                only doc tests
            cargo test -- --nocapture       show println! output from passing tests
            cargo test -- --test-threads=1  run serially (shared resources)
            cargo test -- --ignored         run the #[ignore] tests
            cargo test --release            test the optimized build too
            ```

            Arguments after `--` go to the test harness, not to Cargo. Because
            tests run in parallel, they mustn't share mutable global state,
            fixed file names or ports. Use unique temporary paths, as this
            repo's CLI tests do.
            "#,
        ),
        Section::new(
            "Table-driven tests and golden vectors",
            r#"
            Many cases, one loop: put inputs and expected outputs in an array
            and check them all, with messages that name the failing case. For
            protocols and checksums, use **golden vectors**: known-correct
            values from a specification or a reference implementation. The
            classic check value for CRC-32 is `crc32("123456789") ==
            0xCBF43926`. If your implementation matches it, you've almost
            certainly got the polynomial, reflection and final XOR right.
            "#,
        )
        .demo("table", table_driven),
        Section::new(
            "Property tests with a seeded PRNG",
            r#"
            Instead of hand-picking inputs, generate thousands at random and
            check a **property** that must always hold:

            - **round-trip**: `decode(encode(x)) == x`;
            - **invariants**: output sorted, length preserved, no overflow;
            - **model-based**: your fast structure behaves like a simple,
              obviously-correct one (a `VecDeque`, a `Vec`).

            Crates like `proptest` add input shrinking, but the core needs only
            a PRNG. **Seed it** and print the seed on failure, so any failure
            is reproducible. This program has a 30-line xorshift PRNG in
            `src/rng.rs`.
            "#,
        )
        .demo("property", property_tests),
        Section::new(
            "Test doubles: inject time and I/O",
            r#"
            Code that reads the clock, the filesystem or the network is hard to
            test deterministically. Pass those dependencies in through a
            **trait** (or `impl Read`, `impl Write`), and use a fake in tests.
            A watchdog that takes a `Clock` can be tested instantly, without
            sleeping, by advancing a fake clock. It's ordinary dependency
            injection, and static dispatch makes it zero-cost in production.
            "#,
        )
        .demo("doubles", test_doubles),
        Section::new(
            "Benchmarks without crates",
            r#"
            For quick measurements: run in `--release`, repeat the work many
            times, take the **minimum** (the least-disturbed run), and wrap
            inputs and outputs in `std::hint::black_box`. Otherwise the
            optimizer may compute the result at compile time and you'd be
            timing nothing. For serious work, `criterion` handles warm-up and
            statistics, but the principles are the same.
            "#,
        )
        .demo("bench", benchmark),
        Section::new(
            "Assertions in production code",
            r#"
            Tests aren't the only place assertions belong:

            - `assert!` documents and enforces an invariant in every build.
              Keep it for cheap checks whose failure must never go unnoticed.
            - `debug_assert!` is checked in debug and test builds and compiled
              out of release. Use it for expensive consistency checks.

            Safety-critical projects also measure **coverage**: statement,
            branch and, at the highest integrity levels, MC/DC (modified
            condition/decision coverage). `cargo llvm-cov` reports it, and
            MC/DC support in rustc is an active project goal (Lesson 39).
            "#,
        ),
    ],
    quiz: &[
        Question::new(
            "Why do unit test modules use `#[cfg(test)]`?",
            &[
                "To make tests run faster",
                "So the test code is compiled only for `cargo test`, not into the shipped binary",
                "It's required for #[test] to work",
                "To hide tests from clippy",
            ],
            1,
            "cfg(test) strips the module from normal builds. It also lets tests use private items via `use super::*`.",
        ),
        Question::new(
            "What does `#[should_panic(expected = \"overflow\")]` check?",
            &[
                "That the test panics, with any message",
                "That the test panics with a message containing \"overflow\"",
                "That the test doesn't panic",
                "That the word overflow is printed",
            ],
            1,
            "The expected string must be a substring of the panic message, which guards against panicking for the wrong reason.",
        ),
        Question::new(
            "Why seed a random property test and print the seed?",
            &[
                "Speed",
                "So that any failing case can be reproduced exactly",
                "It's required by the compiler",
                "To get better randomness",
            ],
            1,
            "With the seed you can replay the exact sequence of inputs that failed.",
        ),
        Question::new(
            "In a micro-benchmark, what does `black_box` prevent?",
            &[
                "Timer overhead",
                "The optimizer precomputing or deleting the work being measured",
                "Cache misses",
                "Thread preemption",
            ],
            1,
            "black_box makes values opaque to the optimizer, so the measured code actually runs.",
        ),
        Question::new(
            "Which test can only use your crate's public API?",
            &[
                "Unit tests in mod tests",
                "Integration tests in tests/",
                "Doc tests on private fns",
                "All of them",
            ],
            1,
            "Each file in tests/ is compiled as a separate crate that depends on yours, so only pub items are visible.",
        ),
    ],
    exercises: &[
        "Add a `#[cfg(test)] mod tests` to one of your earlier exercise solutions with at least one table-driven test and one `should_panic` test.",
        "Write a property test checking that sorting any random `Vec<i32>` gives a non-decreasing sequence with the same length and the same multiset of elements.",
        "Model-test a fixed-capacity ring buffer against `VecDeque` for 10,000 random push/pop operations with a seeded PRNG.",
        "Benchmark `Vec::contains` against `HashSet::contains` for 10, 100 and 10,000 elements. Where's the crossover?",
    ],
};

// ANCHOR: harness
/// A miniature test harness: run each test, catch panics, report like libtest.
fn run_tests(tests: &[(&str, fn())]) -> (usize, usize) {
    let previous = panic::take_hook();
    panic::set_hook(Box::new(|_| {})); // silence the default panic message
    let mut passed = 0;
    for (name, test) in tests {
        let ok = panic::catch_unwind(*test).is_ok();
        println!("test {name} ... {}", if ok { "ok" } else { "FAILED" });
        passed += usize::from(ok);
    }
    panic::set_hook(previous);
    (passed, tests.len() - passed)
}

fn clamp_percent(x: i32) -> u8 {
    x.clamp(0, 100) as u8
}

fn harness() {
    let tests: [(&str, fn()); 3] = [
        ("clamps_high", || assert_eq!(clamp_percent(140), 100)),
        ("clamps_low", || assert_eq!(clamp_percent(-5), 0)),
        ("deliberately_wrong", || {
            assert_eq!(clamp_percent(50), 51, "off by one")
        }),
    ];
    let (passed, failed) = run_tests(&tests);
    println!("test result: {passed} passed; {failed} failed");
}
// ANCHOR_END: harness

// ANCHOR: table
/// CRC-32 (IEEE 802.3, reflected, polynomial 0xEDB88320), bit by bit.
fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &byte in data {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg(); // all ones if the low bit is set
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    !crc
}

fn table_driven() {
    // Golden vectors from the CRC catalogue and common references:
    let cases: &[(&str, &[u8], u32)] = &[
        ("empty", b"", 0x0000_0000),
        ("check value", b"123456789", 0xCBF4_3926),
        ("single a", b"a", 0xE8B7_BE43),
        (
            "pangram",
            b"The quick brown fox jumps over the lazy dog",
            0x414F_A339,
        ),
    ];
    for (name, input, expected) in cases {
        let got = crc32(input);
        let verdict = if got == *expected { "ok" } else { "MISMATCH" };
        println!("{name:<12} {got:#010x}  {verdict}");
    }
}
// ANCHOR_END: table

fn property_tests() {
    // ANCHOR: property
    // Property 1: encode → decode round-trips for every u16.
    fn encode(v: u16) -> [u8; 2] {
        v.to_be_bytes()
    }
    fn decode(b: [u8; 2]) -> u16 {
        u16::from_be_bytes(b)
    }

    let seed = 0x5EED_2026;
    let mut rng = Rng::new(seed);
    let mut failures = 0;
    for _ in 0..10_000 {
        let v = rng.next_u64() as u16;
        if decode(encode(v)) != v {
            failures += 1;
        }
    }
    println!("round-trip: 10000 cases, {failures} failures (seed {seed:#x})");

    // Property 2 (model-based): a sorted insert behaves like push + sort.
    fn insert_sorted(v: &mut Vec<i32>, x: i32) {
        let pos = v.partition_point(|&y| y < x);
        v.insert(pos, x);
    }
    let mut fast = Vec::new();
    let mut model = Vec::new();
    for _ in 0..1_000 {
        let x = (rng.next_u64() % 1000) as i32 - 500;
        insert_sorted(&mut fast, x);
        model.push(x);
        model.sort();
    }
    println!("sorted-insert matches the model: {}", fast == model);
    // ANCHOR_END: property
}

fn test_doubles() {
    // ANCHOR: doubles
    trait Clock {
        fn now(&self) -> Duration; // time since some fixed start
    }

    struct Watchdog<C: Clock> {
        clock: C,
        last_kick: Duration,
        timeout: Duration,
    }
    impl<C: Clock> Watchdog<C> {
        fn new(clock: C, timeout: Duration) -> Self {
            let last_kick = clock.now();
            Watchdog {
                clock,
                last_kick,
                timeout,
            }
        }
        fn kick(&mut self) {
            self.last_kick = self.clock.now();
        }
        fn expired(&self) -> bool {
            self.clock.now() - self.last_kick > self.timeout
        }
    }

    // The fake: time only moves when the test says so.
    struct FakeClock(Cell<Duration>);
    impl Clock for &FakeClock {
        fn now(&self) -> Duration {
            self.0.get()
        }
    }
    impl FakeClock {
        fn advance(&self, ms: u64) {
            self.0.set(self.0.get() + Duration::from_millis(ms));
        }
    }

    let clock = FakeClock(Cell::new(Duration::ZERO));
    let mut dog = Watchdog::new(&clock, Duration::from_millis(100));
    clock.advance(60);
    println!("after 60 ms: expired = {}", dog.expired());
    dog.kick();
    clock.advance(90);
    println!("kicked, then 90 ms: expired = {}", dog.expired());
    clock.advance(20);
    println!(
        "then 20 ms more: expired = {}  (no real sleeping happened)",
        dog.expired()
    );
    // ANCHOR_END: doubles
}

fn benchmark() {
    // ANCHOR: bench
    fn bench<T>(label: &str, runs: u32, mut f: impl FnMut() -> T) {
        let mut best = Duration::MAX;
        for _ in 0..runs {
            let start = Instant::now();
            black_box(f()); // keep the result "used"
            best = best.min(start.elapsed());
        }
        // Absolute numbers depend on the machine and build mode; print the shape.
        println!(
            "{label:<22} best of {runs}: {}",
            if best < Duration::from_millis(50) {
                "< 50 ms"
            } else {
                "≥ 50 ms"
            }
        );
    }

    let data: Vec<u32> = (0..10_000).collect();
    bench("iterator sum", 20, || {
        black_box(&data).iter().map(|&x| u64::from(x)).sum::<u64>()
    });
    bench("crc32 of 10 kB", 5, || crc32(black_box(&[0xA5u8; 10_000])));
    println!("(run with --release for meaningful numbers)");
    // ANCHOR_END: bench
}
