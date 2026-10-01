//! Lesson: Concurrency patterns for vehicles.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender, TrySendError};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use crate::lesson::{Lesson, Question, Section};

pub static LESSON: Lesson = Lesson {
    id: "vehicle_patterns",
    title: "Concurrency patterns for vehicles",
    summary: "Drift-free periodic tasks, watchdogs, bounded pipelines, a non-blocking signal bus, latest-value sharing, graceful shutdown, and scheduling caveats.",
    source: include_str!("vehicle_patterns.rs"),
    sections: &[
        Section::new(
            "Periodic tasks without drift",
            r#"
            Control software runs **cyclic** tasks: read sensors every 10 ms,
            update the display every 100 ms. The naive loop, `work();
            sleep(period);`, drifts. Every iteration adds the work time plus the
            scheduler's lateness, so after an hour the task has slipped by
            seconds.

            Schedule against **absolute deadlines** instead:
            `next = start + n × period`, and sleep only until `next`. Lateness
            in one cycle doesn't accumulate. When a cycle overruns its deadline
            entirely, **count** it: an overrun is a timing fault worth
            reporting. Desktop OSes give no timing guarantees, so expect
            jitter. A real-time OS bounds it.
            "#,
        )
        .demo("periodic", periodic),
        Section::new(
            "A watchdog",
            r#"
            A **watchdog** detects a task that has hung or died. The task
            "kicks" it periodically, here by storing a timestamp in an atomic.
            A supervisor checks that the last kick is recent enough, and acts
            if it isn't: log, restart the task, or drive outputs to a safe
            state. Hardware watchdogs reset the whole microcontroller when
            software stops kicking. This is the same pattern in software, for
            individual tasks.
            "#,
        )
        .demo("watchdog", watchdog),
        Section::new(
            "Bounded pipelines",
            r#"
            Split work into stages connected by **bounded** channels:
            acquisition → filtering → logging. Each stage owns its state, so
            there are no locks, and the bounds cap memory. Every boundary needs
            a **full-queue policy** (Lesson 33): block (backpressure), drop
            the newest, or drop the oldest. Whatever you choose, **count the
            drops**. A pipeline that silently loses data under load passes
            every test and fails in the field.
            "#,
        )
        .demo("pipeline", pipeline),
        Section::new(
            "A signal bus that never blocks the publisher",
            r#"
            Many consumers want the same signals: the instrument cluster, the
            data logger, the ADAS function. A **publish/subscribe bus** gives
            each subscriber its own bounded queue. The publisher uses
            `try_send`, so a slow or stuck subscriber **can't stall** the
            publisher, which may be on a critical path. The slow subscriber
            simply drops messages, and the bus counts them. That's freedom
            from interference, in miniature.
            "#,
        )
        .demo("bus", signal_bus),
        Section::new(
            "Latest-value sharing",
            r#"
            Queues are wrong for state-like signals. A consumer that wants "the
            current vehicle speed" doesn't want 50 stale samples. It wants the
            **newest**. Share a single slot that writers overwrite and readers
            copy:

            - an `Arc<Mutex<T>>` or `RwLock<T>` for larger values;
            - an **atomic** for small ones. An `f64` fits in an `AtomicU64`
              via `to_bits`/`from_bits`, and a timestamp next to it tells
              readers how fresh the value is.

            Readers never block writers, and memory stays constant however fast
            either side runs.
            "#,
        )
        .demo("latest", latest_value),
        Section::new(
            "Graceful shutdown",
            r#"
            Every thread needs a way to **stop** cleanly: on ignition-off, on
            update, or in tests. Two common signals:

            - **closing channels**: when all senders are dropped, `recv`
              returns an error and the worker exits its loop;
            - a shared **stop flag** (`AtomicBool`), for workers that aren't
              waiting on a channel.

            Then **join** every thread, so their destructors run and their
            buffers flush. A process that exits with threads still running
            loses that work.
            "#,
        )
        .demo("shutdown", shutdown),
        Section::new(
            "Priorities, inversion and isolation",
            r#"
            Standard threads have **no priority API**. Embedded and automotive
            systems run on an RTOS or AUTOSAR OS that schedules by fixed
            priority, and that brings its own classic bug: **priority
            inversion**. A low-priority task holds a mutex that a high-priority
            task needs, while a medium-priority task keeps the low one from
            running, so the high-priority task waits indefinitely. (It famously
            reset the Mars Pathfinder lander in 1997.) The defences are
            priority-inheritance mutexes, minimal critical sections, and
            designs that need no locks at all: message passing,
            single-writer atomics, and lock-free queues (Lesson 24).

            At a larger scale, ISO 26262 asks for **freedom from interference**
            between components of different criticality. In practice that means
            separate processes or partitions, bounded communication, and
            supervision, which is this lesson's toolkit applied between
            subsystems.
            "#,
        ),
    ],
    quiz: &[
        Question::new(
            "Why does `loop { work(); sleep(period); }` drift?",
            &[
                "sleep is inaccurate",
                "Each iteration adds work time and scheduling delay, and the errors accumulate",
                "It doesn't drift",
                "Because of integer overflow",
            ],
            1,
            "Scheduling against absolute deadlines (start + n·period) prevents accumulation.",
        ),
        Question::new(
            "Why should a signal-bus publisher use `try_send` rather than `send`?",
            &[
                "It's faster",
                "So a slow subscriber can't block the publisher; drops are counted instead",
                "send doesn't work with bounded channels",
                "To avoid allocation",
            ],
            1,
            "Blocking the publisher would let the slowest consumer dictate the timing of a possibly critical producer.",
        ),
        Question::new(
            "Which mechanism suits 'the current vehicle speed' read by many tasks?",
            &[
                "An unbounded channel",
                "A latest-value slot (atomic or lock-protected)",
                "A BinaryHeap",
                "A thread-local",
            ],
            1,
            "State-like signals need the newest value, not a backlog.",
        ),
        Question::new(
            "What is priority inversion?",
            &[
                "Sorting tasks in reverse",
                "A high-priority task waits on a resource held by a low-priority task that is itself starved by medium-priority work",
                "Using Reverse in a BinaryHeap",
                "A deadlock between two equal-priority tasks",
            ],
            1,
            "Mitigations include priority inheritance, short critical sections, and lock-free designs.",
        ),
        Question::new(
            "How does a worker blocked on `rx.recv()` learn that it should stop?",
            &[
                "It polls a timer",
                "All senders are dropped, so recv returns Err",
                "It receives a SIGTERM",
                "It can't",
            ],
            1,
            "Channel closure is a built-in shutdown signal: recv fails once no sender remains.",
        ),
    ],
    exercises: &[
        "Run the periodic task with a 2 ms period while another thread burns CPU. Record jitter (actual minus planned start) in a histogram using a BTreeMap.",
        "Make the watchdog restart the worker thread (up to three times) instead of only reporting it.",
        "Give the signal bus per-topic subscriptions (`subscribe(topic)`), and add a `stats()` method reporting drops per subscriber.",
        "Implement a seqlock-style latest-value cell for a 3-field struct using an `AtomicU32` version counter plus atomics for the fields.",
    ],
};

