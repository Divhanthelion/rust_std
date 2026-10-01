//! Lesson: Bytes & bits.

use std::fmt::Write as _;

use crate::lesson::{Lesson, Question, Section};

pub static LESSON: Lesson = Lesson {
    id: "bytes_bits",
    title: "Bytes & bits",
    summary: "Endianness, masks and shifts, bit fields, sign extension, safe binary parsing and writing, compile-time CRC tables, and hex dumps.",
    source: include_str!("bytes_bits.rs"),
    sections: &[
        Section::new(
            "Endianness",
            r#"
            A multi-byte integer can be stored most-significant byte first
            (**big-endian**, "network order", what CAN calls Motorola) or
            least-significant first (**little-endian**, Intel, and what x86 and
            most ARM cores use natively). Every integer type converts
            explicitly:

            ```text
            to_be_bytes / from_be_bytes    big-endian
            to_le_bytes / from_le_bytes    little-endian
            to_ne_bytes / from_ne_bytes    native: whatever this CPU uses (avoid in protocols)
            ```

            Protocol code should never depend on the host's byte order. With
            explicit `be`/`le` calls, the same code is correct on every target.
            "#,
        )
        .demo("endian", endianness),
        Section::new(
            "Masks and shifts",
            r#"
            The four basic operations on a flag bit `n`:

            ```text
            set     x | (1 << n)
            clear   x & !(1 << n)       (Rust uses ! for bitwise NOT, not ~)
            toggle  x ^ (1 << n)
            test    x & (1 << n) != 0
            ```

            Precedence is friendlier than in C. In Rust, `&`, `^` and `|` bind
            **tighter** than `==` and `!=`, so `x & MASK != 0` means
            `(x & MASK) != 0`, as you'd expect. Shifting by the type's width or
            more is an overflow, and it **panics in debug builds**. Use
            `checked_shl` or `wrapping_shl` when the amount is computed at run
            time.
            "#,
        )
        .demo("masks", masks),
        Section::new(
            "Bit fields: packing several values into one word",
            r#"
            Hardware registers and compact protocols pack several small fields
            into one integer. Two helper functions cover extraction and
            insertion:

            - extract: `(word >> shift) & mask`, where `mask = (1 << width) - 1`;
            - insert: clear the field with `!(mask << shift)`, then OR in
              `(value & mask) << shift`.

            Masking the inserted value keeps an oversized input from silently
            corrupting neighbouring fields. Better still, reject it, as
            `insert_checked` does below.
            "#,
        )
        .demo("fields", bit_fields),
        Section::new(
            "Sign extension",
            r#"
            Sensors often send signed values in odd widths: a 12-bit
            two's-complement acceleration, or a 10-bit temperature. To widen
            such a value to `i16` or `i32`, the field's top bit must be copied
            into all the higher bits. The idiom: shift the field up to the top
            of the wider type, reinterpret as signed with `as`, then shift back
            down. An **arithmetic** right shift on a signed type copies the
            sign bit as it shifts.
            "#,
        )
        .demo("sign", sign_extension),
        Section::new(
            "Parsing binary data without panicking",
            r#"
            Indexing (`buf[4]`) and slicing (`&buf[2..6]`) panic on short
            input, and a malformed frame mustn't crash an ECU. These
            non-panicking tools compose well:

            - `buf.get(a..b)` returns an `Option<&[u8]>`;
            - `split_first_chunk::<N>()` (Rust 1.77+) returns
              `Option<(&[u8; N], &[u8])>`, a fixed-size head plus the rest;
            - `<[u8; N]>::try_from(slice)` is a checked conversion.

            Wrapping them in a tiny **reader** struct with
            `read_u16_be() -> Result<…>` turns a parser into a straight line
            of `?`s.
            "#,
        )
        .demo("parse", parsing),
        Section::new(
            "Writing binary data",
            r#"
            To serialize, push bytes into a `Vec<u8>` with `push` and
            `extend_from_slice(&x.to_be_bytes())`. With no allocator available,
            write into a fixed `[u8; N]` with `copy_from_slice` on sub-slices.
            Write the encoder and decoder together, and test that they
            round-trip.
            "#,
        )
        .demo("write", writing),
        Section::new(
            "Checksums and a compile-time CRC table",
            r#"
            Automotive protocols protect data with CRCs. AUTOSAR's E2E
            profiles, for example, use CRC-8 with the SAE J1850 polynomial
            0x1D. A table-driven CRC is fast, and in Rust the 256-entry table
            can be computed **at compile time** by a `const fn`. The table
            ends up in read-only memory, costs no start-up time, and nobody can
            mistype a hex constant. `while` loops are allowed in `const fn`;
            `for` loops aren't yet, because they rely on the `Iterator` trait.
            "#,
        )
        .demo("crc", crc),
        Section::new(
            "Hex dumps",
            r#"
            When debugging binary data you'll want a hex dump: offsets, hex
            bytes, and printable ASCII, like `xxd` or `hexdump -C`. It's a good
            exercise in `chunks`, `write!` and formatting flags, and a useful
            tool to keep around.
            "#,
        )
        .demo("hexdump", hexdump),
    ],
    quiz: &[
        Question::new(
            "What does `0x1234u16.to_be_bytes()` return?",
            &[
                "[0x34, 0x12]",
                "[0x12, 0x34]",
                "[0x1234]",
                "It depends on the CPU",
            ],
            1,
            "Big-endian puts the most significant byte first, regardless of the host.",
        ),
        Question::new(
            "In Rust, how does `x & 0x80 != 0` parse?",
            &[
                "x & (0x80 != 0) — a type error",
                "(x & 0x80) != 0",
                "It's ambiguous and rejected",
                "x & !(0x80 == 0)",
            ],
            1,
            "Unlike C, Rust gives bitwise operators higher precedence than comparisons.",
        ),
        Question::new(
            "What is the bitwise NOT operator in Rust?",
            &["~", "!", "^", "not"],
            1,
            "`!` is logical NOT on bool and bitwise NOT on integers. There's no `~`.",
        ),
        Question::new(
            "The 4-bit two's-complement field 0b1110 sign-extended to i8 is…",
            &["14", "-2", "-14", "2"],
            1,
            "The top bit is 1, so the value is negative: 0b1110 = -2 in 4-bit two's complement.",
        ),
        Question::new(
            "Why build a CRC table with a `const fn`?",
            &[
                "It's required for CRCs",
                "The table is computed by the compiler and stored in read-only memory: no start-up cost, no typos",
                "const fn is faster at run time",
                "To allow mutation",
            ],
            1,
            "Compile-time evaluation produces the exact same table every build, embedded in the binary.",
        ),
    ],
    exercises: &[
        "Write `fn parity(x: u32) -> bool` with `count_ones`, then without it using the xor-folding trick.",
        "Write `fn reverse_bits_u8(b: u8) -> u8` by hand and compare it with the built-in `u8::reverse_bits` for all 256 inputs.",
        "Implement CRC-16/CCITT-FALSE (poly 0x1021, init 0xFFFF) with a const table, and check that `\"123456789\"` gives 0x29B1.",
        "Write a `Reader` that also supports `read_bits(n)` for fields that don't start on byte boundaries.",
    ],
};

