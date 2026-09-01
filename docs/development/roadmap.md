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
CHANGELOG. What is left below is **2.1.0** — the first release that changes the
public surface — plus two externally-blocked items and the watch list.

---

## 2.x — the work that changes the public surface

### 2.1.0 — two lanes: f32 throughout, and the allocation-failure contract

Both are minor-bump work on the same functions, so they share a release. **Lane B
(allocation) can start today; Lane A (f32) is blocked on svara and naad.** If
Lane A is still blocked when Lane B is ready, ship Lane B as 2.1.0 and Lane A
becomes 2.2.0 — do not hold a finished contract behind a blocked dependency.

#### Lane A — f32 throughout, to match the oracle

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

**Done when**: prani's arithmetic is f32 end to end, the tolerance-loosened
assertions are re-tightened to bit-exact against the oracle, and 2.0.7's
comparison is re-run f32-vs-f32 so the remaining gap is attributable to codegen
alone.

#### Lane B — the allocation-failure contract

**Source**: audit finding **F9**, accepted rather than repaired in 2.0.3
precisely because it is this size.

`alloc()` returns 0 on exhaustion and prani stores through it at **27 sites**;
`vec_new()` can return 0 and `vec_push` returns -1 on failure, and **no call site
in the library checks either**. An exhausted arena therefore produces a null
dereference or a silently truncated buffer — including a synthesis result short
of the length the caller was told to expect.

This is a minor bump rather than a patch because the only real fix puts a
fallible return on every constructor in the library (`crvoice_new`,
`dcblocker_new`, `prani_rng_new`, `species_params`, `crtract_options_new`, the
sequence constructors), which every consumer must then check. Decide the shape
first — the choice is between a negative `PRANI_ERR_ALLOC` from every
constructor, or a single checked-allocation helper that the constructors funnel
through.

Related and already done, from the other end: 2.0.3 removed the two paths that
allocated per frame and per block (`prani_ffi_voice_set_size`, the synthesis
block loop), so the arena is under less pressure than it was — but that reduces
the odds of exhaustion, it does not handle it.

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
  stream, which is correct only while both failure causes are configuration
  properties fixed at `stream_new` time. **Revisit if a transient failure mode is
  ever introduced.**
- **svara's sample-rate floor is a measurement, and it has already moved once.**
  ADR-0001 chose not to restate svara's rule in prani source. That rule was
  ">1000 Hz" until svara **3.5.4** repaired two defects prani found, after which
  the real floor is **>1200 Hz** — svara's fixed 600 Hz subglottal bandpass needs
  nyquist above 600. `tests/hardening.tcyr`'s **F12** group pins the 1200/1201
  and 7500/7501 boundaries with controls. **If those controls fail, the floor
  moved again — re-measure it, do not hardcode a new number in prani.**
- **Re-running the Rust comparison depends on crates.io, not on this repo.**
  The oracle at tag `2.0.3` builds against `svara 1.0.0` / `naad 1.0.0` /
  `hisab 1.2.0`, checksums pinned in its `Cargo.lock`. **The risk that mattered
  is spent**: 2.0.7 captured the comparison while it still built, and
  `docs/benchmarks.md` records it with its method. If a crate is ever yanked, a
  *re-run* becomes impossible — the recorded figures do not. Only revisit this if
  someone needs a fresh comparison, e.g. after 2.1.0 Lane A makes it f32-vs-f32.
