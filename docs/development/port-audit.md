# prani — Rust → Cyrius Port Audit

Per-module parity ledger for the 2.0.0 port. The Rust oracle is frozen at
`rust-old/`; every Cyrius module must match it function-for-function. Update
the relevant row whenever a module's status changes.

**Status:** ✅ ported & tested · 🟡 partial · ⬜ pending
**LOC** = Rust lines (incl. tests) at `rust-old/src/`.

## Conventions established (apply to every module)

Proven reference template: **`src/error.cyr`, `src/rng.cyr`, `src/dsp.cyr`**
(+ their `tests/*.tcyr`). Read those first. Cross-repo analogs: `../svara/src/*.cyr`
(prani's dep, same idioms), `../naad/src/*.cyr`, `../naad/docs/development/port-audit.md`.

- **f32 → f64 everywhere** (svara/naad/hisab are f64-only; widening is forced and
  improves precision). Test tolerances loosened vs the f32 oracle where
  bit-exactness isn't meaningful (`PRANI_EPSILON` = f32::EPSILON promoted to f64).
- **Float literals**: integers via `f64_from(n)`; non-integers as module-top
  `var NAME = 0x...;` holding the IEEE-754 hex bit pattern, decimal in a comment.
  Generate: `python3 -c "import struct;print(hex(struct.unpack('<Q',struct.pack('<d',V))[0]))"`.
- **Math API**: `f64_sin/cos/exp/sqrt/ln` are builtins; `f64_pow/tanh/sinh/cosh/
  asin/acos/atan2/hypot/...` from ganita; arithmetic `f64_add/sub/mul/div/neg/abs/
  lt/le/gt/ge`; helpers `f64_clamp/min/max/lerp/sign/fract/trunc`; constants
  `F64_PI/F64_TAU/F64_HALF/F64_ONE/F64_E`. Comparisons return `1`/`0`.
- **`enum` → integer `var` constants** (see error.cyr / any svara enum port).
- **`enum` errors → integer codes**: fallible fns return `PRANI_ERR_NONE` (0) or a
  negative `PRANI_ERR_*` from `src/error.cyr`. `Option<Error>` → the code directly.
  svara errors map via `prani_from_svara(svara_code)`.
- **`Result<T>` / `Option<T>`** → sentinel returns (error code, or a NaN/-1 sentinel)
  unless a real payload is needed (then `lib/tagged.cyr`).
- **`Vec<T>` → stdlib `vec`** (`vec_new`/`vec_push`/`vec_len`/`vec_get`/`vec_set`);
  f64 elements store directly in the 8-byte slots. `&mut [f32]` / `&[f32]` buffers →
  a `vec` handle (mutated in place / read).
- **structs** via `#derive(accessors)` + `alloc(sizeof(T))`; methods become free
  functions `TypeName_verb(self, …)`; accessors are `TypeName_field(self)` /
  `TypeName_set_field(self, v)`. Fixed `[f32; N]` inline arrays → a `vec`.
- **Every top-level symbol is `prani_`/`PRANI_`/`Pr`-prefixed** — the bundle is one
  flat namespace shared with svara (`svara_*`) and naad (`validate_*`, `ERR_NONE`,
  `flush_denormal`). Front-load collision avoidance. `#derive(accessors)`
  auto-prefixes field accessors by struct name (pick distinct struct names).
- **`&mut u32/u64` PRNG state** → the `PrRng` struct (`src/rng.cyr`); u64 wrap is
  plain `*`/`+` on i64; `>>` is logical; u32 truncate with `& 0xFFFFFFFF`.
- **Module files do NOT `include` each other** — the build/test entry includes them
  in dependency order (stdlib auto-prepends from `cyrius.cyml`; svara/naad/hisab via
  `include "lib/<dep>.cyr"` only in `src/main.cyr`, not in test units).
- **serde RESTORED** (2.0.1) via `#derive(Serialize)` + `bayan`: every type that
  had `#[derive(Serialize, Deserialize)]` gets a `*_to_json` / `*_from_json_str`
  codec + a roundtrip test. f64 fields are typed `i64` so they serialize as exact
  bit patterns (lossless; the derive's 6-digit float writer is not). Display-string
  tests dropped (no Display in Cyrius). Logging RESTORED via sakshi (`src/logging.cyr`).
  All other `#[test]` blocks ported one-for-one. Where a Rust
  module had NO `#[test]` block (its behaviour lives in `tests/integration.rs`),
  write focused behavioural tests that lock parity (golden values from the oracle).
- **`math.rs`** (thin f32 sin/cos/exp/sqrt/powf wrappers) is folded into direct
  `f64_*` builtin + ganita calls — **no `math.cyr` module** (same as svara/naad).
- **Cross-check every module against `rust-old/`** — correctness bar is "matches
  what Rust did". Diverge only with a documented parity note.

## Test harness pattern

Each module gets `tests/<mod>.tcyr`:
```
include "src/error.cyr"        # deps first, in dependency order
include "src/<mod>.cyr"
alloc_init();
test_group("group name");
assert_eq(actual, expected, "message");   # also assert_neq/streq/lt/gt/gte/lte
var rc = assert_summary();
syscall(60, rc);
```
Run: `flock <scratch>/prani-build.lock cyrius test tests/<mod>.tcyr`. Assert on f64
**bit patterns** for exact values, or use `f64_lt(f64_abs(f64_sub(a,b)), tol)` for
tolerance. stdlib (assert/vec/math/alloc) auto-resolves — do not include it.

## Naming contract (flat namespace shared with svara_*/naad — avoid collisions)

| Module | Struct(s) | Fn prefix | Key symbols |
|---|---|---|---|
| error | — | `prani_` | `PRANI_ERR_*`, `prani_is_err`, `prani_err_name`, `prani_from_svara`, `prani_is_finite`, `PRANI_EPSILON` |
| rng | `PrRng` | `prani_rng_` | `prani_rng_new`, `prani_rng_next_u32`, `prani_rng_next_f32` |
| dsp | `DcBlocker` | `dcblocker_` | `dcblocker_new/process/process_buffer/reset`, `prani_map_naad_error` |
| vocalization | `IntentModifiers` | `prani_intent_` | `PRANI_VOC_*` (14), `PRANI_INTENT_*` (7), `prani_intent_modifiers` |
| spatial | — | `spatial_` | `spatial_apply_distance_attenuation`, `spatial_apply_doppler_shift` |
| fatigue | `FatigueState`, `FatigueModifiers` | `fatigue_` | `fatigue_new/record_call/rest/modifiers/reset` |
| emotion | `PrEmotion`, `PrEmotionOut` | `emotion_` | `emotion_new/evaluate/select_intent` |
| species | `SpeciesParams` | `species_` | `PRANI_SP_*` (13), `PRANI_APP_*` (5), `species_params`, `species_supports_vocalization`, `species_bout_template`, `species_params_resonance_seed` |
| sequence | `CallElement`, `CallBout`, `CallPhrase` | `sequence_` | `sequence_call_bout_new`, `sequence_call_bout_synthesize`, `sequence_call_phrase_synthesize`, `sequence_synthesize_chorus` |
| bridge | — | `bridge_` | `bridge_size_from_body_mass`, `bridge_lombard_effort_boost`, `bridge_species_from_f0`, … |
| tract | `CreatureTract`, `SynthesisOptions` | `crtract_` | `crtract_new`, `crtract_synthesize`, `crtract_synthesize_purr`, `crtract_set_formant_blend`, `crtract_apply_spectral_tilt`, `crtract_reset` |
| voice | `CreatureVoice` | `crvoice_` | `crvoice_new`, `crvoice_vocalize`, `crvoice_vocalize_with_intent`, `crvoice_with_f0_offset`, `crvoice_effective_f0`, … (NOT `voice_*` — naad exports `voice_*`) |
| preset | `VoicePreset` | `preset_` | `preset_*` |
| stream | (as needed) | `stream_` | `stream_*` |
| ffi | (as needed) | `prani_ffi_` | `prani_ffi_*` |

## Toolchain & commands

- cycc pin: **6.3.45** (`cyrius.cyml [package].cyrius`).
- Deps: svara/naad/hisab/goonj/sakshi as `dist/*.cyr` git+tag bundles, vendored in
  `lib/`. Do not modify `lib/`.
- **Concurrency**: `cyrius test`/`build`/`deps` re-resolve deps and race on
  `cyrius.lock`. Parallel porting agents MUST serialize every `cyrius …` call
  behind a shared file lock: `flock <scratch>/prani-build.lock cyrius …`.

## Ledger

### L0 — base (no non-error internal deps)

| Module       | LOC | Status | Tests | Notes |
|--------------|----:|--------|------:|-------|
| error        |  38 | ✅ | 21 | Integer codes (5 variants) + `prani_from_svara` (was `impl From<SvaraError>`) + `prani_is_finite` + `PRANI_EPSILON/POS_INF/NEG_INF`. Universal base — every entry includes it first. |
| rng          |  39 | ✅ | 15 | PCG32 (`PrRng`), bit-identical to svara. Golden next_u32 parity (seed 42/7). next_f32 ∈ [0,1) per the Rust *code* (doc claims [-1,1]). |
| dsp          |  65 | ✅ |  9 | `DcBlocker` single-pole (R=0.995) + `process`/`process_buffer`/`reset` + `prani_map_naad_error`→`PRANI_ERR_SYNTHESIS_FAILED`. |
| spatial      |  71 | ✅ | 24 | `spatial_apply_distance_attenuation` (1-pole LPF) + `spatial_apply_doppler_shift` (linear-interp resample). Buffers → vec. |
| vocalization | 129 | ✅ | 16 | `PRANI_VOC_*` (14) + `PRANI_INTENT_*` (7) consts; `IntentModifiers` + `prani_intent_modifiers` lookup. |
| fatigue      | 159 | ✅ | 50 | `FatigueState`/`FatigueModifiers`; accumulation/recovery/habituation curves. |

### L1 — depend on L0

| Module  | LOC | Status | Deps | Notes |
|---------|----:|--------|------|-------|
| emotion | 245 | ✅ 62 | vocalization | `PrEmotion`/`PrEmotionOut`; valence/arousal zones → vocalization+intent+output. |
| species | 426 | ✅ 60 | vocalization, sequence(CallBout struct) | `PRANI_SP_*` (13) + `PRANI_APP_*` (5); 13-species param table; `resonance_seed` stored as precomputed f32-bit-hash consts (all 13 independently verified vs oracle). `bout_template` constructs `CallBout`. |
| sequence| 138 | 🟡 13 | error, rng, vocalization; voice (synth methods, deferred) | `CallElement`/`CallBout`/`CallPhrase` structs + constructors + chorus tested now; `synthesize*` methods reference `crvoice_*` (unreachable until voice) — full parity test lands with voice/stream. |

### L2 — svara vocal-tract bridge

| Module | LOC | Status | Deps | Notes |
|--------|----:|--------|------|-------|
| tract  | 436 | ✅ 43 | dsp, error, rng, species, **svara**, **naad** | `CreatureTract`/`SynthesisOptions`, `crtract_*`. All 5 apparatus paths + purr + formant-blend + spectral-tilt. svara glottal/tract/formant + naad `filter_biquad_new(FILTER_BANDPASS,…)`. Structural+determinism parity (f32→f64 diverges through svara internals). Dropped the svara FormantFilter fallback (naad bandpass always present for NoiseOnly under default features). |
| bridge | 213 | ✅ 75 | species, vocalization | 13 pure value-conversion fns, `bridge_*`. `PrPerturbation` struct for the `(f32,f32)` tuple return; cbrt via `f64_pow(x,1/3)`. NOT a voice dep (its `crate::voice::` refs are doc-links). |

### L3 — voice/bridge (mutually referential, one flat-namespace unit)

| Module | LOC | Status | Deps | Notes |
|--------|----:|--------|------|-------|
| voice  | 705 | ✅ 66 | error, rng, dsp, species, tract, vocalization, bridge, hisab | `CreatureVoice`, `crvoice_*` (helpers prefixed too — stream reuses `crvoice_vocalization_spectral_offset`). Full orchestration: pitch/formant contour tables (flat vecs), 20ms block loop, species effects (chaos `+0xCA05`, fire `+0xF12E`, biphonation, nasal notch, AM, tilt, envelope via hisab `ease_in_out_smooth`). Structural+determinism parity through svara; exact where svara-independent. |

### L4 — composites

| Module   | LOC | Status | Deps | Notes |
|----------|----:|--------|------|-------|
| preset   | 160 | ✅ 93 | species, voice | `VoicePreset` + 11 named preset constructors + `preset_all`. `preset_build` chains crvoice builders. |
| stream   | 271 | ✅ 35 | error, species, tract, vocalization, voice | `SynthStream` pull-based synth; lazy tract init; reuses `crvoice_vocalization_spectral_offset`; `stream_pitch_contour_at` (f0-returning). fill_buffer applies tilt+amp to block then copies (equivalent to oracle). |
| (sequence full synth methods now covered via voice/stream) | | | | |

### L5 — FFI surface

| Module | LOC | Status | Deps | Notes |
|--------|----:|--------|------|-------|
| ffi    | 265 | ✅ 37 | species, stream, vocalization, voice | `prani_ffi_*` C buffer-callback API. Handles=pointers, buffers=vecs, destroy=no-op (arena), index mappers=range checks. `voice_set_size` rebuilds+copies fields; `stream_start` shares the voice handle (read-only). ABI machinery (`extern "C"`/Box) doesn't translate — logic ported. |

**Totals:** 17 Rust files → 16 `.cyr` modules (15 ported + `logging.cyr`;
`math.rs` folded into `f64_*` builtins, `lib.rs` carries only organization/
prelude) · 3,527 Rust lines → ~4,090-line `dist/prani.cyr` bundle. **✅ PORT
COMPLETE — 16/16 modules, zero deferrals. 717 parity assertions green across 16
suites** (all oracle behaviour ported — including serde roundtrips + logging;
only Display-string tests dropped). `dist/prani.cyr` assembled (collision-audited
to zero across all fns/structs/consts), `src/main.cyr` smoke links + runs the
bundle, hot-path benchmarks captured. VERSION 1.1.0 → 2.0.0 (port) → **2.0.1**
(logging via sakshi + serde via `#derive(Serialize)`+bayan, lossless i64 bit
patterns).

### Parity method

svara-independent math (error codes, RNG golden values, contour interpolation,
effective_f0, builder clamps, bridge conversions, species params + all 13
`resonance_seed`s, spatial DSP, intent/fatigue/emotion tables) is asserted at
**exact f64 bit patterns**. Full synthesis paths (which route through svara's
own DSP, where f32→f64 widening diverges bit-for-bit) are asserted
**structurally**: non-error return, exact buffer length, all-finite samples, and
bit-identical determinism across identical runs. This is the only meaningful
parity bar once the port widens f32→f64 through a dependency compiled in f64.

## Deferred / follow-up work

- **`[lib]` distlib bundle** (`dist/prani.cyr`) — assemble in dependency order once
  modules land; cross-module symbol-collision audit at close-out.
- **Version 2.0.0** — bump `VERSION` at port completion (per user directive). Held at
  1.1.0 during the port.
