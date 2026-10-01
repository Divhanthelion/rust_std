//! Lesson: Modules, visibility & crates.

use crate::lesson::{Lesson, Question, Section};

pub static LESSON: Lesson = Lesson {
    id: "modules",
    title: "Modules, visibility & crates",
    summary: "Organizing code with modules, the visibility spectrum, paths and use, re-exports, files and directories, crates, the prelude and conditional compilation.",
    source: include_str!("modules.rs"),
    sections: &[
        Section::new(
            "Modules group and hide",
            r#"
            A **module** is a named namespace declared with `mod name { … }`.
            Modules nest, forming a tree whose root is the crate. Everything in
            a module is **private by default**, visible only inside that module
            and its children. You opt in to exposure with `pub`.

            Privacy is how a module protects invariants: outsiders can only use
            the API you chose to publish. The demo below defines a small module
            tree inside a function. Real projects usually put each module in its
            own file.
            "#,
        )
        .demo("basics", basics),
        Section::new(
            "The visibility spectrum",
            r#"
            Visibility isn't just "private or public":

            ```text
            (nothing)          private: this module and its descendants
            pub(self)          the same as nothing, spelled out
            pub(super)         the parent module too
            pub(in path)       up to a named ancestor module
            pub(crate)         anywhere in this crate, but not outside it
            pub                anyone, including other crates
            ```

            `pub(crate)` is the workhorse for internal APIs. Code shared
            across your crate stays out of your public, semver-stable surface.
            "#,
        )
        .demo("visibility", visibility),
        Section::new(
            "Paths: crate, self and super",
            r#"
            Items are named by **paths**:

            - `crate::a::b` is absolute, starting from the crate root;
            - `self::b` is relative to the current module;
            - `super::b` starts from the parent module, and can be chained
              (`super::super::x`);
            - `std::…`, or any other crate name, starts from an external crate.

            Prefer `crate::` paths for things far away, and `super::` within a
            closely related cluster of modules. That is exactly the
            `use super::*;` idiom in test modules.
            "#,
        )
        .demo("paths", paths),
        Section::new(
            "use: bringing names into scope",
            r#"
            `use` creates a local alias for a path. Forms you'll see everywhere:

            ```rust
            use std::collections::HashMap;           // one item
            use std::io::{self, Read, Write};        // several, plus the module itself
            use std::fmt::Result as FmtResult;       // rename to avoid a clash
            use std::sync::atomic::*;                // glob (use sparingly)
            ```

            The idiom: for **functions**, import the parent module and call
            `io::stdin()`, so readers see where the function comes from. For
            **types, traits and enums**, import the item itself (`HashMap`).
            Trait methods are only callable when the trait is in scope; that's
            why you sometimes see `use std::io::Write as _;`, which imports
            the trait's methods without its name.
            "#,
        )
        .demo("use", use_demo),
        Section::new(
            "Re-exports with pub use",
            r#"
            `pub use inner::Thing;` makes `Thing` available at the current
            module's path as well. Libraries use this to present a **flat,
            stable API** while keeping a deep internal structure that they can
            reorganize freely. Users write `mylib::Frame` rather than
            `mylib::protocol::can::frame::Frame`.
            "#,
        )
        .demo("reexport", reexports),
        Section::new(
            "Visibility of struct fields and enum variants",
            r#"
            A `pub struct` can still have private fields. Outsiders can then
            hold and pass the struct around, but can't construct it with a
            literal or touch its fields. They must go through your
            constructors, which is how invariants are kept (Lesson 10).

            Enum variants, by contrast, are always as public as the enum. A
            variant can't be hidden, because matching must be exhaustive.
            "#,
        )
        .demo("fields", field_visibility),
        Section::new(
            "Modules in files and directories",
            r#"
            `mod lessons;` (with a semicolon) tells the compiler to load the
            module body from a file:

            ```text
            src/
            ├── main.rs          binary crate root
            ├── lib.rs           library crate root: `pub mod cli; pub mod lessons;`
            ├── cli.rs           → crate::cli
            └── lessons/
                ├── mod.rs       → crate::lessons  (or lessons.rs next to the folder)
                ├── hello.rs     → crate::lessons::hello
                └── modules.rs   → crate::lessons::modules  ← this file
            ```

            This very program is organized this way. Run `rust_std code
            modules` to see this lesson's source, and look at `src/lib.rs`
            in the repository. The module tree is declared explicitly. Files
            that no `mod` statement mentions are simply not compiled.
            "#,
        ),
        Section::new(
            "Crates, packages and the prelude",
            r#"
            A **crate** is a compilation unit, either a binary (with `main`) or
            a library. A **package** (one `Cargo.toml`) can contain one library
            crate and any number of binaries. This project has both: `lib.rs`
            holds the logic so that tests can use it, and `main.rs` is a
            three-line wrapper. **Workspaces** group several packages that
            share a lockfile and build directory.

            The **prelude** is a small set of names imported into every module
            automatically: `Option`, `Some`, `Result`, `Vec`, `String`, `Box`,
            `ToString`, `Iterator`, `Drop`, `Clone` and others. In edition 2024
            `Future` and `IntoFuture` joined it. Everything else needs a `use`.

            This course uses no external crates, but in real projects
            crates.io and `cargo add` are how you reuse code. Know std
            well, then reach for a crate when it clearly earns its place.
            "#,
        ),
        Section::new(
            "Conditional compilation",
            r#"
            `#[cfg(...)]` includes an item only when a condition holds at
            compile time: `#[cfg(test)]`, `#[cfg(target_os = "linux")]`,
            `#[cfg(feature = "can-fd")]`, `#[cfg(debug_assertions)]`. The
            macro `cfg!(...)` evaluates the same conditions to a `bool` for use
            in ordinary code. Since Rust 1.95, `cfg_select!` chooses between
            several alternatives, like a `match` on configuration.

            Excluded code isn't type-checked for the current target, so CI
            should build every configuration you ship.
            "#,
        )
        .demo("cfg", conditional),
    ],
    quiz: &[
        Question::new(
            "An item declared without any `pub` inside `mod a` is visible…",
            &[
                "Everywhere",
                "Only inside `a` and its child modules",
                "Only in the parent of `a`",
                "Only in the same file",
            ],
            1,
            "Private items are visible to the module they're in and all its descendants.",
        ),
        Question::new(
            "What does `pub(crate)` mean?",
            &[
                "Public to other crates",
                "Visible anywhere within the current crate only",
                "Visible to the parent module",
                "Private",
            ],
            1,
            "pub(crate) exposes an item crate-wide without making it part of the public API.",
        ),
        Question::new(
            "Why might a library use `pub use internal::deep::Frame;` at its root?",
            &[
                "To make Frame private",
                "To present a short, stable path while keeping internal layout free to change",
                "To copy the type",
                "It's required for every public type",
            ],
            1,
            "Re-exports decouple the public API from the internal module structure.",
        ),
        Question::new(
            "A `pub struct S { x: i32 }` defined in another module: can you write `S { x: 1 }`?",
            &[
                "Yes, the struct is pub",
                "No, field x is private",
                "Only with unsafe",
                "Only in tests",
            ],
            1,
            "Each field has its own visibility. A struct literal needs access to every field.",
        ),
        Question::new(
            "Where does `mod sensors;` in `src/lib.rs` look for the module body?",
            &[
                "src/sensors.rs or src/sensors/mod.rs",
                "Cargo.toml",
                "Any file containing `sensors`",
                "lib/sensors.rs",
            ],
            0,
            "File modules map to `name.rs` or `name/mod.rs` next to the parent module's file.",
        ),
    ],
    exercises: &[
        "Create `mod vehicle { pub mod powertrain { … } mod internal { … } }` where `internal` is used by `powertrain` but invisible outside `vehicle`. Which visibility does the shared helper need?",
        "Split a single-file program into `main.rs`, `lib.rs` and two modules in separate files. Move a test module into each.",
        "Add a `pub use` re-export so callers write `crate::Frame` instead of `crate::protocol::frame::Frame`.",
        "Use `cfg!(target_os = \"…\")` to print a platform-specific path for a log directory.",
    ],
};

