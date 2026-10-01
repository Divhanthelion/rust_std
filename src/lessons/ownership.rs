//! Lesson: Ownership & moves.

use crate::lesson::{Lesson, Question, Section};

pub static LESSON: Lesson = Lesson {
    id: "ownership",
    title: "Ownership & moves",
    summary: "How Rust manages memory without a garbage collector: owners, moves, Copy, Clone, Drop, and mem::take/replace.",
    source: include_str!("ownership.rs"),
    sections: &[
        Section::new(
            "Why ownership?",
            r#"
            Every program must decide when to free memory. Languages sit on a
            spectrum:

            ```text
            manual (C)            you call free — fast, but use-after-free and
                                  double-free bugs are easy to write
            reference counting    runtime bookkeeping; cycles can leak
            tracing GC (Java, Go) safe, but runtime cost and unpredictable pauses
            ownership (Rust)      the compiler inserts the frees — checked at
                                  compile time, deterministic, no runtime cost
            ```

            Determinism matters in embedded and automotive systems. You can
            tell from the source code exactly where memory is released, and no
            collector will pause the brake controller.

            Some background: **stack** values have a size known at compile time
            and are freed by popping the frame. **Heap** values, like the
            characters of a `String` or the elements of a `Vec`, are allocated
            at run time and must be freed by someone. Ownership answers "by
            whom, and when".
            "#,
        ),
        Section::new(
            "The three rules",
            r#"
            1. Each value has an **owner** (a variable, a field, a collection
               slot…).
            2. There is exactly **one** owner at a time.
            3. When the owner goes out of scope, the value is **dropped**: its
               destructor runs and its memory is freed.

            That's the whole model. Everything else in this lesson and the
            next follows from these rules plus borrowing.
            "#,
        )
        .demo("rules", scope_and_drop),
        Section::new(
            "Moves",
            r#"
            Assigning a `String` to another variable, passing it to a function,
            or returning it **moves** it. The three-word header (pointer,
            length, capacity) is copied, and the source is statically marked
            as unusable. No characters are copied, and there's no double free,
            because only the new owner will drop it:

            ```compile_fail,E0382
            let a = String::from("brake");
            let b = a;           // ownership moves to b
            println!("{a}");     // error: borrow of moved value: `a`
            ```

            A move is a compile-time concept. Nothing at run time tracks it;
            the compiler simply refuses programs that use a moved-from value.
            Loops are a common place to trip:

            ```compile_fail,E0382
            let names = vec![String::from("a")];
            for _ in 0..2 {
                let taken = names; // moved in the first iteration
            }
            ```
            "#,
        )
        .demo("moves", moves),
        Section::new(
            "Copy, Clone and moves: a spectrum",
            r#"
            ```text
            Copy    implicit, bitwise, cheap        integers, floats, bool, char,
                                                    &T, tuples/arrays of Copy types
            Clone   explicit .clone(), may allocate String, Vec, HashMap, …
            move    the default for everything else ownership transfers; source unusable
            ```

            A `Copy` type is duplicated on assignment, so the source stays
            valid. A type can only be `Copy` if a bitwise copy is a correct
            duplicate. That means it can't own heap memory and can't implement
            `Drop`. `Clone` makes duplication explicit and visible in the code,
            so expensive copies never hide.
            "#,
        )
        .demo("copy_clone", copy_and_clone),
        Section::new(
            "Ownership through functions",
            r#"
            Passing a value to a function moves it into the parameter, and
            returning a value moves it out to the caller. A function that takes
            `String` consumes its argument; one that returns `String` hands
            ownership back.

            Threading ownership in and out like this works, but it's clumsy.
            Usually you want to let a function **use** a value without taking it.
            That's borrowing, the next lesson. Choosing between them is a
            spectrum too: take `T` when the function needs to keep or destroy
            the value, take `&mut T` to modify it, and take `&T` to read it.
            "#,
        )
        .demo("functions", ownership_and_functions),
        Section::new(
            "Drop and drop order",
            r#"
            Implementing the `Drop` trait runs your code when a value is
            destroyed. This is **RAII** (Resource Acquisition Is
            Initialization): files close, locks unlock and sockets disconnect
            exactly when their owner goes out of scope, without a `finally`
            block.

            Drop order is defined:

            - Locals drop in **reverse** order of declaration, last in, first out.
            - Struct fields drop in **declaration** order, after the struct's
              own `drop`.
            - Vec elements drop in order, front to back.

            To drop something early, call `std::mem::drop(value)` (also in the
            prelude as `drop`). Calling the `Drop::drop` method yourself is
            forbidden, because it would run twice:

            ```compile_fail,E0040
            struct Guard;
            impl Drop for Guard { fn drop(&mut self) {} }
            let g = Guard;
            g.drop(); // error: explicit use of destructor method
            ```
            "#,
        )
        .demo("drop_order", drop_order),
        Section::new(
            "Partial moves and moving out of collections",
            r#"
            You can move a field out of a struct you own. The other fields stay
            usable, but the struct as a whole doesn't. You **can't** move an
            element out of a `Vec` by indexing, because that would leave a hole
            in the vector:

            ```compile_fail,E0507
            let v = vec![String::from("x")];
            let s = v[0]; // error: cannot move out of index of `Vec<String>`
            ```

            Instead, borrow it (`&v[0]`), clone it, or remove it with a method
            that keeps the collection consistent: `remove`, `swap_remove`,
            `pop`, `drain`, or `into_iter`.
            "#,
        )
        .demo("partial", partial_moves),
        Section::new(
            "mem::take, mem::replace and mem::swap",
            r#"
            What if you need to move a value out of something you only have a
            `&mut` to, such as a field of `self`? You can't leave it empty, but
            you can **leave something else behind**:

            - `mem::take(&mut x)` returns `x` and puts `Default::default()` in
              its place.
            - `mem::replace(&mut x, new)` returns `x` and puts `new` in its place.
            - `mem::swap(&mut a, &mut b)` exchanges two values.
            - `Option::take()` is the same idea for options, leaving `None`.

            These come up in interviews and in real state machines alike.
            "#,
        )
        .demo("take", take_replace_swap),
    ],
    quiz: &[
        Question::new(
            "After `let b = a;` where `a: String`, what is true?",
            &[
                "a and b point to separate copies of the text",
                "a and b share the text; it's freed when both are gone",
                "b owns the text; using a is a compile error",
                "a owns the text; b is a reference",
            ],
            2,
            "Assignment moves a non-Copy value. The heap data isn't copied, and the compiler rejects later uses of `a`.",
        ),
        Question::new(
            "Why can't a type be both `Copy` and implement `Drop`?",
            &[
                "Copy types are too small",
                "A bitwise copy would let the destructor run on two copies of the same resource",
                "Drop requires heap allocation",
                "It can — it's just unusual",
            ],
            1,
            "If a value with a destructor were silently duplicated, the resource it manages would be released twice.",
        ),
        Question::new(
            "Two locals are declared `let a = Noisy(\"a\"); let b = Noisy(\"b\");`. In what order are they dropped?",
            &["a then b", "b then a", "Unspecified", "Neither is dropped"],
            1,
            "Locals are dropped in reverse order of declaration.",
        ),
        Question::new(
            "How do you move a `String` out of `self.name` when you only have `&mut self`?",
            &[
                "`let n = self.name;`",
                "`let n = std::mem::take(&mut self.name);`",
                "`let n = &self.name;`",
                "It's impossible",
            ],
            1,
            "`mem::take` swaps in `String::default()` (empty) and returns the old value, so `self` stays valid.",
        ),
        Question::new(
            "What does a move cost at run time for a `Vec<u8>` holding 1 MB?",
            &[
                "Copying 1 MB",
                "Copying the three-word header (pointer, length, capacity)",
                "Incrementing a reference count",
                "Nothing at all, ever",
            ],
            1,
            "A move copies the stack representation (and the optimizer often removes even that). The heap buffer stays where it is.",
        ),
    ],
    exercises: &[
        "Write a struct `Trace(&'static str)` with a `Drop` impl that prints. Put three in a `Vec`, two as locals, and one inside a struct with another as a field. Predict the full drop order, then run it.",
        "Write `fn consume(s: String) -> usize` and call it in a loop over a `Vec<String>` without cloning. (Hint: what does `for s in v` do?)",
        "Implement `fn rotate(names: &mut Vec<String>)` that moves the first element to the end without cloning any String.",
        "Given `struct Job { log: Vec<String> }`, write `fn finish(&mut self) -> Vec<String>` that hands the log to the caller and leaves an empty one behind.",
    ],
};

