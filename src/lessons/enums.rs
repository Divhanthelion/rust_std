//! Lesson: Enums & match.

use crate::lesson::{Lesson, Question, Section};

pub static LESSON: Lesson = Lesson {
    id: "enums",
    title: "Enums & match",
    summary: "Sum types with data, exhaustive matching, methods on enums, discriminants, recursive enums, and enum sizes.",
    source: include_str!("enums.rs"),
    sections: &[
        Section::new(
            "Enums carry data",
            r#"
            An enum is a type whose value is **one of** several variants. Unlike
            C enums, each variant can carry its own data: nothing, a tuple, or
            named fields. A struct is an "and" of fields; an enum is an "or" of
            variants. Together they are called **algebraic data types**.

            Each variant name is also a constructor: `Command::SetSpeed(80)`
            builds a value.
            "#,
        )
        .demo("define", define),
        Section::new(
            "match",
            r#"
            `match` compares a value against patterns from top to bottom and
            runs the first arm that fits. Patterns can **bind** the data inside
            a variant to names. Like `if`, `match` is an expression, so every
            arm must produce the same type.
            "#,
        )
        .demo("match", match_demo),
        Section::new(
            "Exhaustiveness is a feature",
            r#"
            A `match` must cover every possible value, or it doesn't compile:

            ```compile_fail,E0004
            enum Gear { Park, Reverse, Neutral, Drive }
            fn describe(g: Gear) -> &'static str {
                match g {
                    Gear::Park => "P",
                    Gear::Drive => "D",
                } // error: patterns `Gear::Reverse` and `Gear::Neutral` not covered
            }
            ```

            So when you add a variant, the compiler lists every `match` in the
            codebase that must now handle it. Be wary of a catch-all `_` arm:
            it silences that help. Prefer listing the variants when the set is
            yours. Library authors can mark enums `#[non_exhaustive]` to
            force downstream code to include a `_` arm, which leaves room to
            add variants later.
            "#,
        )
        .demo("exhaustive", exhaustive),
        Section::new(
            "Methods and state machines",
            r#"
            Enums get `impl` blocks like structs. A method that takes `self`
            and returns `Self` is a natural **state transition**. Because each
            state is a variant, data that only makes sense in one state lives
            in that variant, and you can't read a `Charging` power level while
            `Idle`.
            "#,
        )
        .demo("states", states),
        Section::new(
            "Discriminants and integer conversion",
            r#"
            Field-less enums can have explicit **discriminants** and a fixed
            representation, `#[repr(u8)]`. Casting to an integer with `as` is
            always fine. Going **back** from an integer is fallible, because
            not every byte is a valid variant. Rust doesn't generate that
            conversion; you write a `TryFrom` impl, which makes the failure
            case explicit. Decoding protocol fields works exactly this way.
            "#,
        )
        .demo("discriminants", discriminants),
        Section::new(
            "Option is just an enum",
            r#"
            The standard library's `Option` is an ordinary enum:

            ```rust
            enum Option<T> {
                None,
                Some(T),
            }
            ```

            The prelude imports it with its variants, which is why you write
            `Some(5)` instead of `Option::Some(5)`. `Result<T, E>` is the same
            idea with `Ok(T)` and `Err(E)`. Lesson 13 covers both in depth.
            "#,
        )
        .demo("option", option_enum),
        Section::new(
            "Recursive enums need indirection",
            r#"
            An enum that contains itself would have infinite size:

            ```compile_fail,E0072
            enum Expr {
                Num(i64),
                Add(Expr, Expr), // error: recursive type has infinite size
            }
            ```

            Put the recursion behind a pointer with a known size, such as
            `Box<Expr>`. Then each `Add` holds two pointers, and the tree lives
            on the heap.
            "#,
        )
        .demo("recursive", recursive),
        Section::new(
            "Enum size, and enums vs trait objects",
            r#"
            An enum is as big as its **largest variant plus a tag**, rounded up
            for alignment. The tag is often free, because the compiler hides it
            in a **niche**: an invalid bit pattern of one of the fields.
            `Option<&T>` uses the null pointer, so it's the same size as `&T`.
            When one variant is much larger than the rest, box its payload to
            keep the whole enum small.

            Enums and trait objects (Lesson 16) form a spectrum of extensibility:

            ```text
            enum         closed set of variants, open set of operations (add a fn,
                         match on it); exhaustive checks; no allocation
            dyn Trait    open set of types (anyone can implement), closed set of
                         operations (the trait's methods); needs indirection
            ```
            "#,
        )
        .demo("sizes", sizes),
    ],
    quiz: &[
        Question::new(
            "What happens if a `match` on an enum doesn't handle one variant?",
            &[
                "It panics at run time if that variant shows up",
                "It's a compile error (E0004)",
                "The arm is silently skipped",
                "It returns ()",
            ],
            1,
            "Matches must be exhaustive. That's how adding a variant reveals every place that needs updating.",
        ),
        Question::new(
            "Why does `enum List { Cons(i32, List), Nil }` fail to compile?",
            &[
                "Enums can't be generic",
                "It has infinite size; use Box<List>",
                "Nil needs data",
                "Cons needs named fields",
            ],
            1,
            "A value would have to contain itself. `Box<List>` has a fixed size (one pointer), which breaks the cycle.",
        ),
        Question::new(
            "Given `#[repr(u8)] enum Gear { P = 0, R = 1 }`, how do you turn a `u8` into a `Gear`?",
            &[
                "`5 as Gear`",
                "`Gear::from(5)` — always generated",
                "Write a `TryFrom<u8>` impl",
                "`unsafe { transmute }` is the only way",
            ],
            2,
            "Integer to enum is fallible and not generated for you. A `TryFrom` impl with a match handles invalid values safely.",
        ),
        Question::new(
            "How big is `Option<&u64>` on a 64-bit machine?",
            &["16 bytes", "9 bytes", "8 bytes", "1 byte"],
            2,
            "References are never null, so `None` is encoded as the null pointer: a niche optimization with no extra tag.",
        ),
        Question::new(
            "When is an enum a better fit than a trait object?",
            &[
                "When third parties must add new types",
                "When the set of variants is known and you want exhaustive matching",
                "When the variants are very large",
                "Never",
            ],
            1,
            "Enums give a closed, exhaustively-checked set without allocation. Trait objects allow open extension.",
        ),
    ],
    exercises: &[
        "Model `enum Shape { Circle { r: f64 }, Rect { w: f64, h: f64 }, Triangle { a: f64, b: f64, c: f64 } }` with an `area` method (Heron's formula for the triangle).",
        "Write `#[repr(u8)] enum DoorState { Closed = 0, Open = 1, Ajar = 2, Fault = 0xFF }` and a `TryFrom<u8>` impl. Test it with every byte 0..=255 and count the valid ones.",
        "Extend the `Expr` evaluator with `Mul`, `Neg` and `Div`. Make `eval` return `Option<i64>` so that division by zero gives `None`.",
        "Model a turn-signal state machine (`Off`, `Left { blinks: u32 }`, `Right { blinks: u32 }`, `Hazard`) with a `tick(self) -> Self` transition.",
    ],
};

