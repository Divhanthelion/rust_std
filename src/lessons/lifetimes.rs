//! Lesson: Lifetimes.

use crate::lesson::{Lesson, Question, Section};

pub static LESSON: Lesson = Lesson {
    id: "lifetimes",
    title: "Lifetimes",
    summary: "What lifetimes are, when you must write them, elision rules, structs that borrow, 'static, variance and higher-ranked bounds.",
    source: include_str!("lifetimes.rs"),
    sections: &[
        Section::new(
            "What a lifetime is",
            r#"
            A **lifetime** is the region of code during which a reference must
            stay valid. The borrow checker gives every reference one and checks
            that the referenced value outlives every use of the reference:

            ```compile_fail,E0597
            let r;
            {
                let x = 5;
                r = &x;        // x's lifetime ends at the closing brace...
            }
            println!("{r}");   // ...but r is used here
            ```

            Inside function bodies, lifetimes are always inferred. You only
            write them in **signatures** and **type definitions**, where the
            compiler needs a contract between separately-checked pieces of
            code.

            Key idea: lifetime annotations **describe** relationships between
            references. They never make anything live longer.
            "#,
        ),
        Section::new(
            "When annotations are needed",
            r#"
            Consider a function that returns one of two string slices:

            ```compile_fail,E0106
            fn longest(a: &str, b: &str) -> &str {
                if a.len() >= b.len() { a } else { b }
            }
            ```

            The caller needs to know how long the result may be used. Is it
            tied to `a`, to `b`, or to both? The signature must say. With a
            lifetime parameter `'a`, the meaning is "the result lives at most
            as long as **both** inputs":

            ```rust
            fn longest<'a>(a: &'a str, b: &'a str) -> &'a str {
                if a.len() >= b.len() { a } else { b }
            }
            ```

            At each call site the compiler picks `'a` as the overlap of the two
            arguments' lifetimes, and checks your use of the result against it.
            "#,
        )
        .demo("longest", longest_demo),
        Section::new(
            "Elision: the three rules",
            r#"
            Most signatures need no annotations because the compiler applies
            three **elision rules**:

            1. Each elided input lifetime becomes its own parameter.
            2. If there is exactly one input lifetime, it is assigned to all
               output references.
            3. If there's a `&self` or `&mut self`, its lifetime is assigned to
               all output references.

            ```rust
            // What you write              // What the compiler sees
            fn first(s: &str) -> &str      { &s[..1] }
            fn first_x<'a>(s: &'a str) -> &'a str { &s[..1] }
            ```

            If the rules don't determine the output lifetime, as in `longest`,
            you get error E0106 and must write it yourself. The anonymous
            lifetime `'_` lets you say "elided here" explicitly, for example
            `Parser<'_>`.
            "#,
        ),
        Section::new(
            "Structs that borrow",
            r#"
            A struct that holds a reference needs a lifetime parameter.
            `Tokenizer<'a>` reads as "a tokenizer that borrows something for
            `'a`" and can't outlive the text it reads. This is how you build
            **zero-copy** parsers: tokens are slices of the input, not new
            `String`s. In embedded code that means no allocation per message.
            "#,
        )
        .demo("tokenizer", tokenizer_demo),
        Section::new(
            "Several lifetimes",
            r#"
            When an output relates to only one input, give the inputs separate
            lifetimes. That frees the caller: the other argument may be dropped
            while the result is still in use. With a single `'a` on both, the
            result would be needlessly tied to the shorter-lived argument.
            "#,
        )
        .demo("multiple", multiple_lifetimes),
        Section::new(
            "'static: two different meanings",
            r#"
            `'static` shows up in two places that are easy to confuse:

            - `&'static T` is a reference valid for the entire program: string
              literals, `static` items, or memory intentionally leaked with
              `Box::leak`.
            - `T: 'static` is a **bound** meaning "T contains no references
              that could expire". Every owned type satisfies it: `String`,
              `Vec<u8>` and `i32` are all `'static`. It does **not** mean the
              value lives forever.

            `std::thread::spawn` requires `F: 'static`, because a detached
            thread may outlive the caller's stack. So you move owned data into
            it rather than lending references. Scoped threads (Lesson 23) relax
            this.
            "#,
        )
        .demo("static", static_demo),
        Section::new(
            "Lifetime bounds and variance",
            r#"
            `T: 'a` means "every reference inside T outlives `'a`". You will
            see it on structs like `struct Ref<'a, T: 'a>(&'a T)`, though it
            is usually inferred.

            **Variance** explains why you can pass a longer-lived reference
            where a shorter one is expected:

            - `&'a T` is **covariant** in `'a`: a `&'static str` can be used as
              a `&'short str`, shrinking the lifetime safely.
            - `&'a mut T` is **invariant** in `T`. You can't treat a
              `&mut &'static str` as a `&mut &'short str`, because you could
              then write a short-lived reference into a slot that promises
              `'static`.

            You rarely think about variance until an error mentions it. When one
            does, the culprit is usually a `&mut` around something that holds a
            reference.
            "#,
        )
        .demo("variance", variance_demo),
        Section::new(
            "Higher-ranked trait bounds",
            r#"
            Sometimes a closure must work for **any** lifetime the caller picks,
            not one fixed lifetime. `for<'a> Fn(&'a str) -> &'a str` says
            "for every `'a`, given a `&'a str`, return a `&'a str`". You
            usually don't write the `for<'a>` part: `Fn(&str) -> &str` in a
            bound means exactly that, through elision. Knowing the expanded
            form helps when you read compiler errors about closures and
            lifetimes.
            "#,
        )
        .demo("hrtb", hrtb_demo),
        Section::new(
            "Fixing common lifetime errors",
            r#"
            Most lifetime errors fall into a few shapes, each with a standard
            fix:

            - **Returning a reference to a local**: return the owned value
              instead.
            - **A struct that borrows from itself**, such as storing a `String`
              and slices into it together: Rust can't express this safely.
              Store indices or ranges into the `String` instead.
            - **Holding a borrow across a mutation**: copy or clone out what you
              need first, or shorten the borrow's scope.
            - **Needing a reference in a thread or a long-lived struct**: use
              owned data, `Arc`, or scoped threads.

            The index-based fix is shown below. It is also how arenas and
            entity systems avoid lifetime tangles entirely.
            "#,
        )
        .demo("fixes", fixes_demo),
    ],
    quiz: &[
        Question::new(
            "What does `fn f<'a>(x: &'a str, y: &'a str) -> &'a str` promise?",
            &[
                "x and y live forever",
                "The result is valid as long as both x and y are",
                "The result is a copy",
                "x and y have exactly the same lifetime at the call site",
            ],
            1,
            "At each call `'a` becomes the overlap of the two argument lifetimes. Callers can pass references of different lifetimes, and the result is valid for the shorter one.",
        ),
        Question::new(
            "Which signature needs explicit lifetimes?",
            &[
                "fn f(s: &str) -> &str",
                "fn f(&self, s: &str) -> &str",
                "fn f(a: &str, b: &str) -> &str",
                "fn f(s: &str) -> usize",
            ],
            2,
            "Two input lifetimes, no self: the elision rules can't decide which one the output borrows from.",
        ),
        Question::new(
            "Is `String` a `'static` type (does `String: 'static` hold)?",
            &[
                "No — only literals are 'static",
                "Yes — it contains no borrowed references",
                "Only if it's stored in a static",
                "Only if leaked",
            ],
            1,
            "`T: 'static` means T holds no non-static references. Any owned type qualifies, even one dropped a moment later.",
        ),
        Question::new(
            "Why does `thread::spawn` require its closure to be `'static`?",
            &[
                "Threads are slow",
                "The thread may outlive the stack frame that created it",
                "Closures can't capture variables otherwise",
                "It doesn't",
            ],
            1,
            "A spawned thread can keep running after the spawning function returns, so borrowed locals could dangle. `thread::scope` lifts this by guaranteeing a join.",
        ),
        Question::new(
            "How do you model a struct that owns a `String` and also wants slices of it?",
            &[
                "Use a lifetime parameter on the struct pointing at itself",
                "Store byte ranges (indices) instead of slices",
                "Use 'static",
                "It's impossible in any form",
            ],
            1,
            "Self-referential borrows can't be expressed safely in plain Rust. Indices into the owned buffer work, and they stay valid even if the struct moves.",
        ),
    ],
    exercises: &[
        "Write `fn split_key_value<'a>(line: &'a str) -> Option<(&'a str, &'a str)>`, then remove the annotations. Does it still compile? Which elision rule applies?",
        "Build a `struct Words<'a> { rest: &'a str }` with a `next_word(&mut self) -> Option<&'a str>` method. Why must the return type be `&'a str` and not `&str`?",
        "Write `fn pick<'a, 'b>(primary: &'a str, _fallback: &'b str) -> &'a str` and a caller in which the fallback is dropped before the result is used.",
        "Turn a struct holding `text: String` and `words: Vec<&str>` (which can't compile) into one holding `text: String` and `words: Vec<std::ops::Range<usize>>`.",
    ],
};

