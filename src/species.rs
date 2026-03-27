//! Species definitions with vocal tract parameters.
//!
//! Each species defines the geometry and acoustic properties of its vocal
//! apparatus: vocal fold characteristics, tract length, resonance chambers,
//! and available vocalization types.

use serde::{Deserialize, Serialize};

/// A creature species with associated vocal parameters.
///
/// Species define the physical constraints of vocalization: vocal tract length,
/// fundamental frequency range, formant scaling, and which call types are available.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Species {
    // Canids
    /// Wolf — long howls, barks, growls, whines. f0: 150-1200 Hz.
    Wolf,
    /// Dog (domestic) — barks, whines, growls, yelps. f0: 100-2000 Hz.
    Dog,

    // Felids
    /// Cat (domestic) — meow, purr, hiss, yowl. f0: 200-1000 Hz.
    Cat,
    /// Lion — roar, growl, grunt. f0: 40-200 Hz (infrasonic capable).
    Lion,

    // Birds
    /// Songbird (generic) — chirps, trills, warbles. f0: 1000-8000 Hz (syrinx).
    Songbird,
    /// Crow/raven — caw, croak, rattle. f0: 500-2000 Hz.
    Crow,
    /// Eagle/raptor — screech, kee. f0: 2000-4000 Hz.
    Raptor,

    // Reptiles
    /// Snake — hiss (noise-based, no vocal folds). No f0.
    Snake,
    /// Crocodilian — bellow, hiss, growl. f0: 30-200 Hz.
    Crocodilian,

    // Insects
    /// Cricket — chirp via stridulation (wing friction). f0: 3000-8000 Hz.
    Cricket,
    /// Bee — buzz via wing vibration. f0: 200-500 Hz.
    Bee,

    // Fantasy
    /// Dragon — roar with subharmonics, fire breath noise component.
    Dragon,
    /// Generic fantasy creature (configurable).
    Fantasy,
}

/// Vocal apparatus type — determines the excitation model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum VocalApparatus {
    /// Larynx with vocal folds (mammals, reptiles). Uses glottal source.
    Laryngeal,
    /// Syrinx (birds). Dual-source capable, higher f0 range.
    Syringeal,
    /// Stridulation (insects). Friction-based, periodic but non-glottal.
    Stridulatory,
    /// Noise-only (snakes, some insects). No periodic source.
    NoiseOnly,
}

/// Species-specific vocal parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeciesParams {
    /// Vocal apparatus type.
    pub apparatus: VocalApparatus,
    /// Minimum fundamental frequency (Hz).
    pub f0_min: f32,
    /// Maximum fundamental frequency (Hz).
    pub f0_max: f32,
    /// Default fundamental frequency (Hz).
    pub f0_default: f32,
    /// Vocal tract length scaling (1.0 = human male reference).
    /// Shorter tracts (< 1.0) = higher formants, longer (> 1.0) = lower.
    pub tract_scale: f32,
    /// Default breathiness (0.0-1.0).
    pub breathiness: f32,
    /// Default jitter (0.0-0.05).
    pub jitter: f32,
    /// Default shimmer (0.0-0.1).
    pub shimmer: f32,
}

impl SpeciesParams {
    /// Returns a deterministic seed derived from the species parameters.
    ///
    /// Different species get different seeds for varied noise character.
    #[must_use]
    pub fn resonance_seed(&self) -> u64 {
        // Hash the f0_default and tract_scale for a species-unique seed
        let bits = (self.f0_default.to_bits() as u64) ^ ((self.tract_scale.to_bits() as u64) << 32);
        bits.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1)
    }
}

