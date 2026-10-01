//! Lesson: Structs & methods.

use crate::lesson::{Lesson, Question, Section};

pub static LESSON: Lesson = Lesson {
    id: "structs",
    title: "Structs & methods",
    summary: "Named, tuple and unit structs; methods and associated functions; derives; invariants through privacy; builders; memory layout and repr.",
    source: include_str!("structs.rs"),
    sections: &[
        Section::new(
            "Defining and creating structs",
            r#"
            A struct groups named fields into one type. Create one by naming
            every field. Two shortcuts help:

            - **Field init shorthand**: `Point { x, y }` when local variables
              have the field names.
            - **Struct update syntax**: `Config { verbose: true, ..base }` copies
              or **moves** the remaining fields from `base`. Any non-`Copy`
              fields moved this way leave `base` partially moved.

            A struct is mutable only through a `mut` binding or a `&mut`
            reference, and then all of it is. Rust has no per-field `mut`.
            "#,
        )
        .demo("define", define),
        Section::new(
            "Methods: self, &self, &mut self",
            r#"
            Methods live in `impl` blocks. Their first parameter says how they
            use the receiver, which is the same ownership spectrum as before:

            ```text
            fn read(&self)        borrows: inspect the value
            fn change(&mut self)  borrows exclusively: modify in place
            fn finish(self)       takes ownership: consume or transform it
            ```

            `Self` is an alias for the type being implemented. When you write
            `car.drive(10)`, Rust auto-references `car` to `&mut car` as
            needed, so callers never write `(&mut car).drive(10)`.
            "#,
        )
        .demo("methods", methods),
        Section::new(
            "Associated functions and constants",
            r#"
            Functions in an `impl` block without a `self` parameter are
            **associated functions**, called with `Type::name()`. Constructors
            are associated functions by convention. `new` is the usual name;
            Rust has no special constructor syntax. Several constructors with
            descriptive names (`from_celsius`, `with_capacity`) are common.

            `impl` blocks can also hold **associated constants** like
            `Self::MAX_RPM`.
            "#,
        )
        .demo("assoc", associated),
        Section::new(
            "Tuple structs, newtypes and unit structs",
            r#"
            A **tuple struct** has unnamed fields: `struct Rgb(u8, u8, u8)`. With
            a single field it's a **newtype**: `struct Meters(f64)`. Newtypes
            cost nothing at run time, but they make unit mix-ups a compile error.
            In 1999 NASA's Mars Climate Orbiter was lost because one team's
            software produced pound-force seconds where another expected
            newton-seconds.

            A **unit struct** has no fields (`struct Marker;`). It takes zero
            bytes and is useful as a type-level tag or a trait implementor.
            "#,
        )
        .demo("newtypes", newtypes),
        Section::new(
            "Deriving common traits",
            r#"
            `#[derive(...)]` asks the compiler to implement standard traits
            field by field:

            ```text
            Debug                 {:?} formatting
            Clone, Copy           duplication (Copy only if all fields are Copy)
            PartialEq, Eq         == and != (Eq: equality is reflexive — no NaN)
            PartialOrd, Ord       <, >, sort — compares fields in declaration order
            Hash                  usable as a HashMap/HashSet key
            Default               a value with every field defaulted
            ```

            Derived orderings compare fields **in declaration order**, so field
            order is part of your type's semantics. Lesson 17 shows how to
            implement these by hand when field-by-field isn't what you want.
            "#,
        )
        .demo("derive", derives),
        Section::new(
            "Privacy protects invariants",
            r#"
            Fields are **private to their module** unless marked `pub`. That
            lets a type guarantee invariants. If the only way to make a
            `Percentage` is a constructor that checks `0..=100`, every
            `Percentage` anywhere in the program is valid. Code that receives
            one never needs to re-check.

            This is "parse, don't validate": check once at the boundary, then
            let the type carry the proof. Safety-critical Rust leans on it
            heavily.
            "#,
        )
        .demo("privacy", privacy),
        Section::new(
            "Destructuring structs",
            r#"
            Patterns can take structs apart: `let Reading { sensor, value, .. }
            = r;` binds fields by name, and `..` ignores the rest. You can
            rename (`value: v`) and destructure directly in function parameters.
            "#,
        )
        .demo("destructure", destructure),
        Section::new(
            "Builders: consuming self for fluent APIs",
            r#"
            Methods that take `self` and return `Self` allow chaining. A
            **builder** collects optional settings, then produces the final
            value in a single `build()` call that can validate everything at
            once. It's the idiomatic answer to "Rust has no default or named
            arguments".
            "#,
        )
        .demo("builder", builder),
        Section::new(
            "Memory layout and repr",
            r#"
            By default (`repr(Rust)`) the compiler may **reorder fields** to
            reduce padding, so you can't assume a layout. Attributes pin it
            down when you need to:

            ```text
            #[repr(C)]            C's rules: declaration order, C padding — for FFI and
                                  memory-mapped hardware
            #[repr(transparent)]  a single-field wrapper with exactly its field's layout
            #[repr(packed)]       no padding at all; references to its fields are
                                  rejected (they could be misaligned)
            #[repr(u8)] etc.      for enums: the size of the discriminant
            ```

            `size_of`, `align_of` and `offset_of!` (Rust 1.77+) let you inspect
            the result. The numbers below differ only by `repr`.
            "#,
        )
        .demo("layout", layout),
    ],
    quiz: &[
        Question::new(
            "Which method signature lets you modify the struct but not consume it?",
            &["fn f(self)", "fn f(&self)", "fn f(&mut self)", "fn f()"],
            2,
            "`&mut self` borrows exclusively. `self` takes ownership, `&self` only reads, and no receiver makes it an associated function.",
        ),
        Question::new(
            "What does `let b = Config { verbose: true, ..a };` do to `a`'s `String` fields?",
            &[
                "Clones them",
                "Moves them into b",
                "Borrows them",
                "Nothing — they're ignored",
            ],
            1,
            "Struct update syntax moves the remaining fields. Non-Copy fields of `a` become unusable, though fields that weren't moved stay usable.",
        ),
        Question::new(
            "Derived `PartialOrd` on `struct V { major: u32, minor: u32 }` compares…",
            &[
                "minor first",
                "major first, then minor",
                "the sum",
                "nothing; it requires Ord",
            ],
            1,
            "Derived comparisons are lexicographic in field declaration order.",
        ),
        Question::new(
            "Why is `struct Meters(f64)` useful when it costs nothing at run time?",
            &[
                "It's faster than f64",
                "It makes mixing up units a compile-time error",
                "It saves memory",
                "It allows operator overloading on f64",
            ],
            1,
            "Newtypes give distinct types to values that share a representation, so `Meters` can't be passed where `Feet` is expected.",
        ),
        Question::new(
            "Which repr guarantees declaration-order fields with C-compatible padding?",
            &["repr(Rust)", "repr(C)", "repr(transparent)", "repr(packed)"],
            1,
            "`repr(C)` follows the C ABI layout rules, which is what FFI and register maps need.",
        ),
    ],
    exercises: &[
        "Define `struct Rect { w: u32, h: u32 }` with `area`, `can_hold(&self, other: &Rect)`, and a `square(size)` constructor.",
        "Create newtypes `Celsius(f64)` and `Fahrenheit(f64)` with conversion methods. Try passing one where the other is expected and read the error.",
        "Make a `struct BatteryLevel(u8)` whose only constructor rejects values over 100, with a private field. Show that outside code can't build an invalid one.",
        "Print `size_of` and `align_of` for `(u8, u64, u8)`, the same fields in a `repr(C)` struct, and in a `repr(Rust)` struct. Explain each number.",
    ],
};

