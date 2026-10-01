//! Lesson: Variables, constants & statics.

use std::sync::LazyLock;
use std::sync::atomic::{AtomicU32, Ordering};

use crate::lesson::{Lesson, Question, Section};

pub static LESSON: Lesson = Lesson {
    id: "bindings",
    title: "Variables, constants & statics",
    summary: "let, mut, shadowing, type inference, deferred initialization, const, static, and the special pattern _.",
    source: include_str!("bindings.rs"),
    sections: &[
        Section::new(
            "let: immutable by default",
            r#"
            `let` introduces a **binding**: a name for a value. Bindings are
            immutable unless you say otherwise:

            ```compile_fail,E0384
            let speed = 50;
            speed = 60; // error: cannot assign twice to immutable variable
            ```

            The default is deliberate. When most values can't change, the few
            that can stand out, and both you and the compiler can reason about
            code locally. Immutability is also what makes sharing data between
            threads safe later on.
            "#,
        )
        .demo("let", immutable),
        Section::new(
            "mut",
            r#"
            Add `mut` to allow reassignment and mutation through the binding.
            Mutability belongs to the **binding**, not to the value: moving a
            value into a `mut` binding makes it mutable there, and moving it
            back out into a plain `let` freezes it again.
            "#,
        )
        .demo("mut", mutable),
        Section::new(
            "Type inference and annotations",
            r#"
            Rust is statically typed, but you rarely write types for locals.
            The compiler infers them from how a value is used, looking at the
            whole function and not only the line where the value is created.

            Sometimes there is genuinely not enough information, for example
            when parsing text, because `parse` can produce many types. Then
            you annotate the binding (`let n: u8 = ...`) or use the
            **turbofish** `::<>` on the call (`"7".parse::<u8>()`). Function
            signatures are never inferred: they are the contract between
            caller and callee, and are always written out.
            "#,
        )
        .demo("inference", inference),
        Section::new(
            "Shadowing",
            r#"
            You can declare a new binding with the same name as an old one. The
            new binding **shadows** the old for the rest of the scope. Unlike
            `mut`, shadowing can change the type, and the result is still
            immutable.

            Here is the spectrum from most to least restricted:

            ```text
            let x = …;         one value, forever
            let x = f(x);      a new, immutable value derived from the old (shadowing)
            let mut x = …;     the same slot, overwritten in place
            ```

            Shadowing shines for "the same thing in a new form": parsing a
            string into a number, or trimming input before using it.
            "#,
        )
        .demo("shadowing", shadowing),
        Section::new(
            "Scopes and blocks",
            r#"
            A binding lives from its `let` to the end of the enclosing `{ }`
            block. Blocks are expressions: their last expression, written
            without a semicolon, is their value. That lets you compute a value
            using temporaries that disappear as soon as the block ends.
            "#,
        )
        .demo("scopes", scopes),
        Section::new(
            "Deferred initialization",
            r#"
            You may declare a binding first and assign it later, even without
            `mut`, as long as it is assigned exactly once on every path before
            it is read. The compiler proves this; reading a possibly
            uninitialized value is a compile error, not undefined behaviour:

            ```compile_fail,E0381
            let limit: u32;
            if std::env::args().count() > 1 {
                limit = 10;
            }
            println!("{limit}"); // error: possibly-uninitialized
            ```
            "#,
        )
        .demo("deferred", deferred),
        Section::new(
            "Constants",
            r#"
            `const` declares a compile-time constant. The type annotation is
            mandatory and the value must be computable at compile time,
            including calls to `const fn` functions. By convention constants
            use `SCREAMING_SNAKE_CASE`.

            A constant has no fixed address. Conceptually its value is pasted
            in wherever you use it, so each use gets a fresh copy. Constants
            can be declared at module level or inside functions and `impl`
            blocks (associated constants, such as `u8::MAX`).
            "#,
        )
        .demo("consts", constants),
        Section::new(
            "Statics",
            r#"
            A `static` is a single value at a **fixed memory address** that
            lives for the whole program (the `'static` lifetime). Taken
            together, the options form a spectrum:

            ```text
            const          a value, copied into each use site, no address
            static         one immutable value, one address
            static Atomic* / Mutex   one address, mutated safely at run time
            static LazyLock<T>       computed on first access, then shared
            static mut     one address, mutation is `unsafe`, avoid
            ```

            `static mut` is a data race waiting to happen. In edition 2024,
            taking a reference to one is a hard error (`static_mut_refs`).
            Use an atomic, a `Mutex`, `OnceLock` or `LazyLock` instead. All of
            them give you mutation behind a shared reference, checked by the
            type system.
            "#,
        )
        .demo("statics", statics),
        Section::new(
            "Underscore: _x versus _",
            r#"
            A leading underscore (`_reading`) tells the compiler "I know this
            is unused", which silences the warning. The name is still a real
            binding and holds its value until the end of the scope.

            A bare `_` is not a binding at all. It is a pattern that matches
            and **discards**. With `let _ = make_value();` the value is dropped
            immediately. This matters for guards whose job is to live until
            the end of the scope, such as mutex locks or timers:
            `let _ = mutex.lock();` unlocks at once, while
            `let _guard = mutex.lock();` holds the lock.
            "#,
        )
        .demo("underscore", underscore),
    ],
    quiz: &[
        Question::new(
            "What is the difference between `let x = x + 1;` (shadowing) and `x += 1;` on a `let mut x`?",
            &[
                "None, they compile to different code but mean the same",
                "Shadowing creates a new binding (which may have a new type); `+=` mutates the existing one",
                "Shadowing is only allowed in nested blocks",
                "`+=` creates a copy",
            ],
            1,
            "Shadowing introduces a brand-new binding that hides the old name; the old value is untouched. `mut` overwrites the same storage.",
        ),
        Question::new(
            "Which statement about `const` is true?",
            &[
                "Its type can be inferred",
                "It has one fixed memory address",
                "Its value is computed at compile time and inlined at each use",
                "It can be mutated with unsafe",
            ],
            2,
            "Constants have a mandatory type, no fixed address, and are evaluated at compile time. `static` is the one with an address.",
        ),
        Question::new(
            "When does `let _ = Noisy;` drop the `Noisy` value?",
            &[
                "At the end of the scope",
                "Immediately",
                "Never",
                "When the program exits",
            ],
            1,
            "`_` is a pattern that does not bind, so the temporary is dropped at the end of the statement. Use `_name` to keep it alive.",
        ),
        Question::new(
            "Why does edition 2024 reject `&COUNTER` where `static mut COUNTER: u32`?",
            &[
                "Statics can't be referenced at all",
                "Any reference to a `static mut` can alias with a mutation, which is undefined behaviour",
                "Because u32 is Copy",
                "It's a style lint",
            ],
            1,
            "A shared reference promises the value won't change while it lives; any other code could mutate the `static mut` meanwhile. Use atomics, `Mutex`, `OnceLock` or `LazyLock`.",
        ),
        Question::new(
            "`let n: u32; if cond { n = 1; }` then `println!(\"{n}\")` does what?",
            &[
                "Prints 0 if cond is false",
                "Fails to compile: n may be uninitialized",
                "Panics at run time if cond is false",
                "Is undefined behaviour",
            ],
            1,
            "Definite-initialization analysis rejects any read that could see an unassigned binding (error E0381).",
        ),
    ],
    exercises: &[
        "Read a line of text into a `String`, then shadow it three times: trimmed (`&str`), parsed (`u32`), and doubled. Print each stage.",
        "Write a `const fn` that computes the number of seconds in N days, and use it to define `const WEEK: u64`.",
        "Make a `static` request counter with `AtomicU64` and a function `next_id()` that returns increasing ids. Call it from a loop.",
        "Create a struct with a `Drop` impl that prints its name. Experiment with `let _ = …`, `let _a = …`, and `drop(…)` to predict the drop order before you run it.",
    ],
};

