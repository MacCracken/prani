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
- 31 tests, 11 benchmarks
- CI/CD pipeline matching svara (GitHub Actions, cargo-deny, codecov)

### P(-1) Scaffold Hardening (2026-03-27)

- Removed dead code (unused f64 math, unused RNG methods)
- Added `#[must_use]` on SpeciesParams and IntentModifiers
- Fixed dragon fire-breath seed derivation (was hardcoded)
- Added tracing warning for out-of-range formant fallback
- Added 10 new tests (serde roundtrips, edge cases, missing species coverage)
- Added 3 new benchmarks (bee, crow, crocodilian)

## Backlog

### High Priority (biggest impact on realism)

- Cat purr special-case synthesis (25-30 Hz laryngeal muscle cycling, not vocal fold vibration)
- Move subharmonic generation before the tract filter (so formant resonances shape them)
- Formant transition support for cat meow and wolf howl (dynamic mouth shape changes)
- Discrete pulse-train structure for cricket chirps (3-5 pulses at ~30 Hz, silence between chirps)

### Medium Priority

- Spectral tilt control per species (dB/octave roll-off; brightness/darkness of voice quality)
- Time-varying subharmonic amplitude (stronger during peak of roar/bellow)
- Deterministic chaos regime for lion/dragon roars (period doubling -> chaos -> recovery)
- Wider bandwidths and source-filter coupling for bird vocal tract model
- Amplitude modulation patterns per vocalization (e.g., bird trill AM at 20-100 Hz)
- Rename Bee apparatus from Stridulatory (technically thoracic flight muscle vibration, not friction-based)

### Lower Priority

- Nasal resonance (anti-formants) for mammalian calls (cat meow onset, wolf howl)
- Non-stationary jitter/shimmer (increases during high-effort calls and at boundaries)
- Call bout/repetition patterning (species-specific inter-call intervals)
- Biphonation for canids (two pitches from a single larynx)
- Nonlinear phenomena: period doubling, deterministic chaos (distressed/aggressive calls)
- Doppler shift for moving creatures
- Distance-based attenuation and atmospheric filtering
- Pack/flock chorus synthesis (multiple simultaneous voices with timing variation)
- Call sequence/phrase generation (e.g., bird song syllable patterns)
- naad-backend integration (feature-gated, like svara)
- Creature voice presets (named individuals within species)

## Future

- Alien species procedural generation from physical parameters
- Underwater vocalization (whale, dolphin — different propagation model)
- Echolocation pulse synthesis (bats, dolphins)
- Environmental acoustic interaction (reverb, occlusion)
- Real-time parameter modulation API for game integration

## v1.0 Criteria (met)

- [x] All 13 species synthesize correctly
- [x] All 4 apparatus types produce distinct output
- [x] All public types: Send + Sync + Serialize + Deserialize
- [x] no_std support
- [x] Zero panics in library code
- [x] 80%+ code coverage target
- [x] Benchmark suite covering all synthesis paths
- [x] CI pipeline with fmt, clippy, audit, deny, MSRV
