//! Lesson: Async from scratch.

use std::collections::VecDeque;
use std::future::Future;
use std::pin::{Pin, pin};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Wake, Waker};
use std::thread::{self, Thread};
use std::time::{Duration, Instant};

use crate::lesson::{Lesson, Question, Section};

pub static LESSON: Lesson = Lesson {
    id: "async",
    title: "Async from scratch",
    summary: "Futures, polling, wakers and Pin — then a block_on executor, a timer, a multi-task executor and join, all with std only.",
    source: include_str!("async_rust.rs"),
    sections: &[
        Section::new(
            "What async is (and isn't)",
            r#"
            `async` lets one thread juggle many tasks that spend most of their
            time **waiting**, for sockets, timers or CAN frames. Instead of
            blocking, a task returns control so another can run. Concurrency
            models form a spectrum:

            ```text
            OS threads     preemptive; each has a stack (KBs–MBs); easy to reason about
            async tasks    cooperative; each is a state machine (bytes); switch only at .await
            ```

            Rust's standard library provides the **vocabulary**: the `Future`
            trait, `Context`, `Waker`, `Pin` and the `async`/`.await` syntax.
            It deliberately ships **no runtime**. Runtimes (Tokio for servers,
            Embassy for microcontrollers) are libraries. This lesson builds a
            small one from scratch, which is the best way to understand what
            those libraries do.
            "#,
        ),
        Section::new(
            "The Future trait",
            r#"
            ```rust
            # use std::pin::Pin; use std::task::{Context, Poll};
            trait MyFuture {
                type Output;
                fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
            }
            ```

            An executor **polls** a future. The future either finishes,
            returning `Poll::Ready(value)`, or returns `Poll::Pending`. Before
            returning `Pending`, it must arrange for `cx.waker()` to be woken
            when progress is possible. That's the contract that keeps an
            executor from busy-polling.

            Below, a hand-written future is polled by hand using
            `Waker::noop()` (Rust 1.85+), a waker that does nothing. It's fine
            when we poll in a loop ourselves.
            "#,
        )
        .demo("future", future_trait),
        Section::new(
            "async fn and .await are state machines",
            r#"
            `async fn f() -> T` is sugar for `fn f() -> impl Future<Output =
            T>`. The compiler turns the body into a state machine. Each
            `.await` is a point where it may return `Pending` and later resume
            with all its local variables intact.

            Futures are **lazy**. Calling an `async fn` runs **nothing**. Only
            polling makes progress. Forgetting to `.await` (or spawn) a future
            is a classic bug, and the compiler warns about it
            (`unused_must_use`).
            "#,
        )
        .demo("lazy", laziness),
        Section::new(
            "A real executor: block_on",
            r#"
            A minimal executor needs a waker that can wake the executor's
            thread. The `Wake` trait turns any `Arc<impl Wake>` into a `Waker`
            safely. Our waker calls `thread.unpark()`.

            `block_on` pins the future, then loops: poll, and if `Pending`,
            `thread::park()` until some waker unparks us. That's about 20
            lines. Production executors add task queues, I/O event loops and
            timers, but the heart is this loop.
            "#,
        )
        .demo("block_on", block_on_demo),
        Section::new(
            "A timer future",
            r#"
            A `Sleep` future must not block the thread. On first poll it
            records the **latest** waker in shared state and starts a helper
            thread that sleeps, marks completion, and wakes the stored waker.
            Later polls just check the completion flag and refresh the waker.
            Real runtimes use one timer wheel for thousands of sleeps instead
            of a thread each, but the protocol is the same: **store the waker,
            wake it when ready**.
            "#,
        )
        .demo("timer", timer_demo),
        Section::new(
            "Many tasks on one thread",
            r#"
            A task executor keeps a **queue of runnable tasks**. Each task owns
            a boxed future, and its waker simply re-queues the task. The run
            loop pops a task and polls it. If the task returns `Pending`, it
            goes back on the queue the next time it is woken.

            With a cooperative `yield_now()` (a future that wakes itself and
            returns `Pending` once), three tasks interleave on one thread. The
            queue is FIFO, so the interleaving is deterministic.
            "#,
        )
        .demo("executor", executor_demo),
        Section::new(
            "Concurrency inside one task: join",
            r#"
            `.await` runs futures **one after another**. To wait on several at
            once, poll them **together**: a `join` future polls each child
            that isn't finished yet, and is ready when all are. Two sleeps of
            30 ms joined take about 30 ms, not 60. `select` is the variation
            that finishes when the **first** child does, which is how timeouts
            are built.
            "#,
        )
        .demo("join", join_demo),
        Section::new(
            "Why Pin, concretely",
            r#"
            An `async` block that holds a reference to its own local across an
            `.await` becomes a struct that points **into itself**. Moving it
            after the first poll would leave that pointer dangling. That's why
            `poll` takes `Pin<&mut Self>`. Once polled, a future stays put.

            Async blocks are therefore `!Unpin`, and this fails to compile:

            ```compile_fail,E0277
            fn needs_unpin<T: Unpin>(_: T) {}
            needs_unpin(async {
                let x = 5;
                let r = &x;
                std::future::ready(()).await;
                println!("{r}");
            });
            ```

            Pin them with `pin!(fut)` (stack) or `Box::pin(fut)` (heap) before
            polling, exactly as our executors do.
            "#,
        ),
        Section::new(
            "Async closures, async traits, and what's next",
            r#"
            Recent Rust filled in the remaining gaps:

            - `async fn` in traits (1.75), with each implementor returning its
              own future type;
            - async closures, `async |x| …`, with the `AsyncFn*` traits (1.85).

            Things to know before using a real runtime:

            - **Dropping a future cancels it** at its last `.await`, so design
              for cancellation.
            - **Never block** inside async code: no `thread::sleep` and no
              blocking I/O. It stalls every task on that thread.
            - Spawned tasks on multi-threaded runtimes need `Send` futures.

            In embedded automotive code, Embassy brings the same model to
            microcontrollers. Interrupts are the wakers, and there's no OS
            underneath.
            "#,
        )
        .demo("closures", async_closures),
    ],
    quiz: &[
        Question::new(
            "What happens when you call an `async fn` but never `.await` or poll the result?",
            &[
                "It runs in the background",
                "Nothing — futures are lazy",
                "It runs synchronously",
                "It panics",
            ],
            1,
            "Calling an async fn only constructs the state machine. Polling drives it.",
        ),
        Question::new(
            "Before returning `Poll::Pending`, what must a future do?",
            &[
                "Sleep",
                "Arrange for the waker from the Context to be called when it can make progress",
                "Return an error",
                "Spawn a thread",
            ],
            1,
            "That's the waker contract. Without it, the executor would never know to poll again (or would have to busy-poll).",
        ),
        Question::new(
            "Why does `Future::poll` take `self: Pin<&mut Self>`?",
            &[
                "For performance",
                "Async state machines may contain self-references, so they must not move after polling starts",
                "To make futures Send",
                "Pin makes them thread-safe",
            ],
            1,
            "Pinning guarantees the future stays at a stable address, so internal borrows across .await remain valid.",
        ),
        Question::new(
            "Two futures each sleep 30 ms. Roughly how long do `a.await; b.await;` and `join(a, b).await` take?",
            &[
                "30 ms and 30 ms",
                "60 ms and 30 ms",
                "60 ms and 60 ms",
                "30 ms and 60 ms",
            ],
            1,
            "Sequential awaits add up; join polls both concurrently.",
        ),
        Question::new(
            "What does the standard library NOT provide for async?",
            &[
                "The Future trait",
                "Waker and Context",
                "A runtime/executor",
                "Pin",
            ],
            2,
            "std defines the interfaces; executors like Tokio or Embassy are libraries.",
        ),
    ],
    exercises: &[
        "Add a `select2` future that completes with whichever of two futures finishes first, and use it to build `timeout(fut, Duration)`.",
        "Give the multi-task executor a `spawn` that works from inside a running task (hint: clone the sender into the task).",
        "Write an async channel: `send` pushes into a `VecDeque` and wakes a stored receiver waker; `recv` returns Pending while empty.",
        "Measure how many 1 ms `Sleep` futures you can join on one thread versus spawning one OS thread per sleep.",
    ],
};

