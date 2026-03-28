# Roadmap

## Completed

### v1.0.0 (2026-03-27)

- 13 species with bioacoustically-informed vocal parameters
- 4 vocal apparatus types (Laryngeal, Syringeal, Stridulatory, NoiseOnly)
- 14 vocalization patterns with species-specific pitch contours
- 7 behavioral intents with prosodic modifiers
- Individual variation (size, f0 offset, breathiness, jitter, shimmer)
- Subharmonic generation for large species (Lion, Dragon, Crocodilian)
- Dragon fire-breath noise synthesis
- High-frequency dual-source syringeal path for birds
- Full serde support on all public types
- no_std + alloc support

### P(-1) Scaffold Hardening (2026-03-27)

- Removed dead code (unused f64 math, unused RNG methods)
- Added `#[must_use]` on SpeciesParams and IntentModifiers
- Fixed dragon fire-breath seed derivation (was hardcoded)
- Added tracing warning for out-of-range formant fallback
- CI/CD pipeline matching svara (GitHub Actions, cargo-deny, codecov)
- 31 tests, 11 benchmarks

### Backlog (2026-03-27)

#### High Priority (all completed)
- Cat purr special-case synthesis (25-30 Hz laryngeal muscle cycling)
- Formant transition support for cat meow and wolf howl
- Discrete pulse-train structure for cricket chirps
- Spectral tilt control per species (dB/octave)

#### Medium Priority (all completed)
- Time-varying subharmonic amplitude (peaks during roar/bellow)
- Deterministic chaos regime for lion/dragon/crocodilian roars
- Wider bandwidths and increased breathiness for bird vocal tract
- AM patterns for bird trills (20 Hz)
- VocalApparatus::Vibratile for bees (renamed from Stridulatory)

#### Lower Priority (all completed)
- Nasal resonance (anti-formants) for cat meow and wolf howl
- Biphonation for canids (two independent pitches from one larynx)
- `spatial` module: Doppler shift, distance attenuation + atmospheric absorption
- `sequence` module: CallBout, CallPhrase, synthesize_chorus
- `preset` module: VoicePreset with 7 built-in presets
- 46 tests, 11 benchmarks

## Backlog

- naad-backend integration (feature-gated, like svara — needs naad dependency)
- Non-stationary jitter/shimmer (increases during high-effort calls and at boundaries)
- Call bout/repetition patterning with species-specific timing templates
- More voice presets (raptor, crow, cricket, crocodilian)

## Future

- Alien species procedural generation from physical parameters
- Underwater vocalization (whale, dolphin — different propagation model)
- Echolocation pulse synthesis (bats, dolphins)
- Environmental acoustic interaction (reverb, occlusion)
- Real-time parameter modulation API for game integration

## v1.0 Criteria (met)

- [x] All 13 species synthesize correctly
- [x] All 5 apparatus types produce distinct output
- [x] All public types: Send + Sync + Serialize + Deserialize
- [x] no_std support
- [x] Zero panics in library code
- [x] 80%+ code coverage target
- [x] Benchmark suite covering all synthesis paths
- [x] CI pipeline with fmt, clippy, audit, deny, MSRV
