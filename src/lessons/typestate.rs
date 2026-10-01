//! Lesson: Typestate & newtypes.

use std::marker::PhantomData;
use std::ops::{Add, Sub};

use crate::lesson::{Lesson, Question, Section};

pub static LESSON: Lesson = Lesson {
    id: "typestate",
    title: "Typestate & newtypes",
    summary: "Making illegal states unrepresentable: unit newtypes, typestate state machines (ignition, UDS sessions), sealed states, typed builders and capability tokens.",
    source: include_str!("typestate.rs"),
    sections: &[
        Section::new(
            "Make illegal states unrepresentable",
            r#"
            Every combination of fields your type allows is a state the program
            can reach. A struct with `engine_running: bool` and
            `ignition_on: bool` permits "engine running with ignition off",
            which is nonsense, and every function must defend against it.

            Model the states as an **enum** instead. Each variant carries only
            the data valid in that state, so the impossible combination can't
            even be written. This is the single most effective
            correctness technique in Rust, and it costs nothing at run time.
            "#,
        )
        .demo("states", illegal_states),
        Section::new(
            "Newtypes for units and identifiers",
            r#"
            `f64` speeds and `f64` distances add up without complaint. Wrap
            each in a newtype, and implement only the operations that make
            physical sense: speed plus speed is a speed, and distance divided
            by time is a speed. Then unit errors become type errors, at zero
            run-time cost. Identifiers deserve the same treatment: a `CanId`
            and a `Dtc` code are both integers, but never interchangeable.
            "#,
        )
        .demo("units", units),
        Section::new(
            "The typestate pattern",
            r#"
            An enum checks states at **run time**: you `match` and handle the
            wrong state somehow. The **typestate** pattern moves the state into
            the **type**, `Ignition<Off>` versus `Ignition<Running>`, so calling
            an operation in the wrong state doesn't compile:

            ```compile_fail,E0599
            struct Off;
            struct Running;
            struct Ignition<S> { state: S }
            impl Ignition<Running> {
                fn drive(&self) {}
            }
            let car = Ignition { state: Off };
            car.drive(); // error: no method `drive` found for `Ignition<Off>`
            ```

            Transitions take `self` by value and return the next state, so the
            old state can't be used again. State marker types are zero-sized,
            so all of this disappears at run time.
            "#,
        )
        .demo("ignition", ignition),
        Section::new(
            "States that carry data: a UDS diagnostic session",
            r#"
            Unified Diagnostic Services (UDS, ISO 14229) is how testers talk to
            ECUs. Sensitive services, such as writing calibration memory, need
            an **extended session** and a successful **security access**: the
            ECU sends a seed, and the tester must answer with the correct key.

            With typestate, `write_memory` simply doesn't exist until the key
            has been verified. The seed lives only in the state that needs it,
            and a wrong key hands back the **previous** state, ready to try
            again. The real seed-to-key algorithm is secret and per
            manufacturer. A toy one stands in here.
            "#,
        )
        .demo("uds", uds),
        Section::new(
            "Sealed states",
            r#"
            Generic code over states, `impl<S: State> Ignition<S>`, should only
            accept **our** states. Combine typestate with a sealed trait
            (Lesson 16): the marker trait requires a private supertrait, so no
            one outside the module can invent `Ignition<Turbo>`. The set of
            states is closed, documented, and checked by the compiler.
            "#,
        )
        .demo("sealed", sealed_states),
        Section::new(
            "Typed builders: required fields at compile time",
            r#"
            A plain builder checks for missing fields at run time, when
            `build()` returns an error. A **typestate builder** tracks which
            required fields have been set in its type parameters. `build()`
            exists only on `FrameBuilder<HasId, HasPayload>`, so forgetting the
            id is a compile error, while optional settings stay ordinary
            methods.
            "#,
        )
        .demo("builder", typed_builder),
        Section::new(
            "Capability tokens",
            r#"
            A **token** is a value that proves something happened. Make its
            constructor private and give it out only after a check. Then any
            function that takes the token is guaranteed to run after the check,
            with no flags to forget. Tokens are typically zero-sized, so they
            cost nothing. Think "proof of calibration unlock", "proof of
            init", or "proof we're on the control thread".
            "#,
        )
        .demo("tokens", tokens),
        Section::new(
            "Typestate or enum? A spectrum",
            r#"
            ```text
            typestate   the state follows the program's structure (API call order,
                        init sequences). Errors are compile errors. The state can't
                        come from outside at run time.
            enum        the state follows external input (messages, timers, faults).
                        You match on it, and the compiler checks the match is
                        exhaustive.
            ```

            Real systems use both. A typestate guards the **API** (you can't
            send frames before the bus is configured). An enum drives the
            **behaviour** (the gearbox reacting to whatever the driver does).
            "#,
        ),
    ],
    quiz: &[
        Question::new(
            "What does the typestate pattern turn into compile-time errors?",
            &[
                "Type mismatches in arithmetic",
                "Calling operations in a state where they are invalid",
                "Memory leaks",
                "Data races",
            ],
            1,
            "Each state is a distinct type, and methods exist only on the states where they're valid.",
        ),
        Question::new(
            "Why do typestate transitions take `self` by value?",
            &[
                "It's faster",
                "So the old state is consumed and can't be used after the transition",
                "To allow cloning",
                "Because PhantomData requires it",
            ],
            1,
            "Moving `self` makes the previous state inaccessible, so there's no 'still holding the Off handle' bug.",
        ),
        Question::new(
            "What does a zero-sized state marker like `struct Running;` cost at run time?",
            &["One byte", "A pointer", "Nothing", "A vtable"],
            2,
            "Marker types have no data. `Ignition<Running>` has the same size as `Ignition<Off>`, and the state exists only for the compiler.",
        ),
        Question::new(
            "When is a plain enum a better fit than typestate?",
            &[
                "When the state is determined by run-time input like received messages",
                "When you want compile-time checking",
                "Never",
                "When states carry data",
            ],
            0,
            "Typestate needs the state to be known at compile time. Externally driven state machines are enums.",
        ),
        Question::new(
            "How does a capability token guarantee that a check happened?",
            &[
                "It stores a bool",
                "Its constructor is private and only called after the check succeeds; functions require it as a parameter",
                "It uses unsafe",
                "It's checked at run time",
            ],
            1,
            "If the only way to obtain the token is through the check, holding one is proof.",
        ),
    ],
    exercises: &[
        "Model a door lock as typestate (`Locked`, `Unlocked`, `Open`) where `open()` exists only when unlocked and `lock()` only when closed.",
        "Add a `Programming` session to the UDS example that is reachable only from `SecurityUnlocked`, with `erase()` and `write_block()` methods.",
        "Convert a runtime-checked builder with three required fields into a typestate builder. Count the generic parameters you need.",
        "Write `Kmh`, `Seconds` and `Meters` newtypes with `Meters / Seconds = MetersPerSecond` and a conversion to `Kmh`.",
    ],
};