fn basics() {
    // ANCHOR: basics
    mod sensors {
        pub fn read_all() -> Vec<f64> {
            vec![calibrate(20.1), calibrate(19.8)]
        }

        fn calibrate(raw: f64) -> f64 {
            // private: an implementation detail
            raw + OFFSET
        }

        const OFFSET: f64 = 0.25;

        pub mod diagnostics {
            pub fn self_test() -> bool {
                super::calibrate(0.0) > 0.0 // children can see private parents' items
            }
        }
    }

    println!("readings {:?}", sensors::read_all());
    println!("self test passed: {}", sensors::diagnostics::self_test());
    // sensors::calibrate(1.0); // error: function `calibrate` is private
    // ANCHOR_END: basics
}

fn visibility() {
    // ANCHOR: visibility
    mod vehicle {
        pub mod powertrain {
            pub fn start() -> String {
                format!(
                    "powertrain started (fuel map v{})",
                    super::shared::fuel_map_version()
                )
            }
            pub(super) fn torque_limit() -> u32 {
                350 // visible in `vehicle`, not outside it
            }
        }

        mod shared {
            pub(in crate::lessons::modules) fn fuel_map_version() -> u32 {
                7 // visible up to this lesson's module
            }
        }

        pub fn summary() -> String {
            format!(
                "{}; torque limit {} Nm",
                powertrain::start(),
                powertrain::torque_limit()
            )
        }

        pub(crate) fn internal_id() -> u32 {
            0xBEEF // usable anywhere in this crate
        }
    }

    println!("{}", vehicle::summary());
    println!("crate-visible id {:#x}", vehicle::internal_id());
    // vehicle::powertrain::torque_limit(); // error: private to `vehicle`
    // ANCHOR_END: visibility
}

