//! Lesson: Iterators.

use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::iter;

use crate::lesson::{Lesson, Question, Section};

pub static LESSON: Lesson = Lesson {
    id: "iterators",
    title: "Iterators",
    summary: "The Iterator trait, iter/iter_mut/into_iter, laziness, adapters, consumers, collect, writing your own iterators and collections, and constructors in std::iter.",
    source: include_str!("iterators.rs"),
    sections: &[
        Section::new(
            "The Iterator trait",
            r#"
            An iterator is anything with a `next` method:

            ```rust
            trait MyIterator {
                type Item;
                fn next(&mut self) -> Option<Self::Item>;
                // ...plus about 75 provided methods built on `next`
            }
            ```

            Each call returns `Some(item)` until the iterator is exhausted, and
            then `None`. Everything else, every adapter and consumer, is a
            provided method built on `next`. A `for` loop is just a `while
            let Some(x) = iter.next()` that the compiler writes for you.
            "#,
        )
        .demo("trait", the_trait),
        Section::new(
            "iter, iter_mut and into_iter",
            r#"
            Collections offer three iterators, one for each point on the
            ownership spectrum:

            ```text
            v.iter()        yields &T       borrow — the collection is untouched
            v.iter_mut()    yields &mut T   modify elements in place
            v.into_iter()   yields T        consume — the collection is gone
            ```

            `for x in &v`, `for x in &mut v` and `for x in v` call these three
            through the `IntoIterator` trait. Arrays (since edition 2021) and
            boxed slices (since 2024) iterate by value with `into_iter()`.
            "#,
        )
        .demo("three", three_ways),
        Section::new(
            "Laziness",
            r#"
            Adapters like `map` and `filter` do **nothing** until something
            pulls items through them. They return a new iterator that wraps
            the old one. This is why you can chain them without creating
            intermediate collections, and why iterators over infinite sequences
            work. The compiler warns if you build an adapter chain and never
            consume it (`unused_must_use`).
            "#,
        )
        .demo("lazy", laziness),
        Section::new(
            "Adapters: transforming streams",
            r#"
            Adapters take an iterator and return a new one:

            ```text
            map, filter, filter_map, flat_map, flatten     transform / select
            enumerate, zip, chain, rev, cycle, step_by     combine / reorder
            take, skip, take_while, skip_while, map_while  limit
            peekable, scan, inspect, fuse                  stateful / debugging
            ```

            `filter_map` combines `filter` and `map` for functions that
            return `Option`. `flat_map` is for functions that return many items
            per input. `scan` carries running state, such as a cumulative sum.
            `peekable` lets a parser look at the next item without consuming
            it.
            "#,
        )
        .demo("adapters", adapters),
        Section::new(
            "Consumers: producing a result",
            r#"
            Consumers drive the iterator and produce a final value:

            ```text
            collect, count, sum, product, last, nth
            min, max, min_by_key, max_by, ...       (Option: empty → None)
            fold, reduce, try_fold                  general accumulation
            any, all, find, find_map, position      searching (short-circuit)
            partition, unzip, for_each              splitting / side effects
            ```

            `fold(init, |acc, x| …)` is the general form; most of the others
            are special cases of it. `try_fold` stops at the first `Err` or
            `None`, which is ideal for overflow-checked sums.
            "#,
        )
        .demo("consumers", consumers),
        Section::new(
            "collect: one method, many destinations",
            r#"
            `collect` builds any type implementing `FromIterator`. The target
            type, from an annotation or a turbofish, decides what you get:
            `Vec`, `String`, `HashMap` (from pairs), `HashSet`, `BTreeMap`,
            `VecDeque`, `Box<[T]>`, and also `Result<Vec<T>, E>` or
            `Option<Vec<T>>`, which short-circuit on the first failure.
            "#,
        )
        .demo("collect", collect),
        Section::new(
            "Writing your own iterator",
            r#"
            Implement `Iterator` with an `Item` type and `next`, and your type
            gains every adapter and consumer. Keep the iteration state in the
            struct. Override `size_hint` when you know the remaining length,
            so `collect` can pre-allocate.
            "#,
        )
        .demo("custom", custom_iterator),
        Section::new(
            "Making a collection iterable",
            r#"
            For `for x in &my_collection` to work, implement `IntoIterator for
            &MyCollection`, usually by returning an existing iterator such as
            `self.items.iter()`. Implement it for `MyCollection` (by value) and
            `&mut MyCollection` too, for the full set.
            "#,
        )
        .demo("into_iter", into_iterator),
        Section::new(
            "Iterator constructors in std::iter",
            r#"
            You don't always need a struct to make an iterator:

            - `iter::from_fn(|| …)` builds one from a closure that returns
              `Option<T>`;
            - `iter::successors(first, |prev| next)` produces each item from
              the previous one;
            - `iter::repeat(x)`, `repeat_n(x, n)`, `once(x)` and `empty()` cover
              the simple cases;
            - `iter::zip(a, b)` is the free-function form of `a.zip(b)`.
            "#,
        )
        .demo("constructors", constructors),
        Section::new(
            "DoubleEnded, ExactSize, and zero cost",
            r#"
            Some iterators can also run backwards (`DoubleEndedIterator`), which
            is required by `rev()` and `next_back()`. Some know their exact
            length (`ExactSizeIterator`, which provides `len()`).

            Iterator chains are a **zero-cost abstraction**. After inlining, a
            `filter().map().sum()` chain compiles to the same loop you would
            write by hand, and slice iterators avoid per-element bounds checks
            that manual indexing can incur. Prefer iterators for clarity
            without fearing the performance cost.
            "#,
        )
        .demo("double", double_ended),
    ],
    quiz: &[
        Question::new(
            "What does `let it = v.iter().map(|x| x * 2);` compute immediately?",
            &[
                "The doubled Vec",
                "Nothing — adapters are lazy",
                "The first element",
                "The length",
            ],
            1,
            "Adapters build a new iterator; work happens only when a consumer (collect, sum, for…) pulls items.",
        ),
        Question::new(
            "Which yields owned `String`s from a `Vec<String>`, consuming the Vec?",
            &[
                "v.iter()",
                "v.iter_mut()",
                "v.into_iter()",
                "v.iter().cloned()",
            ],
            2,
            "`into_iter()` on the Vec by value moves each element out. `iter().cloned()` also yields owned Strings, but by cloning and leaving v intact.",
        ),
        Question::new(
            "What does `[1, 2, 3, 4].iter().scan(0, |acc, &x| { *acc += x; Some(*acc) })` produce?",
            &["1, 2, 3, 4", "1, 3, 6, 10", "10", "0, 1, 3, 6"],
            1,
            "`scan` threads state through the iteration and yields each intermediate running total.",
        ),
        Question::new(
            "To implement Iterator for your type, which method must you write?",
            &["iter", "next", "into_iter", "collect"],
            1,
            "`next` is the only required method; everything else is provided.",
        ),
        Question::new(
            "What does `(1..=4).fold(0, |acc, x| acc * 10 + x)` return?",
            &["10", "1234", "4321", "24"],
            1,
            "fold accumulates left to right: ((((0·10+1)·10+2)·10+3)·10+4) = 1234.",
        ),
    ],
    exercises: &[
        "Using only iterator methods, compute the sum of squares of the odd numbers in `1..=100`.",
        "Given `&[(&str, u32)]` of (sensor, value), build a `HashMap<&str, Vec<u32>>` grouping values per sensor with `fold`.",
        "Implement `struct Fibonacci` as an Iterator over u64 that stops (returns None) instead of overflowing. How many items does it yield?",
        "Write a tokenizer for `12+(3*45)` using `chars().peekable()` that groups consecutive digits into numbers.",
    ],
};

