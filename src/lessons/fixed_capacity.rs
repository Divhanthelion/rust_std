//! Lesson: Fixed-capacity, allocation-free data.

use std::fmt;

use crate::alloc_counter;
use crate::lesson::{Lesson, Question, Section};

pub static LESSON: Lesson = Lesson {
    id: "fixed_capacity",
    title: "Fixed-capacity, allocation-free data",
    summary: "Why safety-critical code avoids the heap, and how to build bounded containers: a stack vector, a ring buffer with overflow policies, a fixed string, and a generational pool — measured to allocate nothing.",
    source: include_str!("fixed_capacity.rs"),
    sections: &[
        Section::new(
            "Why avoid the heap?",
            r#"
            Heap allocation is convenient but awkward to **bound**:

            - **Timing**: an allocation can take microseconds or milliseconds,
              depending on fragmentation. A 1 ms control loop can't afford a
              surprise.
            - **Exhaustion**: running out of memory at 80 km/h isn't a
              recoverable condition, and Rust's default response is to abort.
            - **Fragmentation**: a long-running ECU may fail after weeks, not
              minutes.

            Automotive guidelines (MISRA, AUTOSAR) therefore restrict dynamic
            memory. The policies form a spectrum:

            ```text
            static only          every buffer sized at compile time ([T; N], statics)
            allocate at init     reserve everything at start-up, never after
            bounded allocation   allocate, but with caps and try_reserve
            free allocation      typical desktop/server code
            ```

            This lesson stays at the strict end and **measures** the result.
            This program installs a counting allocator (`src/alloc_counter.rs`),
            so every demo can print how many allocations it made.
            "#,
        )
        .demo("measure", measure),
        Section::new(
            "A stack-allocated vector",
            r#"
            `StackVec<T, N>` keeps up to `N` elements in an inline array. Its
            `try_push` returns `Err(value)` when full, so the caller gets the
            value back and decides what to do. Storing `[Option<T>; N]` keeps
            the code entirely safe for any `T`, at the cost of a discriminant
            per slot. Crates like `heapless` and `arrayvec` use
            `MaybeUninit<T>` and a little `unsafe` to remove even that
            overhead (Lesson 36 shows the tools).
            "#,
        )
        .demo("stackvec", stack_vec),
        Section::new(
            "A ring buffer with an overflow policy",
            r#"
            When a fixed buffer is full, something has to give. The choices are
            a spectrum of who loses:

            ```text
            Reject newest     commands, setpoints: never silently drop a newer request
            Overwrite oldest  logs, sensor history: keep the most recent N
            Block / backpressure   producer waits (threads, Lesson 23)
            Escalate          report a fault: the system is overloaded
            ```

            The ring buffer below makes the policy explicit in the
            constructor, and counts what it dropped. A silent drop is a bug
            report you'll never receive.
            "#,
        )
        .demo("ring", ring),
        Section::new(
            "A fixed-capacity string, with write!",
            r#"
            Formatting usually means `String`, which means the heap. With
            `FixedStr<N>`, a byte array plus a length, and an implementation of
            `fmt::Write`, `write!` formats straight into a stack buffer. On
            overflow it returns `fmt::Error` instead of allocating. The
            invariant "the first `len` bytes are valid UTF-8" holds because we
            only ever append whole `&str`s, and copy only if all of it fits.
            "#,
        )
        .demo("fixedstr", fixed_str),
        Section::new(
            "Pools and generational handles",
            r#"
            Many systems need to create and delete objects at run time, for
            example diagnostic sessions or timers, without a heap. A **pool**
            preallocates `N` slots and hands out **handles** (indices) instead
            of references.

            Plain indices have a trap: delete object 3, create another one in
            slot 3, and a stale handle now silently points at the wrong object.
            **Generational** handles store a counter with the index, and the
            pool rejects a handle whose generation doesn't match the slot's.
            It's use-after-free detection in safe code, and it sidesteps
            lifetime puzzles because handles are just `Copy` integers.
            "#,
        )
        .demo("pool", pool),
        Section::new(
            "Allocate once, at initialization",
            r#"
            The middle of the spectrum is often the pragmatic choice. Size
            collections up front with `Vec::with_capacity`, using
            `try_reserve` to turn allocation failure into an error you can
            report, and never grow them during operation. Measuring confirms
            the steady state allocates nothing. Then a test can **assert**
            it, so a future change that sneaks in a `format!` or a `clone()`
            fails CI instead of failing on the road.
            "#,
        )
        .demo("init", init_once),
    ],
    quiz: &[
        Question::new(
            "Why do automotive coding guidelines restrict heap allocation after initialization?",
            &[
                "The heap is slower to read",
                "Allocation time and failure are hard to bound, and fragmentation can cause late failures",
                "Rust can't allocate on microcontrollers",
                "It's a style preference",
            ],
            1,
            "Real-time and safety arguments need bounded timing and guaranteed memory. Dynamic allocation makes both hard to prove.",
        ),
        Question::new(
            "A command queue is full. Which overflow policy is usually WRONG for it?",
            &[
                "Reject the newest command and report it",
                "Silently overwrite the oldest command",
                "Apply backpressure",
                "Escalate as a fault",
            ],
            1,
            "Silently discarding a pending command (e.g. 'release brake') can be dangerous; overwrite-oldest suits logs, not commands.",
        ),
        Question::new(
            "What problem do generational handles solve?",
            &[
                "Slow lookups",
                "Stale handles silently referring to a new object that reused the same slot",
                "Thread safety",
                "Memory leaks",
            ],
            1,
            "The generation counter changes when a slot is reused, so old handles are detected and rejected.",
        ),
        Question::new(
            "How can `write!` work without any heap allocation?",
            &[
                "It can't",
                "By implementing fmt::Write for a fixed buffer type, which returns fmt::Error when full",
                "By using a static String",
                "By printing directly",
            ],
            1,
            "`write!` only needs a `write_str` method. A stack buffer that implements fmt::Write is enough.",
        ),
        Question::new(
            "Why does `StackVec::try_push` return `Result<(), T>` instead of `Result<(), ()>`?",
            &[
                "Convention only",
                "So the caller gets the value back and can retry, log or drop it deliberately",
                "Because T must be Copy",
                "To allocate",
            ],
            1,
            "Returning ownership of the rejected value means nothing is lost implicitly.",
        ),
    ],
    exercises: &[
        "Add `pop`, `clear` and `iter()` to `StackVec`. Why can't it implement `Deref<Target = [T]>` while its slots are `Option<T>`, and how would `MaybeUninit<T>` change that?",
        "Add an `Escalate` policy to the ring buffer that returns an error *and* counts the overflow.",
        "Implement `fmt::Display` for `FixedStr` and use it to format a CAN frame as text with zero allocations; assert it with `alloc_counter::count`.",
        "Make the pool's `get_mut` and `remove` methods, and write a test that a removed handle can never access the slot's new occupant.",
    ],
};