fn endianness() {
    // ANCHOR: endian
    let value: u32 = 0x1234_5678;
    println!("be bytes {:02x?}", value.to_be_bytes()); // [12, 34, 56, 78]
    println!("le bytes {:02x?}", value.to_le_bytes()); // [78, 56, 34, 12]
    println!(
        "this CPU is {}-endian",
        if cfg!(target_endian = "little") {
            "little"
        } else {
            "big"
        }
    );

    let wire = [0x0C, 0x80]; // two bytes received from a bus
    println!("as big-endian u16:    {}", u16::from_be_bytes(wire)); // 3200
    println!("as little-endian u16: {}", u16::from_le_bytes(wire)); // 32780
    let signed = i16::from_be_bytes([0xFF, 0x38]);
    println!("signed big-endian:    {signed}"); // -200
    println!(
        "f32 from be bytes:    {}",
        f32::from_be_bytes([0x42, 0x48, 0x00, 0x00])
    ); // 50.0
    // ANCHOR_END: endian
}

fn masks() {
    // ANCHOR: masks
    const ENGINE_ON: u8 = 1 << 0;
    const DOOR_OPEN: u8 = 1 << 3;
    const LIGHTS: u8 = 1 << 5;

    let mut status: u8 = 0;
    status |= ENGINE_ON | LIGHTS; // set
    println!("set:    {status:08b}");
    status ^= DOOR_OPEN; //          toggle
    println!("toggle: {status:08b}");
    status &= !LIGHTS; //            clear (! is bitwise NOT)
    println!("clear:  {status:08b}");
    println!(
        "engine on? {}  lights? {}",
        status & ENGINE_ON != 0,
        status & LIGHTS != 0
    );

    let n = 9u32; // a shift amount computed at run time
    println!("1u8.checked_shl(9)  = {:?}", 1u8.checked_shl(n)); // None, not a panic
    println!("1u8.wrapping_shl(9) = {}", 1u8.wrapping_shl(n)); //  shift amount taken mod 8
    println!("lowest set bit of 0b10100: {}", 0b10100u8.trailing_zeros());
    println!(
        "isolate lowest set bit: {:#07b}",
        0b10100u8 & 0b10100u8.wrapping_neg()
    );
    // ANCHOR_END: masks
}

