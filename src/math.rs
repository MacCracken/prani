//! Math compatibility layer for `no_std` support.
//!
//! When `std` is available, delegates to the standard library's `f32` methods.
//! Without `std`, uses `libm` for transcendental functions.

/// f32 math operations.
#[allow(dead_code)]
#[cfg(feature = "std")]
pub(crate) mod f32 {
    #[inline(always)]
    pub fn sin(x: f32) -> f32 {
        x.sin()
    }

    #[inline(always)]
    pub fn cos(x: f32) -> f32 {
        x.cos()
    }

    #[inline(always)]
    pub fn exp(x: f32) -> f32 {
        x.exp()
    }

    #[inline(always)]
    pub fn sqrt(x: f32) -> f32 {
        x.sqrt()
    }

    #[inline(always)]
    pub fn powf(base: f32, exp: f32) -> f32 {
        base.powf(exp)
    }
}

#[allow(dead_code)]
#[cfg(not(feature = "std"))]
pub(crate) mod f32 {
    #[inline(always)]
    pub fn sin(x: f32) -> f32 {
        libm::sinf(x)
    }

    #[inline(always)]
    pub fn cos(x: f32) -> f32 {
        libm::cosf(x)
    }

    #[inline(always)]
    pub fn exp(x: f32) -> f32 {
        libm::expf(x)
    }

    #[inline(always)]
    pub fn sqrt(x: f32) -> f32 {
        libm::sqrtf(x)
    }

    #[inline(always)]
    pub fn powf(base: f32, exp: f32) -> f32 {
        libm::powf(base, exp)
    }
}
