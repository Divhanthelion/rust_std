//! Counting heap allocations, per thread.
//!
//! This installs a global allocator that forwards to the system allocator
//! and bumps a thread-local counter on every allocation. The systems lessons
//! use it to *measure* that code is allocation-free rather than claim it.
//!
//! It is also a compact example of sound `unsafe`: an `unsafe impl` of an
//! `unsafe trait`, with the safety argument written down (Lesson 36).

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

pub struct CountingAllocator;

thread_local! {
    // `const` initialization and a type without `Drop` mean accessing this
    // never allocates, which matters because we read it *inside* `alloc`.
    static ALLOCATIONS: Cell<usize> = const { Cell::new(0) };
}

fn bump() {
    // `try_with` instead of `with`: never panic inside the allocator, even
    // during thread teardown.
    let _ = ALLOCATIONS.try_with(|count| count.set(count.get() + 1));
}

// SAFETY: every method forwards to `System` with the caller's arguments
// unchanged, so all of GlobalAlloc's contract is upheld by the system
// allocator. Counting doesn't touch the memory being managed.
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        bump();
        // SAFETY: our caller guarantees `layout` satisfies `alloc`'s contract.
        unsafe { System.alloc(layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        bump();
        // SAFETY: as for `alloc`.
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        bump();
        // SAFETY: `ptr` was allocated by us (hence by System) with `layout`,
        // as our caller guarantees.
        unsafe { System.realloc(ptr, layout, new_size) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: `ptr` came from this allocator with this `layout`.
        unsafe { System.dealloc(ptr, layout) }
    }
}

#[global_allocator]
static GLOBAL: CountingAllocator = CountingAllocator;

/// Allocations (including reallocations) made by the current thread so far.
pub fn allocations() -> usize {
    ALLOCATIONS.try_with(Cell::get).unwrap_or(0)
}

/// Runs `f` and returns its result with the number of heap allocations it
/// made on this thread.
///
/// ```
/// let (_, n) = rust_std::alloc_counter::count(|| vec![1, 2, 3]);
/// assert_eq!(n, 1);
/// let (_, n) = rust_std::alloc_counter::count(|| [1, 2, 3].iter().sum::<i32>());
/// assert_eq!(n, 0);
/// ```
pub fn count<T>(f: impl FnOnce() -> T) -> (T, usize) {
    let before = allocations();
    let out = f();
    (out, allocations() - before)
}
