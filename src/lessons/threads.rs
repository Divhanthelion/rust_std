//! Lesson: Threads & synchronization.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::mpsc;
use std::sync::{Arc, Barrier, Condvar, LazyLock, Mutex, OnceLock, RwLock};
use std::thread;
use std::time::Duration;

use crate::lesson::{Lesson, Question, Section};

pub static LESSON: Lesson = Lesson {
    id: "threads",
    title: "Threads & synchronization",
    summary: "Spawning and scoped threads, Send and Sync, channels, Arc<Mutex>, poisoning, RwLock, Condvar, Barrier, OnceLock/LazyLock, thread-locals and deadlock avoidance.",
    source: include_str!("threads.rs"),
    sections: &[
        Section::new(
            "Spawning threads",
            r#"
            `thread::spawn(closure)` starts an OS thread and returns a
            `JoinHandle`. `join()` waits for the thread and returns its result,
            as a `Result` because the thread may have panicked. The closure
            must be `'static` (Lesson 9), so it **moves** in the data it needs.

            Threads run concurrently, and the order of their output varies from
            run to run. Joining in a fixed order and collecting results makes
            output deterministic. That's why the demos below print after
            joining.
            "#,
        )
        .demo("spawn", spawn),
        Section::new(
            "Scoped threads: borrowing across threads",
            r#"
            `thread::scope(|s| { s.spawn(…); … })` guarantees that every thread
            spawned inside it is joined before `scope` returns. Because of that
            guarantee, scoped threads can **borrow** local variables, with no
            `Arc` and no `'static` requirement. It's the cleanest way to
            parallelize work over a slice: split it into chunks, process each
            on a thread, and combine.
            "#,
        )
        .demo("scoped", scoped),
        Section::new(
            "Send and Sync",
            r#"
            Two auto traits make thread safety a compile-time property:

            - `T: Send` means a `T` can be **moved** to another thread.
            - `T: Sync` means a `&T` can be **shared** between threads, which is
              the same as saying `&T: Send`.

            Most types are both. The exceptions are deliberate. `Rc<T>` is
            neither, because its count isn't atomic. `Cell` and `RefCell` are
            `Send` but not `Sync`. Raw pointers are neither. `thread::spawn`
            requires `Send`, so this is rejected at compile time:

            ```compile_fail,E0277
            use std::rc::Rc;
            let shared = Rc::new(5);
            std::thread::spawn(move || println!("{shared}"));
            // error: `Rc<i32>` cannot be sent between threads safely
            ```

            Swap `Rc` for `Arc`, the **atomically** reference-counted pointer,
            and it compiles. Rust calls this "fearless concurrency": data races
            are type errors.
            "#,
        ),
        Section::new(
            "Message passing with channels",
            r#"
            "Don't communicate by sharing memory; share memory by
            communicating." `mpsc::channel()` returns a `(Sender, Receiver)`
            pair: **m**ulti-**p**roducer, **s**ingle-**c**onsumer. Clone the
            sender for each producer. Sending **moves** the value to the
            receiver, so it can't be touched afterwards.

            Iterating over the receiver ends when **all** senders are dropped.
            Forgetting to drop the original sender is the classic "my program
            hangs" bug. Use `recv_timeout` when waiting forever is unacceptable.
            "#,
        )
        .demo("channels", channels),
        Section::new(
            "Bounded channels and backpressure",
            r#"
            `mpsc::channel` is unbounded: a fast producer can fill memory.
            `mpsc::sync_channel(n)` holds at most `n` messages, so `send` blocks
            when the channel is full. That's **backpressure**, and it's
            essential in pipelines where the producer, such as a sensor, can
            outrun the consumer, such as a logger. `try_send` doesn't block,
            and returns `Full` so you can drop or count the message instead.
            With `n = 0` the channel is a rendezvous: each `send` waits for a
            `recv`.
            "#,
        )
        .demo("bounded", bounded),
        Section::new(
            "Shared state: Arc<Mutex<T>>",
            r#"
            When threads must share mutable state, wrap it in a `Mutex` and
            share the mutex with `Arc`. `lock()` blocks until the mutex is free
            and returns a **guard**. The guard derefs to the data and unlocks
            when dropped (RAII). You can't forget to unlock, and you can't touch
            the data without locking.

            Keep critical sections short. Copy out what you need, drop the
            guard, then do slow work. A guard held across a long operation
            serializes your threads.
            "#,
        )
        .demo("mutex", mutex),
        Section::new(
            "Poisoning",
            r#"
            If a thread **panics while holding a lock**, the data might be left
            half-updated. Rust marks the mutex as **poisoned**, and later
            `lock()` calls return `Err(PoisonError)`. That's why you see
            `.lock().unwrap()`: in most programs, propagating the panic is the
            right call.

            If you can verify or repair the data, `PoisonError::into_inner()`
            recovers the guard anyway. In a vehicle, a poisoned lock is a
            signal to enter a safe state, not to carry on silently.
            "#,
        )
        .demo("poison", poisoning),
        Section::new(
            "RwLock: many readers or one writer",
            r#"
            `RwLock<T>` allows any number of simultaneous `read()` guards **or**
            one `write()` guard. That's the borrow rule again, enforced at run
            time across threads. It pays off for data that is read constantly
            and written rarely, such as configuration or calibration tables.
            Under heavy write contention, a plain `Mutex` is often faster.
            "#,
        )
        .demo("rwlock", rwlock),
        Section::new(
            "Condvar: waiting for a condition",
            r#"
            A `Condvar` lets a thread **sleep until another thread signals**
            that some state changed, without burning CPU in a loop. It's
            always used with a `Mutex` holding that state. Waits can wake
            spuriously, so always re-check the condition. `wait_while`
            does that loop for you. `notify_one` wakes one waiter and
            `notify_all` wakes all of them.
            "#,
        )
        .demo("condvar", condvar),
        Section::new(
            "Barrier, OnceLock and LazyLock",
            r#"
            - `Barrier::new(n)` makes `n` threads wait until all have arrived,
              which keeps phases of a computation in step.
            - `OnceLock<T>` is a thread-safe write-once cell: the first
              `get_or_init` runs the initializer, and racing callers wait for it.
            - `LazyLock<T>` (Rust 1.80+) bundles the initializer with the cell.
              It's the standard way to have a lazily-built global, with no
              crate needed.
            "#,
        )
        .demo("once", barrier_once),
        Section::new(
            "Thread-locals and thread utilities",
            r#"
            `thread_local!` declares a static with **one copy per thread**,
            accessed through `.with(|v| …)`. It's useful for per-thread
            buffers, counters or RNG state, without any locking.

            Other tools in `std::thread`:

            - `Builder` sets names and stack sizes;
            - `available_parallelism()` reports how many threads can usefully
              run;
            - `sleep` pauses the current thread;
            - `park` and `unpark` are the low-level primitive that channels and
              async executors (Lesson 30) are built on.
            "#,
        )
        .demo("tls", thread_locals),
        Section::new(
            "Deadlocks and how to avoid them",
            r#"
            Rust prevents data races, but **not deadlocks**. Deadlock happens
            when thread A holds lock 1 and waits for lock 2 while thread B holds
            2 and waits for 1. Standard defences:

            - Always acquire locks in a **global order**, for example by
              address or by a documented rank.
            - Hold locks briefly, and never call unknown code (callbacks) while
              holding one.
            - Prefer **message passing**: one thread owns the state and others
              send it requests.
            - Use `try_lock` or timeouts where blocking forever is unacceptable.

            Also: `std::sync::Mutex` isn't reentrant. Locking it twice from the
            same thread deadlocks or panics.
            "#,
        ),
    ],
    quiz: &[
        Question::new(
            "Why can scoped threads borrow local variables while `thread::spawn` threads cannot?",
            &[
                "Scoped threads copy the data",
                "The scope guarantees all its threads are joined before the borrowed data goes out of scope",
                "Scoped threads run sequentially",
                "They use unsafe internally, unchecked",
            ],
            1,
            "Because `thread::scope` joins every thread before returning, borrows can't outlive their data.",
        ),
        Question::new(
            "Which is `Send` but not `Sync`?",
            &["i32", "Arc<i32>", "RefCell<i32>", "Rc<i32>"],
            2,
            "A RefCell can be moved to another thread, but sharing &RefCell across threads would race on its borrow counter. Rc is neither.",
        ),
        Question::new(
            "A `for msg in rx` loop never ends. What's the most likely cause?",
            &[
                "The channel is full",
                "A Sender (often the original one) was never dropped",
                "Receivers can't be iterated",
                "Messages must be Copy",
            ],
            1,
            "Receiver iteration ends only when every Sender is gone. Drop the original after cloning it to the workers.",
        ),
        Question::new(
            "What does it mean when `mutex.lock()` returns `Err(PoisonError)`?",
            &[
                "The lock is held by another thread",
                "A thread panicked while holding this lock, so the data may be inconsistent",
                "The mutex was dropped",
                "Deadlock was detected",
            ],
            1,
            "Poisoning flags possible broken invariants. You can still recover the data with `into_inner()` if you can validate it.",
        ),
        Question::new(
            "Which primitive gives backpressure between a fast producer and a slow consumer?",
            &[
                "mpsc::channel()",
                "mpsc::sync_channel(n)",
                "Arc<Mutex<Vec<T>>>",
                "thread_local!",
            ],
            1,
            "A bounded channel blocks (or rejects, with try_send) the producer when full.",
        ),
    ],
    exercises: &[
        "Use `thread::scope` to count words in four chunks of a large string in parallel, then sum the results.",
        "Build a three-stage pipeline (parse → filter → log) connected by `sync_channel(4)`. Make the logger slow with `sleep` and observe the backpressure.",
        "Create two mutexes and two threads that lock them in opposite orders (with small sleeps) to produce a deadlock. Fix it with a lock order.",
        "Implement a one-slot mailbox with `Mutex<Option<T>>` and a `Condvar`: `put` waits while full, `take` waits while empty.",
    ],
};

