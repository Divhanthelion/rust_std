//! Lesson: Collections.

use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet, BinaryHeap, HashMap, HashSet, VecDeque};

use crate::lesson::{Lesson, Question, Section};

pub static LESSON: Lesson = Lesson {
    id: "collections",
    title: "Collections",
    summary: "Vec, VecDeque, HashMap, HashSet, BTreeMap, BTreeSet and BinaryHeap: what each is for, their key methods, the entry API, and costs.",
    source: include_str!("collections.rs"),
    sections: &[
        Section::new(
            "Choosing a collection",
            r#"
            `std::collections` covers most needs. Pick by the question you
            need answered quickly:

            ```text
            Vec<T>            ordered sequence; the default choice
            VecDeque<T>       queue: cheap push/pop at both ends (ring buffer)
            HashMap<K, V>     look up by key in O(1); no ordering
            HashSet<T>        membership / uniqueness in O(1)
            BTreeMap<K, V>    sorted by key; range queries; deterministic order
            BTreeSet<T>       sorted unique values
            BinaryHeap<T>     always pop the largest (priority queue)
            LinkedList<T>     rarely the right answer: poor cache locality
            ```

            When in doubt, start with `Vec`. Contiguous memory makes it faster
            than its big-O suggests, even against "smarter" structures for
            small sizes.
            "#,
        ),
        Section::new(
            "Vec in depth",
            r#"
            A `Vec` is a pointer, a length and a capacity. Pushing beyond the
            capacity reallocates, usually doubling, so pushes are **amortized
            O(1)**. When you know the size, `Vec::with_capacity(n)` or
            `reserve(n)` avoids repeated reallocation.

            `try_reserve` returns an error instead of aborting the process when
            allocation fails. That matters in memory-constrained systems,
            where "out of memory" must be a handled condition.

            Insertion and removal in the middle shift elements, which is O(n).
            If order doesn't matter, `swap_remove` is O(1).
            "#,
        )
        .demo("vec", vec_demo),
        Section::new(
            "VecDeque: a growable ring buffer",
            r#"
            `VecDeque` stores elements in a ring, so `push_front`, `push_back`,
            `pop_front` and `pop_back` are all O(1). It's the right type for
            FIFO queues and **sliding windows**, such as keeping the last N
            sensor samples. Its storage may wrap around, so `as_slices()`
            returns up to two slices; `make_contiguous()` fixes that when you
            need one.
            "#,
        )
        .demo("deque", deque_demo),
        Section::new(
            "HashMap basics",
            r#"
            `HashMap<K, V>` needs keys that are `Hash + Eq`. `insert` returns the
            previous value, if any. `get` returns an `Option<&V>`, and indexing
            with `map[&key]` panics if the key is missing.

            Iteration order is **arbitrary and changes between runs**. The
            default hasher (SipHash) is seeded randomly per map to resist
            hash-flooding attacks. Never depend on HashMap order. When you need
            deterministic output, as in logs, tests and reproducible builds,
            use a `BTreeMap` or sort the keys.
            "#,
        )
        .demo("hashmap", hashmap_demo),
        Section::new(
            "The entry API",
            r#"
            "Look up a key, insert it if missing, then update it" is so common
            that `HashMap` has a dedicated API for it, which needs only **one**
            hash lookup:

            - `entry(k).or_insert(v)` inserts `v` if the key is absent, and
              returns `&mut V`;
            - `or_insert_with(|| …)` builds the value lazily, and `or_default()`
              uses `Default`;
            - `and_modify(|v| …)` updates an existing value, and chains with
              `or_insert`;
            - matching on `Entry::Occupied`/`Vacant` gives full control.
            "#,
        )
        .demo("entry", entry_demo),
        Section::new(
            "HashSet and set algebra",
            r#"
            A `HashSet<T>` is a `HashMap<T, ()>`: fast membership tests and
            automatic deduplication. Sets support the usual algebra: `union`,
            `intersection`, `difference`, `symmetric_difference`, `is_subset`
            and `is_disjoint`. These return lazy iterators; collect them if you
            need a set. `insert` returns `false` when the value was already
            present, which makes "first time seen?" checks one call.
            "#,
        )
        .demo("sets", sets_demo),
        Section::new(
            "BTreeMap and BTreeSet: sorted and ranged",
            r#"
            B-tree collections keep keys **sorted**. Lookups are O(log n)
            rather than O(1), and in exchange you get:

            - iteration in key order, which is deterministic;
            - `range(a..b)` queries, such as "all events between t=100 and
              t=200";
            - `first_key_value` and `last_key_value`, the minimum and maximum
              in O(log n);
            - keys need `Ord`, not `Hash`.

            Time-series data, schedules and ordered reports are natural fits.
            "#,
        )
        .demo("btree", btree_demo),
        Section::new(
            "BinaryHeap: a priority queue",
            r#"
            `BinaryHeap` is a max-heap. `push` is O(log n), `pop` returns the
            largest element in O(log n), and `peek` is O(1). For a **min**-heap,
            wrap items in `std::cmp::Reverse`. A task scheduler that always
            runs the earliest deadline next is the textbook use, as is
            Dijkstra's shortest-path algorithm.
            "#,
        )
        .demo("heap", heap_demo),
        Section::new(
            "Costs at a glance",
            r#"
            ```text
                          push/insert   remove     lookup     ordered?
            Vec           O(1)* end     O(n)**     O(n)       insertion order
            VecDeque      O(1)* ends    O(1) ends  O(n)       insertion order
            HashMap/Set   O(1)*         O(1)       O(1)       no (random)
            BTreeMap/Set  O(log n)      O(log n)   O(log n)   yes (sorted)
            BinaryHeap    O(log n)      O(log n)†  O(1) max   partially

            *  amortized   ** O(1) with swap_remove   † pop of the maximum
            ```

            In hard real-time code, "amortized" means **occasionally slow**: a
            reallocation can land on any push. Pre-allocate during
            initialization, or use fixed-capacity structures (Lesson 33).
            "#,
        ),
    ],
    quiz: &[
        Question::new(
            "Why shouldn't tests compare the iteration order of a HashMap?",
            &[
                "HashMap iteration is slow",
                "The order is arbitrary and randomized per map instance",
                "HashMap can't be iterated",
                "It's sorted by insertion time",
            ],
            1,
            "SipHash keys are random per map, so order varies between runs. Use BTreeMap or sort for deterministic output.",
        ),
        Question::new(
            "What is the cost of `vec.remove(0)` on a Vec of n elements?",
            &["O(1)", "O(log n)", "O(n)", "O(n log n)"],
            2,
            "All later elements shift left. Use VecDeque::pop_front for O(1), or swap_remove if order doesn't matter.",
        ),
        Question::new(
            "How do you make a BinaryHeap pop the smallest element first?",
            &[
                "BinaryHeap::min()",
                "Push std::cmp::Reverse(x)",
                "Sort it first",
                "Use pop_front",
            ],
            1,
            "Reverse inverts the ordering, turning the max-heap into a min-heap.",
        ),
        Question::new(
            "Which counts words with a single lookup per word?",
            &[
                "if map.contains_key(w) { *map.get_mut(w).unwrap() += 1 } else { map.insert(w, 1); }",
                "*map.entry(w).or_insert(0) += 1",
                "map.insert(w, map[w] + 1)",
                "map.get(w).map(|c| c + 1)",
            ],
            1,
            "The entry API hashes once and returns a mutable reference to the (possibly new) value.",
        ),
        Question::new(
            "You need all CAN messages logged between t=1000 and t=2000 ms. Which collection fits best?",
            &[
                "HashMap<u64, Frame>",
                "BTreeMap<u64, Frame>",
                "BinaryHeap<Frame>",
                "HashSet<u64>",
            ],
            1,
            "BTreeMap keeps keys sorted and supports `range(1000..2000)` directly.",
        ),
    ],
    exercises: &[
        "Count the frequency of each word in a paragraph with the entry API, then print the five most common words in descending order (ties alphabetically).",
        "Keep a rolling average of the last 10 readings with a VecDeque, without ever reallocating after construction.",
        "Store timestamped events in a BTreeMap and write `fn between(&self, from: u64, to: u64) -> Vec<&Event>`.",
        "Implement an earliest-deadline-first scheduler with `BinaryHeap<Reverse<(u64, String)>>`, pushing and popping tasks in a simulated clock loop.",
    ],
};

