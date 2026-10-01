//! Lesson: Standard traits & operators.

use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::{Add, AddAssign, Deref, DerefMut, Index, IndexMut, Mul, Neg, Sub};
use std::str::FromStr;

use crate::lesson::{Lesson, Question, Section};

pub static LESSON: Lesson = Lesson {
    id: "std_traits",
    title: "Standard traits & operators",
    summary: "Implementing Display, Debug, PartialEq/Eq, Ord, Hash, Default, From/TryFrom, FromStr, AsRef/Borrow, Deref, operator traits, Sum and FromIterator.",
    source: include_str!("std_traits.rs"),
    sections: &[
        Section::new(
            "Display by hand",
            r#"
            `Display` is the user-facing format behind `{}` and `to_string()`.
            Implement `fmt(&self, f: &mut Formatter) -> fmt::Result` and write
            into `f` with `write!`.

            The formatter carries the caller's flags. `f.alternate()` is true
            for `{:#}`, `f.precision()` reports `{:.2}`, and `f.pad(s)` honours
            width and alignment for text. Respecting these makes your type
            behave like a built-in inside tables. Implementing `Display` gives
            you `to_string()` for free through a blanket impl.
            "#,
        )
        .demo("display", display),
        Section::new(
            "Debug by hand",
            r#"
            Deriving `Debug` is usually right. Implement it by hand to hide
            secrets, skip noisy fields, or show a more useful form.
            `f.debug_struct("Name").field("x", &self.x).finish()` builds output
            that cooperates with `{:#?}` pretty-printing. Use
            `finish_non_exhaustive()` to show that fields were omitted.
            "#,
        )
        .demo("debug", debug),
        Section::new(
            "Equality: PartialEq and Eq",
            r#"
            `PartialEq` provides `==` and `!=`. `Eq` is a marker adding the
            promise that equality is **reflexive** (`a == a` always). Floats
            break that promise because of NaN, so they're only `PartialEq`.

            Implement `PartialEq` by hand when "equal" means something other
            than "all fields equal", for example ids compared
            case-insensitively. You can also compare **different** types:
            `impl PartialEq<Meters> for Feet`.
            "#,
        )
        .demo("eq", equality),
        Section::new(
            "Ordering: PartialOrd, Ord and sorting",
            r#"
            `Ord` gives a **total** order: `cmp` returns `Ordering::Less`,
            `Equal` or `Greater`, and it powers `sort`, `max`, `BTreeMap` keys
            and binary search. Build multi-key comparisons with
            `a.cmp(&b).then_with(|| …)`. When you implement `Ord`, implement
            `PartialOrd` as `Some(self.cmp(other))` so the two always agree.

            Often you don't need an `Ord` impl at all: `sort_by_key`,
            `sort_by` and `std::cmp::Reverse` express an ordering at the call
            site.
            "#,
        )
        .demo("ord", ordering),
        Section::new(
            "Hash and the Eq/Hash contract",
            r#"
            Types used as `HashMap` keys or in a `HashSet` need `Hash` and `Eq`,
            and the two must agree: **if `a == b`, then `hash(a) == hash(b)`**.
            Break that rule and lookups silently fail. If you customize
            `PartialEq`, for example to be case-insensitive, customize `Hash`
            the same way. This is a classic interview question.
            "#,
        )
        .demo("hash", hashing),
        Section::new(
            "Default",
            r#"
            `Default::default()` produces a sensible starting value. Deriving
            it uses each field's default: zero, `false`, an empty collection,
            `None`. Implement it by hand when "empty" isn't right, such as a
            default bitrate of 500 kbit/s. Combined with struct update syntax,
            `Config { verbose: true, ..Default::default() }` reads like named
            arguments. Enums can derive it by marking one variant `#[default]`.
            "#,
        )
        .demo("default", defaults),
        Section::new(
            "From, Into, TryFrom and TryInto",
            r#"
            `From<T> for U` defines an infallible conversion. You get
            `Into<U> for T` free through a blanket impl, so implement `From`
            and use whichever reads better at the call site. `TryFrom` is the
            fallible version, with an associated `Error` type, and gives you
            `TryInto`.

            These traits are also how `?` converts errors (Lesson 14), and how
            functions accept "anything convertible", as in
            `fn new(name: impl Into<String>)`.
            "#,
        )
        .demo("from", conversions),
        Section::new(
            "FromStr: parsing your own types",
            r#"
            Implement `FromStr` and `str::parse` works for your type:
            `"P0301".parse::<Dtc>()`. Pair it with `Display` so that parsing
            and printing round-trip. That round-trip makes a natural property
            test (Lesson 28).
            "#,
        )
        .demo("fromstr", from_str),
        Section::new(
            "AsRef and Borrow",
            r#"
            `AsRef<T>` is a cheap reference-to-reference conversion. It's used
            in generic parameters to accept many argument types: std's
            `File::open<P: AsRef<Path>>` takes `&str`, `String`, `Path` and
            `PathBuf`.

            `Borrow<T>` is stricter. It promises that the borrowed form
            hashes and compares exactly like the owned form. That's why a
            `HashMap<String, V>` can be searched with a `&str` key without
            allocating a `String`.
            "#,
        )
        .demo("asref", as_ref_borrow),
        Section::new(
            "Deref and DerefMut",
            r#"
            `Deref` customizes `*value` and enables **deref coercion**: a
            `&Wrapper` silently becomes a `&Target` where needed, and methods of
            the target are callable on the wrapper. That's how `String` derefs
            to `str` and `Box<T>` to `T`.

            Implement it for **smart-pointer-like** types only. Using `Deref`
            to fake inheritance ("my struct derefs to its inner struct") is a
            known anti-pattern. It confuses readers and leaks the inner type's
            whole API.
            "#,
        )
        .demo("deref", deref),
        Section::new(
            "Operator overloading",
            r#"
            Operators are traits in `std::ops`: `Add` (`+`), `Sub`, `Mul`,
            `Div`, `Rem`, `Neg` (unary `-`), `Not` (`!`), the bit operators,
            the `…Assign` variants (`+=`), and `Index`/`IndexMut` (`[]`). Each
            has an associated `Output` type, and the right-hand side can be a
            different type, such as a vector times a scalar.

            Overload only where the meaning is obvious: vectors, matrices,
            money, units. Operators should be cheap and must not surprise.
            "#,
        )
        .demo("ops", operators),
        Section::new(
            "Sum, FromIterator and Extend",
            r#"
            Implement `Sum` and your type works with `iter.sum()`. Implement
            `FromIterator` and `collect()` can build your collection. `Extend`
            adds `extend(iter)`, and `IntoIterator` (Lesson 19) makes your
            collection work in `for` loops. Together they let a custom type
            fit into iterator pipelines like a standard one.
            "#,
        )
        .demo("iter_traits", iterator_traits),
    ],
    quiz: &[
        Question::new(
            "If you implement `PartialEq` case-insensitively for a key type, what else must change?",
            &[
                "Nothing",
                "Hash must also ignore case, so equal keys hash equally",
                "Ord must be derived",
                "Display",
            ],
            1,
            "The contract `a == b ⇒ hash(a) == hash(b)` must hold, otherwise HashMap lookups miss.",
        ),
        Question::new(
            "You implement `From<Celsius> for Fahrenheit`. What else do you get for free?",
            &[
                "Nothing",
                "Into<Fahrenheit> for Celsius",
                "From<Fahrenheit> for Celsius",
                "TryFrom in both directions",
            ],
            1,
            "A blanket impl provides `Into<U> for T` whenever `From<T> for U` exists. (TryFrom is also derived from From, for the same direction.)",
        ),
        Question::new(
            "Why can a `HashMap<String, u32>` be queried with `map.get(\"key\")`?",
            &[
                "&str is converted to String automatically",
                "String: Borrow<str>, which guarantees identical hashing and equality",
                "HashMap stores &str internally",
                "Deref coercion",
            ],
            1,
            "`get` accepts any `Q` where the key type implements `Borrow<Q>`. String borrows as str with the same Hash and Eq.",
        ),
        Question::new(
            "Which use of Deref is considered an anti-pattern?",
            &[
                "A smart pointer type",
                "A newtype around Vec acting as a collection",
                "Emulating inheritance by deref'ing a struct to its 'parent' struct",
                "String to str",
            ],
            2,
            "Deref is for pointer-like types. As fake inheritance it surprises readers and exposes the whole inner API.",
        ),
        Question::new(
            "What does `impl Mul<f64> for Vec2 { type Output = Vec2; … }` enable?",
            &["vec * vec", "vec * 2.0", "2.0 * vec", "vec *= 2.0"],
            1,
            "`Mul<Rhs>` on the left operand's type. `2.0 * vec` would need `impl Mul<Vec2> for f64`, and `*=` needs `MulAssign`.",
        ),
    ],
    exercises: &[
        "Implement Display for a `Duration`-like struct so `{}` prints `1h 02m 03s` and `{:#}` prints `3723 s`.",
        "Create `struct CaseInsensitive(String)` with consistent PartialEq, Eq and Hash, and prove it works as a HashSet key.",
        "Implement `Add`, `Sub`, `Mul<f64>`, `Neg` and `AddAssign` for `Vec3`, plus `Sum` so a Vec of forces can be totalled with `.sum()`.",
        "Implement `FromStr` and `Display` for a semantic version `1.2.3`. Write a loop checking that `v.to_string().parse() == Ok(v)` for several versions.",
    ],
};

