# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.0.2] - Toolchain + dependency catch-up

Maintenance release. Bumps the Cyrius pin **6.3.45 → 6.5.36** (124 toolchain
releases) and every pinned dependency to its current tag. **Two breaking upstream
renames** had to be absorbed; behaviour is otherwise unchanged. Suite **717
assertions / 16 suites**, all green — identical to 2.0.1, assertion for assertion.
`cyrius audit`: fmt gate clean, tests and bench green; it still exits 1 on the
pre-existing docs gate (10 undocumented public fns) and reports 7 pre-existing
lint warnings, neither of which this release touches — see Notes.

### Changed — toolchain

- **`[package].cyrius` `6.3.45` → `6.5.36`.**
- **`lib/` re-vendored from the 6.5.36 snapshot** (`cyrius lib sync` — the
  declared `[deps].stdlib` subset, 30 files rewritten). All 32 vendored stdlib
  files were then verified byte-identical to
  `~/.cyrius/versions/6.5.36/lib`, file by file, rather than assumed. This carries
  `lib/bayan.cyr` **1.0.4 → 1.5.2**, which is where the second rename below comes
  from.
- **112 lines reindented across 9 files** (7 in `src/`, 2 in `tests/`; 118
  changed lines total, of which 6 are the renames below). cyrius
  **6.5.28** fixed `cyrfmt`, which had never tracked parentheses: continuation
  lines inside an unclosed `(` were indented at `brace_depth * 4` regardless of
  nesting. Canonical is now 2 spaces per open-paren level. **Whitespace only** —
  `git diff -w` over `src/` and `tests/` is empty apart from the six rename lines
  below. (`cyrius fmt <file> --check` also accepts 4-space continuations; the
  `cyrius audit` fmt gate does not, which is what surfaced these.)

### Changed — dependencies

| Dep | Was | Now | |
|---|---|---|---|
| svara | 3.0.0 | **3.5.3** | glottal source · formant filter · vocal tract |
| naad | 2.1.0 | **2.2.2** | biquad (noise-only bandpass shaping) |
| hisab | 2.6.7 | **2.11.2** | `ease_in_out_smooth` (envelope curves) + transitive math |
| goonj | 2.0.0 | **2.0.4** | acoustics, referenced by the naad bundle |
| sakshi | 2.4.3 | **2.4.12** | structured logging (`prani_log_*`) |

svara 3.5.3 pins hisab 2.11.2 / naad 2.2.1 / goonj 2.0.4, and hisab 2.11.2 pins
sakshi 2.4.11 — so prani's set is coherent with what the bundles themselves pin.
Two deliberate steps past that: **naad 2.2.2** over svara's 2.2.1 (a pure bugfix
release — overlap-save streaming in `naad_convolution_process_block`; identical
dep pins, and prani touches no convolution path), and **sakshi 2.4.12** over
hisab's 2.4.11 (a lower-bound guard on `sakshi_span_enter`'s buffer). prani has
always pinned sakshi one ahead of the toolchain snapshot, so the
`./lib/ shadows version-pinned …` notice on every build is the normal state here,
not a new condition.

### Fixed — BREAKING upstream rename absorbed (`src/tract.cyr`)

naad **2.2.0** renamed all eight `FILTER_*` constants to `NAAD_FILTER_*` to clear
a flat-namespace collision with `nidhi`, which defines `FILTER_LOWPASS..NOTCH` at
identical values. prani calls exactly one of them:

- `FILTER_BANDPASS` → `NAAD_FILTER_BANDPASS` (the NoiseOnly pre-built bandpass,
  `crtract_new`)

The value is unchanged (2), so this is a compile-time break, not a behaviour
change — `tests/tract.tcyr`'s 52 assertions pass unchanged, the Snake hiss path
included.

