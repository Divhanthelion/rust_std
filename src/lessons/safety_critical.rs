//! Lesson: Safety-critical Rust in context.

use crate::lesson::{Lesson, Question, Section};
use crate::lessons::bytes_bits::crc8_sae_j1850;

pub static LESSON: Lesson = Lesson {
    id: "safety_critical",
    title: "Safety-critical Rust in context",
    summary: "ISO 26262 vocabulary, what Rust does and doesn't solve, qualified toolchains, coding guidelines, a project baseline, and defensive patterns: plausibility, voting, E2E protection, degradation and traceability.",
    source: include_str!("safety_critical.rs"),
    sections: &[
        Section::new(
            "The vocabulary",
            r#"
            Automotive software engineering has its own language. These terms
            come up in interviews:

            - **ISO 26262** is *functional safety* for road vehicles: avoiding
              unreasonable risk caused by malfunctioning E/E systems.
            - **HARA** (hazard analysis and risk assessment) rates each hazard
              by severity, exposure and controllability, and assigns an
              **ASIL**.
            - **ASIL** is a spectrum of required rigour:
              `QM → A → B → C → D`. QM means normal quality management, and
              ASIL D (steering, braking) demands the most process, analysis
              and independence.
            - **Safety goals** become **safety requirements** at system,
              hardware and software level, and each is traced to design, code
              and tests.
            - Related standards: **ISO 21448 (SOTIF)** covers hazards without a
              malfunction, such as sensor limitations. **ISO/SAE 21434** covers
              cybersecurity. **ASPICE** assesses process maturity. **MISRA**
              publishes coding guidelines. **AUTOSAR** standardizes ECU
              software architecture (Classic and Adaptive platforms).
            "#,
        ),
        Section::new(
            "What Rust solves, and what it doesn't",
            r#"
            Many defects that C coding standards fight with rules are **ruled
            out by the Rust language** itself:

            ```text
            concern (typical C guideline)        Rust
            use of uninitialized memory          compile error
            null pointer dereference             no null references; Option
            buffer overflow / out-of-bounds      bounds-checked; panics instead of UB
            use-after-free, double free          ownership + borrow checking
            data races                           Send/Sync: compile error
            implicit narrowing conversions       no implicit numeric conversions
            missing switch cases                 exhaustive match
            unchecked error codes                #[must_use] Result
            ```

            Microsoft and Google have each reported that around 70% of their
            serious security vulnerabilities came from memory-safety bugs. That
            class is exactly what safe Rust removes.

            What Rust **doesn't** give you: correct requirements, correct
            algorithms, timing guarantees, freedom from panics or deadlocks,
            or a safety case. Process, analysis and verification are still
            your job. Rust shrinks the space in which mistakes can hide.
            "#,
        ),
        Section::new(
            "Qualified toolchains",
            r#"
            ISO 26262 requires confidence in the **tools**: a compiler bug must
            not silently break safety. That's established through **tool
            qualification**. For Rust, the landscape as of 2025–2026:

            - **Ferrocene** (Ferrous Systems) is a qualified Rust toolchain
              (ISO 26262 up to ASIL D, IEC 61508, IEC 62304), built from
              upstream rustc.
            - The **Ferrocene Language Specification** (FLS) was adopted by the
              Rust project in 2025 as the basis of an official specification.
            - **AdaCore** (GNAT Pro for Rust) and **HighTec** (Rust for Infineon
              AURIX and other automotive microcontrollers) offer commercial
              toolchains aimed at certification.
            - A qualified **compiler** isn't a certified **library**.
              Certifying parts of `core` is a separate effort, and an active
              one.

            Before quoting specifics in an interview, check current release
            notes. This area moves fast.
            "#,
        ),
        Section::new(
            "Guidelines and the community",
            r#"
            The Rust Foundation's **Safety-Critical Rust Consortium**, formed in
            2024, brings together toolchain vendors, chip makers and automotive
            companies, including Woven by Toyota. It develops coding guidelines
            and works with the Rust project on what certification needs:
            specification, MC/DC coverage, and lints. MISRA has published
            guidance on how its C rules map onto Rust. AUTOSAR has studied
            Rust for its Adaptive platform, and vendors integrate Rust
            components into Classic AUTOSAR stacks.

            The practical takeaway for a Rust engineer is the same everywhere:
            **know which language guarantees you rely on, document every
            exception (`unsafe`, `allow`), and make the toolchain enforce the
            rest.**
            "#,
        ),
        Section::new(
            "A project baseline",
            r#"
            A reasonable starting point for a safety-relevant Rust crate:

            ```rust,nowrap
            //! Brake request arbitration (ASIL B). Requirements: SRS-BRK-*.
            #![forbid(unsafe_code)]            // unsafe lives in a separate, reviewed crate
            #![deny(
                missing_docs,
                clippy::unwrap_used,
                clippy::expect_used,
                clippy::panic,
                clippy::indexing_slicing,
                clippy::arithmetic_side_effects,
                clippy::as_conversions,
                clippy::float_cmp
            )]
            ```

            ```text
            rust-toolchain.toml   pin the exact (qualified) toolchain version
            Cargo.toml            rust-version (MSRV); [profile.release]
                                  overflow-checks = true, panic = "abort"
            Cargo.lock            committed; dependencies reviewed and vendored
            CI                    fmt, clippy (deny warnings), tests, Miri for unsafe,
                                  coverage (statement/branch; MC/DC where required)
            ```

            Add design rules the compiler can't check, written down and
            reviewed: no allocation after initialization (Lesson 33), bounded
            loops and no recursion (Lesson 35), deterministic collections, and
            every `unsafe` block justified with a `SAFETY:` comment (Lesson 36).
            "#,
        ),
        Section::new(
            "Defensive pattern: plausibility checks",
            r#"
            A sensor value can be well-formed and still **wrong**: a stuck
            sensor, a broken wire reading full scale, an impossible jump. Check
            each value against physics before trusting it:

            - **range**: inside the physically possible interval;
            - **rate of change**: no faster than physics allows, since a car
              can't gain 50 km/h in 10 ms;
            - **stuck-at**: identical raw values for too long while the vehicle
              is moving.

            A failed check produces a **qualified** value: the reading plus a
            status that downstream functions must respect.
            "#,
        )
        .demo("plausibility", plausibility),
        Section::new(
            "Defensive pattern: redundancy and voting",
            r#"
            For high ASILs, critical inputs are measured **redundantly**: two
            or three sensors, sometimes of different types (diversity). A
            **2-out-of-3 voter** accepts a value only when at least two
            channels agree within a tolerance, and it identifies the outlier.
            A single faulty sensor is then detected **and** tolerated. With
            only two channels, disagreement can be detected but not resolved,
            so the system must degrade.
            "#,
        )
        .demo("voting", voting),
        Section::new(
            "Defensive pattern: end-to-end protection",
            r#"
            Data crossing a bus can be corrupted, lost, repeated, delayed, or
            sent by the wrong ECU. **E2E protection** (as in AUTOSAR's E2E
            profiles) adds to each message:

            - an **alive counter** that increments every message, which detects
              loss and repetition;
            - a **CRC** computed over the payload, the counter and a **data
              ID** that is never transmitted, which detects corruption **and**
              masquerading.

            The receiver checks both and reports a status rather than just a
            value. Below is a simplified scheme in the spirit of E2E Profile 1,
            reusing Lesson 31's CRC-8 (SAE J1850). It's not byte-exact to the
            specification.
            "#,
        )
        .demo("e2e", e2e),
        Section::new(
            "Safe states and graceful degradation",
            r#"
            When faults appear, a system moves along a spectrum of
            functionality instead of failing all at once:

            ```text
            Normal  →  Degraded (reduced function, driver warned)  →  SafeState
            ```

            Transitions use **debouncing**: a fault must persist for N cycles
            before it counts, so one glitch doesn't trigger a reaction.
            Recovery uses **healing**: M good cycles in a row. Modelling the
            modes as an enum, with transitions in one pure function, makes the
            logic exhaustively matched and easy to test cycle by cycle.
            "#,
        )
        .demo("degradation", degradation),
        Section::new(
            "Traceability: requirements to tests",
            r#"
            Safety standards require **bidirectional traceability**. Every
            requirement must be verified by tests, and every test must exist
            for a requirement. A lightweight approach in plain Rust is to name
            or tag tests with requirement IDs, register them in a table, and
            generate the traceability matrix as a build artifact. The demo
            builds such a matrix and flags a requirement with no passing test.
            "#,
        )
        .demo("trace", traceability),
    ],
    quiz: &[
        Question::new(
            "Which ASIL demands the most rigour?",
            &["QM", "ASIL A", "ASIL B", "ASIL D"],
            3,
            "The spectrum runs QM, A, B, C, D. D applies to hazards with the highest severity, exposure and lowest controllability.",
        ),
        Question::new(
            "Which defect does safe Rust NOT prevent?",
            &[
                "Use-after-free",
                "Data races",
                "Deadlocks and logic errors",
                "Reading uninitialized memory",
            ],
            2,
            "Rust prevents memory-safety bugs and data races; deadlocks, wrong algorithms and wrong requirements remain your problem.",
        ),
        Question::new(
            "What does an E2E alive counter detect?",
            &[
                "Bit flips in the payload",
                "Lost or repeated messages",
                "Wrong units",
                "Slow consumers",
            ],
            1,
            "A counter that must increment by one per message reveals gaps (loss) and duplicates (repetition); the CRC covers corruption.",
        ),
        Question::new(
            "With two redundant sensors that disagree, what can a voter do?",
            &[
                "Pick the right one",
                "Detect the disagreement but not decide which is correct — degrade",
                "Average them safely",
                "Ignore it",
            ],
            1,
            "Two channels give detection only. Identifying the faulty channel needs a third (2oo3) or other diagnostics.",
        ),
        Question::new(
            "Is a qualified compiler (e.g. Ferrocene) the same as a certified standard library?",
            &[
                "Yes",
                "No — library certification is a separate effort",
                "Only for ASIL D",
                "Only for no_std",
            ],
            1,
            "Tool qualification covers the compiler. Using library code in a safety context needs its own evidence.",
        ),
    ],
    exercises: &[
        "Add a stuck-at check to the plausibility filter that only triggers while the vehicle is moving (a second input).",
        "Extend the 2oo3 voter to report which channel is faulty, and to degrade to 1oo2 behaviour once a channel is excluded.",
        "Make the E2E receiver tolerate up to one lost message (counter jump of 2) as `OkSomeLost`, and treat larger jumps as errors.",
        "Write a small requirements file (one `ID: text` per line), parse it, and have the traceability demo report requirements missing from the test table.",
    ],
};

