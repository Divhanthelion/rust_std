//! Lesson: Interview clinic.

use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, VecDeque};

use crate::lesson::{Lesson, Question, Section};

pub static LESSON: Lesson = Lesson {
    id: "interview",
    title: "Interview clinic",
    summary: "Model answers to common Rust interview questions, predict-the-output drills, and whiteboard classics in idiomatic Rust: queue from stacks, linked list, O(1) LRU cache, and array/string problems.",
    source: include_str!("interview.rs"),
    sections: &[
        Section::new(
            "How to answer a Rust question",
            r#"
            Strong answers follow a shape. Interviewers listen for it:

            1. **Define** the concept in one sentence.
            2. Say **why it exists**: the problem it solves.
            3. Place it on a **spectrum** of alternatives and name the
               trade-off. "Box, Rc and Arc trade flexibility for cost."
            4. Give a **concrete example**, ideally from your own experience.
            5. Mention how you'd **test** or **verify** it.

            When writing code, think aloud: state the invariants, handle the
            edge cases (empty, one element, overflow, the full buffer), and
            prefer clear code over clever code. For an automotive role, also
            say what happens on bad input. "This returns a `Result`, never
            panics" is a sentence that lands well.
            "#,
        ),
        Section::new(
            "Drills: ownership and borrowing",
            r#"
            - **Q: What is ownership?** Every value has one owner. When the
              owner goes out of scope the value is dropped. Assignment moves
              ownership unless the type is `Copy`. Memory is freed
              deterministically, without a garbage collector.
            - **Q: `&T` vs `&mut T`?** Shared versus exclusive. Many `&T` or one
              `&mut T`, never both at once. That one rule prevents data races
              and iterator invalidation.
            - **Q: Copy vs Clone?** `Copy` is an implicit bitwise duplicate, only
              for types without destructors or owned heap data. `Clone` is
              explicit and may allocate.
            - **Q: Why can't I return a reference to a local?** It would dangle:
              the local is dropped at the end of the function. Return an owned
              value.
            - **Q: What do lifetime annotations do?** They describe how
              references relate, so the compiler can check callers. They never
              extend how long a value lives.
            - **Q: What's `'static` as a bound?** `T: 'static` means T holds no
              non-static borrows. Owned types qualify, which is why
              `thread::spawn` needs it.
            - **Q: `mem::take` / `mem::replace`?** They move a value out of a
              `&mut` by leaving a default or replacement behind. They're the
              standard way to restructure state behind a reference.
            "#,
        ),
        Section::new(
            "Drills: traits, generics and dispatch",
            r#"
            - **Q: Generics vs trait objects?** Generics are monomorphized:
              static dispatch, inlinable, larger binaries, one type per
              instantiation. `dyn Trait` gives one copy of the code, a vtable
              call, and heterogeneous collections. Enums are a third option for
              closed sets.
            - **Q: What makes a trait dyn-compatible (object safe)?** No generic
              methods, no `Self` returned by value, and receivers like `&self`.
              Use `where Self: Sized` to exclude individual methods.
            - **Q: Associated type vs generic parameter?** An associated type
              means one implementation per type (`Iterator::Item`). A generic
              parameter allows several (`From<T>`).
            - **Q: The orphan rule?** An impl needs a local trait or a local
              type. Work around it with a newtype or an extension trait.
            - **Q: What is `Sized`, and `?Sized`?** Generic parameters
              implicitly require a size known at compile time. `?Sized` also
              admits `str`, `[T]` and `dyn Trait` behind pointers.
            - **Q: Fn, FnMut, FnOnce?** Call by `&self`, by `&mut self`, or by
              value. Accept the weakest you need: FnOnce is the most permissive
              for callers.
            "#,
        ),
        Section::new(
            "Drills: memory, smart pointers and concurrency",
            r#"
            - **Q: Box vs Rc vs Arc?** Box: one owner, on the heap. Rc: shared
              ownership, single thread, non-atomic count. Arc: shared across
              threads, atomic count, a little slower.
            - **Q: RefCell vs Mutex?** Both give interior mutability with
              run-time checks. RefCell is single-threaded and panics on
              conflict. Mutex is thread-safe and blocks.
            - **Q: Send and Sync?** `Send`: safe to move to another thread.
              `Sync`: `&T` is safe to share. They're auto traits, so `Rc`'s
              `!Send` spreads to any type containing it.
            - **Q: Data race vs race condition?** A data race is unsynchronized
              concurrent access with at least one write. It's undefined
              behaviour, and safe Rust makes it impossible. A race condition is
              a logic bug in timing (check-then-act), and Rust can't rule it
              out. Neither can it rule out deadlocks.
            - **Q: Memory orderings?** Relaxed gives atomicity only. Release on
              a store paired with Acquire on a load publishes everything
              written before. SeqCst adds a single global order. Use SeqCst
              when unsure, and measure before relaxing.
            - **Q: Stack vs heap?** The stack holds fixed-size data, is freed by
              popping frames, and is very fast. The heap is for dynamic sizes
              and lifetimes beyond a frame, and is managed through owners like
              Box, Vec and String.
            - **Q: What's a niche?** An invalid bit pattern the compiler reuses
              for enum tags, which is why `Option<&T>` is pointer-sized.
            "#,
        ),
        Section::new(
            "Drills: errors, panics and unsafe",
            r#"
            - **Q: Result or panic?** Result for expected, recoverable failures,
              where the caller decides. Panic for bugs and broken invariants.
              In safety-critical code, aim for no panics in production paths.
            - **Q: What does `?` do?** On `Err(e)` it returns
              `Err(From::from(e))` early, and on `Ok(v)` it evaluates to `v`.
              It works for Option too.
            - **Q: Is `unwrap` ever fine?** In tests and prototypes, and where
              an invariant truly guarantees success. Even then, prefer
              `expect("why it can't fail")`.
            - **Q: What does `unsafe` allow?** Dereferencing raw pointers,
              calling unsafe functions (including FFI), accessing `static mut`,
              implementing unsafe traits, and reading union fields. Nothing
              else is turned off.
            - **Q: How do you make unsafe code trustworthy?** Keep it minimal,
              wrap it in safe APIs that are sound for **all** inputs, write a
              `SAFETY:` comment for every block, test it under Miri, and get it
              reviewed.
            - **Q: What happens if Rust panics inside a function called from
              C?** Since 1.81, unwinding out of `extern "C"` aborts. Catch
              panics at the boundary and return error codes.
            "#,
        ),
        Section::new(
            "Predict the output",
            r#"
            Read the code and predict every line **before** you look at the
            output. Each line is a classic interview trap: drop order, shadowing
            inside a block, wrapping arithmetic, laziness, closure capture, and
            sorting with `Reverse`.
            "#,
        )
        .demo("predict", predict),
        Section::new(
            "Whiteboard: a queue from two stacks",
            r#"
            Push onto `inbox`. To pop, if `outbox` is empty, move everything
            from `inbox` to `outbox`, which reverses the order, then pop from
            `outbox`. Each element moves at most twice, so operations are
            **amortized O(1)**. Explaining "amortized" clearly is half the
            answer. In production, use `VecDeque`.
            "#,
        )
        .demo("queue", two_stack_queue),
        Section::new(
            "Whiteboard: a singly linked list",
            r#"
            In Rust a linked list is `Option<Box<Node>>`. `Option::take()`
            is the move that makes it work: it detaches the head while leaving
            `None` behind, so ownership never has two holders. In-place
            reversal is the same three-pointer dance as in C, with ownership
            spelled out. Mention in passing that `Vec` usually beats linked
            lists, because of cache locality.
            "#,
        )
        .demo("list", linked_list),
        Section::new(
            "Whiteboard: an O(1) LRU cache",
            r#"
            An LRU cache needs O(1) lookup **and** O(1) "move to most recent" and
            "evict least recent". The classic answer is a hash map plus a
            doubly linked list. In Rust, build the list from **indices into a
            Vec** instead of pointers: no `unsafe`, no `Rc<RefCell<…>>`, and
            friendly to the borrow checker. The same arena technique scales to
            graphs and entity systems.
            "#,
        )
        .demo("lru", lru),
        Section::new(
            "Whiteboard: array and string classics",
            r#"
            Each of these maps onto a std collection:

            - **two sum** uses a HashMap from value to index: O(n);
            - **balanced brackets** uses a Vec as a stack;
            - **sliding window maximum** uses a VecDeque of indices kept in
              decreasing order (a monotonic queue): O(n);
            - **top-k frequent** uses a HashMap of counts, then a
              `BinaryHeap<Reverse<…>>` of size k: O(n log k);
            - **merge intervals** sorts by start, then sweeps once.
            "#,
        )
        .demo("classics", classics),
        Section::new(
            "Behavioural questions, and questions to ask",
            r#"
            Behavioural rounds commonly use the **STAR** format: Situation,
            Task, Action, Result. Prepare stories about a bug you tracked down,
            a design trade-off you argued for, a time you pushed back on an
            unsafe shortcut, and learning something hard quickly (like Rust).

            Thoughtful questions to ask an automotive Rust team:

            - What ASIL or QM level does the team's Rust code target, and how
              is it argued in the safety case?
            - Which toolchain and version policy do you use (a qualified
              toolchain, MSRV pinning, update cadence)?
            - How is `unsafe` reviewed, and how much of it is there?
            - How does Rust integrate with the existing C/C++ and AUTOSAR code
              (FFI boundaries, build system, ownership of interfaces)?
            - What does testing look like: unit, HIL/SIL, fuzzing, coverage?
            - Where does the team want Rust to grow next?
            "#,
        ),
    ],
    quiz: &[
        Question::new(
            "Which statement about safe Rust is TRUE?",
            &[
                "It prevents deadlocks",
                "It prevents data races",
                "It prevents memory leaks",
                "It prevents panics",
            ],
            1,
            "Data races are compile errors in safe Rust. Deadlocks, leaks (Rc cycles, mem::forget) and panics are all possible.",
        ),
        Question::new(
            "What does this print?\n\n```rust\nlet x = 5;\nlet x = x * 2;\n{ let x = x + 1; println!(\"{x}\"); }\nprintln!(\"{x}\");\n```",
            &["11 then 11", "11 then 10", "6 then 5", "11 then 5"],
            1,
            "Shadowing: the outer x is 10; the inner block shadows it with 11 only inside the braces.",
        ),
        Question::new(
            "Why use indices into a Vec for a doubly linked list in Rust?",
            &[
                "Indices are faster than pointers",
                "It avoids aliasing problems: no unsafe and no Rc<RefCell>, while keeping O(1) operations",
                "Vec can't hold Box",
                "Linked lists are forbidden in Rust",
            ],
            1,
            "An arena of nodes with index links satisfies the borrow checker naturally.",
        ),
        Question::new(
            "Amortized O(1) for the two-stack queue means…",
            &[
                "Every operation is O(1)",
                "Occasional O(n) transfers average out to O(1) per operation over any sequence",
                "It's O(1) on average for random inputs only",
                "It's O(log n)",
            ],
            1,
            "Each element is moved at most twice in total, so n operations cost O(n) overall.",
        ),
        Question::new(
            "What's the type of a `for x in vec_of_strings.iter()` loop variable?",
            &["String", "&String", "&str", "&&String"],
            1,
            "`iter()` yields shared references to the elements.",
        ),
        Question::new(
            "`Rc<RefCell<T>>` is to a single thread what ____ is to many threads.",
            &[
                "Box<Cell<T>>",
                "Arc<Mutex<T>> (or Arc<RwLock<T>>)",
                "Rc<Mutex<T>>",
                "&mut T",
            ],
            1,
            "Shared ownership plus interior mutability: the thread-safe versions are Arc and Mutex/RwLock.",
        ),
    ],
    exercises: &[
        "Explain, out loud and in under two minutes each: ownership, Send/Sync, and dyn vs generics. Record yourself and listen back.",
        "Implement `fn is_palindrome(s: &str) -> bool` that ignores case and non-alphanumerics, without allocating.",
        "Add `get_mut` and a capacity-0 edge case to the LRU cache, plus tests for evicting the most recently *read* entry.",
        "Solve 'merge k sorted lists' with a `BinaryHeap<Reverse<(value, list_index)>>`, then state its complexity.",
    ],
};