fn vec_demo() {
    // ANCHOR: vec
    let mut v: Vec<u32> = Vec::with_capacity(4);
    println!("empty: len {} capacity {}", v.len(), v.capacity());
    for i in 1..=5 {
        v.push(i * 10);
        println!(
            "push {:>2}: len {} capacity {}",
            i * 10,
            v.len(),
            v.capacity()
        );
    }

    v.insert(1, 15); //          O(n): shifts the rest
    let last = v.pop(); //       O(1)
    let removed = v.remove(0); // O(n)
    let swapped = v.swap_remove(0); // O(1), order not kept
    println!("{v:?} (popped {last:?}, removed {removed}, swap_removed {swapped})");

    v.extend([20, 20, 30, 5]);
    v.sort_unstable(); //  faster than sort(); equal elements may reorder
    v.dedup(); //          removes consecutive duplicates
    println!(
        "sorted+dedup {v:?}, binary_search(30) = {:?}",
        v.binary_search(&30)
    );
    v.retain(|&x| x >= 20);
    println!("retain >= 20 {v:?}");

    let tail = v.split_off(2);
    let drained: Vec<u32> = v.drain(..1).collect(); // remove a range, keep the rest
    println!("split_off → {tail:?}, drained {drained:?}, left {v:?}");

    let mut big: Vec<u8> = Vec::new();
    match big.try_reserve(1 << 20) {
        Ok(()) => println!("reserved 1 MiB, capacity now ≥ {}", big.capacity()),
        Err(e) => println!("allocation failed gracefully: {e}"),
    }
    big.shrink_to_fit();
    // ANCHOR_END: vec
}