fn plausibility() {
    // ANCHOR: plausibility
    #[derive(Debug, PartialEq)]
    enum Status {
        Valid,
        OutOfRange,
        ImplausibleJump,
    }

    struct SpeedFilter {
        last_good: Option<f64>,
        max_delta_per_cycle: f64, // physics: max acceleration × cycle time
    }

    impl SpeedFilter {
        fn check(&mut self, kmh: f64) -> (f64, Status) {
            if !(0.0..=300.0).contains(&kmh) || kmh.is_nan() {
                return (self.last_good.unwrap_or(0.0), Status::OutOfRange);
            }
            if let Some(prev) = self.last_good
                && (kmh - prev).abs() > self.max_delta_per_cycle
            {
                return (prev, Status::ImplausibleJump); // keep the last good value
            }
            self.last_good = Some(kmh);
            (kmh, Status::Valid)
        }
    }

    // 10 ms cycle; ~1 g of acceleration ≈ 0.35 km/h per cycle; allow some margin.
    let mut filter = SpeedFilter {
        last_good: None,
        max_delta_per_cycle: 1.0,
    };
    for raw in [50.0, 50.4, 50.9, 120.0, 51.3, 655.35, f64::NAN, 51.8] {
        let (value, status) = filter.check(raw);
        println!("raw {raw:>7.2} → use {value:>5.1} ({status:?})");
    }
    // ANCHOR_END: plausibility
}

