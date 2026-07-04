# Architecture Overview

> **v2.0.0 is the Cyrius port.** The pipeline below is unchanged from the Rust
> line; names are Cyrius (`crvoice_*`, `PRANI_SP_*`, structs via
> `#derive(accessors)`). The frozen Rust oracle lives at `rust-old/`.

## Module Map

15 `.cyr` library modules under `src/` (dependency-ordered; bundled into
`dist/prani.cyr`). `math.rs` folded into `f64_*` builtins; `lib.rs` had no logic.

```text
prani/ src/
  L0  error.cyr        — PRANI_ERR_* codes, prani_from_svara, tolerances, is_finite
      rng.cyr          — PrRng: PCG32 PRNG (stochastic synthesis)
      dsp.cyr          — DcBlocker (applied to every synthesis buffer)
      spatial.cyr      — Doppler shift, distance attenuation (+ HF absorption)
      vocalization.cyr — PRANI_VOC_* (14), PRANI_INTENT_* (7), IntentModifiers
      fatigue.cyr      — FatigueState / FatigueModifiers (vocal fatigue, habituation)
  L1  emotion.cyr      — PrEmotion valence/arousal → vocalization + intent + output
      sequence.cyr     — CallElement / CallBout / CallPhrase, synthesize_chorus
      species.cyr      — PRANI_SP_* (13), PRANI_APP_* (5), SpeciesParams, resonance_seed
      bridge.cyr       — pure science→param conversions (mass, temp, threat, SPL, wind)
  L2  tract.cyr        — CreatureTract (wraps svara VocalTract/Glottal/Formant + naad biquad)
  L3  voice.cyr        — CreatureVoice, pitch/formant contours, envelopes, post-processing
  L4  preset.cyr       — VoicePreset, 11 built-in presets
      stream.cyr       — SynthStream pull-based block synthesis (real-time callbacks)
  L5  ffi.cyr          — prani_ffi_* C buffer-callback API (handles=pointers, buffers=vecs)
```

## Data Flow

```text
species_params(species) -> SpeciesParams
                               |
crvoice_new(species)
    |-- stores SpeciesParams + individual offsets (f0, size, breathiness, effort)
    |
crvoice_vocalize(voc, sample_rate, duration)   # or crvoice_vocalize_with_intent
    |
    |-- validates species supports vocalization (else PRANI_ERR_INVALID_VOCALIZATION)
    |-- applies intent modifiers (pitch, amplitude, duration, urgency)
    |-- [Cat Purr special path: 25-30 Hz laryngeal cycling -> tract]
    |-- crtract_new(SpeciesParams, sr)
    |-- per-block (20ms) synthesis loop:
    |     |-- pitch contour (vocalization-specific f0 envelope)
    |     |-- formant transitions (cat meow mouth shape, wolf howl)
    |     |-- selects synthesis path by VocalApparatus:
    |     |     |-- Laryngeal:    svara glottal source -> vocal tract
    |     |     |-- Syringeal:    glottal (<=2kHz) or dual-sine (>2kHz) -> vocal tract
    |     |     |-- NoiseOnly:    PRNG noise -> naad bandpass (svara FormantFilter fallback dropped)
    |     |     |-- Stridulatory: pulse-train chirps (cricket) or continuous AM
    |     |     |-- Vibratile:    wing-beat AM modulation (bee)
    |-- post-tract processing:
    |     |-- time-varying subharmonics + deterministic chaos (Lion, Dragon, Croc)
    |     |-- fire-breath noise (Dragon); biphonation (Wolf/Dog howls/whines)
    |     |-- nasal anti-formant (Cat/Wolf onset); AM (bird trills)
    |     |-- spectral tilt (species + vocalization + effort); amplitude; envelope
    v
vec of f64 output samples   (or a negative PRANI_ERR_* on failure)
    |
    |-- optional: spatial_apply_distance_attenuation / spatial_apply_doppler_shift
    |-- optional: sequence_call_bout_synthesize / _call_phrase_synthesize / _synthesize_chorus
    |-- optional: SynthStream (stream_new / stream_fill_buffer) for pull-based blocks
```

## Consumers

- **kiran** (game engine) — real-time creature audio
- **joshua** (game manager) — triggered vocalizations
- Any AGNOS component needing non-human vocal synthesis

## Key Dependencies (Cyrius distlib bundles)

- **svara** 3.0.0 — glottal source, formant filter, vocal tract, vowel target
- **naad** 2.1.0 — biquad filters (NoiseOnly bandpass shaping)
- **hisab** 2.6.7 — `ease_in_out_smooth` (envelope curves)
- **goonj** 2.0.0, **sakshi** 2.4.3 — referenced transitively by svara/naad