fn spawn() {
    // ANCHOR: spawn
    let handles: Vec<thread::JoinHandle<(usize, u64)>> = (0..4)
        .map(|id| {
            thread::spawn(move || {
                // each thread gets its own `id` (moved in)
                let work: u64 = (1..=100_000u64).filter(|n| n % (id as u64 + 2) == 0).sum();
                (id, work)
            })
        })
        .collect();

    for handle in handles {
        let (id, result) = handle.join().expect("worker panicked");
        println!("worker {id}: {result}");
    }

    let name = thread::current().name().map(str::to_owned);
    println!("main thread name: {name:?}");
    // ANCHOR_END: spawn
}

fn scoped() {
    // ANCHOR: scoped
    let samples: Vec<u32> = (1..=1_000).collect();
    let mut maxima = [0u32; 4];

    thread::scope(|s| {
        let chunk_len = samples.len().div_ceil(maxima.len());
        // Borrow `samples` immutably and each slot of `maxima` mutably:
        for (chunk, slot) in samples.chunks(chunk_len).zip(maxima.iter_mut()) {
            s.spawn(move || {
                *slot = chunk.iter().copied().max().unwrap_or(0);
            });
        }
    }); // every thread is joined here

    println!("chunk maxima {maxima:?}");
    println!("overall max {}", maxima.iter().max().unwrap());
    // ANCHOR_END: scoped
}