// ANCHOR: paths
mod garage {
    pub mod tools {
        pub fn wrench() -> &'static str {
            "wrench"
        }
        pub fn kit() -> String {
            // relative paths: self:: and super::
            format!("{} + {}", self::wrench(), super::spare_parts::tire())
        }
    }
    pub mod spare_parts {
        pub fn tire() -> &'static str {
            "tire"
        }
    }
}

fn paths() {
    // absolute path from the crate root:
    println!("{}", crate::lessons::modules::garage::tools::kit());
    // relative path from the current module:
    println!("{}", garage::spare_parts::tire());
}
// ANCHOR_END: paths

fn use_demo() {
    // ANCHOR: use
    use std::collections::{BTreeMap, HashMap as Map}; // nested + renamed
    use std::fmt::Write as _; // trait methods only, no name in scope
    use std::io; //                 a module: call io::stdout()

    let mut m: Map<&str, i32> = Map::new();
    m.insert("a", 1);
    let sorted: BTreeMap<_, _> = m.iter().collect();

    let mut s = String::new();
    write!(s, "{sorted:?}").unwrap(); // needs fmt::Write in scope
    println!("{s}");

    use io::Write as _;
    let mut out = io::stdout().lock();
    writeln!(out, "written through io::stdout()").unwrap();
    // ANCHOR_END: use
}

fn reexports() {
    // ANCHOR: reexport
    mod mylib {
        mod protocol {
            pub mod can {
                #[derive(Debug)]
                pub struct Frame {
                    pub id: u16,
                }
            }
        }
        pub use protocol::can::Frame; // flat public path
    }

    let f = mylib::Frame { id: 0x7E8 };
    println!("{f:?} (id {:#x})", f.id);
    // mylib::protocol::can::Frame — not reachable: `protocol` is private
    // ANCHOR_END: reexport
}

fn field_visibility() {
    // ANCHOR: fields
    mod bus {
        pub struct Message {
            pub id: u16,      //    readable and writable by anyone
            payload: Vec<u8>, // private: only via methods
        }
        impl Message {
            pub fn new(id: u16, payload: &[u8]) -> Option<Message> {
                (payload.len() <= 8).then(|| Message {
                    id,
                    payload: payload.to_vec(),
                })
            }
            pub fn len(&self) -> usize {
                self.payload.len()
            }
        }

        #[derive(Debug)]
        pub enum Priority {
            High, // variants are always as visible as the enum
            Low,
        }
    }

    let m = bus::Message::new(0x100, &[1, 2, 3]).unwrap();
    println!("id {:#x}, {} bytes", m.id, m.len());
    println!("too long → {}", bus::Message::new(1, &[0; 9]).is_none());
    println!("{:?} {:?}", bus::Priority::High, bus::Priority::Low);
    // bus::Message { id: 1, payload: vec![] }; // error: field `payload` is private
    // ANCHOR_END: fields
}

fn conditional() {
    // ANCHOR: cfg
    #[cfg(target_os = "linux")]
    fn platform() -> &'static str {
        "Linux"
    }
    #[cfg(target_os = "windows")]
    fn platform() -> &'static str {
        "Windows"
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    fn platform() -> &'static str {
        "something else"
    }

    println!("compiled for {} ({} bit pointers)", platform(), usize::BITS);
    println!("debug assertions on? {}", cfg!(debug_assertions));
    println!("running under `cargo test`? {}", cfg!(test));
    println!("little-endian target? {}", cfg!(target_endian = "little"));

    let separator = cfg_select! {
        windows => { '\\' }
        _ => { '/' }
    };
    println!("path separator {separator:?}");
    // ANCHOR_END: cfg
}
