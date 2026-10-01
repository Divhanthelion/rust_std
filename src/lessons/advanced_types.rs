//! Lesson: Advanced types.

use std::convert::Infallible;
use std::fmt::Display;
use std::marker::PhantomPinned;
use std::mem::size_of;
use std::num::NonZero;
use std::pin::{Pin, pin};

use crate::lesson::{Lesson, Question, Section};

pub static LESSON: Lesson = Lesson {
    id: "advanced_types",
    title: "Advanced types",
    summary: "Type aliases, the never type and Infallible, dynamically sized types and ?Sized, zero-sized types, niches, fn item types, Pin, impl Trait in traits and GATs.",
    source: include_str!("advanced_types.rs"),
    sections: &[
        Section::new(
            "Type aliases",
            r#"
            `type Name = ExistingType;` creates a second name, **not** a new
            type. The two are interchangeable. Aliases shorten long types and
            give a module a default error type, as in `std::io::Result<T>`,
            which is `Result<T, io::Error>`. When you need the compiler to keep
            two things apart, use a newtype (Lesson 10) instead.
            "#,
        )
        .demo("alias", aliases),
        Section::new(
            "The never type and Infallible",
            r#"
            `!` is the type of expressions that never produce a value
            (Lesson 5). Its library cousin, `std::convert::Infallible`, is an
            enum with **no variants**, so a value of it can never exist. It's
            used as the error type of conversions that can't fail:
            `Result<T, Infallible>`.

            Because the `Err` arm is impossible, Rust 1.82+ accepts
            `let Ok(v) = result;` for such results with no `else`. The
            compiler knows the pattern is irrefutable.
            "#,
        )
        .demo("never", never),
        Section::new(
            "Dynamically sized types and ?Sized",
            r#"
            Most types have a size known at compile time (`Sized`). Three kinds
            don't: `str`, slices `[T]`, and trait objects `dyn Trait`. They are
            **dynamically sized types** (DSTs) and can only live behind a
            pointer (`&str`, `Box<[T]>`, `Arc<dyn Trait>`) that carries the
            missing information: a length or a vtable.

            Generic parameters have an implicit `T: Sized` bound. Write
            `T: ?Sized` to also accept DSTs, which is what lets
            `fn show<T: Display + ?Sized>(x: &T)` take a `&str` or a
            `&dyn Display`.
            "#,
        )
        .demo("dst", dsts),
        Section::new(
            "Zero-sized types",
            r#"
            Types with no data occupy **zero bytes**: `()`, unit structs,
            `PhantomData<T>`, `[T; 0]`, and every function item. They cost
            nothing at run time, but carry meaning at compile time: markers,
            typestates (Lesson 34) and capability tokens.

            Collections exploit them too. `HashSet<T>` is literally a
            `HashMap<T, ()>`, and a `Vec<()>` never allocates.
            "#,
        )
        .demo("zst", zsts),
        Section::new(
            "Niches revisited",
            r#"
            When a type has bit patterns that can never occur, called a
            **niche**, enums wrapping it can store their tag there for free:

            - references and `Box` are never null;
            - `NonZero*` is never 0;
            - `bool` only uses 0 and 1, and `char` stops at 0x10FFFF;
            - an enum with fewer than 256 variants leaves unused tag values.

            So `Option<Box<T>>` is a single pointer, and nested options can
            share a niche. This is why idiomatic `Option` costs nothing in
            the common cases.
            "#,
        )
        .demo("niches", niches),
        Section::new(
            "Function items, pointers and closures",
            r#"
            Every `fn` has its own **unique zero-sized type**, called a function
            item. Calling through it is a direct, inlinable call. It coerces to
            a **function pointer** `fn(i32) -> i32` (one word, an indirect
            call) when you need one type for several functions. Closures are
            also unique types whose size is the size of their captures.
            Generics over `F: Fn` keep everything static; `fn` pointers and
            `dyn Fn` erase it.
            "#,
        )
        .demo("fn_types", fn_types),
        Section::new(
            "Pin: values that must not move",
            r#"
            Normally, any value can be moved to a new address: assigned,
            returned, or pushed into a `Vec`. A few types must **not** move once
            in use, because they contain pointers into themselves. The state
            machines generated for `async fn` are the main example.

            `Pin<P>` wraps a pointer `P` (such as `&mut T` or `Box<T>`) and
            promises that the pointee won't move again until it's dropped.
            Most types are `Unpin`, meaning they don't care, so `Pin` is no
            restriction for them. Types that do care, such as futures and
            anything holding `PhantomPinned`, can only be reached through
            `Pin` API once pinned. Create pins safely with the `pin!` macro
            (on the stack) or `Box::pin` (on the heap). Lesson 30 shows where
            this matters.
            "#,
        )
        .demo("pin", pinning),
        Section::new(
            "impl Trait in traits",
            r#"
            Since Rust 1.75, trait methods can return `impl Trait` and can be
            `async fn`. Each implementor picks its own concrete type. Before
            that, people boxed the return value (`Box<dyn Iterator>`) or added
            an associated type. One caveat: such traits aren't dyn-compatible,
            because the hidden return type differs per implementation.
            "#,
        )
        .demo("rpitit", rpitit),
        Section::new(
            "Generic associated types",
            r#"
            An associated type can have its own generic parameters, including
            lifetimes. That's a **GAT** (Rust 1.65+). The classic use is a
            container trait whose iterator **borrows** from the container:
            `type Iter<'a>: Iterator<Item = &'a u32> where Self: 'a;`. Without
            GATs, the iterator type couldn't mention the lifetime of `&self`.
            "#,
        )
        .demo("gat", gats),
    ],
    quiz: &[
        Question::new(
            "What is the difference between `type Meters = f64;` and `struct Meters(f64);`?",
            &[
                "None",
                "The alias is just another name for f64; the struct is a distinct type",
                "The struct is slower",
                "The alias allocates",
            ],
            1,
            "Aliases are interchangeable with the original type. Newtypes are checked as different types.",
        ),
        Question::new(
            "Which of these is NOT dynamically sized?",
            &["str", "[u8]", "dyn Display", "&str"],
            3,
            "`&str` is a sized fat pointer (two words). The `str` it points to is the DST.",
        ),
        Question::new(
            "Why does `fn f<T: Display + ?Sized>(x: &T)` accept a `&str` while `fn f<T: Display>(x: &T)` does not?",
            &[
                "str isn't Display",
                "Generic parameters are implicitly Sized; ?Sized lifts that so T = str is allowed",
                "Lifetimes",
                "It does accept it",
            ],
            1,
            "With the implicit Sized bound, T can't be `str`. `?Sized` opts out.",
        ),
        Question::new(
            "How big is `Option<Box<u64>>`?",
            &[
                "16 bytes",
                "9 bytes",
                "8 bytes (one pointer)",
                "It depends on the u64",
            ],
            2,
            "Box is never null, so None uses the null niche.",
        ),
        Question::new(
            "What does `Pin<&mut T>` guarantee?",
            &[
                "T is immutable",
                "The pointee won't be moved again until it's dropped (unless T: Unpin)",
                "T is on the heap",
                "T is Send",
            ],
            1,
            "Pinning prevents moves of !Unpin values, so self-references inside them stay valid.",
        ),
    ],
    exercises: &[
        "Define `type Res<T> = Result<T, AppError>;` in a module and use it across several functions. Then turn the alias into a newtype and note what changes.",
        "Print `size_of` for `Option<Option<bool>>`, `Option<char>`, `Option<NonZero<u8>>` and `Option<(u8, bool)>`. Explain each with niches.",
        "Write `fn print_all<T: Debug + ?Sized>(items: &[&T])` and call it with string slices and with `&dyn Debug` values.",
        "Write a trait `Sensors { fn readings(&self) -> impl Iterator<Item = f64> + '_; }` and implement it for two structs.",
    ],
};

