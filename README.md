# rust_std

**Learn Rust with nothing but the standard library.** An interactive command-line course — 41 lessons, 300+ live demos, 200+ quiz questions — written in Rust, with **zero dependencies**.

Every code sample you read in a lesson is a real function in this program's source. The CLI cuts it out of its own file, shows it with syntax highlighting, and then **runs it** so you see the actual output. Nothing is pasted from elsewhere, so nothing can go stale.

The course runs from `println!` to lock-free ring buffers and ends with a dedicated **systems & automotive track**: byte-level protocol parsing, CAN frames, allocation-free data structures, typestate, panic freedom, FFI against real libc functions, `no_std`, vehicle concurrency patterns, and safety-critical context (ISO 26262, E2E protection, redundancy voting). It closes with an interview clinic and a capstone project.

## Quick start

```sh
cargo run --release            # interactive menu
cargo run --release -- list    # all lessons
cargo run --release -- learn 6 # step through lesson 6 (Ownership)
```

Requires Rust **1.95 or newer** (edition 2024). Install with [rustup](https://rustup.rs).

## Commands

| Command | What it does |
|---|---|
| *(none)* | Interactive menu. Type a number or a name to start a lesson. |
| `list` | All lessons, with ✓ for finished ones and your best quiz score |
| `learn <lesson>` | Step through a lesson section by section (`b` back, `r` repeat, `c` full source) |
| `show <lesson>` | Print a whole lesson without pausing (handy with `less -R`) |
| `run <lesson>` | Only the code and live output of each demo |
| `code <lesson>` | The lesson's complete source file, highlighted |
| `quiz [<lesson>]` | Quiz one lesson, or 15 random review questions from finished lessons |
| `search <text>` | Find every lesson and section that mentions `<text>` |
| `next` | Continue with the first unfinished lesson |
| `progress` / `reset` | Show or clear your progress |

`<lesson>` is a number (`7`), an id (`borrowing`), or a unique prefix or title fragment (`borrow`). Flags: `--no-color`, `--color`, `--width N`. `NO_COLOR` is respected. Progress is saved to `$RUST_STD_PROGRESS` or `~/.rust_std_progress`. Quiz choices are shuffled every time.

## Why you can trust the material

- **Shown code is compiled code.** Lessons mark regions with `// ANCHOR: name` comments; the CLI extracts and runs exactly those regions (`src/snippet.rs`).
- **Every prose example is compiled by the tests.** `tests/prose_examples.rs` compiles each `rust` block in the lesson text with `rustc`, and checks that each "does not compile" example *fails* with the error code it claims (E0382, E0499, E0502, E0597, …). If a future compiler changes behaviour, the test suite says so.
- **Claims are measured.** A counting global allocator (`src/alloc_counter.rs`) lets the systems lessons print — and assert — "0 heap allocations". Fuzz loops count panics (zero) instead of asserting robustness in prose.
- **Every demo runs in CI**, and every lesson is rendered end-to-end through the real binary (`tests/cli.rs`).

## Curriculum

**Part I — Foundations**

| # | Lesson | id | What you'll learn |
|---|---|---|---|
| 1 | Hello, Rust & formatting | `hello` | The anatomy of a Rust program, the printing macros, and the full format-string language. |
| 2 | Variables, constants & statics | `bindings` | let, mut, shadowing, type inference, deferred initialization, const, static, and the special pattern _. |
| 3 | Numbers, bools & chars | `numbers` | Integer and float types, overflow policies, conversions with as/From/TryFrom, parsing, bool, char and NonZero. |
| 4 | Tuples, arrays & slices | `aggregates` | Fixed-size groupings of values, and slices: borrowed views into contiguous data. |
| 5 | Functions & control flow | `flow` | Functions, expressions vs statements, if/loop/while/for, labels, break values, ranges, the never type and fn pointers. |

**Part II — Ownership & borrowing**

| # | Lesson | id | What you'll learn |
|---|---|---|---|
| 6 | Ownership & moves | `ownership` | How Rust manages memory without a garbage collector: owners, moves, Copy, Clone, Drop, and mem::take/replace. |
| 7 | Borrowing & references | `borrowing` | Shared and exclusive references, the aliasing rule that prevents data races at compile time, NLL, reborrowing and split borrows. |
| 8 | Strings & UTF-8 | `strings` | String vs &str, UTF-8 and why you can't index by position, iteration, searching, splitting, building, and the other string types. |
| 9 | Lifetimes | `lifetimes` | What lifetimes are, when you must write them, elision rules, structs that borrow, 'static, variance and higher-ranked bounds. |

**Part III — Modeling data**

| # | Lesson | id | What you'll learn |
|---|---|---|---|
| 10 | Structs & methods | `structs` | Named, tuple and unit structs; methods and associated functions; derives; invariants through privacy; builders; memory layout and repr. |
| 11 | Enums & match | `enums` | Sum types with data, exhaustive matching, methods on enums, discriminants, recursive enums, and enum sizes. |
| 12 | Patterns in depth | `patterns` | Every pattern form: literals, ranges, alternatives, destructuring, guards, @ bindings, slice patterns, binding modes, if let, let-else, while let and let chains. |
| 13 | Option & Result | `option_result` | Absence and failure as values: extracting, transforming and combining Options and Results, the ? operator, and collecting. |
| 14 | Error handling | `errors` | Panics vs Results, custom error types, Display and Error, ? with From, Box<dyn Error>, error chains, and exit codes. |

**Part IV — Abstraction**

| # | Lesson | id | What you'll learn |
|---|---|---|---|
| 15 | Generics | `generics` | Generic functions, structs, enums and impls; bounds and where clauses; monomorphization; turbofish; const generics; PhantomData. |
| 16 | Traits & trait objects | `traits` | Defining and implementing traits, default methods, bounds, impl Trait, dyn Trait, dispatch trade-offs, dyn compatibility, associated types, supertraits, upcasting, coherence and sealed traits. |
| 17 | Standard traits & operators | `std_traits` | Implementing Display, Debug, PartialEq/Eq, Ord, Hash, Default, From/TryFrom, FromStr, AsRef/Borrow, Deref, operator traits, Sum and FromIterator. |
| 18 | Closures | `closures` | Anonymous functions that capture their environment: syntax, capture modes, Fn/FnMut/FnOnce, move, passing and returning closures, and callbacks. |
| 19 | Iterators | `iterators` | The Iterator trait, iter/iter_mut/into_iter, laziness, adapters, consumers, collect, writing your own iterators and collections, and constructors in std::iter. |

**Part V — The standard library**

| # | Lesson | id | What you'll learn |
|---|---|---|---|
| 20 | Collections | `collections` | Vec, VecDeque, HashMap, HashSet, BTreeMap, BTreeSet and BinaryHeap: what each is for, their key methods, the entry API, and costs. |
| 21 | Smart pointers & interior mutability | `smart_pointers` | Box, Rc, Weak, Cell, RefCell, Cow, OnceCell and LazyCell — and the spectrum from compile-time to run-time checking. |
| 22 | Modules, visibility & crates | `modules` | Organizing code with modules, the visibility spectrum, paths and use, re-exports, files and directories, crates, the prelude and conditional compilation. |
| 23 | Threads & synchronization | `threads` | Spawning and scoped threads, Send and Sync, channels, Arc<Mutex>, poisoning, RwLock, Condvar, Barrier, OnceLock/LazyLock, thread-locals and deadlock avoidance. |
| 24 | Atomics & memory ordering | `atomics` | Lock-free primitives: atomic operations, compare-and-swap, the memory-ordering spectrum, release/acquire publication, a spinlock and a lock-free SPSC ring buffer. |
| 25 | I/O, files & processes | `io` | stdin/stdout, the Read/Write/BufRead traits, buffering, files and directories, paths, io::Error, environment variables and child processes. |
| 26 | A tour of std | `std_tour` | The smaller modules you reach for every week: std::time, std::mem, std::cmp, std::hash, std::any, std::num, std::ops ranges and std::hint. |

**Part VI — Advanced Rust**

| # | Lesson | id | What you'll learn |
|---|---|---|---|
| 27 | Declarative macros | `macros` | macro_rules!: matching, fragment specifiers, repetition, recursion, hygiene, generating items such as register tables, and the built-in macros. |
| 28 | Testing | `testing` | Unit, integration and doc tests; should_panic and Result tests; table-driven golden vectors; property tests with a seeded PRNG; test doubles; benchmarks without crates. |
| 29 | Advanced types | `advanced_types` | Type aliases, the never type and Infallible, dynamically sized types and ?Sized, zero-sized types, niches, fn item types, Pin, impl Trait in traits and GATs. |
| 30 | Async from scratch | `async` | Futures, polling, wakers and Pin — then a block_on executor, a timer, a multi-task executor and join, all with std only. |

**Part VII — Systems & automotive Rust**

| # | Lesson | id | What you'll learn |
|---|---|---|---|
| 31 | Bytes & bits | `bytes_bits` | Endianness, masks and shifts, bit fields, sign extension, safe binary parsing and writing, compile-time CRC tables, and hex dumps. |
| 32 | Parsing CAN frames | `can_frames` | A validated CAN frame type, text and binary parsers that never panic, Intel/Motorola signal extraction, scaling, encoding, OBD-II decoding and fuzzing. |
| 33 | Fixed-capacity, allocation-free data | `fixed_capacity` | Why safety-critical code avoids the heap, and how to build bounded containers: a stack vector, a ring buffer with overflow policies, a fixed string, and a generational pool — measured to allocate nothing. |
| 34 | Typestate & newtypes | `typestate` | Making illegal states unrepresentable: unit newtypes, typestate state machines (ignition, UDS sessions), sealed states, typed builders and capability tokens. |
| 35 | Panic freedom & determinism | `panic_free` | Where panics hide, how to write total functions, lint-enforced panic policies, overflow and panic settings, panic hooks, and sources of non-determinism. |
| 36 | Unsafe Rust & FFI | `unsafe_ffi` | What unsafe permits, raw pointers, SAFETY contracts, building safe abstractions, calling C (strlen, qsort, abs), exposing Rust to C, ownership across FFI, static mut rules, MaybeUninit and tooling. |
| 37 | core, alloc & std | `no_std` | The three layers of the standard library, what #![no_std] code can use, writing portable code from day one, and what a bare-metal binary needs. |
| 38 | Concurrency patterns for vehicles | `vehicle_patterns` | Drift-free periodic tasks, watchdogs, bounded pipelines, a non-blocking signal bus, latest-value sharing, graceful shutdown, and scheduling caveats. |
| 39 | Safety-critical Rust in context | `safety_critical` | ISO 26262 vocabulary, what Rust does and doesn't solve, qualified toolchains, coding guidelines, a project baseline, and defensive patterns: plausibility, voting, E2E protection, degradation and traceability. |

**Part VIII — Career**

| # | Lesson | id | What you'll learn |
|---|---|---|---|
| 40 | Interview clinic | `interview` | Model answers to common Rust interview questions, predict-the-output drills, and whiteboard classics in idiomatic Rust: queue from stacks, linked list, O(1) LRU cache, and array/string problems. |
| 41 | Capstone: a signal gateway | `capstone` | Everything together: parse a CAN capture, verify E2E protection, decode and plausibility-check signals, fan out over threads to a dashboard and a bounded log, behind a typestate API that never panics. |

## Study paths

**Learning Rust from scratch.** Go in order. Do the "Try it yourself" exercises at the end of each lesson in a scratch project (`cargo new scratch`), and run `quiz` (review mode) every few days.

**Preparing for a systems or automotive Rust role** (for example, an interview at an automotive software group). If you already program in C/C++:

1. Skim Parts I–III. Read lessons 6–9 (ownership, borrowing, lifetimes) carefully: interviews always probe them.
2. Lessons 16 (traits & dyn), 21 (smart pointers), 23–24 (threads, atomics), 30 (async).
3. All of Part VII, in order. It's the domain-specific material: protocols, determinism, `unsafe`/FFI, `no_std`, safety-critical vocabulary.
4. Lesson 40 out loud: answer every drill without looking, then compare. Re-implement the whiteboard problems from memory.
5. Lesson 41: read it, then do its exercises. Being able to talk through a design like this — and its failure modes — is the best preparation.

Safety-critical tooling and standards move quickly (qualified toolchains, the Ferrocene Language Specification, consortium guidelines). Lesson 39 gives the landscape as of 2025–2026. Verify current specifics before quoting them.

## Project layout

```text
src/
├── main.rs            three lines: calls rust_std::cli::main()
├── lib.rs             module tree
├── cli.rs             argument parsing, interactive menu, commands
├── viewer.rs          prints lessons and runs demos (catching panics)
├── lesson.rs          the Lesson / Section / Question data model
├── snippet.rs         extracts ANCHOR regions from source text
├── ui.rs              colors, word wrapping, a mini Markdown renderer, a Rust highlighter
├── quiz.rs            multiple-choice quizzes
├── progress.rs        plain-text progress file
├── rng.rs             xorshift PRNG (quiz shuffling, property tests)
├── console.rs         line input, swappable for tests
├── alloc_counter.rs   counting global allocator
└── lessons/
    ├── mod.rs         course order (PARTS) and registry tests
    └── *.rs           one file per lesson: prose, quiz, exercises, and demo code
tests/
├── cli.rs             drives the real binary
└── prose_examples.rs  compiles every code block in the lesson prose
```

The machinery is written to be read too: argument parsing, a terminal renderer, a syntax highlighter, a PRNG, file-based persistence and a global allocator, all with `std` only.

## Development

```sh
cargo test                                  # unit, integration, doc tests + prose compilation
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

The tests check, among other things, that every lesson file is registered, every anchor exists and is shown, every quiz is well-formed, every demo runs without panicking, every lesson renders through the binary, and every prose example compiles (or fails) as stated.

### Adding a lesson

1. Create `src/lessons/<id>.rs` with a `pub static LESSON: Lesson` (copy any lesson as a template).
2. Write demo functions and wrap the code to display in `// ANCHOR: name` / `// ANCHOR_END: name`.
3. Add sections with `Section::new(title, prose).demo("name", function)` (or `.code("name")` to show without running).
4. Add `mod <id>;` and an entry in `PARTS` in `src/lessons/mod.rs`.
5. Use fenced blocks in prose: `` ```rust `` (must compile), `` ```compile_fail,E0xxx `` (must fail with that code), `` ```text `` (not checked). Run `cargo test`.
