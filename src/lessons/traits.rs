//! Lesson: Traits & trait objects.

use std::fmt;

use crate::lesson::{Lesson, Question, Section};

pub static LESSON: Lesson = Lesson {
    id: "traits",
    title: "Traits & trait objects",
    summary: "Defining and implementing traits, default methods, bounds, impl Trait, dyn Trait, dispatch trade-offs, dyn compatibility, associated types, supertraits, upcasting, coherence and sealed traits.",
    source: include_str!("traits.rs"),
    sections: &[
        Section::new(
            "Defining and implementing a trait",
            r#"
            A **trait** names a set of methods that types can implement. It
            plays the role of an interface, but a trait can be implemented for
            types you didn't write (with limits, covered below), and it can
            provide default behaviour.

            `impl Trait for Type { … }` supplies the methods. Implementations
            live separately from the type's definition, so one type can
            implement many traits, each in its own block.
            "#,
        )
        .demo("define", define),
        Section::new(
            "Default methods",
            r#"
            A trait can provide method bodies. Implementors get them for free,
            and may override them. A common design is a small set of
            **required** methods plus a larger set of **provided** methods
            built on top of them. `Iterator` is the extreme case: implement
            `next` and receive more than seventy adapters.
            "#,
        )
        .demo("defaults", defaults),
        Section::new(
            "Traits as bounds and impl Trait arguments",
            r#"
            To accept "anything that implements `Sensor`", use a generic with a
            bound, or the shorthand `impl Sensor` in argument position. Both
            are monomorphized. The difference: with a named type parameter
            (`<S: Sensor>`), callers can use the turbofish, and two arguments
            can be forced to share one type.
            "#,
        )
        .demo("bounds", bounds),
        Section::new(
            "Returning impl Trait",
            r#"
            `-> impl Trait` returns **some single concrete type** that
            implements the trait, without naming it. That's essential for
            closures and iterator chains, whose types can't be written down.
            "Single" is the catch. Every return path must produce the same
            type:

            ```compile_fail,E0308
            fn numbers(even: bool) -> impl Iterator<Item = u32> {
                if even { (0..10).step_by(2) } else { (1..10).step_by(2).rev() }
            }
            ```

            To return one of several types, use an enum or `Box<dyn Trait>`.
            In edition 2024, an `impl Trait` return type captures every
            generic parameter in scope, lifetimes included. Narrow it with
            `+ use<'a, T>` when you need to.
            "#,
        )
        .demo("return_impl", return_impl),
        Section::new(
            "Trait objects: dyn Trait",
            r#"
            A **trait object** like `&dyn Shape` or `Box<dyn Shape>` erases the
            concrete type. Its pointer is **fat**: a data pointer plus a pointer
            to a **vtable** that holds the type's method addresses, its size
            and its destructor. Calls go through the vtable at run time.

            Trait objects let you store different types in one collection,
            choose implementations at run time, or keep code size down.
            "#,
        )
        .demo("dyn", trait_objects),
        Section::new(
            "Static vs dynamic dispatch",
            r#"
            ```text
                        generics / impl Trait        dyn Trait
            dispatch    static (resolved at compile)  dynamic (vtable lookup)
            inlining    yes                            rarely
            code size   one copy per type              one copy total
            mixing      one type per instantiation     many types in one collection
            pointer     thin                           fat (data + vtable)
            ```

            Neither is "better". Default to generics for hot paths and
            libraries, and use `dyn` for plugin-like flexibility, heterogeneous
            collections, or to keep firmware images small. An enum of known
            variants (Lesson 11) is a third option: static dispatch over a
            closed set of types.
            "#,
        ),
        Section::new(
            "Dyn compatibility (object safety)",
            r#"
            Not every trait can be used as `dyn Trait`. The vtable needs one
            entry per method, and some methods can't have a single entry:

            - **generic methods**, which would need one entry per type argument;
            - methods returning `Self` by value, whose size is unknown behind `dyn`;
            - associated functions without `self`, since there's no object to
              dispatch on;
            - associated constants.

            ```compile_fail,E0038
            trait Cloner {
                fn duplicate(&self) -> Self;
            }
            fn store(items: Vec<Box<dyn Cloner>>) {}
            ```

            The escape hatch is `where Self: Sized` on the offending method.
            That method then simply isn't available on trait objects, and the
            rest of the trait is.
            "#,
        )
        .demo("dyn_compat", dyn_compat),
        Section::new(
            "Associated types and constants",
            r#"
            An **associated type** is a type chosen by each implementation:
            `Iterator` has `type Item`. Compare this with a generic parameter on
            the trait (`trait Convert<T>`):

            - associated type: **one** implementation per type, and the type is
              determined by the implementor, so callers never specify it;
            - generic parameter: **many** implementations per type
              (`impl Convert<u8> for X` and `impl Convert<u16> for X` can coexist).

            Traits can also declare **associated constants**, which give each
            implementor its own value.
            "#,
        )
        .demo("assoc", associated),
        Section::new(
            "Supertraits and upcasting",
            r#"
            `trait Actuator: Device` means "every Actuator must also be a
            Device", so inside the trait and in generic code you can call
            `Device` methods on any `Actuator`. Since Rust 1.86, a `&dyn
            Actuator` coerces directly to a `&dyn Device`. That's **trait
            upcasting**, and it previously needed a hand-written `as_device()`
            method.
            "#,
        )
        .demo("supertraits", supertraits),
        Section::new(
            "Coherence: blanket impls, the orphan rule and extension traits",
            r#"
            You can implement a trait for many types at once with a **blanket
            impl**: `impl<T: Display> Loggable for T`. To prevent conflicting
            implementations across crates, the **orphan rule** says an impl
            must involve a local trait **or** a local type:

            ```compile_fail,E0117
            impl std::fmt::Display for Vec<u8> {
                fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { Ok(()) }
            }
            ```

            Two standard workarounds:

            - wrap the foreign type in a local **newtype**;
            - define a local **extension trait** and implement it for the
              foreign type, which adds methods like `2.5.kph()` to `f64`.
            "#,
        )
        .demo("coherence", coherence),
        Section::new(
            "Fully qualified syntax",
            r#"
            When two traits give a type methods with the same name, a plain
            method call is ambiguous. Name the trait you mean:
            `Trait::method(&value)`, or in full,
            `<Type as Trait>::method(&value)`. The full form is also how you
            call associated functions that have no `self`.
            "#,
        )
        .demo("qualified", qualified),
        Section::new(
            "Sealed traits",
            r#"
            Sometimes you want a public trait that outsiders can **use** but
            not **implement**, so you can add methods later without breaking
            anyone, or because your code relies on knowing every implementor.
            The **sealed trait** pattern makes the public trait require a
            supertrait that lives in a private module. Outside code can't name
            the supertrait, so it can't implement the public trait.
            "#,
        )
        .demo("sealed", sealed),
        Section::new(
            "Marker traits",
            r#"
            Some traits have no methods. They mark a property the compiler
            relies on:

            ```text
            Copy    bitwise duplication is a valid copy
            Send    safe to move to another thread
            Sync    safe to share (&T) between threads
            Sized   size known at compile time (implicit bound on generics)
            Unpin   safe to move even after being pinned (Lesson 30)
            ```

            `Send`, `Sync` and `Unpin` are **auto traits**: the compiler
            implements them automatically when all of a type's fields do. That
            is how `Rc` (not thread-safe) keeps any struct containing it from
            crossing threads, with no annotation needed (Lesson 23).
            "#,
        ),
    ],
    quiz: &[
        Question::new(
            "What is inside a `&dyn Trait` pointer?",
            &[
                "Just a data pointer",
                "A data pointer and a vtable pointer",
                "A copy of the value",
                "A type id",
            ],
            1,
            "Trait object references are fat pointers: data plus a vtable of method pointers, size, alignment and destructor.",
        ),
        Question::new(
            "Why can't `fn make() -> impl Shape` return a Circle on one branch and a Square on another?",
            &[
                "impl Trait means one concrete type chosen by the function",
                "Shapes can't be returned",
                "It needs a lifetime",
                "It can, if both implement Shape",
            ],
            0,
            "Return-position impl Trait hides a single concrete type. For several, use an enum or Box<dyn Shape>.",
        ),
        Question::new(
            "Which method makes a trait unusable as `dyn Trait` (unless it has `where Self: Sized`)?",
            &[
                "fn name(&self) -> String",
                "fn reset(&mut self)",
                "fn convert<T>(&self, t: T)",
                "fn id(&self) -> u32",
            ],
            2,
            "Generic methods would need a vtable entry per type argument, so they break dyn compatibility.",
        ),
        Question::new(
            "When should a trait use an associated type rather than a generic parameter?",
            &[
                "When each implementing type has exactly one natural choice (like Iterator::Item)",
                "When you want several impls per type",
                "Always",
                "Never; they're equivalent",
            ],
            0,
            "Associated types encode 'one per implementor'. Generic parameters allow many impls for the same type.",
        ),
        Question::new(
            "The orphan rule forbids which impl in your crate?",
            &[
                "impl MyTrait for Vec<u8>",
                "impl Display for MyType",
                "impl Display for Vec<u8>",
                "impl<T> MyTrait for T",
            ],
            2,
            "Both Display and Vec are foreign. At least one of the trait or the type must be local.",
        ),
        Question::new(
            "What does trait upcasting (Rust 1.86) allow?",
            &[
                "Casting any type to any trait",
                "Coercing &dyn Sub to &dyn Super when Sub: Super",
                "Downcasting dyn Any",
                "Converting generics to dyn",
            ],
            1,
            "A trait object of a subtrait can be used where a trait object of its supertrait is expected.",
        ),
    ],
    exercises: &[
        "Define `trait Shape { fn area(&self) -> f64; fn name(&self) -> String { \"shape\".into() } }`, implement it for three types, and print the total area of a `Vec<Box<dyn Shape>>`.",
        "Rewrite the same total-area function generically (`fn total<S: Shape>(shapes: &[S])`). Why can't it accept the mixed Vec any more?",
        "Write an extension trait `trait Clamp01 { fn clamp01(self) -> Self; }` for `f32` and `f64`.",
        "Create a sealed trait `CanFrameKind` implemented only for `Standard` and `Extended` marker types, and a function generic over it.",
    ],
};