fn define() {
    // ANCHOR: define
    #[derive(Debug)]
    struct Vehicle {
        vin: String,
        model: String,
        year: u16,
        electric: bool,
    }

    let model = String::from("bZ4X");
    let year = 2025;
    let v1 = Vehicle {
        vin: String::from("JT123"),
        model, // shorthand for `model: model`
        year,
        electric: true,
    };
    println!("{v1:?}");

    let v2 = Vehicle {
        vin: String::from("JT456"),
        ..v1 // moves v1.model; copies year and electric
    };
    println!(
        "{} {} {} electric={}",
        v2.vin, v2.model, v2.year, v2.electric
    );
    println!("v1.vin is still usable: {}", v1.vin); // not moved
    // println!("{}", v1.model); // error: moved into v2

    let mut v3 = v2;
    v3.year += 1; // the whole struct is mutable through `mut`
    println!("{v3:?}");
    // ANCHOR_END: define
}

fn methods() {
    // ANCHOR: methods
    struct Car {
        odometer_km: u32,
        fuel_l: f32,
    }

    impl Car {
        fn range_km(&self) -> f32 {
            self.fuel_l * 15.0 // reads only
        }
        fn drive(&mut self, km: u32) {
            self.odometer_km += km; // modifies
            self.fuel_l -= km as f32 / 15.0;
        }
        fn scrap(self) -> u32 {
            self.odometer_km // consumes the car; returns final mileage
        }
    }

    let mut car = Car {
        odometer_km: 0,
        fuel_l: 40.0,
    };
    println!("range: {:.0} km", car.range_km());
    car.drive(150); // auto-ref: (&mut car).drive(150)
    println!("after 150 km: range {:.0} km", car.range_km());
    let final_km = car.scrap();
    println!("scrapped at {final_km} km");
    // car.drive(1); // error: `car` was moved by scrap()
    // ANCHOR_END: methods
}