impl Species {
    /// Returns the default vocal parameters for this species.
    #[must_use]
    pub fn params(self) -> SpeciesParams {
        match self {
            Self::Wolf => SpeciesParams {
                apparatus: VocalApparatus::Laryngeal,
                f0_min: 150.0,
                f0_max: 1200.0,
                f0_default: 400.0,
                tract_scale: 1.2,
                breathiness: 0.05,
                jitter: 0.015,
                shimmer: 0.03,
            },
            Self::Dog => SpeciesParams {
                apparatus: VocalApparatus::Laryngeal,
                f0_min: 100.0,
                f0_max: 2000.0,
                f0_default: 300.0,
                tract_scale: 0.9,
                breathiness: 0.03,
                jitter: 0.012,
                shimmer: 0.025,
            },
            Self::Cat => SpeciesParams {
                apparatus: VocalApparatus::Laryngeal,
                f0_min: 200.0,
                f0_max: 1000.0,
                f0_default: 500.0,
                tract_scale: 0.5,
                breathiness: 0.08,
                jitter: 0.02,
                shimmer: 0.04,
            },
            Self::Lion => SpeciesParams {
                apparatus: VocalApparatus::Laryngeal,
                f0_min: 40.0,
                f0_max: 200.0,
                f0_default: 80.0,
                tract_scale: 2.0,
                breathiness: 0.1,
                jitter: 0.025,
                shimmer: 0.05,
            },
            Self::Songbird => SpeciesParams {
                apparatus: VocalApparatus::Syringeal,
                f0_min: 1000.0,
                f0_max: 8000.0,
                f0_default: 3000.0,
                tract_scale: 0.15,
                breathiness: 0.02,
                jitter: 0.005,
                shimmer: 0.01,
            },
            Self::Crow => SpeciesParams {
                apparatus: VocalApparatus::Syringeal,
                f0_min: 500.0,
                f0_max: 2000.0,
                f0_default: 800.0,
                tract_scale: 0.3,
                breathiness: 0.15,
                jitter: 0.03,
                shimmer: 0.05,
            },
            Self::Raptor => SpeciesParams {
                apparatus: VocalApparatus::Syringeal,
                f0_min: 2000.0,
                f0_max: 4000.0,
                f0_default: 2500.0,
                tract_scale: 0.2,
                breathiness: 0.1,
                jitter: 0.01,
                shimmer: 0.02,
            },
            Self::Snake => SpeciesParams {
                apparatus: VocalApparatus::NoiseOnly,
                f0_min: 0.0,
                f0_max: 0.0,
                f0_default: 0.0,
                tract_scale: 0.4,
                breathiness: 1.0,
                jitter: 0.0,
                shimmer: 0.0,
            },
            Self::Crocodilian => SpeciesParams {
                apparatus: VocalApparatus::Laryngeal,
                f0_min: 30.0,
                f0_max: 200.0,
                f0_default: 60.0,
                tract_scale: 2.5,
                breathiness: 0.15,
                jitter: 0.03,
                shimmer: 0.06,
            },
            Self::Cricket => SpeciesParams {
                apparatus: VocalApparatus::Stridulatory,
                f0_min: 3000.0,
                f0_max: 8000.0,
                f0_default: 4500.0,
                tract_scale: 0.05,
                breathiness: 0.0,
                jitter: 0.002,
                shimmer: 0.005,
            },
            Self::Bee => SpeciesParams {
                apparatus: VocalApparatus::Stridulatory,
                f0_min: 200.0,
                f0_max: 500.0,
                f0_default: 300.0,
                tract_scale: 0.03,
                breathiness: 0.0,
                jitter: 0.001,
                shimmer: 0.003,
            },
            Self::Dragon => SpeciesParams {
                apparatus: VocalApparatus::Laryngeal,
                f0_min: 30.0,
                f0_max: 500.0,
                f0_default: 70.0,
                tract_scale: 3.0,
                breathiness: 0.2,
                jitter: 0.04,
                shimmer: 0.08,
            },
            Self::Fantasy => SpeciesParams {
                apparatus: VocalApparatus::Laryngeal,
                f0_min: 50.0,
                f0_max: 2000.0,
                f0_default: 200.0,
                tract_scale: 1.0,
                breathiness: 0.05,
                jitter: 0.015,
                shimmer: 0.03,
            },
        }
    }

    /// Returns whether this species can produce the given vocalization.
    ///
    /// Some combinations are bioacoustically invalid (e.g., a snake cannot howl,
    /// a cricket cannot roar). Invalid combinations will be rejected by
    /// [`CreatureVoice::vocalize()`](crate::voice::CreatureVoice::vocalize).
    #[must_use]
    pub fn supports_vocalization(self, v: &crate::vocalization::Vocalization) -> bool {
        use crate::vocalization::Vocalization;
        let apparatus = self.params().apparatus;
        match apparatus {
            VocalApparatus::NoiseOnly => matches!(v, Vocalization::Hiss | Vocalization::Growl),
            VocalApparatus::Stridulatory => {
                matches!(
                    v,
                    Vocalization::Stridulate | Vocalization::Buzz | Vocalization::Chirp
                )
            }
            VocalApparatus::Laryngeal | VocalApparatus::Syringeal => {
                // Laryngeal/syringeal can do most things, but not stridulate/buzz
                !matches!(v, Vocalization::Stridulate | Vocalization::Buzz)
            }
        }
    }
}
