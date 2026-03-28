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
    /// Running phase accumulator for sample-accurate synthesis across blocks.
    phase: f32,
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
        if let Err(e) = tract.set_formants_from_target(&target) {
            tracing::warn!(?e, "species formants out of range, using defaults");
        }

        Self {
            tract,
            params: params.clone(),
            rng: Rng::new(params.resonance_seed()),
            sample_rate,
            phase: 0.0,
        }
    }

    /// Synthesizes a block of samples using the creature's vocal apparatus.
    ///
    /// For laryngeal/syringeal species, pipes a glottal source through the
    /// vocal tract. For noise-only species, generates filtered noise.
    /// For stridulatory species, generates amplitude-modulated tone.
    ///
    /// `options` carries per-block modifiers (subharmonics, formant blend, etc.)
    /// that are layered on top of the species defaults.
    #[inline]
    pub fn synthesize(
        &mut self,
        f0: f32,
        num_samples: usize,
        options: &SynthesisOptions,
    ) -> Result<Vec<f32>> {
        match self.params.apparatus {
            VocalApparatus::Laryngeal => self.synthesize_laryngeal(f0, num_samples, options),
            VocalApparatus::Syringeal => self.synthesize_syringeal(f0, num_samples, options),
            VocalApparatus::NoiseOnly => self.synthesize_noise(num_samples),
            VocalApparatus::Stridulatory => self.synthesize_stridulatory(f0, num_samples, options),
            VocalApparatus::Vibratile => self.synthesize_vibratile(f0, num_samples),
        }
    }

    /// Laryngeal synthesis: glottal source through vocal tract.
    fn synthesize_laryngeal(
        &mut self,
        f0: f32,
        num_samples: usize,
        options: &SynthesisOptions,
    ) -> Result<Vec<f32>> {
        // Clamp to species f0 range, within svara's GlottalSource limits
        let clamped_f0 = f0.clamp(self.params.f0_min.max(20.0), self.params.f0_max.min(2000.0));
        let mut glottal = GlottalSource::new(clamped_f0, self.sample_rate)?;
        glottal.set_breathiness(self.params.breathiness);
        // Non-stationary perturbation: scale jitter/shimmer by block intensity
        let ps = options.perturbation_scale.max(0.0);
        glottal.set_jitter((self.params.jitter * ps).min(0.05));
        glottal.set_shimmer((self.params.shimmer * ps).min(0.1));

        if options.subharmonic_amp > 0.0 {
            // Inject subharmonics into excitation before tract filtering.
            // This lets formant resonances shape the subharmonic energy.
            let sub_f0 = clamped_f0 * 0.5;
            let mut output = Vec::with_capacity(num_samples);
            let phase_inc = sub_f0 / self.sample_rate;
            for _ in 0..num_samples {
                let glottal_sample = glottal.next_sample();
                let sub = crate::math::f32::sin(core::f32::consts::TAU * self.phase)
                    * options.subharmonic_amp;
                self.phase += phase_inc;
                if self.phase >= 1.0 {
                    self.phase -= 1.0;
                }
                output.push(self.tract.process_sample(glottal_sample + sub));
            }
            Ok(output)
        } else {
            Ok(self.tract.synthesize(&mut glottal, num_samples))
        }
    }

    /// Syringeal synthesis: dual-source capable, high-frequency range.
    fn synthesize_syringeal(
        &mut self,
        f0: f32,
        num_samples: usize,
        options: &SynthesisOptions,
    ) -> Result<Vec<f32>> {
        let clamped_f0 = f0.clamp(self.params.f0_min.max(20.0), self.params.f0_max);
        if clamped_f0 <= 2000.0 {
            let mut glottal = GlottalSource::new(clamped_f0, self.sample_rate)?;
            glottal.set_breathiness(self.params.breathiness);
            let ps = options.perturbation_scale.max(0.0);
            glottal.set_jitter((self.params.jitter * ps).min(0.05));
            glottal.set_shimmer((self.params.shimmer * ps).min(0.1));
            Ok(self.tract.synthesize(&mut glottal, num_samples))
        } else {
            // High-frequency syringeal synthesis with dual-source capability.
            // The syrinx has two independent sound sources (left/right bronchus)
            // that can produce different pitches simultaneously.
            let f0_2 = clamped_f0 * 1.498; // ~perfect fifth above
            let _ = options; // no subharmonics for high-freq syringeal
            let mut output = Vec::with_capacity(num_samples);
            for i in 0..num_samples {
                let t = i as f32 / self.sample_rate;
                let tone1 = crate::math::f32::sin(core::f32::consts::TAU * clamped_f0 * t);
                let tone2 = crate::math::f32::sin(core::f32::consts::TAU * f0_2 * t) * 0.6;
                let noise = self.rng.next_f32() * self.params.breathiness;
                let excitation = (tone1 + tone2) * 0.5 * (1.0 - self.params.breathiness) + noise;
                output.push(self.tract.process_sample(excitation));
            }
            Ok(output)
        }
    }

    /// Noise-only synthesis: filtered broadband noise (snakes).
    fn synthesize_noise(&mut self, num_samples: usize) -> Result<Vec<f32>> {
        let f = &self.params.formants;
        let b = &self.params.bandwidths;
        let formants = [Formant::new(f[0], b[0], 0.6), Formant::new(f[1], b[1], 0.3)];
        let mut filter = FormantFilter::new(&formants, self.sample_rate)
            .map_err(|e| crate::error::PraniError::SynthesisFailed(alloc::format!("{e}")))?;
        let mut output = Vec::with_capacity(num_samples);
        for _ in 0..num_samples {
            let noise = self.rng.next_f32() * 0.5;
            output.push(filter.process_sample(noise));
        }
        Ok(output)
    }

    /// Stridulatory synthesis: amplitude-modulated tone for insects.
    ///
    /// For high-carrier (cricket, f0 > 1000 Hz): discrete pulse-train chirps.
    /// Each chirp is 3-5 wing-stroke pulses at ~30 Hz, with inter-chirp silence
    /// at ~2-3 Hz repetition rate.
    ///
    /// For low-carrier (bee, f0 <= 1000 Hz): continuous wing-beat modulation.
    fn synthesize_stridulatory(
        &mut self,
        f0: f32,
        num_samples: usize,
        options: &SynthesisOptions,
    ) -> Result<Vec<f32>> {
        let _ = options;
        let mut output = Vec::with_capacity(num_samples);

        if f0 > 1000.0 {
            // Cricket-style discrete pulse-train chirps.
            // Each chirp: 3-5 pulses at syllable_rate Hz, repeated at chirp_rate Hz.
            let syllable_rate = 30.0; // wing strokes per second within a chirp
            let chirp_rate = 2.5; // chirps per second
            let pulses_per_chirp = 4u32; // typical field cricket
            let chirp_period = self.sample_rate / chirp_rate;
            let syllable_period = self.sample_rate / syllable_rate;
            let chirp_active = syllable_period * pulses_per_chirp as f32;

            for i in 0..num_samples {
                let t = i as f32 / self.sample_rate;
                let pos_in_chirp = (i as f32) % chirp_period;

                let carrier = crate::math::f32::sin(core::f32::consts::TAU * f0 * t);
                let h2 = crate::math::f32::sin(core::f32::consts::TAU * f0 * 2.0 * t) * 0.3;
                let signal = (carrier + h2) * 0.25;

                if pos_in_chirp < chirp_active {
                    // Within active chirp: pulse envelope per syllable
                    let pos_in_syllable = pos_in_chirp % syllable_period;
                    let syllable_frac = pos_in_syllable / syllable_period;
                    // Sharp onset, quick decay per pulse (wing-stroke shape)
                    let pulse_env = if syllable_frac < 0.1 {
                        syllable_frac / 0.1
                    } else if syllable_frac < 0.5 {
                        1.0
                    } else {
                        1.0 - (syllable_frac - 0.5) / 0.5
                    };
                    output.push(signal * pulse_env);
                } else {
                    // Inter-chirp silence
                    output.push(0.0);
                }
            }
        } else {
            // Bee-style continuous wing-beat modulation
            let mod_rate = f0 * 0.5;
            for i in 0..num_samples {
                let t = i as f32 / self.sample_rate;
                let carrier = crate::math::f32::sin(core::f32::consts::TAU * f0 * t);
                let h2 = crate::math::f32::sin(core::f32::consts::TAU * f0 * 2.0 * t) * 0.3;
                let modulator =
                    0.5 + 0.5 * crate::math::f32::sin(core::f32::consts::TAU * mod_rate * t);
                output.push((carrier + h2) * modulator * 0.25);
            }
        }
        Ok(output)
    }

    /// Synthesizes purring: 25-30 Hz laryngeal muscle cycling.
    ///
    /// Cat purring uses a fundamentally different mechanism than normal phonation.
    /// The laryngeal muscles contract and relax at ~25 Hz, producing a characteristic
    /// asymmetric waveform (longer closed phase). The sharp glottal burst at ~25 Hz
    /// excites the vocal tract resonances, producing harmonics at multiples of 25 Hz
    /// shaped by the tract formants.
    #[inline]
    pub fn synthesize_purr(&mut self, num_samples: usize, purr_f0: f32) -> Result<Vec<f32>> {
        let phase_inc = purr_f0 / self.sample_rate;
        let mut output = Vec::with_capacity(num_samples);
        for _ in 0..num_samples {
            // Asymmetric waveform: longer closed phase (0.6 duty), sharp open burst
            let phase_frac = self.phase;
            let excitation = if phase_frac < 0.4 {
                // Open phase: sharp glottal-like pulse burst
                let t = phase_frac / 0.4;
                let pulse = crate::math::f32::sin(core::f32::consts::PI * t);
                pulse * pulse // squared for sharper pulse
            } else {
                // Closed phase: near silence with slight noise
                self.rng.next_f32() * 0.02
            };
            self.phase += phase_inc;
            if self.phase >= 1.0 {
                self.phase -= 1.0;
            }
            // Sharp pulse excites the vocal tract, producing resonant harmonics
            output.push(self.tract.process_sample(excitation));
        }
        Ok(output)
    }

    /// Updates the formant targets for dynamic formant transitions.
    ///
    /// Blends the current formants toward `target` formants by `blend` factor (0.0-1.0).
    /// At blend=0.0, uses the species default formants. At blend=1.0, uses `target`.
    pub fn set_formant_blend(
        &mut self,
        target: &[f32; 3],
        target_bw: &[f32; 3],
        blend: f32,
    ) -> Result<()> {
        let blend = blend.clamp(0.0, 1.0);
        let f = &self.params.formants;
        let b = &self.params.bandwidths;

        let blended_f = [
            f[0] + (target[0] - f[0]) * blend,
            f[1] + (target[1] - f[1]) * blend,
            f[2] + (target[2] - f[2]) * blend,
        ];
        let blended_b = [
            b[0] + (target_bw[0] - b[0]) * blend,
            b[1] + (target_bw[1] - b[1]) * blend,
            b[2] + (target_bw[2] - b[2]) * blend,
        ];

        let vowel = VowelTarget::with_bandwidths(
            [
                blended_f[0],
                blended_f[1],
                blended_f[2],
                blended_f[2] * 1.3,
                blended_f[2] * 1.5,
            ],
            [
                blended_b[0],
                blended_b[1],
                blended_b[2],
                blended_b[2] * 1.2,
                blended_b[2] * 1.4,
            ],
        );
        self.tract
            .set_formants_from_target(&vowel)
            .map_err(Into::into)
    }

    /// Vibratile synthesis: continuous wing-beat modulation (bees, flies).
    ///
    /// Thoracic flight muscle vibration transmitted through wings.
    /// Produces a steady buzz with AM at half the wing frequency.
    fn synthesize_vibratile(&mut self, f0: f32, num_samples: usize) -> Result<Vec<f32>> {
        let mod_rate = f0 * 0.5;
        let mut output = Vec::with_capacity(num_samples);
        for i in 0..num_samples {
            let t = i as f32 / self.sample_rate;
            let carrier = crate::math::f32::sin(core::f32::consts::TAU * f0 * t);
            let h2 = crate::math::f32::sin(core::f32::consts::TAU * f0 * 2.0 * t) * 0.3;
            let modulator =
                0.5 + 0.5 * crate::math::f32::sin(core::f32::consts::TAU * mod_rate * t);
            output.push((carrier + h2) * modulator * 0.25);
        }
        Ok(output)
    }

    /// Applies spectral tilt (first-order lowpass) to a buffer.
    ///
    /// `tilt_db` is dB/octave: negative = darker (lowpass), 0 = neutral.
    /// Implemented as a one-pole filter: `y[n] = (1-a)*x[n] + a*y[n-1]`.
    pub fn apply_spectral_tilt(samples: &mut [f32], tilt_db: f32, sample_rate: f32) {
        if tilt_db.abs() < 0.1 {
            return;
        }
        // Convert dB/octave to filter coefficient.
        // At -6 dB/octave, we want a = ~0.5 (strong lowpass).
        // At -1 dB/octave, we want a = ~0.1 (mild lowpass).
        let a = (-tilt_db / 12.0).clamp(0.0, 0.85);
        if a < 0.01 {
            return;
        }
        let _ = sample_rate; // coefficient derived from tilt_db directly
        let mut prev = 0.0f32;
        for s in samples.iter_mut() {
            *s = (1.0 - a) * *s + a * prev;
            prev = *s;
        }
    }

    /// Resets the tract state.
    pub fn reset(&mut self) {
        self.tract.reset();
        self.phase = 0.0;
    }
}

/// Per-block synthesis options passed from voice to tract.
#[derive(Debug, Clone, Default)]
pub struct SynthesisOptions {
    /// Subharmonic amplitude (0.0 = none). Injected into excitation before tract.
    pub subharmonic_amp: f32,
    /// Perturbation intensity multiplier (0.0-2.0). Scales jitter/shimmer.
    /// Driven by call urgency and position — higher at call boundaries and peaks.
    pub perturbation_scale: f32,
}
