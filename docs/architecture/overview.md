# Architecture Overview

## Module Map

```text
prani/
  src/
    lib.rs          — Crate root, prelude, trait assertions
    species.rs      — Species enum, VocalApparatus (5 types), SpeciesParams
    tract.rs        — CreatureTract (wraps svara::VocalTract), SynthesisOptions
    vocalization.rs — Vocalization enum (14 types), CallIntent (7), IntentModifiers
    voice.rs        — CreatureVoice, pitch/formant contours, envelopes, post-processing
    error.rs        — PraniError, Result alias
    math.rs         — no_std math shim (sin via libm)
    rng.rs          — PCG32 PRNG for stochastic synthesis
    preset.rs       — VoicePreset, 7 built-in presets
    sequence.rs     — CallBout, CallPhrase, synthesize_chorus
    spatial.rs      — Doppler shift, distance attenuation
```

## Data Flow

```text
Species::params() -> SpeciesParams
                         |
CreatureVoice::new(species)
    |-- stores SpeciesParams + individual offsets (f0, size, breathiness)
    |
vocalize(Vocalization, sample_rate, duration)
    |
    |-- validates species supports vocalization
    |-- applies CallIntent modifiers (pitch, amplitude, duration, urgency)
    |-- [Cat Purr special path: 25-30 Hz laryngeal cycling -> tract]
    |-- creates CreatureTract from SpeciesParams
    |-- per-block synthesis loop:
    |     |-- pitch contour (vocalization-specific f0 envelope)
    |     |-- formant transitions (cat meow mouth shape, wolf howl)
    |     |-- selects synthesis path by VocalApparatus:
    |     |     |-- Laryngeal:    svara::GlottalSource -> VocalTract
    |     |     |-- Syringeal:    GlottalSource (<=2kHz) or dual-sine (>2kHz) -> VocalTract
    |     |     |-- NoiseOnly:    PRNG noise -> FormantFilter
    |     |     |-- Stridulatory: Pulse-train chirps (cricket) or continuous AM
    |     |     |-- Vibratile:    Wing-beat AM modulation (bee)
    |-- post-tract processing:
    |     |-- time-varying subharmonics + deterministic chaos (Lion, Dragon, Croc)
    |     |-- fire-breath noise (Dragon)
    |     |-- biphonation (Wolf, Dog howls/whines)
    |     |-- nasal anti-formant (Cat meow onset, Wolf howl onset)
    |     |-- AM patterns (bird trills)
    |     |-- spectral tilt (per-species dB/octave roll-off)
    |     |-- amplitude scaling from intent
    |     |-- attack/release envelope
    |
    v
Vec<f32> output samples
    |
    |-- optional: spatial::apply_distance_attenuation
    |-- optional: spatial::apply_doppler_shift
    |-- optional: sequence::CallBout / CallPhrase / synthesize_chorus
```

## Consumers

- **kiran** (game engine) — real-time creature audio
- **joshua** (game manager) — triggered vocalizations
- Any AGNOS component needing non-human vocal synthesis

## Key Dependencies

- **svara** — GlottalSource, FormantFilter, VocalTract, VowelTarget
- **hisab** — ease_in_out_smooth for envelopes
- **libm** — no_std transcendental math fallback
