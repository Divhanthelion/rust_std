//! Lesson: core, alloc & std.

use std::any::TypeId;
use std::fmt::Write as _;

use crate::lesson::{Lesson, Question, Section};

pub static LESSON: Lesson = Lesson {
    id: "no_std",
    title: "core, alloc & std",
    summary: "The three layers of the standard library, what #![no_std] code can use, writing portable code from day one, and what a bare-metal binary needs.",
    source: include_str!("no_std.rs"),
    sections: &[
        Section::new(
            "Three layers",
            r#"
            "The standard library" is really three libraries stacked on top
            of each other:

            ```text
            std     everything below, plus the operating system: threads, files,
                    networking, processes, env, Instant/SystemTime, HashMap
              ▲
            alloc   needs a heap allocator: Box, Vec, String, Rc, Arc,
                    BTreeMap, VecDeque, BinaryHeap, format!
              ▲
            core    needs nothing at all: Option, Result, iterators, slices,
                    str, char, fmt, cell, mem, ptr, num, atomics, Duration,
                    Future/Poll/Waker, the panic machinery
            ```

            A microcontroller with no OS and no heap still gets **all of
            core**. Most of this course's Parts I–IV work there unchanged.
            `std` re-exports `core` and `alloc`, so `std::option::Option` *is*
            `core::option::Option`. Same type, two paths.
            "#,
        )
        .demo("layers", layers),
        Section::new(
            "Why HashMap is std-only, and other surprises",
            r#"
            Some placements look odd until you ask what each type needs:

            - `HashMap` needs **randomness** (for its DoS-resistant hasher
              seed), which comes from the OS, so it lives in std. `BTreeMap`
              only needs allocation, so it lives in alloc.
            - `Duration` is plain arithmetic, so it's in core. `Instant` reads
              a clock, so it's in std.
            - `fmt::Write` and `format_args!` are in core. `format!`
              produces a `String`, so it needs alloc.
            - `Error` moved to core in Rust 1.81, so no_std libraries can
              implement it.
            - Float methods like `sqrt` and `sin` historically came from the
              platform's libm, and mostly live in std.
            "#,
        ),
        Section::new(
            "A #![no_std] library",
            r#"
            A library opts out of std with one attribute. Then it can only
            name `core` (and `alloc`, if it adds `extern crate alloc;`). This
            signal decoder compiles as a real `#![no_std]` crate. The test
            suite of this project compiles it on every run:

            ```rust,nowrap
            #![no_std]

            /// A CAN signal decoder usable on any target, with or without an OS.
            #[derive(Debug, Clone, Copy, PartialEq)]
            pub enum DecodeError {
                Layout,
            }

            pub fn extract_le(data: &[u8; 8], start: u32, len: u32) -> Result<u64, DecodeError> {
                if len == 0 || start + len > 64 {
                    return Err(DecodeError::Layout);
                }
                let word = u64::from_le_bytes(*data);
                let mask = if len == 64 { u64::MAX } else { (1u64 << len) - 1 };
                Ok((word >> start) & mask)
            }

            impl core::fmt::Display for DecodeError {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    f.write_str("signal does not fit in 8 bytes")
                }
            }

            impl core::error::Error for DecodeError {}
            ```

            A common setup is `#![cfg_attr(not(test), no_std)]`: the crate is
            no_std when shipped, but its tests use std (and `println!`) on the
            host.
            "#,
        ),
        Section::new(
            "Write portable code from day one",
            r#"
            Even in std programs, you can keep the core of your logic
            no_std-ready:

            - write `core::` paths for things that live in core (`core::mem`,
              `core::fmt`), so moving the module later is a no-op;
            - format into **fixed buffers** through `core::fmt::Write`, as the
              `FixedStr` of Lesson 33 does, instead of using `format!`;
            - take `&[u8]` and return `Result`, rather than reading files or
              printing;
            - keep I/O, threads and the clock at the **edges**, behind traits
              (Lesson 28).

            Then the same decoding, validation and state-machine code can run
            on a gateway ECU running Linux and on a microcontroller.
            "#,
        )
        .demo("portable", portable),
        Section::new(
            "What a bare-metal binary needs",
            r#"
            A `#![no_std]` **binary** for a microcontroller must supply what std
            normally provides:

            - a **panic handler**: `#[panic_handler] fn panic(info: &PanicInfo)
              -> !`. Usually it logs, puts outputs in a safe state, and resets.
            - an **entry point**: `#![no_main]` plus a runtime crate such as
              `cortex-m-rt` that sets up memory and calls your `main`.
            - an optional **global allocator**, if you want `alloc`. Many
              projects don't.
            - a **target**: `rustup target add thumbv7em-none-eabihf` (Cortex-M4F),
              a linker script giving the memory layout, and a flashing tool.

            ```rust,nowrap
            #![no_std]
            use core::panic::PanicInfo;

            #[panic_handler]
            fn panic(_info: &PanicInfo) -> ! {
                // e.g. drive outputs to a safe state, then wait for the watchdog
                loop {}
            }
            ```
            "#,
        ),
        Section::new(
            "The embedded ecosystem",
            r#"
            Beyond std, the embedded Rust ecosystem is organized in layers.
            These are crates, outside this course's std-only rule, but they're
            worth knowing by name for an interview:

            ```text
            PAC          peripheral access crate, generated from the chip's SVD
                         description: typed registers
            HAL          hardware abstraction (GPIO, SPI, CAN, timers), often
                         implementing the embedded-hal traits
            embedded-hal common traits, so drivers work on any chip
            RTIC         interrupt-driven concurrency with compile-time priorities
            Embassy      async/await executor and HALs for microcontrollers
            heapless     fixed-capacity Vec, String, queues (Lesson 33, as a crate)
            defmt        compact logging for constrained devices
            ```

            Safety-critical projects add qualified toolchains and certified
            core libraries (Lesson 39).
            "#,
        ),
    ],
    quiz: &[
        Question::new(
            "Which type is available in a `#![no_std]` crate without `alloc`?",
            &["Vec<u8>", "String", "Option<u8>", "Box<u8>"],
            2,
            "Option lives in core. Vec, String and Box need a heap allocator (alloc).",
        ),
        Question::new(
            "Why is HashMap in std rather than alloc?",
            &[
                "It's too big",
                "Its default hasher needs a random seed from the operating system",
                "It uses threads",
                "Historical accident only",
            ],
            1,
            "RandomState seeds SipHash from OS randomness. BTreeMap needs no randomness, so it's in alloc.",
        ),
        Question::new(
            "Are `std::option::Option` and `core::option::Option` the same type?",
            &[
                "No, std's version wraps core's",
                "Yes — std re-exports core",
                "Only on Linux",
                "Only in no_std crates",
            ],
            1,
            "std re-exports core and alloc items, so they're identical types reachable through two paths.",
        ),
        Question::new(
            "What must every `#![no_std]` binary provide?",
            &[
                "A main function returning ExitCode",
                "A #[panic_handler]",
                "A HashMap",
                "A thread pool",
            ],
            1,
            "Without std, nothing defines what happens on panic; the binary must supply the handler.",
        ),
        Question::new(
            "How can a no_std library still use `println!` in its tests?",
            &[
                "It can't",
                "With #![cfg_attr(not(test), no_std)], so test builds link std",
                "By re-implementing println!",
                "With unsafe",
            ],
            1,
            "The cfg_attr makes the crate no_std only for non-test builds; tests run on the host with std.",
        ),
    ],
    exercises: &[
        "Take the bit-field helpers from Lesson 31 and move them into a `#![no_std]` crate (in a scratch directory). Build it with `rustc --edition 2024 --crate-type lib`.",
        "Make that crate `#![cfg_attr(not(test), no_std)]` and add unit tests that use `println!`.",
        "List every `std::` path in one of your earlier exercises and mark which ones could be `core::` instead.",
        "Write a `core::fmt::Write` implementation over `&mut [u8]` (a cursor) so you can `write!` into any byte buffer.",
    ],
};