fn periodic() {
    // ANCHOR: periodic
    let period = Duration::from_millis(5);
    let start = Instant::now();
    let mut overruns = 0;
    let mut worst_lateness = Duration::ZERO;

    for cycle in 1..=10u32 {
        // the task's work: simulate a slow cycle now and then
        let work = if cycle == 4 {
            Duration::from_millis(8)
        } else {
            Duration::from_micros(300)
        };
        thread::sleep(work);

        let deadline = start + period * cycle; // absolute: lateness can't accumulate
        let now = Instant::now();
        if now > deadline {
            overruns += 1; // count timing faults; don't hide them
            worst_lateness = worst_lateness.max(now - deadline);
        } else {
            thread::sleep(deadline - now);
        }
    }
    let total = start.elapsed();
    println!(
        "10 cycles of 5 ms took ≈ {} ms (drift-free target: 50)",
        (total.as_millis() / 5) * 5
    );
    println!("overruns: {overruns} (cycle 4 was slow)");
    println!(
        "worst lateness under 10 ms: {}",
        worst_lateness < Duration::from_millis(10)
    );
    // ANCHOR_END: periodic
}

fn watchdog() {
    // ANCHOR: watchdog
    let epoch = Instant::now();
    let last_kick_us = Arc::new(AtomicU64::new(0));
    let stop = Arc::new(AtomicBool::new(false));

    let worker = {
        let (kick, stop) = (Arc::clone(&last_kick_us), Arc::clone(&stop));
        thread::spawn(move || {
            for i in 0.. {
                if stop.load(Ordering::Relaxed) {
                    break;
                }
                if i < 5 {
                    kick.store(epoch.elapsed().as_micros() as u64, Ordering::Release); // "I'm alive"
                } // after 5 iterations the worker "hangs": it stops kicking
                thread::sleep(Duration::from_millis(2));
            }
        })
    };

    let timeout = Duration::from_millis(15);
    let mut verdict = "worker healthy";
    for _ in 0..50 {
        thread::sleep(Duration::from_millis(2));
        let last = Duration::from_micros(last_kick_us.load(Ordering::Acquire));
        if epoch.elapsed().saturating_sub(last) > timeout {
            verdict = "watchdog expired → entering safe state";
            break;
        }
    }
    stop.store(true, Ordering::Relaxed);
    worker.join().unwrap();
    println!("{verdict}");
    // ANCHOR_END: watchdog
}

