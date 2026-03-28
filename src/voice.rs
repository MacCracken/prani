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
use crate::tract::{CreatureTract, SynthesisOptions};
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
    /// Vocal effort (0.0 = whisper, 0.5 = normal, 1.0 = shout).
    /// Modulates subglottal pressure (amplitude), spectral slope, formant
    /// bandwidth, and harmonic-to-noise ratio.
    vocal_effort: f32,
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
            vocal_effort: 0.5,
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

    /// Sets the vocal effort (0.0 = whisper, 0.5 = normal, 1.0 = shout).
    ///
    /// Effort modulates:
    /// - **Amplitude**: louder at high effort (subglottal pressure)
    /// - **Spectral slope**: brighter at high effort (more HF energy)
    /// - **Formant bandwidth**: narrower at high effort (sharper resonances)
    /// - **Breathiness**: U-shaped — breathy at whisper and shout extremes
    #[must_use]
    pub fn with_vocal_effort(mut self, effort: f32) -> Self {
        self.vocal_effort = effort.clamp(0.0, 1.0);
        self
    }

    /// Sets the vocal effort mutably (for real-time RTPC updates).
    pub fn set_vocal_effort(&mut self, effort: f32) {
        self.vocal_effort = effort.clamp(0.0, 1.0);
    }

    /// Returns the current vocal effort.
    #[must_use]
    pub fn vocal_effort(&self) -> f32 {
        self.vocal_effort
    }

    /// Applies the Lombard effect: involuntary vocal effort increase in
    /// response to ambient noise level.
    ///
    /// Approximately +3 dB vocal output per 10 dB ambient noise above
    /// a quiet baseline (~40 dB SPL). The boost is additive on top of
    /// the current vocal effort.
    ///
    /// `ambient_spl_db` — ambient sound pressure level in dB SPL.
    pub fn apply_lombard_effect(&mut self, ambient_spl_db: f32) {
        let boost = crate::bridge::lombard_effort_boost(f64::from(ambient_spl_db));
        self.vocal_effort = (self.vocal_effort + boost).clamp(0.0, 1.0);
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

        // Vocal effort modulations:
        // - Amplitude scales 0.3 (whisper) to 1.5 (shout)
        // - Spectral tilt offset: +3 dB/oct at shout (brighter), -2 dB/oct at whisper (darker)
        // - Breathiness: U-shaped — breathy at whisper (0.0) and shout (1.0), clear at normal (0.5)
        let effort = self.vocal_effort;
        let effort_amp = 0.3 + effort * 1.2;
        let effort_tilt_offset = (effort - 0.5) * 6.0; // -3..+3 dB/oct
        let effort_breathiness_delta = (effort - 0.5).abs() * 0.3; // 0 at 0.5, +0.15 at extremes

        trace!(
            species = ?self.species,
            ?vocalization,
            ?intent,
            f0,
            num_samples,
            effort,
            "synthesizing creature vocalization"
        );

        // Cat purr uses a special synthesis path (25-30 Hz laryngeal muscle cycling)
        if *vocalization == Vocalization::Purr && self.species == Species::Cat {
            return self.synthesize_cat_purr(sample_rate, num_samples, &modifiers);
        }

        // Build a working copy of params with effort-adjusted breathiness
        let mut effort_params = self.params.clone();
        effort_params.breathiness =
            (effort_params.breathiness + effort_breathiness_delta).clamp(0.0, 1.0);

        let mut tract = CreatureTract::new(&effort_params, sample_rate);

        // Get formant transition contour (if applicable)
        let formant_contour = formant_transition_contour(vocalization, self.species);

        // Synthesize with vocalization-specific pitch contour
        let contour = pitch_contour(vocalization, f0);
        let block_size = (sample_rate * 0.02) as usize; // 20ms blocks
        let mut samples = Vec::with_capacity(num_samples);
        let mut rendered = 0;
        while rendered < num_samples {
            let block_len = block_size.min(num_samples - rendered);
            let t = rendered as f32 / num_samples.max(1) as f32;
            let block_f0 = contour.f0_at(t);

            // Apply formant transition if this vocalization has one
            if let Some(ref fc) = formant_contour {
                let (target_f, target_b, blend) = fc.at(t);
                let _ = tract.set_formant_blend(&target_f, &target_b, blend);
            }

            // Source-filter coupling for birds: the syrinx actively tunes
            // the vocal tract so F1 tracks near the current f0. We nudge
            // the species' F1 toward block_f0 with 40% coupling strength.
            if self.params.apparatus == crate::species::VocalApparatus::Syringeal {
                let f = &self.params.formants;
                let b = &self.params.bandwidths;
                let coupled_f1 = f[0] + (block_f0 - f[0]) * 0.4;
                let _ = tract.set_formant_blend(&[coupled_f1, f[1], f[2]], b, 1.0);
            }

            // Non-stationary perturbation: jitter/shimmer increase at call
            // boundaries and during high-urgency calls.
            // Base = 1.0, peaks at onset/offset (~1.5x), urgency adds up to 1x.
            let boundary_boost = if t < 0.1 {
                1.0 + (1.0 - t / 0.1) * 0.5
            } else if t > 0.85 {
                1.0 + (t - 0.85) / 0.15 * 0.5
            } else {
                1.0
            };
            let perturbation_scale = boundary_boost + modifiers.urgency;

            let options = SynthesisOptions {
                perturbation_scale,
                ..SynthesisOptions::default()
            };

            let block = tract.synthesize(block_f0, block_len, &options)?;
            samples.extend_from_slice(&block);
            rendered += block_len;
        }

        // Add subharmonics with time-varying amplitude for species that produce them.
        // Amplitude peaks during the middle of the call (strongest during full roar).
        // Added post-tract because svara's VocalTract formant filters attenuate
        // frequencies far below F1.
        if matches!(
            self.species,
            Species::Lion | Species::Dragon | Species::Crocodilian
        ) {
            let sub_f0 = f0 * 0.5;
            let mut chaos_rng =
                crate::rng::Rng::new(self.params.resonance_seed().wrapping_add(0xCA05));
            for (i, sample) in samples.iter_mut().enumerate() {
                let t_norm = i as f32 / num_samples.max(1) as f32;
                let t_sec = i as f32 / sample_rate;

                // Time-varying subharmonic: peaks at 0.3-0.7 of the call
                let sub_env = if t_norm < 0.15 {
                    t_norm / 0.15
                } else if t_norm > 0.8 {
                    (1.0 - t_norm) / 0.2
                } else {
                    1.0
                };
                let sub_amp = 0.4 * sub_env;
                *sample += crate::math::f32::sin(core::f32::consts::TAU * sub_f0 * t_sec) * sub_amp;

                // Deterministic chaos: noise modulated by subharmonic period.
                // Adds roughness during peak intensity (period doubling -> chaos).
                // Characteristic of lion roars cycling through tonal -> chaotic regimes.
                if sub_env > 0.6 {
                    let chaos_intensity = (sub_env - 0.6) / 0.4; // 0..1 during peak
                    let chaos_noise = chaos_rng.next_f32() * 0.15 * chaos_intensity;
                    *sample += chaos_noise;
                }
            }
        }

        // Dragon fire-breath: mix broadband turbulent noise with the vocalization.
        if self.species == Species::Dragon {
            let mut fire_rng =
                crate::rng::Rng::new(self.params.resonance_seed().wrapping_add(0xF12E));
            let fire_amp = self.params.breathiness * 0.4;
            for (i, sample) in samples.iter_mut().enumerate() {
                let t = i as f32 / num_samples.max(1) as f32;
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

        // Biphonation for canids: add a second independent pitch from the same larynx.
        // Documented in wolves and foxes. The second pitch is typically a non-harmonic
        // interval (minor seventh or tritone), creating a distinctive two-toned quality.
        if matches!(self.species, Species::Wolf | Species::Dog)
            && matches!(vocalization, Vocalization::Howl | Vocalization::Whine)
        {
            let bipho_f0 = f0 * 1.78; // ~minor seventh interval
            let bipho_amp = 0.15; // Subtle secondary pitch
            for (i, sample) in samples.iter_mut().enumerate() {
                let t = i as f32 / sample_rate;
                let t_norm = i as f32 / num_samples.max(1) as f32;
                // Biphonation appears in the middle section of the call
                let bipho_env = if !(0.3..=0.8).contains(&t_norm) {
                    0.0
                } else {
                    let mid = (t_norm - 0.3) / 0.5;
                    if mid < 0.3 {
                        mid / 0.3
                    } else if mid > 0.7 {
                        (1.0 - mid) / 0.3
                    } else {
                        1.0
                    }
                };
                *sample += crate::math::f32::sin(core::f32::consts::TAU * bipho_f0 * t)
                    * bipho_amp
                    * bipho_env;
            }
        }

        // Nasal resonance: apply anti-formant (spectral notch) during nasal phases.
        // Cat meow starts nasal ("m"), wolf howl has nasal onset.
        // Implemented as a notch at ~250 Hz (nasal anti-formant frequency).
        if let Some(nasal_phase) = nasal_phase_fraction(vocalization, self.species) {
            apply_nasal_antiformant(&mut samples, sample_rate, nasal_phase);
        }

        // Apply vocalization-specific AM patterns (bird trills, purr cycling)
        apply_am_pattern(&mut samples, vocalization, sample_rate);

        // Apply spectral tilt: species base + vocalization + vocal effort.
        // High effort = brighter (positive offset), whisper = darker (negative).
        let vocalization_tilt = vocalization_spectral_offset(vocalization);
        CreatureTract::apply_spectral_tilt(
            &mut samples,
            self.params.spectral_tilt + vocalization_tilt + effort_tilt_offset,
            sample_rate,
        );

        // Apply amplitude scaling from intent × vocal effort
        let amp = modifiers.amplitude_scale * effort_amp;
        for s in &mut samples {
            *s *= amp;
        }

        // Apply amplitude envelope based on vocalization shape
        apply_vocalization_envelope(&mut samples, vocalization, sample_rate);

        Ok(samples)
    }

    /// Synthesizes a cat purr using 25-30 Hz laryngeal muscle cycling.
    ///
    /// This is fundamentally different from normal vocal fold vibration.
    /// The laryngeal muscles contract/relax at ~25 Hz, producing an asymmetric
    /// waveform with formant-filtered resonance.
    fn synthesize_cat_purr(
        &self,
        sample_rate: f32,
        num_samples: usize,
        modifiers: &crate::vocalization::IntentModifiers,
    ) -> Result<Vec<f32>> {
        let mut tract = CreatureTract::new(&self.params, sample_rate);

        // Purr f0 is 25-30 Hz, independent of the species' vocal f0 range.
        // Individual variation via size: larger cat = slightly lower purr.
        let purr_f0 = (27.0 / self.size_scale).clamp(20.0, 35.0);

        let mut samples = tract.synthesize_purr(num_samples, purr_f0)?;

        // Apply amplitude scaling from intent × vocal effort
        let effort_amp = 0.3 + self.vocal_effort * 1.2;
        let amp = modifiers.amplitude_scale * effort_amp;
        for s in &mut samples {
            *s *= amp;
        }

        // Purr envelope: gentle fade-in/fade-out
        apply_vocalization_envelope(&mut samples, &Vocalization::Purr, sample_rate);

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

/// Formant transition contour — defines how formants change during a vocalization.
///
/// Used for vocalizations where mouth shape changes over time (cat meow, wolf howl).
struct FormantTransitionContour {
    /// Keyframes: (time, target_formants_F1F2F3, target_bandwidths, blend_amount).
    /// blend_amount = how far from species default toward the target at this keyframe.
    keyframes: &'static [(f32, [f32; 3], [f32; 3], f32)],
}

impl FormantTransitionContour {
    /// Returns the (target_formants, target_bandwidths, blend) at normalized time t.
    fn at(&self, t: f32) -> ([f32; 3], [f32; 3], f32) {
        let t = t.clamp(0.0, 1.0);
        if self.keyframes.is_empty() {
            return ([0.0; 3], [0.0; 3], 0.0);
        }
        for i in 0..self.keyframes.len() - 1 {
            let (t0, f0, b0, bl0) = self.keyframes[i];
            let (t1, f1, b1, bl1) = self.keyframes[i + 1];
            if t >= t0 && t <= t1 {
                let frac = if (t1 - t0).abs() < f32::EPSILON {
                    0.0
                } else {
                    (t - t0) / (t1 - t0)
                };
                let f = [
                    f0[0] + (f1[0] - f0[0]) * frac,
                    f0[1] + (f1[1] - f0[1]) * frac,
                    f0[2] + (f1[2] - f0[2]) * frac,
                ];
                let b = [
                    b0[0] + (b1[0] - b0[0]) * frac,
                    b0[1] + (b1[1] - b0[1]) * frac,
                    b0[2] + (b1[2] - b0[2]) * frac,
                ];
                let bl = bl0 + (bl1 - bl0) * frac;
                return (f, b, bl);
            }
        }
        // SAFETY: all static keyframe slices have at least one entry;
        // the empty check above handles the degenerate case.
        let last = match self.keyframes.last() {
            Some(kf) => kf,
            None => return ([0.0; 3], [0.0; 3], 0.0),
        };
        (last.1, last.2, last.3)
    }
}

/// Returns a formant transition contour for vocalizations with dynamic mouth shape.
fn formant_transition_contour(
    v: &Vocalization,
    species: Species,
) -> Option<FormantTransitionContour> {
    // Cat meow: "m" (nasal, closed) -> "e" (open) -> "ow" (closing)
    // Formant targets represent the open-mouth position; blend controls transition.
    static CAT_MEOW_HOWL: &[(f32, [f32; 3], [f32; 3], f32)] = &[
        // Start: mouth mostly closed (nasal "m"), low F1
        (0.0, [400.0, 2000.0, 3500.0], [150.0, 200.0, 250.0], 0.8),
        // Mid: mouth open ("e"/"a"), high F1, wide formants
        (0.4, [1000.0, 2800.0, 4500.0], [100.0, 130.0, 180.0], 1.0),
        // End: mouth closing ("ow"), F1 drops back
        (1.0, [500.0, 1800.0, 3800.0], [140.0, 180.0, 220.0], 0.6),
    ];

    // Wolf howl: gradual mouth opening then closing, subtle formant shift
    static WOLF_HOWL: &[(f32, [f32; 3], [f32; 3], f32)] = &[
        // Start: neutral
        (0.0, [420.0, 1250.0, 2100.0], [100.0, 120.0, 150.0], 0.0),
        // Peak: mouth slightly more open
        (0.3, [500.0, 1400.0, 2300.0], [90.0, 110.0, 140.0], 0.6),
        // Sustained
        (0.7, [480.0, 1350.0, 2200.0], [95.0, 115.0, 145.0], 0.4),
        // Close
        (1.0, [400.0, 1200.0, 2000.0], [110.0, 130.0, 160.0], 0.3),
    ];

    match (v, species) {
        (Vocalization::Howl | Vocalization::Whine, Species::Cat) => {
            Some(FormantTransitionContour {
                keyframes: CAT_MEOW_HOWL,
            })
        }
        (Vocalization::Howl, Species::Wolf | Species::Dog) => Some(FormantTransitionContour {
            keyframes: WOLF_HOWL,
        }),
        _ => None,
    }
}

/// Returns the fraction of the call that is nasal (0.0-1.0 from start).
/// None if no nasal phase applies.
fn nasal_phase_fraction(v: &Vocalization, species: Species) -> Option<f32> {
    match (v, species) {
        // Cat meow: nasal "m" at the start (~20% of call)
        (Vocalization::Howl | Vocalization::Whine, Species::Cat) => Some(0.2),
        // Wolf howl: brief nasal onset (~10%)
        (Vocalization::Howl, Species::Wolf | Species::Dog) => Some(0.1),
        _ => None,
    }
}

/// Applies a nasal anti-formant (notch filter) during the nasal phase of a call.
///
/// The notch at ~250 Hz simulates the coupling of the nasal cavity, which
/// introduces a spectral zero (anti-resonance) characteristic of nasalized sounds.
fn apply_nasal_antiformant(samples: &mut [f32], sample_rate: f32, nasal_fraction: f32) {
    let nasal_len = (samples.len() as f32 * nasal_fraction) as usize;
    if nasal_len == 0 {
        return;
    }

    // Simple notch at 250 Hz using a second-order IIR
    let notch_freq = 250.0;
    let notch_bw = 80.0; // bandwidth of the notch
    let r = 1.0 - (core::f32::consts::PI * notch_bw / sample_rate);
    let r = r.clamp(0.5, 0.999);
    let theta = core::f32::consts::TAU * notch_freq / sample_rate;
    let cos_theta = crate::math::f32::sin(core::f32::consts::PI / 2.0 - theta); // cos via sin

    // Notch filter: H(z) = (1 - 2cos(θ)z^-1 + z^-2) / (1 - 2r·cos(θ)z^-1 + r²z^-2)
    let a1 = -2.0 * r * cos_theta;
    let a2 = r * r;
    let b1 = -2.0 * cos_theta;

    let mut x1 = 0.0f32;
    let mut x2 = 0.0f32;
    let mut y1 = 0.0f32;
    let mut y2 = 0.0f32;

    for (i, sample) in samples.iter_mut().enumerate().take(nasal_len) {
        let x0 = *sample;
        let y0 = x0 + b1 * x1 + x2 - a1 * y1 - a2 * y2;
        x2 = x1;
        x1 = x0;
        y2 = y1;
        y1 = y0;

        // Fade the notch effect: full at start, fading out by end of nasal phase
        let fade = 1.0 - (i as f32 / nasal_len as f32);
        *sample = *sample * (1.0 - fade) + y0 * fade;
    }
}

/// Returns a spectral tilt offset (dB/octave) for a given vocalization type.
///
/// Layered on top of the species spectral tilt. Negative = darker.
#[must_use]
fn vocalization_spectral_offset(v: &Vocalization) -> f32 {
    match v {
        // Growls and rumbles are darker
        Vocalization::Growl | Vocalization::Rumble => -2.0,
        // Roars have moderate darkness
        Vocalization::Roar => -1.0,
        // Screeches and chirps are brighter
        Vocalization::Screech | Vocalization::Chirp => 1.5,
        // Hisses are HF-heavy
        Vocalization::Hiss => 2.0,
        // Trills are slightly bright
        Vocalization::Trill => 0.5,
        // Purr, howl, bark, whine, yelp — neutral
        _ => 0.0,
    }
}

/// Applies vocalization-specific amplitude modulation patterns.
///
/// Bird trills get rapid AM at species-typical rates.
fn apply_am_pattern(samples: &mut [f32], vocalization: &Vocalization, sample_rate: f32) {
    let am_rate = match vocalization {
        // Bird trills: rapid AM at 15-30 Hz typical for songbird trills
        Vocalization::Trill => 20.0,
        _ => return,
    };

    for (i, sample) in samples.iter_mut().enumerate() {
        let t = i as f32 / sample_rate;
        // Sinusoidal AM: modulate between 0.3 and 1.0 (never fully silent)
        let modulator = 0.65 + 0.35 * crate::math::f32::sin(core::f32::consts::TAU * am_rate * t);
        *sample *= modulator;
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
