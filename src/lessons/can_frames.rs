//! Lesson: Parsing CAN frames.

use std::fmt;
use std::str::FromStr;

use crate::lesson::{Lesson, Question, Section};
use crate::rng::Rng;

pub static LESSON: Lesson = Lesson {
    id: "can_frames",
    title: "Parsing CAN frames",
    summary: "A validated CAN frame type, text and binary parsers that never panic, Intel/Motorola signal extraction, scaling, encoding, OBD-II decoding and fuzzing.",
    source: include_str!("can_frames.rs"),
    sections: &[
        Section::new(
            "CAN in one page",
            r#"
            The **Controller Area Network** links the ECUs in a vehicle. A
            classic CAN data frame carries:

            ```text
            ┌────────────────┬─────┬──────────────────────────────┬─────┐
            │ identifier     │ DLC │ data: 0–8 bytes              │ CRC │ …
            │ 11 bit (std)   │ 0–8 │                              │     │
            │ 29 bit (ext)   │     │                              │     │
            └────────────────┴─────┴──────────────────────────────┴─────┘
            ```

            - The **identifier** names the message, not the sender. A lower
              id wins bus arbitration, so it has higher priority.
            - The **DLC** (data length code) gives the payload length, 0 to 8.
              CAN FD extends payloads to 64 bytes.
            - **Signals** such as engine speed or wheel speed are bit fields
              inside the payload. A DBC file describes each one: start bit,
              length, byte order (Intel or Motorola), signedness, scale and
              offset.

            Hardware handles CRCs and acknowledgements. Software sees ids,
            lengths and bytes, which is what we'll model and parse here, with
            parsers that never panic.
            "#,
        ),
        Section::new(
            "A frame type that can't be invalid",
            r#"
            Start with types that make bad frames impossible to construct. An
            `enum CanId` distinguishes standard from extended ids, and its
            constructors reject ids that don't fit in 11 or 29 bits. A
            `CanFrame` stores up to 8 bytes **inline** (`[u8; 8]` plus a
            length), so it's `Copy`, never allocates, and can only be built
            through a constructor that checks the length. Every other function
            can then rely on those invariants.
            "#,
        )
        .demo("frame", frame_type),
        Section::new(
            "Parsing the text format",
            r#"
            Linux's `can-utils` write frames as `123#DEADBEEF`: three hex
            digits for a standard id, eight for an extended one, then the
            payload in hex. Implementing `FromStr` gives us `"…".parse()`, and
            `Display` prints the same format back, so text round-trips.

            The parser decodes hex pairs straight into the fixed array, with
            no `Vec` and no `String`, and reports **why** a line is bad
            through a small error enum.
            "#,
        )
        .demo("text", text_format),
        Section::new(
            "Parsing a binary layout",
            r#"
            Linux's SocketCAN hands frames to user space as a 16-byte
            `struct can_frame`: a 32-bit id word with flag bits (bit 31 =
            extended, bit 30 = remote request, bit 29 = error frame), a length
            byte, three padding bytes, and 8 data bytes. Taking the input as
            `&[u8; 16]` instead of `&[u8]` moves the length check into the
            type system. Then **destructuring the array** with a slice pattern
            names every field at once, and no index can be out of bounds.
            "#,
        )
        .demo("binary", binary_format),
        Section::new(
            "Signals: Intel (little-endian) byte order",
            r#"
            For an **Intel** signal, the DBC start bit is the signal's
            **least** significant bit, numbered `byte × 8 + bit` with bit 0 the
            LSB of the byte. Read the 8 payload bytes as one little-endian
            `u64`, and the signal is just `(word >> start) & mask`. No loops,
            no per-bit fiddling.
            "#,
        )
        .demo("intel", intel),
        Section::new(
            "Signals: Motorola (big-endian) byte order",
            r#"
            For a **Motorola** signal, the DBC start bit is the signal's
            **most** significant bit, and the value continues into the
            *following* bytes. That's the infamous "sawtooth" bit numbering.
            Read the payload as a **big-endian** `u64` instead. In that word,
            the start bit's position counted from the top is
            `byte × 8 + (7 − bit)`, and the value is
            `(word >> (64 − (top + len))) & mask`.

            Get this wrong and values look plausible, just slightly off. That
            is why golden test vectors matter so much here.
            "#,
        )
        .demo("motorola", motorola),
        Section::new(
            "Scale, offset, sign and range",
            r#"
            A raw integer becomes a physical value with
            `physical = raw × factor + offset`. A temperature byte with offset
            −40 covers −40…215 °C, and engine speed with factor 0.25 gives
            quarter-rpm resolution. Signed signals need sign extension first
            (Lesson 31). DBC files also give a valid **range**. A value outside
            it means a fault, not data, so the decoder below reports it
            instead of passing it on.
            "#,
        )
        .demo("scaling", scaling),
        Section::new(
            "Encoding signals into frames",
            r#"
            Transmitting is the inverse: convert the physical value to raw
            (`(physical − offset) / factor`, rounded), check that it fits in the
            signal's width, and insert it into the payload word without
            disturbing neighbouring signals. Then test the property that ties
            the two halves together: `decode(encode(x)) ≈ x`, within the
            signal's resolution.
            "#,
        )
        .demo("encode", encoding),
        Section::new(
            "Real-world decoding: OBD-II",
            r#"
            Every car sold in the US since 1996, and in the EU since 2001
            (petrol) or 2004 (diesel), answers standardized **OBD-II**
            diagnostic requests. A tester broadcasts on id 0x7DF, and the
            engine ECU answers on 0x7E8 with
            `[length, 0x41, PID, A, B, …]`. The SAE J1979 formulas are
            public:

            ```text
            PID 0x05  coolant temperature   A − 40          °C
            PID 0x0C  engine speed          (256·A + B) / 4  rpm
            PID 0x0D  vehicle speed         A               km/h
            PID 0x11  throttle position     100 · A / 255   %
            ```
            "#,
        )
        .demo("obd", obd),
        Section::new(
            "Fuzzing: proving the parsers don't panic",
            r#"
            Parsers face hostile or corrupted input. A cheap, effective check
            is **fuzzing**: feed in thousands of random inputs and confirm
            that nothing panics. Every input should give `Ok` or a clean
            `Err`. Seeded randomness makes any failure reproducible. For the
            real thing, `cargo fuzz` (libFuzzer) mutates inputs guided by
            coverage. But even this 20-line loop catches most "forgot a bounds
            check" bugs.
            "#,
        )
        .demo("fuzz", fuzz),
    ],
    quiz: &[
        Question::new(
            "Two nodes start transmitting at once, with ids 0x100 and 0x0F0. Which wins arbitration?",
            &[
                "0x100",
                "0x0F0",
                "Both are lost",
                "The one that started first",
            ],
            1,
            "Dominant (0) bits win during arbitration, so the numerically lower identifier has priority.",
        ),
        Question::new(
            "Why does `CanFrame` store `[u8; 8]` plus a length instead of a `Vec<u8>`?",
            &[
                "Vec can't hold bytes",
                "It's Copy, needs no allocation, and its size is fixed and known — ideal for embedded and real-time code",
                "Arrays are faster to index",
                "CAN requires it",
            ],
            1,
            "A fixed inline buffer avoids the heap entirely, which makes frame handling deterministic.",
        ),
        Question::new(
            "For an Intel signal starting at bit 16 with length 8, what is the raw value?",
            &[
                "Byte 1",
                "Byte 2",
                "Byte 3",
                "Bits 16..24 of the big-endian word",
            ],
            1,
            "Intel start bit 16 = byte 2, bit 0. Read little-endian, then shift right by 16 and mask 8 bits.",
        ),
        Question::new(
            "An OBD-II response carries PID 0x0C with A = 0x1A, B = 0xF8. What is the engine speed?",
            &["1726 rpm", "6904 rpm", "1720 rpm", "431 rpm"],
            0,
            "(256·0x1A + 0xF8) / 4 = (6656 + 248) / 4 = 1726 rpm.",
        ),
        Question::new(
            "Why accept `&[u8; 16]` instead of `&[u8]` in the binary parser?",
            &[
                "It's required by SocketCAN",
                "The length check moves into the type: the caller must prove there are 16 bytes, and indexing can't fail",
                "Slices are slower",
                "To allow mutation",
            ],
            1,
            "Fixed-size arrays carry their length in the type, so destructuring and constant indexing can't go out of bounds.",
        ),
    ],
    exercises: &[
        "Add CAN FD support: payloads up to 64 bytes, but only with the lengths the DLC can encode (0–8, 12, 16, 20, 24, 32, 48, 64).",
        "Parse full candump log lines like `(1700000000.123456) can0 7E8#04410C1AF8` into (timestamp, interface, frame).",
        "Write golden-vector tests for a Motorola signal that crosses three bytes, using values you compute by hand on paper.",
        "Extend the fuzz loop to the encoder: for random physical values within each signal's range, check `decode(encode(x))` stays within one resolution step of x.",
    ],
};

