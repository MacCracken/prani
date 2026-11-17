# prani — Roadmap

> **Open work only.** Completed milestones are not restated here — they live in
> [`CHANGELOG.md`](../../CHANGELOG.md), and the port ledger is
> [`port-audit.md`](port-audit.md). Current state is [`state.md`](state.md).

Every item carries its provenance. An item with no source in the tree is not on
this roadmap.

**The port is finished and the oracle is gone.** The 2.0.x arc ran from the
language port through parity re-verification, input-range validation, runnable
examples and the Rust comparison, and ended by retiring `rust-old/` in 2.0.8;
2.0.9–2.0.11 then repaired what its own measurements found. All of it is in the
CHANGELOG, and so is **2.2.0**, the first release that changed the public surface
(the allocation-failure contract, ADR-0008). What is left below is **2.3.0**, the f32
lane that 2.2.0 shipped without, plus two externally-blocked items and the watch
list.

---

## 2.x — the work that changes the public surface

### 2.3.0 — Lane A: f32 throughout, to match the oracle

*Planned as one of 2.2.0's two lanes (and numbered 2.1.0 before that). Lane B, the
allocation-failure contract, was ready first and shipped alone as 2.2.0. That
followed this roadmap's own rule: do not hold a finished contract behind a blocked
dependency. The dated records still call this lane by the older numbers.*

**Source**: 2.0.7's measured comparison, and the fact that the constraint this
port was built on **no longer holds**.

`port-audit.md` has carried this as a standing convention since 2.0.0:

> **f32 → f64 everywhere** (svara/naad/hisab are f64-only; widening is **forced**
> and improves precision)

**It is not forced any more.** ganita **1.1.4** ships a 23-function f32 scalar
tier — `sin cos exp ln sqrt pow atan2 hypot cbrt floor ceil trunc round abs neg
min max clamp lerp sign log2 exp2` — and it is **already vendored in
`lib/ganita.cyr`**. prani calls nothing outside that set: no `tanh`, `sinh`,
`cosh`, `asin` or `acos` appears anywhere in `src/`. Every one of prani's own
190 `f64_mul`s, 69 `f64_add`s, 16 `f64_sin`s and 55 `f64_clamp`s has an f32
equivalent available today.

**What 2.0.7 measured, on one host, same species, same durations, same 44100 Hz:**

| | Rust oracle (f32) | Cyrius port (f64) | |
|---|---:|---:|---:|
| `wolf_howl_1s` | 1.39 ms | 21.9 ms | **15.8× slower** |
| median across 13 synthesis benchmarks | | | **15.7× slower** |
| range | | | 9.9× – 17.4× |
| realtime factor, wolf howl | **719×** | **45.6×** | |

The gap is remarkably *uniform* — every apparatus, every duration, within a
9.9–17.4× band. That is the signature of something systemic, not one slow path.

⚠ **f32 is not proven to be the cause, and this item must not assume it is.**
Three things differ between those two columns and only one is float width: the
oracle is prani 1.1.0 on **svara 1.0.0 / naad 1.0.0** compiled by LLVM at
`--release`; the port is 2.0.6 on **svara 3.5.4 / naad 2.2.2** compiled by cycc.
Three major versions of the DSP dependency and a different optimizer sit in that
15.7×. The honest prior is that **codegen dominates and float width is second**.

What makes f32 worth doing anyway, independent of the ratio:

1. **Parity.** The oracle is f32. The port's whole correctness bar is "matches
   what Rust did", and every tolerance loosened in the suites
   (`PRANI_EPSILON` = f32::EPSILON promoted to f64) exists because of the
   widening. f32 would let those become bit-exact.
2. **SIMD width.** cycc has `f32v8` — eight lanes against `f64v4`'s four. A
   vectorised f32 DSP path has twice the lanes of the f64 one.
3. **Half the memory traffic** on every sample buffer, on a bump allocator that
   never frees.

**Blocked on the dependencies, and that is the point.** prani cannot do this
alone: `svara_glottal_next_sample` and `svara_tract_process_sample` are called
per sample and are f64 on both sides. Filed as **P0 on svara and naad**. Until
they move, prani could convert only its own arithmetic and would widen at the
boundary — a hybrid that costs work and buys neither parity nor speed.

**Do not start this before svara and naad have.** Sequence: naad → svara → prani.

