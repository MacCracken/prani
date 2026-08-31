# prani — Current State

> Refreshed every release. CLAUDE.md is preferences/process/procedures
> (durable); this file is **state** (volatile).

## Version

**2.0.2** — toolchain + dependency catch-up on top of the completed port. The
Rust line shipped through 1.1.0; the language migration was a major break
(2.0.0); 2.0.1 restored logging (sakshi) + serde (`#derive(Serialize)`+bayan);
2.0.2 moves the Cyrius pin 124 releases forward and every dependency to its
current tag, absorbing two breaking upstream renames (`FILTER_BANDPASS` →
`NAAD_FILTER_BANDPASS`, `bayan_json_v_parse_str` → `bayan_json_v_parse_buf`).
All 16 modules (15 ported + `logging.cyr`) remain cross-checked
function-for-function against the frozen 3,527-line Rust oracle at `rust-old/`.
**717 parity assertions green across 16 suites** — unchanged from 2.0.1,
assertion for assertion. Per-module parity ledger:
[`port-audit.md`](port-audit.md).

## Toolchain

- **Cyrius pin**: `6.5.36` (in `cyrius.cyml [package].cyrius`).
- `lib/` is vendored from the matching snapshot — refresh with `cyrius lib sync`
  (declared `[deps].stdlib` subset; `--full` copies the whole 108-file snapshot,
  which this project does not want). Carries bayan **1.5.2**.
- Build: `cyrius build src/main.cyr build/prani`
- Test ONE suite: `cyrius test tests/<mod>.tcyr` (explicit path — no discovery).
- Bundle: `cyrius distlib` → `dist/prani.cyr` (reads `[lib].modules`).
- Gate: `cyrius audit` (fmt · lint · docs · tests · bench). Note its fmt gate is
  **stricter than `cyrius fmt <file> --check`** — the per-file check accepts
  4-space continuation indents, the audit gate requires the canonical 2 spaces
  per open paren (cyrfmt became paren-aware in 6.5.28).
- **Parallel-porting concurrency**: every `cyrius …` call re-resolves deps and
  races on `cyrius.lock`. Serialize all toolchain calls behind
  `flock <scratch>/prani-build.lock cyrius …`.

## Source

- Rust reference: 3,527 lines across 17 files at `rust-old/` (frozen). `lib.rs`
  (organization/prelude) and `math.rs` (f32 transcendental wrappers, folded into
  `f64_*` builtins) carry no independent Cyrius module.
- Cyrius port: `src/main.cyr` (smoke) + 15 per-module `src/*.cyr`, each validated
  by `tests/*.tcyr`.

## Dependencies

Consumed as Cyrius distlib bundles (git+tag in `cyrius.cyml`):

- **svara** 3.5.3 — glottal source, formant filter, vocal tract (the excitation
  + resonance engine tract/voice bridge to).
- **naad** 2.2.2 — biquad filters (noise-only bandpass shaping).
- **hisab** 2.11.2 — `ease_in_out_smooth` (envelope curves) + transitive math.
- **goonj** 2.0.4, **sakshi** 2.4.12 — goonj referenced transitively by the
  svara/naad bundles; sakshi both transitive and called directly by
  `src/logging.cyr`.
- stdlib: syscalls, string, alloc, str, fmt, vec, io, args, assert, math, ganita,
  hashmap, bayan, tagged, fnptr, callback, bench.

svara 3.5.3 pins hisab 2.11.2 / naad 2.2.1 / goonj 2.0.4, and hisab 2.11.2 pins
sakshi 2.4.11, so this set is coherent with what the bundles themselves pin.
naad 2.2.2 (over svara's 2.2.1) and sakshi 2.4.12 (over hisab's 2.4.11) are
deliberate one-step-ahead bugfix picks. sakshi being one ahead of the toolchain
snapshot is why every build prints `./lib/ shadows version-pinned …` — expected,
not a fault.

## Port progress

**15 / 15 Rust modules ported (+ `logging.cyr`) — PORT COMPLETE.** Parity
per-module in [`port-audit.md`](port-audit.md).

| Layer | Modules | Status |
|-------|---------|--------|
| L0 foundation | error, rng, dsp | ✅ |
| L0/L1 leaves | spatial, vocalization, fatigue, emotion | ✅ |
| L1 spine | sequence, species, bridge | ✅ |
| L2 svara bridge | tract | ✅ |
| L3 orchestration | voice | ✅ |
| L4 composites | preset, stream | ✅ |
| L5 FFI | ffi | ✅ |

Delivered solo (foundation + keystone vocalization/sequence) + dependency-ordered
parallel workflow waves (leaves → tract+bridge → voice → preset+stream → ffi),
each integrated and independently re-verified in the main tree against `rust-old/`.

## Tests

One `tests/<module>.tcyr` per module, cross-checked against the Rust oracle
serde roundtrip tests INCLUDED (serde restored in 2.0.1 via `#derive(Serialize)`
+bayan, f64 fields as lossless i64 bit patterns); Display-string tests dropped.
svara-independent math asserted at exact f64 bit patterns; full synthesis paths
(through svara's DSP, where f32→f64 diverges) asserted structurally
(non-error + exact length + all-finite + bit-identical determinism).

## Consumers

- **kiran** (game engine), **joshua** (game manager) — once they port up the stack.

## In flight

Nothing — the port is complete and current on toolchain + dependencies.
Follow-ups:
- Broaden hot-path benchmarks + capture a Rust-vs-Cyrius comparison.
- Consumer-green (kiran/joshua) once they port up the stack.
- Pre-existing, unrelated to 2.0.2: 7 `cyrius lint` line-length warnings in
  `src/vocalization.cyr` (lines 100-106, the aligned `prani_intent_modifiers`
  dispatch table — `cyrius lint` itself still exits 0), and 10 undocumented
  public fns on `cyrius audit`'s docs gate, which is the one thing making
  `cyrius audit` exit 1. Both carried forward unchanged from 2.0.1.