**The other renames in that wave were checked and none reach prani.** Every
dependency symbol prani actually references was extracted from `src/` and
`tests/` and diffed against the new bundles: the naad/svara/hisab/goonj/sakshi
call set is `filter_biquad_new`, `filter_biquad_process_sample`,
`ease_in_out_smooth`, twelve `svara_*` (`svara_tract_new/_process_sample/_reset/
_set_formants_from_target/_synthesize`, `svara_glottal_new/_next_sample/
_set_breathiness/_set_jitter/_set_shimmer`, `svara_vowel_target_with_bandwidths`,
`svara_is_err`) and six `sakshi_*` — each of which resolves against the new
bundles with an unchanged signature. `ERR_* → NAAD_ERR_*` (naad 2.1.3) is inert
here because prani's codes are `PRANI_ERR_*`-prefixed and its one bare `ERR_NONE`
mention is a comment in `src/error.cyr`.

### Fixed — BREAKING stdlib rename absorbed (bayan 1.3.0, 4 call sites)

The re-vendored `lib/bayan.cyr` renames `bayan_json_v_parse_str` →
**`bayan_json_v_parse_buf`**, and the rename is load-bearing rather than
cosmetic: Cyrius routes a call `X(a, …)` to `X_str` whenever `a` is Str-typed at
the call site and `X_str` exists, so while the cstr+len form occupied that name,
every `bayan_json_v_parse(someStr)` in the ecosystem was silently rewritten into
a 1-arg call to a 2-arg function and returned 0 for valid JSON.

prani's four deserializers — `crtract_from_json_str`, `crvoice_from_json_str`,
`preset_from_json_str` and `sequence_call_phrase_from_json_str` — all called the
explicit 2-arg `(buf, len)` form, so **none of them was ever exposed to that
bug**; this is a pure rename. Signature and behaviour are identical, and the
serde roundtrip assertions in `tests/{tract,voice,preset,sequence}.tcyr` pass
unchanged.

### Fixed — two undefined functions the old bayan could not satisfy

Building at 2.0.1 emitted `undefined function 'bayan_f64_to_json'` and
`'bayan_f64_from_json'`. Neither name appears anywhere in prani's source or in
any dependency bundle, old or new — they are emitted by the compiler's
`#derive(Serialize)` expansion for f64 fields, and bayan **1.0.4 did not define
them**, so every build carried two dangling references. bayan 1.5.2 defines both.
The build is now warning-free apart from the standing `lib/` shadow notice.

This never reached prani's serde output: 2.0.1 types its f64 fields as `i64` bit
patterns precisely because the derive's float writer is lossy at 6 decimal
places, so the round-tripped values came from the integer path either way. The
dangling references were dead weight, not a wrong answer.

### Notes

- **Benchmarks re-run, no regression** (`tests/prani.bcyr`, x86_64 Linux, single
  core): `dcblocker_process` 20 ns/sample, `prani_rng_next_f32` 14 ns/sample,
  `emotion_evaluate` 70 ns/frame, `crvoice_vocalize` (Wolf howl, 0.05 s @ 8 kHz)
  214 µs/call. `emotion_evaluate` and the full synthesis path are ~16% and ~6%
  faster than the 2.0.0 figures; `docs/benchmarks.md` is updated. cyrius
  **6.5.19** taught `lib/bench.cyr` to measure the timer floor and subtract it
  from every sample — 1.31 µs on this host — so the harness now reports its own
  floor rather than the batch pattern merely amortizing it.
- **The svara behaviour changes between 3.0.0 and 3.5.3 do not move prani's
  output.** 3.5.0 collapsed the tract input delay line to two scalars, 3.3.0 gave
  the tract its own formant/nasal-place state, and 3.5.2 made
  `svara_formant_validate` test positively so NaN and ±inf are rejected — which
  closes the path through `svara_tract_set_formants_from_target`, one of prani's
  call sites. prani never passes NaN formants (species params are compile-time
  constants), and the svara-dependent suites assert structurally (non-error +
  exact length + all-finite + bit-identical determinism), by design for exactly
  this class of upstream change. All 717 assertions hold.