fn display() {
    // ANCHOR: display
    struct Temperature {
        celsius: f64,
    }
    impl fmt::Display for Temperature {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let precision = f.precision().unwrap_or(1);
            if f.alternate() {
                write!(f, "{:.*} °F", precision, self.celsius * 9.0 / 5.0 + 32.0)
            } else {
                write!(f, "{:.*} °C", precision, self.celsius)
            }
        }
    }

    struct Label(&'static str);
    impl fmt::Display for Label {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.pad(self.0) // honours width and alignment
        }
    }

    let t = Temperature { celsius: 21.456 };
    println!("{t}  |  {t:.2}  |  {t:#}");
    println!(
        "[{:>8}] [{:<8}] [{:^8}]",
        Label("oil"),
        Label("fuel"),
        Label("tire")
    );
    let s: String = t.to_string(); // free, via Display
    println!("to_string → {s:?}");
    // ANCHOR_END: display
}

fn debug() {
    // ANCHOR: debug
    struct Credentials {
        user: String,
        token: String,
        retries: u8,
    }
    impl fmt::Debug for Credentials {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_struct("Credentials")
                .field("user", &self.user)
                .field("token", &"<redacted>") // never log secrets
                .finish_non_exhaustive() // `retries` omitted, shown as ..
        }
    }

    let c = Credentials {
        user: "svc-ota".into(),
        token: "s3cr3t".into(),
        retries: 3,
    };
    println!("{c:?}");
    println!("{c:#?}");
    println!("(token length {}, retries {})", c.token.len(), c.retries);
    // ANCHOR_END: debug
}