fn deque_demo() {
    // ANCHOR: deque
    const WINDOW: usize = 4;
    let mut window: VecDeque<f64> = VecDeque::with_capacity(WINDOW);
    for reading in [10.0, 12.0, 11.0, 15.0, 30.0, 14.0] {
        if window.len() == WINDOW {
            window.pop_front(); // drop the oldest
        }
        window.push_back(reading);
        let avg = window.iter().sum::<f64>() / window.len() as f64;
        println!("window {window:?} → average {avg:.2}");
    }

    let mut q: VecDeque<&str> = VecDeque::new();
    q.push_back("job-2");
    q.push_front("urgent");
    q.push_back("job-3");
    println!("front {:?}, back {:?}", q.front(), q.back());
    q.rotate_left(1);
    println!("rotated {q:?}");
    let contiguous: &mut [&str] = q.make_contiguous();
    contiguous.sort();
    println!("sorted in place {q:?}");
    // ANCHOR_END: deque
}

fn hashmap_demo() {
    // ANCHOR: hashmap
    let mut ecus: HashMap<u16, &str> = HashMap::new();
    ecus.insert(0x7E0, "engine");
    ecus.insert(0x7E1, "transmission");
    let previous = ecus.insert(0x7E0, "engine (v2)"); // replaces
    println!("previous value for 0x7E0: {previous:?}");

    println!("get(0x7E1) = {:?}", ecus.get(&0x7E1));
    println!("get(0x123) = {:?}", ecus.get(&0x123));
    println!("contains 0x7E0? {}", ecus.contains_key(&0x7E0));
    if let Some(name) = ecus.get_mut(&0x7E1) {
        *name = "gearbox";
    }
    println!("removed: {:?}, len now {}", ecus.remove(&0x7E1), ecus.len());

    // From an array of pairs, then iterate in *sorted* order for stable output:
    let limits = HashMap::from([("speed", 180), ("rpm", 6500), ("temp", 110)]);
    let mut keys: Vec<_> = limits.keys().copied().collect();
    keys.sort();
    for k in keys {
        println!("  {k:<5} = {}", limits[k]); // indexing panics if missing
    }
    // ANCHOR_END: hashmap
}

