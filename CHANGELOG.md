# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.0.3] - The P(-1) sweep: one segfault on the primary API, and the contract that changed underneath it

Scaffold-hardening and security sweep of the whole tree, closing the one unmet
2.0.0 criterion the roadmap still carried (*"Security audit pass — deferred to a
work-loop cycle"*). **11 findings, 8 repaired, 3 accepted and written down**;
full report in [`docs/audit/2026-08-30-audit.md`](docs/audit/2026-08-30-audit.md),
three divergences from the oracle in [`docs/adr/`](docs/adr/). Suite **717 → 770
assertions / 17 suites**; `cyrius audit` **exits 0 for the first time** (fmt ·
lint · docs · tests · bench all clean). No behaviour change on any valid input:
every one of the 717 pre-existing parity assertions passes untouched.

### Fixed — CRITICAL: `crvoice_vocalize` segfaulted on any sample rate svara rejects

`rust-old`'s `VocalTract::new` returned `Self` — it could not fail, so
`CreatureTract::new` returns `Self` too. svara 3.x hardened that constructor into
a checkable negative code (svara's own ADR: *"`svara_tract_new` errors where Rust
panics"*) and it rejects **every sample rate at or below 1000 Hz**, plus negative
and non-finite ones. The port kept the oracle's shape and used the return value
as a pointer regardless:

```
crvoice_vocalize(wolf, HOWL, /*sample_rate*/ 10.0, /*duration*/ 1.0)   ->  exit 139
```

`svara_tract_set_formants_from_target(-1, target)` dereferenced `-1`. Reachable
from `crvoice_vocalize`, `crvoice_vocalize_with_intent`, the cat-purr path,
`stream_fill_buffer` and `crtract_from_json_str` — the entire public synthesis
surface. `crtract_new` now checks `svara_is_err` and returns a negative
`PRANI_ERR_*`; all five call sites propagate it
([ADR-0001](docs/adr/0001-check-svara-tract-constructor.md)).

⭐ **This is a third defect category, distinct from the two the port's parity bar
was built to catch.** It is not an inherited oracle defect and not a
transcription error: it is **a contract that changed underneath the port**. hisab
recorded the value-level form of this as *"a constant derived from a dependency
is a measurement, and it goes stale silently."* This is the same sentence one
level up — **a contract derived from a dependency is a measurement.** 2.0.2's
dependency bump checked every symbol prani calls for renames and signature
changes and found two; nothing in that check asks whether a return value still
*means* what it used to, and nothing in 717 parity assertions asks either,
because the oracle has no such failure to compare against.

### Fixed — HIGH: a JSON parse failure was indistinguishable from a successful parse

The four deserializers never looked at `bayan_json_v_parse_buf`'s result. bayan's
value accessors are all null-safe by design (`bayan_json_v_obj_get(0, k)` and
`bayan_json_v_int(0)` both return 0), so the two behaviours composed into a
silent one: **a malformed document produced a fully-formed struct with every
field 0, returned as a success.** The zeros are not inert — a `CreatureVoice`
with `size_scale` 0.0 makes `crvoice_effective_f0` divide by zero, and a
`SpeciesParams` with `f0_min == f0_max == 0` collapses every downstream clamp.

`crvoice_from_json_str`, `crtract_from_json_str`, `preset_from_json_str` and
`sequence_call_phrase_from_json_str` now return a negative `PRANI_ERR_*` on a
null or unparseable input. This **restores** the oracle's contract rather than
diverging from it — Rust's `serde_json::from_str` returns `Result`, and 2.0.1
restored the codecs without the failure half
([ADR-0002](docs/adr/0002-deserializers-report-parse-failure.md)). Field-range
validation is explicitly still out of scope and now written down as such.

### Fixed — HIGH: NoiseOnly synthesis dereferenced a filter that may be absent

`crtract_synthesize_noise` carried the comment *"noise_filter is always present
here"* while `crtract_new` populates that field only for `PRANI_APP_NOISE_ONLY`
and only `if (prani_is_err(nf) == 0)`. The invariant the comment asserted is one
the code beside it admits can be false. Now checked, returning
`PRANI_ERR_SYNTHESIS_FAILED`.

### Fixed — MEDIUM: a failed stream fill reported success

`rust-old/src/stream.rs:184` guards only the copy with `if let Ok(block)`, then
unconditionally advances `samples_rendered` and returns `to_render` — telling the
host it wrote N samples having written none, so the host plays whatever the
buffer held before (on a reused audio buffer, the previous block: an audible
repeat attributed to prani, with no error anywhere). Inherited by the port. A
failed fill now returns **0** and retires the stream, so a host draining
`while (stream_is_finished(s) == 0)` terminates instead of spinning on a failure
that repeats every call ([ADR-0003](docs/adr/0003-failed-fill-reports-zero-and-retires.md)).
A zero-length caller buffer also returns 0 immediately — it could never advance
the stream either. The same ADR records a divergence that had never been written
down: the port applies tilt and amplitude only when a block was produced, where
the oracle applies them to the stale buffer regardless.

### Fixed — MEDIUM: an RTPC setter allocated on every call, forever

`prani_ffi_voice_set_size` is a real-time parameter a host may call every frame,
per creature. It built a fresh `CreatureVoice` **and**, through `crvoice_new`, a
fresh `SpeciesParams`, copied five fields out and dropped the shell. Cyrius's
allocator is a bump arena that never frees, so **every call retained 168 bytes
permanently** — measured, not calculated: 1000 calls, `alloc_used()` before and
after, 168,000 bytes (`sizeof(CreatureVoice)` 40 + `sizeof(SpeciesParams)` 128).
At 60 fps across 100 creatures that is about 1 MB/second retained for the life of
the process. svara's own `streaming.cyr` states the rule this broke: *an
allocation inside an audio callback is not a leak that grows slowly, it is one
that ends the process.*

It now allocates **exactly 0 bytes**, asserted over 1000 calls in
`tests/hardening.tcyr`, and is proven field-for-field identical to the rebuild it
replaced. Making that possible is the one structural change in this release:
`species_params` is split into `species_params_into` (the 13-species table,
writing into a caller-owned struct) and a thin allocating wrapper, so both an
allocating and a non-allocating entry point are served by **one** copy of the
table.

⚠ The first attempt at this repair still measured **128 B/call** and the
assertion failed — `crvoice_reset_individual` had been written to call
`species_params()`, which allocates. Only the table split got it to 0. The
assertion was written before the repair was believed, which is the only reason
the shortfall was caught rather than shipped as "no longer allocates".

### Fixed — LOW: two allocation and loop-termination hardenings

- **One `SynthesisOptions` per synthesis call, not one per 20 ms block** (50 per
  second of audio, each retained forever). Nothing reads it after
  `crtract_synthesize` returns, so it is allocated once above the loop and
  re-armed per block.
- **`block_size` is floored at 1.** `(sample_rate * 0.02) as usize` is 0 below
  50 Hz and the oracle's loop (`voice.rs:227-231`) advances `rendered` by exactly
  that, so it never terminates. This is **defense in depth, not a fix for a live
  defect** — the CRITICAL guard above now rejects everything below ~1000 Hz
  before the loop is built, so nothing can reach it. Kept because a loop should
  not depend on a distant guard for termination, and asserted as arithmetic.

### Accepted, not repaired — written down rather than silently carried

- **Unchecked allocation (27 sites).** `alloc()` returns 0 on exhaustion and
  prani stores through it; `vec_push` returns -1 and no call site checks, so an
  exhausted arena silently truncates a buffer. The fix is an error return on
  every constructor in the library — a larger API decision than a repair release
  should make. On the roadmap; the two measured allocation reductions above
  attack the same problem from the other end.
- **`sequence_synthesize_chorus` length overflow** on a `timing_spread` around
  1e14 seconds, and **NaN propagation** through `f64_clamp` into svara. Both are
  bounded rather than open (`f64_to(NaN)` saturates to i64::MIN, so every
  affected loop bound is negative and simply does not run), and both belong with
  the general input-range validation deferred in ADR-0002.

### Changed — refactor

Two loop shapes had reached **three verbatim call sites each**, which is the bar
CLAUDE.md sets, so they were extracted into `dsp.cyr`:
`prani_vec_extend(dst, src)` and `prani_vec_push_zeros(v, n)`. Behaviour is
unchanged; `prani_vec_push_zeros` makes one shared edge explicit instead of
incidental — every call site derives its count from `f64_to(seconds * rate)`,
which saturates to i64::MIN on NaN, so a non-positive count must append nothing.

**Two duplications were found and deliberately left alone**, both at two
instances rather than three: `crtract_synthesize_stridulatory`'s bee branch is
byte-identical to `crtract_synthesize_vibratile`, and the `boundary_boost` block
is duplicated verbatim between `voice.cyr` and `stream.cyr`. Both are on the
roadmap for the third instance.

### Added — `tests/hardening.tcyr` (53 assertions)

One group per repaired finding, each written so it **fails on the 2.0.2 tree** —
the memory-safety groups crash the process there rather than reporting, which is
what makes them worth keeping. Every group carries controls (44100 and 8000 still
build a tract; valid JSON is still accepted; a 128-sample buffer still fills) so
it cannot pass vacuously by rejecting everything.

### Fixed — the lint and docs gates

- **7 lint warnings** in `src/vocalization.cyr:100-106` — the aligned
  `prani_intent_modifiers` dispatch table ran to 123 characters against a
  120-character limit. Rewrapped to block form; `cyrius lint` is clean tree-wide.
- **10 undocumented public functions** now documented: the four
  `*_from_json_str` deserializers (each stating its failure code), the three
  `sequence_*_new` constructors, the two `*_modifiers_make` builders, the five
  `prani_log_*` wrappers, and `main`. `cyrius audit`'s docs gate is complete, and
  with fmt and lint already clean **`cyrius audit` now exits 0** — it had exited
  1 on the docs gate since the port landed.

### Notes

- **No performance claim.** `dcblocker_process` 19 ns/sample, `prani_rng_next_f32`
  14 ns/sample, `emotion_evaluate` 69 ns/frame, `crvoice_vocalize` (wolf howl,
  0.05 s @ 8 kHz) 211 µs — every row within noise of 2.0.2. The optimization work
  in this release is measured in **bytes retained**, not nanoseconds, and both
  ends of each figure were measured with `alloc_used()`.
- ⚠ **The probe was wrong before the code was.** The first three bisect runs
  reported a hang, and the CRITICAL was very nearly filed as an infinite loop.
  Two instrument defects produced that: `${PIPESTATUS[0]}` is a bashism that
  expands to nothing under this project's zsh, so every *"exit=124 / timed out"*
  reading was fabricated by the harness rather than measured; and the probes
  printed through `print`, which was silently emitting nothing in that unit, so
  the bisect markers that would have located the fault never appeared. With `$?`
  captured directly and markers written through `test_group`'s raw syscall, the
  same input reported **exit 139 — SIGSEGV**, and the real defect was three
  frames further up than the one being chased. *Check the probe before believing
  the probe.*
- **What was checked and found clean** is recorded in the audit report rather
  than omitted: `vec_get` is bounds-checked and aborts loudly; both contour
  interpolations guard their divisor correctly (the guard class hisab has seen
  fail thirty times); the chorus modulo divisor is floored before use; the Doppler
  output length is bounded by the velocity clamp; every `prani_ffi_*` entry point
  null-checks its handle; and prani has no syscall, command-execution, path or
  network surface at all.
- `dist/prani.cyr` regenerated at v2.0.3.

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