fn the_trait() {
    // ANCHOR: trait
    let gears = ['P', 'R', 'N', 'D'];
    let mut it = gears.iter(); // an iterator over &char
    println!("{:?}", it.next()); // Some('P')
    println!("{:?}", it.next()); // Some('R')
    println!("rest via for:");
    for g in it {
        // `for` keeps calling next() until None
        println!("  {g}");
    }

    // What `for x in gears` expands to (roughly):
    let mut iter = IntoIterator::into_iter(gears);
    while let Some(g) = iter.next() {
        print!("{g} ");
    }
    println!();
    // ANCHOR_END: trait
}

fn three_ways() {
    // ANCHOR: three
    let mut readings = vec![String::from("88.0"), String::from("89.5")];

    for r in readings.iter() {
        print!("{r} "); // r: &String
    }
    println!("← iter()");

    for r in readings.iter_mut() {
        r.push_str(" °C"); // r: &mut String
    }
    println!("{readings:?} ← after iter_mut()");

    let owned: Vec<String> = readings.into_iter().collect(); // r: String
    println!("{owned:?} ← into_iter() moved them");
    // println!("{readings:?}"); // error: readings was consumed

    for x in [1, 2, 3] {
        print!("{x} "); // arrays yield values (edition 2021+)
    }
    println!();
    // ANCHOR_END: three
}

