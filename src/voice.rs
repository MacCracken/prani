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
    #[must_use]
    pub fn with_f0_offset(mut self, offset: f32) -> Self {
        self.f0_offset = offset;
        self
    }

    /// Sets the individual size scale (affects tract length and f0).
    ///
    /// Values > 1.0 = larger creature (lower pitch, longer tract).
    /// Values < 1.0 = smaller creature (higher pitch, shorter tract).
    #[must_use]
    pub fn with_size(mut self, scale: f32) -> Self {
        self.size_scale = scale.max(0.1);
        self
    }

    /// Sets the breathiness.
    #[must_use]
    pub fn with_breathiness(mut self, breathiness: f32) -> Self {
        self.params.breathiness = breathiness.clamp(0.0, 1.0);
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
        let mut samples = tract.synthesize(f0, num_samples)?;

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

/// Applies a vocalization-specific amplitude envelope.
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
