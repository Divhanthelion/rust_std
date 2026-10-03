//! Lesson: Atomics & memory ordering.

use std::cell::UnsafeCell;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, AtomicUsize, Ordering};
use std::thread;

use crate::lesson::{Lesson, Question, Section};

pub static LESSON: Lesson = Lesson {
    id: "atomics",
    title: "Atomics & memory ordering",
    summary: "Lock-free primitives: atomic operations, compare-and-swap, the memory-ordering spectrum, release/acquire publication, a spinlock and a lock-free SPSC ring buffer.",
    source: include_str!("atomics.rs"),
    sections: &[
        Section::new(
            "What atomics are",
            r#"
            An **atomic** type (`AtomicBool`, `AtomicU32`, `AtomicUsize`,
            `AtomicPtr`, …) supports operations that happen **indivisibly**.
            No other thread can observe them half-done, and they need no lock.
            All of them work through `&self`, so you can share them with plain
            references, `Arc`, or a `static`.

            They're the building blocks underneath `Mutex`, `Arc`, channels and
            `OnceLock`. In embedded systems they're also how an interrupt
            handler and the main loop share flags safely. Not every target has
            every width: 64-bit atomics are missing on some 32-bit
            microcontrollers.
            "#,
        )
        .demo("basics", basics),
        Section::new(
            "Read-modify-write and compare-and-swap",
            r#"
            Beyond `load` and `store`, atomics offer read-modify-write
            operations that return the **previous** value: `swap`,
            `fetch_add`, `fetch_sub`, `fetch_and`, `fetch_or`, `fetch_xor`,
            `fetch_max` and `fetch_min`.

            The universal one is **compare-and-swap**:
            `compare_exchange(expected, new, …)` stores `new` only if the
            current value equals `expected`, and otherwise reports the actual
            value. Any lock-free algorithm can be built from it.
            `compare_exchange_weak` may fail spuriously, but is cheaper inside
            a retry loop. `try_update` (named `fetch_update` before Rust 1.95)
            packages that loop around a closure.
            "#,
        )
        .demo("rmw", read_modify_write),
        Section::new(
            "The memory-ordering spectrum",
            r#"
            Compilers and CPUs **reorder** memory operations for speed. A single
            thread never notices, but other threads can. Each atomic operation
            takes an `Ordering` that limits the reordering:

            ```text
            Relaxed   atomic, but no ordering with other memory operations.
                      Enough for counters and statistics.
            Release   (on a store) every write before it is visible to whoever
                      Acquires this value
            Acquire   (on a load) every read after it sees what was published
            AcqRel    both, for read-modify-write operations
            SeqCst    Acquire/Release plus a single global order of all
                      SeqCst operations, the easiest to reason about
            ```

            The key pattern is **release/acquire**. A writer prepares data,
            then sets a flag with `Release`. A reader that sees the flag with
            `Acquire` is guaranteed to see the data. Without that pairing,
            the reader could see the flag set and still read stale data, even
            on x86 if the compiler reorders.
            "#,
        )
        .demo("publish", release_acquire),
        Section::new(
            "Relaxed is enough for counters",
            r#"
            When the atomic **is** the data, as with a hit counter, a
            statistics total, or a unique id generator, and no other memory
            depends on it, `Relaxed` is correct and fastest. Every increment
            still counts. What you give up is any ordering relative to other
            variables.
            "#,
        )
        .demo("relaxed", relaxed_counters),
        Section::new(
            "Building a spinlock",
            r#"
            A spinlock is the smallest useful lock. An `AtomicBool` says
            "locked", and the data lives in an `UnsafeCell`, Rust's only
            sanctioned way to mutate through a shared reference. Lock with
            `compare_exchange_weak(false, true, Acquire, Relaxed)` in a loop,
            and unlock with `store(false, Release)`. That pairing is exactly
            what makes writes inside the critical section visible to the next
            owner.

            This needs `unsafe` to promise the compiler that our flag provides
            exclusive access (`unsafe impl Sync`). Lesson 36 covers `unsafe`
            properly. Spinlocks suit very short critical sections, or code with
            no OS to sleep on. Elsewhere, `Mutex` parks waiting threads instead
            of burning CPU.
            "#,
        )
        .demo("spinlock", spinlock),
        Section::new(
            "A lock-free SPSC ring buffer",
            r#"
            A **single-producer, single-consumer** queue is the classic
            lock-free structure. An interrupt handler or a receive thread pushes
            CAN frames while the main loop pops them. This version stays in safe
            Rust by making each slot an `AtomicU32`:

            - the producer writes a slot (Relaxed), then publishes the new
              `tail` with **Release**;
            - the consumer reads `tail` with **Acquire** and then the slot,
              and publishes the new `head` with **Release** so the producer may
              reuse the slot.

            Nothing ever blocks, and the capacity is fixed at compile time with
            a const generic, so the buffer never allocates.
            "#,
        )
        .demo("spsc", spsc),
        Section::new(
            "When not to use atomics",
            r#"
            Atomics are sharp tools. Ordering bugs pass every test on your
            machine, then fail once a week in the field on a different CPU. Use
            this rough guide, from safest to most specialized:

            ```text
            channels / owning thread   simplest to reason about
            Mutex / RwLock             general shared state
            atomic counters, flags     Relaxed / SeqCst single variables
            release/acquire protocols  custom lock-free structures — review carefully
            ```

            If you're unsure, `SeqCst` is the safest ordering. Then measure
            before optimizing. Tools like Miri (in the Rust project) and model
            checkers like the `loom` crate explore interleavings that tests
            rarely hit.
            "#,
        ),
    ],
    quiz: &[
        Question::new(
            "What does `fetch_add` return?",
            &["The new value", "The previous value", "Nothing", "A bool"],
            1,
            "All fetch_* operations return the value before the update. That makes them perfect for unique id generation.",
        ),
        Question::new(
            "A writer fills a buffer, then sets READY. Which orderings make the reader see the full buffer after seeing READY?",
            &[
                "Relaxed store, Relaxed load",
                "Release store, Acquire load",
                "Acquire store, Release load",
                "Any ordering works on x86",
            ],
            1,
            "Release on the flag store publishes earlier writes; Acquire on the flag load makes them visible. x86 hardware is strong, but the compiler may still reorder.",
        ),
        Question::new(
            "When is `Ordering::Relaxed` appropriate?",
            &[
                "Never",
                "When the atomic value itself is the only shared data (e.g., a statistics counter)",
                "For locks",
                "For publishing pointers",
            ],
            1,
            "Relaxed keeps the operation atomic but imposes no ordering with other memory, which is fine when nothing else depends on it.",
        ),
        Question::new(
            "Why does a hand-written spinlock need `unsafe impl Sync`?",
            &[
                "Atomics are unsafe",
                "UnsafeCell is !Sync; we promise the compiler our flag guarantees exclusive access",
                "Spinlocks can't be shared",
                "It doesn't",
            ],
            1,
            "The compiler can't verify our locking protocol, so we assert Sync ourselves and must uphold it.",
        ),
        Question::new(
            "What is the difference between compare_exchange and compare_exchange_weak?",
            &[
                "weak uses Relaxed ordering",
                "weak may fail spuriously even if the value matches, so use it in loops",
                "weak doesn't return the old value",
                "There is none",
            ],
            1,
            "On LL/SC architectures (like ARM) the weak form can fail spuriously but is cheaper inside a retry loop.",
        ),
    ],
    exercises: &[
        "Write a lock-free `fn next_id() -> u64` with a static AtomicU64. Spawn 8 threads that each take 1000 ids, and prove there are no duplicates with a HashSet.",
        "Implement `fn record_max(max: &AtomicU32, value: u32)` with a `compare_exchange_weak` loop. Then replace it with `fetch_max`.",
        "Extend the SPSC ring buffer to report how many pushes failed because the buffer was full (a dropped-frame counter).",
        "Add a `try_lock` method to the spinlock that returns `Option<Guard>` without spinning.",
    ],
};