fn aliases() {
    // ANCHOR: alias
    use std::collections::HashMap;

    type SignalMap = HashMap<u32, Vec<(String, f64)>>; // a long type, named once
    type Res<T> = Result<T, String>; //                    a module-wide error type

    fn lookup(map: &SignalMap, id: u32) -> Res<usize> {
        map.get(&id)
            .map(Vec::len)
            .ok_or(format!("no frame {id:#x}"))
    }

    let mut map: SignalMap = HashMap::new();
    map.insert(0x7E8, vec![("rpm".into(), 3200.0), ("speed".into(), 88.0)]);
    println!("{:?} {:?}", lookup(&map, 0x7E8), lookup(&map, 0x100));

    type Meters = f64;
    let height: Meters = 1.75;
    let plain: f64 = height; // aliases are interchangeable: no type safety gained
    println!("{plain}");
    // ANCHOR_END: alias
}

fn never() {
    // ANCHOR: never
    // Widening u32 → u64 can't fail, so TryFrom's error type is Infallible.
    let converted: Result<u64, Infallible> = u64::try_from(7u32);
    let Ok(value) = converted; // irrefutable: Err(Infallible) can't exist
    println!("value = {value}");

    // FromStr for String can't fail either:
    let text: Result<String, Infallible> = "hello".parse::<String>();
    let Ok(s) = text;
    println!(
        "parsed {s:?}, size_of Infallible = {}",
        size_of::<Infallible>()
    );
    // ANCHOR_END: never
}

