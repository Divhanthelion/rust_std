//! Lesson: Unsafe Rust & FFI.

use std::ffi::{CStr, CString, c_char, c_int, c_void};
use std::mem::MaybeUninit;

use crate::lesson::{Lesson, Question, Section};

pub static LESSON: Lesson = Lesson {
    id: "unsafe_ffi",
    title: "Unsafe Rust & FFI",
    summary: "What unsafe permits, raw pointers, SAFETY contracts, building safe abstractions, calling C (strlen, qsort, abs), exposing Rust to C, ownership across FFI, static mut rules, MaybeUninit and tooling.",
    source: include_str!("unsafe_ffi.rs"),
    sections: &[
        Section::new(
            "What unsafe means",
            r#"
            Safe Rust guarantees no undefined behaviour (UB). Some correct
            programs can't be proven safe by the compiler, such as hardware
            access, FFI and certain data structures. `unsafe` lets you take on
            that proof obligation yourself. Inside an `unsafe` block you may:

            1. dereference a **raw pointer**;
            2. call an **`unsafe fn`**, including foreign (C) functions;
            3. access or modify a **`static mut`**;
            4. implement an **`unsafe trait`** (like `Send`, `Sync`, `GlobalAlloc`);
            5. read fields of a **`union`**.

            That's all. Borrow checking, type checking and bounds checks on safe
            operations still apply inside `unsafe`. The keyword marks where a
            human must have **verified** the invariants. Code review and audits
            focus exactly there.
            "#,
        ),
        Section::new(
            "Raw pointers",
            r#"
            `*const T` and `*mut T` are pointers without lifetimes or aliasing
            guarantees. **Creating** one is safe, with `&raw const x`, `&raw mut x`
            (Rust 1.82+) or a cast from a reference. **Dereferencing** one is
            unsafe, because nothing guarantees it's non-null, aligned, pointing
            at a live and initialized `T`, and not aliased by a `&mut`.

            Every unsafe block gets a `// SAFETY:` comment arguing why those
            conditions hold right there. The convention is enforced by Clippy's
            `undocumented_unsafe_blocks` lint.
            "#,
        )
        .demo("raw", raw_pointers),
        Section::new(
            "Unsafe functions and their contracts",
            r#"
            An `unsafe fn` has **preconditions** the compiler can't check. They
            are written in a `# Safety` doc section, and every caller must
            uphold them. `slice::get_unchecked` is the classic example: it skips
            the bounds check, and in exchange the caller promises the index is
            in bounds.

            In edition 2024, the body of an `unsafe fn` is **no longer**
            implicitly an unsafe block. Each unsafe operation inside needs its
            own `unsafe { }` with its own justification
            (`unsafe_op_in_unsafe_fn`).
            "#,
        )
        .demo("contracts", contracts),
        Section::new(
            "Safe abstractions over unsafe code",
            r#"
            The goal of `unsafe` is to build **safe interfaces**. A function
            whose signature is safe must be sound for **every** possible call.
            `split_at_mut` is the textbook case. The borrow checker can't see
            that two halves of a slice don't overlap, so std implements it with
            raw pointers, checks the one precondition (`mid <= len`), and
            exposes a signature that can't be misused. Below is the same
            function written from scratch.
            "#,
        )
        .demo("abstraction", safe_abstraction),
        Section::new(
            "Calling C: unsafe extern \"C\"",
            r#"
            Foreign functions are declared in an `extern` block. In edition 2024
            the block itself must be marked **`unsafe extern`**, because writing
            a wrong signature is itself a source of UB. Each item can then be
            marked `safe fn` if **no** argument can cause UB (like libm's
            `sqrt`), or left `unsafe`. `strlen` needs a valid NUL-terminated
            pointer, and even `abs` is UB for `INT_MIN`. Those get safe Rust
            wrappers that check the precondition.

            Use the `std::ffi` types that match C: `c_int`, `c_char`, `c_void`,
            `CStr` and `CString`. The C library is already linked into every
            std program, so the demos below call real libc functions,
            including `qsort` with a Rust callback.
            "#,
        )
        .demo("call_c", call_c),
        Section::new(
            "Exposing Rust to C",
            r#"
            For C (or C++, or an AUTOSAR stack) to call Rust:

            ```rust,nowrap
            /// # Safety
            /// `data` must point to `len` readable bytes (or be null when len is 0).
            #[unsafe(no_mangle)] // a stable symbol name; `unsafe(...)` in edition 2024
            pub unsafe extern "C" fn rs_checksum(data: *const u8, len: usize) -> u8 {
                if data.is_null() {
                    return 0;
                }
                // SAFETY: the caller promises `data` points to `len` bytes.
                let bytes = unsafe { std::slice::from_raw_parts(data, len) };
                bytes.iter().fold(0, |acc, b| acc ^ b)
            }
            ```

            Shared structs need `#[repr(C)]`. A **panic must never unwind into
            C**. Since Rust 1.81, a panic escaping an `extern "C"` function
            aborts the process. Use `extern "C-unwind"` only if the other side
            is built to unwind. Better still, catch panics at the boundary and
            return an error code. Tools like `cbindgen` generate the C header.
            "#,
        )
        .demo("expose", expose),
        Section::new(
            "Ownership across the boundary",
            r#"
            C can't own Rust objects directly. Hand it an **opaque handle**
            instead: `Box::into_raw` turns a `Box<T>` into a raw pointer and
            gives up ownership, and `Box::from_raw` takes it back so it drops
            normally. The API comes in pairs, `*_new` and `*_free`, and each
            handle must be freed **exactly once**, by Rust's allocator. Strings
            follow the same pattern: `CString::into_raw` and
            `CString::from_raw`.
            "#,
        )
        .demo("ownership", ffi_ownership),
        Section::new(
            "static mut, MaybeUninit and unions",
            r#"
            A **`static mut`** is a global anyone can write at any time. Since
            edition 2024 even **creating a reference** to one is an error
            (`static_mut_refs`). The only remaining access is through raw
            pointers (`&raw mut COUNTER`). In practice, use an atomic, a
            `Mutex` or `OnceLock` (Lesson 2), and leave `static mut` to
            hardware-adjacent code.

            **`MaybeUninit<T>`** holds memory that may not be initialized yet.
            Reading it is UB, writing is fine, and `assume_init` is the
            `unsafe` promise that you've written it. It's how fixed-capacity
            containers avoid the `Option` overhead (Lesson 33).

            **Unions** reinterpret the same bytes as different types. Prefer
            the safe, explicit conversions: `f32::to_bits`, `from_be_bytes`.
            "#,
        )
        .demo("uninit", uninit),
        Section::new(
            "Tools and policy",
            r#"
            - **Miri** (`cargo +nightly miri test`) runs your tests in an
              interpreter that detects UB: out-of-bounds access, use-after-free,
              invalid aliasing, data races.
            - **Sanitizers** (ASan, TSan) cover the FFI side too.
            - `#![forbid(unsafe_code)]` keeps crates that need no unsafe free of
              it. Concentrate the rest in small, reviewed modules.
            - Clippy's `undocumented_unsafe_blocks` and
              `missing_safety_doc` enforce the comment conventions.

            This program contains exactly two `unsafe` areas, both commented:
            the counting allocator (`src/alloc_counter.rs`) and the spinlock in
            Lesson 24. Plus this lesson, of course.
            "#,
        ),
    ],
    quiz: &[
        Question::new(
            "Which of these requires an unsafe block?",
            &[
                "Creating a raw pointer with &raw const x",
                "Dereferencing a raw pointer",
                "Comparing two raw pointers",
                "Casting a reference to a raw pointer",
            ],
            1,
            "Making raw pointers is safe; reading or writing through them is where UB can occur.",
        ),
        Question::new(
            "What does `unsafe` turn off?",
            &[
                "The borrow checker",
                "Type checking",
                "Nothing — it unlocks five extra operations whose safety you must prove",
                "Bounds checks everywhere",
            ],
            2,
            "All normal checks still apply. unsafe adds capabilities; it doesn't remove rules.",
        ),
        Question::new(
            "In edition 2024, why must `extern \"C\"` blocks be written `unsafe extern \"C\"`?",
            &[
                "C is slow",
                "Declaring a foreign signature incorrectly is itself a source of undefined behaviour",
                "To link libc",
                "It's only a style change",
            ],
            1,
            "The compiler trusts the declared signature. Marking the block unsafe makes that trust explicit, and lets items be marked `safe fn`.",
        ),
        Question::new(
            "What happens if a Rust panic reaches the boundary of an `extern \"C\"` function?",
            &[
                "It unwinds into C",
                "The process aborts",
                "It's converted to errno",
                "It's silently ignored",
            ],
            1,
            "Since Rust 1.81, unwinding out of an extern \"C\" function aborts. Use catch_unwind or extern \"C-unwind\" deliberately.",
        ),
        Question::new(
            "How should a Rust object handed to C as `*mut T` be freed?",
            &[
                "With C's free()",
                "By passing it back to a Rust function that calls Box::from_raw exactly once",
                "It doesn't need to be",
                "With mem::forget",
            ],
            1,
            "Memory must be freed by the allocator that created it; Box::from_raw restores ownership so Drop runs.",
        ),
    ],
    exercises: &[
        "Declare `memcmp` in an `unsafe extern \"C\"` block and write a safe wrapper `fn bytes_equal(a: &[u8], b: &[u8]) -> bool` with a SAFETY comment.",
        "Implement a safe `fn split_first_mut<T>(s: &mut [T]) -> Option<(&mut T, &mut [T])>` with raw pointers, then compare it with the std method.",
        "Write a C-style API `counter_new() -> *mut Counter`, `counter_inc(*mut Counter)`, `counter_free(*mut Counter)` and test it from Rust, including a null pointer.",
        "Wrap an `extern \"C\"` function body in `catch_unwind` and return -1 if the Rust code panicked.",
    ],
};

