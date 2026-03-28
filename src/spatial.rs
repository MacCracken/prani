//! Spatial audio effects for creature vocalizations.
//!
//! Provides distance-based attenuation, atmospheric high-frequency absorption,
//! and Doppler pitch shift for moving sound sources.

use alloc::vec::Vec;

/// Applies distance-based attenuation and atmospheric HF absorption.
///
/// Uses inverse-distance law (1/r) for amplitude, with air absorption
/// modeled as a simple first-order lowpass that increases with distance.
///
/// - `distance`: distance in meters from source to listener (clamped to >= 1.0)
/// - `reference_distance`: distance at which amplitude = 1.0 (typically 1.0 m)
#[must_use]
pub fn apply_distance_attenuation(
    samples: &[f32],
    distance: f32,
    reference_distance: f32,
    sample_rate: f32,
) -> Vec<f32> {
    let distance = distance.max(reference_distance);
    let gain = reference_distance / distance;

    // Atmospheric absorption: ~0.005 dB/m at 4 kHz, increasing with frequency.
    // Model as a lowpass filter whose cutoff decreases with distance.
    // At 1m: cutoff ~20 kHz (essentially no filtering).
    // At 100m: cutoff ~2 kHz (noticeable HF loss).
    let cutoff_hz = (20000.0 / (1.0 + distance * 0.02)).max(200.0);
    let rc = 1.0 / (core::f32::consts::TAU * cutoff_hz);
    let dt = 1.0 / sample_rate;
    let alpha = dt / (rc + dt);

    let mut output = Vec::with_capacity(samples.len());
    let mut prev = 0.0f32;
    for &s in samples {
        let filtered = prev + alpha * (s * gain - prev);
        prev = filtered;
        output.push(filtered);
    }
    output
}

/// Applies Doppler pitch shift for a moving sound source.
///
/// `velocity` is the radial velocity in m/s (positive = moving away, negative = approaching).
/// Speed of sound is assumed to be 343 m/s.
///
/// Uses linear interpolation for fractional sample delays.
#[must_use]
pub fn apply_doppler_shift(samples: &[f32], velocity: f32, _sample_rate: f32) -> Vec<f32> {
    const SPEED_OF_SOUND: f32 = 343.0;
    // Clamp velocity to avoid singularities
    let v = velocity.clamp(-SPEED_OF_SOUND * 0.9, SPEED_OF_SOUND * 0.9);
    let ratio = SPEED_OF_SOUND / (SPEED_OF_SOUND + v);

    let len = samples.len();
    let out_len = (len as f32 / ratio) as usize;
    let mut output = Vec::with_capacity(out_len);

    for i in 0..out_len {
        let src_pos = i as f32 * ratio;
        let idx = src_pos as usize;
        let frac = src_pos - idx as f32;

        let s0 = if idx < len { samples[idx] } else { 0.0 };
        let s1 = if idx + 1 < len { samples[idx + 1] } else { 0.0 };
        output.push(s0 + (s1 - s0) * frac);
    }
    output
}