// ANCHOR: define
#[derive(Debug)]
enum Command {
    Stop,                                // no data
    SetSpeed(u32),                       // tuple-like
    Turn { degrees: i16, signal: bool }, // struct-like
    Display(String),                     // owns heap data
}
// ANCHOR_END: define

fn define() {
    let commands = [
        Command::Stop,
        Command::SetSpeed(80),
        Command::Turn {
            degrees: -15,
            signal: true,
        },
        Command::Display(String::from("Hello")),
    ];
    for c in &commands {
        println!("{c:?}");
    }
}

fn match_demo() {
    // ANCHOR: match
    fn execute(cmd: &Command) -> String {
        match cmd {
            Command::Stop => "braking to a halt".to_string(),
            Command::SetSpeed(kmh) => format!("cruise control at {kmh} km/h"),
            Command::Turn {
                degrees,
                signal: true,
            } => format!("signalling, turning {degrees}°"),
            Command::Turn {
                degrees,
                signal: false,
            } => format!("turning {degrees}° (no signal!)"),
            Command::Display(text) => format!("showing {text:?}"),
        }
    }

    let cmds = [
        Command::SetSpeed(100),
        Command::Turn {
            degrees: 30,
            signal: false,
        },
        Command::Stop,
    ];
    for c in &cmds {
        println!("{}", execute(c));
    }
    // ANCHOR_END: match
}

fn exhaustive() {
    // ANCHOR: exhaustive
    #[derive(Debug, Clone, Copy)]
    enum Gear {
        Park,
        Reverse,
        Neutral,
        Drive,
    }

    fn letter(g: Gear) -> char {
        match g {
            Gear::Park => 'P',
            Gear::Reverse => 'R',
            Gear::Neutral => 'N',
            Gear::Drive => 'D',
            // No `_ => …` arm: if someone adds `Gear::Sport`, this stops
            // compiling until it's handled. That's what we want.
        }
    }

    fn may_move(g: Gear) -> bool {
        matches!(g, Gear::Reverse | Gear::Drive) // grouping variants
    }

    for g in [Gear::Park, Gear::Reverse, Gear::Neutral, Gear::Drive] {
        println!("{g:?} = {} (moves: {})", letter(g), may_move(g));
    }
    // ANCHOR_END: exhaustive
}