fn laziness() {
    // ANCHOR: lazy
    let chain = (1..=5)
        .inspect(|x| println!("  produced {x}"))
        .filter(|x| x % 2 == 1)
        .map(|x| x * 100);
    println!("chain built — nothing has run yet");

    let first_two: Vec<i32> = chain.take(2).collect(); // pull only what's needed
    println!("first two: {first_two:?}");

    // Infinite iterators are fine when you limit them:
    let powers: Vec<u64> = (0..)
        .map(|n| 2u64.pow(n))
        .take_while(|&p| p < 100)
        .collect();
    println!("powers of two below 100: {powers:?}");
    // ANCHOR_END: lazy
}

fn adapters() {
    // ANCHOR: adapters
    let raw = ["12", "x", "7", "", "30"];
    let nums: Vec<u32> = raw.iter().filter_map(|s| s.parse().ok()).collect();
    println!("filter_map   {nums:?}");

    let doubled: Vec<u32> = nums.iter().map(|n| n * 2).filter(|n| *n > 20).collect();
    println!("map+filter   {doubled:?}");

    let words = ["brake pad", "oil filter"];
    let all: Vec<&str> = words.iter().flat_map(|w| w.split(' ')).collect();
    println!("flat_map     {all:?}");
    println!(
        "flatten      {:?}",
        vec![vec![1, 2], vec![], vec![3]]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
    );

    let names = ["FL", "FR", "RL", "RR"];
    let pressures = [32.0, 31.5, 29.0, 32.5];
    for (i, (n, p)) in names.iter().zip(pressures).enumerate() {
        print!("{i}:{n}={p} ");
    }
    println!(" ← zip+enumerate");

    println!("chain        {:?}", (1..3).chain(7..9).collect::<Vec<_>>());
    println!(
        "skip/take    {:?}",
        (0..20).skip(5).step_by(4).take(3).collect::<Vec<_>>()
    );
    println!("rev          {:?}", "abc".chars().rev().collect::<String>());
    println!(
        "take_while   {:?}",
        [1, 3, 5, 6, 7]
            .iter()
            .take_while(|n| *n % 2 == 1)
            .collect::<Vec<_>>()
    );
    println!(
        "skip_while   {:?}",
        [0, 0, 5, 0, 1]
            .iter()
            .skip_while(|n| **n == 0)
            .collect::<Vec<_>>()
    );
    println!(
        "map_while    {:?}",
        ["1", "2", "x", "4"]
            .iter()
            .map_while(|s| s.parse::<i32>().ok())
            .collect::<Vec<_>>()
    );
    println!(
        "cycle        {:?}",
        ['L', 'R'].iter().cycle().take(5).collect::<String>()
    );

    let running: Vec<i32> = [3, -1, 4, -2]
        .iter()
        .scan(0, |total, &x| {
            *total += x;
            Some(*total)
        })
        .collect();
    println!("scan         {running:?}");

    let mut tokens = "12+345".chars().peekable();
    let mut digits = String::new();
    while let Some(c) = tokens.next_if(|c| c.is_ascii_digit()) {
        digits.push(c);
    }
    println!(
        "peekable     first number {digits:?}, next is {:?}",
        tokens.peek()
    );
    // ANCHOR_END: adapters
}

