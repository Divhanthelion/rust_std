//! Lesson: A tour of std — time, mem, cmp, hash, any, num, ops, hint.

use std::any::{Any, TypeId};
use std::cmp::{self, Ordering, Reverse};
use std::hash::{BuildHasher, DefaultHasher, Hash, Hasher, RandomState};
use std::mem;
use std::num::{FpCategory, IntErrorKind, NonZero};
use std::ops::{Bound, RangeBounds, RangeInclusive};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::lesson::{Lesson, Question, Section};

pub static LESSON: Lesson = Lesson {
    id: "std_tour",
    title: "A tour of std: time, mem, cmp, hash, any",
    summary: "The smaller modules you reach for every week: std::time, std::mem, std::cmp, std::hash, std::any, std::num, std::ops ranges and std::hint.",
    source: include_str!("std_tour.rs"),
    sections: &[
        Section::new(
            "std::time: two clocks for two jobs",
            r#"
            Time in std sits on a spectrum of guarantees:

            ```text
            Instant      monotonic: never goes backwards; for measuring elapsed
                         time, timeouts, deadlines. Has no meaning as a date.
            SystemTime   wall-clock: can jump (NTP, DST, the user): for
                         timestamps and logs. duration_since can fail!
            Duration     a span: seconds + nanoseconds, checked arithmetic
            ```

            Never measure intervals with `SystemTime`. A clock adjustment
            can make `elapsed()` fail, or report hours that never happened. For
            control loops and watchdogs, always use `Instant`.
            "#,
        )
        .demo("time", time),
        Section::new(
            "std::mem: sizes and moving values around",
            r#"
            - `size_of`, `align_of` and `size_of_val` describe memory layout.
            - `swap`, `replace` and `take` move values in and out of `&mut`
              places (Lesson 6).
            - `drop` destroys a value now, and `forget` skips its destructor,
              leaking it.
            - `discriminant` compares enum variants while ignoring their data.
            - `needs_drop::<T>()` tells generic code whether `T` has
              destructor work.
            "#,
        )
        .demo("mem", mem_demo),
        Section::new(
            "std::cmp: ordering helpers",
            r#"
            `cmp::min`, `cmp::max`, `min_by_key` and `max_by_key` work on any
            `Ord` type, and `clamp` bounds a value. An `Ordering` can be
            combined (`then`, `then_with`), inverted (`reverse`) and queried
            (`is_lt`, `is_ge`). Tuples and arrays compare
            **lexicographically**, so sorting by `(priority, timestamp)` just
            works. `Reverse` flips an order without writing a comparator.
            "#,
        )
        .demo("cmp", cmp_demo),
        Section::new(
            "std::hash: hashing by hand",
            r#"
            `Hash` feeds a value into a `Hasher`, and `finish()` produces a
            `u64`. `DefaultHasher::new()` is deterministic within one build of
            your program, but the algorithm may change between Rust releases,
            so **never persist these hashes** to disk or the network. Use a
            specified algorithm, such as CRC32 (Lesson 31), for that.

            `HashMap` uses `RandomState`, which seeds a fresh random key per map.
            "#,
        )
        .demo("hash", hash_demo),
        Section::new(
            "std::any: run-time type information",
            r#"
            `type_name::<T>()` returns a readable name, meant for debugging
            only, since its exact output isn't guaranteed. `TypeId::of::<T>()` is
            a comparable identity for `'static` types. `dyn Any` lets you store
            values of unknown type and **downcast** them back with
            `downcast_ref::<T>()`, which returns `None` on a mismatch. It's the
            mechanism behind panic payloads (Lesson 14). Prefer enums or traits
            when you know the set of types in advance.
            "#,
        )
        .demo("any", any_demo),
        Section::new(
            "std::num: number utilities",
            r#"
            Beyond the integer and float methods from Lesson 3, `std::num` has:

            - `NonZero<T>`, with its niche optimization;
            - `Wrapping<T>` and `Saturating<T>`, which change what operators do
              on overflow;
            - `ParseIntError` and `ParseFloatError`, with `IntErrorKind` for
              precise handling;
            - `FpCategory`, from `f64::classify()`, which tells normal,
              subnormal, zero, infinite and NaN apart.
            "#,
        )
        .demo("num", num_demo),
        Section::new(
            "std::ops: ranges are values",
            r#"
            `a..b`, `a..=b`, `..b` and `a..` are ordinary values of types
            `Range`, `RangeInclusive`, `RangeTo` and `RangeFrom`. Store them,
            pass them, and ask `contains(&x)`, which reads better than two
            comparisons. Functions can accept **any** range through the
            `RangeBounds` trait, which is how `Vec::drain(..)` and
            `BTreeMap::range` work.
            "#,
        )
        .demo("ranges", ranges_demo),
        Section::new(
            "std::hint and std::process odds and ends",
            r#"
            - `hint::black_box(x)` hides a value from the optimizer, so
              micro-benchmarks don't optimize away the work they measure.
            - `hint::spin_loop()` tells the CPU you're busy-waiting (Lesson 24).
            - `process::id()` returns the OS process id.
            - `process::exit(code)` ends the process **without running
              destructors**, and `process::abort()` ends it abnormally.

            `black_box` is the one you'll actually use, every time you measure
            performance without a benchmarking crate.
            "#,
        )
        .demo("hint", hint_demo),
    ],
    quiz: &[
        Question::new(
            "Which clock should a watchdog use to detect a 100 ms timeout?",
            &["SystemTime", "Instant", "UNIX_EPOCH", "Either"],
            1,
            "Instant is monotonic. SystemTime can jump when the wall clock is adjusted, causing false or missed timeouts.",
        ),
        Question::new(
            "Is it safe to store `DefaultHasher` outputs in a database and compare them next year?",
            &[
                "Yes",
                "No — the algorithm may change between Rust releases",
                "Only on Linux",
                "Only for strings",
            ],
            1,
            "DefaultHasher's algorithm is unspecified. Persisted hashes need a fixed, documented algorithm.",
        ),
        Question::new(
            "What does `mem::discriminant(&a) == mem::discriminant(&b)` test?",
            &[
                "Whether a and b are equal",
                "Whether a and b are the same enum variant, ignoring their data",
                "Whether they have the same size",
                "Whether both are Some",
            ],
            1,
            "Discriminants identify the variant only.",
        ),
        Question::new(
            "How do you sort tasks by priority descending, then by id ascending, with a key?",
            &[
                "sort_by_key(|t| (t.priority, t.id))",
                "sort_by_key(|t| (Reverse(t.priority), t.id))",
                "sort_by_key(|t| Reverse((t.priority, t.id)))",
                "It needs a custom Ord impl",
            ],
            1,
            "Tuples compare lexicographically, and Reverse flips only the first component.",
        ),
        Question::new(
            "What does `std::hint::black_box` do?",
            &[
                "Encrypts a value",
                "Prevents the optimizer from assuming anything about a value",
                "Hides a value from Debug",
                "Makes code run faster",
            ],
            1,
            "It's an identity function that the optimizer must treat as opaque, so benchmarks measure real work.",
        ),
    ],
    exercises: &[
        "Write a `Stopwatch` with `start()`, `lap() -> Duration` and `total() -> Duration` using Instant. Print laps as milliseconds with 3 decimals.",
        "Store `Box<dyn Any>` values of three types in a Vec and print only the `String`s and `u32`s using downcast_ref.",
        "Write `fn in_band(x: u32, band: impl RangeBounds<u32>) -> bool` and call it with `10..20`, `..=5` and `100..`.",
        "Benchmark summing 1..=10_000_000 with and without `black_box` in release mode. Explain the difference.",
    ],
};

