# prani — Roadmap

> **Open work only.** Completed milestones are not restated here — they live in
> [`CHANGELOG.md`](../../CHANGELOG.md), and the port ledger is
> [`port-audit.md`](port-audit.md). Current state is [`state.md`](state.md).
>
> **2.0.x** is one arc with one destination: **retire `rust-old/`**. Every item
> in it is either a gate on that removal or a thing the oracle still holds that
> the Cyrius tree does not. **2.x** is the work that changes the public surface
> or is blocked outside this repo.

Every item carries its provenance. An item with no source in the tree is not on
this roadmap.

---

## 2.0.x — the preserve-first arc, ending in `rust-old/`'s removal

`rust-old/` is 4,646 lines (3,527 in `src/`, 939 in `tests/integration.rs`, plus
`benches/`). It is the parity oracle and CLAUDE.md forbids modifying it. Deleting
it is irreversible in the working tree, so this arc follows the preserve-first
gate svara and goonj both used: **everything the Rust still holds that nothing in
the Cyrius tree does must exist before the directory goes.**

Measured facts this arc rests on, all verified 2026-08-31:

| Fact | Value |
|---|---|
| Rust `#[test]` blocks | **73** — 72 in `tests/integration.rs`, 1 in `src/lib.rs`. No `#[cfg(test)]` mods in the source modules. |
| Cyrius assertions | 770 across 17 suites |
| Rust benchmarks | **10** in `benches/benchmarks.rs` — wolf howl, cat purr, cricket stridulate, dragon roar, snake hiss, songbird trill, lion roar, wolf alarm howl, bee buzz, crow screech |
| Cyrius benchmarks | 4 |
| `rust-old/` citations in tracked files | **176**, across 45 files (`src/`, `tests/`, and every doc) |
| Tags carrying `rust-old/` | **2.0.0 and later** (17 `src/*.rs` + `integration.rs`). **1.1.0 and earlier carry the Rust at `src/` instead** — the port created `rust-old/`, so the recovery path differs by era. |
| Rust build reproducible? | **Yes** — `cargo fetch --locked` succeeds; `svara 1.0.0`, `naad 1.0.0`, `hisab 1.2.0` are crates.io releases with checksums pinned in `Cargo.lock`. |

### 2.0.4 — Port parity re-verification (the gate's first half)

**Source**: this arc. Removal is only safe if the Cyrius suites assert everything
the Rust suites assert; nothing has checked that claim directly.

The port was verified module-by-module against the oracle's *source*. It has
never been verified against the oracle's *tests*. Those are different questions,
and svara found five real gaps when it asked the second one (its 3.5.1).

- **Audit all 73 Rust `#[test]` blocks against the 17 Cyrius suites.** For each,
  identify the assertion that covers it, or record that none does. The output is
  a table with a row per Rust test — that table is the artifact, not a summary.
- **Every gap found becomes a new 2.0.x item**, sequenced after this one. A gap
  that turns out to be a real behavioural difference is a defect, not a test gap,
  and takes priority over everything else in the arc.
- **Establish how to read the oracle after it is gone.** This is the release that
  reads `rust-old/` most closely, so it is where the recovery rule belongs:
  add an ADR plus a `CONTRIBUTING.md` note carrying the incantation, and make
  "cite a tag, not a bare path" a standing instruction. The era split above is
  part of the rule, and both halves are verified to work today:

  ```sh
  git show 2.0.3:rust-old/src/voice.rs    # the port era — rust-old/ exists
  git show 1.1.0:src/voice.rs             # the Rust era — the Rust WAS src/
  ```

**Done when**: the per-test table exists in `docs/development/`, every gap is
either closed or filed as a 2.0.x item, and the recovery rule is written down.

### 2.0.5 — Input-range validation beyond parse success

**Source**: [ADR-0002](../adr/0002-deserializers-report-parse-failure.md)
("Deferred, not rejected"), audit findings **F10** and **F11**.

2.0.3 made the deserializers reject input that does not *parse*. A document that
parses but carries nonsense still produces a struct, and the same gap shows up in
two other places, so it is one piece of work rather than three:

- **Deserializer field ranges.** `crvoice_from_json_str` will accept
  `size_scale` = 0.0 or a negative `f0_min`; the builders (`crvoice_with_size`
  et al.) clamp, the deserializers do not. Decide whether prani validates its own
  serialized output, then apply it uniformly to all four codecs.
- **F11 — NaN propagation.** `f64_clamp` returns NaN for a NaN input (measured;
  both comparisons are false), so a NaN `sample_rate`, `duration`, `velocity` or
  `distance` passes every clamp in the tree. Bounded today rather than open —
  `f64_to(NaN)` saturates to i64::MIN, so affected loop bounds go negative and
  simply do not run, and ADR-0001's guard rejects a NaN sample rate outright —
  but the result is a silent empty buffer where an error code belongs.
- **F10 — chorus length overflow.** `base_samples + max_offset_samples * 2`
  overflows i64 at a `timing_spread` around 1e14 seconds.

**Done when**: every public entry point rejects non-finite and out-of-range
numeric input with a `PRANI_ERR_*` instead of returning empty or NaN, with the
range for each parameter written down; `tests/hardening.tcyr` grows a group per
entry point.

### 2.0.6 — Runnable examples

**Source**: `CLAUDE.md` advertises *"[`docs/examples/`](../examples/) — Runnable
examples"*; the directory contains a `.gitkeep` and nothing else.

**A gate item, not a nicety.** `rust-old/` has no `examples/` either, so nothing
is lost by removal here — but once the oracle is gone, a worked example is the
only executable statement of how the API is meant to be driven, and the fastest
way for a consumer to be wrong about an API is to have no example of it. svara
closed exactly this gap before its own oracle retirement.