fn equality() {
    // ANCHOR: eq
    #[derive(Debug)]
    struct PartNumber(String);

    impl PartialEq for PartNumber {
        fn eq(&self, other: &Self) -> bool {
            self.0.eq_ignore_ascii_case(&other.0)
        }
    }
    impl Eq for PartNumber {} // our equality is reflexive

    let a = PartNumber("90915-YZZD4".into());
    let b = PartNumber("90915-yzzd4".into());
    println!("{a:?} == {b:?}? {}", a == b);

    // Comparing different types:
    struct Meters(f64);
    struct Feet(f64);
    impl PartialEq<Feet> for Meters {
        fn eq(&self, other: &Feet) -> bool {
            (self.0 - other.0 * 0.3048).abs() < 1e-9
        }
    }
    println!("3.048 m == 10 ft? {}", Meters(3.048) == Feet(10.0));
    // ANCHOR_END: eq
}

fn ordering() {
    // ANCHOR: ord
    use std::cmp::Reverse;

    #[derive(Debug, PartialEq, Eq)]
    struct Fault {
        severity: u8,
        code: String,
    }

    // Highest severity first; ties broken alphabetically by code.
    impl Ord for Fault {
        fn cmp(&self, other: &Self) -> Ordering {
            other
                .severity
                .cmp(&self.severity)
                .then_with(|| self.code.cmp(&other.code))
        }
    }
    impl PartialOrd for Fault {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other)) // always consistent with Ord
        }
    }

    let mut faults = vec![
        Fault {
            severity: 2,
            code: "U0100".into(),
        },
        Fault {
            severity: 5,
            code: "P0301".into(),
        },
        Fault {
            severity: 2,
            code: "B1234".into(),
        },
    ];
    faults.sort();
    for f in &faults {
        println!("{} {}", f.severity, f.code);
    }

    // Orderings at the call site, without an Ord impl:
    let mut speeds = vec![88, 120, 45, 101];
    speeds.sort_by_key(|&s| Reverse(s));
    println!("descending {speeds:?}");
    let words = ["brake", "ECU", "airbag"];
    let longest = words.iter().max_by_key(|w| w.len());
    println!("longest {longest:?}, cmp(3, 7) = {:?}", 3.cmp(&7));
    // ANCHOR_END: ord
}