// ANCHOR: longest
fn longest<'a>(a: &'a str, b: &'a str) -> &'a str {
    if a.len() >= b.len() { a } else { b }
}

fn longest_demo() {
    let model = String::from("Land Cruiser");
    {
        let other = String::from("Yaris");
        let winner = longest(&model, &other); // 'a = the inner block
        println!("longest: {winner}");
    } // `other` dies here; `winner` must not be used beyond this point
    println!("literals live forever: {}", longest("bZ4X", "Crown"));
}
// ANCHOR_END: longest

// ANCHOR: tokenizer
/// Splits text into words without allocating: every token borrows `input`.
struct Tokenizer<'a> {
    rest: &'a str,
}

impl<'a> Tokenizer<'a> {
    fn new(input: &'a str) -> Self {
        Tokenizer { rest: input }
    }

    /// The token borrows from the *input* ('a), not from `self`, so tokens
    /// stay usable even after the tokenizer itself is gone.
    fn next_token(&mut self) -> Option<&'a str> {
        let trimmed = self.rest.trim_start();
        if trimmed.is_empty() {
            return None;
        }
        let end = trimmed.find(char::is_whitespace).unwrap_or(trimmed.len());
        let (token, rest) = trimmed.split_at(end);
        self.rest = rest;
        Some(token)
    }
}