// ANCHOR: define
trait Sensor {
    fn name(&self) -> String;
    fn read(&mut self) -> f64;
}

struct Thermometer {
    celsius: f64,
}

struct Tachometer {
    rpm: u32,
}

impl Sensor for Thermometer {
    fn name(&self) -> String {
        "thermometer".to_string()
    }
    fn read(&mut self) -> f64 {
        self.celsius += 0.5; // pretend the engine is warming up
        self.celsius
    }
}

impl Sensor for Tachometer {
    fn name(&self) -> String {
        format!("tachometer ({} rpm max)", 7000)
    }
    fn read(&mut self) -> f64 {
        self.rpm as f64
    }
}

fn define() {
    let mut t = Thermometer { celsius: 20.0 };
    let mut r = Tachometer { rpm: 3200 };
    println!("{} → {}", t.name(), t.read());
    println!("{} → {}", r.name(), r.read());
}
// ANCHOR_END: define

fn defaults() {
    // ANCHOR: defaults
    trait Describe {
        fn label(&self) -> String; // required

        fn describe(&self) -> String {
            // provided: built on top of `label`
            format!("<{}>", self.label())
        }
        fn shout(&self) -> String {
            self.describe().to_uppercase()
        }
    }

    struct Door;
    struct Window;

    impl Describe for Door {
        fn label(&self) -> String {
            "door".into()
        }
    }
    impl Describe for Window {
        fn label(&self) -> String {
            "window".into()
        }
        fn describe(&self) -> String {
            format!("[{} — overridden]", self.label())
        }
    }

    println!("{} {}", Door.describe(), Door.shout());
    println!("{} {}", Window.describe(), Window.shout()); // shout uses the override
    // ANCHOR_END: defaults
}