fn time() {
    // ANCHOR: time
    let start = Instant::now();
    let work: u64 = (1..=200_000u64).map(|n| n % 7).sum();
    let elapsed = start.elapsed();
    println!(
        "work = {work}, took less than a second: {}",
        elapsed < Duration::from_secs(1)
    );

    let timeout = Duration::from_millis(250);
    let deadline = start + timeout;
    println!("deadline passed? {}", Instant::now() >= deadline);

    let d = Duration::from_secs(90) + Duration::from_millis(1500);
    println!(
        "{d:?} = {} s = {:.3} min",
        d.as_secs(),
        d.as_secs_f64() / 60.0
    );
    println!(
        "checked_sub underflow → {:?}",
        Duration::from_secs(1).checked_sub(Duration::from_secs(2))
    );
    println!(
        "saturating_sub → {:?}",
        Duration::from_secs(1).saturating_sub(Duration::from_secs(2))
    );

    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(since) => println!(
            "seconds since 1970: more than 1.7 billion? {}",
            since.as_secs() > 1_700_000_000
        ),
        Err(e) => println!("clock is before 1970?! {e}"),
    }
    // ANCHOR_END: time
}

fn mem_demo() {
    // ANCHOR: mem
    enum Signal {
        Speed(u32),
        Gear(char),
    }
    println!(
        "size_of Signal = {}, align_of u64 = {}",
        mem::size_of::<Signal>(),
        mem::align_of::<u64>()
    );
    let text = "héllo";
    println!("size_of_val(\"héllo\") = {} bytes", mem::size_of_val(text));

    let a = Signal::Speed(50);
    let b = Signal::Speed(90);
    let c = Signal::Gear('D');
    println!(
        "same variant a/b: {}, a/c: {}",
        mem::discriminant(&a) == mem::discriminant(&b),
        mem::discriminant(&a) == mem::discriminant(&c)
    );

    println!(
        "needs_drop u32: {}, String: {}",
        mem::needs_drop::<u32>(),
        mem::needs_drop::<String>()
    );

    let mut slot = String::from("old");
    let previous = mem::replace(&mut slot, String::from("new"));
    println!("replace: got {previous:?}, slot now {slot:?}");
    // ANCHOR_END: mem
}