- **Two pre-existing conditions this release does not address**, both carried
  forward unchanged:
  - `cyrius lint` reports 7 line-length warnings in `src/vocalization.cyr`
    (lines 100-106, the aligned `prani_intent_modifiers` dispatch table). That
    file is untouched by this release. `cyrius lint` itself still exits 0.
  - `cyrius audit`'s docs gate reports 10 undocumented public fns — the four
    `*_from_json_str` deserializers, the three `sequence_*_new` constructors,
    the two `*_modifiers_make` builders and the `prani_log_*` wrappers. This
    release adds and removes no function and no doc comment (`git diff -w` over
    `src/` is the six rename lines, none of them a `fn` line), so the set is
    exactly the 2.0.1 set. It is what makes `cyrius audit` exit 1. Fixing it is
    a docs change, deliberately not bundled into a version bump.
- `dist/prani.cyr` regenerated at v2.0.2 (4,091 lines).

## [2.0.1] - Restore logging + serde

The 2.0.0 port **wrongly dropped logging and serde** — Cyrius has first-class
replacements for both, so this restores them to full parity.

### Added

- **Structured logging via sakshi** (`src/logging.cyr`, `prani_log_*`). All three
  Rust `tracing` sites are now wired for real (not dropped): `dsp` naad-error
  (`error!`), `tract` out-of-range-formant (`warn!`), and `voice` synthesis
  (`trace!` → `debug`). Always compiled in; runtime verbosity via `sakshi_set_level`.
- **serde restored on all 14 types** that had `#[derive(Serialize, Deserialize)]`
  — DcBlocker, Rng, IntentModifiers, EmotionState/Output, CallElement/Bout/Phrase,
  FatigueState/Modifiers, SpeciesParams, VoicePreset (incl. its display `name`),
  CreatureTract, CreatureVoice — via `#derive(Serialize)` + `bayan`. Each has a
  JSON `*_to_json` / `*_from_json_str` codec and a roundtrip test (**+90 assertions,
  717 total, all green**).
- **Lossless f64 serde**: f64 fields are typed `i64` so they serialize as their
  exact 64-bit bit patterns (bayan handles full 64-bit ints losslessly). The
  `#derive(Serialize)` float writer is only 6 decimal places (loses ~1 ULP on
  values like 0.7/0.15/0.05); the bit-pattern encoding roundtrips bit-exact.
  Restored `Rng` state roundtrips resume the identical stream.

### Notes

- `VoicePreset.name` (a `Cow<str>` display string) is serialized as a JSON string
  and restored — no longer stored as `0`.
- **Deviation (documented):** `CreatureTract` serde serializes the reconstructable
  state (params, rng, phase, dc_blocker, sample_rate) and rebuilds the opaque svara
  `VocalTract` + naad filter from params on deserialize — svara/naad have no Cyrius
  serialize surface. The species-determined response is bit-identical after rebuild;
  only a few samples of in-flight filter memory reset. `SynthStream` carries no
  serde (the Rust oracle never derived Serialize on it).

## [2.0.0] - Cyrius port

Complete rewrite from Rust to **Cyrius**. prani's Rust line shipped through 1.1.0;
the language port is a major break, so it lands as 2.0.0. The 3,527-line Rust
source is frozen at `rust-old/` as the parity oracle — every Cyrius module is
cross-checked against it function-for-function. Per-module ledger in
[`docs/development/port-audit.md`](docs/development/port-audit.md).

### Breaking

- **Language**: Rust crate → Cyrius library (`.cyr`). Consumers no longer add a
  `[dependencies] prani = …` Cargo entry; they include the `dist/prani.cyr`
  distlib bundle and resolve svara/naad/hisab/goonj/sakshi from their own
  `cyrius.cyml` (same pattern svara/naad use). No C ABI stability guarantee
  carried over — the `ffi` surface is re-expressed in Cyrius conventions.
- **Error handling**: the `PraniError` enum → integer codes (`PRANI_ERR_*`);
  fallible functions return `PRANI_ERR_NONE`/negative codes; `Result`/`Option`
  → code or sentinel returns (vec pointer on success, negative on error; `-1`
  for absent). `svara::error::SvaraError` mapping → `prani_from_svara`.
- **API shape**: methods → free functions (`CreatureVoice::vocalize` →
  `crvoice_vocalize`, etc.); enums → integer constants (`Species::Wolf` →
  `PRANI_SP_WOLF`); structs via `#derive(accessors)`. See the naming contract in
  the port audit.

### Changed

