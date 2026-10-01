//! Lesson: Numbers, bools & chars.

use std::num::{NonZero, Saturating, Wrapping};

use crate::lesson::{Lesson, Question, Section};

pub static LESSON: Lesson = Lesson {
    id: "numbers",
    title: "Numbers, bools & chars",
    summary: "Integer and float types, overflow policies, conversions with as/From/TryFrom, parsing, bool, char and NonZero.",
    source: include_str!("numbers.rs"),
    sections: &[
        Section::new(
            "Integer types",
            r#"
            Rust's integers say their size and signedness in their name:

            ```text
            signed     i8   i16   i32   i64   i128   isize
            unsigned   u8   u16   u32   u64   u128   usize
            ```

            `isize`/`usize` are pointer-sized (64 bits on most machines today).
            `usize` is the type of lengths and indices. With no other clues, an
            integer literal is `i32`.

            There is no implicit conversion between integer types, not even
            from `u8` to `u32`. That feels strict at first. It is the reason a
            whole class of C bugs (silent truncation, sign confusion) cannot
            happen by accident.
            "#,
        )
        .demo("int_types", integer_types),
        Section::new(
            "Literals",
            r#"
            Underscores can go anywhere in a number for readability. Prefixes
            choose the base, and a suffix fixes the type: `0xFF_u8`, `1e6`
            (an `f64`), `2.5f32`. A **byte literal** `b'A'` is a `u8` holding
            the ASCII code.
            "#,
        )
        .demo("literals", literals),
        Section::new(
            "Overflow: a spectrum of policies",
            r#"
            What should `255_u8 + 1` be? Rust makes you choose, and the choices
            form a spectrum from loud to silent:

            ```text
            a + b                 debug: panic · release: wrap (by default)
            a.strict_add(b)       always panics on overflow
            a.checked_add(b)      Option: None on overflow — you must handle it
            a.overflowing_add(b)  (wrapped value, did_overflow: bool)
            a.saturating_add(b)   clamps to MIN/MAX — e.g. sensor values
            a.wrapping_add(b)     modular arithmetic — e.g. sequence counters
            ```

            Plain operators behave differently in debug and release builds.
            Debug builds check for overflow and panic. Release builds wrap by
            default, controlled by `overflow-checks` in `Cargo.toml`. In
            safety-critical code, don't rely on either: pick the explicit
            method that states your intent. `Wrapping<T>` and `Saturating<T>`
            are wrapper types that make the ordinary operators use that policy.
            "#,
        )
        .demo("overflow", overflow),
        Section::new(
            "Division, remainder and Euclid",
            r#"
            Integer division truncates toward zero, and `%` takes the sign of
            the left operand: `-7 / 2 == -3` and `-7 % 2 == -1`. For clock-style
            arithmetic, where you want a result in `0..n`, use `rem_euclid`.

            Dividing by zero always panics, in debug and release alike. So does
            `i32::MIN / -1`, whose true result doesn't fit. `checked_div`
            turns both into `None`.
            "#,
        )
        .demo("division", division),
        Section::new(
            "Bit and math helpers",
            r#"
            Integers carry a toolbox of methods that compile down to single
            CPU instructions where possible: population count, leading and
            trailing zeros, rotations, byte swaps, powers, integer logarithms,
            integer square root, and ceiling division. Reach for these rather
            than rewriting them. They are correct at the edges, such as zero,
            MAX and negative values, where hand-written versions often fail.
            "#,
        )
        .demo("int_methods", int_methods),
        Section::new(
            "Floating point",
            r#"
            `f64` (the default) and `f32` are IEEE-754 binary floating point.
            Three facts trip up everyone once:

            - Most decimals can't be represented exactly, so `0.1 + 0.2 != 0.3`.
              Compare with a tolerance.
            - `NaN` is not equal to anything, including itself. Floats are
              therefore only `PartialEq` and `PartialOrd`, not `Eq` or `Ord`, so
              `sort()` won't accept a `Vec<f64>`. Use
              `sort_by(|a, b| a.total_cmp(b))`.
            - Float arithmetic never panics. Overflow gives `inf`, and invalid
              operations give `NaN`, which then spreads silently through every
              calculation it touches.
            "#,
        )
        .demo("floats", floats),
        Section::new(
            "Conversions: From, TryFrom and as",
            r#"
            Converting between numeric types is a spectrum from "can't fail" to
            "never complains":

            ```text
            u32::from(x_u8)        lossless widening, only exists when it can't fail
            u8::try_from(x_u32)    checked: Result, Err if the value doesn't fit
            x as u8                always "succeeds": truncates, wraps, or saturates
            ```

            `as` is fast and sometimes exactly what you want, for example
            taking the low byte of a word. But it silently changes values:
            integer-to-integer `as` keeps the low bits, and float-to-integer
            `as` saturates, mapping NaN to 0. Prefer `From`/`TryFrom` and use
            `as` only where truncation is the point.
            "#,
        )
        .demo("conversions", conversions),
        Section::new(
            "Parsing numbers from text",
            r#"
            `str::parse` returns a `Result`. The error says why parsing failed:
            empty input, an invalid digit, or overflow. Note that `"-1"` is an
            invalid digit for an unsigned type, not an overflow. For other
            bases there is `from_str_radix`.
            "#,
        )
        .demo("parsing", parsing),
        Section::new(
            "bool",
            r#"
            `bool` is `true` or `false` and takes one byte. `&&` and `||`
            short-circuit. `&`, `|` and `^` also work on bools and always
            evaluate both sides. There is no "truthiness": `if 1 { … }` is a
            type error, so write `if n != 0`.

            Two handy helpers turn conditions into `Option`s:
            `cond.then_some(value)` and `cond.then(|| expensive())`.
            "#,
        )
        .demo("bool", booleans),
        Section::new(
            "char",
            r#"
            A `char` is a **Unicode scalar value**: 4 bytes, any code point
            except surrogates. It is not a byte (that's `u8`) and not a
            user-perceived character. "é" can be one `char` or two (`e` plus a
            combining accent). Strings, covered in Lesson 8, are UTF-8 and
            store chars in 1 to 4 bytes each.
            "#,
        )
        .demo("char", chars),
        Section::new(
            "NonZero: types that rule out a value",
            r#"
            `NonZero<u32>` (generic since Rust 1.79; older code uses
            `NonZeroU32`) is a `u32` that can never be zero. Because zero is
            impossible, `Option<NonZero<u32>>` can use 0 to mean `None` and
            stay 4 bytes. That trick is called a **niche**. More importantly,
            the type documents and enforces an invariant, such as a divisor or
            an identifier that must never be zero, once at construction
            instead of at every use.
            "#,
        )
        .demo("nonzero", nonzero),
    ],
    quiz: &[
        Question::new(
            "In a default release build, what is `x + 1` when `x: u8 = 255`?",
            &[
                "A panic",
                "0 (it wraps)",
                "255 (it saturates)",
                "Undefined behaviour",
            ],
            1,
            "Release builds disable overflow checks by default, so it wraps. Debug builds panic. It is never undefined behaviour. If the result matters, use `checked_add`, `wrapping_add`, or `saturating_add` to make your intent explicit.",
        ),
        Question::new(
            "What is `300_i32 as u8`?",
            &["255", "44", "A compile error", "A panic"],
            1,
            "Integer `as` keeps the low 8 bits: 300 = 0x12C, low byte 0x2C = 44. Use `u8::try_from(300)` to get an error instead.",
        ),
        Question::new(
            "Why can't you call `.sort()` on a `Vec<f64>`?",
            &[
                "Floats are too large",
                "f64 doesn't implement Ord, because NaN has no place in a total order",
                "sort only works on integers",
                "You can — it works fine",
            ],
            1,
            "`NaN` compares unequal to everything, so f64 is only `PartialOrd`. Use `sort_by(|a, b| a.total_cmp(b))`.",
        ),
        Question::new(
            "What is `(-7_i32).rem_euclid(3)`?",
            &["-1", "2", "1", "-2"],
            1,
            "`-7 % 3 == -1` (sign follows the dividend), but `rem_euclid` always returns a value in `0..3`: -7 = 3·(-3) + 2.",
        ),
        Question::new(
            "How big is `Option<NonZero<u64>>`?",
            &[
                "16 bytes",
                "9 bytes",
                "8 bytes",
                "It depends on the platform",
            ],
            2,
            "Zero is a forbidden value of NonZero, so the compiler uses it to encode `None` — a niche. No extra tag byte needed.",
        ),
        Question::new(
            "Which conversion exists as `From` (infallible)?",
            &["u32 → u8", "i32 → u32", "u8 → i16", "i64 → usize"],
            2,
            "Every u8 value fits in i16, so `i16::from(x_u8)` exists. The others can lose information (or depend on pointer width), so only `TryFrom` exists.",
        ),
    ],
    exercises: &[
        "Write `fn average(values: &[u8]) -> u8` that cannot overflow, no matter how many 255s you pass. (Hint: accumulate in a wider type, and convert back with `try_from`.)",
        "A 16-bit message counter wraps from 65535 to 0. Write `fn is_newer(a: u16, b: u16) -> bool` that works across the wrap. (Hint: `b.wrapping_sub(a)` and interpret the result.)",
        "Parse a list of strings into `u8`s, printing a different message for empty input, invalid digits, and out-of-range values. (Hint: `ParseIntError::kind()`.)",
        "Sort `vec![2.5, f64::NAN, -1.0, 0.0, -0.0]` with `total_cmp` and print the result. Where did NaN and -0.0 end up?",
    ],
};

fn integer_types() {
    // ANCHOR: int_types
    use std::mem::size_of;
    println!("type   bytes  range");
    println!(
        "i8     {}      {} ..= {}",
        size_of::<i8>(),
        i8::MIN,
        i8::MAX
    );
    println!(
        "u8     {}      {} ..= {}",
        size_of::<u8>(),
        u8::MIN,
        u8::MAX
    );
    println!(
        "i16    {}      {} ..= {}",
        size_of::<i16>(),
        i16::MIN,
        i16::MAX
    );
    println!(
        "u16    {}      {} ..= {}",
        size_of::<u16>(),
        u16::MIN,
        u16::MAX
    );
    println!(
        "i32    {}      {} ..= {}",
        size_of::<i32>(),
        i32::MIN,
        i32::MAX
    );
    println!(
        "u64    {}      {} ..= {}",
        size_of::<u64>(),
        u64::MIN,
        u64::MAX
    );
    println!("i128   {}     {} ..= …", size_of::<i128>(), i128::MIN);
    println!(
        "usize  {}      (this machine: {}-bit)",
        size_of::<usize>(),
        usize::BITS
    );

    let default = 7; // no suffix, no other clues
    println!(
        "an unconstrained literal is {}",
        std::any::type_name_of_val(&default)
    );
    // ANCHOR_END: int_types
}

fn literals() {
    // ANCHOR: literals
    let million = 1_000_000;
    let mask = 0xFF_u8; // hex, with a type suffix
    let perms = 0o755; // octal
    let flags = 0b1010_0101; // binary
    let letter = b'A'; // u8, the ASCII code 65
    let tiny = 2.5e-3; // f64
    let single = 1.5f32;
    println!("{million} {mask} {perms} {flags} {letter} {tiny} {single}");
    // ANCHOR_END: literals
}

fn overflow() {
    // ANCHOR: overflow
    let x: u8 = 250;
    println!("checked     {:?}", x.checked_add(10)); //    None
    println!("checked ok  {:?}", x.checked_add(5)); //     Some(255)
    println!("wrapping    {}", x.wrapping_add(10)); //     4
    println!("saturating  {}", x.saturating_add(10)); //   255
    println!("overflowing {:?}", x.overflowing_add(10)); // (4, true)
    println!("saturating sub {}", 3u8.saturating_sub(10)); // 0, not a panic

    // Wrapper types make the operators themselves use a policy:
    let seq = Wrapping(u16::MAX) + Wrapping(1);
    let level = Saturating(250u8) + Saturating(10);
    println!("Wrapping: {}  Saturating: {}", seq.0, level.0);

    // What does plain `+` do in *this* build?
    if cfg!(debug_assertions) {
        println!("this is a debug build: `x + 10` would panic");
    } else {
        println!("this is a release build: `x + 10` would wrap (by default)");
    }
    // ANCHOR_END: overflow
}

fn division() {
    // ANCHOR: division
    println!("-7 / 2 = {}   -7 % 2 = {}", -7 / 2, -7 % 2);
    println!(
        "div_euclid {}  rem_euclid {}",
        (-7i32).div_euclid(2),
        (-7i32).rem_euclid(2)
    );

    // An hour hand going back 5 hours from 2 o'clock:
    let hour: i32 = 2;
    println!("2 o'clock − 5h = {} o'clock", (hour - 5).rem_euclid(12));

    println!("10.checked_div(0)      = {:?}", 10i32.checked_div(0));
    println!("i32::MIN.checked_div(-1) = {:?}", i32::MIN.checked_div(-1));
    println!("i32::MIN.checked_abs()   = {:?}", i32::MIN.checked_abs());
    // ANCHOR_END: division
}

fn int_methods() {
    // ANCHOR: int_methods
    let n: u32 = 0b0000_0000_0000_0000_0000_0001_0110_1000; // 360
    println!("count_ones    {}", n.count_ones());
    println!("leading_zeros {}", n.leading_zeros());
    println!("trailing_zeros {}", n.trailing_zeros());
    println!("rotate_left(8) {:#010x}", n.rotate_left(8));
    println!("swap_bytes    {:#010x}", n.swap_bytes());
    println!("pow           {}", 3u64.pow(4));
    println!("checked_pow   {:?}", 10u32.checked_pow(10)); // None: too big
    println!("ilog2 / ilog10 {} {}", 1000u32.ilog2(), 1000u32.ilog10());
    println!("isqrt(99)     {}", 99u32.isqrt());
    println!("div_ceil      {}", 17u32.div_ceil(8)); // bytes needed for 17 bits
    println!(
        "is_power_of_two {} {}",
        64u32.is_power_of_two(),
        96u32.is_power_of_two()
    );
    println!("next_power_of_two {}", 100u32.next_power_of_two());
    println!("abs_diff      {}", 3u8.abs_diff(250));
    println!("signum / abs  {} {}", (-42i32).signum(), (-42i32).abs());
    println!(
        "min / max / clamp {} {} {}",
        4.min(9),
        4.max(9),
        120.clamp(0, 100)
    );
    // ANCHOR_END: int_methods
}

fn floats() {
    // ANCHOR: floats
    // Annotated: you can't call methods on a float whose type isn't known yet.
    let sum: f64 = 0.1 + 0.2;
    println!("0.1 + 0.2 = {sum}  (== 0.3? {})", sum == 0.3);
    println!("close enough? {}", (sum - 0.3).abs() < 1e-9);

    let nan = f64::NAN;
    println!("NaN == NaN? {}   is_nan? {}", nan == nan, nan.is_nan());
    println!("1.0 / 0.0 = {}   -1.0 / 0.0 = {}", 1.0 / 0.0, -1.0 / 0.0);
    println!("0.0 / 0.0 = {}", 0.0_f64 / 0.0);
    println!("NaN spreads: {}", (nan * 0.0 + 5.0) / 2.0);
    println!("…except through max/min, which skip it: {}", nan.max(1.0));

    let mut temps: Vec<f64> = vec![21.5, -3.0, 18.25, 0.0];
    temps.sort_by(|a, b| a.total_cmp(b));
    println!("sorted: {temps:?}");

    let x: f64 = 2.75;
    println!(
        "floor {} ceil {} round {} trunc {} fract {}",
        x.floor(),
        x.ceil(),
        x.round(),
        x.trunc(),
        x.fract()
    );
    println!(
        "sqrt {:.4}  powi {}  powf {:.4}",
        x.sqrt(),
        x.powi(2),
        x.powf(0.5)
    );
    println!("f32 precision: {}  vs f64: {}", 1.0f32 / 3.0, 1.0f64 / 3.0);
    println!("bit pattern of 1.0f32: {:#010x}", 1.0f32.to_bits());
    // ANCHOR_END: floats
}

fn conversions() {
    // ANCHOR: conversions
    let small: u8 = 200;
    let wide = u32::from(small); //   lossless: From exists
    let wide2: u64 = small.into(); // the same, spelled with Into
    println!("from: {wide} {wide2}");

    println!("try_from(300) → {:?}", u8::try_from(300_i32));
    println!("try_from(-1)  → {:?}", u32::try_from(-1_i64));
    println!("try_from(42)  → {:?}", u8::try_from(42_i32));

    // `as` never fails — but look what it does:
    println!("300 as u8      = {}", 300_i32 as u8); //   low byte: 44
    println!("-1 as u8       = {}", -1_i32 as u8); //    255
    println!("200u8 as i8    = {}", 200_u8 as i8); //    -56 (same bits)
    println!("3.99 as i32    = {}", 3.99_f64 as i32); // 3 (truncates)
    println!("-1.5 as u8     = {}", -1.5_f64 as u8); //  0 (saturates)
    println!("1e10 as i32    = {}", 1e10_f64 as i32); // i32::MAX
    println!("NaN as i32     = {}", f64::NAN as i32); //  0
    println!("'A' as u8      = {}", 'A' as u8);
    println!("u8 as char     = {}", 97u8 as char);
    // ANCHOR_END: conversions
}

fn parsing() {
    // ANCHOR: parsing
    for text in ["42", " 42", "", "-1", "256", "x7"] {
        match text.parse::<u8>() {
            Ok(n) => println!("{text:?} → Ok({n})"),
            Err(e) => println!("{text:?} → Err({:?}): {e}", e.kind()),
        }
    }
    println!("trim first: {:?}", " 42".trim().parse::<u8>());
    println!("hex 'ff'  → {:?}", u8::from_str_radix("ff", 16));
    println!("bin '101' → {:?}", u8::from_str_radix("101", 2));
    println!("float     → {:?}", "2.5e3".parse::<f64>());
    // ANCHOR_END: parsing
}

fn booleans() {
    // ANCHOR: bool
    let engine_on = true;
    let door_open = false;
    println!("can drive: {}", engine_on && !door_open);
    println!("xor: {}", engine_on ^ door_open);
    println!("as integer: {} {}", engine_on as u8, door_open as u8);

    fn expensive_check() -> bool {
        println!("  (expensive_check ran)");
        true
    }
    println!("short-circuit &&: {}", door_open && expensive_check()); // never runs
    println!("non-short &:      {}", door_open & expensive_check()); //   runs

    let speed = 72;
    let warning = (speed > 60).then_some("slow down");
    let none = (speed > 100).then(|| format!("{speed} is far too fast"));
    println!("{warning:?} {none:?}");
    // ANCHOR_END: bool
}

fn chars() {
    // ANCHOR: char
    let c = 'é';
    let crab = '🦀';
    println!("size_of::<char>() = {}", std::mem::size_of::<char>());
    println!(
        "{c} is U+{:04X}, takes {} bytes in UTF-8",
        c as u32,
        c.len_utf8()
    );
    println!(
        "{crab} is U+{:X}, takes {} bytes in UTF-8",
        crab as u32,
        crab.len_utf8()
    );

    for ch in ['7', 'x', 'Z', ' ', 'ß'] {
        println!(
            "{ch:?}: alphabetic={} numeric={} upper={} digit={:?}",
            ch.is_alphabetic(),
            ch.is_numeric(),
            ch.is_uppercase(),
            ch.to_digit(10)
        );
    }
    println!("from_digit(7)   = {:?}", char::from_digit(7, 10));
    println!("from_u32(0x41)  = {:?}", char::from_u32(0x41));
    println!(
        "from_u32(0xD800) = {:?} (a surrogate)",
        char::from_u32(0xD800)
    );
    println!("'ß' uppercased  = {}", 'ß'.to_uppercase()); // two chars: "SS"
    // ANCHOR_END: char
}

fn nonzero() {
    // ANCHOR: nonzero
    use std::mem::size_of;

    let id = NonZero::new(42u32).expect("42 is not zero");
    let bad = NonZero::new(0u32);
    println!("id = {id}, NonZero::new(0) = {bad:?}");

    // Division by a NonZero divisor cannot panic, so std offers it directly:
    let per_wheel = 1600u32 / NonZero::new(4u32).unwrap();
    println!("1600 / 4 = {per_wheel}");

    println!("size_of u32                  = {}", size_of::<u32>());
    println!(
        "size_of Option<u32>          = {}",
        size_of::<Option<u32>>()
    );
    println!(
        "size_of Option<NonZero<u32>> = {}",
        size_of::<Option<NonZero<u32>>>()
    );
    // ANCHOR_END: nonzero
}
