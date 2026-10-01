//! Lesson: Borrowing & references.

use crate::lesson::{Lesson, Question, Section};

pub static LESSON: Lesson = Lesson {
    id: "borrowing",
    title: "Borrowing & references",
    summary: "Shared and exclusive references, the aliasing rule that prevents data races at compile time, NLL, reborrowing and split borrows.",
    source: include_str!("borrowing.rs"),
    sections: &[
        Section::new(
            "References",
            r#"
            A **reference** lets you use a value without owning it. `&x` creates
            a shared reference to `x`, of type `&T`. The owner keeps ownership,
            and the value isn't dropped when the reference goes away. Creating
            a reference is called **borrowing**.

            To get at the value behind a reference, dereference it with `*r`.
            Method calls and field access auto-dereference, so `r.len()` works
            directly. `println!` formats the referenced value, not the address.
            "#,
        )
        .demo("refs", references),
        Section::new(
            "Mutable references",
            r#"
            `&mut x` creates a mutable reference, of type `&mut T`, through
            which the value can be changed. The binding must itself be `mut`,
            because you can't lend out write access you don't have. Assign
            through it with `*r = …`; methods that take `&mut self` work
            directly.
            "#,
        )
        .demo("mut_refs", mutable_references),
        Section::new(
            "The rule: many readers XOR one writer",
            r#"
            At any point, a value can have **either** any number of `&T`
            **or** exactly one `&mut T`, never both. It's clearer to read the
            two kinds as **shared** (`&`) and **exclusive** (`&mut`) than as
            immutable and mutable:

            ```compile_fail,E0499
            let mut s = String::new();
            let a = &mut s;
            let b = &mut s; // error: second exclusive borrow while `a` is live
            a.push('x');
            ```

            ```compile_fail,E0502
            let mut v = vec![1, 2, 3];
            let first = &v[0];   // shared borrow of v...
            v.push(4);           // error: ...while v is mutated
            println!("{first}");
            ```

            The second example isn't pedantry. `push` may reallocate the buffer,
            which would leave `first` pointing at freed memory. That bug class
            is called iterator invalidation, and C++ allows it. The same rule
            makes **data races impossible** in safe Rust: a race needs two
            accesses at once where at least one is a write, which is exactly
            what the rule forbids.
            "#,
        )
        .demo("rule", aliasing_rule),
        Section::new(
            "Borrows end at their last use (NLL)",
            r#"
            A borrow lasts until the reference is **last used**, not until the
            end of the block. This is called non-lexical lifetimes (NLL). So
            you can take a shared borrow, use it, and then mutate, all in one
            scope, as long as the uses don't interleave.
            "#,
        )
        .demo("nll", non_lexical),
        Section::new(
            "No dangling references",
            r#"
            A reference can never outlive the value it points to. Returning a
            reference to a local variable is rejected, because the local is
            dropped when the function returns:

            ```compile_fail,E0515
            fn dangle<'a>() -> &'a String {
                let s = String::from("temporary");
                &s // error: returns a reference to data owned by this function
            }
            ```

            ```compile_fail,E0597
            let r;
            {
                let x = 5;
                r = &x; // error: `x` does not live long enough
            }
            println!("{r}");
            ```

            The fix is almost always to return or store an **owned** value
            instead. Lifetimes (Lesson 9) are the vocabulary for these rules.
            "#,
        )
        .demo("no_dangle", no_dangling),
        Section::new(
            "Borrowing in function signatures",
            r#"
            A function's parameter types say exactly what it may do with its
            arguments, and that is visible at every call site:

            ```text
            fn inspect(v: &Vec<i32>)       reads; caller keeps ownership
            fn modify(v: &mut Vec<i32>)    modifies in place; caller keeps ownership
            fn consume(v: Vec<i32>)        takes ownership; caller loses access
            ```

            Callers write `&x` or `&mut x` explicitly, so lending something out
            for writing is never a surprise. Prefer the most general borrowed
            form: `&str` over `&String`, and `&[T]` over `&Vec<T>`.
            "#,
        )
        .demo("signatures", signatures),
        Section::new(
            "Reborrowing",
            r#"
            When you pass an `&mut T` you hold to a function expecting `&mut T`,
            Rust implicitly **reborrows** it as `&mut *r`. That creates a new,
            shorter exclusive borrow instead of moving your reference away,
            which is why you can keep using `r` after the call. You can also
            reborrow explicitly: `&*r` turns an exclusive reference into a
            shared one for a while.
            "#,
        )
        .demo("reborrow", reborrowing),
        Section::new(
            "Splitting borrows",
            r#"
            The borrow checker tracks struct **fields** separately, so you can
            mutably borrow `car.engine` and `car.wheels` at the same time.

            It can't see inside indexing, though: `&mut v[0]` and `&mut v[1]`
            look like two borrows of `v`. For slices, use `split_at_mut`, which
            proves the halves are disjoint, or `get_disjoint_mut` (Rust 1.86+)
            for arbitrary indices. Inside a method that takes `&mut self`,
            calling another `&mut self` method borrows all of `self`.
            Destructuring `self` into its fields is the usual workaround.
            "#,
        )
        .demo("split", split_borrows),
        Section::new(
            "Moving out of a borrow",
            r#"
            You can read through `&T`, but you can't move a non-`Copy` value out
            of it. That would steal from the owner:

            ```compile_fail,E0507
            fn first_name(names: &Vec<String>) -> String {
                names[0] // error: cannot move out of index of `Vec<String>`
            }
            ```

            The choices form a spectrum of cost: return a reference
            (`&names[0]`), which is free; `clone()` it, which allocates; or
            change the signature to take ownership. With `&mut`, you can also
            `mem::take` it (Lesson 6).
            "#,
        )
        .demo("move_out", moving_out_of_borrow),
    ],
    quiz: &[
        Question::new(
            "Which combination of live borrows of the same value is allowed?",
            &[
                "Two &mut",
                "One &mut and one &",
                "Three &",
                "One &mut and the owner reading directly",
            ],
            2,
            "Any number of shared borrows, or exactly one exclusive borrow. The owner itself can't be used while an exclusive borrow is live.",
        ),
        Question::new(
            "Why does Rust reject holding `&v[0]` across `v.push(4)`?",
            &[
                "push is unsafe",
                "push may reallocate, which would leave the reference dangling",
                "Vec elements can't be borrowed",
                "Because v isn't declared mut",
            ],
            1,
            "Reallocation moves the elements. The aliasing rule turns this classic C++ bug into a compile error.",
        ),
        Question::new(
            "When does a borrow end under non-lexical lifetimes?",
            &[
                "At the end of the enclosing block",
                "At its last use",
                "When the owner is dropped",
                "At the end of the function",
            ],
            1,
            "NLL ends a borrow after the reference's last use, so later code may mutate the value.",
        ),
        Question::new(
            "Which compiles?",
            &[
                "`let a = &mut v[0]; let b = &mut v[1]; *a += *b;`",
                "`let (x, y) = v.split_at_mut(1); x[0] += y[0];`",
                "`let a = &mut v; let b = &v; a.push(b[0]);`",
                "None of them",
            ],
            1,
            "`split_at_mut` returns two non-overlapping `&mut [T]`, which the borrow checker can verify through its signature.",
        ),
        Question::new(
            "What is the best description of `&mut T`?",
            &[
                "A mutable pointer",
                "An exclusive (unique) borrow",
                "A copy that can be modified",
                "A reference-counted pointer",
            ],
            1,
            "The guarantee of `&mut` is uniqueness: while it's live, nobody else can read or write the value. Mutation is a consequence of that guarantee.",
        ),
    ],
    exercises: &[
        "Write `fn longest_word(text: &str) -> &str` and call it on a `String`. Why does it compile without lifetime annotations? (Preview of Lesson 9.)",
        "Write `fn normalize(values: &mut [f64])` that divides every element by the maximum. Handle the empty slice and the all-zero slice.",
        "Using `split_at_mut`, write `fn swap_halves(v: &mut [u8])` for an even-length slice, element by element.",
        "Take this broken code and fix it two ways (once by cloning, once by reordering): `let mut v = vec![1]; let f = &v[0]; v.push(*f);`",
    ],
};