// ANCHOR: fields
fn extract(word: u32, shift: u32, width: u32) -> u32 {
    let mask = if width >= 32 {
        u32::MAX
    } else {
        (1 << width) - 1
    };
    (word >> shift) & mask
}

fn insert_checked(word: u32, shift: u32, width: u32, value: u32) -> Option<u32> {
    let mask = if width >= 32 {
        u32::MAX
    } else {
        (1 << width) - 1
    };
    if value > mask {
        return None; // would overflow into the neighbouring field
    }
    Some((word & !(mask << shift)) | (value << shift))
}

fn bit_fields() {
    // A 16-bit status word: [15..12 mode][11..4 temp °C + 40][3..0 error code]
    let word = 0b0011_0111_1010_0101u32;
    let mode = extract(word, 12, 4);
    let temp_c = extract(word, 4, 8) as i32 - 40;
    let error = extract(word, 0, 4);
    println!("mode {mode}, temp {temp_c} °C, error {error}");

    let updated = insert_checked(word, 0, 4, 0).unwrap(); // clear the error code
    println!("cleared error: {updated:016b}");
    println!("value too wide → {:?}", insert_checked(word, 0, 4, 16));
}
// ANCHOR_END: fields

fn sign_extension() {
    // ANCHOR: sign
    /// Interprets the low `bits` bits of `raw` as a two's-complement number.
    fn sign_extend(raw: u32, bits: u32) -> i32 {
        let shift = 32 - bits;
        ((raw << shift) as i32) >> shift // arithmetic shift copies the sign bit
    }

    for raw in [0x7FF, 0x800, 0xFFF, 0x001] {
        println!("12-bit {raw:#05x} → {}", sign_extend(raw, 12));
    }
    println!("4-bit 0b1110 → {}", sign_extend(0b1110, 4));
    println!("raw as i32 without extension: {} (wrong)", 0xFFF_u32 as i32);
    // ANCHOR_END: sign
}

fn parsing() {
    // ANCHOR: parse
    #[derive(Debug)]
    enum ParseError {
        TooShort { needed: usize, available: usize },
        BadMagic(u16),
    }

    /// A cursor over a byte slice whose reads never panic.
    struct Reader<'a> {
        rest: &'a [u8],
    }
    impl<'a> Reader<'a> {
        fn take<const N: usize>(&mut self) -> Result<[u8; N], ParseError> {
            let (head, rest) = self
                .rest
                .split_first_chunk::<N>()
                .ok_or(ParseError::TooShort {
                    needed: N,
                    available: self.rest.len(),
                })?;
            self.rest = rest;
            Ok(*head)
        }
        fn u8(&mut self) -> Result<u8, ParseError> {
            Ok(self.take::<1>()?[0])
        }
        fn u16_be(&mut self) -> Result<u16, ParseError> {
            Ok(u16::from_be_bytes(self.take()?))
        }
        fn u32_le(&mut self) -> Result<u32, ParseError> {
            Ok(u32::from_le_bytes(self.take()?))
        }
    }

    #[derive(Debug)]
    struct Header {
        version: u8,
        sequence: u16,
        timestamp_ms: u32,
    }

    fn parse_header(bytes: &[u8]) -> Result<Header, ParseError> {
        let mut r = Reader { rest: bytes };
        let magic = r.u16_be()?;
        if magic != 0xCAFE {
            return Err(ParseError::BadMagic(magic));
        }
        Ok(Header {
            version: r.u8()?,
            sequence: r.u16_be()?,
            timestamp_ms: r.u32_le()?,
        })
    }

    let good = [0xCA, 0xFE, 0x02, 0x00, 0x2A, 0xE8, 0x03, 0x00, 0x00];
    println!("{:?}", parse_header(&good));
    println!("{:?}", parse_header(&good[..6])); // truncated
    println!("{:?}", parse_header(&[0xBE, 0xEF, 0, 0, 0, 0, 0, 0, 0]));
    // ANCHOR_END: parse
}

