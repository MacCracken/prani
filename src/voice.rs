//! Creature voice — combines species, individual variation, and synthesis.
//!
//! A `CreatureVoice` is an instance of a species with individual characteristics:
//! slightly different f0, tract dimensions, breathiness. It's the equivalent of
//! svara's `VoiceProfile` for non-human creatures.

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};
use tracing::trace;

use crate::error::Result;
use crate::species::{Species, SpeciesParams};
use crate::tract::CreatureTract;
use crate::vocalization::{CallIntent, Vocalization};

/// A creature's voice — species + individual variation.
///
/// Create from a `Species` preset, then optionally adjust individual parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatureVoice {
    /// The species this voice belongs to.
    species: Species,
    /// Species-specific vocal parameters (may be individually modified).
    params: SpeciesParams,
    /// Individual f0 offset from species default (Hz).
    f0_offset: f32,
    /// Individual size scaling (affects tract length and f0).
    size_scale: f32,
}

impl CreatureVoice {
    /// Creates a new creature voice with default species parameters.
    #[must_use]
    pub fn new(species: Species) -> Self {
        let params = species.params();
        Self {
            species,
            params,
            f0_offset: 0.0,
            size_scale: 1.0,
        }
    }

    /// Sets the individual f0 offset (Hz) from species default.
    ///
    /// Clamped to keep effective f0 within the species range.
    #[must_use]
    pub fn with_f0_offset(mut self, offset: f32) -> Self {
        let range = self.params.f0_max - self.params.f0_min;
        self.f0_offset = offset.clamp(-range, range);
        self
    }

    /// Sets the individual size scale (affects tract length and f0).
    ///
    /// Values > 1.0 = larger creature (lower pitch, longer tract).
    /// Values < 1.0 = smaller creature (higher pitch, shorter tract).
    #[must_use]
    pub fn with_size(mut self, scale: f32) -> Self {
        self.size_scale = scale.clamp(0.1, 10.0);
        self
    }

    /// Sets the breathiness (0.0 = clear, 1.0 = very breathy).
    #[must_use]
    pub fn with_breathiness(mut self, breathiness: f32) -> Self {
        self.params.breathiness = breathiness.clamp(0.0, 1.0);
        self
    }

    /// Sets the jitter (cycle-to-cycle f0 perturbation, 0.0-0.05).
    #[must_use]
    pub fn with_jitter(mut self, jitter: f32) -> Self {
        self.params.jitter = jitter.clamp(0.0, 0.05);
        self
    }

    /// Sets the shimmer (cycle-to-cycle amplitude perturbation, 0.0-0.1).
    #[must_use]
    pub fn with_shimmer(mut self, shimmer: f32) -> Self {
        self.params.shimmer = shimmer.clamp(0.0, 0.1);
        self
    }

    /// Returns the species.
    #[must_use]
    pub fn species(&self) -> Species {
        self.species
    }

    /// Returns the effective f0 for this individual.
    #[must_use]
    pub fn effective_f0(&self) -> f32 {
        let base = self.params.f0_default + self.f0_offset;
        // Larger creatures have lower pitch
        (base / self.size_scale).clamp(self.params.f0_min, self.params.f0_max)
    }

    /// Returns the effective tract scale for this individual.
    #[must_use]
    pub fn effective_tract_scale(&self) -> f32 {
        self.params.tract_scale * self.size_scale
    }

    /// Synthesizes a vocalization with default intent (Idle).
    ///
    /// # Errors
    ///
    /// Returns `PraniError` if synthesis parameters are invalid.
    pub fn vocalize(
        &self,
        vocalization: &Vocalization,
        sample_rate: f32,
        duration: f32,
    ) -> Result<Vec<f32>> {
        self.vocalize_with_intent(vocalization, CallIntent::Idle, sample_rate, duration)
    }

    /// Synthesizes a vocalization with a specific behavioral intent.
    ///
    /// The intent modifies pitch, amplitude, duration, and urgency of the call.
    ///
    /// # Errors
    ///
    /// Returns `PraniError` if synthesis parameters are invalid.
    pub fn vocalize_with_intent(
        &self,
        vocalization: &Vocalization,
        intent: CallIntent,
        sample_rate: f32,
        duration: f32,
    ) -> Result<Vec<f32>> {
        // Validate species can produce this vocalization
        if !self.species.supports_vocalization(vocalization) {
            return Err(crate::error::PraniError::InvalidVocalization(
                alloc::format!(
                    "{:?} cannot produce {:?} — incompatible vocal apparatus",
                    self.species,
                    vocalization
                ),
            ));
        }

        let modifiers = intent.modifiers();
        let f0 = self.effective_f0() * modifiers.pitch_scale;
        let effective_duration = duration * modifiers.duration_scale;
        let num_samples = (effective_duration * sample_rate) as usize;

        trace!(
            species = ?self.species,
            ?vocalization,
            ?intent,
            f0,
            num_samples,
            "synthesizing creature vocalization"
        );

        let mut tract = CreatureTract::new(&self.params, sample_rate);

        // Synthesize with vocalization-specific pitch contour
        let contour = pitch_contour(vocalization, f0);
        let block_size = (sample_rate * 0.02) as usize; // 20ms blocks
        let mut samples = Vec::with_capacity(num_samples);
        let mut rendered = 0;
        while rendered < num_samples {
            let block_len = block_size.min(num_samples - rendered);
            let t = rendered as f32 / num_samples as f32;
            let block_f0 = contour.f0_at(t);
            let block = tract.synthesize(block_f0, block_len)?;
            samples.extend_from_slice(&block);
            rendered += block_len;
        }

        // Add subharmonics for species that produce them (lions, dragons, crocodilians)
        if matches!(
            self.species,
            Species::Lion | Species::Dragon | Species::Crocodilian
        ) {
            let sub_f0 = f0 * 0.5; // Half-frequency subharmonic
            let sub_amp = 0.3; // Subharmonic is quieter than fundamental
            for (i, sample) in samples.iter_mut().enumerate() {
                let t = i as f32 / sample_rate;
                *sample += crate::math::f32::sin(core::f32::consts::TAU * sub_f0 * t) * sub_amp;
            }
        }

        // Dragon fire-breath: mix broadband turbulent noise with the vocalization.
        // Scales with breathiness and matches the roar/growl envelope.
        if self.species == Species::Dragon {
            let mut fire_rng = crate::rng::Rng::new(8888);
            let fire_amp = self.params.breathiness * 0.4;
            for (i, sample) in samples.iter_mut().enumerate() {
                let t = i as f32 / num_samples.max(1) as f32;
                // Fire noise ramps up during the middle of the call
                let fire_env = if t < 0.2 {
                    t / 0.2
                } else if t > 0.8 {
                    (1.0 - t) / 0.2
                } else {
                    1.0
                };
                let noise = fire_rng.next_f32();
                *sample += noise * fire_amp * fire_env;
            }
        }

        // Apply amplitude scaling from intent
        let amp = modifiers.amplitude_scale;
        for s in &mut samples {
            *s *= amp;
        }

        // Apply amplitude envelope based on vocalization shape
        apply_vocalization_envelope(&mut samples, vocalization, sample_rate);

        Ok(samples)
    }
}