fn raw_pointers() {
    // ANCHOR: raw
    let mut speed: u32 = 80;
    let read_ptr: *const u32 = &raw const speed; // creating: safe
    let write_ptr: *mut u32 = &raw mut speed;

    // SAFETY: both pointers come from a live local, are aligned, and no
    // reference to `speed` is used while we go through them.
    unsafe {
        *write_ptr += 20;
        println!("through the raw pointer: {}", *read_ptr);
    }

    let values = [10u16, 20, 30, 40];
    let base = values.as_ptr();
    // SAFETY: offset 2 is within the 4-element array.
    let third = unsafe { *base.add(2) };
    println!("base.add(2) → {third}");

    let null: *const u8 = std::ptr::null();
    println!("null.is_null() = {}", null.is_null()); // checking is safe; deref would be UB
    // ANCHOR_END: raw
}

fn contracts() {
    // ANCHOR: contracts
    /// Returns the sum of the elements at `i` and `j`.
    ///
    /// # Safety
    /// Both `i` and `j` must be less than `values.len()`.
    unsafe fn sum_unchecked(values: &[u32], i: usize, j: usize) -> u32 {
        // SAFETY: the caller guarantees both indices are in bounds.
        unsafe { values.get_unchecked(i) + values.get_unchecked(j) }
    }

    let v = [5, 7, 11];
    // SAFETY: 0 and 2 are both < 3.
    let s = unsafe { sum_unchecked(&v, 0, 2) };
    println!("sum_unchecked = {s}");

    // The safe wrapper checks once, then calls the fast version.
    fn sum_checked(values: &[u32], i: usize, j: usize) -> Option<u32> {
        if i < values.len() && j < values.len() {
            // SAFETY: just checked both bounds.
            Some(unsafe { sum_unchecked(values, i, j) })
        } else {
            None
        }
    }
    println!(
        "checked: {:?} {:?}",
        sum_checked(&v, 1, 2),
        sum_checked(&v, 1, 9)
    );
    // ANCHOR_END: contracts
}