fn bounds() {
    // ANCHOR: bounds
    fn log_once(sensor: &mut impl Sensor) {
        // shorthand for <S: Sensor>(sensor: &mut S)
        println!("{}: {:.1}", sensor.name(), sensor.read());
    }

    fn compare<S: Sensor>(a: &mut S, b: &mut S) -> f64 {
        // the named parameter forces both to be the *same* type
        a.read() - b.read()
    }

    fn average<S>(sensor: &mut S, samples: u32) -> f64
    where
        S: Sensor + ?Sized, // ?Sized: also accept dyn Sensor
    {
        (0..samples).map(|_| sensor.read()).sum::<f64>() / samples as f64
    }

    let mut a = Thermometer { celsius: 80.0 };
    let mut b = Thermometer { celsius: 85.0 };
    log_once(&mut a);
    println!("difference {:.1}", compare(&mut a, &mut b));
    println!("average {:.2}", average(&mut a, 4));
    // compare(&mut a, &mut Tachometer { rpm: 1 }); // error: different types
    // ANCHOR_END: bounds
}

fn return_impl() {
    // ANCHOR: return_impl
    fn evens(limit: u32) -> impl Iterator<Item = u32> {
        (0..limit).filter(|n| n % 2 == 0)
    }

    fn scaler(factor: f64) -> impl Fn(f64) -> f64 {
        move |x| x * factor
    }

    // To return different types, box them behind a trait object:
    fn pick(odd: bool) -> Box<dyn Iterator<Item = u32>> {
        if odd {
            Box::new((1..10).step_by(2))
        } else {
            Box::new(evens(10))
        }
    }

    println!("{:?}", evens(10).collect::<Vec<_>>());
    let to_mph = scaler(0.621_371);
    println!("100 km/h = {:.1} mph", to_mph(100.0));
    println!("{:?}", pick(true).collect::<Vec<_>>());
    // ANCHOR_END: return_impl
}