fn measure() {
    // ANCHOR: measure
    let (_, heap) = alloc_counter::count(|| {
        let v: Vec<u32> = (0..100).collect(); // allocates
        let s = format!("{} items", v.len()); // allocates again
        s.len()
    });
    let (_, stack) = alloc_counter::count(|| {
        let a = [0u32; 100]; // lives on the stack
        a.iter().map(|x| x + 1).sum::<u32>()
    });
    println!("Vec + format!: {heap} allocations");
    println!("array + iterator: {stack} allocations");
    // ANCHOR_END: measure
}

// ANCHOR: stackvec
/// Up to N elements, stored inline. Never allocates.
pub struct StackVec<T, const N: usize> {
    slots: [Option<T>; N],
    len: usize,
}

impl<T, const N: usize> StackVec<T, N> {
    pub fn new() -> Self {
        StackVec {
            slots: std::array::from_fn(|_| None),
            len: 0,
        }
    }

    /// Adds `value`, or hands it back if the vector is full.
    pub fn try_push(&mut self, value: T) -> Result<(), T> {
        match self.slots.get_mut(self.len) {
            Some(slot) => {
                *slot = Some(value);
                self.len += 1;
                Ok(())
            }
            None => Err(value),
        }
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.slots.get(index)?.as_ref()
    }