fn safe_abstraction() {
    // ANCHOR: abstraction
    fn my_split_at_mut<T>(slice: &mut [T], mid: usize) -> (&mut [T], &mut [T]) {
        let len = slice.len();
        assert!(mid <= len, "mid out of bounds"); // the one precondition, checked
        let ptr = slice.as_mut_ptr();
        // SAFETY: [0, mid) and [mid, len) are in bounds and don't overlap, so
        // the two &mut slices never alias. Their lifetimes are tied to `slice`.
        unsafe {
            (
                std::slice::from_raw_parts_mut(ptr, mid),
                std::slice::from_raw_parts_mut(ptr.add(mid), len - mid),
            )
        }
    }

    let mut wheel_speeds = [10.0, 10.2, 9.9, 10.1];
    let (front, rear) = my_split_at_mut(&mut wheel_speeds, 2);
    front[0] += 1.0; // two simultaneous &mut, proven disjoint
    rear[1] -= 1.0;
    println!("{wheel_speeds:?}");
    // ANCHOR_END: abstraction
}

// ANCHOR: call_c
unsafe extern "C" {
    /// libm's square root has no preconditions (negative input gives NaN),
    /// so it can be declared `safe`: calling it needs no unsafe block.
    safe fn sqrt(x: f64) -> f64;
    /// `abs(INT_MIN)` is undefined behaviour in C, so this one stays unsafe
    /// and gets a checking wrapper below.
    fn abs(x: c_int) -> c_int;
    /// Needs a valid pointer to a NUL-terminated string, so it stays unsafe.
    fn strlen(s: *const c_char) -> usize;
    fn qsort(
        base: *mut c_void,
        count: usize,
        size: usize,
        compare: extern "C" fn(*const c_void, *const c_void) -> c_int,
    );
}