fn cmp_demo() {
    // ANCHOR: cmp
    println!(
        "min {}  max {}  clamp {}",
        cmp::min(3, 9),
        cmp::max("a", "b"),
        140.clamp(0, 100)
    );
    println!("min_by_key: {:?}", cmp::min_by_key(-7i32, 5, |x| x.abs()));

    let o = 3.cmp(&7);
    println!("{o:?} reversed {:?}, is_lt {}", o.reverse(), o.is_lt());
    println!("then: {:?}", Ordering::Equal.then(Ordering::Greater));

    #[derive(Debug)]
    struct Task {
        priority: u8,
        id: u32,
    }
    let mut tasks = vec![
        Task {
            priority: 1,
            id: 30,
        },
        Task {
            priority: 3,
            id: 20,
        },
        Task {
            priority: 3,
            id: 10,
        },
    ];
    tasks.sort_by_key(|t| (Reverse(t.priority), t.id)); // priority ↓, id ↑
    println!(
        "{:?}",
        tasks.iter().map(|t| (t.priority, t.id)).collect::<Vec<_>>()
    );
    println!("tuples compare lexicographically: {}", (1, "b") < (1, "c"));
    // ANCHOR_END: cmp
}

fn hash_demo() {
    // ANCHOR: hash
    fn hash_of<T: Hash>(value: &T) -> u64 {
        let mut h = DefaultHasher::new();
        value.hash(&mut h);
        h.finish()
    }
    println!(
        "same input, same hash: {}",
        hash_of(&"P0301") == hash_of(&"P0301")
    );
    println!(
        "different input differs: {}",
        hash_of(&"P0301") != hash_of(&"P0302")
    );

    // RandomState: a fresh random key each time (what HashMap uses).
    let s1 = RandomState::new();
    let s2 = RandomState::new();
    println!(
        "two RandomStates agree? {} (almost surely not)",
        s1.hash_one("x") == s2.hash_one("x")
    );
    // ANCHOR_END: hash
}