fn channels() {
    // ANCHOR: channels
    let (tx, rx) = mpsc::channel::<(String, f64)>();

    let mut producers = Vec::new();
    for (sensor, base) in [("oil", 90.0), ("coolant", 85.0)] {
        let tx = tx.clone(); // one Sender per producer
        producers.push(thread::spawn(move || {
            for i in 0..3 {
                tx.send((sensor.to_string(), base + i as f64)).unwrap();
            }
        }));
    }
    drop(tx); // drop the original, or the loop below never ends

    let mut received: Vec<(String, f64)> = rx.iter().collect(); // until all senders are gone
    received.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.total_cmp(&b.1)));
    for (sensor, value) in &received {
        println!("{sensor:<8} {value}");
    }
    for p in producers {
        p.join().unwrap();
    }

    let (_tx2, rx2) = mpsc::channel::<u8>();
    println!(
        "recv_timeout on a quiet channel: {:?}",
        rx2.recv_timeout(Duration::from_millis(10))
    );
    // ANCHOR_END: channels
}

fn bounded() {
    // ANCHOR: bounded
    let (tx, rx) = mpsc::sync_channel::<u32>(2); // capacity 2

    tx.send(1).unwrap();
    tx.send(2).unwrap();
    match tx.try_send(3) {
        Err(mpsc::TrySendError::Full(v)) => println!("channel full, {v} not sent (would block)"),
        other => println!("unexpected: {other:?}"),
    }

    let consumer = thread::spawn(move || {
        let mut got = Vec::new();
        for v in rx {
            thread::sleep(Duration::from_millis(2)); // a slow consumer
            got.push(v);
        }
        got
    });
    for v in 3..=6 {
        tx.send(v).unwrap(); // blocks while the channel holds 2 items
    }
    drop(tx);
    println!("consumer received {:?}", consumer.join().unwrap());
    // ANCHOR_END: bounded
}