fn future_trait() {
    // ANCHOR: future
    /// Needs to be polled `remaining + 1` times before it's ready.
    struct Countdown {
        remaining: u32,
    }

    impl Future for Countdown {
        type Output = &'static str;
        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            if self.remaining == 0 {
                Poll::Ready("liftoff")
            } else {
                self.remaining -= 1;
                cx.waker().wake_by_ref(); // "poll me again soon"
                Poll::Pending
            }
        }
    }

    let mut cx = Context::from_waker(Waker::noop());
    let mut fut = Countdown { remaining: 3 };
    let mut fut = Pin::new(&mut fut); // Countdown is Unpin, so Pin::new is fine
    let mut polls = 0;
    loop {
        polls += 1;
        match fut.as_mut().poll(&mut cx) {
            Poll::Pending => println!("poll {polls}: Pending"),
            Poll::Ready(msg) => {
                println!("poll {polls}: Ready({msg:?})");
                break;
            }
        }
    }
    // ANCHOR_END: future
}

fn laziness() {
    // ANCHOR: lazy
    async fn read_sensor(log: &Mutex<Vec<&'static str>>) -> u32 {
        log.lock().unwrap().push("read_sensor body ran");
        42
    }

    let log = Mutex::new(Vec::new());
    let fut = read_sensor(&log); // nothing runs yet
    println!("after calling: {:?}", log.lock().unwrap());

    let mut fut = pin!(fut);
    let mut cx = Context::from_waker(Waker::noop());
    let result = fut.as_mut().poll(&mut cx); // now the body runs
    println!("after polling: {:?} → {result:?}", log.lock().unwrap());
    // ANCHOR_END: lazy
}