fn immutable() {
    // ANCHOR: let
    let speed = 50; // type inferred as i32
    let unit = "km/h";
    println!("Cruising at {speed} {unit}");
    // speed = 60;  // ← uncommenting this is a compile error (E0384)
    // ANCHOR_END: let
}

fn mutable() {
    // ANCHOR: mut
    let mut odometer = 12_000;
    odometer += 42;
    println!("odometer: {odometer}");

    let frozen = vec![1, 2, 3]; // immutable binding
    let mut thawed = frozen; // move the Vec into a mutable binding
    thawed.push(4);
    let frozen_again = thawed; // and back into an immutable one
    println!("{frozen_again:?}");
    // ANCHOR_END: mut
}

fn inference() {
    // ANCHOR: inference
    let mut readings = Vec::new(); // Vec<?> — element type not known yet
    readings.push(3.5); //            now it's Vec<f64>

    let a: u8 = "200".parse().unwrap(); // annotate the binding...
    let b = "55".parse::<u8>().unwrap(); // ...or use the turbofish
    println!("{readings:?} {a} {b}");

    // The *use* of a value can decide its type:
    let x = 7; // would default to i32...
    let y: u64 = 1;
    let sum = x + y; // ...but this forces x to be u64
    println!("sum = {sum}");

    // Overflowing parse is an error value, not a panic:
    let too_big = "256".parse::<u8>();
    println!("parse 256 as u8 → {too_big:?}");
    // ANCHOR_END: inference
}

