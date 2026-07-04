//! Pull-based streaming synthesis.
//!
//! `SynthStream` wraps a `CreatureVoice` and yields audio blocks on demand,
//! avoiding the full-buffer allocation of [`CreatureVoice::vocalize`].
//! Suitable for real-time audio callbacks where block size is determined
//! by the host (Wwise, FMOD, Godot, JACK).
//!
//! ```rust
//! use prani::prelude::*;
//! use prani::stream::SynthStream;
//!
//! let voice = CreatureVoice::new(Species::Wolf);
//! let mut stream = SynthStream::new(
//!     voice, Vocalization::Howl, CallIntent::Social, 44100.0, 2.0,
//! ).expect("valid vocalization");
//!
//! // Pull blocks on demand (e.g., in an audio callback)
//! let mut buffer = vec![0.0f32; 512];
//! while !stream.is_finished() {
//!     let written = stream.fill_buffer(&mut buffer);
//!     // ... send buffer[..written] to audio output ...
//! }
//! ```

use alloc::vec::Vec;

use crate::error::Result;
use crate::species::Species;
use crate::tract::{CreatureTract, SynthesisOptions};
use crate::vocalization::{CallIntent, Vocalization};
use crate::voice::CreatureVoice;

/// A pull-based streaming synthesizer.
///
/// Produces audio block-by-block without allocating the full output buffer
/// up front. Each call to [`fill_buffer`](Self::fill_buffer) or
/// [`next_block`](Self::next_block) advances the stream.
pub struct SynthStream {
    /// The voice producing this stream.
    voice: CreatureVoice,
    /// Vocalization being synthesized.
    vocalization: Vocalization,
    /// Behavioral intent.
    intent: CallIntent,
    /// Sample rate (Hz).
    sample_rate: f32,
    /// Total samples to produce.
    total_samples: usize,
    /// Samples produced so far.
    samples_rendered: usize,
    /// The configured vocal tract (lazily initialized on first block).
    tract: Option<CreatureTract>,
    /// Pitch contour base f0 (after intent modifiers).
    f0: f32,
    /// Intent modifiers snapshot.
    amplitude_scale: f32,
    /// Vocal effort amplitude scale.
    effort_amp: f32,
    /// Combined spectral tilt.
    spectral_tilt: f32,
    /// Whether this is a cat purr (special path).
    is_cat_purr: bool,
}

impl SynthStream {
    /// Creates a new streaming synthesizer.
    ///
    /// Does not allocate the full output — audio is produced incrementally
    /// via [`fill_buffer`](Self::fill_buffer) or [`next_block`](Self::next_block).
    ///
    /// # Errors
    ///
    /// Returns `PraniError::InvalidVocalization` if the species cannot produce
    /// the requested vocalization.
    pub fn new(
        voice: CreatureVoice,
        vocalization: Vocalization,
        intent: CallIntent,
        sample_rate: f32,
        duration: f32,
    ) -> Result<Self> {
        if !voice.species().supports_vocalization(&vocalization) {
            return Err(crate::error::PraniError::InvalidVocalization(
                alloc::format!(
                    "{:?} cannot produce {:?} — incompatible vocal apparatus",
                    voice.species(),
                    vocalization
                ),
            ));
        }

        let modifiers = intent.modifiers();
        let f0 = voice.effective_f0() * modifiers.pitch_scale;
        let effective_duration = duration * modifiers.duration_scale;
        let total_samples = (effective_duration * sample_rate) as usize;

        let effort = voice.vocal_effort();
        let effort_amp = 0.3 + effort * 1.2;
        let effort_tilt_offset = (effort - 0.5) * 6.0;
        let vocalization_tilt = vocalization_spectral_offset(&vocalization);
        let species_params = voice.species().params();
        let spectral_tilt = species_params.spectral_tilt + vocalization_tilt + effort_tilt_offset;

        let is_cat_purr = vocalization == Vocalization::Purr && voice.species() == Species::Cat;

        Ok(Self {
            voice,
            vocalization,
            intent,
            sample_rate,
            total_samples,
            samples_rendered: 0,
            tract: None,
            f0,
            amplitude_scale: modifiers.amplitude_scale,
            effort_amp,
            spectral_tilt,
            is_cat_purr,
        })
    }