fn basics() {
    // ANCHOR: basics
    static REQUESTS: AtomicUsize = AtomicUsize::new(0);
    let engine_running = AtomicBool::new(false);
    let before = REQUESTS.load(Ordering::Relaxed); // statics persist between runs

    thread::scope(|s| {
        for _ in 0..4 {
            s.spawn(|| {
                for _ in 0..250 {
                    REQUESTS.fetch_add(1, Ordering::Relaxed);
                }
            });
        }
        s.spawn(|| engine_running.store(true, Ordering::Release));
    });

    let added = REQUESTS.load(Ordering::Relaxed) - before;
    println!("requests added: {added} (4 threads × 250, none lost)");
    println!(
        "engine running = {}",
        engine_running.load(Ordering::Acquire)
    );
    println!(
        "AtomicU64 is {} bytes, like u64",
        std::mem::size_of::<AtomicU64>()
    );
    // ANCHOR_END: basics
}

fn read_modify_write() {
    // ANCHOR: rmw
    let a = AtomicU32::new(10);
    println!(
        "fetch_add(5) returned {}, now {}",
        a.fetch_add(5, Ordering::SeqCst),
        a.load(Ordering::SeqCst)
    );
    println!("swap(100) returned {}", a.swap(100, Ordering::SeqCst));
    println!(
        "fetch_max(42) returned {}, now {}",
        a.fetch_max(42, Ordering::SeqCst),
        a.load(Ordering::SeqCst)
    );

    let flags = AtomicU32::new(0b0000);
    flags.fetch_or(0b0101, Ordering::SeqCst); //  set bits
    flags.fetch_and(!0b0001, Ordering::SeqCst); // clear a bit
    println!("flags = {:04b}", flags.load(Ordering::SeqCst));

    // compare_exchange: only one "claimant" can win.
    let owner = AtomicU32::new(0);
    for id in [7, 9] {
        match owner.compare_exchange(0, id, Ordering::AcqRel, Ordering::Acquire) {
            Ok(_) => println!("thread {id} claimed the resource"),
            Err(current) => println!("thread {id} lost: owned by {current}"),
        }
    }

    // try_update: a CAS loop around a closure (here: saturating decrement)
    let credits = AtomicU32::new(1);
    for _ in 0..2 {
        let r = credits.try_update(Ordering::AcqRel, Ordering::Acquire, |c| c.checked_sub(1));
        println!("take credit → {r:?}");
    }
    // ANCHOR_END: rmw
}