fn shadowing() {
    // ANCHOR: shadowing
    let input = "  42  "; //        &str with spaces
    let input = input.trim(); //    &str, trimmed
    let input: u32 = input.parse().unwrap(); // now a number!
    let input = input * 2; //       still immutable
    println!("input = {input}");

    let level = 1;
    {
        let level = "inner"; // shadows only inside this block
        println!("inside: {level}");
    }
    println!("outside: {level}"); // the outer binding was never changed
    // ANCHOR_END: shadowing
}

fn scopes() {
    // ANCHOR: scopes
    let area = {
        let width = 3; //   temporaries...
        let height = 4;
        width * height // ← no semicolon: this is the block's value
    }; //                   ...gone here
    println!("area = {area}");

    let label = {
        let n = area % 2;
        if n == 0 { "even" } else { "odd" }
    };
    println!("{area} is {label}");
    // ANCHOR_END: scopes
}

#[allow(clippy::needless_late_init)] // late initialization is the point of this demo
fn deferred() {
    // ANCHOR: deferred
    let gear: &str; // declared, not yet initialized, and not `mut`
    let rpm = 3200;
    if rpm < 1000 {
        gear = "idle";
    } else if rpm < 4000 {
        gear = "cruise";
    } else {
        gear = "redline";
    }
    // Every path assigned `gear` exactly once, so reading it is fine.
    println!("{rpm} rpm → {gear}");
    // ANCHOR_END: deferred
}

// ANCHOR: consts
const MAX_SPEED_KMH: u32 = 180;
const SECONDS_PER_HOUR: u32 = 60 * 60; // arithmetic is fine

/// `const fn` can run at compile time *and* at run time.
const fn kmh_to_mps_x1000(kmh: u32) -> u32 {
    kmh * 1000 * 1000 / SECONDS_PER_HOUR
}

const MAX_SPEED_MMPS: u32 = kmh_to_mps_x1000(MAX_SPEED_KMH); // evaluated by the compiler

fn constants() {
    println!("max speed: {MAX_SPEED_KMH} km/h = {MAX_SPEED_MMPS} mm/s");
    println!("at run time too: {}", kmh_to_mps_x1000(90));
    println!(
        "associated constants: u8::MAX = {}, i64::MIN = {}",
        u8::MAX,
        i64::MIN
    );

    const LOCAL_LIMIT: usize = 3; // constants can be local
    let buffer = [0u8; LOCAL_LIMIT]; // array lengths must be constants
    println!("buffer has {} slots", buffer.len());
}
// ANCHOR_END: consts

// ANCHOR: statics
static VEHICLE_NAME: &str = "test-bench";

// A mutable global, done safely: atomics need no `unsafe` and no lock.
static FRAMES_SEEN: AtomicU32 = AtomicU32::new(0);

// Computed on first access, exactly once, even if many threads race.
static LOOKUP: LazyLock<Vec<u32>> = LazyLock::new(|| {
    println!("  (building LOOKUP — this prints only once per process)");
    (0..5).map(|i| i * i).collect()
});

fn on_frame() -> u32 {
    // fetch_add returns the previous value.
    FRAMES_SEEN.fetch_add(1, Ordering::Relaxed) + 1
}

fn statics() {
    println!("vehicle: {VEHICLE_NAME}");
    for _ in 0..3 {
        on_frame();
    }
    println!(
        "frames seen so far: {}",
        FRAMES_SEEN.load(Ordering::Relaxed)
    );
    println!("LOOKUP[3] = {}", LOOKUP[3]);
    println!("LOOKUP[4] = {}", LOOKUP[4]); // already built
}
// ANCHOR_END: statics

// ANCHOR: underscore
struct Noisy(&'static str);

impl Drop for Noisy {
    fn drop(&mut self) {
        println!("  drop {}", self.0);
    }
}

fn underscore() {
    println!("start");
    let _ = Noisy("A (bound to _)"); //   dropped right here
    let _b = Noisy("B (bound to _b)"); // lives to end of scope
    let _unused = 5; //                   leading _ silences the warning
    println!("end of function body");
} // _b dropped here
// ANCHOR_END: underscore