fn predict() {
    // ANCHOR: predict
    struct Loud(&'static str);
    impl Drop for Loud {
        fn drop(&mut self) {
            println!("drop {}", self.0);
        }
    }

    let _a = Loud("a");
    {
        let _b = Loud("b");
        let _ = Loud("c"); // not bound!
        println!("end of block");
    }

    let x = 5;
    let x = x * 2;
    {
        let x = x + 1;
        println!("inner x = {x}");
    }
    println!("outer x = {x}");

    println!("{}", 250u8.wrapping_add(10));

    let lazy = (1..=3).map(|n| {
        println!("mapping {n}");
        n * 10
    });
    println!("before consuming");
    let total: i32 = lazy.sum();
    println!("total {total}");

    let mut count = 0;
    let mut inc = || count += 1;
    inc();
    inc();
    println!("count {count}");

    let mut v = vec![3, 1, 2];
    v.sort_by_key(|&n| Reverse(n));
    println!("{v:?}");
    // ANCHOR_END: predict
}

fn two_stack_queue() {
    // ANCHOR: queue
    #[derive(Default)]
    struct Queue<T> {
        inbox: Vec<T>,
        outbox: Vec<T>,
    }

    impl<T> Queue<T> {
        fn push(&mut self, value: T) {
            self.inbox.push(value);
        }
        fn pop(&mut self) -> Option<T> {
            if self.outbox.is_empty() {
                // Reversing once makes the oldest element the top of outbox.
                while let Some(v) = self.inbox.pop() {
                    self.outbox.push(v);
                }
            }
            self.outbox.pop()
        }
        fn len(&self) -> usize {
            self.inbox.len() + self.outbox.len()
        }
    }

    let mut q = Queue::default();
    for job in ["read", "decode", "log"] {
        q.push(job);
    }
    print!("{} ", q.pop().unwrap_or("-"));
    q.push("send");
    while let Some(job) = q.pop() {
        print!("{job} ");
    }
    println!("(len now {})", q.len());
    // ANCHOR_END: queue
}

fn linked_list() {
    // ANCHOR: list
    struct Node<T> {
        value: T,
        next: Option<Box<Node<T>>>,
    }

    struct List<T> {
        head: Option<Box<Node<T>>>,
        len: usize,
    }

    impl<T> List<T> {
        fn new() -> Self {
            List { head: None, len: 0 }
        }
        fn push(&mut self, value: T) {
            let old = self.head.take(); // detach the old head
            self.head = Some(Box::new(Node { value, next: old }));
            self.len += 1;
        }
        fn pop(&mut self) -> Option<T> {
            self.head.take().map(|node| {
                self.head = node.next;
                self.len -= 1;
                node.value
            })
        }
        fn peek(&self) -> Option<&T> {
            self.head.as_ref().map(|node| &node.value)
        }
        fn reverse(&mut self) {
            let mut prev: Option<Box<Node<T>>> = None;
            let mut current = self.head.take();
            while let Some(mut node) = current {
                current = node.next.take(); // remember the rest
                node.next = prev; //            point backwards
                prev = Some(node);
            }
            self.head = prev;
        }
        fn iter(&self) -> impl Iterator<Item = &T> {
            std::iter::successors(self.head.as_deref(), |node| node.next.as_deref())
                .map(|n| &n.value)
        }
    }

    let mut list = List::new();
    for n in 1..=4 {
        list.push(n);
    }
    println!(
        "list {:?}, peek {:?}, len {}",
        list.iter().collect::<Vec<_>>(),
        list.peek(),
        list.len
    );
    list.reverse();
    println!("reversed {:?}", list.iter().collect::<Vec<_>>());
    println!(
        "pop {:?}, then {:?}",
        list.pop(),
        list.iter().collect::<Vec<_>>()
    );
    // ANCHOR_END: list
}

fn lru() {
    // ANCHOR: lru
    const NONE: usize = usize::MAX; // "null" index

    struct Entry<V> {
        key: u32,
        value: V,
        prev: usize,
        next: usize,
    }

    /// Doubly linked list threaded through a Vec: head = most recent.
    struct Lru<V> {
        map: HashMap<u32, usize>,
        nodes: Vec<Entry<V>>,
        head: usize,
        tail: usize,
        capacity: usize,
    }

    impl<V> Lru<V> {
        fn new(capacity: usize) -> Self {
            Lru {
                map: HashMap::new(),
                nodes: Vec::with_capacity(capacity),
                head: NONE,
                tail: NONE,
                capacity,
            }
        }

        fn unlink(&mut self, i: usize) {
            let (prev, next) = (self.nodes[i].prev, self.nodes[i].next);
            if prev != NONE {
                self.nodes[prev].next = next
            } else {
                self.head = next
            }
            if next != NONE {
                self.nodes[next].prev = prev
            } else {
                self.tail = prev
            }
        }

        fn push_front(&mut self, i: usize) {
            self.nodes[i].prev = NONE;
            self.nodes[i].next = self.head;
            if self.head != NONE {
                self.nodes[self.head].prev = i;
            }
            self.head = i;
            if self.tail == NONE {
                self.tail = i;
            }
        }

        fn get(&mut self, key: u32) -> Option<&V> {
            let i = *self.map.get(&key)?;
            self.unlink(i);
            self.push_front(i); // mark as most recently used
            Some(&self.nodes[i].value)
        }

        fn put(&mut self, key: u32, value: V) -> Option<u32> {
            if let Some(&i) = self.map.get(&key) {
                self.nodes[i].value = value;
                self.unlink(i);
                self.push_front(i);
                return None;
            }
            if self.nodes.len() < self.capacity {
                self.nodes.push(Entry {
                    key,
                    value,
                    prev: NONE,
                    next: NONE,
                });
                let i = self.nodes.len() - 1;
                self.map.insert(key, i);
                self.push_front(i);
                return None;
            }
            // Full: reuse the least recently used slot (the tail).
            let i = self.tail;
            let evicted = self.nodes[i].key;
            self.unlink(i);
            self.map.remove(&evicted);
            self.nodes[i] = Entry {
                key,
                value,
                prev: NONE,
                next: NONE,
            };
            self.map.insert(key, i);
            self.push_front(i);
            Some(evicted)
        }
    }

    let mut cache: Lru<&str> = Lru::new(2);
    cache.put(0x7E0, "engine");
    cache.put(0x7E1, "transmission");
    println!("get 0x7E0 → {:?}", cache.get(0x7E0)); // 0x7E0 is now most recent
    println!("put 0x7E2 evicts {:x?}", cache.put(0x7E2, "abs")); // evicts 0x7E1
    println!("get 0x7E1 → {:?}", cache.get(0x7E1));
    println!("get 0x7E2 → {:?}", cache.get(0x7E2));
    // ANCHOR_END: lru
}

fn classics() {
    // ANCHOR: classics
    fn two_sum(nums: &[i32], target: i32) -> Option<(usize, usize)> {
        let mut seen: HashMap<i32, usize> = HashMap::new();
        for (i, &n) in nums.iter().enumerate() {
            if let Some(&j) = seen.get(&(target - n)) {
                return Some((j, i));
            }
            seen.insert(n, i);
        }
        None
    }

    fn balanced(s: &str) -> bool {
        let mut stack = Vec::new();
        for c in s.chars() {
            let opener = match c {
                '(' | '[' | '{' => {
                    stack.push(c);
                    continue;
                }
                ')' => '(',
                ']' => '[',
                '}' => '{',
                _ => continue,
            };
            if stack.pop() != Some(opener) {
                return false;
            }
        }
        stack.is_empty()
    }

    fn window_max(values: &[i32], k: usize) -> Vec<i32> {
        let mut out = Vec::new();
        let mut dq: VecDeque<usize> = VecDeque::new(); // indices, values decreasing
        for (i, &v) in values.iter().enumerate() {
            while dq.back().is_some_and(|&j| values[j] <= v) {
                dq.pop_back();
            }
            dq.push_back(i);
            if dq.front().is_some_and(|&j| j + k <= i) {
                dq.pop_front(); // fell out of the window
            }
            if i + 1 >= k {
                out.push(values[dq[0]]);
            }
        }
        out
    }

    fn top_k<'a>(words: &[&'a str], k: usize) -> Vec<(&'a str, usize)> {
        let mut counts: HashMap<&str, usize> = HashMap::new();
        for w in words {
            *counts.entry(w).or_default() += 1;
        }
        let mut heap = BinaryHeap::new(); // min-heap of size k via Reverse
        for (w, c) in counts {
            heap.push(Reverse((c, w)));
            if heap.len() > k {
                heap.pop(); // drop the least frequent
            }
        }
        let mut result: Vec<(&str, usize)> =
            heap.into_iter().map(|Reverse((c, w))| (w, c)).collect();
        result.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
        result
    }

    fn merge_intervals(mut v: Vec<(u32, u32)>) -> Vec<(u32, u32)> {
        v.sort_unstable();
        let mut merged: Vec<(u32, u32)> = Vec::new();
        for (start, end) in v {
            match merged.last_mut() {
                Some(last) if start <= last.1 => last.1 = last.1.max(end),
                _ => merged.push((start, end)),
            }
        }
        merged
    }

    println!("two_sum      {:?}", two_sum(&[2, 7, 11, 15], 9));
    println!("balanced     {} {}", balanced("{[()()]}"), balanced("([)]"));
    println!(
        "window_max   {:?}",
        window_max(&[1, 3, -1, -3, 5, 3, 6, 7], 3)
    );
    println!(
        "top_k        {:?}",
        top_k(&["abs", "esc", "abs", "lka", "esc", "abs"], 2)
    );
    println!(
        "merge        {:?}",
        merge_intervals(vec![(1, 3), (8, 10), (2, 6), (9, 12)])
    );
    // ANCHOR_END: classics
}