> **Update, 2.1.1 (2026-11-07): both P0s are closed, and neither dependency will
> move.** naad measured `filter_biquad_process_sample` at 18.3 ns in f64 and in f32,
> and closed its P0 in its ADR-0008. svara followed in its own ADR-0008, *The
> synthesis core stays f64*, which says prani's roadmap is no longer blocked on it.
> naad also found that ganita's f32 tier routes through f64 (`sin` 15% slower, `exp`
> 93%) and that `f32` cannot be a fn return type at cyrius 6.7.18. So the hybrid
> above is the only form this lane can take. **It needs a decision: close it with an
> ADR as svara did, or re-scope it.** That decision is not made here.

**Done when**: prani's arithmetic is f32 end to end, the tolerance-loosened
assertions are re-tightened to bit-exact against the oracle, and 2.0.7's
comparison is re-run f32-vs-f32 so the remaining gap is attributable to codegen
alone.

### Consumer-green — kiran and joshua

**Source**: the 2.0.0 criteria carried this unmet, and it is the one criterion
still open.

Neither consumer has ported up the stack, so this is **blocked externally** and
has no version. It is the last real check on the FFI surface: nothing has driven
`prani_ffi_stream_start` / `_fill` / `_is_finished` from a host callback, which is
where 2.0.3's stream repairs (ADR-0003) would be exercised.

**It no longer gates `rust-old/`'s retirement** — decided 2026-08-31, with
2.0.6's `streaming.cyr` standing in for the lifecycle check. It remains open as
real work: an example is not a host, and a host will find things an example
cannot (threading, callback timing, lifetime across frames).

### Conditional — a raw C ABI

**Source**: the previous roadmap's "Out of scope (for 2.0.0)" note.

The `ffi` module is re-expressed in Cyrius conventions (handles = pointers,
buffers = vecs), so it is not ABI-compatible with the old Rust `extern "C"`
surface. **Only worth building if a non-Cyrius host actually needs it** — no such
consumer exists today. Recorded so the decision is not rediscovered.

---

## Watch items — not scheduled, but written down

Things deliberately left alone, with the condition that would change that.

- **Two duplications held at two instances.** CLAUDE.md says wait for the third
  before extracting. `crtract_synthesize_stridulatory`'s bee branch is
  byte-identical to `crtract_synthesize_vibratile`, and the `boundary_boost`
  block is duplicated verbatim between `voice.cyr` and `stream.cyr`. **Extract on
  the third instance**, not before. (Source: audit report, Refactor.)
- **ADR-0003's retirement rule.** A failed `stream_fill_buffer` retires the
  stream. 2.2.0 revisited this when a refused allocation became a third failure
  cause (ADR-0008), and retirement stood: the bump arena frees nothing, so a refusal
  is not one a later fill could wait out. `stream_last_error` now says which cause it
  was. **Revisit if prani ever allocates from something that can free**, an arena a
  host can reset for instance, because then a refusal can be transient.
- **svara's sample-rate floor is a measurement, and it has already moved once.**
  ADR-0001 chose not to restate svara's rule in prani source. That rule was
  ">1000 Hz" until svara **3.5.4** repaired two defects prani found, after which
  the real floor is **>1200 Hz** — svara's fixed 600 Hz subglottal bandpass needs
  nyquist above 600. `tests/hardening.tcyr`'s **F12** group pins the 1200/1201
  and 7500/7501 boundaries with controls. **If those controls fail, the floor
  moved again — re-measure it, do not hardcode a new number in prani.**
- **prani's excitation gain is solved against svara's pulse.** `PR_TR_EXC_GAIN`
  (`src/tract.cyr`, THE EXCITATION LEVEL) puts svara 3.11.0's corrected Rosenberg
  pulse back at the oracle's loudness: +7.37 dB, solved so the median of 56
  reference renders matches 2.1.1. It divides svara's own `SV_G_ROSENBERG_GAIN`
  back out, so a recalibration of that constant flows through. A change to the
  pulse's shape does not. `tests/voice.tcyr`'s excitation-level group pins the
  median, the ±7.5 dB spread and full scale, and `tests/tract.tcyr` pins the two
  svara facts the maths rests on. **If either group fails on a svara bump,
  re-solve the gain by the method in that block. Do not nudge it.**
- **Re-running the Rust comparison depends on crates.io, not on this repo.**
  The oracle at tag `2.0.3` builds against `svara 1.0.0` / `naad 1.0.0` /
  `hisab 1.2.0`, checksums pinned in its `Cargo.lock`. **The risk that mattered
  is spent**: 2.0.7 captured the comparison while it still built, and
  `docs/benchmarks.md` records it with its method. If a crate is ever yanked, a
  *re-run* becomes impossible — the recorded figures do not. Only revisit this if
  someone needs a fresh comparison, e.g. after 2.3.0 Lane A makes it f32-vs-f32.