fn illegal_states() {
    // ANCHOR: states
    // Booleans allow nonsense combinations:
    struct VehicleFlags {
        ignition_on: bool,
        engine_running: bool, // running with ignition off?!
        rpm: u32,             // meaningful only while running
    }

    // An enum allows exactly the meaningful states:
    #[derive(Debug)]
    enum Power {
        Off,
        Accessory,
        On,
        Running { rpm: u32 },
    }

    fn dashboard(p: &Power) -> String {
        match p {
            Power::Off => "dark".into(),
            Power::Accessory => "radio only".into(),
            Power::On => "warning lamps test".into(),
            Power::Running { rpm } => format!("tachometer {rpm}"),
        }
    }
    for p in [
        Power::Off,
        Power::Accessory,
        Power::On,
        Power::Running { rpm: 780 },
    ] {
        println!("{p:?} → {}", dashboard(&p));
    }
    // ANCHOR_END: states
}

fn units() {
    // ANCHOR: units
    #[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
    struct Kmh(f64);
    #[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
    struct Meters(f64);
    #[derive(Debug, Clone, Copy)]
    struct Seconds(f64);

    impl Add for Kmh {
        type Output = Kmh;
        fn add(self, o: Kmh) -> Kmh {
            Kmh(self.0 + o.0)
        }
    }
    impl Sub for Kmh {
        type Output = Kmh;
        fn sub(self, o: Kmh) -> Kmh {
            Kmh(self.0 - o.0)
        }
    }
    impl Meters {
        fn per(self, t: Seconds) -> Kmh {
            Kmh(self.0 / t.0 * 3.6) // distance / time = speed
        }
    }

    let cruise = Kmh(100.0);
    let boost = Kmh(20.0);
    println!("{:?}", cruise + boost);
    println!("{:.1} km/h", Meters(1000.0).per(Seconds(30.0)).0);
    // let nonsense = cruise + Meters(5.0); // error: expected Kmh, found Meters
    println!("over the limit? {}", cruise - boost > Kmh(70.0));
    // ANCHOR_END: units
}