fn layers() {
    // ANCHOR: layers
    // The same types, reached through different crates:
    println!(
        "std Option == core Option? {}",
        TypeId::of::<std::option::Option<u8>>() == TypeId::of::<core::option::Option<u8>>()
    );
    println!(
        "std Duration == core Duration? {}",
        TypeId::of::<std::time::Duration>() == TypeId::of::<core::time::Duration>()
    );
    println!("Vec's defining crate: {}", std::any::type_name::<Vec<u8>>()); // alloc, re-exported

    // core works with no heap and no OS:
    let checksum = core::iter::successors(Some(1u32), |n| n.checked_mul(3))
        .take(5)
        .fold(0, |a, b| a ^ b);
    let size = core::mem::size_of::<core::num::NonZero<u32>>();
    println!("core-only computation: checksum {checksum}, NonZero size {size}");
    // ANCHOR_END: layers
}

fn portable() {
    // ANCHOR: portable
    /// Formats into any `core::fmt::Write` sink: a String on a server,
    /// a fixed buffer on a microcontroller. The logic doesn't care.
    fn format_reading(
        out: &mut impl core::fmt::Write,
        name: &str,
        value: i32,
        unit: &str,
    ) -> core::fmt::Result {
        write!(out, "{name}={value}{unit}")
    }

    /// A tiny cursor over a byte buffer: core-only, no allocation.
    struct SliceWriter<'a> {
        buf: &'a mut [u8],
        len: usize,
    }
    impl core::fmt::Write for SliceWriter<'_> {
        fn write_str(&mut self, s: &str) -> core::fmt::Result {
            let end = self.len.checked_add(s.len()).ok_or(core::fmt::Error)?;
            self.buf
                .get_mut(self.len..end)
                .ok_or(core::fmt::Error)?
                .copy_from_slice(s.as_bytes());
            self.len = end;
            Ok(())
        }
    }

    let mut on_server = String::new();
    format_reading(&mut on_server, "coolant", 88, "C").unwrap();

    let mut storage = [0u8; 32];
    let mut on_mcu = SliceWriter {
        buf: &mut storage,
        len: 0,
    };
    format_reading(&mut on_mcu, "coolant", 88, "C").unwrap();
    let written = on_mcu.len;

    println!("String sink:  {on_server}");
    println!(
        "buffer sink:  {}",
        core::str::from_utf8(&storage[..written]).unwrap_or("?")
    );
    let mut tiny = [0u8; 4];
    let result = format_reading(
        &mut SliceWriter {
            buf: &mut tiny,
            len: 0,
        },
        "coolant",
        88,
        "C",
    );
    println!("too small:    {result:?}");
    let _ = write!(
        on_server,
        " (std also implements core::fmt::Write for String)"
    );
    println!("{on_server}");
    // ANCHOR_END: portable
}