| Example | Shows |
|---|---|
| `basic.cyr` | a wolf howl, sample statistics, the vec-or-negative-code return convention |
| `species_tour.cyr` | one call per vocal apparatus — laryngeal, syringeal, stridulatory, vibratile, noise-only |
| `error_handling.cyr` | every failure shape, including the three ADR divergences (rejected sample rate, rejected JSON, a retired stream) |
| `streaming.cyr` | the `stream_fill_buffer` real-time path, and the allocation rule that governs it |
| `sequencing.cyr` | bouts, phrases, and a multi-voice chorus |

**Done when**: CI builds and runs all five on every push, so they cannot rot
against the API.

### 2.0.7 — Benchmark breadth and the Rust comparison

**Source**: [`docs/benchmarks.md`](../benchmarks.md) — *"A like-for-like
Rust-vs-Cyrius comparison on identical hardware is a follow-up"*.

⚠ **This item must complete before 2.0.8. It is the only one that does.** The
comparison needs `rust-old/` present and buildable; after removal it can only be
run from a tag, and it depends on three crates.io releases staying published.
`cargo fetch --locked` succeeds today — that is a window, not a guarantee.

The Rust harness answers both halves of this item at once. Its **10 benchmarks**
cover exactly the spread prani's 4 miss — every vocal apparatus (songbird trill
and crow screech for syringeal, cricket for stridulatory, bee for vibratile,
snake hiss for noise-only) — so mirroring it broadens coverage *and* makes the
comparison like-for-like by construction.

- **Mirror the 10 Rust benchmarks** in `tests/prani.bcyr`, same species, same
  durations. A regression in syringeal, stridulatory, vibratile or noise-only
  synthesis is currently invisible.
- **Run both harnesses on one host and record the method.** The 1.1.0 crate
  claimed ~1000× realtime against the port's measured ~236×, and that gap has
  never been measured under one method. Note that `lib/bench.cyr` measures and
  subtracts the timer floor (cyrius 6.5.19+) and criterion does not — the
  comparison must say so or it is not like-for-like.
- Build into `CARGO_TARGET_DIR` outside the repo; `rust-old/` stays unmodified.

**Done when**: `docs/benchmarks.md` carries a per-apparatus table and a dated
Rust-vs-Cyrius run with its method written down.

### 2.0.8 — Retire `rust-old/`

**Source**: this arc. The removal itself.

Patch-level because it changes nothing a consumer sees — no API, no behaviour,
no bundle content. What it changes is what prani's parity guarantee *rests on*:
after this, parity is asserted by the suites alone.

**Gate — every one of these before the directory goes:**

- [ ] 2.0.4 complete; every gap it found closed, not merely filed
- [ ] 2.0.6 complete — examples exist and CI runs them
- [ ] **2.0.7 complete — the comparison captured while the oracle still builds**
- [ ] **Reference sweep**: all **176** `rust-old/…` citations across 45 tracked
      files converted to the tag-qualified form from 2.0.4. A bare
      `rust-old/src/x.rs:NN` in a comment becomes unresolvable the day the
      directory goes; `git show <tag>:…` does not. Note `dist/prani.cyr` carries
      these too and is regenerated, not edited.
- [ ] **Tree green with the directory moved aside** — `mv rust-old ../` then
      `cyrius audit` exit 0, proving nothing in the build, tests, or bundle
      reads it. Do this *before* deleting, not after.
- [ ] `CLAUDE.md` updated: it currently says *"Original Rust at `rust-old/` is
      the reference oracle — do not modify it; cross-check the port against it"*,
      which becomes false. Replace with the recovery rule.
- [ ] **Consumer-green — see the open question below.**

#### ⚠ Open question: is consumer-green a hard gate?

svara treated it as one. For prani it means kiran or joshua building against
`dist/prani.cyr`, and **neither has ported up the stack, so there is no date**.
Making it a hard gate blocks 2.0.8 indefinitely on work outside this repo.

The argument for holding: nothing has ever driven the FFI surface from a real
host, so `prani_ffi_stream_start` / `_fill` / `_is_finished` are the least-tested
part of the library — and that is exactly where 2.0.3's stream repairs
(ADR-0003) live.

The argument for proceeding: `rust-old/` would not help a failing consumer
anyway. It is a *source* oracle for parity questions, and a consumer integration
failure is an API-shape or lifecycle question, which the 2.0.6 examples answer
better than the Rust does.

**Recommendation**: not a hard gate, on the condition that 2.0.6's
`streaming.cyr` drives the full FFI lifecycle (create → start → fill to
completion → is_finished → destroy), which is the part consumer-green was
standing in for. **Decide this before starting 2.0.4** — it determines whether
this arc has an end date.

---

## 2.x — larger work

Changes the public surface, or is gated on something outside this repo.

### 2.1.0 — The allocation-failure contract

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
where 2.0.3's stream repairs (ADR-0003) would be exercised. Its relationship to
`rust-old/`'s retirement is the open question in 2.0.8.

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
- **svara's sample-rate threshold is a measurement, not a constant.** prani does
  not restate svara's ">1000 Hz" rule anywhere in source, deliberately
  (ADR-0001). `tests/hardening.tcyr` pins it with controls at 8000 and 44100 that
  fail if svara ever widens or narrows the range. **If those controls fail, the
  threshold moved — do not hardcode it in response.**
- **The Rust oracle's dependencies are someone else's to keep published.**
  `rust-old` builds against `svara 1.0.0` / `naad 1.0.0` / `hisab 1.2.0` from
  crates.io, checksums pinned in `Cargo.lock`. Verified resolvable 2026-08-31. If
  any is ever yanked, 2.0.7's comparison becomes impossible and 2.0.8's gate
  must drop it rather than wait.
