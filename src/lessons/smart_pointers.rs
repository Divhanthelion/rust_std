//! Lesson: Smart pointers & interior mutability.

use std::borrow::Cow;
use std::cell::{Cell, LazyCell, OnceCell, RefCell};
use std::rc::{Rc, Weak};

use crate::lesson::{Lesson, Question, Section};

pub static LESSON: Lesson = Lesson {
    id: "smart_pointers",
    title: "Smart pointers & interior mutability",
    summary: "Box, Rc, Weak, Cell, RefCell, Cow, OnceCell and LazyCell — and the spectrum from compile-time to run-time checking.",
    source: include_str!("smart_pointers.rs"),
    sections: &[
        Section::new(
            "Box<T>: one owner, on the heap",
            r#"
            `Box<T>` puts a value on the heap and owns it. It's a single
            pointer, and dropping the box frees the value. Reach for it when:

            - a type would otherwise have infinite size, as with recursive enums
              (Lesson 11);
            - you need a trait object: `Box<dyn Trait>`;
            - a value is large and you want to move it cheaply (moving a box
              copies one pointer);
            - you need a stable heap address.

            `Box` implements `Deref`, so `*b` and method calls work just like
            on the value itself.
            "#,
        )
        .demo("box", boxes),
        Section::new(
            "Rc<T>: shared ownership",
            r#"
            Sometimes one value has several owners: a configuration shared by
            many components, or a node in a graph. `Rc<T>` (reference counted)
            keeps a count of owners. `Rc::clone(&rc)` increments it, which is
            cheap and doesn't copy `T`. Each drop decrements it, and the value
            is freed when it reaches zero.

            `Rc` gives **shared** access, so you can't mutate through it
            directly. It's also **not thread-safe**: the count isn't atomic,
            so `Rc` is `!Send`. Use `Arc` (Lesson 23) across threads.
            "#,
        )
        .demo("rc", rc_demo),
        Section::new(
            "Weak<T>: references that don't keep values alive",
            r#"
            Two `Rc`s pointing at each other never reach count zero, so they
            **leak**. `Weak<T>` is a non-owning pointer. It doesn't count
            toward keeping the value alive, and `upgrade()` returns
            `Option<Rc<T>>`, which is `None` if the value is gone. Use `Rc`
            for "owns" edges (parent → child) and `Weak` for back-references
            (child → parent).
            "#,
        )
        .demo("weak", weak_demo),
        Section::new(
            "Interior mutability: Cell<T>",
            r#"
            Rust normally ties mutation to exclusive access (`&mut`). **Interior
            mutability** types allow mutation through a shared `&` reference
            by enforcing safety some other way.

            `Cell<T>` never hands out references to its contents. You `get` a
            copy, `set`, `replace`, `take`, or `update` (Rust 1.88+) the value as
            a whole. With no references inside, no aliasing problem is possible,
            so it costs nothing at run time. It's ideal for counters and flags
            in single-threaded code.
            "#,
        )
        .demo("cell", cell_demo),
        Section::new(
            "RefCell<T>: borrow checking at run time",
            r#"
            `RefCell<T>` does hand out references. `borrow()` returns a `Ref`
            and `borrow_mut()` a `RefMut`. It enforces the usual rule of many
            readers or one writer, but **at run time**, with a counter.
            Breaking the rule **panics**. `try_borrow_mut()` returns a
            `Result` instead.

            Use it when the borrow pattern is correct but too dynamic for the
            compiler to prove, for example observers calling back into shared
            state. A `RefCell` panic is still a bug, merely caught later.
            "#,
        )
        .demo("refcell", refcell_demo),
        Section::new(
            "Rc<RefCell<T>>: shared, mutable, single-threaded",
            r#"
            Combining the two gives multiple owners that can each mutate the
            shared value. It's the single-threaded counterpart of
            `Arc<Mutex<T>>`. It's common in graphs, GUI widget trees and
            simulations. Use it sparingly: a clear single owner, with others
            holding ids or indices, is often simpler.
            "#,
        )
        .demo("rc_refcell", rc_refcell),
        Section::new(
            "The checking spectrum",
            r#"
            Every way of sharing or mutating sits somewhere between "proved by
            the compiler" and "checked while running":

            ```text
                         single-threaded            multi-threaded
            compile time &T / &mut T                &T / &mut T (+ Send/Sync)
            no checks    Cell<T> (copy in/out)      Atomic* (Lesson 24)
            run time     RefCell<T> (panics)        Mutex<T>, RwLock<T> (blocks)
            ownership    Rc<T>, Weak<T>             Arc<T>, sync::Weak<T>
            one-time     OnceCell, LazyCell         OnceLock, LazyLock
            ```

            Move toward run-time checking only when you need to. Each step
            trades a compile-time guarantee for flexibility.
            "#,
        ),
        Section::new(
            "Cow: clone on write",
            r#"
            `Cow<'a, B>` holds either a borrowed `&'a B` or an owned value. Read
            access is free in both cases. `to_mut()` clones **only if** the
            value is still borrowed, then hands back `&mut` to the owned copy.
            Functions that usually return their input unchanged, but sometimes
            must modify it, can avoid allocating in the common case.
            "#,
        )
        .demo("cow", cow_demo),
        Section::new(
            "OnceCell and LazyCell",
            r#"
            `OnceCell<T>` can be written **once** and read many times through
            `&self`. `get_or_init(|| …)` computes the value the first time it's
            needed. `LazyCell<T>` (Rust 1.80+) packages the initializer
            together with the cell, so you just dereference it. These are the
            single-threaded versions of `OnceLock` and `LazyLock`.
            "#,
        )
        .demo("once", once_demo),
        Section::new(
            "Leaks are safe (but still bugs)",
            r#"
            Rust's safety guarantees exclude use-after-free and data races, not
            **leaks**. `mem::forget`, `Box::leak` and `Rc` cycles all leak
            memory, and none of them needs `unsafe`. Deliberate leaks have
            uses, such as `Box::leak` for a `&'static` built at startup.
            Accidental `Rc` cycles are a real bug class. Watch `strong_count`
            and use `Weak` for back-edges.
            "#,
        )
        .demo("leaks", leaks),
    ],
    quiz: &[
        Question::new(
            "What does `Rc::clone(&a)` do?",
            &[
                "Deep-copies the value",
                "Increments the reference count and returns another pointer to the same value",
                "Moves a",
                "Creates a Weak",
            ],
            1,
            "Rc::clone is cheap: it bumps the strong count. The data is shared, not copied.",
        ),
        Question::new(
            "What happens if you call `borrow_mut()` on a RefCell that is already borrowed?",
            &[
                "Compile error",
                "It waits",
                "It panics at run time",
                "It returns a copy",
            ],
            2,
            "RefCell enforces the borrowing rules dynamically and panics on violation. `try_borrow_mut` returns an error instead.",
        ),
        Question::new(
            "Two Rc nodes point at each other with strong references. What happens when the last outside owner is dropped?",
            &[
                "Both are freed",
                "A panic",
                "They leak: each keeps the other's count above zero",
                "A compile error",
            ],
            2,
            "Reference cycles never reach zero. Use Weak for back-references.",
        ),
        Question::new(
            "Which type is the thread-safe counterpart of Rc<RefCell<T>>?",
            &[
                "Box<Cell<T>>",
                "Arc<Mutex<T>>",
                "Rc<Mutex<T>>",
                "Arc<RefCell<T>>",
            ],
            1,
            "Arc's count is atomic (Send + Sync), and Mutex provides exclusive access that blocks instead of panicking.",
        ),
        Question::new(
            "Why does `Cell<T>` need no run-time borrow tracking?",
            &[
                "It's unsafe",
                "It never gives out references to its contents; values are copied or swapped in and out",
                "It only works with integers",
                "It uses a lock",
            ],
            1,
            "Without references into the cell, there's nothing to alias, so the set/get operations are always sound.",
        ),
    ],
    exercises: &[
        "Build a tree where each `Node` has `children: RefCell<Vec<Rc<Node>>>` and `parent: RefCell<Weak<Node>>`. Print strong and weak counts as you add children.",
        "Write a `Counter` struct with a `hits: Cell<u32>` field and a `fn record(&self)` method. Why does `&self` suffice?",
        "Trigger a RefCell double-borrow panic deliberately, then rewrite the code with `try_borrow_mut` to handle it gracefully.",
        "Write `fn normalize_path(p: &str) -> Cow<str>` that replaces backslashes with slashes only if there are any.",
    ],
};

