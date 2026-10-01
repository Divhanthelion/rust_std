//! Lesson: Closures.

use crate::lesson::{Lesson, Question, Section};

pub static LESSON: Lesson = Lesson {
    id: "closures",
    title: "Closures",
    summary: "Anonymous functions that capture their environment: syntax, capture modes, Fn/FnMut/FnOnce, move, passing and returning closures, and callbacks.",
    source: include_str!("closures.rs"),
    sections: &[
        Section::new(
            "Syntax and inference",
            r#"
            A closure is an anonymous function written `|params| body`.
            Parameter and return types are usually inferred. The body can be a
            single expression or a block.

            Every closure has its own unique, unnameable type. Its parameter
            types are inferred from the **first** use and then fixed, so
            calling the same closure with two different types fails:

            ```compile_fail,E0308
            let identity = |x| x;
            let a = identity(5);
            let b = identity("five"); // error: expected integer, found &str
            ```
            "#,
        )
        .demo("syntax", syntax),
        Section::new(
            "Capturing the environment",
            r#"
            Unlike `fn` items, closures can use variables from the scope where
            they're defined. The compiler picks the **least powerful** capture
            that works for each variable:

            ```text
            by shared reference (&T)      the closure only reads it
            by mutable reference (&mut T) the closure modifies it
            by value (T)                  the closure moves or consumes it
            ```

            Since edition 2021, closures capture **disjoint fields**. A closure
            using `car.speed` borrows only that field, so you can still use
            `car.name` elsewhere at the same time.
            "#,
        )
        .demo("capture", capture),
        Section::new(
            "Fn, FnMut and FnOnce",
            r#"
            How a closure captures decides which **call traits** it implements.
            They form a spectrum of what the **caller** can do:

            ```text
            Fn       call any number of times, even concurrently (only reads captures)
            FnMut    call many times, but exclusively (mutates captures)
            FnOnce   call at most once (consumes captures)
            ```

            Each level includes the ones below it: every `Fn` is also `FnMut`,
            and every `FnMut` is also `FnOnce`. So a function **accepting**
            `FnOnce` is the most permissive for callers, and one accepting `Fn`
            is the most restrictive. Ask for the weakest bound you need:
            `Option::map` takes `FnOnce` because it calls the closure at most
            once, and `Iterator::map` takes `FnMut`.
            "#,
        )
        .demo("traits", fn_traits),
        Section::new(
            "move closures",
            r#"
            `move |…| …` forces every captured variable to be captured **by
            value**, even if the body only reads it. You need it when the
            closure outlives the current scope: spawning a thread, returning a
            closure, or storing a callback. For `Copy` types, `move` copies;
            for others, it transfers ownership into the closure.

            Note that `move` decides **how variables are captured**. It does
            not decide which `Fn*` trait the closure implements. A `move`
            closure that only reads its captures is still `Fn`.
            "#,
        )
        .demo("move", move_closures),
        Section::new(
            "Passing closures to functions",
            r#"
            Three ways to accept "something callable", from most to least
            static:

            - **generic** `F: Fn(i32) -> i32`, or `impl Fn(i32) -> i32`, is
              monomorphized and inlinable;
            - **trait object** `&dyn Fn(i32) -> i32` gives one compiled copy
              and dynamic dispatch;
            - **function pointer** `fn(i32) -> i32` accepts plain functions and
              closures that capture nothing, since those coerce.
            "#,
        )
        .demo("passing", passing),
        Section::new(
            "Returning closures",
            r#"
            Return `impl Fn(…) -> …` when there's one closure type, almost
            always with `move` so it owns what it captured. To return one of
            several closures, box them as `Box<dyn Fn(…) -> …>`. This is how
            you write function factories, such as a configurable filter or a
            counter generator.
            "#,
        )
        .demo("returning", returning),
        Section::new(
            "Storing closures: callbacks and strategies",
            r#"
            A struct can hold a closure in a generic field (`struct Retry<F: Fn()
            -> bool>`). That's zero-cost, but the struct's type then depends on
            the closure. A `Vec<Box<dyn Fn(&Event)>>` is the classic **callback
            registry**: different closures, one collection. Event buses, UI
            handlers and signal subscribers all work this way.
            "#,
        )
        .demo("storing", storing),
        Section::new(
            "Closures across std",
            r#"
            The standard library takes closures everywhere: `sort_by_key`,
            `retain`, `Option::map_or_else`, `HashMap::entry(…).or_insert_with`,
            `thread::spawn`, `iter::from_fn`, `std::panic::catch_unwind`… A
            fluent Rust programmer reads `|x| …` as easily as a loop.
            "#,
        )
        .demo("std_uses", std_uses),
    ],
    quiz: &[
        Question::new(
            "A closure increments a captured counter. Which trait does it implement at most?",
            &["Fn", "FnMut", "FnOnce only", "None"],
            1,
            "Mutating a capture requires exclusive access per call, so it is FnMut (and therefore also FnOnce), but not Fn.",
        ),
        Question::new(
            "Which bound should a function use if it calls the closure exactly once?",
            &["Fn", "FnMut", "FnOnce", "fn()"],
            2,
            "FnOnce is the weakest requirement, so it accepts the most closures, including ones that consume captures.",
        ),
        Question::new(
            "What does `move` change?",
            &[
                "It makes the closure FnOnce",
                "It forces captures by value instead of by reference",
                "It moves the closure to the heap",
                "It makes the closure Send",
            ],
            1,
            "`move` affects capture mode only. The Fn* traits implemented depend on what the body does with the captures.",
        ),
        Question::new(
            "Which closure can coerce to a `fn(i32) -> i32` pointer?",
            &[
                "|x| x + offset",
                "|x| x * 2",
                "move |x| x + offset",
                "None of them",
            ],
            1,
            "Only non-capturing closures coerce to function pointers. The others capture `offset`.",
        ),
        Question::new(
            "How do you return one of two different closures from a function?",
            &["impl Fn", "Box<dyn Fn>", "fn pointer only", "You can't"],
            1,
            "Each closure has a distinct type, and `impl Fn` names one type. Boxing erases the type behind a trait object.",
        ),
    ],
    exercises: &[
        "Write `fn apply_n<F: FnMut()>(n: usize, mut f: F)` and use it with a closure that pushes into a captured Vec.",
        "Write `fn make_limiter(max: u32) -> impl FnMut(u32) -> u32` that returns the input clamped, and counts (in its own state) how many values were clamped.",
        "Build an `EventBus` with `subscribe(&mut self, f: Box<dyn Fn(&str)>)` and `publish(&self, msg: &str)`. Register three subscribers that capture different data.",
        "Sort a list of `(name, speed)` tuples by speed descending, then by name, using `sort_by` with a closure and `then_with`.",
    ],
};