// A helper type for this lesson: it announces its own destruction.
struct Noisy(&'static str);

impl Drop for Noisy {
    fn drop(&mut self) {
        println!("  drop({})", self.0);
    }
}

fn scope_and_drop() {
    // ANCHOR: rules
    println!("enter outer scope");
    let outer = String::from("outer"); // `outer` owns a heap buffer
    {
        let inner = String::from("inner");
        println!("inner scope sees {inner} and {outer}");
    } // ← `inner` is dropped here: its buffer is freed
    println!("back outside, `inner` no longer exists");
    let _n = Noisy("n");
    println!("about to leave");
    // ANCHOR_END: rules
}

fn moves() {
    // ANCHOR: moves
    let a = String::from("brake fluid");
    let b = a; // move: `a` is now unusable
    println!("b = {b}");

    fn shout(s: String) -> String {
        s.to_uppercase() // `s` dropped at the end; a new String is returned
    }
    let c = shout(b); // `b` moves into the function
    println!("c = {c}");

    // Moving a pointer, not the data: same heap address before and after.
    let v = vec![1, 2, 3];
    let before = v.as_ptr();
    let moved = v;
    println!("same buffer after move? {}", before == moved.as_ptr());
    // ANCHOR_END: moves
}

fn copy_and_clone() {
    // ANCHOR: copy_clone
    let x = 5; // i32 is Copy
    let y = x; // a copy; x is still fine
    println!("x = {x}, y = {y}");

    let point = (1.5, 2.5); // tuples of Copy types are Copy
    let p2 = point;
    println!("{point:?} {p2:?}");

    let original = vec![String::from("a"), String::from("b")];
    let mut copy = original.clone(); // explicit deep copy: allocates
    copy.push(String::from("c"));
    println!("original {original:?}, clone {copy:?}");

    let r = &original; // shared references are Copy too
    let r2 = r;
    println!("two refs to the same Vec: {} {}", r.len(), r2.len());
    // ANCHOR_END: copy_clone
}

fn ownership_and_functions() {
    // ANCHOR: functions
    fn take(s: String) {
        println!("  take() now owns {s:?}");
    } // s dropped here

    fn give() -> String {
        String::from("fresh")
    }

    fn take_and_give_back(mut s: String) -> String {
        s.push('!');
        s
    }

    let s = give();
    let s = take_and_give_back(s); // ownership goes in, and back out
    println!("got back {s:?}");
    take(s);
    // println!("{s}"); // error: `s` was moved into take()
    // ANCHOR_END: functions
}

fn drop_order() {
    // ANCHOR: drop_order
    struct Car {
        _engine: Noisy,
        _wheels: Noisy,
    }

    let _first = Noisy("first local");
    let _second = Noisy("second local");
    let _car = Car {
        _engine: Noisy("car.engine"),
        _wheels: Noisy("car.wheels"),
    };
    let early = Noisy("early");
    drop(early); // dropped now, not at the end
    let _list = vec![Noisy("vec[0]"), Noisy("vec[1]")];
    println!("end of scope — watch the order:");
    // ANCHOR_END: drop_order
}

fn partial_moves() {
    // ANCHOR: partial
    struct Driver {
        name: String,
        license: String,
    }
    let d = Driver {
        name: String::from("Kiichiro"),
        license: String::from("B-123"),
    };
    let name = d.name; // partial move of one field
    println!("moved name: {name}, license still usable: {}", d.license);
    // println!("{}", d.name); // error: value moved

    let mut queue = vec![
        String::from("job-1"),
        String::from("job-2"),
        String::from("job-3"),
    ];
    let first = queue.remove(0); //      shifts the rest left: O(n)
    let last = queue.swap_remove(0); //  fills the gap with the last: O(1)
    println!("{first} {last}, left: {queue:?}");
    for job in queue {
        // into_iter: moves each element out, consuming the Vec
        println!("processing {job}");
    }
    // ANCHOR_END: partial
}

fn take_replace_swap() {
    // ANCHOR: take
    use std::mem;

    #[derive(Debug, Default)]
    struct Recorder {
        samples: Vec<u16>,
        state: String,
    }

    impl Recorder {
        /// Hands the collected samples to the caller and starts fresh.
        fn flush(&mut self) -> Vec<u16> {
            mem::take(&mut self.samples) // leaves Vec::new() behind
        }
        fn set_state(&mut self, new: &str) -> String {
            mem::replace(&mut self.state, new.to_string()) // returns the old state
        }
    }

    let mut rec = Recorder::default();
    rec.samples.extend([512, 515, 509]);
    let batch = rec.flush();
    println!("flushed {batch:?}, recorder now {rec:?}");

    let old = rec.set_state("recording");
    println!("old state {old:?} → {:?}", rec.state);

    let mut a = String::from("left");
    let mut b = String::from("right");
    mem::swap(&mut a, &mut b);
    println!("swapped: a = {a}, b = {b}");

    let mut slot = Some(7);
    let taken = slot.take(); // Option's own take: leaves None
    println!("taken {taken:?}, slot {slot:?}");
    // ANCHOR_END: take
}