fn boxes() {
    // ANCHOR: box
    let b = Box::new(42u64); // the u64 lives on the heap
    println!("*b + 1 = {}", *b + 1);

    let big = Box::new([0u8; 4096]); // 4 KiB on the heap
    let moved = big; // moving copies one pointer, not 4 KiB
    println!(
        "moved box holds {} bytes; the box itself is {} bytes",
        moved.len(),
        std::mem::size_of::<Box<[u8; 4096]>>()
    );

    #[derive(Debug)]
    enum List {
        Cons(i32, Box<List>),
        Nil,
    }
    use List::{Cons, Nil};
    let list = Cons(1, Box::new(Cons(2, Box::new(Cons(3, Box::new(Nil))))));
    println!("{list:?}");

    let shapes: Vec<Box<dyn Fn(f64) -> f64>> = vec![
        Box::new(|r| std::f64::consts::PI * r * r),
        Box::new(|s| s * s),
    ];
    println!("areas: {:.2} {:.2}", shapes[0](1.0), shapes[1](3.0));
    // ANCHOR_END: box
}

fn rc_demo() {
    // ANCHOR: rc
    #[derive(Debug)]
    struct Calibration {
        offsets: Vec<f32>,
    }

    let shared = Rc::new(Calibration {
        offsets: vec![0.1, -0.2, 0.05],
    });
    println!("count after new: {}", Rc::strong_count(&shared));

    let sensor_a = Rc::clone(&shared); // cheap: bumps the count
    let sensor_b = Rc::clone(&shared);
    println!("count with two sensors: {}", Rc::strong_count(&shared));
    println!("same allocation? {}", Rc::ptr_eq(&sensor_a, &sensor_b));
    println!("sensor_b sees {:?}", sensor_b.offsets);

    drop(sensor_a);
    println!("count after dropping one: {}", Rc::strong_count(&shared));
    // ANCHOR_END: rc
}