- Toolchain pinned via `cyrius.cyml [package].cyrius` (6.3.45). Build with
  `cyrius build src/main.cyr build/prani`; test a suite with
  `cyrius test tests/<mod>.tcyr`.
- **All 15 modules ported** (L0 → FFI): error, rng, dsp, spatial, vocalization,
  fatigue, emotion, sequence, species, bridge, tract, voice, preset, stream, ffi.
  (`math.rs`'s thin f32 transcendental wrappers fold into direct `f64_*` builtin
  + ganita calls; `lib.rs` carried no independent logic.)
- **f32 → f64** throughout (svara/naad/hisab are f64-only; widening is forced and
  improves precision). Test tolerances loosened where bit-exactness through
  svara's DSP isn't meaningful.
- **Dependencies**: svara (glottal/formant/vocal-tract), naad (biquad filters),
  hisab (`ease_in_out_smooth`), goonj, sakshi consumed as Cyrius distlib bundles.
  `thiserror`/`libm`/`criterion` dropped (`Vec` → stdlib `vec`; transcendentals
  via ganita builtins). serde and `tracing` were also dropped here — **incorrectly;
  restored in 2.0.1** via `#derive(Serialize)`+bayan and sakshi.

### Added

- **Parity test suites**: one `tests/<mod>.tcyr` per module — **627 assertions
  across 15 suites, all green** — cross-checked against the frozen Rust oracle
  (serde round-trip + Display-string tests dropped).
- **`dist/prani.cyr`** distlib bundle (15 modules, 3,760 lines, dependency-ordered,
  one flat namespace; cross-module symbol collisions audited to zero across every
  fn/struct/const). Consumers supply stdlib + svara + naad + hisab + goonj +
  sakshi. `src/main.cyr` smoke-builds and links the bundle (a Wolf howl end-to-end).
- **`resonance_seed` parity**: the 13 species seeds (an f32-bit-pattern hash in
  the oracle) are precomputed and stored, exactly matching Rust's
  `f32::to_bits`-based values (independently re-derived).
- **Hot-path benchmarks** (`cyrius bench tests/prani.bcyr`): DC blocker
  19 ns/sample, PCG32 15 ns/sample, `emotion_evaluate` 83 ns/frame, full Wolf-howl
  synthesis 227 µs (≈ 220× realtime, whole svara/naad stack). See
  [`docs/benchmarks.md`](docs/benchmarks.md).

### Removed

- `thiserror`, `libm`, `criterion` dependencies. (serde and `tracing` were also
  dropped here but that was a mistake — **restored in 2.0.1**; Cyrius provides
  both via `#derive(Serialize)`+bayan and sakshi.)
- `tracing-subscriber` dependency;
  the Cargo `std`/`naad-backend`/`logging`/`ffi` feature flags (Cyrius uses
  `cyrius.cyml` dep resolution instead).

## [1.1.0] - 2026-03-28

### Added