fn release_acquire() {
    // ANCHOR: publish
    // Shared between threads: the payload and a "ready" flag.
    let payload = AtomicU64::new(0);
    let ready = AtomicBool::new(false);

    thread::scope(|s| {
        s.spawn(|| {
            payload.store(0xCAFE_F00D, Ordering::Relaxed); // 1. write the data
            ready.store(true, Ordering::Release); //          2. publish it
        });
        s.spawn(|| {
            while !ready.load(Ordering::Acquire) {
                // 3. wait until published
                std::hint::spin_loop();
            }
            // 4. Acquire synchronized with Release: the payload write is visible.
            println!("reader sees {:#X}", payload.load(Ordering::Relaxed));
        });
    });
    // ANCHOR_END: publish
}

fn relaxed_counters() {
    // ANCHOR: relaxed
    struct Stats {
        frames: AtomicU64,
        errors: AtomicU64,
    }
    let stats = Stats {
        frames: AtomicU64::new(0),
        errors: AtomicU64::new(0),
    };

    thread::scope(|s| {
        for worker in 0..4u64 {
            let stats = &stats;
            s.spawn(move || {
                for i in 0..10_000u64 {
                    stats.frames.fetch_add(1, Ordering::Relaxed);
                    if (i + worker) % 1000 == 0 {
                        stats.errors.fetch_add(1, Ordering::Relaxed);
                    }
                }
            });
        }
    });
    println!(
        "frames {} errors {}",
        stats.frames.load(Ordering::Relaxed),
        stats.errors.load(Ordering::Relaxed)
    );
    // ANCHOR_END: relaxed
}