fn syntax() {
    // ANCHOR: syntax
    let add_one = |x: i32| -> i32 { x + 1 }; // fully annotated
    let double = |x| x * 2; //                   inferred
    let greet = || println!("no parameters");
    let clamp_to_pct = |v: f64| {
        let clamped = v.clamp(0.0, 100.0);
        clamped.round()
    };

    println!("{} {}", add_one(1), double(21));
    greet();
    println!("{} {}", clamp_to_pct(140.7), clamp_to_pct(42.4));

    let mut counter = 0;
    let mut tick = || counter += 1; // mutates a capture: needs `mut` binding
    tick();
    tick();
    println!("counter = {counter}");
    // ANCHOR_END: syntax
}

fn capture() {
    // ANCHOR: capture
    let limit = 120; //                     captured by &
    let mut log: Vec<String> = Vec::new(); // captured by &mut
    let unit = String::from("km/h"); //     captured by value (moved)

    let is_speeding = |speed: u32| speed > limit;
    let mut record = |msg: &str| log.push(msg.to_string());
    let consume_unit = move || unit; // returns the String: consumes it

    println!("130 speeding? {}", is_speeding(130));
    record("first");
    record("second");
    println!("log = {log:?}");
    let u = consume_unit();
    println!("took {u}");

    // Disjoint field capture (edition 2021+):
    struct Car {
        name: String,
        speed: u32,
    }
    let mut car = Car {
        name: "GR86".into(),
        speed: 0,
    };
    let mut accelerate = || car.speed += 10; // borrows only car.speed
    accelerate();
    println!("{} still readable while the closure lives", car.name);
    accelerate();
    println!("{} at {}", car.name, car.speed);
    // ANCHOR_END: capture
}

fn fn_traits() {
    // ANCHOR: traits
    fn call_twice<F: Fn() -> String>(f: F) -> String {
        format!("{} / {}", f(), f())
    }
    fn call_mut_twice<F: FnMut()>(mut f: F) {
        f();
        f();
    }
    fn call_once<F: FnOnce() -> Vec<u8>>(f: F) -> Vec<u8> {
        f()
    }

    let name = String::from("ECU");
    println!("{}", call_twice(|| name.clone())); // Fn: only reads

    let mut count = 0;
    call_mut_twice(|| count += 1); // FnMut: modifies
    println!("count = {count}");

    let buffer = vec![1, 2, 3];
    let out = call_once(move || buffer); // FnOnce: gives away its capture
    println!("got {out:?}");

    // An Fn closure is accepted where FnMut or FnOnce is asked for:
    let shout = || println!("  (Fn closure used as FnMut)");
    call_mut_twice(shout);
    // ANCHOR_END: traits
}