fn hashing() {
    // ANCHOR: hash
    #[derive(Debug)]
    struct Vin(String);

    impl PartialEq for Vin {
        fn eq(&self, other: &Self) -> bool {
            self.0.eq_ignore_ascii_case(&other.0)
        }
    }
    impl Eq for Vin {}

    impl Hash for Vin {
        fn hash<H: Hasher>(&self, state: &mut H) {
            // Hash exactly what eq compares: the case-folded bytes.
            for byte in self.0.bytes() {
                state.write_u8(byte.to_ascii_uppercase());
            }
        }
    }

    let mut seen = HashSet::new();
    for vin in [
        "JTDKN3DU0A0012345",
        "jtdkn3du0a0012345",
        "JTDKB20U093123456",
    ] {
        let fresh = seen.insert(Vin(vin.to_string()));
        println!("{vin}: {}", if fresh { "new" } else { "duplicate" });
    }
    println!("{} unique vehicles", seen.len());
    // ANCHOR_END: hash
}

fn defaults() {
    // ANCHOR: default
    #[derive(Debug)]
    struct BusConfig {
        bitrate: u32,
        listen_only: bool,
        filters: Vec<u32>,
    }
    impl Default for BusConfig {
        fn default() -> Self {
            BusConfig {
                bitrate: 500_000,
                listen_only: false,
                filters: Vec::new(),
            }
        }
    }

    #[derive(Debug, Default)]
    enum Mode {
        #[default]
        Normal,
        Diagnostic,
    }

    let config = BusConfig {
        listen_only: true,
        ..Default::default()
    };
    println!("{config:?}");
    println!("{:?} / {:?}", Mode::default(), Mode::Diagnostic);
    println!(
        "defaults: {:?} {:?} {:?}",
        i32::default(),
        String::default(),
        Option::<u8>::default()
    );
    // ANCHOR_END: default
}

fn conversions() {
    // ANCHOR: from
    #[derive(Debug, Clone, Copy)]
    struct Celsius(f64);
    #[derive(Debug, Clone, Copy)]
    struct Fahrenheit(f64);

    impl From<Celsius> for Fahrenheit {
        fn from(c: Celsius) -> Self {
            Fahrenheit(c.0 * 9.0 / 5.0 + 32.0)
        }
    }

    let boiling = Celsius(100.0);
    let f1 = Fahrenheit::from(boiling);
    let f2: Fahrenheit = boiling.into(); // Into comes free
    println!("{f1:?} {f2:?}");

    #[derive(Debug)]
    struct EvenNumber(u32);
    impl TryFrom<u32> for EvenNumber {
        type Error = String;
        fn try_from(n: u32) -> Result<Self, Self::Error> {
            if n % 2 == 0 {
                Ok(EvenNumber(n))
            } else {
                Err(format!("{n} is odd"))
            }
        }
    }
    println!(
        "{:?} {:?}",
        EvenNumber::try_from(8),
        EvenNumber::try_from(7)
    );
    let attempt: Result<EvenNumber, _> = 10u32.try_into();
    println!("{attempt:?}");

    // `impl Into<String>` accepts &str and String alike:
    fn label(name: impl Into<String>) -> String {
        let mut s = name.into();
        s.push(':');
        s
    }
    println!("{} {}", label("rpm"), label(String::from("temp")));
    // ANCHOR_END: from
}