// ANCHOR: frame
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CanId {
    Standard(u16), // 11 bits: 0x000..=0x7FF
    Extended(u32), // 29 bits: 0x0000_0000..=0x1FFF_FFFF
}

impl fmt::Debug for CanId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CanId::Standard(id) => write!(f, "Standard({id:#05X})"), // ids read best in hex
            CanId::Extended(id) => write!(f, "Extended({id:#010X})"),
        }
    }
}

impl CanId {
    pub fn standard(id: u16) -> Option<CanId> {
        (id <= 0x7FF).then_some(CanId::Standard(id))
    }
    pub fn extended(id: u32) -> Option<CanId> {
        (id <= 0x1FFF_FFFF).then_some(CanId::Extended(id))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrameError {
    PayloadTooLong(usize),
    MissingSeparator,
    BadId,
    BadHex(char),
    OddHexDigits,
    BadLength(u8),
    UnsupportedFrameKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CanFrame {
    id: CanId,
    len: u8,       // invariant: len <= 8
    data: [u8; 8], // bytes beyond `len` are always zero
}

impl CanFrame {
    pub fn new(id: CanId, payload: &[u8]) -> Result<CanFrame, FrameError> {
        if payload.len() > 8 {
            return Err(FrameError::PayloadTooLong(payload.len()));
        }
        let mut data = [0u8; 8];
        data[..payload.len()].copy_from_slice(payload); // can't panic: checked above
        Ok(CanFrame {
            id,
            len: payload.len() as u8,
            data,
        })
    }
    pub fn id(&self) -> CanId {
        self.id
    }
    pub fn data(&self) -> &[u8] {
        &self.data[..usize::from(self.len)] // can't panic: len <= 8 is an invariant
    }
    /// The full 8-byte buffer (zero-padded), as signal decoders want it.
    pub fn padded(&self) -> &[u8; 8] {
        &self.data
    }
}

fn frame_type() {
    let id = CanId::standard(0x7E8).expect("fits in 11 bits");
    let frame = CanFrame::new(id, &[0x04, 0x41, 0x0C, 0x1A, 0xF8]).unwrap();
    println!("{frame:?}");
    println!("payload {:02X?}", frame.data());
    println!("standard 0x800 → {:?}", CanId::standard(0x800));
    println!("9-byte payload → {:?}", CanFrame::new(id, &[0; 9]));
    println!(
        "size_of::<CanFrame>() = {} bytes, Copy, no heap",
        std::mem::size_of::<CanFrame>()
    );
}
// ANCHOR_END: frame

// ANCHOR: text
fn hex_digit(c: char) -> Result<u8, FrameError> {
    c.to_digit(16).map(|d| d as u8).ok_or(FrameError::BadHex(c))
}

impl FromStr for CanFrame {
    type Err = FrameError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (id_text, data_text) = s
            .trim()
            .split_once('#')
            .ok_or(FrameError::MissingSeparator)?;
        let raw = u32::from_str_radix(id_text, 16).map_err(|_| FrameError::BadId)?;
        let id = match id_text.len() {
            3 => u16::try_from(raw).ok().and_then(CanId::standard),
            8 => CanId::extended(raw),
            _ => None,
        }
        .ok_or(FrameError::BadId)?;

        let mut data = [0u8; 8];
        let mut len = 0;
        let mut digits = data_text.chars().filter(|&c| c != '.');
        while let Some(hi) = digits.next() {
            let lo = digits.next().ok_or(FrameError::OddHexDigits)?;
            let slot = data
                .get_mut(len)
                .ok_or(FrameError::PayloadTooLong(len + 1))?;
            *slot = (hex_digit(hi)? << 4) | hex_digit(lo)?;
            len += 1;
        }
        CanFrame::new(id, &data[..len])
    }
}

impl fmt::Display for CanFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.id {
            CanId::Standard(id) => write!(f, "{id:03X}#")?,
            CanId::Extended(id) => write!(f, "{id:08X}#")?,
        }
        for byte in self.data() {
            write!(f, "{byte:02X}")?;
        }
        Ok(())
    }
}

