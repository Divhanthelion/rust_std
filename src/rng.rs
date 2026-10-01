//! A tiny pseudo-random number generator (xorshift64*), so quizzes can be
//! shuffled without pulling in the `rand` crate.
//!
//! Not suitable for cryptography — it only needs to look random to a human.

use std::time::{SystemTime, UNIX_EPOCH};

pub struct Rng(u64);

impl Rng {
    /// Seeds from the clock and the process id.
    pub fn from_entropy() -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64) // truncation is fine for a seed
            .unwrap_or(0x9E37_79B9_7F4A_7C15);
        Rng::new(nanos ^ u64::from(std::process::id()).rotate_left(32))
    }

    pub fn new(seed: u64) -> Self {
        // The all-zero state is a fixed point of xorshift; avoid it.
        Rng(if seed == 0 {
            0x2545_F491_4F6C_DD1D
        } else {
            seed
        })
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// A number in `0..bound`. (The tiny modulo bias is irrelevant here.)
    pub fn below(&mut self, bound: usize) -> usize {
        assert!(bound > 0, "bound must be positive");
        (self.next_u64() % bound as u64) as usize
    }

    /// Fisher–Yates shuffle.
    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        for i in (1..items.len()).rev() {
            let j = self.below(i + 1);
            items.swap(i, j);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_for_a_seed() {
        let a: Vec<u64> = {
            let mut r = Rng::new(42);
            (0..5).map(|_| r.next_u64()).collect()
        };
        let mut r = Rng::new(42);
        let b: Vec<u64> = (0..5).map(|_| r.next_u64()).collect();
        assert_eq!(a, b);
    }

    #[test]
    fn shuffle_is_a_permutation() {
        let mut items: Vec<u32> = (0..50).collect();
        Rng::new(7).shuffle(&mut items);
        let mut sorted = items.clone();
        sorted.sort();
        assert_eq!(sorted, (0..50).collect::<Vec<_>>());
    }
}