    pub fn len(&self) -> usize {
        self.len
    }
}

fn stack_vec() {
    // Measure only the container work; printing happens afterwards.
    let ((ids, rejected), allocations) = alloc_counter::count(|| {
        let mut ids: StackVec<u16, 4> = StackVec::new();
        let mut rejected = None;
        for id in [0x100, 0x200, 0x7E0, 0x7E8, 0x7DF] {
            if let Err(id) = ids.try_push(id) {
                rejected = Some(id);
            }
        }
        (ids, rejected)
    });
    println!("full: rejected {rejected:x?}");
    println!(
        "len {}, ids[2] = {:x?}, ids[9] = {:?}",
        ids.len(),
        ids.get(2),
        ids.get(9)
    );
    println!("allocations: {allocations}");
}
// ANCHOR_END: stackvec

// ANCHOR: ring
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Overflow {
    RejectNewest,
    OverwriteOldest,
}

pub struct Ring<T, const N: usize> {
    slots: [Option<T>; N],
    head: usize, // index of the oldest element
    len: usize,
    policy: Overflow,
    dropped: u32,
}

impl<T, const N: usize> Ring<T, N> {
    pub fn new(policy: Overflow) -> Self {
        Ring {
            slots: std::array::from_fn(|_| None),
            head: 0,
            len: 0,
            policy,
            dropped: 0,
        }
    }

    pub fn push(&mut self, value: T) -> Result<(), T> {
        if N == 0 {
            return Err(value);
        }
        if self.len == N {
            self.dropped += 1;
            match self.policy {
                Overflow::RejectNewest => return Err(value),
                Overflow::OverwriteOldest => {
                    self.slots[self.head] = Some(value); // replaces the oldest
                    self.head = (self.head + 1) % N;
                    return Ok(());
                }
            }
        }
        let tail = (self.head + self.len) % N;
        self.slots[tail] = Some(value);
        self.len += 1;
        Ok(())
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        let value = self.slots[self.head].take();
        self.head = (self.head + 1) % N;
        self.len -= 1;
        value
    }

    /// Oldest to newest, without removing anything.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        (0..self.len).filter_map(move |i| self.slots[(self.head + i) % N].as_ref())
    }

    pub fn dropped(&self) -> u32 {
        self.dropped
    }
}

fn ring() {
    let mut log: Ring<u32, 3> = Ring::new(Overflow::OverwriteOldest);
    let mut commands: Ring<&str, 2> = Ring::new(Overflow::RejectNewest);

    let (rejected, allocations) = alloc_counter::count(|| {
        for t in 1..=5 {
            let _ = log.push(t * 100);
        }
        let mut rejected = None;
        for cmd in ["unlock", "lights on", "horn"] {
            if let Err(cmd) = commands.push(cmd) {
                rejected = Some(cmd);
            }
        }
        rejected
    });
    println!("command queue full: {rejected:?} rejected");
    println!(
        "log keeps the newest: {:?} (dropped {})",
        log.iter().collect::<Vec<_>>(),
        log.dropped()
    );
    println!(
        "commands keep the oldest: {:?} (dropped {})",
        commands.iter().collect::<Vec<_>>(),
        commands.dropped()
    );
    println!("next command: {:?}", commands.pop());
    println!("allocations while pushing: {allocations}");
}
// ANCHOR_END: ring

// ANCHOR: fixedstr
/// Up to N bytes of UTF-8 text on the stack.
pub struct FixedStr<const N: usize> {
    buf: [u8; N],
    len: usize, // invariant: buf[..len] is valid UTF-8
}

impl<const N: usize> FixedStr<N> {
    pub const fn new() -> Self {
        FixedStr {
            buf: [0; N],
            len: 0,
        }
    }

    pub fn as_str(&self) -> &str {
        // Never fails: we only append whole &str values (see write_str).
        std::str::from_utf8(&self.buf[..self.len]).unwrap_or("")
    }
}