// ANCHOR: ignition
pub struct Off;
pub struct Accessory;
pub struct On;
pub struct Running;

pub struct Ignition<S> {
    odometer_km: u32,
    _state: PhantomData<S>, // zero-sized: the state exists only in the type
}

impl Ignition<Off> {
    pub fn new(odometer_km: u32) -> Self {
        Ignition {
            odometer_km,
            _state: PhantomData,
        }
    }
    pub fn turn_to_accessory(self) -> Ignition<Accessory> {
        Ignition {
            odometer_km: self.odometer_km,
            _state: PhantomData,
        }
    }
}

impl Ignition<Accessory> {
    pub fn turn_on(self) -> Ignition<On> {
        Ignition {
            odometer_km: self.odometer_km,
            _state: PhantomData,
        }
    }
}

impl Ignition<On> {
    /// Starting requires the brake pedal; failure keeps us in `On`.
    pub fn start(self, brake_pressed: bool) -> Result<Ignition<Running>, Ignition<On>> {
        if brake_pressed {
            Ok(Ignition {
                odometer_km: self.odometer_km,
                _state: PhantomData,
            })
        } else {
            Err(self)
        }
    }
}

impl Ignition<Running> {
    pub fn drive(&mut self, km: u32) {
        self.odometer_km += km;
    }
    pub fn stop(self) -> Ignition<Off> {
        Ignition {
            odometer_km: self.odometer_km,
            _state: PhantomData,
        }
    }
}

fn ignition() {
    let on = Ignition::new(12_000).turn_to_accessory().turn_on();
    let on = match on.start(false) {
        Ok(_) => unreachable!("brake was not pressed"),
        Err(still_on) => {
            println!("start refused: press the brake");
            still_on
        }
    };
    let Ok(mut running) = on.start(true) else {
        return;
    };
    running.drive(42);
    let parked = running.stop();
    println!("parked at {} km", parked.odometer_km);
    println!(
        "size_of Ignition<Off> = {} = size_of Ignition<Running> = {}",
        std::mem::size_of::<Ignition<Off>>(),
        std::mem::size_of::<Ignition<Running>>()
    );
    // parked.drive(1); // error: no method `drive` for Ignition<Off>
}
// ANCHOR_END: ignition

fn uds() {
    // ANCHOR: uds
    struct DefaultSession;
    struct ExtendedSession;
    struct SeedSent {
        seed: u32,
    }
    struct SecurityUnlocked;

    struct Ecu<S> {
        memory: [u8; 4],
        state: S,
    }

    /// A toy seed→key algorithm. Real ones are secret, per manufacturer.
    fn key_for(seed: u32) -> u32 {
        seed.rotate_left(7) ^ 0xA5A5_5A5A
    }

    impl Ecu<DefaultSession> {
        fn read_memory(&self) -> [u8; 4] {
            self.memory // reading is allowed in every session
        }
        fn enter_extended(self) -> Ecu<ExtendedSession> {
            Ecu {
                memory: self.memory,
                state: ExtendedSession,
            }
        }
    }
    impl Ecu<ExtendedSession> {
        fn request_seed(self, seed: u32) -> (u32, Ecu<SeedSent>) {
            (
                seed,
                Ecu {
                    memory: self.memory,
                    state: SeedSent { seed },
                },
            )
        }
    }
    impl Ecu<SeedSent> {
        fn send_key(self, key: u32) -> Result<Ecu<SecurityUnlocked>, Ecu<ExtendedSession>> {
            if key == key_for(self.state.seed) {
                Ok(Ecu {
                    memory: self.memory,
                    state: SecurityUnlocked,
                })
            } else {
                Err(Ecu {
                    memory: self.memory,
                    state: ExtendedSession,
                }) // back one step
            }
        }
    }
    impl Ecu<SecurityUnlocked> {
        fn write_memory(&mut self, data: [u8; 4]) {
            self.memory = data; // only reachable after a correct key
        }
    }

    let ecu = Ecu {
        memory: [1, 2, 3, 4],
        state: DefaultSession,
    };
    println!("read in default session: {:?}", ecu.read_memory());
    let (seed, waiting) = ecu.enter_extended().request_seed(0x1234_5678);
    let ecu = match waiting.send_key(0xDEAD_BEEF) {
        Ok(_) => unreachable!(),
        Err(extended) => {
            println!("wrong key: still in the extended session, locked");
            extended
        }
    };
    let (seed2, waiting) = ecu.request_seed(seed.wrapping_add(1));
    if let Ok(mut unlocked) = waiting.send_key(key_for(seed2)) {
        unlocked.write_memory([9, 9, 9, 9]);
        println!("unlocked and wrote {:?}", unlocked.memory);
    }
    // ANCHOR_END: uds
}

