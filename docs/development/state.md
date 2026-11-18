# prani — Current State

> Refreshed every release. CLAUDE.md is preferences/process/procedures
> (durable); this file is **state** (volatile).

## Version

**2.3.1** — the deferred decisions. A pitch the sample rate cannot represent is
refused ([ADR-0011](../adr/0011-a-pitch-past-nyquist-is-refused.md)), and a steady
wind adds no Doppler shift
([ADR-0012](../adr/0012-a-steady-wind-does-not-shift-the-pitch.md)). CI and the
release workflow fetch cyrius's installer at the pinned tag. **2546
assertions / 22 suites, in each format.**

On top of **2.3.0** — two float formats
([ADR-0010](../adr/0010-two-float-formats.md)). A real is f64 by default and f32 when
the build defines `PRANI_F32`, from one bundle. The f64 build is 2.2.1's arithmetic,
bit for bit: the 716,010-line sweep is byte-identical to tag 2.2.1, JSON included.
The f32 build is held to accuracy against it. Its render levels are within a relative
6.4e-7, its single computations within an ulp or two, and an oscillator's time and
phase stay f64. Documents carry f64 bit patterns in both builds. The 2.2.1 work
underneath is unchanged ([audit](../audit/2026-11-17-audit.md),
[ADR-0009](../adr/0009-renders-do-not-depend-on-how-they-are-cut.md)). No dependency
moved since 2.1.2 (cyrius 6.7.18, svara **3.14.0**). Consumers still need
**cyrius 6.7.2 or later**, in either format. Audit exit 0, and both builds are
warning-free.

Standing facts from 2.0.14: `f64_to` saturates, so a count is ranged in f64 before the
cast, and past `VEC_CAP_MAX` (2^28 samples) a push exits the process, so every path
that builds one buffer refuses such a count first (`prani_buffer_count_ok`).
`stream_new` keeps 2^53 and carries anything longer.

## Toolchain

- **Cyrius pin**: `6.7.18` (in `cyrius.cyml [package].cyrius`).
- `lib/` is a clean resolve at that pin: 50 files, 46 byte-identical to the
  snapshot (sakshi 2.5.10, bayan **1.11.1** and ganita **1.2.15** among them) and
  svara/naad/hisab/goonj byte-identical to their tags' `dist/`. `cyrius deps --verify`
  passes all 50, and `cyrius deps --locked` confirms `lib/` and the lock are what the
  tags resolve. After a pin move, delete `cyrius.lock` and re-run `cyrius lib sync`
  then `cyrius deps`: deps keeps old commit rows beside the new ones. Then drop
  `lib/hashmap_fast.cyr`, which `lib sync` copies and nothing includes:
  `rm lib/hashmap_fast.cyr && cyrius deps --lock`, or `cyrius deps` names it on every
  build. `--full` copies the whole snapshot, which this project does not want.
- Build: `cyrius build src/main.cyr build/prani` (f64); add `-D PRANI_F32` for f32.
- **Two formats** (ADR-0010): every suite runs in each, `cyrius test` and
  `cyrius test -D PRANI_F32`. The f64 build compiles `src/f64/`, generated from `src/`
  by `sh scripts/gen-f64.sh`: regenerate it after any `src/` edit, then `distlib`.
- Test ONE suite: `cyrius test tests/<mod>.tcyr`; all of them: `cyrius test tests`.
  `cyrius tests` is gone since cyrius 6.7.12.
- Bundle: `cyrius distlib` → `dist/prani.cyr` (reads `[lib].modules`). There is no
  `dist/prani.deps`: distlib 6.6.18+ names only what a fold needs beyond its named
  deps, and for prani that is nothing. `src/main.cyr`, the benches and the examples
  include the bundle, so run `distlib` after a `src/` change or they see stale code.
- Gate: `cyrius audit` (fmt · lint · docs · tests · bench). Since cyrius 6.7.12 its
  fmt stage takes `cyrfmt --check`'s verdict, so the per-file check and the audit
  agree: a 4-space continuation step passes, as does the canonical 2.
- **Parallel-porting concurrency**: every `cyrius …` call re-resolves deps and
  races on `cyrius.lock`. Serialize all toolchain calls behind
  `flock <scratch>/prani-build.lock cyrius …`.

