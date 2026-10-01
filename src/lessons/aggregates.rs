//! Lesson: Tuples, arrays & slices.

use crate::lesson::{Lesson, Question, Section};

pub static LESSON: Lesson = Lesson {
    id: "aggregates",
    title: "Tuples, arrays & slices",
    summary: "Fixed-size groupings of values, and slices: borrowed views into contiguous data.",
    source: include_str!("aggregates.rs"),
    sections: &[
        Section::new(
            "Tuples",
            r#"
            A tuple groups a fixed number of values of possibly different
            types: `(i32, &str, bool)`. Fields are accessed by position (`t.0`,
            `t.1`), or, more readably, by **destructuring** with a pattern.

            Tuples are handy for returning several values from a function. Once
            the fields deserve names, a struct (Lesson 10) is better. The empty
            tuple `()` is the **unit type**: the "nothing" value that functions
            return when they don't return anything.
            "#,
        )
        .demo("tuples", tuples),
        Section::new(
            "Arrays",
            r#"
            An array `[T; N]` holds exactly `N` values of type `T`, stored
            inline, so on the stack when it's a local. The length is part of
            the type: `[u8; 4]` and `[u8; 8]` are different types. `[0; 16]`
            makes sixteen zeros.

            Indexing is **bounds-checked**. An out-of-range index panics instead
            of reading random memory. If the index might be out of range,
            `get(i)` returns an `Option` instead. Arrays of `Copy` types are
            themselves `Copy`, so assigning one copies all its elements.
            "#,
        )
        .demo("arrays", arrays),
        Section::new(
            "Slices: views into contiguous data",
            r#"
            A slice `&[T]` is a borrowed view of a run of `T`s that live
            somewhere else, such as an array, a `Vec`, or another slice. It's a
            **fat pointer**: a pointer to the first element plus a length.

            Create one with a range: `&a[1..4]` (start inclusive, end
            exclusive), `&a[..2]`, `&a[2..]`, `&a[..]`, `&a[1..=3]`. A function
            that takes `&[T]` accepts arrays, vectors, and sub-slices alike, so
            prefer `&[T]` over `&Vec<T>` in parameters.
            "#,
        )
        .demo("slices", slices),
        Section::new(
            "Mutable slices",
            r#"
            `&mut [T]` lets a function modify elements in place. It can't grow
            or shrink the underlying storage: a slice has no capacity, only a
            length. Sorting, reversing, filling, swapping and copying all
            operate on mutable slices, so they work on arrays and `Vec`s alike.
            "#,
        )
        .demo("mut_slices", mutable_slices),
        Section::new(
            "The slice toolbox",
            r#"
            Slices have a large set of methods. A few you will use constantly:

            - `first`, `last`, `get`, `split_first`, `split_last` all return
              `Option`s, so they can't panic.
            - `split_at(mid)` divides one slice into two.
            - `chunks(n)` and `chunks_exact(n)` split into pieces, and
              `windows(n)` gives overlapping views.
            - `contains`, `starts_with`, `iter().position(...)`, and
              `binary_search` on sorted data.
            - `concat` and `join` flatten slices of slices.
            "#,
        )
        .demo("toolbox", toolbox),
        Section::new(
            "From slice to array",
            r#"
            Many APIs want a fixed-size array, for example `u32::from_be_bytes`,
            which takes a `[u8; 4]`. If you have a slice, convert with
            `try_into()`: it checks the length at run time and gives you
            `[u8; 4]` or an error. This pattern is the backbone of binary
            protocol parsing (Lesson 31).

            Pattern matching on slices is often even cleaner; you'll see slice
            patterns in Lesson 12.
            "#,
        )
        .demo("to_array", slice_to_array),
        Section::new(
            "Arrays, slices and Vec: a spectrum of ownership",
            r#"
            ```text
            [T; N]      owned, fixed length known at compile time, stored inline
            Box<[T]>    owned, fixed length chosen at run time, on the heap
            Vec<T>      owned, growable, on the heap (pointer, length, capacity)
            &[T]        borrowed view of any of the above
            &mut [T]    borrowed view that may modify elements in place
            ```

            The sizes below make the representation visible. A reference to an
            array is a thin pointer, because the length is in the type. A slice
            reference is two words, and a `Vec` is three.
            "#,
        )
        .demo("sizes", sizes),
    ],
    quiz: &[
        Question::new(
            "Are `[u8; 4]` and `[u8; 5]` the same type?",
            &[
                "Yes",
                "No — the length is part of the type",
                "Only on 64-bit platforms",
                "Only if both are slices",
            ],
            1,
            "Array length is part of the type. A slice `&[u8]` erases the length into a run-time value, which is why it can refer to either.",
        ),
        Question::new(
            "What does `let a = [1, 2, 3]; a.get(10)` return?",
            &["A panic", "None", "0", "A compile error"],
            1,
            "`get` returns `Option<&T>`. Direct indexing `a[i]` with an out-of-range runtime index panics instead.",
        ),
        Question::new(
            "Which parameter type is the most flexible for a read-only function over a sequence of `i32`?",
            &["Vec<i32>", "&Vec<i32>", "&[i32]", "[i32; 10]"],
            2,
            "`&[i32]` accepts arrays, Vecs, and sub-slices. `&Vec<i32>` only accepts Vecs, and `Vec<i32>` takes ownership.",
        ),
        Question::new(
            "On a 64-bit machine, how big is `&[u8]`?",
            &[
                "8 bytes",
                "16 bytes",
                "24 bytes",
                "It depends on the slice length",
            ],
            1,
            "A slice reference is a fat pointer: data pointer (8) + length (8).",
        ),
        Question::new(
            "What does `[1, 2, 3, 4, 5].windows(2).count()` return?",
            &["2", "3", "4", "5"],
            2,
            "Windows overlap: [1,2], [2,3], [3,4], [4,5] — four of them. `chunks(2)` would give three non-overlapping pieces.",
        ),
    ],
    exercises: &[
        "Write `fn min_max(values: &[i32]) -> Option<(i32, i32)>` that returns `None` for an empty slice. Destructure the result at the call site.",
        "Given `[u8; 8]` representing a CAN payload, print it as hex bytes separated by spaces, then as two big-endian `u32`s using `chunks_exact(4)` and `try_into`.",
        "Use `windows(3)` to compute a 3-point moving average over a slice of `f64` sensor readings.",
        "Write `fn rotate_right(data: &mut [i32], k: usize)` without allocating. Then compare with the built-in `rotate_right`.",
    ],
};