fn text_format() {
    for line in [
        "7E8#04410C1AF8",
        "1F334455#11.22.33",
        "123#",
        "7E8#0441G0",
        "7E8#123",
        "800#00",
        "7E8 0441",
        "123#000102030405060708",
    ] {
        match line.parse::<CanFrame>() {
            Ok(frame) => println!(
                "{line:<24} → {frame}  (round-trip: {})",
                frame.to_string() == line.replace('.', "")
            ),
            Err(e) => println!("{line:<24} → {e:?}"),
        }
    }
}
// ANCHOR_END: text

// ANCHOR: binary
const CAN_EFF_FLAG: u32 = 0x8000_0000; // extended frame format
const CAN_RTR_FLAG: u32 = 0x4000_0000; // remote transmission request
const CAN_ERR_FLAG: u32 = 0x2000_0000; // error frame

fn from_socketcan(raw: &[u8; 16]) -> Result<CanFrame, FrameError> {
    // Destructure the whole array: every field named, nothing can go out of bounds.
    let [i0, i1, i2, i3, len, _pad0, _pad1, _pad2, data @ ..] = *raw;
    let id_word = u32::from_le_bytes([i0, i1, i2, i3]); // our capture files are little-endian
    if id_word & (CAN_RTR_FLAG | CAN_ERR_FLAG) != 0 {
        return Err(FrameError::UnsupportedFrameKind);
    }
    let id = if id_word & CAN_EFF_FLAG != 0 {
        CanId::extended(id_word & 0x1FFF_FFFF)
    } else {
        u16::try_from(id_word).ok().and_then(CanId::standard)
    }
    .ok_or(FrameError::BadId)?;
    let payload = data
        .get(..usize::from(len))
        .ok_or(FrameError::BadLength(len))?;
    CanFrame::new(id, payload)
}