fn references() {
    // ANCHOR: refs
    #[allow(clippy::ptr_arg)] // clippy prefers &str here; see the signatures section
    fn length(s: &String) -> usize {
        s.len() // auto-deref: no need to write (*s).len()
    }

    let model = String::from("Corolla");
    let r = &model; // borrow
    println!("{r} has {} letters", length(r));
    println!("model is still owned here: {model}");

    let n = 10;
    let rn = &n;
    println!("*rn + 1 = {}", *rn + 1); // explicit deref for arithmetic
    println!("rn == &10? {}", rn == &10); // comparing references compares values
    // ANCHOR_END: refs
}

fn mutable_references() {
    // ANCHOR: mut_refs
    fn add_reading(log: &mut Vec<u16>, value: u16) {
        log.push(value);
    }

    let mut log = Vec::new();
    add_reading(&mut log, 500);
    add_reading(&mut log, 512);
    println!("log = {log:?}");

    let mut temp = 20;
    let t = &mut temp;
    *t += 5; // write through the reference
    println!("temp = {temp}");
    // ANCHOR_END: mut_refs
}

fn aliasing_rule() {
    // ANCHOR: rule
    let mut speeds = vec![30, 50, 70];

    // Many shared borrows at once: fine.
    let a = &speeds;
    let b = &speeds;
    println!("readers: {} {}", a.len(), b[0]);

    // One exclusive borrow (the shared ones above are no longer used):
    let w = &mut speeds;
    w.push(90);
    println!("after write: {speeds:?}");

    // To modify while reading, finish reading first:
    let first = speeds[0]; // copy the i32 out; no borrow is kept
    speeds.push(first);
    println!("pushed a copy of the first: {speeds:?}");
    // ANCHOR_END: rule
}

