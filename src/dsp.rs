//! Shared DSP primitives for creature vocal synthesis.
//!
//! Provides a DC blocker (applied to all synthesis output) and
//! naad error mapping when the `naad-backend` feature is active.

use serde::{Deserialize, Serialize};

/// Single-pole DC blocker: removes DC offset that accumulates from
/// asymmetric excitation sources (purr, stridulation, subharmonics).
///
/// Transfer function: `H(z) = (1 - z^-1) / (1 - R·z^-1)`, R ≈ 0.995.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct DcBlocker {
    x_prev: f32,
    y_prev: f32,
    r: f32,
}

impl DcBlocker {
    /// Creates a new DC blocker with the standard pole radius.
    pub fn new() -> Self {
        Self {
            x_prev: 0.0,
            y_prev: 0.0,
            r: 0.995,
        }
    }

    /// Processes a single sample through the DC blocker.
    #[inline]
    pub fn process(&mut self, x: f32) -> f32 {
        let y = x - self.x_prev + self.r * self.y_prev;
        self.x_prev = x;
        self.y_prev = y;
        y
    }

    /// Processes an entire buffer in place.
    #[inline]
    pub fn process_buffer(&mut self, samples: &mut [f32]) {
        for s in samples.iter_mut() {
            *s = self.process(*s);
        }
    }

    /// Resets the blocker state.
    pub fn reset(&mut self) {
        self.x_prev = 0.0;
        self.y_prev = 0.0;
    }
}

/// Maps a `naad::NaadError` to a `PraniError::SynthesisFailed` with context.
#[cfg(feature = "naad-backend")]
#[allow(dead_code)]
pub(crate) fn map_naad_error(
    synth_name: &str,
    component: &str,
    err: naad::NaadError,
) -> crate::error::PraniError {
    let msg = alloc::format!("{synth_name}: {component} init failed: {err}");
    #[cfg(feature = "logging")]
    tracing::error!(synth = synth_name, component, %err, "naad backend error");
    crate::error::PraniError::SynthesisFailed(msg)
}