fn binary_format() {
    let mut raw = [0u8; 16];
    raw[..4].copy_from_slice(&0x7E8u32.to_le_bytes());
    raw[4] = 3;
    raw[8..11].copy_from_slice(&[0x02, 0x41, 0x0D]);
    println!(
        "standard → {:?}",
        from_socketcan(&raw).map(|f| f.to_string())
    );

    raw[..4].copy_from_slice(&(0x18DA_F110 | CAN_EFF_FLAG).to_le_bytes());
    println!(
        "extended → {:?}",
        from_socketcan(&raw).map(|f| f.to_string())
    );

    raw[4] = 12; // a length that classic CAN can't have
    println!("bad length → {:?}", from_socketcan(&raw));
    raw[4] = 3;
    raw[..4].copy_from_slice(&(0x123 | CAN_ERR_FLAG).to_le_bytes());
    println!("error frame → {:?}", from_socketcan(&raw));
}
// ANCHOR_END: binary

// ANCHOR: intel
fn mask(len: u32) -> u64 {
    if len >= 64 {
        u64::MAX
    } else {
        (1u64 << len) - 1
    }
}

/// Intel: `start` is the LSB, counted from bit 0 of byte 0.
fn extract_intel(data: &[u8; 8], start: u32, len: u32) -> Option<u64> {
    if len == 0 || start + len > 64 {
        return None;
    }
    let word = u64::from_le_bytes(*data);
    Some((word >> start) & mask(len))
}

fn intel() {
    // Engine status frame: speed (16 bit, 0.25 rpm/bit) at bit 0, coolant at bit 16.
    let data = [0x80, 0x32, 0x7A, 0x00, 0, 0, 0, 0];
    let raw_rpm = extract_intel(&data, 0, 16).unwrap();
    let raw_temp = extract_intel(&data, 16, 8).unwrap();
    println!(
        "raw rpm {raw_rpm:#06x} = {} → {} rpm",
        raw_rpm,
        raw_rpm as f64 * 0.25
    );
    println!("raw coolant {raw_temp} → {} °C", raw_temp as i64 - 40);
    println!("out-of-range request → {:?}", extract_intel(&data, 60, 8));
}
// ANCHOR_END: intel

// ANCHOR: motorola
/// Motorola: `start` is the MSB in DBC "sawtooth" numbering (byte × 8 + bit).
fn extract_motorola(data: &[u8; 8], start: u32, len: u32) -> Option<u64> {
    let top = (start / 8) * 8 + (7 - start % 8); // position counted from the word's top
    if len == 0 || top + len > 64 {
        return None;
    }
    let word = u64::from_be_bytes(*data);
    Some((word >> (64 - (top + len))) & mask(len))
}

