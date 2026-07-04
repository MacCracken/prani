//! Science crate bridge — pure value conversions.
//!
//! Converts bioacoustic measurements and behavioral state into prani
//! synthesis parameters. These functions accept primitive values so
//! consumers (kiran, joshua) can call them regardless of which science
//! crates are in their dependency graph.
//!
//! ```text
//! jantu (behavior)  ──┐
//! goonj (acoustics)   ┤
//! ushma (thermo)      ├──> bridge functions ──> prani parameters
//! pavan (aerodynamics)┘
//! ```

use crate::species::Species;
use crate::vocalization::CallIntent;

// ---------------------------------------------------------------------------
// Body-mass / allometry → voice parameters
// ---------------------------------------------------------------------------

/// Estimates creature size scale from body mass in kilograms.
///
/// Uses the allometric relationship: vocal tract length scales with
/// body mass^(1/3). Reference mass is 30 kg (wolf-sized).
///
/// Returns a size scale suitable for [`CreatureVoice::with_size`](crate::voice::CreatureVoice::with_size).
#[must_use]
pub fn size_from_body_mass(mass_kg: f64) -> f32 {
    if mass_kg <= 0.0 {
        return 0.1;
    }
    let reference_mass = 30.0_f64; // wolf ~ 30 kg
    (mass_kg / reference_mass).cbrt() as f32
}

/// Estimates f0 offset from body temperature deviation.
///
/// Ectotherms (reptiles, insects) have temperature-dependent call rates.
/// Cricket chirp rate follows Dolbear's law. Warmer = higher f0.
///
/// `temp_c` — body or ambient temperature in Celsius.
/// `baseline_c` — the species' reference temperature (typically 20-25°C).
///
/// Returns an f0 offset in Hz to pass to [`CreatureVoice::with_f0_offset`](crate::voice::CreatureVoice::with_f0_offset).
#[must_use]
pub fn f0_offset_from_temperature(temp_c: f64, baseline_c: f64) -> f32 {
    // ~3% f0 change per °C (Q10 of ~1.3 for ectotherm call rates)
    let delta = temp_c - baseline_c;
    (delta * 0.03 * 100.0) as f32 // scale to Hz-range offset
}

// ---------------------------------------------------------------------------
// Behavioral state → call intent
// ---------------------------------------------------------------------------

/// Maps a threat-level value (0.0–1.0) to the most appropriate call intent.
///
/// Designed for integration with jantu's creature behavior state machine.
#[must_use]
pub fn intent_from_threat_level(threat: f64) -> CallIntent {
    if threat >= 0.8 {
        CallIntent::Distress
    } else if threat >= 0.6 {
        CallIntent::Alarm
    } else if threat >= 0.4 {
        CallIntent::Threat
    } else if threat >= 0.2 {
        CallIntent::Territorial
    } else {
        CallIntent::Idle
    }
}

/// Maps an arousal level (0.0–1.0) to a breathiness value.
///
/// Higher arousal → more breathiness (panting, stressed phonation).
#[must_use]
pub fn breathiness_from_arousal(arousal: f64) -> f32 {
    (arousal * 0.3).clamp(0.0, 1.0) as f32
}

// ---------------------------------------------------------------------------
// Acoustic environment → spatial parameters
// ---------------------------------------------------------------------------

/// Converts sound pressure level (SPL) at source to an amplitude scale.
///
/// Reference: 94 dB SPL ≈ 1.0 amplitude (1 Pa RMS).
#[must_use]
pub fn amplitude_from_spl(spl_db: f64) -> f32 {
    let reference_spl = 94.0;
    let diff = spl_db - reference_spl;
    // 20 * log10(amp) = diff => amp = 10^(diff/20)
    (10.0_f64.powf(diff / 20.0) as f32).clamp(0.0, 10.0)
}

/// Estimates atmospheric HF absorption distance factor from humidity
/// and temperature.
///
/// Higher humidity + higher temperature = less HF absorption at distance.
/// Returns a multiplier for the reference distance in
/// [`apply_distance_attenuation`](crate::spatial::apply_distance_attenuation).
#[must_use]
pub fn absorption_factor(humidity_pct: f64, temp_c: f64) -> f32 {
    // Simplified ISO 9613-1: absorption decreases with humidity up to ~40%
    let humidity_factor = if humidity_pct < 40.0 {
        0.8 + (humidity_pct / 40.0) * 0.2
    } else {
        1.0
    };
    let temp_factor = 1.0 + (temp_c - 20.0) * 0.005;
    (humidity_factor * temp_factor).clamp(0.5, 2.0) as f32
}