fn weak_demo() {
    // ANCHOR: weak
    #[derive(Debug)]
    struct Node {
        name: String,
        parent: RefCell<Weak<Node>>,      //    back-edge: doesn't own
        children: RefCell<Vec<Rc<Node>>>, // owns
    }

    let vehicle = Rc::new(Node {
        name: "vehicle".into(),
        parent: RefCell::new(Weak::new()),
        children: RefCell::new(vec![]),
    });
    let engine = Rc::new(Node {
        name: "engine".into(),
        parent: RefCell::new(Rc::downgrade(&vehicle)),
        children: RefCell::new(vec![]),
    });
    vehicle.children.borrow_mut().push(Rc::clone(&engine));

    let parent_name = engine.parent.borrow().upgrade().map(|p| p.name.clone());
    println!("engine's parent: {parent_name:?}");
    println!(
        "vehicle strong {}, weak {}",
        Rc::strong_count(&vehicle),
        Rc::weak_count(&vehicle)
    );
    println!(
        "engine strong {} (owned by vehicle and by us)",
        Rc::strong_count(&engine)
    );

    drop(vehicle); // no cycle, so the vehicle really is freed
    println!(
        "after dropping vehicle, parent upgrade = {:?}",
        engine.parent.borrow().upgrade().map(|p| p.name.clone())
    );
    // ANCHOR_END: weak
}

fn cell_demo() {
    // ANCHOR: cell
    struct Sensor {
        name: &'static str,
        reads: Cell<u32>, // mutable even through &self
    }
    impl Sensor {
        fn read(&self) -> f64 {
            self.reads.set(self.reads.get() + 1);
            21.5
        }
    }

    let s = Sensor {
        name: "cabin",
        reads: Cell::new(0),
    };
    let r1 = &s;
    let r2 = &s; // two shared references, both can "mutate"
    r1.read();
    r2.read();
    s.reads.update(|n| n + 10); // Rust 1.88+
    println!("{} read {} times", s.name, s.reads.get());
    println!(
        "replace → old {}, take → {}",
        s.reads.replace(100),
        s.reads.take()
    );
    // ANCHOR_END: cell
}

