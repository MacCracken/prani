//! Creature-specific vocal tract models.
//!
//! Wraps svara's `VocalTract` and `FormantFilter` with species-specific
//! configurations: tract length scaling, additional resonance chambers,
//! and non-human vocal apparatus characteristics.
//!
//! Synthesis strategy depends on the vocal apparatus:
//! - **Laryngeal/Syringeal**: Glottal source through formant filter (mammals, birds)
//! - **NoiseOnly**: Filtered broadband noise (snakes)
//! - **Stridulatory**: Amplitude-modulated tone (insects)

use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use svara::formant::{Formant, FormantFilter, VowelTarget};
use svara::glottal::GlottalSource;
use svara::tract::VocalTract;

use crate::error::Result;
use crate::rng::Rng;
use crate::species::{SpeciesParams, VocalApparatus};

/// A creature's vocal tract, configured for a specific species.
///
/// Wraps svara's `VocalTract` with species-specific formant targets
/// and excitation source selection based on the vocal apparatus type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatureTract {
    /// Inner svara vocal tract.
    tract: VocalTract,
    /// Species vocal parameters.
    params: SpeciesParams,
    /// PRNG for noise-based synthesis.
    rng: Rng,
    /// Sample rate.
    sample_rate: f32,
}

impl CreatureTract {
    /// Creates a new creature vocal tract for the given species parameters.
    ///
    /// Configures the svara vocal tract with species-specific formant
    /// frequencies derived from the species' vocal tract geometry.
    #[must_use]
    pub fn new(params: &SpeciesParams, sample_rate: f32) -> Self {
        let mut tract = VocalTract::new(sample_rate);

        // Configure species-specific formants on the vocal tract
        let f = &params.formants;
        let b = &params.bandwidths;
        let target = VowelTarget::with_bandwidths(
            [f[0], f[1], f[2], f[2] * 1.3, f[2] * 1.5], // F4/F5 estimated
            [b[0], b[1], b[2], b[2] * 1.2, b[2] * 1.4],
        );
        // Ignore error — fallback to default if formants are out of range
        let _ = tract.set_formants_from_target(&target);

        Self {
            tract,
            params: params.clone(),
            rng: Rng::new(params.resonance_seed()),
            sample_rate,
        }
    }

    /// Synthesizes a block of samples using the creature's vocal apparatus.
    ///
    /// For laryngeal/syringeal species, pipes a glottal source through the
    /// vocal tract. For noise-only species, generates filtered noise.
    /// For stridulatory species, generates amplitude-modulated tone.
    #[inline]
    pub fn synthesize(&mut self, f0: f32, num_samples: usize) -> Result<Vec<f32>> {
        match self.params.apparatus {
            VocalApparatus::Laryngeal => {
                // Clamp to species f0 range, within svara's GlottalSource limits
                let clamped_f0 =
                    f0.clamp(self.params.f0_min.max(20.0), self.params.f0_max.min(2000.0));
                let mut glottal = GlottalSource::new(clamped_f0, self.sample_rate)?;
                glottal.set_breathiness(self.params.breathiness);
                glottal.set_jitter(self.params.jitter);
                glottal.set_shimmer(self.params.shimmer);
                Ok(self.tract.synthesize(&mut glottal, num_samples))
            }
            VocalApparatus::Syringeal => {
                // Syrinx can produce f0 above svara's 2000Hz glottal limit.
                // Use GlottalSource for low f0, sine excitation for high f0.
                let clamped_f0 = f0.clamp(self.params.f0_min.max(20.0), self.params.f0_max);
                if clamped_f0 <= 2000.0 {
                    let mut glottal = GlottalSource::new(clamped_f0, self.sample_rate)?;
                    glottal.set_breathiness(self.params.breathiness);
                    glottal.set_jitter(self.params.jitter);
                    glottal.set_shimmer(self.params.shimmer);
                    Ok(self.tract.synthesize(&mut glottal, num_samples))
                } else {
                    // High-frequency syringeal synthesis with dual-source capability.
                    // The syrinx has two independent sound sources (left/right bronchus)
                    // that can produce different pitches simultaneously.
                    let f0_2 = clamped_f0 * 1.498; // ~perfect fifth above
                    let mut output = Vec::with_capacity(num_samples);
                    for i in 0..num_samples {
                        let t = i as f32 / self.sample_rate;
                        let tone1 = crate::math::f32::sin(core::f32::consts::TAU * clamped_f0 * t);
                        let tone2 = crate::math::f32::sin(core::f32::consts::TAU * f0_2 * t) * 0.6;
                        let noise = self.rng.next_f32() * self.params.breathiness;
                        let excitation =
                            (tone1 + tone2) * 0.5 * (1.0 - self.params.breathiness) + noise;
                        output.push(self.tract.process_sample(excitation));
                    }
                    Ok(output)
                }
            }
            VocalApparatus::NoiseOnly => {
                // Filtered broadband noise shaped by species formants
                let f = &self.params.formants;
                let b = &self.params.bandwidths;
                let formants = [Formant::new(f[0], b[0], 0.6), Formant::new(f[1], b[1], 0.3)];
                let mut filter = FormantFilter::new(&formants, self.sample_rate).map_err(|e| {
                    crate::error::PraniError::SynthesisFailed(alloc::format!("{e}"))
                })?;
                let mut output = Vec::with_capacity(num_samples);
                for _ in 0..num_samples {
                    let noise = self.rng.next_f32() * 0.5;
                    output.push(filter.process_sample(noise));
                }
                Ok(output)
            }
            VocalApparatus::Stridulatory => {
                // Amplitude-modulated tone for insects.
                // Modulation rate is species-specific:
                //   Cricket: ~30 Hz chirp rate (wing stroke rate)
                //   Bee: ~200 Hz wing beat (f0 IS the wing frequency)
                //   Cicada: ~100 Hz tymbal click rate
                let mod_rate = if f0 > 1000.0 {
                    // High carrier (cricket): slow chirp modulation
                    30.0 + self.rng.next_f32() * 5.0
                } else {
                    // Low carrier (bee): modulation at half the wing frequency
                    f0 * 0.5
                };

                let mut output = Vec::with_capacity(num_samples);
                for i in 0..num_samples {
                    let t = i as f32 / self.sample_rate;
                    let carrier = crate::math::f32::sin(core::f32::consts::TAU * f0 * t);
                    // Add harmonics for richer tone
                    let h2 = crate::math::f32::sin(core::f32::consts::TAU * f0 * 2.0 * t) * 0.3;
                    let modulator =
                        0.5 + 0.5 * crate::math::f32::sin(core::f32::consts::TAU * mod_rate * t);
                    output.push((carrier + h2) * modulator * 0.25);
                }
                Ok(output)
            }
        }
    }

    /// Sets the formant targets for the tract.
    pub fn set_formants(&mut self, target: &VowelTarget) -> Result<()> {
        self.tract
            .set_formants_from_target(target)
            .map_err(Into::into)
    }

    /// Resets the tract state.
    pub fn reset(&mut self) {
        self.tract.reset();
    }
}