fn pipeline() {
    // ANCHOR: pipeline
    let (raw_tx, raw_rx) = mpsc::sync_channel::<u16>(4); //    sensor → filter
    let (avg_tx, avg_rx) = mpsc::sync_channel::<f64>(4); //    filter → logger
    let dropped = Arc::new(AtomicU32::new(0));

    let sensor = {
        let dropped = Arc::clone(&dropped);
        thread::spawn(move || {
            for sample in 0..200u16 {
                // Drop-newest policy at this boundary: never block the sensor.
                if let Err(TrySendError::Full(_)) = raw_tx.try_send(500 + sample % 50) {
                    dropped.fetch_add(1, Ordering::Relaxed);
                }
                thread::sleep(Duration::from_micros(100)); // a 10 kHz sensor
            }
        }) // raw_tx dropped here → the filter's loop ends
    };

    let filter = thread::spawn(move || {
        let mut window = [0u16; 4];
        let mut n = 0usize;
        for sample in raw_rx {
            window[n % 4] = sample;
            n += 1;
            if n >= 4 {
                let avg = window.iter().map(|&s| f64::from(s)).sum::<f64>() / 4.0;
                if avg_tx.send(avg).is_err() {
                    break; // logger gone
                }
            }
        }
    });

    let logger = thread::spawn(move || {
        let mut count = 0;
        let mut last = 0.0;
        for avg in avg_rx {
            thread::sleep(Duration::from_micros(200)); // a slow consumer
            count += 1;
            last = avg;
        }
        (count, last)
    });

    sensor.join().unwrap();
    filter.join().unwrap();
    let (logged, last) = logger.join().unwrap();
    let dropped = dropped.load(Ordering::Relaxed);
    println!(
        "logged {logged} averages (last {last:.1}), dropped {dropped} samples — counts vary with timing"
    );
    println!(
        "every sample accounted for: {}",
        logged as u32 + 3 + dropped == 200
    );
    // ANCHOR_END: pipeline
}