fn dsts() {
    // ANCHOR: dst
    fn describe<T: Display + ?Sized>(x: &T) -> String {
        format!("[{x}] (pointer is {} bytes)", size_of::<&T>())
    }
    println!("{}", describe(&42)); //                  T = i32       (thin pointer)
    println!("{}", describe("a str")); //              T = str       (pointer + length)
    let d: &dyn Display = &3.5;
    println!("{}", describe(d)); //                    T = dyn Display (pointer + vtable)

    let boxed: Box<str> = "owned DST".into();
    let slice: Box<[u16]> = vec![1, 2, 3].into_boxed_slice();
    println!("{boxed} / {slice:?}");
    println!("size_of_val of the str: {}", std::mem::size_of_val(&*boxed));
    // ANCHOR_END: dst
}

fn zsts() {
    // ANCHOR: zst
    use std::collections::{HashMap, HashSet};
    use std::marker::PhantomData;

    struct Token; // a zero-sized capability
    println!("()            {}", size_of::<()>());
    println!("Token         {}", size_of::<Token>());
    println!("PhantomData   {}", size_of::<PhantomData<String>>());
    println!("[u64; 0]      {}", size_of::<[u64; 0]>());

    let mut units: Vec<()> = Vec::new();
    for _ in 0..1_000_000 {
        units.push(()); // no memory traffic at all
    }
    // Capacity is usize::MAX: a Vec of ZSTs never needs to allocate.
    println!("Vec<()> len {} capacity {}", units.len(), units.capacity());

    let as_set: HashSet<&str> = ["a", "b"].into_iter().collect();
    let as_map: HashMap<&str, ()> = ["a", "b"].into_iter().map(|k| (k, ())).collect();
    println!("set {} == map {}", as_set.len(), as_map.len());
    let _ = Token;
    // ANCHOR_END: zst
}

fn niches() {
    // ANCHOR: niches
    println!("&u8                  {}", size_of::<&u8>());
    println!("Option<&u8>          {}", size_of::<Option<&u8>>());
    println!(
        "Option<Box<[u8; 64]>> {}",
        size_of::<Option<Box<[u8; 64]>>>()
    );
    println!("Option<NonZero<u32>> {}", size_of::<Option<NonZero<u32>>>());
    println!("Option<u32>          {}", size_of::<Option<u32>>());
    println!("Option<char>         {}", size_of::<Option<char>>());
    println!("Option<Option<bool>> {}", size_of::<Option<Option<bool>>>());
    println!(
        "Option<String>       {} (same as String)",
        size_of::<Option<String>>()
    );
    println!("Result<u32, ()>      {}", size_of::<Result<u32, ()>>());
    // ANCHOR_END: niches
}