fn tokenizer_demo() {
    let command = String::from("SET  speed_limit 120 km/h");
    let tokens: Vec<&str> = {
        let mut t = Tokenizer::new(&command);
        let mut out = Vec::new();
        while let Some(tok) = t.next_token() {
            out.push(tok);
        }
        out
    }; // the Tokenizer is dropped; the tokens live on (they borrow `command`)
    println!("{tokens:?}");
}
// ANCHOR_END: tokenizer

fn multiple_lifetimes() {
    // ANCHOR: multiple
    // The result borrows only from `text`, never from `separator`.
    fn before<'t, 's>(text: &'t str, separator: &'s str) -> &'t str {
        text.split(separator).next().unwrap_or(text)
    }

    let text = String::from("brake::front::left");
    let head;
    {
        let sep = String::from("::"); // short-lived
        head = before(&text, &sep);
    } // sep dropped — fine, `head` doesn't borrow from it
    println!("head = {head}");
    // ANCHOR_END: multiple
}

fn static_demo() {
    // ANCHOR: static
    static GREETING: &str = "hello";
    let literal: &'static str = "baked into the binary";
    println!("{GREETING}, {literal}");

    // A T: 'static bound accepts any *owned* value, however short-lived:
    fn needs_static<T: 'static + std::fmt::Debug>(value: T) {
        println!("got {value:?}");
    }
    let owned = String::from("owned String");
    needs_static(owned); //   OK: String holds no borrows
    needs_static(42); //      OK
    needs_static(literal); // OK: &'static str
    // let local = String::from("x");
    // needs_static(&local); // error: `local` does not live long enough

    // thread::spawn wants 'static, so we *move* owned data in:
    let data = vec![1, 2, 3];
    let handle = std::thread::spawn(move || data.iter().sum::<i32>());
    println!("sum from thread: {}", handle.join().unwrap());

    // Box::leak trades memory for a &'static (useful for one-time config):
    let config: &'static str = Box::leak(String::from("leaked config").into_boxed_str());
    println!("{config}");
    // ANCHOR_END: static
}

fn variance_demo() {
    // ANCHOR: variance
    fn print_both<'a>(x: &'a str, y: &'a str) {
        println!("{x} / {y}");
    }
    let forever: &'static str = "static";
    let local = String::from("local");
    // `forever` is &'static, but covariance lets it shrink to match `local`:
    print_both(forever, &local);

    // A struct with a lifetime bound on its type parameter:
    struct Pair<'a, T: 'a> {
        left: &'a T,
        right: &'a T,
    }
    let (a, b) = (1, 2);
    let pair = Pair {
        left: &a,
        right: &b,
    };
    println!("pair = ({}, {})", pair.left, pair.right);
    // ANCHOR_END: variance
}

fn hrtb_demo() {
    // ANCHOR: hrtb
    // F must work for *every* lifetime 'x the function picks internally.
    fn apply_to_parts<F>(text: &str, f: F) -> Vec<&str>
    where
        F: for<'x> Fn(&'x str) -> &'x str, // same as Fn(&str) -> &str
    {
        text.split(',').map(&f).collect() // `&F` is callable too
    }

    let trimmed = apply_to_parts(" a , b ,c ", |s| s.trim());
    println!("{trimmed:?}");
    let first_chars = apply_to_parts("alpha,beta", |s| s.get(..1).unwrap_or(""));
    println!("{first_chars:?}");
    // ANCHOR_END: hrtb
}

fn fixes_demo() {
    // ANCHOR: fixes
    use std::ops::Range;

    // Instead of `words: Vec<&str>` borrowing from `text` in the same
    // struct (self-referential: impossible), store ranges.
    struct Document {
        text: String,
        words: Vec<Range<usize>>,
    }

    impl Document {
        fn new(text: String) -> Self {
            let mut words = Vec::new();
            let mut start = None;
            for (i, c) in text.char_indices() {
                match (c.is_whitespace(), start) {
                    (false, None) => start = Some(i),
                    (true, Some(s)) => {
                        words.push(s..i);
                        start = None;
                    }
                    _ => {}
                }
            }
            if let Some(s) = start {
                words.push(s..text.len());
            }
            Document { text, words }
        }

        fn word(&self, n: usize) -> Option<&str> {
            self.words.get(n).map(|r| &self.text[r.clone()])
        }
    }

    let doc = Document::new(String::from("kaizen means continuous improvement"));
    let moved = doc; // moving the struct is fine: ranges don't dangle
    println!("{} words; third = {:?}", moved.words.len(), moved.word(2));
    // ANCHOR_END: fixes
}
