//! Lesson: Error handling.

use std::error::Error;
use std::fmt;
use std::io::{self, Read};
use std::num::ParseIntError;

use crate::lesson::{Lesson, Question, Section};

pub static LESSON: Lesson = Lesson {
    id: "errors",
    title: "Error handling",
    summary: "Panics vs Results, custom error types, Display and Error, ? with From, Box<dyn Error>, error chains, and exit codes.",
    source: include_str!("errors.rs"),
    sections: &[
        Section::new(
            "Two kinds of failure",
            r#"
            Rust separates failures you **expect** from failures that mean the
            program itself is **wrong**:

            ```text
            Result<T, E>   expected failure: bad input, missing file, timeout.
                           The caller decides what to do.
            panic!         a bug: a broken invariant, an impossible state.
                           Unwinds (or aborts) the thread.
            ```

            It's a spectrum, and where a failure sits depends on context. A
            missing config file is a `Result` in a library, but at startup an
            application might reasonably exit. Indexing out of bounds panics
            because it means the code's own logic is wrong. `get` exists for
            when out-of-range is an expected possibility.

            Results are `#[must_use]`: if you ignore one, the compiler warns.
            Errors can't be dropped by accident the way unchecked return codes
            are in C.
            "#,
        ),
        Section::new(
            "panic! and its relatives",
            r#"
            - `panic!("msg")` signals a bug.
            - `unreachable!()` documents an impossible branch, and panics if it
              ever runs.
            - `todo!()` and `unimplemented!()` are placeholders that type-check
              as anything (they return `!`).
            - `assert!`, `assert_eq!` and `assert_ne!` check invariants in
              every build.
            - `debug_assert!` is checked only in debug builds.

            By default a panic **unwinds** the thread: destructors run, the
            panic message prints through the **panic hook**, and other threads
            keep going. `catch_unwind` can stop the unwinding at a boundary.
            That's for frameworks and FFI, not for ordinary error handling.
            With `panic = "abort"` in your Cargo profile, the process stops
            immediately instead, which is common in embedded systems.
            "#,
        )
        .demo("panic", panics),
        Section::new(
            "A custom error type",
            r#"
            For a library or a subsystem, define an **enum** of the ways an
            operation can fail. Callers can then `match` and react to each
            case. Then implement:

            - `Display`, the human-readable message, all lowercase and without
              a trailing period by convention, so it composes into longer
              messages.
            - `std::error::Error`, which marks it as an error and optionally
              exposes the underlying cause through `source()`. In Rust 1.81+
              the same trait is also available as `core::error::Error`.
            "#,
        )
        .demo("custom", custom_error),
        Section::new(
            "? and From: converting errors automatically",
            r#"
            Recall that `?` calls `From::from` on the error. If your error type
            implements `From<io::Error>` and `From<ParseIntError>`, one function
            can use `?` on both, and each error is wrapped automatically.

            The trade-off is context. `From` knows **what** failed but not
            **where** or **why**. When context matters, add it explicitly with
            `map_err` before `?`.
            "#,
        )
        .demo("from", from_conversion),
        Section::new(
            "Box<dyn Error>: the flexible end of the spectrum",
            r#"
            `Box<dyn Error>` (often with `+ Send + Sync`) can hold **any**
            error type. `?` converts into it automatically, and so does a plain
            string: `Err("bad header".into())`. That makes it ideal for
            applications, scripts and tests, where errors are reported rather
            than handled case by case.

            Callers can still inspect the concrete type with
            `downcast_ref::<T>()`, but if they need to, a dedicated enum is
            usually the better design:

            ```text
            specific enum        callers match on variants   libraries, subsystems
            Box<dyn Error>       callers report the message  applications, glue code
            ```
            "#,
        )
        .demo("boxed", boxed_errors),
        Section::new(
            "Error chains",
            r#"
            High-level errors often wrap lower-level ones: "could not load
            calibration" ← "invalid number on line 3" ← "invalid digit".
            `source()` links each error to its cause, and a reporter walks the
            chain. Each layer's `Display` should describe **its own** level only
            and leave the details to `source()`, otherwise messages repeat
            themselves.
            "#,
        )
        .demo("chain", error_chain),
        Section::new(
            "Errors at the top: main and exit codes",
            r#"
            `main` can return `Result<(), E>` where `E: Debug`. On `Err`, Rust
            prints the error's `Debug` form and exits with a failure code. For
            full control, return `std::process::ExitCode`:

            ```rust,nowrap
            use std::process::ExitCode;

            fn run() -> Result<(), String> {
                Err("sensor bus not found".into())
            }

            fn main() -> ExitCode {
                match run() {
                    Ok(()) => ExitCode::SUCCESS,
                    Err(e) => {
                        eprintln!("error: {e}");
                        ExitCode::from(2)
                    }
                }
            }
            ```

            Prefer this to `std::process::exit(code)`, which ends the process
            immediately **without running destructors**, so buffered output
            might not be flushed. This program's own `main` uses `ExitCode`.
            "#,
        ),
    ],
    quiz: &[
        Question::new(
            "Which failure should normally be a `panic!` rather than a `Result`?",
            &[
                "The user typed a non-number",
                "A file is missing",
                "An internal invariant was violated (a bug)",
                "The network timed out",
            ],
            2,
            "Expected, recoverable conditions are Results. Panics are for situations that mean the program itself is wrong.",
        ),
        Question::new(
            "What does `source()` on `std::error::Error` return?",
            &[
                "The file and line of the error",
                "The underlying cause, if any",
                "The error code",
                "A backtrace",
            ],
            1,
            "`source` links an error to the lower-level error that caused it, forming a chain.",
        ),
        Question::new(
            "Why does `Err(\"oops\".into())` work in a function returning `Result<T, Box<dyn Error>>`?",
            &[
                "Strings are errors",
                "std implements From<&str> for Box<dyn Error>",
                "into() is magic",
                "It doesn't compile",
            ],
            1,
            "std provides `From<&str>` and `From<String>` for boxed errors, wrapping the message in a private error type.",
        ),
        Question::new(
            "What's a drawback of `std::process::exit(1)` compared to returning an `ExitCode` from main?",
            &[
                "It's slower",
                "Destructors don't run, so buffers may not be flushed",
                "It can't set the code",
                "It panics",
            ],
            1,
            "`exit` terminates the process on the spot. Returning from main unwinds normally first.",
        ),
        Question::new(
            "Which type is the better choice for a reusable library's errors?",
            &[
                "String",
                "Box<dyn Error>",
                "A specific enum implementing Error",
                "i32 codes",
            ],
            2,
            "An enum lets callers match on failure kinds, and stays stable and documented. Boxed errors suit applications.",
        ),
    ],
    exercises: &[
        "Write `enum DtcError { Empty, BadPrefix(char), BadDigits(ParseIntError) }` with Display, Error (with source) and From<ParseIntError>. Then write `fn parse_dtc(s: &str) -> Result<(char, u16), DtcError>` for codes like `P0301`.",
        "Write a function returning `Result<(), Box<dyn Error>>` that reads a number from stdin and divides 100 by it. Return a string error for division by zero.",
        "Write `fn report(e: &dyn Error)` that prints the whole chain, numbered, and use it on an error three levels deep.",
        "Make `main` return `ExitCode`, with a different exit code for each variant of your error enum. Check the code with `echo $?`.",
    ],
};

