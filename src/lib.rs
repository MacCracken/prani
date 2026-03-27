//! # prani — Creature and Animal Vocal Synthesis
//!
//! **prani** (Sanskrit: living being / creature) provides vocal synthesis for
//! non-human creatures: animals, fantasy beings, and alien species. Built on
//! [svara](https://docs.rs/svara)'s formant synthesis engine with species-specific
//! vocal tract models, call pattern generators, and behavioral vocalization mapping.
//!
//! ## Architecture
//!
//! ```text
//! CreatureVoice (species params)
//!       |
//!       v
//! svara::GlottalSource → CreatureTract → Call Pattern → Output
//!   (excitation)         (species tract)  (vocalization)
//! ```
//!
//! ## Key Concepts
//!
//! - **Species**: Defines vocal tract geometry, f0 range, and available call types
//! - **CreatureVoice**: Instance of a species with individual variation
//! - **Vocalization**: A call pattern (growl, chirp, howl, roar, hiss, purr)
//! - **CallIntent**: Why the creature is vocalizing (alarm, territorial, mating, pain, idle)
//!
//! ## Quick Start
//!
//! ```rust
//! use prani::prelude::*;
//!
//! // Create a wolf voice and synthesize a howl
//! let voice = CreatureVoice::new(Species::Wolf);
//! let samples = voice.vocalize(
//!     &Vocalization::Howl,
//!     44100.0,
//!     2.0,  // 2 seconds
//! ).expect("synthesis should succeed");
//! ```
//!
//! ## Feature Flags
//!
//! | Feature | Default | Description |
//! |---------|---------|-------------|
//! | `std` | Yes | Standard library support. Disable for `no_std` + `alloc` |
//! | `logging` | No | Structured logging via tracing-subscriber |

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod error;
#[allow(dead_code)]
mod math;
#[allow(dead_code)]
pub(crate) mod rng;
pub mod species;
pub mod tract;
pub mod vocalization;
pub mod voice;

/// Convenience re-exports for common usage.
pub mod prelude {
    pub use crate::error::{PraniError, Result};
    pub use crate::species::Species;
    pub use crate::tract::CreatureTract;
    pub use crate::vocalization::{CallIntent, Vocalization};
    pub use crate::voice::CreatureVoice;
}

// Compile-time trait assertions: all public types must be Send + Sync.
#[cfg(test)]
mod assert_traits {
    fn _assert_send_sync<T: Send + Sync>() {}

    #[test]
    fn public_types_are_send_sync() {
        _assert_send_sync::<crate::error::PraniError>();
        _assert_send_sync::<crate::species::Species>();
        _assert_send_sync::<crate::tract::CreatureTract>();
        _assert_send_sync::<crate::vocalization::Vocalization>();
        _assert_send_sync::<crate::vocalization::CallIntent>();
        _assert_send_sync::<crate::voice::CreatureVoice>();
    }
}