fn trait_objects() {
    // ANCHOR: dyn
    use std::mem::size_of;

    // One Vec, several concrete types:
    let mut sensors: Vec<Box<dyn Sensor>> = vec![
        Box::new(Thermometer { celsius: 70.0 }),
        Box::new(Tachometer { rpm: 900 }),
        Box::new(Thermometer { celsius: 30.0 }),
    ];
    for s in sensors.iter_mut() {
        println!("{:<26} {:>7.1}", s.name(), s.read()); // vtable call
    }

    println!("&Thermometer     = {} bytes", size_of::<&Thermometer>());
    println!(
        "&dyn Sensor      = {} bytes (data + vtable)",
        size_of::<&dyn Sensor>()
    );
    println!("Box<dyn Sensor>  = {} bytes", size_of::<Box<dyn Sensor>>());
    // ANCHOR_END: dyn
}

fn dyn_compat() {
    // ANCHOR: dyn_compat
    trait Component {
        fn id(&self) -> u32;

        // Generic method: excluded from dyn via `where Self: Sized`.
        fn send<T: fmt::Debug>(&self, msg: T)
        where
            Self: Sized,
        {
            println!("component {} sends {msg:?}", self.id());
        }

        // Returning Self: also excluded from dyn.
        fn duplicate(&self) -> Self
        where
            Self: Sized;
    }

    #[derive(Clone)]
    struct Ecu(u32);
    impl Component for Ecu {
        fn id(&self) -> u32 {
            self.0
        }
        fn duplicate(&self) -> Self {
            self.clone()
        }
    }

    let ecu = Ecu(7);
    ecu.send("hello"); // fine on the concrete type
    let copy = ecu.duplicate();

    let parts: Vec<&dyn Component> = vec![&ecu, &copy]; // the trait is still dyn-compatible
    let ids: Vec<u32> = parts.iter().map(|c| c.id()).collect();
    println!("ids via dyn: {ids:?}");
    // parts[0].send(1); // error: `send` can't be called on dyn Component
    // ANCHOR_END: dyn_compat
}

fn associated() {
    // ANCHOR: assoc
    trait Channel {
        type Sample; //         each channel decides its sample type
        const RATE_HZ: u32; //  and its own sampling rate

        fn sample(&mut self) -> Self::Sample;
    }

    struct Microphone {
        t: u32,
    }
    struct Gps;

    impl Channel for Microphone {
        type Sample = i16;
        const RATE_HZ: u32 = 48_000;
        fn sample(&mut self) -> i16 {
            self.t += 1;
            ((self.t as f32 * 0.3).sin() * 1000.0) as i16
        }
    }
    impl Channel for Gps {
        type Sample = (f64, f64);
        const RATE_HZ: u32 = 10;
        fn sample(&mut self) -> (f64, f64) {
            (35.0839, 137.1576) // Toyota City
        }
    }

    fn describe<C: Channel>(c: &mut C) -> String
    where
        C::Sample: fmt::Debug,
    {
        format!("{:?} at {} Hz", c.sample(), C::RATE_HZ)
    }

    println!("{}", describe(&mut Microphone { t: 0 }));
    println!("{}", describe(&mut Gps));
    // ANCHOR_END: assoc
}