fn motorola() {
    // Vehicle speed: start 7 (MSB of byte 0), 16 bits → bytes 0..2 big-endian.
    let data = [0x1F, 0x40, 0xFF, 0xA0, 0, 0, 0, 0];
    println!("speed raw {:#06x}", extract_motorola(&data, 7, 16).unwrap());
    // A 12-bit signal starting at byte 2, bit 7: byte 2 + the high nibble of byte 3.
    println!(
        "12-bit raw {:#05x}",
        extract_motorola(&data, 23, 12).unwrap()
    );
    // The same bits read as Intel give a different (wrong) number:
    println!(
        "misread as Intel: {:#06x}",
        extract_intel(&data, 0, 16).unwrap()
    );
}
// ANCHOR_END: motorola

// ANCHOR: scaling
#[derive(Debug, Clone, Copy)]
enum ByteOrder {
    Intel,
    Motorola,
}

#[derive(Debug)]
struct Signal {
    name: &'static str,
    start: u32,
    len: u32,
    order: ByteOrder,
    signed: bool,
    factor: f64,
    offset: f64,
    min: f64,
    max: f64,
    unit: &'static str,
}

#[derive(Debug, PartialEq)]
enum SignalError {
    Layout,
    OutOfRange(f64),
}

impl Signal {
    fn raw(&self, data: &[u8; 8]) -> Option<u64> {
        match self.order {
            ByteOrder::Intel => extract_intel(data, self.start, self.len),
            ByteOrder::Motorola => extract_motorola(data, self.start, self.len),
        }
    }

    fn decode(&self, data: &[u8; 8]) -> Result<f64, SignalError> {
        let raw = self.raw(data).ok_or(SignalError::Layout)?;
        let value = if self.signed {
            let shift = 64 - self.len;
            ((raw << shift) as i64 >> shift) as f64 // sign-extend
        } else {
            raw as f64
        };
        let physical = value * self.factor + self.offset;
        if physical < self.min || physical > self.max {
            return Err(SignalError::OutOfRange(physical));
        }
        Ok(physical)
    }
}

const ENGINE_SPEED: Signal = Signal {
    name: "EngineSpeed",
    start: 0,
    len: 16,
    order: ByteOrder::Intel,
    signed: false,
    factor: 0.25,
    offset: 0.0,
    min: 0.0,
    max: 8000.0,
    unit: "rpm",
};
const COOLANT_TEMP: Signal = Signal {
    name: "CoolantTemp",
    start: 16,
    len: 8,
    order: ByteOrder::Intel,
    signed: false,
    factor: 1.0,
    offset: -40.0,
    min: -40.0,
    max: 150.0,
    unit: "°C",
};
const LONG_ACCEL: Signal = Signal {
    name: "LongAccel",
    start: 23,
    len: 12,
    order: ByteOrder::Motorola,
    signed: true,
    factor: 0.01,
    offset: 0.0,
    min: -15.0,
    max: 15.0,
    unit: "m/s²",
};

fn scaling() {
    let engine = [0x80, 0x32, 0x7A, 0x00, 0, 0, 0, 0];
    let dynamics = [0x00, 0x00, 0xFF, 0xA0, 0, 0, 0, 0]; // 0xFFA = -6 raw
    for (signal, data) in [
        (&ENGINE_SPEED, &engine),
        (&COOLANT_TEMP, &engine),
        (&LONG_ACCEL, &dynamics),
    ] {
        match signal.decode(data) {
            Ok(v) => println!("{:<12} {v:>8.2} {}", signal.name, signal.unit),
            Err(e) => println!("{:<12} {e:?}", signal.name),
        }
    }
    let overheated = [0, 0, 0xF0, 0, 0, 0, 0, 0]; // 240 - 40 = 200 °C: implausible
    println!(
        "implausible coolant → {:?}",
        COOLANT_TEMP.decode(&overheated)
    );
}
// ANCHOR_END: scaling

// ANCHOR: encode
impl Signal {
    fn encode(&self, physical: f64, data: &mut [u8; 8]) -> Result<(), SignalError> {
        if !(self.min..=self.max).contains(&physical) {
            return Err(SignalError::OutOfRange(physical));
        }
        let raw = ((physical - self.offset) / self.factor).round() as i64;
        let field = (raw as u64) & mask(self.len); // two's complement for signed
        match self.order {
            ByteOrder::Intel => {
                if self.start + self.len > 64 {
                    return Err(SignalError::Layout);
                }
                let mut word = u64::from_le_bytes(*data);
                word = (word & !(mask(self.len) << self.start)) | (field << self.start);
                *data = word.to_le_bytes();
            }
            ByteOrder::Motorola => {
                let top = (self.start / 8) * 8 + (7 - self.start % 8);
                if top + self.len > 64 {
                    return Err(SignalError::Layout);
                }
                let shift = 64 - (top + self.len);
                let mut word = u64::from_be_bytes(*data);
                word = (word & !(mask(self.len) << shift)) | (field << shift);
                *data = word.to_be_bytes();
            }
        }
        Ok(())
    }
}