fn associated() {
    // ANCHOR: assoc
    #[derive(Debug)]
    struct Engine {
        rpm: u32,
    }

    impl Engine {
        const IDLE_RPM: u32 = 750;
        const MAX_RPM: u32 = 6500;

        fn new() -> Self {
            Engine {
                rpm: Self::IDLE_RPM,
            }
        }

        fn at(rpm: u32) -> Self {
            Engine {
                rpm: rpm.min(Self::MAX_RPM),
            }
        }
    }

    println!("{:?}", Engine::new());
    println!("rpm = {}", Engine::at(9000).rpm); // clamped
    println!("redline is {}", Engine::MAX_RPM);
    // ANCHOR_END: assoc
}

fn newtypes() {
    // ANCHOR: newtypes
    #[derive(Debug, Clone, Copy)]
    struct NewtonSeconds(f64);
    #[derive(Debug, Clone, Copy)]
    struct PoundForceSeconds(f64);

    impl PoundForceSeconds {
        fn to_si(self) -> NewtonSeconds {
            NewtonSeconds(self.0 * 4.448_222)
        }
    }

    fn apply_impulse(impulse: NewtonSeconds) {
        println!("applying {:.2} N·s", impulse.0);
    }

    let from_contractor = PoundForceSeconds(10.0);
    // apply_impulse(from_contractor); // error: expected NewtonSeconds
    apply_impulse(from_contractor.to_si());

    struct Rgb(u8, u8, u8); // tuple struct
    let toyota_red = Rgb(235, 10, 30);
    println!("rgb({}, {}, {})", toyota_red.0, toyota_red.1, toyota_red.2);

    struct Marker; // unit struct
    println!("size_of Marker = {}", std::mem::size_of::<Marker>());
    println!(
        "size_of NewtonSeconds = {}",
        std::mem::size_of::<NewtonSeconds>()
    );
    let _ = Marker;
    // ANCHOR_END: newtypes
}

fn derives() {
    // ANCHOR: derive
    use std::collections::HashSet;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
    struct Version {
        major: u16,
        minor: u16,
        patch: u16,
    }

    let a = Version {
        major: 1,
        minor: 4,
        patch: 2,
    };
    let b = Version {
        major: 1,
        minor: 10,
        patch: 0,
    };
    println!("a < b? {}  (minor 4 < 10; patch never consulted)", a < b);
    println!("a == a.clone()? {}", a == a.clone());
    println!("default: {:?}", Version::default());

    let mut versions = vec![
        b,
        a,
        Version {
            major: 0,
            minor: 9,
            patch: 9,
        },
    ];
    versions.sort(); // needs Ord
    println!("sorted: {versions:?}");

    let unique: HashSet<Version> = versions.iter().copied().collect(); // needs Hash + Eq
    println!("{} unique versions", unique.len());
    // ANCHOR_END: derive
}

fn privacy() {
    // ANCHOR: privacy
    mod units {
        /// A value guaranteed to be within 0..=100.
        #[derive(Debug, Clone, Copy)]
        pub struct Percentage(u8); // the field is private to this module

        impl Percentage {
            pub fn new(value: u8) -> Result<Self, String> {
                if value <= 100 {
                    Ok(Percentage(value))
                } else {
                    Err(format!("{value} is not a percentage"))
                }
            }
            pub fn get(self) -> u8 {
                self.0
            }
        }
    }
    use units::Percentage;

    let charge = Percentage::new(87).unwrap();
    println!("charge {}%", charge.get());
    println!("new(140) → {:?}", Percentage::new(140));
    // let forged = Percentage(140); // error: field is private

    fn needs_no_check(p: Percentage) -> &'static str {
        // No range check needed: the type guarantees it.
        if p.get() > 20 { "ok" } else { "low" }
    }
    println!("battery is {}", needs_no_check(charge));
    // ANCHOR_END: privacy
}

