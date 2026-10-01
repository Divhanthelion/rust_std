//! Lesson: Generics.

use std::fmt::Debug;
use std::marker::PhantomData;

use crate::lesson::{Lesson, Question, Section};

pub static LESSON: Lesson = Lesson {
    id: "generics",
    title: "Generics",
    summary: "Generic functions, structs, enums and impls; bounds and where clauses; monomorphization; turbofish; const generics; PhantomData.",
    source: include_str!("generics.rs"),
    sections: &[
        Section::new(
            "Why generics?",
            r#"
            Without generics you would write `largest_i32`, `largest_f64` and
            `largest_char`: the same logic three times. A generic function takes
            **type parameters** in angle brackets and works for any type that
            meets its requirements.

            Those requirements are **trait bounds**. To find the largest
            element we need to compare (`PartialOrd`) and to return a copy
            (`Copy`). The compiler checks the body against the bounds, so a
            generic function that compiles works for **every** type satisfying
            them. That's a stronger guarantee than C++ templates, which are only
            checked when instantiated.
            "#,
        )
        .demo("largest", largest_demo),
        Section::new(
            "Generic structs and enums",
            r#"
            Types can be generic too. `Point<T>` has both coordinates of the
            same type, and `Pair<A, B>` may mix types. You've already used the
            most important generic enums, `Option<T>` and `Result<T, E>`, and
            generic structs like `Vec<T>` and `HashMap<K, V>`.
            "#,
        )
        .demo("types", generic_types),
        Section::new(
            "Generic and specialized impl blocks",
            r#"
            `impl<T> Point<T> { … }` adds methods for every `T`. The `<T>` after
            `impl` declares the parameter, and the one after `Point` uses it. You
            can also write `impl Point<f64> { … }`, whose methods exist **only**
            for that concrete type. And a method can have type parameters of its
            own, independent of the struct's.
            "#,
        )
        .demo("impls", generic_impls),
        Section::new(
            "Bounds and where clauses",
            r#"
            Bounds can be written inline (`T: Debug + Clone`) or in a `where`
            clause after the signature. `where` is easier to read with several
            parameters, and it can express bounds that don't fit inline, such
            as bounds on associated types (`I::Item: Debug`) or on other types.

            Ask only for what the body needs. Each bound you add rules out some
            callers.
            "#,
        )
        .demo("bounds", bounds),
        Section::new(
            "Monomorphization: zero-cost, at a price",
            r#"
            When you call `largest::<i32>` and `largest::<f64>`, the compiler
            generates a separate, fully specialized copy of the function for
            each type. This is **monomorphization**. Each copy is as fast as
            hand-written code, and can be inlined and optimized for its type.

            The cost is compile time and binary size, since every
            instantiation is more machine code. That's one end of a spectrum.
            Trait objects (`dyn Trait`, Lesson 16) are the other end: one copy
            of the code, with a small run-time cost per call. In
            flash-constrained embedded firmware the trade-off can tip toward
            `dyn`.
            "#,
        )
        .demo("mono", monomorphization),
        Section::new(
            "Turbofish and inference",
            r#"
            Usually the compiler infers type arguments. When it can't, supply
            them with the **turbofish** `::<>`: `"5".parse::<u8>()`,
            `iter.collect::<Vec<_>>()`, or `size_of::<u64>()`. The `_` says
            "infer this part". On a type, the turbofish goes after the type
            name: `Vec::<u8>::with_capacity(16)`.
            "#,
        )
        .demo("turbofish", turbofish),
        Section::new(
            "Const generics",
            r#"
            Type parameters can also be **values**: `const N: usize`. Arrays
            already use this, since `[T; N]` is generic over `N`. Your own types
            can do the same, for example a fixed-capacity buffer
            `Buffer<T, const N: usize>`. The size is part of the type, the
            storage is inline, and nothing touches the heap. That's a key
            technique for embedded code (Lesson 33).

            Since Rust 1.89, `_` can stand for an inferred const argument.
            Arithmetic on const parameters in types (`[T; N + 1]`) is still
            unstable, so stable code works around it.
            "#,
        )
        .demo("const_generics", const_generics),
        Section::new(
            "PhantomData and default type parameters",
            r#"
            Sometimes a type parameter exists only to **distinguish types**,
            and no field actually stores a `T`. Take `Id<User>` versus
            `Id<Vehicle>`: both are just a `u32`, but mixing them up should
            be a compile error. The compiler rejects unused parameters, so you
            add a zero-sized `PhantomData<T>` field to "use" it.

            Type parameters can also have **defaults**: `struct Meters<T =
            f64>`. The standard library uses this in operator traits, for
            example `trait Add<Rhs = Self>`, so `impl Add for Point` means
            "add two Points".
            "#,
        )
        .demo("phantom", phantom),
    ],
    quiz: &[
        Question::new(
            "Why does `fn largest<T>(items: &[T]) -> T` fail to compile when its body compares items?",
            &[
                "Generic functions can't return T",
                "T has no bound saying it can be compared (PartialOrd) or copied out (Copy)",
                "Slices can't be generic",
                "It needs a lifetime",
            ],
            1,
            "Generic bodies are checked against their bounds. Without `T: PartialOrd + Copy`, `>` and copying out aren't allowed.",
        ),
        Question::new(
            "What is monomorphization?",
            &[
                "Converting objects to a single type at run time",
                "Generating a specialized copy of generic code for each concrete type used",
                "Removing unused generics",
                "A form of dynamic dispatch",
            ],
            1,
            "Each instantiation becomes its own machine code, which is why generics are zero-cost at run time but grow binaries.",
        ),
        Question::new(
            "Where do methods in `impl Point<f64> { fn norm(&self) -> f64 { … } }` exist?",
            &[
                "On every Point<T>",
                "Only on Point<f64>",
                "On Point<f32> and Point<f64>",
                "Nowhere; it's a compile error",
            ],
            1,
            "An impl for a concrete instantiation adds methods only to that type.",
        ),
        Question::new(
            "What does `PhantomData<T>` cost at run time?",
            &[
                "One pointer",
                "size_of::<T>()",
                "Nothing — it's zero-sized",
                "One byte",
            ],
            2,
            "PhantomData is a zero-sized marker that tells the compiler the type logically involves T.",
        ),
        Question::new(
            "Which compiles on stable Rust?",
            &[
                "fn f<const N: usize>(a: [u8; N]) -> [u8; N + 1]",
                "fn f<const N: usize>(a: [u8; N]) -> usize { N }",
                "fn f<const S: String>()",
                "fn f<const N: f64>()",
            ],
            1,
            "Const parameters can be used as values and as array lengths. Generic const arithmetic in types, String parameters and float parameters are not stable.",
        ),
    ],
    exercises: &[
        "Write `fn count_matching<T: PartialEq>(items: &[T], target: &T) -> usize` and call it with `i32`, `&str` and a struct deriving `PartialEq`.",
        "Make `struct Stack<T> { items: Vec<T> }` with `push`, `pop`, `peek` (returning `Option<&T>`) and `len`. Add `impl<T: Debug> Stack<T> { fn dump(&self) }`.",
        "Write `fn average<const N: usize>(samples: [f32; N]) -> f32`. What happens when you call it with an empty array? Fix it.",
        "Create typed ids `Id<T>` with PhantomData and a `Registry<T>` that only accepts `Id<T>` of the right type. Try passing the wrong kind of id.",
    ],
};

