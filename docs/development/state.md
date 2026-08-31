# prani — Current State

> Refreshed every release. CLAUDE.md is preferences/process/procedures
> (durable); this file is **state** (volatile).

## Version

**2.0.3** — the P(-1) scaffold-hardening + security sweep, on top of the
completed port. 1.1.0 was the last Rust release; 2.0.0 was the language port;
2.0.1 restored logging + serde; 2.0.2 was the toolchain and dependency catch-up;
2.0.3 audits the whole tree and repairs what it found. **11 findings, 8 repaired,
3 accepted and documented** — report in
[`../audit/2026-08-30-audit.md`](../audit/2026-08-30-audit.md), three oracle
divergences in [`../adr/`](../adr/). Headline: `crvoice_vocalize` **segfaulted**
on any sample rate svara rejects (<= 1000 Hz), because svara 3.x turned an
infallible constructor into a checkable code and the port kept using the return
as a pointer. **770 assertions across 17 suites**, and `cyrius audit` **exits 0
for the first time**. Per-module parity ledger:
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

`cyrius audit` is green end to end as of 2.0.3: fmt · lint · docs · tests · bench,
**exit 0**. 17 suites / **770 assertions**.

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
(non-error + exact length + all-finite + bit-identical determinism).

## Consumers

- **kiran** (game engine), **joshua** (game manager) — once they port up the stack.

## In flight

Nothing. The port is complete, current on toolchain and dependencies, and
audited.

**Open work lives in [`roadmap.md`](roadmap.md)**, not here. It is open work
only, and **2.0.x is now one arc with one destination: retiring `rust-old/`**
(2.0.8). The gate is preserve-first, the same one svara and goonj used — nothing
the oracle still holds may be lost:

| | |
|---|---|
| **2.0.4** | Port parity re-verification — audit all **73** Rust `#[test]` blocks against the 17 Cyrius suites. The port was verified against the oracle's *source*, never against its *tests*. Every gap found becomes a new 2.0.x item. Also establishes the recovery rule (`git show <tag>:…`). |
| **2.0.5** | Input-range validation beyond parse success (ADR-0002's deferral + audit F10/F11). |
| **2.0.6** | Runnable examples — `CLAUDE.md` advertises `docs/examples/` and it holds only a `.gitkeep`. A gate item: after the oracle goes, an example is the only executable statement of how the API is driven. |
| **2.0.7** | Benchmark breadth + the Rust comparison. ⚠ **Must complete before 2.0.8** — it is the only item that must; it needs the oracle buildable. |
| **2.0.8** | Retire `rust-old/`. Gated on the above plus a **176-citation** reference sweep and a green tree with the directory moved aside. |

One decision is open and should be made before 2.0.4 starts: **whether
consumer-green is a hard gate on 2.0.8.** svara treated it as one; for prani it
would block the arc indefinitely on kiran/joshua. The roadmap carries the
argument both ways and a recommendation.

**2.x** carries what changes the public surface (the allocation-failure contract
from audit F9) or is blocked outside this repo (consumer-green). Four watch items
are recorded there too, with the condition that would schedule each.