fn fn_types() {
    // ANCHOR: fn_types
    fn double(x: i32) -> i32 {
        x * 2
    }
    fn negate(x: i32) -> i32 {
        -x
    }

    let item = double; // a unique, zero-sized fn item type
    let pointer: fn(i32) -> i32 = double; // coerced to a fn pointer
    let offset = 10;
    let closure = move |x: i32| x + offset; // captures one i32

    println!("fn item:  {} bytes", std::mem::size_of_val(&item));
    println!("fn ptr:   {} bytes", std::mem::size_of_val(&pointer));
    println!(
        "closure:  {} bytes (its capture)",
        std::mem::size_of_val(&closure)
    );

    let table: [fn(i32) -> i32; 2] = [double, negate]; // one type for both
    println!("{:?}", table.iter().map(|f| f(21)).collect::<Vec<_>>());
    println!("{} {}", item(1), closure(1));
    // ANCHOR_END: fn_types
}

fn pinning() {
    // ANCHOR: pin
    // Unpin types: pinning changes nothing, you can still get &mut.
    let mut number = 5;
    let mut pinned: Pin<&mut i32> = Pin::new(&mut number);
    *pinned.as_mut().get_mut() += 1; // allowed because i32: Unpin
    println!("pinned Unpin value: {}", *pinned);

    // A !Unpin type: PhantomPinned opts out of Unpin.
    struct SelfAware {
        value: u32,
        _pin: PhantomPinned,
    }
    impl SelfAware {
        fn value(self: Pin<&Self>) -> u32 {
            self.value
        }
    }
    let stack_pinned: Pin<&mut SelfAware> = pin!(SelfAware {
        value: 7,
        _pin: PhantomPinned
    });
    println!("value through Pin: {}", stack_pinned.as_ref().value());
    // `Pin::get_mut` isn't available here: SelfAware is !Unpin.

    let heap_pinned: Pin<Box<SelfAware>> = Box::pin(SelfAware {
        value: 9,
        _pin: PhantomPinned,
    });
    println!("Box::pin value: {}", heap_pinned.as_ref().value());

    fn assert_unpin<T: Unpin>(_: &T) -> &'static str {
        "yes" // compiles only if T: Unpin
    }
    println!("String: Unpin? {}", assert_unpin(&String::new()));
    // assert_unpin(&*heap_pinned); // error: PhantomPinned cannot be unpinned
    // ANCHOR_END: pin
}

fn rpitit() {
    // ANCHOR: rpitit
    trait Fleet {
        fn ids(&self) -> impl Iterator<Item = u32> + '_;
    }

    struct Garage(Vec<u32>);
    struct Range(u32, u32);

    impl Fleet for Garage {
        fn ids(&self) -> impl Iterator<Item = u32> + '_ {
            self.0.iter().copied() // one concrete iterator type...
        }
    }
    impl Fleet for Range {
        fn ids(&self) -> impl Iterator<Item = u32> + '_ {
            self.0..self.1 // ...and a completely different one here
        }
    }

    fn total<F: Fleet>(f: &F) -> u32 {
        f.ids().sum()
    }
    println!(
        "{} {}",
        total(&Garage(vec![1, 2, 3])),
        total(&Range(10, 13))
    );
    // ANCHOR_END: rpitit
}

fn gats() {
    // ANCHOR: gat
    trait Container {
        type Iter<'a>: Iterator<Item = &'a u32>
        where
            Self: 'a;
        fn items<'a>(&'a self) -> Self::Iter<'a>;
    }

    struct Readings(Vec<u32>);
    impl Container for Readings {
        type Iter<'a> = std::slice::Iter<'a, u32>; // borrows from &'a self
        fn items<'a>(&'a self) -> Self::Iter<'a> {
            self.0.iter()
        }
    }

    fn max_item<C: Container>(c: &C) -> Option<&u32> {
        c.items().max()
    }
    let r = Readings(vec![512, 530, 498]);
    println!(
        "max {:?}, all {:?}",
        max_item(&r),
        r.items().collect::<Vec<_>>()
    );
    // ANCHOR_END: gat
}