fn mutex() {
    // ANCHOR: mutex
    let odometer = Arc::new(Mutex::new(0u64));
    let mut handles = Vec::new();

    for _ in 0..8 {
        let odo = Arc::clone(&odometer);
        handles.push(thread::spawn(move || {
            for _ in 0..1_000 {
                let mut km = odo.lock().unwrap(); // blocks until available
                *km += 1;
            } // guard dropped → unlocked, every iteration
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
    println!(
        "total: {} (always 8000: no lost updates)",
        *odometer.lock().unwrap()
    );
    println!("Arc strong count now {}", Arc::strong_count(&odometer));
    // ANCHOR_END: mutex
}

fn poisoning() {
    // ANCHOR: poison
    use std::panic;

    let state = Arc::new(Mutex::new(vec![1, 2, 3]));
    let previous = panic::take_hook();
    panic::set_hook(Box::new(|_| {})); // keep the demo's output quiet

    let s = Arc::clone(&state);
    let result = thread::spawn(move || {
        let mut data = s.lock().unwrap();
        data.push(4);
        panic!("crashed mid-update"); // the guard is dropped during unwinding
    })
    .join();
    panic::set_hook(previous);

    println!("worker result is_err: {}", result.is_err());
    match state.lock() {
        Ok(data) => println!("not poisoned: {data:?}"),
        Err(poisoned) => {
            let data = poisoned.into_inner(); // recover after checking the data
            println!("poisoned! data was {data:?}; validate before trusting it");
        }
    }
    state.clear_poison(); // Rust 1.77+: mark it healthy again after repair
    println!("is_poisoned now: {}", state.is_poisoned());
    // ANCHOR_END: poison
}

fn rwlock() {
    // ANCHOR: rwlock
    let calibration = Arc::new(RwLock::new(HashMap::from([
        ("throttle", 1.00),
        ("brake", 0.98),
    ])));

    thread::scope(|s| {
        for name in ["throttle", "brake", "throttle"] {
            let cal = &calibration;
            s.spawn(move || {
                let table = cal.read().unwrap(); // many readers at once
                let _ = table.get(name);
            });
        }
    });

    {
        let mut table = calibration.write().unwrap(); // exclusive
        table.insert("brake", 1.02);
    }
    let table = calibration.read().unwrap();
    let mut entries: Vec<_> = table.iter().collect();
    entries.sort_by(|a, b| a.0.cmp(b.0));
    println!("calibration now {entries:?}");
    // ANCHOR_END: rwlock
}

fn condvar() {
    // ANCHOR: condvar
    let pair = Arc::new((Mutex::new(false), Condvar::new()));

    let waiter = {
        let pair = Arc::clone(&pair);
        thread::spawn(move || {
            let (ready, cvar) = &*pair;
            let guard = cvar
                .wait_while(ready.lock().unwrap(), |is_ready| !*is_ready) // handles spurious wakeups
                .unwrap();
            format!("waiter woke up, ready = {}", *guard)
        })
    };

    thread::sleep(Duration::from_millis(5)); // pretend to initialize hardware
    {
        let (ready, cvar) = &*pair;
        *ready.lock().unwrap() = true;
        cvar.notify_one();
    }
    println!("{}", waiter.join().unwrap());
    // ANCHOR_END: condvar
}

// ANCHOR: once
static VEHICLE_ID: OnceLock<String> = OnceLock::new();
static DTC_TABLE: LazyLock<HashMap<&'static str, &'static str>> = LazyLock::new(|| {
    HashMap::from([
        ("P0301", "cylinder 1 misfire"),
        ("P0420", "catalyst efficiency low"),
    ])
});

fn barrier_once() {
    let barrier = Barrier::new(3);
    let log = Mutex::new(Vec::new());

    thread::scope(|s| {
        for id in 0..3 {
            let (barrier, log) = (&barrier, &log);
            s.spawn(move || {
                log.lock().unwrap().push(format!("phase 1: thread {id}"));
                barrier.wait(); // nobody starts phase 2 until all finish phase 1
                log.lock().unwrap().push(format!("phase 2: thread {id}"));
            });
        }
    });
    let log = log.into_inner().unwrap();
    let phase1_first = log.iter().take(3).all(|l| l.starts_with("phase 1"));
    println!("all phase-1 entries came first: {phase1_first}");

    // Many threads race to initialize; exactly one closure runs.
    thread::scope(|s| {
        for i in 0..4 {
            s.spawn(move || VEHICLE_ID.get_or_init(|| format!("VIN-from-thread-{i}")));
        }
    });
    println!(
        "VEHICLE_ID = {} (whichever thread won the race)",
        VEHICLE_ID.get().unwrap()
    );
    println!("P0420 = {:?}", DTC_TABLE.get("P0420"));
}
// ANCHOR_END: once

fn thread_locals() {
    // ANCHOR: tls
    thread_local! {
        static SCRATCH: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
    }

    let lens: Vec<usize> = thread::scope(|s| {
        let handles: Vec<_> = (1..=3)
            .map(|n| {
                s.spawn(move || {
                    SCRATCH.with(|buf| {
                        buf.borrow_mut().extend(std::iter::repeat_n(0u8, n * 10));
                        buf.borrow().len() // each thread has its own buffer
                    })
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    println!("per-thread buffer lengths {lens:?}");
    println!(
        "main thread's buffer is still empty: {}",
        SCRATCH.with(|b| b.borrow().is_empty())
    );

    let named = thread::Builder::new()
        .name("can-rx".into())
        .stack_size(64 * 1024)
        .spawn(|| thread::current().name().unwrap_or("?").to_string())
        .unwrap();
    println!("spawned a thread named {:?}", named.join().unwrap());
    let cores = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);
    println!("available parallelism ≥ 1: {}", cores >= 1);
    // ANCHOR_END: tls
}