- `bridge` module: pure science-crate value conversions (body mass → size scale, temperature → f0 offset, threat level → intent, SPL → amplitude, wind → Doppler, f0 → species). No dependency on external science crates — consumers call bridge functions with primitive values
- `dsp` module: `DcBlocker` applied to all synthesis output paths (removes DC offset from asymmetric excitation sources); `map_naad_error` helper behind `naad-backend` feature gate
- Expanded `math.rs`: added `cos`, `exp`, `sqrt`, `powf` with std/libm dual paths (matching garjan pattern)
- naad dual code paths in `CreatureTract`: noise-only synthesis (snake) uses `naad::filter::BiquadFilter` when `naad-backend` is active, falling back to svara `FormantFilter` otherwise
- `#[must_use]` on `Species::params()`, `CallIntent::modifiers()`, `presets::all()` with descriptive messages
- `emotion` module: `EmotionState` valence/arousal model with smooth transitions. `evaluate()` maps 2D emotion space to vocalization selection, call intent, vocal effort, pitch scale, and breathiness. 9-region mapping (3×3 valence×arousal grid)
- `fatigue` module: `FatigueState` tracks vocal fatigue (pitch drift, breathiness increase, amplitude loss) and alarm habituation (unreinforced alarms lose intensity). Recovery during rest, reinforcement resets habituation
- `stream` module: `SynthStream` pull-based streaming synthesizer — yields audio blocks on demand via `fill_buffer()` or `next_block()` without full-buffer allocation. Suitable for real-time audio callbacks (Wwise, FMOD, Godot, JACK)
- `ffi` module (behind `ffi` feature gate): C FFI buffer-callback API with `extern "C"` functions — `prani_voice_create/destroy`, `prani_voice_set_effort/set_size/apply_lombard`, `prani_stream_start/fill/is_finished/destroy`. Species/vocalization/intent via integer indices
- Vocal effort parameter on `CreatureVoice` (0.0=whisper, 0.5=normal, 1.0=shout). Modulates amplitude (0.3–1.5×), spectral tilt (±3 dB/oct), and breathiness (U-shaped: breathy at extremes). Builder (`with_vocal_effort`) and real-time setter (`set_vocal_effort`)
- Lombard effect: `CreatureVoice::apply_lombard_effect(ambient_spl_db)` — involuntary vocal effort boost ~0.05 per 10 dB above 40 dB SPL baseline
- RTPC bridge functions: `pitch_scale_from_valence`, `vocal_effort_from_arousal`, `perturbation_from_urgency`, `lombard_effort_boost` — continuous parameter converters for game AI integration
- `ffi` feature flag (implies `std`)
- 22 new integration tests (72 total): vocal effort, emotion state, Lombard effect, fatigue/habituation, streaming, bridge functions, serde roundtrips for new types
- 3 new benchmarks (14 total): `wolf_howl_shout_1s`, `stream_wolf_howl_1s`, `emotion_evaluate`
- Send+Sync compile-time assertions for `EmotionState`, `EmotionOutput`, `FatigueState`, `FatigueModifiers`

### Fixed

- Removed `.unwrap()` in `FormantTransitionContour::at()` — replaced with safe match (zero-panic compliance)
- Fixed orphaned `#[inline]` attribute between `apply_am_pattern` and `vocalization_spectral_offset` doc comments
- `naad-backend` feature now implies `std` (matching garjan pattern — high-quality DSP requires stdlib)

### Changed

- DC blocker now applied to all 5 synthesis paths (laryngeal, syringeal, noise, stridulatory, vibratile, purr) — prevents DC offset accumulation from asymmetric excitation

---

## [Unreleased - pre-1.1.0]

### Added

- `naad` as optional dependency with `naad-backend` feature flag (default on), matching svara
- Non-stationary jitter/shimmer: perturbation scales with call urgency and position (stronger at boundaries and during alarm/distress)
- `Species::bout_template()`: species-specific default `CallBout` for all 13 species (e.g., dogs bark 5x at 0.25s intervals, wolves howl 3x with 2s gaps)
- 4 new voice presets: Bald Eagle, Raven, Field Cricket, American Alligator (11 total)
- Spectral envelope per vocalization: growls/rumbles darker (-2 dB/oct offset), screeches/hisses brighter (+1.5 to +2 dB/oct)
- Source-filter coupling for birds: F1 tracks toward f0 at 40% coupling strength, simulating syrinx-tract interaction
- 4 new tests: bout templates, spectral envelope, source-filter coupling, non-stationary perturbation (50 total)
- CI/CD pipeline: GitHub Actions workflows (ci.yml, release.yml) matching svara
- Makefile, rust-toolchain.toml, codecov.yml, scripts/bench-history.sh
- `spatial` module: `apply_distance_attenuation` (inverse-distance + atmospheric HF absorption), `apply_doppler_shift` (linear interpolation resampling)
- `sequence` module: `CallBout` (repeated calls with intervals), `CallPhrase` (ordered vocalization sequences), `synthesize_chorus` (multiple voices with timing spread)
- `preset` module: `VoicePreset` with 7 built-in presets (Alpha Wolf, Wolf Pup, House Cat, Kitten, Male Lion, Ancient Dragon, Young Dragon)
- `VocalApparatus::Vibratile`: new variant for bees (thoracic flight muscle vibration)
- `spectral_tilt` field on `SpeciesParams`: per-species dB/octave roll-off (lion: -6, bird: -1)
- Cat purr special-case synthesis: 25-30 Hz laryngeal muscle cycling with asymmetric waveform through vocal tract
- Formant transitions: dynamic formant changes during cat meow (nasal -> open -> closing) and wolf howl
- Cricket discrete pulse-train chirps: 3-5 pulse groups at ~30 Hz with inter-chirp silence
- Time-varying subharmonic amplitude for lion/dragon/crocodilian (peaks during middle of call)
- Deterministic chaos injection during peak intensity of roars (period-doubling roughness)
- Biphonation for canids: second independent pitch (~minor seventh) during wolf/dog howls
- Nasal resonance: anti-formant notch at ~250 Hz during nasal phases of cat meow and wolf howl
- AM patterns: bird trill rapid amplitude modulation at 20 Hz
- `#[must_use]` on `SpeciesParams`, `IntentModifiers`, `VoicePreset`
- Tracing warning when species formants fall out of svara's valid range
- 26 new integration tests (46 total), covering all new modules and features
- docs/architecture/overview.md with full data flow diagram
- docs/development/roadmap.md