fn writing() {
    // ANCHOR: write
    fn encode(version: u8, sequence: u16, timestamp_ms: u32) -> Vec<u8> {
        let mut out = Vec::with_capacity(9);
        out.extend_from_slice(&0xCAFEu16.to_be_bytes());
        out.push(version);
        out.extend_from_slice(&sequence.to_be_bytes());
        out.extend_from_slice(&timestamp_ms.to_le_bytes());
        out
    }

    // The same, into a fixed buffer, with no allocation:
    fn encode_into(buf: &mut [u8; 9], version: u8, sequence: u16, timestamp_ms: u32) {
        buf[0..2].copy_from_slice(&0xCAFEu16.to_be_bytes());
        buf[2] = version;
        buf[3..5].copy_from_slice(&sequence.to_be_bytes());
        buf[5..9].copy_from_slice(&timestamp_ms.to_le_bytes());
    }

    let v = encode(2, 42, 1000);
    let mut fixed = [0u8; 9];
    encode_into(&mut fixed, 2, 42, 1000);
    println!("vec   {v:02X?}");
    println!("array {fixed:02X?}");
    println!("identical: {}", v == fixed);
    // ANCHOR_END: write
}

// ANCHOR: crc
/// Builds the CRC-8 lookup table for `poly` at compile time.
const fn crc8_table(poly: u8) -> [u8; 256] {
    let mut table = [0u8; 256];
    let mut i = 0;
    while i < 256 {
        let mut crc = i as u8;
        let mut bit = 0;
        while bit < 8 {
            crc = if crc & 0x80 != 0 {
                (crc << 1) ^ poly
            } else {
                crc << 1
            };
            bit += 1;
        }
        table[i] = crc;
        i += 1;
    }
    table
}

/// SAE J1850 polynomial, as used by AUTOSAR E2E profiles 1 and 2.
const CRC8_J1850: [u8; 256] = crc8_table(0x1D);

pub(crate) fn crc8_sae_j1850(data: &[u8]) -> u8 {
    let mut crc = 0xFFu8; // initial value
    for &byte in data {
        crc = CRC8_J1850[usize::from(crc ^ byte)];
    }
    crc ^ 0xFF // final XOR
}

fn crc() {
    println!(
        "table[1] = {:#04x}, table[255] = {:#04x}",
        CRC8_J1850[1], CRC8_J1850[255]
    );
    println!(
        "check value \"123456789\" → {:#04x} (expected 0x4b)",
        crc8_sae_j1850(b"123456789")
    );
    let frame = [0x00, 0x12, 0x34, 0x56];
    let crc = crc8_sae_j1850(&frame);
    let mut corrupted = frame;
    corrupted[2] ^= 0x01; // flip one bit
    println!(
        "frame crc {crc:#04x}, after a bit flip {:#04x}",
        crc8_sae_j1850(&corrupted)
    );
}
// ANCHOR_END: crc

fn hexdump() {
    // ANCHOR: hexdump
    fn hex_dump(data: &[u8]) -> String {
        let mut out = String::new();
        for (row, chunk) in data.chunks(16).enumerate() {
            let _ = write!(out, "{:08x}  ", row * 16);
            for i in 0..16 {
                match chunk.get(i) {
                    Some(b) => {
                        let _ = write!(out, "{b:02x} ");
                    }
                    None => out.push_str("   "),
                }
                if i == 7 {
                    out.push(' ');
                }
            }
            out.push_str(" |");
            for &b in chunk {
                out.push(if b.is_ascii_graphic() || b == b' ' {
                    b as char
                } else {
                    '.'
                });
            }
            out.push_str("|\n");
        }
        out
    }

    let mut data = b"VIN JTDKN3DU0A0123456\x00\x01".to_vec();
    data.extend_from_slice(&3200u16.to_be_bytes());
    print!("{}", hex_dump(&data));
    // ANCHOR_END: hexdump
}