// ANCHOR: largest
fn largest<T: PartialOrd + Copy>(items: &[T]) -> Option<T> {
    let mut iter = items.iter();
    let mut best = *iter.next()?; // None for an empty slice
    for &item in iter {
        if item > best {
            best = item;
        }
    }
    Some(best)
}

fn largest_demo() {
    println!("{:?}", largest(&[3, 9, 2])); //           T = i32
    println!("{:?}", largest(&[0.5, -1.0, 2.25])); //   T = f64
    println!("{:?}", largest(&['r', 'u', 's', 't'])); // T = char
    println!("{:?}", largest::<u8>(&[])); //             empty → None
}
// ANCHOR_END: largest

fn generic_types() {
    // ANCHOR: types
    #[derive(Debug)]
    struct Point<T> {
        x: T,
        y: T,
    }

    #[derive(Debug)]
    struct Pair<A, B> {
        first: A,
        second: B,
    }

    #[derive(Debug)]
    enum Measurement<T> {
        Valid(T),
        OutOfRange { raw: T },
        Missing,
    }

    let int_point = Point { x: 3, y: 4 };
    let float_point = Point { x: 1.5, y: -2.0 };
    // let mixed = Point { x: 1, y: 2.0 }; // error: x and y must be the same T
    let pair = Pair {
        first: "rpm",
        second: 3200,
    };
    let readings = [
        Measurement::Valid(88.5),
        Measurement::OutOfRange { raw: 999.0 },
        Measurement::Missing,
    ];
    println!(
        "{int_point:?}\n{float_point:?}\n{pair:?} ({} = {})",
        pair.first, pair.second
    );
    println!("{readings:?}");
    // ANCHOR_END: types
}

fn generic_impls() {
    // ANCHOR: impls
    #[derive(Debug, Clone, Copy)]
    struct Point<T> {
        x: T,
        y: T,
    }

    impl<T: Copy> Point<T> {
        fn swap(self) -> Point<T> {
            Point {
                x: self.y,
                y: self.x,
            }
        }
        // A method with its own type parameter U:
        fn with_label<U>(self, label: U) -> (U, Point<T>) {
            (label, self)
        }
    }

    impl Point<f64> {
        // Only exists for Point<f64>: sqrt needs a float.
        fn distance_from_origin(&self) -> f64 {
            (self.x * self.x + self.y * self.y).sqrt()
        }
    }

    let p = Point { x: 3.0, y: 4.0 };
    println!("{:?} → {:?}", p, p.swap());
    println!("distance {}", p.distance_from_origin());
    println!("{:?}", Point { x: 1, y: 2 }.with_label("integers"));
    // Point { x: 1, y: 2 }.distance_from_origin(); // error: no such method for Point<i32>
    // ANCHOR_END: impls
}