fn voting() {
    // ANCHOR: voting
    #[derive(Debug)]
    enum Vote {
        Agreed(f64),
        Outlier { value: f64, faulty_channel: usize },
        NoMajority,
    }

    /// 2-out-of-3: accept when at least two channels agree within `tol`.
    /// With full agreement, the median ("mid-value select") is used.
    fn vote_2oo3(ch: [f64; 3], tol: f64) -> Vote {
        let agree = |a: f64, b: f64| (a - b).abs() <= tol;
        let pairs = [(0, 1, 2), (0, 2, 1), (1, 2, 0)];
        let agreeing: Vec<(usize, usize, usize)> = pairs
            .iter()
            .copied()
            .filter(|&(i, j, _)| agree(ch[i], ch[j]))
            .collect();
        match agreeing.as_slice() {
            [] => Vote::NoMajority,
            [(i, j, odd)] => Vote::Outlier {
                value: (ch[*i] + ch[*j]) / 2.0,
                faulty_channel: *odd,
            },
            _ => {
                let mut sorted = ch;
                sorted.sort_by(f64::total_cmp);
                Vote::Agreed(sorted[1])
            }
        }
    }

    for channels in [[80.1, 80.0, 80.2], [80.1, 80.0, 95.0], [10.0, 50.0, 90.0]] {
        println!("{channels:?} → {:?}", vote_2oo3(channels, 0.5));
    }
    // ANCHOR_END: voting
}