fn refcell_demo() {
    // ANCHOR: refcell
    let log = RefCell::new(Vec::<String>::new());

    log.borrow_mut().push("ignition".into()); // temporary RefMut, released at `;`
    {
        let reader1 = log.borrow();
        let reader2 = log.borrow(); // many readers: fine
        println!("{} entries, first {:?}", reader1.len(), reader2.first());
        // log.borrow_mut() here would panic: readers are still alive
        match log.try_borrow_mut() {
            Ok(_) => println!("unexpected"),
            Err(e) => println!("try_borrow_mut while reading → {e}"),
        }
    } // readers dropped here
    log.borrow_mut().push("engine start".into());
    println!("{:?}", log.borrow());
    // ANCHOR_END: refcell
}

fn rc_refcell() {
    // ANCHOR: rc_refcell
    #[derive(Debug, Default)]
    struct Dashboard {
        warnings: Vec<String>,
    }

    let dash = Rc::new(RefCell::new(Dashboard::default()));

    // Two subsystems, each holding a handle to the same dashboard:
    let brakes = Rc::clone(&dash);
    let engine = Rc::clone(&dash);

    brakes.borrow_mut().warnings.push("pad wear 85%".into());
    engine.borrow_mut().warnings.push("oil change due".into());

    println!("{:#?}", dash.borrow());
    println!("owners: {}", Rc::strong_count(&dash));
    // ANCHOR_END: rc_refcell
}

fn cow_demo() {
    // ANCHOR: cow
    fn sanitize(label: &str) -> Cow<'_, str> {
        if label.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            Cow::Borrowed(label) // no allocation
        } else {
            let cleaned: String = label
                .chars()
                .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
                .collect();
            Cow::Owned(cleaned)
        }
    }
    for label in ["engine_rpm", "oil temp (°C)"] {
        let s = sanitize(label);
        let kind = if matches!(s, Cow::Borrowed(_)) {
            "borrowed"
        } else {
            "owned"
        };
        println!("{label:?} → {s:?} ({kind})");
    }

    let base = [1, 2, 3];
    let mut data: Cow<[i32]> = Cow::Borrowed(&base);
    println!(
        "before to_mut: borrowed? {}",
        matches!(data, Cow::Borrowed(_))
    );
    data.to_mut().push(4); // clones the slice into a Vec now
    println!("after: {:?}, base untouched {:?}", data, base);
    // ANCHOR_END: cow
}

fn once_demo() {
    // ANCHOR: once
    let serial: OnceCell<String> = OnceCell::new();
    let get_serial = || {
        serial.get_or_init(|| {
            println!("  (reading serial number from EEPROM)");
            "SN-00042".to_string()
        })
    };
    println!("{}", get_serial());
    println!("{}", get_serial()); // initializer doesn't run again
    println!("set again? {:?}", serial.set("other".into()).is_err());

    let table: LazyCell<Vec<u16>> = LazyCell::new(|| {
        println!("  (building CRC table)");
        (0..4).map(|i| i * 0x1021).collect()
    });
    println!("before first use");
    println!("table[3] = {:#06x}", table[3]); // initialized here
    // ANCHOR_END: once
}

fn leaks() {
    // ANCHOR: leaks
    struct Peer {
        other: RefCell<Option<Rc<Peer>>>,
    }

    let a = Rc::new(Peer {
        other: RefCell::new(None),
    });
    let b = Rc::new(Peer {
        other: RefCell::new(Some(Rc::clone(&a))),
    });
    *a.other.borrow_mut() = Some(Rc::clone(&b)); // a → b → a: a cycle
    println!(
        "a strong = {}, b strong = {}",
        Rc::strong_count(&a),
        Rc::strong_count(&b)
    );
    // When `a` and `b` go out of scope, the counts drop to 1, not 0: leaked.
    // Breaking the cycle by hand fixes it:
    a.other.borrow_mut().take();
    println!("after breaking: b strong = {}", Rc::strong_count(&b));

    let config: &'static str = Box::leak(String::from("loaded once").into_boxed_str());
    println!("deliberately leaked: {config}");
    // ANCHOR_END: leaks
}