fn supertraits() {
    // ANCHOR: supertraits
    trait Device {
        fn name(&self) -> &str;
    }
    trait Actuator: Device {
        // every Actuator is also a Device
        fn set(&mut self, level: u8);
        fn report(&self) -> String {
            format!("{} is an actuator", self.name()) // supertrait method available
        }
    }

    struct Throttle {
        level: u8,
    }
    impl Device for Throttle {
        fn name(&self) -> &str {
            "throttle"
        }
    }
    impl Actuator for Throttle {
        fn set(&mut self, level: u8) {
            self.level = level.min(100);
        }
    }

    let mut t = Throttle { level: 0 };
    t.set(140);
    let actuator: &dyn Actuator = &t;
    println!("{} (level {})", actuator.report(), t.level);

    let device: &dyn Device = actuator; // trait upcasting (Rust 1.86+)
    println!("as a plain device: {}", device.name());
    // ANCHOR_END: supertraits
}

fn coherence() {
    // ANCHOR: coherence
    use std::fmt::Display;

    // A blanket impl: every Display type gets `log_line` for free.
    trait Loggable {
        fn log_line(&self) -> String;
    }
    impl<T: Display> Loggable for T {
        fn log_line(&self) -> String {
            format!("[log] {self}")
        }
    }
    println!("{}", 42.log_line());
    println!("{}", "brakes ok".log_line());

    // Newtype: implement a foreign trait (Display) for a foreign type (Vec<u8>).
    struct HexBytes(Vec<u8>);
    impl Display for HexBytes {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            for (i, b) in self.0.iter().enumerate() {
                if i > 0 {
                    write!(f, " ")?;
                }
                write!(f, "{b:02X}")?;
            }
            Ok(())
        }
    }
    println!("{}", HexBytes(vec![0x7E, 0x08, 0xFF]));

    // Extension trait: add methods to a foreign type (f64).
    trait SpeedExt {
        fn kph_to_mps(self) -> f64;
    }
    impl SpeedExt for f64 {
        fn kph_to_mps(self) -> f64 {
            self / 3.6
        }
    }
    println!("100 km/h = {:.2} m/s", 100.0_f64.kph_to_mps());
    // ANCHOR_END: coherence
}

fn qualified() {
    // ANCHOR: qualified
    trait Metric {
        fn unit() -> &'static str;
        fn value(&self) -> f64;
    }
    trait Imperial {
        fn unit() -> &'static str;
        fn value(&self) -> f64;
    }

    struct Distance {
        km: f64,
    }
    impl Metric for Distance {
        fn unit() -> &'static str {
            "km"
        }
        fn value(&self) -> f64 {
            self.km
        }
    }
    impl Imperial for Distance {
        fn unit() -> &'static str {
            "mi"
        }
        fn value(&self) -> f64 {
            self.km * 0.621_371
        }
    }

    let d = Distance { km: 42.195 };
    // d.value(); // error: multiple applicable items in scope
    println!("{:.3} {}", Metric::value(&d), <Distance as Metric>::unit());
    println!(
        "{:.3} {}",
        Imperial::value(&d),
        <Distance as Imperial>::unit()
    );
    // ANCHOR_END: qualified
}

fn sealed() {
    // ANCHOR: sealed
    mod can {
        mod private {
            pub trait Sealed {} // public trait in a private module: unnameable outside
        }

        pub trait FrameKind: private::Sealed {
            const ID_BITS: u32;
        }

        pub struct Standard;
        pub struct Extended;
        impl private::Sealed for Standard {}
        impl private::Sealed for Extended {}
        impl FrameKind for Standard {
            const ID_BITS: u32 = 11;
        }
        impl FrameKind for Extended {
            const ID_BITS: u32 = 29;
        }

        pub fn max_id<K: FrameKind>() -> u32 {
            (1 << K::ID_BITS) - 1
        }
    }

    println!("standard max id {:#X}", can::max_id::<can::Standard>());
    println!("extended max id {:#X}", can::max_id::<can::Extended>());
    // struct Mine; impl can::FrameKind for Mine { … } // error: Sealed is not accessible
    // ANCHOR_END: sealed
}