fn e2e() {
    // ANCHOR: e2e
    const DATA_ID: u16 = 0x0123; // agreed per message; never sent on the bus

    /// Layout: [crc, counter (low nibble), payload…]
    fn protect(counter: u8, payload: [u8; 6]) -> [u8; 8] {
        let mut frame = [0u8; 8];
        frame[1] = counter & 0x0F;
        frame[2..].copy_from_slice(&payload);
        frame[0] = crc_over(&frame);
        frame
    }

    fn crc_over(frame: &[u8; 8]) -> u8 {
        let mut input = [0u8; 9];
        input[..2].copy_from_slice(&DATA_ID.to_le_bytes()); // implicit data ID
        input[2..].copy_from_slice(&frame[1..]); //           counter + payload
        crc8_sae_j1850(&input)
    }

    #[derive(Debug, PartialEq)]
    enum E2eStatus {
        Ok,
        Repeated,
        Lost(u8),
        WrongCrc,
    }

    struct Receiver {
        last_counter: Option<u8>,
    }
    impl Receiver {
        fn check(&mut self, frame: &[u8; 8]) -> E2eStatus {
            if crc_over(frame) != frame[0] {
                return E2eStatus::WrongCrc; // corrupted, or not from our sender
            }
            let counter = frame[1] & 0x0F;
            let status = match self.last_counter {
                None => E2eStatus::Ok,
                Some(last) => match counter.wrapping_sub(last) & 0x0F {
                    0 => E2eStatus::Repeated,
                    1 => E2eStatus::Ok,
                    gap => E2eStatus::Lost(gap - 1),
                },
            };
            if status != E2eStatus::Repeated {
                self.last_counter = Some(counter);
            }
            status
        }
    }

    let payload = [0x0C, 0x80, 0x7A, 0, 0, 0];
    let mut rx = Receiver { last_counter: None };
    let f14 = protect(14, payload);
    let f15 = protect(15, payload);
    let f0 = protect(0, payload); // the 4-bit counter wraps 15 → 0
    let f3 = protect(3, payload); // 1 and 2 never arrive
    let mut corrupted = protect(4, payload);
    corrupted[3] ^= 0x10;

    for (label, frame) in [
        ("14", &f14),
        ("15", &f15),
        ("0 (wrap)", &f0),
        ("0 again", &f0),
        ("3", &f3),
        ("4 corrupted", &corrupted),
    ] {
        println!("counter {label:<12} → {:?}", rx.check(frame));
    }
    // ANCHOR_END: e2e
}