fn sealed_states() {
    // ANCHOR: sealed
    mod bus {
        use std::marker::PhantomData;

        mod private {
            pub trait Sealed {}
        }
        pub trait State: private::Sealed {
            const NAME: &'static str;
        }

        pub struct Configuring;
        pub struct Active;
        impl private::Sealed for Configuring {}
        impl private::Sealed for Active {}
        impl State for Configuring {
            const NAME: &'static str = "configuring";
        }
        impl State for Active {
            const NAME: &'static str = "active";
        }

        pub struct CanBus<S: State> {
            pub bitrate: u32,
            _s: PhantomData<S>,
        }
        impl<S: State> CanBus<S> {
            pub fn describe(&self) -> String {
                format!("bus {} at {} bit/s", S::NAME, self.bitrate) // generic over our states only
            }
        }
        impl CanBus<Configuring> {
            pub fn new() -> Self {
                CanBus {
                    bitrate: 500_000,
                    _s: PhantomData,
                }
            }
            pub fn activate(self) -> CanBus<Active> {
                CanBus {
                    bitrate: self.bitrate,
                    _s: PhantomData,
                }
            }
        }
        impl CanBus<Active> {
            pub fn send(&self, id: u16) -> String {
                format!("sent {id:#x}")
            }
        }
    }

    let cfg = bus::CanBus::new();
    println!("{}", cfg.describe());
    let active = cfg.activate();
    println!("{} / {}", active.describe(), active.send(0x123));
    // struct Turbo; impl bus::State for Turbo {} // error: Sealed is private
    // ANCHOR_END: sealed
}

fn typed_builder() {
    // ANCHOR: builder
    struct NoId;
    struct HasId(u16);
    struct NoPayload;
    struct HasPayload(Vec<u8>);

    struct FrameBuilder<I, P> {
        id: I,
        payload: P,
        priority: u8, // optional setting with a default
    }

    impl FrameBuilder<NoId, NoPayload> {
        fn new() -> Self {
            FrameBuilder {
                id: NoId,
                payload: NoPayload,
                priority: 3,
            }
        }
    }
    impl<P> FrameBuilder<NoId, P> {
        fn id(self, id: u16) -> FrameBuilder<HasId, P> {
            FrameBuilder {
                id: HasId(id),
                payload: self.payload,
                priority: self.priority,
            }
        }
    }
    impl<I> FrameBuilder<I, NoPayload> {
        fn payload(self, bytes: &[u8]) -> FrameBuilder<I, HasPayload> {
            FrameBuilder {
                id: self.id,
                payload: HasPayload(bytes.to_vec()),
                priority: self.priority,
            }
        }
    }
    impl<I, P> FrameBuilder<I, P> {
        fn priority(mut self, p: u8) -> Self {
            self.priority = p;
            self
        }
    }
    impl FrameBuilder<HasId, HasPayload> {
        fn build(self) -> (u16, Vec<u8>, u8) {
            (self.id.0, self.payload.0, self.priority) // only when both are set
        }
    }

    let frame = FrameBuilder::new()
        .payload(&[0x02, 0x01, 0x0C])
        .priority(1)
        .id(0x7DF)
        .build();
    println!("{frame:02X?}");
    // FrameBuilder::new().payload(&[1]).build(); // error: no method `build` (id missing)
    // FrameBuilder::new().id(1).id(2);            // error: no method `id` (already set)
    // ANCHOR_END: builder
}

fn tokens() {
    // ANCHOR: tokens
    mod calibration {
        /// Proof that calibration access was granted. Only this module can create one.
        pub struct Unlocked(());

        pub fn unlock(pin: u32) -> Option<Unlocked> {
            (pin == 2468).then_some(Unlocked(()))
        }

        pub fn set_idle_rpm(_proof: &Unlocked, rpm: u32) -> u32 {
            rpm.clamp(600, 1000) // no check needed here: the token is the check
        }
    }

    match calibration::unlock(1111) {
        Some(_) => println!("unexpected"),
        None => println!("wrong PIN: no token, so set_idle_rpm can't even be called"),
    }
    if let Some(token) = calibration::unlock(2468) {
        println!(
            "idle rpm set to {}",
            calibration::set_idle_rpm(&token, 1200)
        );
        println!("token size: {} bytes", std::mem::size_of_val(&token));
    }
    // let forged = calibration::Unlocked(()); // error: field is private
    // ANCHOR_END: tokens
}