fn from_str() {
    // ANCHOR: fromstr
    #[derive(Debug, PartialEq, Clone, Copy)]
    struct Dtc {
        system: char,
        code: u16,
    }

    impl FromStr for Dtc {
        type Err = String;
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let mut chars = s.chars();
            let system = chars.next().ok_or("empty code")?;
            if !matches!(system, 'P' | 'C' | 'B' | 'U') {
                return Err(format!("unknown system {system:?}"));
            }
            let digits = chars.as_str();
            if digits.len() != 4 {
                return Err(format!("expected 4 hex digits, got {digits:?}"));
            }
            let code = u16::from_str_radix(digits, 16).map_err(|e| e.to_string())?;
            Ok(Dtc { system, code })
        }
    }
    impl fmt::Display for Dtc {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}{:04X}", self.system, self.code)
        }
    }

    for text in ["P0301", "U0100", "X1234", "P03"] {
        match text.parse::<Dtc>() {
            Ok(dtc) => println!(
                "{text} → {dtc:?} → round-trip {dtc} ({})",
                dtc.to_string() == text
            ),
            Err(e) => println!("{text} → error: {e}"),
        }
    }
    // ANCHOR_END: fromstr
}

fn as_ref_borrow() {
    // ANCHOR: asref
    use std::path::{Path, PathBuf};

    fn extension_of<P: AsRef<Path>>(path: P) -> Option<String> {
        let p: &Path = path.as_ref();
        p.extension().map(|e| e.to_string_lossy().into_owned())
    }
    println!("{:?}", extension_of("log.txt")); //                &str
    println!("{:?}", extension_of(String::from("cal.bin"))); //  String
    println!("{:?}", extension_of(PathBuf::from("/etc/can.conf"))); // PathBuf

    fn byte_count<B: AsRef<[u8]>>(data: B) -> usize {
        data.as_ref().len()
    }
    println!(
        "{} {} {}",
        byte_count("héllo"),
        byte_count(vec![1, 2]),
        byte_count([0u8; 4])
    );

    // Borrow: look up String keys with &str, no allocation.
    let mut limits: HashMap<String, u32> = HashMap::new();
    limits.insert("speed".to_string(), 180);
    println!("speed limit {:?}", limits.get("speed"));
    // ANCHOR_END: asref
}

fn deref() {
    // ANCHOR: deref
    /// A Vec that refuses to grow beyond a fixed number of samples.
    struct SampleLog {
        samples: Vec<u16>,
        max: usize,
    }
    impl SampleLog {
        fn record(&mut self, s: u16) -> bool {
            if self.samples.len() < self.max {
                self.samples.push(s);
                true
            } else {
                false
            }
        }
    }
    // Read-only slice access is safe to expose wholesale...
    impl Deref for SampleLog {
        type Target = [u16];
        fn deref(&self) -> &[u16] {
            &self.samples
        }
    }
    // ...and so is in-place mutation (it can't change the length).
    impl DerefMut for SampleLog {
        fn deref_mut(&mut self) -> &mut [u16] {
            &mut self.samples
        }
    }

    let mut log = SampleLog {
        samples: Vec::new(),
        max: 3,
    };
    for s in [512, 530, 498, 600] {
        if !log.record(s) {
            println!("log full, dropped {s}");
        }
    }
    // Slice methods through Deref:
    println!(
        "len {} first {:?} max {:?}",
        log.len(),
        log.first(),
        log.iter().max()
    );
    log.sort(); // DerefMut → <[u16]>::sort
    println!("sorted {:?}", &log[..]);

    fn average(values: &[u16]) -> f64 {
        values.iter().map(|&v| v as f64).sum::<f64>() / values.len() as f64
    }
    println!("average {:.1}", average(&log)); // &SampleLog coerces to &[u16]
    // ANCHOR_END: deref
}