impl<const N: usize> fmt::Write for FixedStr<N> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let end = self.len.checked_add(s.len()).ok_or(fmt::Error)?;
        let dest = self.buf.get_mut(self.len..end).ok_or(fmt::Error)?; // all or nothing
        dest.copy_from_slice(s.as_bytes());
        self.len = end;
        Ok(())
    }
}

fn fixed_str() {
    use std::fmt::Write as _;

    let mut line: FixedStr<32> = FixedStr::new();
    let (result, allocations) = alloc_counter::count(|| write!(line, "rpm={} temp={}°C", 3200, 88));
    println!(
        "{:?} → {:?} ({} bytes, {allocations} allocations)",
        result,
        line.as_str(),
        line.as_str().len()
    );

    let mut tiny: FixedStr<8> = FixedStr::new();
    let overflow = write!(tiny, "speed={}", 123_456);
    // write_str copies all or nothing, so the buffer never holds a torn piece.
    println!(
        "overflow → {overflow:?}, contents afterwards: {:?}",
        tiny.as_str()
    );
}
// ANCHOR_END: fixedstr

// ANCHOR: pool
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Handle {
    index: u16,
    generation: u16,
}

struct Slot<T> {
    generation: u16,
    value: Option<T>,
}

pub struct Pool<T, const N: usize> {
    slots: [Slot<T>; N],
}

impl<T, const N: usize> Pool<T, N> {
    pub fn new() -> Self {
        Pool {
            slots: std::array::from_fn(|_| Slot {
                generation: 0,
                value: None,
            }),
        }
    }

    pub fn insert(&mut self, value: T) -> Result<Handle, T> {
        let Some((index, slot)) = self
            .slots
            .iter_mut()
            .enumerate()
            .find(|(_, s)| s.value.is_none())
        else {
            return Err(value); // pool exhausted
        };
        slot.value = Some(value);
        Ok(Handle {
            index: index as u16,
            generation: slot.generation,
        })
    }

    pub fn get(&self, h: Handle) -> Option<&T> {
        let slot = self.slots.get(usize::from(h.index))?;
        if slot.generation != h.generation {
            return None; // stale handle: the slot was reused
        }
        slot.value.as_ref()
    }

    pub fn remove(&mut self, h: Handle) -> Option<T> {
        let slot = self.slots.get_mut(usize::from(h.index))?;
        if slot.generation != h.generation {
            return None;
        }
        slot.generation = slot.generation.wrapping_add(1); // invalidate old handles
        slot.value.take()
    }
}

fn pool() {
    let mut sessions: Pool<&str, 2> = Pool::new();
    let a = sessions.insert("diag session A").unwrap();
    let b = sessions.insert("diag session B").unwrap();
    println!("full → {:?}", sessions.insert("C"));

    sessions.remove(a);
    let c = sessions.insert("diag session C").unwrap(); // reuses slot 0
    println!("a = {a:?}, c = {c:?}");
    println!("get(c) = {:?}", sessions.get(c));
    println!("get(a) = {:?}  ← stale handle detected", sessions.get(a));
    println!("get(b) = {:?}", sessions.get(b));
}
// ANCHOR_END: pool

fn init_once() {
    // ANCHOR: init
    struct Recorder {
        samples: Vec<u16>,
    }
    impl Recorder {
        fn with_capacity(n: usize) -> Result<Self, std::collections::TryReserveError> {
            let mut samples = Vec::new();
            samples.try_reserve_exact(n)?; // fail at start-up, not mid-drive
            Ok(Recorder { samples })
        }
        fn record(&mut self, s: u16) -> bool {
            if self.samples.len() < self.samples.capacity() {
                self.samples.push(s); // never reallocates
                true
            } else {
                false
            }
        }
    }

    let (recorder, init_allocs) = alloc_counter::count(|| Recorder::with_capacity(1000));
    let mut recorder = recorder.expect("start-up allocation");
    let (stored, steady_allocs) =
        alloc_counter::count(|| (0..1200).filter(|&i| recorder.record(i)).count());
    println!(
        "init: {init_allocs} allocation(s); steady state: {steady_allocs} for {stored} samples"
    );
    assert_eq!(steady_allocs, 0, "the control loop must not allocate");
    // ANCHOR_END: init
}