fn consumers() {
    // ANCHOR: consumers
    let speeds = [62, 88, 45, 120, 97];
    println!(
        "count {}  sum {}  product of first 3 {}",
        speeds.len(),
        speeds.iter().sum::<i32>(),
        speeds.iter().take(3).product::<i32>()
    );
    println!(
        "min {:?}  max {:?}  last {:?}  nth(1) {:?}",
        speeds.iter().min(),
        speeds.iter().max(),
        speeds.iter().last(),
        speeds.iter().nth(1)
    );

    let cars: [(&str, f64); 3] = [("Prius", 1.8), ("Supra", 3.0), ("GR86", 2.4)];
    let biggest = cars.iter().max_by(|a, b| a.1.total_cmp(&b.1));
    let shortest = cars.iter().min_by_key(|c| c.0.len());
    println!("max_by {biggest:?}  min_by_key {shortest:?}");

    let total = speeds.iter().fold(0, |acc, s| acc + s);
    let longest = ["a", "abc", "ab"]
        .into_iter()
        .reduce(|a, b| if b.len() > a.len() { b } else { a });
    println!("fold {total}  reduce {longest:?}");

    let checked: Option<u8> = [100u8, 100, 100]
        .iter()
        .try_fold(0u8, |acc, &x| acc.checked_add(x));
    println!("try_fold overflow → {checked:?}");

    println!(
        "any > 100? {}  all > 40? {}",
        speeds.iter().any(|&s| s > 100),
        speeds.iter().all(|&s| s > 40)
    );
    println!(
        "find {:?}  position {:?}",
        speeds.iter().find(|&&s| s > 90),
        speeds.iter().position(|&s| s > 90)
    );
    println!(
        "find_map {:?}",
        ["x", "42", "7"].iter().find_map(|s| s.parse::<u8>().ok())
    );

    let (fast, slow): (Vec<i32>, Vec<i32>) = speeds.iter().partition(|&&s| s > 80);
    println!("partition fast {fast:?} slow {slow:?}");
    let (models, litres): (Vec<&str>, Vec<f64>) = cars.iter().copied().unzip();
    println!("unzip {models:?} {litres:?}");
    // ANCHOR_END: consumers
}

fn collect() {
    // ANCHOR: collect
    let pairs = [("rpm", 3200), ("temp", 88), ("rpm", 3400)];

    let v: Vec<u32> = pairs.iter().map(|p| p.1).collect();
    let s: String = ['o', 'k'].iter().collect();
    let joined: String = pairs.iter().map(|p| p.0).collect::<Vec<_>>().join(",");
    let map: HashMap<&str, u32> = pairs.iter().copied().collect(); // later keys win
    let set: HashSet<&str> = pairs.iter().map(|p| p.0).collect();
    let sorted: BTreeMap<&str, u32> = pairs.iter().copied().collect();
    let queue: VecDeque<u32> = (1..=3).collect();
    let boxed: Box<[u8]> = (0..4).collect();

    println!(
        "Vec {v:?}\nString {s:?} / {joined:?}\nHashMap rpm={:?}",
        map.get("rpm")
    );
    println!(
        "HashSet len {}\nBTreeMap {sorted:?}\nVecDeque {queue:?}\nBox<[u8]> {boxed:?}",
        set.len()
    );

    let ok: Result<Vec<i32>, _> = ["1", "2"].iter().map(|s| s.parse::<i32>()).collect();
    let none: Option<Vec<char>> = ["a", "", "c"].iter().map(|s| s.chars().next()).collect();
    println!("Result {ok:?}  Option {none:?}");
    // ANCHOR_END: collect
}

