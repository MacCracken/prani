# prani — Current State

> Refreshed every release. CLAUDE.md is preferences/process/procedures
> (durable); this file is **state** (volatile).

## Version

**2.0.10** — the two performance defects the 2.0.x arc's own measurements found,
closing roadmap 2.0.9 and 2.0.10 together. **`stream_fill_buffer` retained 8,800 B
per call → 512** (94%): `stream.cyr`'s share is 0, via `crtract_synthesize_into`
alongside the allocating wrapper, and `voice.cyr`'s block loop reuses one buffer
(~400 KB per 1 s call). The residual 512 B is one `svara_glottal_new` per block
and is a **behaviour question** — caching the source carries its phase forward and
changes the audio. **`emotion_evaluate` 154 → 114 ns**, recovering 37 of the 82 ns
2.0.5's guards cost; the 69 ns baseline is not reached and not claimed.

⭐ Found while verifying: **the benchmark harness was measuring stale code.**
`tests/prani.bcyr` benchmarks `dist/prani.cyr`, so a stale bundle reports the last
bundled code — `cyrius audit` read 155 ns for a change that measured 114 ns. CI now
runs bundle coherence **before** the audit. **1931 assertions / 17 suites**, exit 0.

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

- Rust reference: 3,527 lines across 17 files, frozen at tag `2.0.3` and read
  with `git show 2.0.3:rust-old/src/<file>.rs`
  ([ADR-0004](../adr/0004-cite-the-oracle-by-tag.md)). `lib.rs`
  (organization/prelude) and `math.rs` (f32 transcendental wrappers, folded into
  `f64_*` builtins) carry no independent Cyrius module.
- Cyrius port: `src/main.cyr` (smoke) + 15 per-module `src/*.cyr`, each validated
  by `tests/*.tcyr`.

## Dependencies

Consumed as Cyrius distlib bundles (git+tag in `cyrius.cyml`):

- **svara** 3.5.4 — glottal source, formant filter, vocal tract (the excitation
  + resonance engine tract/voice bridge to). **Bumped from 3.5.3 in prani 2.0.5**
  for two memory-safety repairs prani found: every sample rate in (1000, 7500]
  aborted the process on the first synthesis call.
- **naad** 2.2.2 — biquad filters (noise-only bandpass shaping).
- **hisab** 2.11.2 — `ease_in_out_smooth` (envelope curves) + transitive math.
- **goonj** 2.0.4, **sakshi** 2.4.12 — goonj referenced transitively by the
  svara/naad bundles; sakshi both transitive and called directly by
  `src/logging.cyr`.
- stdlib: syscalls, string, alloc, str, fmt, vec, io, args, assert, math, ganita,
  hashmap, bayan, tagged, fnptr, callback, bench.

svara 3.5.4 pins hisab 2.11.2 / naad 2.2.1 / goonj 2.0.4, and hisab 2.11.2 pins
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
each integrated and independently re-verified in the main tree against the
oracle at `2.0.3`.

## Tests

`cyrius audit` is green end to end as of 2.0.10: fmt · lint · docs · tests · bench,
**exit 0**. 17 suites / **1900 assertions** (770 at 2.0.3; 2.0.4 added 430 closing
the parity shortfalls — see [`rust-test-parity.md`](rust-test-parity.md)). CI runs
the audit and a `dist/` bundle-coherence check on every push, as of 2.0.4; before
that it ran only build + test, so fmt · lint · docs · bench were local-only.

`tests/hardening.tcyr` (53 assertions, added 2.0.3) is the P(-1) sweep's
regression suite — one group per repaired finding, each written so it FAILS on
the 2.0.2 tree (the memory-safety groups crash the process there). Every group
carries controls so it cannot pass vacuously. It also holds the two allocation
budgets, measured with `alloc_used()` rather than asserted as wall-clock.

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

Nothing. The port is complete, current on toolchain and dependencies, audited
(2.0.3), re-verified against the oracle's tests (2.0.4), and range-guarded (2.0.5), exercised by runnable examples (2.0.6), measured against the oracle (2.0.7), no longer carrying it (2.0.8), and with its two measured perf defects repaired (2.0.10).

**Open work lives in [`roadmap.md`](roadmap.md)**, not here. It is open work
only, and **2.0.x is one arc with one destination: retiring `rust-old/`**
(2.0.8). The gate is preserve-first, the same one svara and goonj used — nothing
the oracle still holds may be lost:

| | |
|---|---|
| ~~**2.0.4**~~ | ✅ **Done** — parity re-verification + all 41 shortfalls closed; recovery rule in [ADR-0004](../adr/0004-cite-the-oracle-by-tag.md). Ledger: [`rust-test-parity.md`](rust-test-parity.md). |
| ~~**2.0.5**~~ | ✅ **Done** — input-range validation (109 guards, [`input-ranges.md`](../architecture/input-ranges.md)), plus the svara 3.5.4 bump that fixed the (1000, 7500] abort. |
| ~~**2.0.6**~~ | ✅ **Done** — five examples, run by CI; `streaming.cyr` drives the full FFI lifecycle, satisfying the consumer-green condition. |
| ~~**2.0.7**~~ | ✅ **Done** — the Rust comparison, captured while the oracle still builds. **The arc's only hard ordering constraint.** |
| ~~**2.0.8**~~ | ✅ **Done** — `rust-old/` retired. **The arc is complete.** |
| ~~**2.0.9 + 2.0.10**~~ | ✅ **Done** — stream allocation 8,800 → 512 B/call; `emotion_evaluate` 154 → 114 ns. |
| **2.0.11** | The bench suite has no baseline, so regressions ship silently — and it measures `dist/`, so a stale bundle reports old code. Three options filed. |
| **2.0.8** | Retire `rust-old/`. Gated on the above plus a **297-citation** reference sweep across 53 maintained files, tags pushed to `origin`, and a green tree with the directory moved aside. |

**Decided 2026-08-31: consumer-green is not a hard gate on 2.0.8**, on the
condition that 2.0.6's `streaming.cyr` drives the full FFI lifecycle. The arc now
has an end date rather than being blocked on kiran/joshua ports with no date.

**2.x** carries what changes the public surface (the allocation-failure contract
from audit F9) or is blocked outside this repo (consumer-green). Four watch items
are recorded there too, with the condition that would schedule each.
