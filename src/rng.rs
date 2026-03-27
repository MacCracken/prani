//! PCG32 PRNG for stochastic sound synthesis.

use serde::{Deserialize, Serialize};

/// PCG32 random number generator for environmental sound stochastic processes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Rng {
    state: u64,
    inc: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        let inc = (seed << 1) | 1;
        let mut rng = Self { state: 0, inc };
        rng.next_u32();
        rng.state = rng.state.wrapping_add(seed);
        rng.next_u32();
        rng
    }

    #[inline]
    pub fn next_u32(&mut self) -> u32 {
        let old_state = self.state;
        self.state = old_state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(self.inc);
        let xor_shifted = (((old_state >> 18) ^ old_state) >> 27) as u32;
        let rot = (old_state >> 59) as u32;
        (xor_shifted >> rot) | (xor_shifted << (rot.wrapping_neg() & 31))
    }

    /// Returns a value in [-1.0, 1.0].
    #[inline]
    pub fn next_f32(&mut self) -> f32 {
        let bits = (self.next_u32() >> 1) as i32;
        bits as f32 * (1.0 / i32::MAX as f32)
    }

    /// Returns a value in [0.0, 1.0).
    #[inline]
    pub fn next_f32_unsigned(&mut self) -> f32 {
        self.next_u32() as f32 * (1.0 / u32::MAX as f32)
    }

    /// Returns a value in [lo, hi).
    #[inline]
    pub fn next_f32_range(&mut self, lo: f32, hi: f32) -> f32 {
        lo + (hi - lo) * self.next_f32_unsigned()
    }

    /// Poisson-distributed count for a given rate (events per unit time).
    ///
    /// Uses Knuth's algorithm — suitable for small rates (< 30).
    pub fn poisson(&mut self, rate: f32) -> u32 {
        let l = crate::math::f32::exp(-rate);
        let mut k = 0u32;
        let mut p = 1.0f32;
        loop {
            k += 1;
            p *= self.next_f32_unsigned();
            if p < l {
                break;
            }
        }
        k - 1
    }
}