fn panics() {
    // ANCHOR: panic
    use std::panic;

    // Replace the default hook (which prints to stderr) for this demo.
    let previous = panic::take_hook();
    panic::set_hook(Box::new(|info| {
        let location = info.location().map(|l| l.line()).unwrap_or(0);
        println!("  [panic hook] caught a panic on line {location}");
    }));

    let result = panic::catch_unwind(|| {
        let readings: Vec<u32> = vec![1, 2, 3];
        let index = readings.len() + 2; // a bug
        readings[index] // out of bounds → panic, not a buffer over-read
    });

    panic::set_hook(previous); // restore normal behaviour

    match result {
        Ok(v) => println!("no panic: {v}"),
        Err(payload) => {
            // The payload is the panic message as `String` or `&str`.
            let msg = payload
                .downcast_ref::<String>()
                .map(String::as_str)
                .or_else(|| payload.downcast_ref::<&str>().copied())
                .unwrap_or("<non-string payload>");
            println!("recovered from: {msg}");
        }
    }

    let temps = [88.0, 90.5, 87.25];
    assert!(
        temps.iter().all(|t: &f64| t.is_finite()),
        "sensor returned NaN/inf"
    );
    debug_assert_eq!(temps.len(), 3, "expected one reading per cylinder bank"); // debug only
    println!("assertions passed");
    // ANCHOR_END: panic
}

// ANCHOR: custom
#[derive(Debug)]
enum ConfigError {
    Missing(&'static str),
    NotANumber {
        key: &'static str,
        source: ParseIntError,
    },
    OutOfRange {
        key: &'static str,
        value: u32,
        max: u32,
    },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Missing(key) => write!(f, "missing setting `{key}`"),
            ConfigError::NotANumber { key, .. } => write!(f, "setting `{key}` is not a number"),
            ConfigError::OutOfRange { key, value, max } => {
                write!(f, "setting `{key}` = {value} exceeds the maximum {max}")
            }
        }
    }
}

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ConfigError::NotANumber { source, .. } => Some(source),
            _ => None,
        }
    }
}