fn move_closures() {
    // ANCHOR: move
    use std::thread;

    let readings = vec![88.0, 89.5, 91.0];
    // The thread may outlive this function, so it must own its data:
    let handle = thread::spawn(move || {
        let avg = readings.iter().sum::<f64>() / readings.len() as f64;
        format!("average {avg:.2} over {} readings", readings.len())
    });
    println!("{}", handle.join().unwrap());
    // println!("{readings:?}"); // error: moved into the closure

    let threshold = 90.0; // f64 is Copy: `move` copies it
    let above = move |x: f64| x > threshold;
    println!(
        "91 above? {}  threshold still usable: {threshold}",
        above(91.0)
    );
    // ANCHOR_END: move
}

fn passing() {
    // ANCHOR: passing
    fn apply_generic<F: Fn(i32) -> i32>(f: F, x: i32) -> i32 {
        f(x) // monomorphized per closure type
    }
    fn apply_dyn(f: &dyn Fn(i32) -> i32, x: i32) -> i32 {
        f(x) // one copy, vtable call
    }
    fn apply_ptr(f: fn(i32) -> i32, x: i32) -> i32 {
        f(x) // plain function pointer
    }
    fn triple(x: i32) -> i32 {
        x * 3
    }

    let offset = 10;
    let add_offset = |x| x + offset;
    println!("{}", apply_generic(add_offset, 1));
    println!("{}", apply_dyn(&add_offset, 2));
    println!("{}", apply_ptr(triple, 3));
    println!("{}", apply_ptr(|x| x - 1, 4)); // non-capturing closure → fn pointer
    // apply_ptr(add_offset, 5); // error: closure captures `offset`
    // ANCHOR_END: passing
}

fn returning() {
    // ANCHOR: returning
    fn make_adder(n: i32) -> impl Fn(i32) -> i32 {
        move |x| x + n // `move`: the closure must own n
    }

    fn make_counter() -> impl FnMut() -> u32 {
        let mut count = 0;
        move || {
            count += 1;
            count
        }
    }

    fn make_filter(kind: &str) -> Box<dyn Fn(u32) -> bool> {
        match kind {
            "even" => Box::new(|x| x % 2 == 0),
            "big" => Box::new(|x| x > 100),
            _ => Box::new(|_| true),
        }
    }

    let add5 = make_adder(5);
    println!("add5(10) = {}", add5(10));

    let mut next_id = make_counter();
    println!("ids: {} {} {}", next_id(), next_id(), next_id());

    let values = [4, 7, 150, 200, 33];
    for kind in ["even", "big", "all"] {
        let keep = make_filter(kind);
        let kept: Vec<_> = values.iter().copied().filter(|&v| keep(v)).collect();
        println!("{kind:>4}: {kept:?}");
    }
    // ANCHOR_END: returning
}

fn storing() {
    // ANCHOR: storing
    #[derive(Debug)]
    enum Event {
        DoorOpened(u8),
        SpeedChanged(u32),
    }

    struct EventBus {
        handlers: Vec<Box<dyn Fn(&Event)>>,
    }
    impl EventBus {
        fn subscribe(&mut self, handler: impl Fn(&Event) + 'static) {
            self.handlers.push(Box::new(handler));
        }
        fn publish(&self, event: Event) {
            for handler in &self.handlers {
                handler(&event);
            }
        }
    }

    let mut bus = EventBus {
        handlers: Vec::new(),
    };
    bus.subscribe(|e| println!("  logger: {e:?}"));
    let limit = 100;
    bus.subscribe(move |e| {
        if let Event::SpeedChanged(s) = e
            && *s > limit
        {
            println!("  warning: {s} km/h exceeds {limit}");
        }
    });
    bus.subscribe(|e| {
        if let Event::DoorOpened(door) = e {
            println!("  chime: door {door} open");
        }
    });

    bus.publish(Event::SpeedChanged(80));
    bus.publish(Event::SpeedChanged(130));
    bus.publish(Event::DoorOpened(2));
    // ANCHOR_END: storing
}

fn std_uses() {
    // ANCHOR: std_uses
    use std::collections::HashMap;

    let mut parts = vec![("gasket", 3), ("bolt", 120), ("filter", 0), ("pump", 1)];
    parts.sort_by_key(|&(_, qty)| qty);
    println!("by quantity: {parts:?}");
    parts.retain(|&(_, qty)| qty > 0);
    println!("in stock:    {parts:?}");

    let mut by_letter: HashMap<char, Vec<&str>> = HashMap::new();
    for (name, _) in &parts {
        let first = name.chars().next().unwrap_or('?');
        by_letter
            .entry(first)
            .or_insert_with(|| Vec::with_capacity(4))
            .push(*name);
    }
    let mut keys: Vec<_> = by_letter.keys().collect();
    keys.sort();
    println!("letters: {keys:?}");

    let maybe: Option<u32> = None;
    let text = maybe.map_or_else(|| "n/a".to_string(), |v| v.to_string());
    println!("map_or_else → {text}");
    // ANCHOR_END: std_uses
}