fn non_lexical() {
    // ANCHOR: nll
    let mut text = String::from("abc");
    let r = &text; // shared borrow starts
    println!("reading {r}"); // ...and ends here: last use of r
    text.push('d'); // so mutating is fine
    println!("now {text}");
    // ANCHOR_END: nll
}

fn no_dangling() {
    // ANCHOR: no_dangle
    // Return an owned value instead of a reference to a local:
    fn make_label(id: u32) -> String {
        format!("ECU-{id:03}")
    }
    let label = make_label(7);
    println!("{label}");

    // Returning a reference *into an argument* is fine: the data outlives the call.
    fn first_part(s: &str) -> &str {
        s.split('-').next().unwrap_or(s)
    }
    println!("{}", first_part(&label));
    // ANCHOR_END: no_dangle
}

fn signatures() {
    // ANCHOR: signatures
    fn total(values: &[u32]) -> u32 {
        values.iter().sum()
    }
    fn double_all(values: &mut [u32]) {
        for v in values {
            *v *= 2;
        }
    }
    fn into_report(values: Vec<u32>) -> String {
        format!("{} values, total {}", values.len(), total(&values))
    } // `values` dropped here

    let mut data = vec![1, 2, 3];
    println!("total {}", total(&data));
    double_all(&mut data);
    println!("doubled {data:?}");
    println!("{}", into_report(data));
    // `data` is gone now.
    // ANCHOR_END: signatures
}

fn reborrowing() {
    // ANCHOR: reborrow
    fn bump(counter: &mut u32) {
        *counter += 1;
    }

    let mut count = 0;
    let r = &mut count;
    bump(r); // implicitly `bump(&mut *r)`: r is reborrowed, not moved
    bump(r); // ...so r is still usable
    let shared: &u32 = &*r; // reborrow as shared, temporarily
    println!("shared view: {shared}");
    *r += 10;
    println!("count = {count}");
    // ANCHOR_END: reborrow
}

fn split_borrows() {
    // ANCHOR: split
    struct Car {
        engine_temp: f64,
        wheel_speeds: [f64; 4],
    }
    let mut car = Car {
        engine_temp: 90.0,
        wheel_speeds: [10.0; 4],
    };

    // Two &mut borrows of *different fields* at once:
    let temp = &mut car.engine_temp;
    let wheels = &mut car.wheel_speeds;
    *temp += 1.5;
    wheels[0] = 11.0;
    println!("temp {} wheels {:?}", car.engine_temp, car.wheel_speeds);

    // For slices, split_at_mut proves the halves are disjoint:
    let mut data = [1, 2, 3, 4, 5, 6];
    let (left, right) = data.split_at_mut(3);
    left[0] += right[0];
    println!("after split_at_mut: {data:?}");

    // get_disjoint_mut checks arbitrary indices at run time:
    if let Ok([a, b]) = data.get_disjoint_mut([1, 4]) {
        std::mem::swap(a, b);
    }
    println!("after swapping [1] and [4]: {data:?}");
    println!(
        "overlapping request: {:?}",
        data.get_disjoint_mut([2, 2]).map(|_| ())
    );
    // ANCHOR_END: split
}

fn moving_out_of_borrow() {
    // ANCHOR: move_out
    let names = vec![String::from("Sakichi"), String::from("Kiichiro")];

    fn first_ref(names: &[String]) -> &String {
        &names[0] // free: just a reference
    }
    fn first_clone(names: &[String]) -> String {
        names[0].clone() // costs an allocation
    }
    fn first_owned(mut names: Vec<String>) -> String {
        names.swap_remove(0) // we own the Vec, so we can take from it
    }

    println!("by ref:   {}", first_ref(&names));
    println!("by clone: {}", first_clone(&names));
    println!("by value: {}", first_owned(names));
    // ANCHOR_END: move_out
}