// ANCHOR: custom
/// Counts down from `from` to 1, then stops.
struct Countdown {
    remaining: u32,
}

impl Iterator for Countdown {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        if self.remaining == 0 {
            None
        } else {
            self.remaining -= 1;
            Some(self.remaining + 1)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.remaining as usize;
        (n, Some(n)) // exact: lets collect() allocate once
    }
}

fn custom_iterator() {
    let launch: Vec<u32> = Countdown { remaining: 5 }.collect();
    println!("{launch:?}");
    // All the adapters come for free:
    let even_squares: Vec<u32> = Countdown { remaining: 10 }
        .filter(|n| n % 2 == 0)
        .map(|n| n * n)
        .collect();
    println!("{even_squares:?}");
    println!("sum {}", Countdown { remaining: 100 }.sum::<u32>());
}
// ANCHOR_END: custom

fn into_iterator() {
    // ANCHOR: into_iter
    struct Fleet {
        vins: Vec<String>,
    }

    impl<'a> IntoIterator for &'a Fleet {
        type Item = &'a String;
        type IntoIter = std::slice::Iter<'a, String>;
        fn into_iter(self) -> Self::IntoIter {
            self.vins.iter()
        }
    }

    impl IntoIterator for Fleet {
        type Item = String;
        type IntoIter = std::vec::IntoIter<String>;
        fn into_iter(self) -> Self::IntoIter {
            self.vins.into_iter()
        }
    }

    let fleet = Fleet {
        vins: vec!["JT1".into(), "JT2".into(), "JT3".into()],
    };
    for vin in &fleet {
        print!("{vin} "); // borrowed
    }
    println!();
    let count = (&fleet).into_iter().filter(|v| v.ends_with('2')).count();
    println!("{count} VIN(s) end in 2");
    let owned: Vec<String> = fleet.into_iter().collect(); // consumed
    println!("{owned:?}");
    // ANCHOR_END: into_iter
}

fn constructors() {
    // ANCHOR: constructors
    let mut state = 1u32;
    let doubling = iter::from_fn(|| {
        state *= 2;
        (state < 100).then_some(state)
    });
    println!("from_fn    {:?}", doubling.collect::<Vec<_>>());

    let halvings: Vec<u32> =
        iter::successors(Some(1000u32), |&n| (n > 1).then_some(n / 2)).collect();
    println!("successors {halvings:?}");

    let header: Vec<u8> = iter::once(0x7E).chain(iter::repeat_n(0x00, 3)).collect();
    println!("once+repeat_n {header:02x?}");
    let nothing: Vec<i32> = iter::empty().collect();
    println!("empty      {nothing:?}");
    let pairs: Vec<(char, i32)> = iter::zip("abc".chars(), 1..).collect();
    println!("zip        {pairs:?}");
    // ANCHOR_END: constructors
}

fn double_ended() {
    // ANCHOR: double
    let mut it = [1, 2, 3, 4, 5].iter();
    println!(
        "front {:?}, back {:?}, len now {}",
        it.next(),
        it.next_back(),
        it.len()
    );
    println!("remaining {:?}", it.collect::<Vec<_>>());

    // The same computation, two styles: the iterator chain compiles to an
    // equally tight loop.
    let data: Vec<u32> = (1..=1000).collect();
    let mut manual = 0u64;
    for i in 0..data.len() {
        if data[i] % 3 == 0 {
            manual += u64::from(data[i]) * 2;
        }
    }
    let chained: u64 = data
        .iter()
        .filter(|&&x| x % 3 == 0)
        .map(|&x| u64::from(x) * 2)
        .sum();
    println!(
        "manual {manual} == chained {chained}: {}",
        manual == chained
    );
    // ANCHOR_END: double
}
