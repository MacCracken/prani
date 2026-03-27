//! Math compatibility layer for `no_std` support.
//!
//! When `std` is available, delegates to the standard library's `f32`/`f64` methods.
//! Without `std`, uses `libm` for transcendental functions.

/// f32 math operations (sin, cos, exp, sqrt, sinh).
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
    pub fn sinh(x: f32) -> f32 {
        x.sinh()
    }
}

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
    pub fn sinh(x: f32) -> f32 {
        libm::sinhf(x)
    }
}

/// f64 math operations.
#[cfg(feature = "std")]
pub(crate) mod f64 {
    #[inline(always)]
    pub fn sin(x: f64) -> f64 {
        x.sin()
    }
    #[inline(always)]
    pub fn cos(x: f64) -> f64 {
        x.cos()
    }
    #[inline(always)]
    pub fn sinh(x: f64) -> f64 {
        x.sinh()
    }
}

#[cfg(not(feature = "std"))]
pub(crate) mod f64 {
    #[inline(always)]
    pub fn sin(x: f64) -> f64 {
        libm::sin(x)
    }
    #[inline(always)]
    pub fn cos(x: f64) -> f64 {
        libm::cos(x)
    }
    #[inline(always)]
    pub fn sinh(x: f64) -> f64 {
        libm::sinh(x)
    }
}
