# prani — Current State

> Refreshed every release. CLAUDE.md is preferences/process/procedures
> (durable); this file is **state** (volatile).

## Version

**2.0.0** — Rust → Cyrius port. The Rust line shipped through 1.1.0; the
language migration is a major break (2.0.0). All 15 modules ported and
cross-checked function-for-function against the frozen 3,527-line Rust oracle at
`rust-old/`. Per-module parity ledger: [`port-audit.md`](port-audit.md).

## Toolchain

- **Cyrius pin**: `6.3.45` (in `cyrius.cyml [package].cyrius`).
- Build: `cyrius build src/main.cyr build/prani`
- Test ONE suite: `cyrius test tests/<mod>.tcyr` (explicit path — no discovery).
- Bundle: `cyrius distlib` → `dist/prani.cyr` (reads `[lib].modules`).
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

- **svara** 3.0.0 — glottal source, formant filter, vocal tract (the excitation
  + resonance engine tract/voice bridge to).
- **naad** 2.1.0 — biquad filters (noise-only bandpass shaping).
- **hisab** 2.6.7 — `ease_in_out_smooth` (envelope curves) + transitive math.
- **goonj** 2.0.0, **sakshi** 2.4.3 — referenced transitively by svara/naad bundles.
- stdlib: syscalls, string, alloc, str, fmt, vec, io, args, assert, math, ganita,
  hashmap, bayan, tagged, fnptr, callback, bench.

## Port progress

**15 / 15 modules ported — PORT COMPLETE.** Parity per-module in
[`port-audit.md`](port-audit.md).

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
(serde round-trip + Display-string tests dropped — no serde, integer codes).
svara-independent math asserted at exact f64 bit patterns; full synthesis paths
(through svara's DSP, where f32→f64 diverges) asserted structurally
(non-error + exact length + all-finite + bit-identical determinism).

## Consumers

- **kiran** (game engine), **joshua** (game manager) — once they port up the stack.

## In flight

Nothing — the 2.0.0 port is complete. Follow-ups for a later cycle:
- Broaden hot-path benchmarks + capture a Rust-vs-Cyrius comparison.
- Consumer-green (kiran/joshua) once they port up the stack.