// ANCHOR: block_on
/// Wakes the executor by unparking its thread.
struct ThreadWaker(Thread);

impl Wake for ThreadWaker {
    fn wake(self: Arc<Self>) {
        self.0.unpark();
    }
}

/// Runs a future to completion on the current thread.
pub fn block_on<F: Future>(fut: F) -> F::Output {
    let mut fut = pin!(fut);
    let waker = Waker::from(Arc::new(ThreadWaker(thread::current())));
    let mut cx = Context::from_waker(&waker);
    loop {
        match fut.as_mut().poll(&mut cx) {
            Poll::Ready(value) => return value,
            Poll::Pending => thread::park(), // sleep until a waker unparks us
        }
    }
}

fn block_on_demo() {
    async fn add(a: u32, b: u32) -> u32 {
        a + b
    }
    async fn compute() -> u32 {
        let x = add(1, 2).await; // .await = poll until Ready
        let y = add(x, 10).await;
        y * 2
    }
    println!("block_on(compute()) = {}", block_on(compute()));
}
// ANCHOR_END: block_on

// ANCHOR: timer
#[derive(Default)]
struct SleepState {
    done: bool,
    waker: Option<Waker>,
}

pub struct Sleep {
    duration: Duration,
    state: Option<Arc<Mutex<SleepState>>>, // created on first poll
}

pub fn sleep(duration: Duration) -> Sleep {
    Sleep {
        duration,
        state: None,
    }
}

impl Future for Sleep {
    type Output = ();
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        let duration = self.duration;
        let state = self.state.get_or_insert_with(|| {
            let state = Arc::new(Mutex::new(SleepState::default()));
            let shared = Arc::clone(&state);
            thread::spawn(move || {
                thread::sleep(duration);
                let mut s = shared.lock().unwrap();
                s.done = true;
                if let Some(w) = s.waker.take() {
                    w.wake(); // tell the executor to poll us again
                }
            });
            state
        });
        let mut s = state.lock().unwrap();
        if s.done {
            Poll::Ready(())
        } else {
            s.waker = Some(cx.waker().clone()); // always store the latest waker
            Poll::Pending
        }
    }
}

fn timer_demo() {
    let start = Instant::now();
    block_on(async {
        println!("sleeping 20 ms (the executor parks its thread; no busy-waiting)");
        sleep(Duration::from_millis(20)).await;
    });
    println!(
        "woke after ≥ 20 ms: {}",
        start.elapsed() >= Duration::from_millis(20)
    );
}
// ANCHOR_END: timer

// ANCHOR: executor
type BoxedTask = Pin<Box<dyn Future<Output = ()> + Send>>;

struct Task {
    future: Mutex<Option<BoxedTask>>,
    queue: mpsc::Sender<Arc<Task>>,
}

impl Wake for Task {
    fn wake(self: Arc<Self>) {
        let queue = self.queue.clone();
        let _ = queue.send(self); // re-queue: "I can make progress"
    }
}

fn run_all(futures: Vec<BoxedTask>) {
    let (sender, queue) = mpsc::channel::<Arc<Task>>();
    for fut in futures {
        let task = Arc::new(Task {
            future: Mutex::new(Some(fut)),
            queue: sender.clone(),
        });
        sender.send(task).unwrap();
    }
    drop(sender); // when every task (and its waker) is gone, recv() fails and we stop

    while let Ok(task) = queue.recv() {
        let mut slot = task.future.lock().unwrap();
        if let Some(mut fut) = slot.take() {
            let waker = Waker::from(Arc::clone(&task));
            let mut cx = Context::from_waker(&waker);
            if fut.as_mut().poll(&mut cx).is_pending() {
                *slot = Some(fut); // not done: keep it for the next wake
            }
        }
    }
}