fn operators() {
    // ANCHOR: ops
    #[derive(Debug, Clone, Copy, PartialEq)]
    struct Vec2 {
        x: f64,
        y: f64,
    }

    impl Add for Vec2 {
        type Output = Vec2;
        fn add(self, o: Vec2) -> Vec2 {
            Vec2 {
                x: self.x + o.x,
                y: self.y + o.y,
            }
        }
    }
    impl Sub for Vec2 {
        type Output = Vec2;
        fn sub(self, o: Vec2) -> Vec2 {
            Vec2 {
                x: self.x - o.x,
                y: self.y - o.y,
            }
        }
    }
    impl Mul<f64> for Vec2 {
        type Output = Vec2;
        fn mul(self, k: f64) -> Vec2 {
            Vec2 {
                x: self.x * k,
                y: self.y * k,
            }
        }
    }
    impl Neg for Vec2 {
        type Output = Vec2;
        fn neg(self) -> Vec2 {
            Vec2 {
                x: -self.x,
                y: -self.y,
            }
        }
    }
    impl AddAssign for Vec2 {
        fn add_assign(&mut self, o: Vec2) {
            self.x += o.x;
            self.y += o.y;
        }
    }

    let position = Vec2 { x: 1.0, y: 2.0 };
    let velocity = Vec2 { x: 0.5, y: -0.25 };
    let mut p = position + velocity * 4.0;
    p += -velocity;
    println!("{:?}  displacement {:?}", p, p - position);

    // Index / IndexMut: [] on your own type.
    struct Wheels([f64; 4]);
    #[derive(Clone, Copy)]
    enum Corner {
        FrontLeft,
        FrontRight,
        RearLeft,
        RearRight,
    }
    impl Index<Corner> for Wheels {
        type Output = f64;
        fn index(&self, c: Corner) -> &f64 {
            &self.0[c as usize]
        }
    }
    impl IndexMut<Corner> for Wheels {
        fn index_mut(&mut self, c: Corner) -> &mut f64 {
            &mut self.0[c as usize]
        }
    }
    let mut pressure = Wheels([32.0; 4]);
    pressure[Corner::RearLeft] -= 4.5; // a slow leak
    println!(
        "FL {} FR {} RL {} RR {}",
        pressure[Corner::FrontLeft],
        pressure[Corner::FrontRight],
        pressure[Corner::RearLeft],
        pressure[Corner::RearRight]
    );
    // ANCHOR_END: ops
}

fn iterator_traits() {
    // ANCHOR: iter_traits
    use std::iter::Sum;

    #[derive(Debug, Clone, Copy, Default)]
    struct Force {
        x: f64,
        y: f64,
    }
    impl Sum for Force {
        fn sum<I: Iterator<Item = Force>>(iter: I) -> Force {
            iter.fold(Force::default(), |acc, f| Force {
                x: acc.x + f.x,
                y: acc.y + f.y,
            })
        }
    }
    let forces = [
        Force { x: 1.0, y: 0.0 },
        Force { x: 0.0, y: 2.0 },
        Force { x: -0.5, y: 0.5 },
    ];
    let net: Force = forces.iter().copied().sum();
    println!("net force {net:?}");

    /// A collection that keeps only the latest reading per sensor.
    #[derive(Debug, Default)]
    struct Latest(HashMap<&'static str, f64>);

    impl FromIterator<(&'static str, f64)> for Latest {
        fn from_iter<I: IntoIterator<Item = (&'static str, f64)>>(iter: I) -> Self {
            let mut latest = Latest::default();
            latest.extend(iter);
            latest
        }
    }
    impl Extend<(&'static str, f64)> for Latest {
        fn extend<I: IntoIterator<Item = (&'static str, f64)>>(&mut self, iter: I) {
            for (sensor, value) in iter {
                self.0.insert(sensor, value);
            }
        }
    }

    let mut latest: Latest = [("oil", 90.0), ("fuel", 0.6), ("oil", 92.5)]
        .into_iter()
        .collect();
    latest.extend([("fuel", 0.55)]);
    let mut entries: Vec<_> = latest.0.into_iter().collect();
    entries.sort_by(|a, b| a.0.cmp(b.0));
    println!("{entries:?}");
    // ANCHOR_END: iter_traits
}