fn any_demo() {
    // ANCHOR: any
    println!("type_name: {}", std::any::type_name::<Vec<Option<u8>>>());
    println!(
        "TypeId equal: {}",
        TypeId::of::<u32>() == TypeId::of::<u32>()
    );

    let bag: Vec<Box<dyn Any>> = vec![
        Box::new(42u32),
        Box::new(String::from("brake")),
        Box::new(2.5f64),
    ];
    for item in &bag {
        if let Some(n) = item.downcast_ref::<u32>() {
            println!("a u32: {n}");
        } else if let Some(s) = item.downcast_ref::<String>() {
            println!("a String: {s}");
        } else {
            // Pitfall: `item.type_id()` would be the TypeId of Box<dyn Any>
            // itself. Deref to the trait object to ask about the contents.
            let is_f64 = (**item).type_id() == TypeId::of::<f64>();
            println!("something else (is it an f64? {is_f64})");
        }
    }
    // ANCHOR_END: any
}

fn num_demo() {
    // ANCHOR: num
    let id: NonZero<u16> = NonZero::new(7).unwrap();
    println!("NonZero {id}, get() = {}", id.get());

    for text in ["", "70000", "-3", "x"] {
        match text.parse::<u16>() {
            Ok(v) => println!("{text:?} → {v}"),
            Err(e) => {
                let why = match e.kind() {
                    IntErrorKind::Empty => "empty",
                    IntErrorKind::PosOverflow => "too big",
                    IntErrorKind::InvalidDigit => "not a digit",
                    _ => "other",
                };
                println!("{text:?} → {why}");
            }
        }
    }

    for x in [1.0, 0.0, f64::MIN_POSITIVE / 2.0, f64::INFINITY, f64::NAN] {
        let class = match x.classify() {
            FpCategory::Normal => "normal",
            FpCategory::Subnormal => "subnormal",
            FpCategory::Zero => "zero",
            FpCategory::Infinite => "infinite",
            FpCategory::Nan => "NaN",
        };
        println!("{x:e} is {class}");
    }
    // ANCHOR_END: num
}

fn ranges_demo() {
    // ANCHOR: ranges
    let normal_rpm: RangeInclusive<u32> = 700..=6500;
    for rpm in [650, 3000, 7200] {
        println!("{rpm} rpm normal? {}", normal_rpm.contains(&rpm));
    }
    println!("bounds: {} ..= {}", normal_rpm.start(), normal_rpm.end());

    fn describe(r: impl RangeBounds<u32>) -> String {
        let lo = match r.start_bound() {
            Bound::Included(x) => format!("[{x}"),
            Bound::Excluded(x) => format!("({x}"),
            Bound::Unbounded => "(-∞".to_string(),
        };
        let hi = match r.end_bound() {
            Bound::Included(x) => format!("{x}]"),
            Bound::Excluded(x) => format!("{x})"),
            Bound::Unbounded => "∞)".to_string(),
        };
        format!("{lo}, {hi}")
    }
    println!(
        "{}  {}  {}  {}",
        describe(1..5),
        describe(1..=5),
        describe(..5),
        describe(10..)
    );
    // ANCHOR_END: ranges
}

fn hint_demo() {
    // ANCHOR: hint
    use std::hint::black_box;

    let start = Instant::now();
    let mut total = 0u64;
    for i in 0..100_000u64 {
        total = total.wrapping_add(black_box(i)); // the optimizer must do the work
    }
    let elapsed = start.elapsed();
    println!(
        "total {total}, measured {} (a real loop ran)",
        if elapsed > Duration::ZERO { "> 0" } else { "0" }
    );
    println!("process id is non-zero: {}", std::process::id() > 0);
    // ANCHOR_END: hint
}