### Changed

- Bee species now uses `VocalApparatus::Vibratile` (was `Stridulatory`)
- Bird species (Songbird, Crow, Raptor) have wider formant bandwidths for less defined resonances
- Crow breathiness increased (0.15 -> 0.18) for more realistic harsh/noisy calls
- Dragon fire-breath RNG seed derived from species params (was hardcoded 8888)
- Subharmonics now have time-varying envelope with chaos (was constant 0.3 amplitude sine)
- Removed unused f64 math module and unused RNG methods (poisson, next_f32_range, next_f32_unsigned)
- Removed `#[allow(dead_code)]` suppressions

### Performance

- New features add processing to the synthesis pipeline. Regressions are proportional to added complexity:
  - wolf_howl_1s: 1.29 -> 1.49 ms (+15%) — biphonation, nasal resonance, formant transitions, spectral tilt
  - wolf_alarm_howl_1s: 773 -> 1040 us (+35%) — same pipeline additions
  - lion_roar_1s: 1.64 -> 1.47 ms (-10%) — net improvement despite new subharmonic envelope + chaos
  - dragon_roar_1s: 1.52 -> 1.55 ms (+2%) — near-neutral
  - songbird_trill_500ms: 872 -> 802 us (-8%) — improved despite new AM pattern
  - snake_hiss_500ms: 519 -> 252 us (-51%) — improved (dead code removal, no new processing)
  - cricket_stridulate_300ms: 221 -> 235 us (+6%) — pulse-train replaces continuous AM

## [1.0.0] - 2026-03-27

### Added

- Initial scaffold of the prani crate
- `Species` enum: 13 species (Wolf, Dog, Cat, Lion, Songbird, Crow, Raptor, Snake, Crocodilian, Cricket, Bee, Dragon, Fantasy)
- `VocalApparatus` enum: Laryngeal, Syringeal, Stridulatory, NoiseOnly
- `SpeciesParams`: Per-species vocal parameters (f0 range, tract scale, breathiness, jitter, shimmer)
- `CreatureTract`: Species-specific vocal tract wrapping svara's VocalTract with apparatus-dependent synthesis
- `Vocalization` enum: 14 call types (Howl, Bark, Growl, Roar, Hiss, Chirp, Trill, Whine, Rumble, Purr, Yelp, Screech, Stridulate, Buzz)
- `CallIntent` enum: 7 behavioral intents (Alarm, Territorial, Mating, Distress, Idle, Threat, Social) with prosodic modifiers
- `CreatureVoice`: Species instance with individual variation (size, f0 offset, breathiness) and builder pattern
- `PraniError`: Error type with svara error conversion
- Integration tests: all species synthesize, intent modifies output, individual variation, serde roundtrips
- Criterion benchmarks: wolf howl, cat purr, cricket stridulate, dragon roar, snake hiss
- `no_std` support via `libm` + `alloc`
- Feature flags: `std` (default), `logging`, `full`
- Strict `deny.toml` matching hisab production patterns
- Send/Sync compile-time assertions on all public types