/// A comparator C can call. `extern "C"` gives it the C calling convention.
extern "C" fn compare_i32(a: *const c_void, b: *const c_void) -> c_int {
    // SAFETY: qsort only passes pointers to elements of the i32 array we gave it.
    let (a, b) = unsafe { (*(a as *const i32), *(b as *const i32)) };
    a.cmp(&b) as c_int // Ordering is -1, 0 or 1
}

/// A safe wrapper: checks C's precondition, then calls it.
fn c_abs(x: c_int) -> Option<c_int> {
    if x == c_int::MIN {
        return None; // C's abs would be undefined here
    }
    // SAFETY: x != INT_MIN, so abs is well-defined.
    Some(unsafe { abs(x) })
}

fn call_c() {
    println!("sqrt(2.0) via libm = {:.6}  (declared safe)", sqrt(2.0));
    println!(
        "c_abs(-42) = {:?}, c_abs(INT_MIN) = {:?}",
        c_abs(-42),
        c_abs(c_int::MIN)
    );

    let name = CString::new("Toyota").expect("no interior NUL");
    // SAFETY: `name` is a valid NUL-terminated string that outlives the call.
    let len = unsafe { strlen(name.as_ptr()) };
    println!("strlen(\"Toyota\") = {len}");

    let literal: &CStr = c"bZ4X"; // a C string literal (Rust 1.77+)
    // SAFETY: as above; literals are 'static.
    println!("strlen(c\"bZ4X\") = {}", unsafe {
        strlen(literal.as_ptr())
    });

    let mut readings: [i32; 6] = [42, -7, 19, 0, 88, 3];
    // SAFETY: the pointer, count and element size describe exactly our array,
    // and compare_i32 matches the element type.
    unsafe {
        qsort(
            readings.as_mut_ptr().cast(),
            readings.len(),
            std::mem::size_of::<i32>(),
            compare_i32,
        );
    }
    println!("sorted by C's qsort: {readings:?}");
}
// ANCHOR_END: call_c

// ANCHOR: expose
/// # Safety
/// `data` must point to `len` readable bytes, or be null.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_checksum(data: *const u8, len: usize) -> u8 {
    if data.is_null() {
        return 0;
    }
    // SAFETY: the caller promises `data` points to `len` readable bytes.
    let bytes = unsafe { std::slice::from_raw_parts(data, len) };
    bytes.iter().fold(0, |acc, b| acc ^ b)
}