fn states() {
    // ANCHOR: states
    #[derive(Debug)]
    enum Charger {
        Idle,
        Charging { kw: f32, delivered_kwh: f32 },
        Fault(String),
    }

    impl Charger {
        fn plug_in(self, kw: f32) -> Charger {
            match self {
                Charger::Idle => Charger::Charging {
                    kw,
                    delivered_kwh: 0.0,
                },
                other => other, // plugging in again changes nothing
            }
        }
        fn tick(self, hours: f32) -> Charger {
            match self {
                Charger::Charging { kw, delivered_kwh } if kw > 350.0 => {
                    Charger::Fault(format!("{kw} kW exceeds rating after {delivered_kwh} kWh"))
                }
                Charger::Charging { kw, delivered_kwh } => Charger::Charging {
                    kw,
                    delivered_kwh: delivered_kwh + kw * hours,
                },
                other => other,
            }
        }
    }

    let c = Charger::Idle.plug_in(50.0).tick(0.5).tick(0.25);
    println!("{c:?}");
    if let Charger::Fault(reason) = Charger::Idle.plug_in(400.0).tick(0.1) {
        println!("fault: {reason}");
    }
    // ANCHOR_END: states
}

fn discriminants() {
    // ANCHOR: discriminants
    #[derive(Debug, Clone, Copy, PartialEq)]
    #[repr(u8)]
    enum Gear {
        Park = 0x00,
        Reverse = 0x01,
        Neutral = 0x02,
        Drive = 0x03,
        Sport = 0x10,
    }

    impl TryFrom<u8> for Gear {
        type Error = u8; // hand back the bad byte

        fn try_from(byte: u8) -> Result<Self, Self::Error> {
            match byte {
                0x00 => Ok(Gear::Park),
                0x01 => Ok(Gear::Reverse),
                0x02 => Ok(Gear::Neutral),
                0x03 => Ok(Gear::Drive),
                0x10 => Ok(Gear::Sport),
                other => Err(other),
            }
        }
    }

    println!("Drive as u8 = {:#04x}", Gear::Drive as u8);
    for byte in [0x03, 0x10, 0x07] {
        println!("{byte:#04x} → {:?}", Gear::try_from(byte));
    }
    println!("size_of::<Gear>() = {}", std::mem::size_of::<Gear>());
    // ANCHOR_END: discriminants
}

fn option_enum() {
    // ANCHOR: option
    fn find_ecu(id: u16) -> Option<&'static str> {
        match id {
            0x7E0 => Some("engine"),
            0x7E1 => Some("transmission"),
            _ => None,
        }
    }
    for id in [0x7E0, 0x123] {
        match find_ecu(id) {
            Some(name) => println!("{id:#x} is the {name} ECU"),
            None => println!("{id:#x} is unknown"),
        }
    }
    // ANCHOR_END: option
}

fn recursive() {
    // ANCHOR: recursive
    #[derive(Debug)]
    enum Expr {
        Num(i64),
        Add(Box<Expr>, Box<Expr>),
        Mul(Box<Expr>, Box<Expr>),
    }
    use Expr::*; // bring the variants into scope

    fn eval(e: &Expr) -> i64 {
        match e {
            Num(n) => *n,
            Add(a, b) => eval(a) + eval(b),
            Mul(a, b) => eval(a) * eval(b),
        }
    }

    // (2 + 3) * 4
    let expr = Mul(
        Box::new(Add(Box::new(Num(2)), Box::new(Num(3)))),
        Box::new(Num(4)),
    );
    println!("{expr:?}");
    println!("= {}", eval(&expr));
    // ANCHOR_END: recursive
}

fn sizes() {
    // ANCHOR: sizes
    use std::mem::size_of;

    #[allow(dead_code)]
    enum Small {
        A(u8),
        B(u16),
    }
    #[allow(dead_code)]
    #[allow(clippy::large_enum_variant)] // clippy flags exactly this problem
    enum Lopsided {
        Tiny(u8),
        Huge([u8; 1024]),
    }
    #[allow(dead_code)]
    enum Boxed {
        Tiny(u8),
        Huge(Box<[u8; 1024]>),
    }

    println!(
        "Small        = {} bytes (u16 + tag, aligned)",
        size_of::<Small>()
    );
    println!(
        "Lopsided     = {} bytes (every value pays for Huge)",
        size_of::<Lopsided>()
    );
    println!(
        "Boxed        = {} bytes (Huge lives on the heap)",
        size_of::<Boxed>()
    );
    println!("&u64         = {}", size_of::<&u64>());
    println!(
        "Option<&u64> = {} (niche: null means None)",
        size_of::<Option<&u64>>()
    );
    println!(
        "Option<u64>  = {} (no niche: needs a tag)",
        size_of::<Option<u64>>()
    );
    println!(
        "Option<bool> = {} (bool has 254 spare values)",
        size_of::<Option<bool>>()
    );
    // ANCHOR_END: sizes
}