fn encoding() {
    let mut engine = [0u8; 8];
    ENGINE_SPEED.encode(3200.0, &mut engine).unwrap();
    COOLANT_TEMP.encode(88.0, &mut engine).unwrap(); // neighbours are preserved
    println!("engine payload   {engine:02X?}");
    for s in [&ENGINE_SPEED, &COOLANT_TEMP] {
        println!("  {} decodes back to {:?}", s.name, s.decode(&engine));
    }
    let mut dynamics = [0u8; 8];
    LONG_ACCEL.encode(-2.37, &mut dynamics).unwrap();
    println!(
        "dynamics payload {dynamics:02X?} → {:?}",
        LONG_ACCEL.decode(&dynamics)
    );
    println!(
        "encode 9000 rpm → {:?}",
        ENGINE_SPEED.encode(9000.0, &mut engine)
    );

    // Property: decode(encode(x)) is within half a resolution step of x.
    let mut rng = Rng::new(7);
    let mut worst: f64 = 0.0;
    for _ in 0..10_000 {
        let x = (rng.next_u64() % 3000) as f64 / 100.0 - 15.0; // -15.00 .. 14.99
        let mut d = [0u8; 8];
        LONG_ACCEL.encode(x, &mut d).unwrap();
        worst = worst.max((LONG_ACCEL.decode(&d).unwrap() - x).abs());
    }
    println!(
        "round-trip worst error {worst:.4} (resolution {})",
        LONG_ACCEL.factor
    );
}
// ANCHOR_END: encode

fn obd() {
    // ANCHOR: obd
    fn decode_obd(frame: &CanFrame) -> Option<String> {
        if frame.id() != CanId::Standard(0x7E8) {
            return None;
        }
        // [length, 0x41 (positive response to mode 01), pid, A, B, …]
        match frame.data() {
            [_, 0x41, 0x05, a, ..] => Some(format!("coolant {} °C", i16::from(*a) - 40)),
            [_, 0x41, 0x0C, a, b, ..] => Some(format!(
                "engine {} rpm",
                (u32::from(*a) * 256 + u32::from(*b)) / 4
            )),
            [_, 0x41, 0x0D, a, ..] => Some(format!("speed {a} km/h")),
            [_, 0x41, 0x11, a, ..] => {
                Some(format!("throttle {:.1} %", f64::from(*a) * 100.0 / 255.0))
            }
            [_, 0x41, pid, ..] => Some(format!("PID {pid:#04x}: not decoded")),
            _ => None,
        }
    }

    let capture = [
        "7E8#03410582",
        "7E8#04410C1AF8",
        "7E8#03410D3C",
        "7E8#0341114D",
        "7E8#0341A6FF",
        "7E0#0201",
    ];
    for line in capture {
        let decoded = line.parse::<CanFrame>().ok().and_then(|f| decode_obd(&f));
        println!(
            "{line:<16} → {}",
            decoded.as_deref().unwrap_or("(not an OBD-II response)")
        );
    }
    // ANCHOR_END: obd
}

fn fuzz() {
    // ANCHOR: fuzz
    use std::panic;

    let previous = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));
    let mut rng = Rng::new(0xF022);
    let (mut panics, mut ok, mut rejected) = (0, 0, 0);

    for _ in 0..20_000 {
        // Random binary frames:
        let mut raw = [0u8; 16];
        for b in raw.iter_mut() {
            *b = rng.next_u64() as u8;
        }
        match panic::catch_unwind(|| from_socketcan(&raw)) {
            Ok(Ok(_)) => ok += 1,
            Ok(Err(_)) => rejected += 1,
            Err(_) => panics += 1,
        }
        // Random text lines from a hex-ish alphabet:
        let alphabet = b"0123456789ABCDEFxyz#. ";
        let len = rng.below(24);
        let line: String = (0..len)
            .map(|_| alphabet[rng.below(alphabet.len())] as char)
            .collect();
        if panic::catch_unwind(|| line.parse::<CanFrame>()).is_err() {
            panics += 1;
        }
    }
    panic::set_hook(previous);
    println!("binary: {ok} accepted, {rejected} rejected; panics in all parsers: {panics}");
    // ANCHOR_END: fuzz
}