fn tuples() {
    // ANCHOR: tuples
    let reading: (&str, f64, bool) = ("coolant", 88.5, true);
    println!("{} = {} (ok: {})", reading.0, reading.1, reading.2);

    let (name, value, _) = reading; // destructure, ignoring the third
    println!("destructured: {name} {value}");

    fn min_max(values: &[i32]) -> (i32, i32) {
        let mut lo = values[0];
        let mut hi = values[0];
        for &v in values {
            lo = lo.min(v);
            hi = hi.max(v);
        }
        (lo, hi) // return two values at once
    }
    let (lo, hi) = min_max(&[4, -2, 9, 7]);
    println!("min {lo}, max {hi}");

    let nested = ((1, 2), [3, 4]);
    let ((a, b), [c, d]) = nested;
    println!("{a} {b} {c} {d}");

    let unit = ();
    println!("unit prints as {unit:?}");
    // ANCHOR_END: tuples
}

fn arrays() {
    // ANCHOR: arrays
    let wheels = [32.1, 32.0, 31.8, 32.2]; // [f64; 4] — tire pressures (psi)
    let zeros = [0u8; 8]; //                   eight zero bytes
    println!("{wheels:?} has {} elements", wheels.len());
    println!("{zeros:?}");

    let mut total = 0.0;
    for p in wheels {
        total += p;
    }
    println!("average pressure {:.2}", total / wheels.len() as f64);

    // Bounds checks: `get` instead of a possible panic.
    let index = 7;
    match wheels.get(index) {
        Some(p) => println!("wheel {index}: {p}"),
        None => println!("there is no wheel {index}"),
    }

    let grid = [[0u8; 3]; 2]; // two rows of three
    println!("grid {grid:?}, grid[1][2] = {}", grid[1][2]);

    let copy = wheels; // arrays of Copy types are Copy
    println!(
        "still usable: {:?} == {:?}: {}",
        wheels[0],
        copy[0],
        wheels == copy
    );
    // ANCHOR_END: arrays
}

fn slices() {
    // ANCHOR: slices
    fn sum(values: &[i32]) -> i32 {
        let mut total = 0;
        for v in values {
            total += v;
        }
        total
    }

    let array = [10, 20, 30, 40, 50];
    let vector = vec![1, 2, 3];

    println!("whole array   {}", sum(&array)); //      &[i32; 5] coerces to &[i32]
    println!("a vector      {}", sum(&vector)); //     &Vec<i32> coerces too
    println!("middle three  {}", sum(&array[1..4])); // 20+30+40
    println!("first two     {:?}", &array[..2]);
    println!("from index 3  {:?}", &array[3..]);
    println!("inclusive     {:?}", &array[1..=2]);
    println!("empty         {:?}", &array[2..2]);
    // ANCHOR_END: slices
}