/// Returns Pending once, after asking to be polled again.
fn yield_now() -> impl Future<Output = ()> {
    struct YieldNow(bool);
    impl Future for YieldNow {
        type Output = ();
        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
            if self.0 {
                Poll::Ready(())
            } else {
                self.0 = true;
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }
    YieldNow(false)
}

fn executor_demo() {
    let log = Arc::new(Mutex::new(Vec::new()));
    let tasks: Vec<BoxedTask> = ["brakes", "engine", "display"]
        .into_iter()
        .map(|name| {
            let log = Arc::clone(&log);
            Box::pin(async move {
                for step in 1..=2 {
                    log.lock().unwrap().push(format!("{name} step {step}"));
                    yield_now().await; // let the others run
                }
            }) as BoxedTask
        })
        .collect();
    run_all(tasks);
    for line in log.lock().unwrap().iter() {
        println!("{line}");
    }
}
// ANCHOR_END: executor

fn join_demo() {
    // ANCHOR: join
    /// Polls two futures concurrently; ready when both are.
    struct Join2<A: Future, B: Future> {
        a: Pin<Box<A>>,
        b: Pin<Box<B>>,
        a_out: Option<A::Output>,
        b_out: Option<B::Output>,
    }

    impl<A: Future, B: Future> Future for Join2<A, B>
    where
        A::Output: Unpin,
        B::Output: Unpin,
    {
        type Output = (A::Output, B::Output);
        fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
            let this = &mut *self; // all fields are Unpin (the futures are boxed)
            if this.a_out.is_none()
                && let Poll::Ready(v) = this.a.as_mut().poll(cx)
            {
                this.a_out = Some(v);
            }
            if this.b_out.is_none()
                && let Poll::Ready(v) = this.b.as_mut().poll(cx)
            {
                this.b_out = Some(v);
            }
            if this.a_out.is_some() && this.b_out.is_some() {
                Poll::Ready((this.a_out.take().unwrap(), this.b_out.take().unwrap()))
            } else {
                Poll::Pending
            }
        }
    }

    fn join<A: Future, B: Future>(a: A, b: B) -> Join2<A, B> {
        Join2 {
            a: Box::pin(a),
            b: Box::pin(b),
            a_out: None,
            b_out: None,
        }
    }

    let start = Instant::now();
    block_on(async {
        sleep(Duration::from_millis(30)).await;
        sleep(Duration::from_millis(30)).await;
    });
    let sequential = start.elapsed();

    let start = Instant::now();
    let (x, y) = block_on(join(
        async {
            sleep(Duration::from_millis(30)).await;
            "front radar"
        },
        async {
            sleep(Duration::from_millis(30)).await;
            "rear camera"
        },
    ));
    let joined = start.elapsed();
    println!("joined results: {x}, {y}");
    println!(
        "sequential ≥ 60 ms: {}",
        sequential >= Duration::from_millis(60)
    );
    println!("joined faster than sequential: {}", joined < sequential);
    // ANCHOR_END: join
}

fn async_closures() {
    // ANCHOR: closures
    let offset = 100;
    let add_offset = async |x: u32| x + offset; // async closure (Rust 1.85+)
    println!("async closure → {}", block_on(add_offset(23)));

    trait Sensor {
        async fn read(&self) -> f64; // async fn in a trait (Rust 1.75+)
    }
    struct Lidar;
    impl Sensor for Lidar {
        async fn read(&self) -> f64 {
            sleep(Duration::from_millis(1)).await;
            12.5
        }
    }
    println!("lidar distance {} m", block_on(Lidar.read()));

    // A small work queue that is processed concurrently by two tasks:
    let jobs = Arc::new(Mutex::new(VecDeque::from([1u32, 2, 3, 4, 5])));
    let done = Arc::new(Mutex::new(Vec::new()));
    let workers: Vec<BoxedTask> = (0..2)
        .map(|w| {
            let (jobs, done) = (Arc::clone(&jobs), Arc::clone(&done));
            Box::pin(async move {
                loop {
                    let job = jobs.lock().unwrap().pop_front();
                    let Some(job) = job else { break };
                    done.lock().unwrap().push((w, job));
                    yield_now().await;
                }
            }) as BoxedTask
        })
        .collect();
    run_all(workers);
    println!("(worker, job): {:?}", done.lock().unwrap());
    // ANCHOR_END: closures
}