/// A boundary that never lets a panic escape: errors become return codes.
///
/// # Safety
/// `text` must be null or a NUL-terminated string; `out` must be null or
/// valid for writing a `u32`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rs_parse_rpm(text: *const c_char, out: *mut u32) -> c_int {
    let result = std::panic::catch_unwind(|| {
        if text.is_null() || out.is_null() {
            return -1;
        }
        // SAFETY: non-null; the caller promises a NUL-terminated string.
        let s = unsafe { CStr::from_ptr(text) };
        match s.to_str().ok().and_then(|s| s.trim().parse::<u32>().ok()) {
            Some(rpm) => {
                // SAFETY: non-null, and the caller promises it's writable.
                unsafe { out.write(rpm) };
                0
            }
            None => -2,
        }
    });
    result.unwrap_or(-3) // a panic becomes an error code, not an abort
}

fn expose() {
    let data = [0x12u8, 0x34, 0x56];
    // SAFETY: pointer and length describe `data`.
    println!("rs_checksum = {:#04x}", unsafe {
        rs_checksum(data.as_ptr(), data.len())
    });
    // SAFETY: null is explicitly allowed.
    println!("rs_checksum(null) = {}", unsafe {
        rs_checksum(std::ptr::null(), 5)
    });

    let mut rpm = 0u32;
    // SAFETY: C string literals are NUL-terminated; `rpm` is writable; null is allowed.
    let (ok, bad, null) = unsafe {
        (
            rs_parse_rpm(c"3200".as_ptr(), &mut rpm),
            rs_parse_rpm(c"fast".as_ptr(), &mut rpm),
            rs_parse_rpm(std::ptr::null(), &mut rpm),
        )
    };
    println!("parse \"3200\" → code {ok}, rpm {rpm}");
    println!("parse \"fast\" → code {bad}");
    println!("parse null   → code {null}");
}
// ANCHOR_END: expose

// ANCHOR: ownership
pub struct Odometer {
    km: u64,
}

#[unsafe(no_mangle)]
pub extern "C" fn odometer_new() -> *mut Odometer {
    Box::into_raw(Box::new(Odometer { km: 0 })) // ownership handed to the caller
}

/// # Safety
/// `odo` must come from `odometer_new` and not have been freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn odometer_add(odo: *mut Odometer, km: u64) {
    // SAFETY: the caller guarantees a live, exclusive handle (or null).
    if let Some(o) = unsafe { odo.as_mut() } {
        o.km = o.km.saturating_add(km);
    }
}

/// # Safety
/// `odo` must come from `odometer_new` and must not be used afterwards.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn odometer_free(odo: *mut Odometer) -> u64 {
    if odo.is_null() {
        return 0;
    }
    // SAFETY: we created it with Box::into_raw; taking it back frees it once.
    let boxed = unsafe { Box::from_raw(odo) };
    boxed.km
}

fn ffi_ownership() {
    let handle = odometer_new(); // what C code would do
    // SAFETY: handle is live; we free it exactly once below.
    unsafe {
        odometer_add(handle, 120);
        odometer_add(handle, 30);
        println!("final reading {} km (freed)", odometer_free(handle));
    }
}
// ANCHOR_END: ownership

fn uninit() {
    // ANCHOR: uninit
    static mut BOOT_COUNT: u32 = 0;
    // `&BOOT_COUNT` would be rejected (static_mut_refs). Go through a raw pointer:
    let boot_ptr = &raw mut BOOT_COUNT;
    // SAFETY: only this demo touches BOOT_COUNT, from one thread at a time.
    let boots = unsafe {
        *boot_ptr += 1;
        *boot_ptr
    };
    println!("boot count (static mut via &raw mut): {boots}");

    // MaybeUninit: write first, then promise it's initialized.
    let mut buf: [MaybeUninit<u16>; 4] = [MaybeUninit::uninit(); 4];
    for (i, slot) in buf.iter_mut().enumerate() {
        slot.write(i as u16 * 100); // writing is always fine
    }
    // SAFETY: every element was written by the loop above.
    let ready: [u16; 4] = buf.map(|slot| unsafe { slot.assume_init() });
    println!("initialized array {ready:?}");

    // Prefer safe bit reinterpretation over unions/transmute:
    let bits = 1.5f32.to_bits();
    println!("1.5f32 bits {bits:#010x} → back {}", f32::from_bits(bits));
    // ANCHOR_END: uninit
}