fn degradation() {
    // ANCHOR: degradation
    #[derive(Debug, Clone, Copy, PartialEq)]
    enum Mode {
        Normal,
        Degraded,
        SafeState,
    }

    struct Monitor {
        mode: Mode,
        bad_cycles: u8,
        good_cycles: u8,
    }

    const DEBOUNCE: u8 = 3; // faults must persist for 3 cycles
    const HEAL: u8 = 5; //     5 good cycles to recover from Degraded

    impl Monitor {
        /// One pure transition per cycle: easy to test exhaustively.
        fn cycle(&mut self, fault: bool) -> Mode {
            if fault {
                self.good_cycles = 0;
                self.bad_cycles = self.bad_cycles.saturating_add(1);
            } else {
                self.bad_cycles = 0;
                self.good_cycles = self.good_cycles.saturating_add(1);
            }
            self.mode = match (self.mode, self.bad_cycles, self.good_cycles) {
                (Mode::SafeState, _, _) => Mode::SafeState, // latched until service
                (Mode::Normal, b, _) if b >= DEBOUNCE => Mode::Degraded,
                (Mode::Degraded, b, _) if b >= 2 * DEBOUNCE => Mode::SafeState,
                (Mode::Degraded, _, g) if g >= HEAL => Mode::Normal,
                (mode, _, _) => mode,
            };
            self.mode
        }
    }

    let mut m = Monitor {
        mode: Mode::Normal,
        bad_cycles: 0,
        good_cycles: 0,
    };
    let faults = "..x.xxx......xxxxxx..";
    let modes: String = faults
        .chars()
        .map(|c| match m.cycle(c == 'x') {
            Mode::Normal => 'N',
            Mode::Degraded => 'D',
            Mode::SafeState => 'S',
        })
        .collect();
    println!("fault input: {faults}");
    println!("mode output: {modes}");
    println!("(N normal, D degraded, S safe state — a single glitch changes nothing)");
    // ANCHOR_END: degradation
}

fn traceability() {
    // ANCHOR: trace
    struct Requirement {
        id: &'static str,
        text: &'static str,
    }
    struct Test {
        name: &'static str,
        verifies: &'static [&'static str],
        run: fn() -> bool,
    }

    fn clamp_brake(request_pct: i32) -> u8 {
        request_pct.clamp(0, 100) as u8
    }

    let requirements = [
        Requirement {
            id: "SRS-BRK-001",
            text: "brake request is limited to 0..=100 %",
        },
        Requirement {
            id: "SRS-BRK-002",
            text: "negative requests yield 0 %",
        },
        Requirement {
            id: "SRS-BRK-003",
            text: "request changes are rate-limited",
        },
    ];
    let tests = [
        Test {
            name: "clamps_above_100",
            verifies: &["SRS-BRK-001"],
            run: || clamp_brake(140) == 100,
        },
        Test {
            name: "clamps_negative",
            verifies: &["SRS-BRK-001", "SRS-BRK-002"],
            run: || clamp_brake(-5) == 0,
        },
    ];

    println!("{:<12} {:<40} verified by", "requirement", "text");
    for req in &requirements {
        let covering: Vec<String> = tests
            .iter()
            .filter(|t| t.verifies.contains(&req.id))
            .map(|t| format!("{} ({})", t.name, if (t.run)() { "pass" } else { "FAIL" }))
            .collect();
        let verdict = if covering.is_empty() {
            "⚠ NOT VERIFIED".to_string()
        } else {
            covering.join(", ")
        };
        println!("{:<12} {:<40} {verdict}", req.id, req.text);
    }
    // ANCHOR_END: trace
}