fn mutable_slices() {
    // ANCHOR: mut_slices
    fn scale(values: &mut [i32], factor: i32) {
        for v in values.iter_mut() {
            *v *= factor; // `*` writes through the reference
        }
    }

    let mut data = [5, 3, 9, 1, 7];
    scale(&mut data[..2], 10); // only the first two
    println!("scaled  {data:?}");

    data.sort();
    println!("sorted  {data:?}");
    data.reverse();
    println!("reversed {data:?}");
    data.swap(0, 4);
    println!("swapped {data:?}");
    data[1..3].fill(0);
    println!("filled  {data:?}");
    data[3..].copy_from_slice(&[8, 8]); // lengths must match
    println!("copied  {data:?}");
    data.rotate_left(1);
    println!("rotated {data:?}");
    // ANCHOR_END: mut_slices
}

fn toolbox() {
    // ANCHOR: toolbox
    let frame = [0x7E, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x7E];
    println!("first {:?} last {:?}", frame.first(), frame.last());

    if let Some((header, rest)) = frame.split_first() {
        println!("header {header:#04x}, {} bytes follow", rest.len());
    }

    let (left, right) = frame.split_at(3);
    println!("split_at(3): {left:?} | {right:?}");

    for chunk in frame.chunks(3) {
        print!("{chunk:?} "); // the last chunk may be shorter
    }
    println!();

    let exact = frame.chunks_exact(3);
    println!("chunks_exact remainder: {:?}", exact.remainder());

    let rising = [1, 3, 4, 8, 9];
    let diffs: Vec<i32> = rising.windows(2).map(|w| w[1] - w[0]).collect();
    println!("differences {diffs:?}");

    println!(
        "contains 4? {}  position of 8: {:?}",
        rising.contains(&4),
        rising.iter().position(|&x| x == 8)
    );
    println!(
        "binary_search 8 → {:?}, 5 → {:?}",
        rising.binary_search(&8),
        rising.binary_search(&5)
    );
    println!("starts_with [1,3]? {}", rising.starts_with(&[1, 3]));

    let words = [["ab", "cd"], ["ef", "gh"]];
    println!("concat {:?}", words.concat());
    println!("join   {:?}", ["a", "b", "c"].join("-"));
    // ANCHOR_END: toolbox
}

fn slice_to_array() {
    // ANCHOR: to_array
    let packet: &[u8] = &[0x00, 0x00, 0x01, 0x2C, 0xFF];

    // Take the first four bytes as a fixed-size array:
    let head: [u8; 4] = packet[..4].try_into().expect("slice has 4 bytes");
    println!("as u32 (big-endian): {}", u32::from_be_bytes(head));

    // A length mismatch is an error, not a panic:
    let wrong: Result<[u8; 4], _> = packet[..3].try_into();
    println!("3 bytes into [u8; 4]: {wrong:?}");

    // `get` + `try_into` = never panics, whatever the input length:
    fn read_u16(bytes: &[u8], at: usize) -> Option<u16> {
        let pair: [u8; 2] = bytes.get(at..at + 2)?.try_into().ok()?;
        Some(u16::from_be_bytes(pair))
    }
    println!("read_u16(2) = {:?}", read_u16(packet, 2));
    println!("read_u16(4) = {:?}", read_u16(packet, 4)); // only one byte left
    // ANCHOR_END: to_array
}

fn sizes() {
    // ANCHOR: sizes
    use std::mem::size_of;
    let word = size_of::<usize>();
    println!("one machine word      = {word} bytes");
    println!(
        "[u8; 100]             = {} bytes (inline data)",
        size_of::<[u8; 100]>()
    );
    println!(
        "&[u8; 100]            = {} bytes (thin pointer)",
        size_of::<&[u8; 100]>()
    );
    println!(
        "&[u8]                 = {} bytes (pointer + len)",
        size_of::<&[u8]>()
    );
    println!(
        "Box<[u8]>             = {} bytes (pointer + len)",
        size_of::<Box<[u8]>>()
    );
    println!(
        "Vec<u8>               = {} bytes (pointer + len + capacity)",
        size_of::<Vec<u8>>()
    );

    let v = vec![1, 2, 3];
    let boxed: Box<[i32]> = v.into_boxed_slice(); // drop spare capacity
    println!("boxed slice {boxed:?}");
    // ANCHOR_END: sizes
}