    /// Returns true when all samples have been produced.
    #[must_use]
    pub fn is_finished(&self) -> bool {
        self.samples_rendered >= self.total_samples
    }

    /// Returns the total number of samples the stream will produce.
    #[must_use]
    pub fn total_samples(&self) -> usize {
        self.total_samples
    }

    /// Returns the number of samples produced so far.
    #[must_use]
    pub fn samples_rendered(&self) -> usize {
        self.samples_rendered
    }

    /// Fills the provided buffer with audio samples.
    ///
    /// Returns the number of samples actually written (may be less than
    /// `buffer.len()` at the end of the stream).
    pub fn fill_buffer(&mut self, buffer: &mut [f32]) -> usize {
        if self.is_finished() {
            return 0;
        }

        let remaining = self.total_samples - self.samples_rendered;
        let to_render = buffer.len().min(remaining);

        // Ensure tract is initialized
        let tract = self.tract.get_or_insert_with(|| {
            let params = self.voice.species().params();
            CreatureTract::new(&params, self.sample_rate)
        });

        if self.is_cat_purr {
            // Cat purr special path
            let purr_f0 = (27.0 / 1.0_f32).clamp(20.0, 35.0); // size_scale already baked in
            if let Ok(block) = tract.synthesize_purr(to_render, purr_f0) {
                buffer[..to_render].copy_from_slice(&block[..to_render]);
            }
        } else {
            // Standard synthesis with pitch contour
            let t = self.samples_rendered as f32 / self.total_samples.max(1) as f32;
            let contour = pitch_contour_at(&self.vocalization, self.f0, t);

            let modifiers = self.intent.modifiers();
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

            if let Ok(block) = tract.synthesize(contour, to_render, &options) {
                buffer[..to_render].copy_from_slice(&block[..to_render]);
            }
        }

        // Apply spectral tilt to this block
        CreatureTract::apply_spectral_tilt(
            &mut buffer[..to_render],
            self.spectral_tilt,
            self.sample_rate,
        );

        // Apply amplitude
        let amp = self.amplitude_scale * self.effort_amp;
        for s in buffer[..to_render].iter_mut() {
            *s *= amp;
        }

        self.samples_rendered += to_render;
        to_render
    }

    /// Produces the next block of audio as an owned `Vec`.
    ///
    /// `block_size` — number of samples to produce (will be clamped to remaining).
    pub fn next_block(&mut self, block_size: usize) -> Vec<f32> {
        let remaining = self.total_samples.saturating_sub(self.samples_rendered);
        let actual = block_size.min(remaining);
        if actual == 0 {
            return Vec::new();
        }
        let mut buf = alloc::vec![0.0f32; actual];
        self.fill_buffer(&mut buf);
        buf
    }
}

/// Returns the pitch contour f0 at a given normalized time for a vocalization.
fn pitch_contour_at(v: &Vocalization, base_f0: f32, t: f32) -> f32 {
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

    let t = t.clamp(0.0, 1.0);
    if points.is_empty() {
        return base_f0;
    }
    for i in 0..points.len() - 1 {
        let (t0, v0) = points[i];
        let (t1, v1) = points[i + 1];
        if t >= t0 && t <= t1 {
            let frac = if (t1 - t0).abs() < f32::EPSILON {
                0.0
            } else {
                (t - t0) / (t1 - t0)
            };
            return base_f0 * (v0 + (v1 - v0) * frac);
        }
    }
    base_f0 * points.last().map_or(1.0, |p| p.1)
}

/// Returns a spectral tilt offset for a vocalization (duplicated from voice.rs
/// to keep stream.rs self-contained without circular dependencies).
fn vocalization_spectral_offset(v: &Vocalization) -> f32 {
    match v {
        Vocalization::Growl | Vocalization::Rumble => -2.0,
        Vocalization::Roar => -1.0,
        Vocalization::Screech | Vocalization::Chirp => 1.5,
        Vocalization::Hiss => 2.0,
        Vocalization::Trill => 0.5,
        _ => 0.0,
    }
}