fn signal_bus() {
    // ANCHOR: bus
    #[derive(Debug, Clone, Copy)]
    struct Signal {
        id: u16,
        value: f32,
    }

    struct Subscriber {
        name: &'static str,
        queue: SyncSender<Signal>,
        dropped: u32,
    }

    #[derive(Default)]
    struct Bus {
        subscribers: Vec<Subscriber>,
    }
    impl Bus {
        fn subscribe(&mut self, name: &'static str, capacity: usize) -> Receiver<Signal> {
            let (tx, rx) = mpsc::sync_channel(capacity);
            self.subscribers.push(Subscriber {
                name,
                queue: tx,
                dropped: 0,
            });
            rx
        }
        fn publish(&mut self, s: Signal) {
            for sub in &mut self.subscribers {
                if sub.queue.try_send(s).is_err() {
                    sub.dropped += 1; // full (or gone): never block the publisher
                }
            }
        }
        fn stats(&self) -> BTreeMap<&'static str, u32> {
            self.subscribers
                .iter()
                .map(|s| (s.name, s.dropped))
                .collect()
        }
    }

    let mut bus = Bus::default();
    let cluster = bus.subscribe("cluster", 64);
    let _stuck_logger = bus.subscribe("stuck_logger", 8); // never reads

    let display = thread::spawn(move || cluster.iter().map(|s| (s.id, s.value)).last());

    let start = Instant::now();
    for i in 0..50u16 {
        bus.publish(Signal {
            id: 0x0D,
            value: f32::from(i) * 2.0,
        });
    }
    let publish_time = start.elapsed();
    let stats = bus.stats();
    drop(bus); // closes the queues: the display thread finishes

    println!("display's last value: {:?}", display.join().unwrap());
    println!("drops per subscriber: {stats:?}");
    println!(
        "publisher never blocked: {}",
        publish_time < Duration::from_millis(50)
    );
    // ANCHOR_END: bus
}

fn latest_value() {
    // ANCHOR: latest
    /// One writer, many readers; readers always see the newest value.
    struct LatestF64 {
        bits: AtomicU64,
        stamp_us: AtomicU64,
    }
    impl LatestF64 {
        fn publish(&self, v: f64, now_us: u64) {
            self.bits.store(v.to_bits(), Ordering::Relaxed);
            self.stamp_us.store(now_us, Ordering::Release); // publish after the value
        }
        fn read(&self) -> (f64, u64) {
            let stamp = self.stamp_us.load(Ordering::Acquire);
            (f64::from_bits(self.bits.load(Ordering::Relaxed)), stamp)
        }
    }

    let speed = LatestF64 {
        bits: AtomicU64::new(0f64.to_bits()),
        stamp_us: AtomicU64::new(0),
    };
    let epoch = Instant::now();
    thread::scope(|s| {
        s.spawn(|| {
            for i in 1..=100 {
                speed.publish(f64::from(i), epoch.elapsed().as_micros() as u64 + 1);
                thread::sleep(Duration::from_micros(50));
            }
        });
        s.spawn(|| {
            let mut previous = 0.0;
            let mut monotonic = true;
            for _ in 0..50 {
                let (v, _) = speed.read();
                monotonic &= v >= previous; // readers never see older-than-seen values here
                previous = v;
                thread::sleep(Duration::from_micros(80));
            }
            println!("reader saw non-decreasing speeds: {monotonic}");
        });
    });
    let (final_speed, stamp) = speed.read();
    println!("latest value {final_speed} (stamped: {})", stamp > 0);
    // ANCHOR_END: latest
}

fn shutdown() {
    // ANCHOR: shutdown
    let stop = Arc::new(AtomicBool::new(false));
    let processed = Arc::new(Mutex::new(Vec::new()));
    let (tx, rx) = mpsc::channel::<u32>();

    // Worker 1 stops when the channel closes.
    let consumer = {
        let processed = Arc::clone(&processed);
        thread::spawn(move || {
            for job in rx {
                processed.lock().unwrap().push(job);
            }
            "consumer: channel closed, exiting"
        })
    };
    // Worker 2 stops when the flag is raised.
    let heartbeat = {
        let stop = Arc::clone(&stop);
        thread::spawn(move || {
            let mut beats = 0u32;
            while !stop.load(Ordering::Acquire) {
                beats += 1;
                thread::sleep(Duration::from_millis(1));
            }
            format!("heartbeat: stop flag seen after ≥1 beats: {}", beats >= 1)
        })
    };

    for job in 1..=5 {
        tx.send(job).unwrap();
    }
    thread::sleep(Duration::from_millis(5));
    drop(tx); //                           signal 1: close the channel
    stop.store(true, Ordering::Release); // signal 2: raise the flag

    println!("{}", consumer.join().unwrap()); // join everything
    println!("{}", heartbeat.join().unwrap());
    println!(
        "jobs processed before shutdown: {:?}",
        processed.lock().unwrap()
    );
    // ANCHOR_END: shutdown
}