fn bounds() {
    // ANCHOR: bounds
    use std::fmt::Display;

    // Inline bounds:
    fn show_all<T: Display>(items: &[T]) -> String {
        items
            .iter()
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    }

    // A where clause: easier to read, and can constrain associated types.
    fn summarize<I>(items: I) -> String
    where
        I: IntoIterator,
        I::Item: Debug,
    {
        let parts: Vec<String> = items.into_iter().map(|i| format!("{i:?}")).collect();
        format!("[{}] ({} items)", parts.join(" "), parts.len())
    }

    println!("{}", show_all(&[1.5, 2.0]));
    println!("{}", summarize(vec!["a", "b"]));
    println!("{}", summarize(1..=3));
    println!("{}", summarize([Some('x'), None]));
    // ANCHOR_END: bounds
}

fn monomorphization() {
    // ANCHOR: mono
    fn describe<T: Debug>(value: T) -> String {
        // type_name reveals which copy of `describe` is running.
        format!("{value:?}: {}", std::any::type_name::<T>())
    }
    println!("{}", describe(42u8));
    println!("{}", describe("text"));
    println!("{}", describe(vec![1.0f32]));
    println!("{}", describe(Some(true)));
    // Four calls, four distinct functions in the binary.
    // ANCHOR_END: mono
}

fn turbofish() {
    // ANCHOR: turbofish
    use std::collections::HashSet;
    use std::mem::size_of;

    let parsed = "200".parse::<u8>();
    let unique = [3, 1, 3, 2].into_iter().collect::<HashSet<_>>();
    let mut buffer = Vec::<u16>::with_capacity(8);
    buffer.push(0xBEEF);
    println!("{parsed:?} {} {buffer:x?}", unique.len());
    println!("size_of::<u128>() = {}", size_of::<u128>());
    println!("default: {}", <u32 as Default>::default()); // a trait method, qualified
    // ANCHOR_END: turbofish
}

fn const_generics() {
    // ANCHOR: const_generics
    /// A fixed-capacity stack: no heap allocation, ever.
    #[derive(Debug)]
    struct Buffer<T, const N: usize> {
        items: [T; N],
        len: usize,
    }

    impl<T: Copy + Default, const N: usize> Buffer<T, N> {
        fn new() -> Self {
            Buffer {
                items: [T::default(); N],
                len: 0,
            }
        }
        fn push(&mut self, value: T) -> Result<(), T> {
            if self.len == N {
                return Err(value); // full: give the value back
            }
            self.items[self.len] = value;
            self.len += 1;
            Ok(())
        }
        fn as_slice(&self) -> &[T] {
            &self.items[..self.len]
        }
    }

    let mut buf: Buffer<u8, 3> = Buffer::new();
    for b in [10, 20, 30, 40] {
        if let Err(rejected) = buf.push(b) {
            println!("buffer full, rejected {rejected}");
        }
    }
    println!(
        "{:?}  size_of = {} bytes",
        buf.as_slice(),
        std::mem::size_of::<Buffer<u8, 3>>()
    );

    fn sum<const N: usize>(values: [i32; N]) -> i32 {
        values.iter().sum()
    }
    println!("sum of 3: {}, sum of 5: {}", sum([1, 2, 3]), sum([1; 5]));
    let zeros: [u8; _] = [0; 4]; // `_` infers the length (Rust 1.89+)
    println!("inferred length {}", zeros.len());
    // ANCHOR_END: const_generics
}

fn phantom() {
    // ANCHOR: phantom
    struct User;
    struct Vehicle;

    /// A typed identifier: an `Id<User>` can't be passed as an `Id<Vehicle>`.
    struct Id<T> {
        raw: u32,
        _kind: PhantomData<T>,
    }
    impl<T> Id<T> {
        fn new(raw: u32) -> Self {
            Id {
                raw,
                _kind: PhantomData,
            }
        }
    }

    fn lookup_vehicle(id: Id<Vehicle>) -> String {
        format!("vehicle #{}", id.raw)
    }

    let user_id: Id<User> = Id::new(7);
    let vehicle_id: Id<Vehicle> = Id::new(7);
    println!("{}", lookup_vehicle(vehicle_id));
    // lookup_vehicle(user_id); // error: expected Id<Vehicle>, found Id<User>
    println!(
        "user id {} — same size as u32? {}",
        user_id.raw,
        std::mem::size_of::<Id<User>>() == std::mem::size_of::<u32>()
    );

    // A default type parameter:
    struct Reading<T = f64> {
        value: T,
    }
    let default_kind: Reading = Reading { value: 1.5 }; // T = f64
    let integer: Reading<i32> = Reading { value: 3 };
    println!("{} {}", default_kind.value, integer.value);
    // ANCHOR_END: phantom
}