fn entry_demo() {
    // ANCHOR: entry
    use std::collections::hash_map::Entry;

    let log = "P0301 P0420 P0301 U0100 P0301 P0420";
    let mut counts: HashMap<&str, u32> = HashMap::new();
    for code in log.split_whitespace() {
        *counts.entry(code).or_insert(0) += 1; // one lookup per code
    }
    let mut sorted: Vec<_> = counts.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    println!("counts: {sorted:?}");

    let mut by_system: HashMap<char, Vec<&str>> = HashMap::new();
    for code in log.split_whitespace() {
        by_system
            .entry(code.chars().next().unwrap())
            .or_default()
            .push(code);
    }
    println!("P codes: {}", by_system[&'P'].len());

    let mut odometer: HashMap<&str, u32> = HashMap::new();
    for (car, km) in [("A", 10), ("B", 5), ("A", 7)] {
        odometer
            .entry(car)
            .and_modify(|total| *total += km)
            .or_insert(km);
    }
    println!("A drove {} km", odometer["A"]);

    match odometer.entry("C") {
        Entry::Occupied(e) => println!("C already has {}", e.get()),
        Entry::Vacant(e) => {
            e.insert(0);
            println!("registered C");
        }
    }
    // ANCHOR_END: entry
}

fn sets_demo() {
    // ANCHOR: sets
    let equipped: HashSet<&str> = ["abs", "esc", "acc", "lka"].into_iter().collect();
    let required: HashSet<&str> = ["abs", "esc", "aeb"].into_iter().collect();

    let mut missing: Vec<_> = required.difference(&equipped).collect();
    missing.sort();
    let mut common: Vec<_> = required.intersection(&equipped).collect();
    common.sort();
    println!("missing {missing:?}, present {common:?}");
    println!("all required present? {}", required.is_subset(&equipped));
    println!("union has {} features", equipped.union(&required).count());

    let mut seen = HashSet::new();
    for id in [0x7E8, 0x7E9, 0x7E8] {
        if !seen.insert(id) {
            println!("duplicate response from {id:#x}");
        }
    }
    // ANCHOR_END: sets
}

fn btree_demo() {
    // ANCHOR: btree
    let mut events: BTreeMap<u64, &str> = BTreeMap::new();
    events.insert(2500, "door closed");
    events.insert(100, "ignition on");
    events.insert(1800, "seatbelt fastened");
    events.insert(900, "door opened");

    for (t, e) in &events {
        println!("{t:>5} ms  {e}"); // always sorted by key
    }
    println!("between 500 and 2000 ms:");
    for (t, e) in events.range(500..2000) {
        println!("  {t} {e}");
    }
    println!(
        "first {:?}, last {:?}",
        events.first_key_value(),
        events.last_key_value()
    );

    let ids: BTreeSet<u16> = [0x7E8, 0x100, 0x7DF, 0x100].into_iter().collect();
    println!("sorted unique ids {:x?}", ids);
    // ANCHOR_END: btree
}

fn heap_demo() {
    // ANCHOR: heap
    let mut priorities = BinaryHeap::from([3, 9, 1, 7]);
    priorities.push(5);
    println!("peek {:?}", priorities.peek());
    print!("pops in order:");
    while let Some(p) = priorities.pop() {
        print!(" {p}");
    }
    println!();

    // Earliest deadline first: Reverse turns the max-heap into a min-heap.
    let mut schedule = BinaryHeap::new();
    schedule.push(Reverse((40, "log flush")));
    schedule.push(Reverse((10, "read wheel speeds")));
    schedule.push(Reverse((25, "update display")));
    while let Some(Reverse((deadline, task))) = schedule.pop() {
        println!("t={deadline:>3} ms  {task}");
    }
    // ANCHOR_END: heap
}