// ---------------------------------------------------------------------------
// Wind / aerodynamics → Doppler
// ---------------------------------------------------------------------------

/// Converts wind speed (m/s) and bearing to an effective radial velocity
/// for Doppler shift.
///
/// `wind_speed_ms` — wind speed in meters per second.
/// `angle_rad` — angle between wind direction and source-listener axis
///   (0 = wind blowing source toward listener, π = away).
#[must_use]
pub fn doppler_velocity_from_wind(wind_speed_ms: f64, angle_rad: f64) -> f32 {
    // Project wind onto the source-listener axis
    (wind_speed_ms * angle_rad.cos()) as f32
}

// ---------------------------------------------------------------------------
// RTPC-style continuous parameters (what game AI systems drive)
// ---------------------------------------------------------------------------

/// Maps a valence value (−1.0 = negative, +1.0 = positive) to a pitch scale.
///
/// Positive valence (contentment, playfulness) raises pitch slightly.
/// Negative valence (fear, aggression) lowers pitch.
/// Designed for the emotion state machine's valence axis.
#[must_use]
pub fn pitch_scale_from_valence(valence: f64) -> f32 {
    // ±15% pitch shift across the full valence range
    (1.0 + valence * 0.15).clamp(0.7, 1.3) as f32
}

/// Maps arousal (0.0–1.0) to a vocal effort value (0.0–1.0).
///
/// Low arousal → whisper-level effort. High arousal → shout-level.
/// This is the primary bridge between the emotion model and the
/// vocal effort parameter on `CreatureVoice`.
#[must_use]
pub fn vocal_effort_from_arousal(arousal: f64) -> f32 {
    // Sigmoid-ish: slow start, fast middle, plateaus near 1.0
    let a = arousal.clamp(0.0, 1.0);
    (3.0 * a * a - 2.0 * a * a * a) as f32
}

/// Maps an urgency value (0.0–1.0) to jitter and shimmer scale factors.
///
/// Returns `(jitter_scale, shimmer_scale)`. Higher urgency = more
/// perturbation (vocal instability under stress).
#[must_use]
pub fn perturbation_from_urgency(urgency: f64) -> (f32, f32) {
    let u = urgency.clamp(0.0, 1.0);
    // Jitter scales 1.0–2.5x, shimmer 1.0–2.0x
    let jitter_scale = (1.0 + u * 1.5) as f32;
    let shimmer_scale = (1.0 + u * 1.0) as f32;
    (jitter_scale, shimmer_scale)
}

/// Maps ambient noise SPL to a Lombard effect vocal effort boost.
///
/// The Lombard effect is an involuntary ~3 dB vocal increase per 10 dB
/// ambient noise increase above a quiet baseline (~40 dB SPL).
///
/// Returns an additive effort boost (0.0–0.5) to add to the current
/// vocal effort parameter.
#[must_use]
pub fn lombard_effort_boost(ambient_spl_db: f64) -> f32 {
    let baseline = 40.0; // quiet environment
    if ambient_spl_db <= baseline {
        return 0.0;
    }
    // ~0.05 effort per 10 dB above baseline, capped at 0.5
    let excess = ambient_spl_db - baseline;
    (excess * 0.005).clamp(0.0, 0.5) as f32
}

/// Suggests a species from a fundamental frequency measurement.
///
/// Useful when a bioacoustic detector provides an f0 and the caller
/// needs the closest prani species match.
#[must_use]
pub fn species_from_f0(f0_hz: f64) -> Species {
    if f0_hz >= 3000.0 {
        Species::Cricket
    } else if f0_hz >= 1000.0 {
        Species::Songbird
    } else if f0_hz >= 500.0 {
        Species::Crow
    } else if f0_hz >= 200.0 {
        Species::Cat
    } else if f0_hz >= 100.0 {
        Species::Wolf
    } else if f0_hz >= 40.0 {
        Species::Lion
    } else if f0_hz > 0.0 {
        Species::Crocodilian
    } else {
        Species::Snake // f0 = 0 → noise-only
    }
}