## Source

- Rust reference: 3,527 lines across 17 files, frozen at tag `2.0.3` and read
  with `git show 2.0.3:rust-old/src/<file>.rs`
  ([ADR-0004](../adr/0004-cite-the-oracle-by-tag.md)). `lib.rs`
  (organization/prelude) and `math.rs` (f32 transcendental wrappers, folded into
  `f64_*` builtins) carry no independent Cyrius module.
- Cyrius port: `src/main.cyr` (smoke) + 15 per-module `src/*.cyr` (+
  `logging.cyr`, which has no Rust counterpart), each validated by `tests/*.tcyr`.
  Since 2.3.0, `src/real.cyr` (the float format) comes first, and the 13 modules
  that do arithmetic each have a generated f64 build in `src/f64/`.

## Dependencies

Consumed as Cyrius distlib bundles (git+tag in `cyrius.cyml`):

- **svara** 3.14.0 — glottal source, formant filter, vocal tract (the excitation
  + resonance engine tract/voice bridge to). Its pulse peaks at
  `SV_G_ROSENBERG_GAIN`, which `src/tract.cyr` divides back out (ADR-0007).
- **naad** 2.3.5 — biquad filters (noise-only bandpass shaping).
- **hisab** 3.5.0 — `ease_in_out_smooth` (envelope curves) + transitive math.
- **goonj** 2.3.2 — referenced transitively by the svara/naad bundles.
- stdlib: syscalls, string, alloc, str, fmt, vec, io, args, assert, math, ganita,
  hashmap, bayan, tagged, fnptr, callback, bench, **sakshi**. sakshi (2.5.10, the
  snapshot's copy) is called by svara, goonj and `src/logging.cyr`. It is a leaf
  since 2.1.2; through 2.1.1 it was a `[deps.sakshi]` package.

This is the matched set for cyrius 6.7.18, and it moves together. svara 3.14.0, naad
2.3.5, hisab 3.5.0 and goonj 2.3.2 each pin 6.7.18 exactly, svara pins the other
three at these tags, and naad and goonj both pin hisab 3.5.0. As of 2.1.2 each is
also its project's newest tag.

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
each integrated and independently re-verified in the main tree against the
oracle at `2.0.3`.

## Tests

`cyrius audit` is green end to end as of 2.3.0: fmt · lint · docs · tests · bench,
**exit 0**. 22 suites / **2546 assertions, in each format** (770 at 2.0.3;
2.0.4 added 430 closing the parity shortfalls — see
[`rust-test-parity.md`](rust-test-parity.md)), plus the decoders' fuzz harness,
`tests/prani.fcyr` (2.2.1). Measured with `for t in tests/*.tcyr; do cyrius test $t;
done` and the same with `-D PRANI_F32`, summing each total. CI runs the audit and a
`dist/` bundle-coherence check on every push, as of 2.0.4 — plus
`scripts/run-examples.sh` (2.0.6), `scripts/bench-check.sh` (2.0.11), and since 2.3.0
the f32 build's suites, examples and fuzz harness, and `scripts/gen-f64.sh --check`.
Before 2.0.4 it ran only build + test, so fmt · lint · docs · bench were local-only.

`tests/real.tcyr` (2.3.0, **36** assertions) is the format layer's suite: the
primitives' NaN, signed-zero and saturation behaviour in both formats, integers in
the format, the f64 time base, and document portability (the same bytes from either
build). `tests/realt.cyr` holds the shared helpers: `assert_real` is exact in the f64
build and within a measured ulp budget in the f32 build.

`tests/hardening.tcyr` (added 2.0.3 at 53 assertions, **263** at 2.3.1) is the
P(-1) sweeps' regression suite — one group per repaired finding, each written so it FAILS on
the tree before it (the memory-safety groups crash the process there). Every group
carries controls so it cannot pass vacuously. It also holds the two allocation
budgets, measured with `alloc_used()` rather than asserted as wall-clock.

`tests/allocrefused.tcyr` (2.2.0, **173** assertions at 2.3.1) is the allocation contract's
proof. It redefines `prani_alloc_raw` to refuse each of prani's own requests in
every public path, singly and from then on, and requires exactly `PRANI_ERR_ALLOC`
back. It redefines the stdlib `alloc` too, to reach naad's reporting constructors
in measured windows. Eight mutants each fail it (CHANGELOG 2.2.0, *Verified*).
`scripts/check-alloc.sh` (CI) keeps every allocation where the suite can see it.

`tests/layers.tcyr` (2.2.1, 45 assertions) pins the RMS of 15 renders to a relative
1e-9: every synthesis layer past the tract. A level move is a failure there, and the
14 mutants in the 2.2.1 audit report each fail it. Re-record a level only with the
change that moves it, and name it in that release's notes.

One `tests/<module>.tcyr` per module, cross-checked against the Rust oracle
serde roundtrip tests INCLUDED (serde restored in 2.0.1 via `#derive(Serialize)`
+bayan, f64 fields as lossless i64 bit patterns); Display-string tests dropped.
svara-independent math asserted at exact f64 bit patterns; full synthesis paths
(through svara's DSP, where f32→f64 diverges) asserted structurally
(non-error + exact length + all-finite + bit-identical determinism) **and, since
2.0.4, by magnitude** — peak / energy against the oracle's own bars. Before that,
an all-zero buffer satisfied every synthesis assertion in the project.

## Consumers

- **kiran** (game engine), **joshua** (game manager) — once they port up the stack.

## In flight

Nothing. The port is complete, on the cyrius 6.7.18 matched dep set, audited
(2.0.3), re-verified against the oracle's tests (2.0.4), and range-guarded (2.0.5), exercised by runnable examples (2.0.6), measured against the oracle (2.0.7), no longer carrying it (2.0.8), with its two measured perf defects repaired (2.0.10), with a gate that would catch the next one (2.0.11), reporting a refused allocation (2.2.0), and swept a second time (2.2.1).

**Open work lives in [`roadmap.md`](roadmap.md)**, not here. It is open work
only, and **2.0.x was one arc with one destination: retiring `rust-old/`** — it
reached it in 2.0.8, and 2.0.9–2.0.11 then repaired what the arc's own
measurements found. The gate was preserve-first, the same one svara and goonj
used — nothing the oracle still held could be lost:

| | |
|---|---|
| ~~**2.0.4**~~ | ✅ **Done** — parity re-verification + all 41 shortfalls closed; recovery rule in [ADR-0004](../adr/0004-cite-the-oracle-by-tag.md). Ledger: [`rust-test-parity.md`](rust-test-parity.md). |
| ~~**2.0.5**~~ | ✅ **Done** — input-range validation (109 guards, [`input-ranges.md`](../architecture/input-ranges.md)), plus the svara 3.5.4 bump that fixed the (1000, 7500] abort. |
| ~~**2.0.6**~~ | ✅ **Done** — five examples, run by CI; `streaming.cyr` drives the full FFI lifecycle, satisfying the consumer-green condition. |
| ~~**2.0.7**~~ | ✅ **Done** — the Rust comparison, captured while the oracle still builds. **The arc's only hard ordering constraint.** |
| ~~**2.0.8**~~ | ✅ **Done** — `rust-old/` retired, after a **297-citation** reference sweep across 53 maintained files (the gate's own estimate of 83 across 42 was stale), tags pushed to `origin`, and a green tree with the directory moved aside. **The arc is complete.** |
| ~~**2.0.9 + 2.0.10**~~ | ✅ **Done** — stream allocation 8,800 → 512 B/call; `emotion_evaluate` 154 → 114 ns. |
| ~~**2.0.11**~~ | ✅ **Done** — allocation hard-gated (`allocbudget.tcyr`), timing recorded against a committed baseline. |

**Decided 2026-08-31: consumer-green is not a hard gate on 2.0.8**, on the
condition that 2.0.6's `streaming.cyr` drives the full FFI lifecycle. The arc now
has an end date rather than being blocked on kiran/joshua ports with no date.

**2.x** carries what changes the public surface (2.3.0's f32 lane; 2.2.0's
allocation-failure contract has shipped) or is blocked outside this repo
(consumer-green). Five watch items
are recorded there too, with the condition that would schedule each.