// ANCHOR: spinlock
pub struct SpinLock<T> {
    locked: AtomicBool,
    value: UnsafeCell<T>,
}

// SAFETY: access to `value` is serialized by `locked` (see `lock`), so
// sharing a SpinLock between threads is sound whenever T can be sent.
unsafe impl<T: Send> Sync for SpinLock<T> {}

pub struct Guard<'a, T> {
    lock: &'a SpinLock<T>,
}

impl<T> SpinLock<T> {
    pub const fn new(value: T) -> Self {
        SpinLock {
            locked: AtomicBool::new(false),
            value: UnsafeCell::new(value),
        }
    }

    pub fn lock(&self) -> Guard<'_, T> {
        while self
            .locked
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            std::hint::spin_loop(); // tell the CPU we're busy-waiting
        }
        Guard { lock: self }
    }
}

impl<T> Deref for Guard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        // SAFETY: we hold the lock, so no one else can access the value.
        unsafe { &*self.lock.value.get() }
    }
}

impl<T> DerefMut for Guard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: we hold the lock exclusively.
        unsafe { &mut *self.lock.value.get() }
    }
}

impl<T> Drop for Guard<'_, T> {
    fn drop(&mut self) {
        self.lock.locked.store(false, Ordering::Release); // publish our writes
    }
}

fn spinlock() {
    let log = SpinLock::new(Vec::new());
    thread::scope(|s| {
        for id in 0..4 {
            let log = &log;
            s.spawn(move || {
                for n in 0..100 {
                    log.lock().push(id * 1000 + n);
                }
            });
        }
    });
    let entries = log.lock();
    println!("{} entries, none lost", entries.len());
}
// ANCHOR_END: spinlock

// ANCHOR: spsc
/// A lock-free single-producer/single-consumer queue of `u32`s.
/// One slot is kept empty to tell "full" from "empty".
pub struct SpscRing<const N: usize> {
    slots: [AtomicU32; N],
    head: AtomicUsize, // next slot to read  (written by the consumer)
    tail: AtomicUsize, // next slot to write (written by the producer)
}

impl<const N: usize> SpscRing<N> {
    pub fn new() -> Self {
        SpscRing {
            slots: std::array::from_fn(|_| AtomicU32::new(0)),
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
        }
    }

    /// Producer side. Returns the value back if the ring is full.
    pub fn push(&self, value: u32) -> Result<(), u32> {
        let tail = self.tail.load(Ordering::Relaxed); // only we write tail
        let next = (tail + 1) % N;
        if next == self.head.load(Ordering::Acquire) {
            return Err(value); // full
        }
        self.slots[tail].store(value, Ordering::Relaxed);
        self.tail.store(next, Ordering::Release); // publish the slot
        Ok(())
    }

    /// Consumer side.
    pub fn pop(&self) -> Option<u32> {
        let head = self.head.load(Ordering::Relaxed); // only we write head
        if head == self.tail.load(Ordering::Acquire) {
            return None; // empty
        }
        let value = self.slots[head].load(Ordering::Relaxed);
        self.head.store((head + 1) % N, Ordering::Release); // free the slot
        Some(value)
    }
}

fn spsc() {
    let ring = Arc::new(SpscRing::<8>::new());
    let producer_ring = Arc::clone(&ring);

    let producer = thread::spawn(move || {
        for frame_id in 1..=1000u32 {
            while producer_ring.push(frame_id).is_err() {
                std::hint::spin_loop(); // full: wait (a real ISR would count and drop)
            }
        }
    });

    let mut received = 0u32;
    let mut last = 0u32;
    let mut in_order = true;
    while received < 1000 {
        if let Some(id) = ring.pop() {
            in_order &= id == last + 1;
            last = id;
            received += 1;
        } else {
            std::hint::spin_loop();
        }
    }
    producer.join().unwrap();
    println!("received {received} frames through an 8-slot ring, in order: {in_order}");
}
// ANCHOR_END: spsc
