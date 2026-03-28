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
    /// Vibratile (bees, flies). Thoracic flight muscle vibration through wings.
    Vibratile,
    /// Noise-only (snakes, some insects). No periodic source.
    NoiseOnly,
}

/// Species-specific vocal parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[must_use]
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
    /// Species formant frequencies F1-F3 (Hz). Scaled by tract_scale internally.
    /// These define the characteristic resonance of the species' vocal tract.
    pub formants: [f32; 3],
    /// Formant bandwidths B1-B3 (Hz). Wider = less defined resonance.
    pub bandwidths: [f32; 3],
    /// Default breathiness (0.0-1.0).
    pub breathiness: f32,
    /// Default jitter (0.0-0.05).
    pub jitter: f32,
    /// Default shimmer (0.0-0.1).
    pub shimmer: f32,
    /// Spectral tilt in dB/octave (negative = darker, 0 = neutral, positive = brighter).
    /// Lion roars: steep roll-off (-6). Bird calls: relatively flat (-1).
    #[serde(default)]
    pub spectral_tilt: f32,
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
    pub fn params(self) -> SpeciesParams {
        // Formant frequencies derived from vocal tract length:
        // F_n ≈ (2n-1) * c / (4L), c=343 m/s
        // Human male tract ~17cm → F1≈500, F2≈1500, F3≈2500
        // Species tracts scaled proportionally.
        match self {
            Self::Wolf => SpeciesParams {
                apparatus: VocalApparatus::Laryngeal,
                f0_min: 150.0,
                f0_max: 1200.0,
                f0_default: 400.0,
                tract_scale: 1.2,
                formants: [420.0, 1250.0, 2100.0], // ~20cm tract
                bandwidths: [100.0, 120.0, 150.0],
                breathiness: 0.05,
                jitter: 0.015,
                shimmer: 0.03,
                spectral_tilt: -3.0,
            },
            Self::Dog => SpeciesParams {
                apparatus: VocalApparatus::Laryngeal,
                f0_min: 100.0,
                f0_max: 2000.0,
                f0_default: 300.0,
                tract_scale: 0.9,
                formants: [550.0, 1650.0, 2750.0], // ~15cm tract
                bandwidths: [100.0, 130.0, 160.0],
                breathiness: 0.03,
                jitter: 0.012,
                shimmer: 0.025,
                spectral_tilt: -2.0,
            },
            Self::Cat => SpeciesParams {
                apparatus: VocalApparatus::Laryngeal,
                f0_min: 200.0,
                f0_max: 1000.0,
                f0_default: 500.0,
                tract_scale: 0.5,
                formants: [850.0, 2500.0, 4200.0], // ~10cm tract
                bandwidths: [120.0, 150.0, 200.0],
                breathiness: 0.08,
                jitter: 0.02,
                shimmer: 0.04,
                spectral_tilt: -2.0,
            },
            Self::Lion => SpeciesParams {
                apparatus: VocalApparatus::Laryngeal,
                f0_min: 40.0,
                f0_max: 200.0,
                f0_default: 80.0,
                tract_scale: 2.0,
                formants: [250.0, 750.0, 1250.0], // ~35cm tract
                bandwidths: [80.0, 100.0, 130.0],
                breathiness: 0.1,
                jitter: 0.025,
                shimmer: 0.05,
                spectral_tilt: -6.0, // Steep roll-off, very dark
            },
            Self::Songbird => SpeciesParams {
                apparatus: VocalApparatus::Syringeal,
                f0_min: 1000.0,
                f0_max: 8000.0,
                f0_default: 3000.0,
                tract_scale: 0.15,
                formants: [3500.0, 7000.0, 10500.0], // ~2.5cm tract
                bandwidths: [500.0, 800.0, 1100.0],  // Wider: birds have less defined formants
                breathiness: 0.02,
                jitter: 0.005,
                shimmer: 0.01,
                spectral_tilt: -1.0, // Relatively flat
            },
            Self::Crow => SpeciesParams {
                apparatus: VocalApparatus::Syringeal,
                f0_min: 500.0,
                f0_max: 2000.0,
                f0_default: 800.0,
                tract_scale: 0.3,
                formants: [1400.0, 4200.0, 7000.0], // ~6cm tract
                bandwidths: [350.0, 500.0, 700.0],  // Wider for harsh corvid calls
                breathiness: 0.18,                  // Notably harsh/noisy
                jitter: 0.03,
                shimmer: 0.05,
                spectral_tilt: -2.0,
            },
            Self::Raptor => SpeciesParams {
                apparatus: VocalApparatus::Syringeal,
                f0_min: 2000.0,
                f0_max: 4000.0,
                f0_default: 2500.0,
                tract_scale: 0.2,
                formants: [2100.0, 5500.0, 8500.0], // ~4cm tract
                bandwidths: [400.0, 600.0, 800.0],  // Wider for birds
                breathiness: 0.1,
                jitter: 0.01,
                shimmer: 0.02,
                spectral_tilt: -1.5,
            },
            Self::Snake => SpeciesParams {
                apparatus: VocalApparatus::NoiseOnly,
                f0_min: 0.0,
                f0_max: 0.0,
                f0_default: 0.0,
                tract_scale: 0.4,
                formants: [2500.0, 5000.0, 8000.0], // Noise shaping bands
                bandwidths: [1500.0, 2000.0, 2500.0], // Very wide = noisy
                breathiness: 1.0,
                jitter: 0.0,
                shimmer: 0.0,
                spectral_tilt: 0.0,
            },
            Self::Crocodilian => SpeciesParams {
                apparatus: VocalApparatus::Laryngeal,
                f0_min: 30.0,
                f0_max: 200.0,
                f0_default: 60.0,
                tract_scale: 2.5,
                formants: [200.0, 600.0, 1000.0], // ~40cm+ tract
                bandwidths: [70.0, 90.0, 120.0],
                breathiness: 0.15,
                jitter: 0.03,
                shimmer: 0.06,
                spectral_tilt: -5.0, // Dark, low-frequency dominant
            },
            Self::Cricket => SpeciesParams {
                apparatus: VocalApparatus::Stridulatory,
                f0_min: 3000.0,
                f0_max: 8000.0,
                f0_default: 4500.0,
                tract_scale: 0.05,
                formants: [4500.0, 9000.0, 13500.0], // Resonant body
                bandwidths: [500.0, 800.0, 1000.0],
                breathiness: 0.0,
                jitter: 0.002,
                shimmer: 0.005,
                spectral_tilt: -1.0,
            },
            Self::Bee => SpeciesParams {
                apparatus: VocalApparatus::Vibratile,
                f0_min: 200.0,
                f0_max: 500.0,
                f0_default: 300.0,
                tract_scale: 0.03,
                formants: [300.0, 600.0, 900.0], // Wing resonance
                bandwidths: [100.0, 200.0, 300.0],
                breathiness: 0.0,
                jitter: 0.001,
                shimmer: 0.003,
                spectral_tilt: -1.0,
            },
            Self::Dragon => SpeciesParams {
                apparatus: VocalApparatus::Laryngeal,
                f0_min: 30.0,
                f0_max: 500.0,
                f0_default: 70.0,
                tract_scale: 3.0,
                formants: [170.0, 500.0, 850.0], // ~50cm+ tract
                bandwidths: [60.0, 80.0, 110.0],
                breathiness: 0.2,
                jitter: 0.04,
                shimmer: 0.08,
                spectral_tilt: -4.0,
            },
            Self::Fantasy => SpeciesParams {
                apparatus: VocalApparatus::Laryngeal,
                f0_min: 50.0,
                f0_max: 2000.0,
                f0_default: 200.0,
                tract_scale: 1.0,
                formants: [500.0, 1500.0, 2500.0], // Human-like default
                bandwidths: [80.0, 100.0, 130.0],
                breathiness: 0.05,
                jitter: 0.015,
                shimmer: 0.03,
                spectral_tilt: -3.0,
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
            VocalApparatus::Vibratile => {
                // Wing vibration: buzz and chirp
                matches!(v, Vocalization::Buzz | Vocalization::Chirp)
            }
            VocalApparatus::Laryngeal | VocalApparatus::Syringeal => {
                // Laryngeal/syringeal can do most things, but not stridulate/buzz
                !matches!(v, Vocalization::Stridulate | Vocalization::Buzz)
            }
        }
    }

    /// Returns a typical call bout template for this species.
    ///
    /// Each species has a characteristic calling pattern: dogs bark in rapid
    /// bursts, wolves howl with long pauses, crickets stridulate continuously
    /// with inter-chirp gaps, etc.
    #[must_use]
    pub fn bout_template(self) -> crate::sequence::CallBout {
        use crate::vocalization::{CallIntent, Vocalization};
        match self {
            Self::Wolf => crate::sequence::CallBout {
                vocalization: Vocalization::Howl,
                count: 3,
                call_duration: 3.0,
                interval: 2.0,
                intent: CallIntent::Social,
            },
            Self::Dog => crate::sequence::CallBout {
                vocalization: Vocalization::Bark,
                count: 5,
                call_duration: 0.15,
                interval: 0.25,
                intent: CallIntent::Alarm,
            },
            Self::Cat => crate::sequence::CallBout {
                vocalization: Vocalization::Howl, // meow
                count: 2,
                call_duration: 0.6,
                interval: 1.0,
                intent: CallIntent::Social,
            },
            Self::Lion => crate::sequence::CallBout {
                vocalization: Vocalization::Roar,
                count: 4,
                call_duration: 2.0,
                interval: 1.5,
                intent: CallIntent::Territorial,
            },
            Self::Songbird => crate::sequence::CallBout {
                vocalization: Vocalization::Trill,
                count: 6,
                call_duration: 0.3,
                interval: 0.15,
                intent: CallIntent::Mating,
            },
            Self::Crow => crate::sequence::CallBout {
                vocalization: Vocalization::Screech, // caw
                count: 4,
                call_duration: 0.2,
                interval: 0.3,
                intent: CallIntent::Alarm,
            },
            Self::Raptor => crate::sequence::CallBout {
                vocalization: Vocalization::Screech,
                count: 2,
                call_duration: 0.8,
                interval: 1.5,
                intent: CallIntent::Territorial,
            },
            Self::Snake => crate::sequence::CallBout {
                vocalization: Vocalization::Hiss,
                count: 1,
                call_duration: 1.5,
                interval: 0.0,
                intent: CallIntent::Threat,
            },
            Self::Crocodilian => crate::sequence::CallBout {
                vocalization: Vocalization::Rumble,
                count: 3,
                call_duration: 2.5,
                interval: 2.0,
                intent: CallIntent::Territorial,
            },
            Self::Cricket => crate::sequence::CallBout {
                vocalization: Vocalization::Stridulate,
                count: 8,
                call_duration: 0.5,
                interval: 0.3,
                intent: CallIntent::Mating,
            },
            Self::Bee => crate::sequence::CallBout {
                vocalization: Vocalization::Buzz,
                count: 1,
                call_duration: 2.0,
                interval: 0.0,
                intent: CallIntent::Idle,
            },
            Self::Dragon => crate::sequence::CallBout {
                vocalization: Vocalization::Roar,
                count: 2,
                call_duration: 3.0,
                interval: 2.0,
                intent: CallIntent::Threat,
            },
            Self::Fantasy => crate::sequence::CallBout {
                vocalization: Vocalization::Growl,
                count: 3,
                call_duration: 1.0,
                interval: 1.0,
                intent: CallIntent::Idle,
            },
        }
    }
}