fn read_limit(settings: &[(&str, &str)], key: &'static str, max: u32) -> Result<u32, ConfigError> {
    let (_, raw) = settings
        .iter()
        .find(|(k, _)| *k == key)
        .ok_or(ConfigError::Missing(key))?;
    let value: u32 = raw
        .parse()
        .map_err(|source| ConfigError::NotANumber { key, source })?;
    if value > max {
        return Err(ConfigError::OutOfRange { key, value, max });
    }
    Ok(value)
}

fn custom_error() {
    let settings = [("speed_limit", "300"), ("rpm_limit", "fast")];
    for key in ["speed_limit", "rpm_limit", "torque_limit"] {
        match read_limit(&settings, key, 250) {
            Ok(v) => println!("{key} = {v}"),
            Err(ConfigError::Missing(k)) => println!("{k}: using default"), // react to one kind
            Err(e) => println!(
                "error: {e} (source: {:?})",
                e.source().map(|s| s.to_string())
            ),
        }
    }
}
// ANCHOR_END: custom

fn from_conversion() {
    // ANCHOR: from
    #[derive(Debug)]
    enum LoadError {
        Io(io::Error),
        Parse(ParseIntError),
    }
    impl From<io::Error> for LoadError {
        fn from(e: io::Error) -> Self {
            LoadError::Io(e)
        }
    }
    impl From<ParseIntError> for LoadError {
        fn from(e: ParseIntError) -> Self {
            LoadError::Parse(e)
        }
    }

    // Generic over any reader: a file, a socket, or bytes in memory.
    fn load_count(mut source: impl Read) -> Result<u32, LoadError> {
        let mut text = String::new();
        source.read_to_string(&mut text)?; //  io::Error → LoadError::Io
        let count = text.trim().parse()?; //   ParseIntError → LoadError::Parse
        Ok(count)
    }

    let inputs: [&[u8]; 3] = [b"  42\n", b"forty-two", &[0xFF, 0xFE]]; // &[u8] implements Read
    for input in inputs {
        match load_count(input) {
            Ok(n) => println!("count = {n}"),
            Err(LoadError::Io(e)) => println!("I/O error ({:?}): {e}", e.kind()),
            Err(LoadError::Parse(e)) => println!("parse error: {e}"),
        }
    }
    // ANCHOR_END: from
}

fn boxed_errors() {
    // ANCHOR: boxed
    fn parse_frame(line: &str) -> Result<(u16, Vec<u8>), Box<dyn Error + Send + Sync>> {
        let (id, data) = line.split_once('#').ok_or("missing '#' separator")?;
        let id = u16::from_str_radix(id, 16)?; // ParseIntError, boxed by ?
        if data.len() % 2 != 0 {
            return Err(format!("odd number of hex digits in {data:?}").into());
        }
        let bytes = (0..data.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&data[i..i + 2], 16))
            .collect::<Result<Vec<u8>, _>>()?;
        Ok((id, bytes))
    }

    for line in ["7E8#0241", "7E8-0241", "XYZ#00", "7E8#024"] {
        match parse_frame(line) {
            Ok((id, data)) => println!("{line:<10} → id {id:#x} data {data:02x?}"),
            Err(e) => {
                let kind = if e.downcast_ref::<ParseIntError>().is_some() {
                    "parse"
                } else {
                    "format"
                };
                println!("{line:<10} → {kind} error: {e}");
            }
        }
    }
    // ANCHOR_END: boxed
}

fn error_chain() {
    // ANCHOR: chain
    #[derive(Debug)]
    struct CalibrationError {
        file: &'static str,
        cause: ConfigError,
    }
    impl fmt::Display for CalibrationError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "could not load calibration from {}", self.file) // own level only
        }
    }
    impl Error for CalibrationError {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            Some(&self.cause)
        }
    }

    fn report(err: &dyn Error) {
        println!("error: {err}");
        let mut cause = err.source();
        while let Some(c) = cause {
            println!("  caused by: {c}");
            cause = c.source();
        }
    }

    let settings = [("max_boost", "1.8bar")];
    if let Err(cause) = read_limit(&settings, "max_boost", 3) {
        report(&CalibrationError {
            file: "turbo.cal",
            cause,
        });
    }
    // ANCHOR_END: chain
}