fn destructure() {
    // ANCHOR: destructure
    struct Reading {
        sensor: &'static str,
        value: f64,
        timestamp_ms: u64,
    }

    let r = Reading {
        sensor: "intake_temp",
        value: 31.5,
        timestamp_ms: 1_000,
    };
    let Reading {
        sensor, value: v, ..
    } = r; // rename `value` to `v`, skip the rest
    println!("{sensor} = {v}");

    fn age_ms(&Reading { timestamp_ms, .. }: &Reading, now: u64) -> u64 {
        now - timestamp_ms // destructured right in the parameter
    }
    println!("age: {} ms", age_ms(&r, 1_250));
    // ANCHOR_END: destructure
}

fn builder() {
    // ANCHOR: builder
    #[derive(Debug)]
    struct CanConfig {
        bitrate: u32,
        loopback: bool,
        filters: Vec<u32>,
    }

    #[derive(Default)]
    struct CanConfigBuilder {
        bitrate: Option<u32>,
        loopback: bool,
        filters: Vec<u32>,
    }

    impl CanConfigBuilder {
        fn bitrate(mut self, bps: u32) -> Self {
            self.bitrate = Some(bps);
            self
        }
        fn loopback(mut self, on: bool) -> Self {
            self.loopback = on;
            self
        }
        fn filter(mut self, id: u32) -> Self {
            self.filters.push(id);
            self
        }
        fn build(self) -> Result<CanConfig, String> {
            let bitrate = self.bitrate.unwrap_or(500_000);
            if ![125_000, 250_000, 500_000, 1_000_000].contains(&bitrate) {
                return Err(format!("unsupported bitrate {bitrate}"));
            }
            Ok(CanConfig {
                bitrate,
                loopback: self.loopback,
                filters: self.filters,
            })
        }
    }

    let config = CanConfigBuilder::default()
        .bitrate(250_000)
        .filter(0x7DF)
        .filter(0x7E8)
        .loopback(true)
        .build();
    if let Ok(c) = &config {
        println!(
            "{} bit/s, loopback {}, filters {:x?}",
            c.bitrate, c.loopback, c.filters
        );
    }
    println!("{:?}", CanConfigBuilder::default().bitrate(42).build());
    // ANCHOR_END: builder
}

fn layout() {
    // ANCHOR: layout
    use std::mem::{align_of, offset_of, size_of};

    #[allow(dead_code)]
    struct RustLayout {
        a: u8,
        b: u32,
        c: u8,
    }
    #[allow(dead_code)]
    #[repr(C)]
    struct CLayout {
        a: u8,  //  offset 0, then 3 bytes padding
        b: u32, // offset 4
        c: u8,  //  offset 8, then 3 bytes padding (size must be a multiple of 4)
    }
    #[allow(dead_code)]
    #[repr(C, packed)]
    struct Packed {
        a: u8,
        b: u32,
        c: u8,
    }
    #[allow(dead_code)]
    #[repr(transparent)]
    struct Wrapper(u32);

    println!(
        "repr(Rust):   size {}, align {}",
        size_of::<RustLayout>(),
        align_of::<RustLayout>()
    );
    println!(
        "repr(C):      size {}, align {}",
        size_of::<CLayout>(),
        align_of::<CLayout>()
    );
    println!(
        "   offsets a={} b={} c={}",
        offset_of!(CLayout, a),
        offset_of!(CLayout, b),
        offset_of!(CLayout, c)
    );
    println!(
        "repr(packed): size {}, align {}",
        size_of::<Packed>(),
        align_of::<Packed>()
    );
    println!(
        "transparent:  size {}, align {}",
        size_of::<Wrapper>(),
        align_of::<Wrapper>()
    );
    // repr(Rust) offsets are the compiler's choice and may change between versions:
    println!("repr(Rust) put b at offset {}", offset_of!(RustLayout, b));
    // ANCHOR_END: layout
}