/// Pitch contour for a vocalization — defines how f0 varies over time.
struct PitchContour {
    /// Base f0.
    base: f32,
    /// Contour points: (normalized_time, f0_multiplier).
    points: &'static [(f32, f32)],
}

impl PitchContour {
    /// Returns the f0 at normalized time t (0.0-1.0).
    fn f0_at(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        if self.points.is_empty() {
            return self.base;
        }
        // Linear interpolation between contour points
        for i in 0..self.points.len() - 1 {
            let (t0, v0) = self.points[i];
            let (t1, v1) = self.points[i + 1];
            if t >= t0 && t <= t1 {
                let frac = if (t1 - t0).abs() < f32::EPSILON {
                    0.0
                } else {
                    (t - t0) / (t1 - t0)
                };
                return self.base * (v0 + (v1 - v0) * frac);
            }
        }
        self.base * self.points.last().map_or(1.0, |p| p.1)
    }
}

/// Returns the pitch contour for a vocalization type.
fn pitch_contour(v: &Vocalization, base_f0: f32) -> PitchContour {
    use crate::vocalization::Vocalization;
    // Static contour points: (time, f0_multiplier)
    static FLAT: &[(f32, f32)] = &[(0.0, 1.0), (1.0, 1.0)];
    static HOWL: &[(f32, f32)] = &[(0.0, 0.8), (0.3, 1.3), (0.7, 1.2), (1.0, 0.6)];
    static BARK: &[(f32, f32)] = &[(0.0, 1.2), (0.1, 1.0), (1.0, 0.8)];
    static ROAR: &[(f32, f32)] = &[(0.0, 0.9), (0.15, 1.2), (0.5, 1.0), (1.0, 0.7)];
    static WHINE: &[(f32, f32)] = &[(0.0, 1.0), (0.5, 1.4), (1.0, 1.6)];
    static SCREECH: &[(f32, f32)] = &[(0.0, 1.5), (0.3, 1.2), (1.0, 0.7)];
    static TRILL: &[(f32, f32)] = &[(0.0, 1.0), (0.25, 1.1), (0.5, 0.9), (0.75, 1.1), (1.0, 1.0)];

    let points = match v {
        Vocalization::Howl => HOWL,
        Vocalization::Bark | Vocalization::Yelp => BARK,
        Vocalization::Roar | Vocalization::Rumble => ROAR,
        Vocalization::Whine => WHINE,
        Vocalization::Screech => SCREECH,
        Vocalization::Trill => TRILL,
        _ => FLAT,
    };

    PitchContour {
        base: base_f0,
        points,
    }
}

/// Applies a vocalization-specific amplitude envelope.
#[inline]
fn apply_vocalization_envelope(
    samples: &mut [f32],
    vocalization: &Vocalization,
    _sample_rate: f32,
) {
    let len = samples.len();
    if len == 0 {
        return;
    }

    // Get attack/release fractions based on vocalization type
    let (attack_frac, release_frac) = match vocalization {
        Vocalization::Bark | Vocalization::Yelp | Vocalization::Chirp => (0.05, 0.1),
        Vocalization::Howl => (0.15, 0.3),
        Vocalization::Growl | Vocalization::Rumble | Vocalization::Purr => (0.1, 0.1),
        Vocalization::Roar => (0.08, 0.2),
        Vocalization::Hiss => (0.2, 0.3),
        Vocalization::Trill | Vocalization::Stridulate | Vocalization::Buzz => (0.05, 0.05),
        Vocalization::Whine => (0.1, 0.2),
        Vocalization::Screech => (0.03, 0.15),
    };

    let attack_len = (len as f32 * attack_frac) as usize;
    let release_len = (len as f32 * release_frac) as usize;

    // Attack ramp
    for (i, sample) in samples.iter_mut().enumerate().take(attack_len.min(len)) {
        let t = i as f32 / attack_len.max(1) as f32;
        *sample *= hisab::calc::ease_in_out_smooth(t);
    }

    // Release ramp
    for i in 0..release_len.min(len) {
        let idx = len - 1 - i;
        let t = i as f32 / release_len.max(1) as f32;
        samples[idx] *= hisab::calc::ease_in_out_smooth(t);
    }
}
