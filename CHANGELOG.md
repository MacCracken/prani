# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

> **Reading `rust-old/…` citations in the entries below.** The Rust oracle this
> project was ported from lived at `rust-old/` until **2.0.8** removed it. Entries
> written before then cite it by bare path, and those paths are left as written —
> a released changelog is a record of what was true at the time, not a document to
> retrofit. To resolve any of them, name the tag
> ([ADR-0004](docs/adr/0004-cite-the-oracle-by-tag.md)):
>
> ```sh
> git show 2.0.3:rust-old/src/voice.rs   # port era: 2.0.0 and later
> git show 1.1.0:src/voice.rs            # Rust era: the Rust WAS src/
> ```
>
> The path changed at 2.0.0 because the port created `rust-old/` by moving the
> Rust aside, so the era decides which form resolves.

## [2.0.11] - The benchmark suite compares something now

`cyrius audit` has always run `tests/prani.bcyr` and reported
**`1 passed, 0 failed`** — because all the bench harness checks is that it *ran*.
It compared nothing. **Three regressions reached a release through that gap**,
and every one was caught by a human reading numbers:

- **2.0.5** shipped a 2.2× slowdown in `emotion_evaluate`, found two releases
  later by diffing a table in `docs/benchmarks.md`.
- **2.0.6** found `stream_fill_buffer` retaining 8,800 B/call — by someone
  *writing an example*.
- **2.0.10** found the harness measuring a **stale `dist/` bundle**: it read
  155 ns for a change that actually measured 114 ns.

Suite **1931 → 1946 assertions / 18 suites**, `cyrius audit` exit 0.

### The split: allocation is gated, timing is recorded

⭐ **This is the design decision, and it is deliberate asymmetry.**

`alloc_used()` is **deterministic** — same tree, same byte count, any host, any
load. So allocation is a **hard gate**: `tests/allocbudget.tcyr` (new, 15
assertions) budgets every benchmarked path, and `cyrius audit` runs it.

Wall-clock on a shared CI runner is **not** deterministic. A 20% swing on an
unchanged tree is ordinary, and **a gate that cries wolf gets disabled within a
month, which is worse than no gate.** So timing is *recorded*:
`scripts/bench-check.sh` compares a run against the committed
`benches/baseline.csv` and prints a delta per benchmark, exiting 0.
`BENCH_GATE=<factor>` opts into failing — to be turned on only once recorded
history shows the runner is quiet enough to earn it.

The cost is stated rather than hidden: **2.0.5's regression would NOT have failed
this gate.** What it buys is a gate nobody will be tempted to switch off. It does
fail hard on one timing condition — a baseline benchmark that stops running,
which is a real regression rather than noise.

### The budgets

| Path | Budget | Why |
|---|---|---|
| `dcblocker_process`, `prani_rng_next_f32` | **0 B** | per-sample; one byte here is 44 KB/s retained |
| `prani_ffi_voice_set_size` | **0 B** | an RTPC a host calls per frame (pins 2.0.3's F7) |
| `emotion_evaluate` | ≤ **40 B** | one `PrEmotionOut`, which the caller receives |
| `stream_fill_buffer` | ≤ **512 B** | the audio-callback path (was 8,800 at 2.0.6) |
| `crvoice_vocalize` | **sub-linear in duration** | allocation may scale with audio produced, never with calls |

Every budget carries a **control** — a `<= N` assertion passes trivially if the
path never runs. Verified adversarially: injecting one `vec_new()` into
`dcblocker_process` fails **two** budgets, the per-sample one directly and the
stream fill it cascades into.

The `crvoice_vocalize` budget is expressed as *the marginal cost of doubling the
duration*, not an absolute byte count. That cancels fixed setup and survives any
buffer-growth change, while still catching the 2.0.9 defect — per-block overhead
being retained.

### Fixed — the baseline was gitignored

Found on the last check before release: `.gitignore`'s `*.csv` would have
excluded `benches/baseline.csv`, so CI on a fresh checkout would have failed with
*"no baseline"* — a gate that never ran. Negated explicitly, with the reason.

## [2.0.10] - The two performance defects the arc's own measurements found

Closes roadmap **2.0.9** and **2.0.10** together — both were found by the 2.0.x
arc measuring itself, and both are fixed in one tree state. Suite **1900 → 1931
assertions / 17 suites**, `cyrius audit` exit 0. No API change; output
bit-identical on every path.

### Fixed — `stream_fill_buffer` retained 8,800 bytes per call → 512

Found by 2.0.6's `streaming.cyr`, the first thing to drive the streaming path the
way a host does. At 44100 Hz with 512-sample blocks that was **~757 KB/s retained
for the life of the process** on a path the module header advertises for *"audio
callbacks (Wwise, FMOD, Godot, JACK)"*.

**94% removed.** `stream.cyr`'s own share is now **0**: the contour vec, the
`SynthesisOptions`, and a per-fill `prani_intent_modifiers` — a **fourth** source
the original analysis had missed, allocating a whole struct to read one immutable
field — are hoisted into `SynthStream`. The big one, the 8,088-byte block vec,
needed `crtract_synthesize_into` alongside the allocating wrapper (the
`species_params_into` split, following 2.0.3's F7/F8 precedent). `voice.cyr`'s
block loop reuses one buffer too, saving **~400 KB per 1 s call**.

⚠ **Residual 512 B/fill is a behaviour question, not a perf one.** One
`svara_glottal_new` per block. svara exposes `svara_glottal_set_f`, so caching the
source looks like a free win — **it is not**: a fresh source per block resets its
phase and jitter/shimmer state, a cached one carries them forward, and the audio
changes. The oracle rebuilt per block too, so rebuilding is parity-correct.
Closing it needs a decision that the continuity change is wanted.

### Fixed — `emotion_evaluate` regressed 2.2×; 37 of the 82 ns recovered

2.0.5's guards validated the same two fields **four times** per call. Split into
public checked wrappers over internal unchecked cores; `evaluate` validates once.
**154 → 114 ns**, bit-identical across a 6,451-state sweep.

**The 69 ns baseline was not reached and is not claimed.** The remaining ~45 ns is
the entry-point validation 2.0.5 correctly added. `docs/benchmarks.md` is
re-baselined at 118 ns rather than carrying a target nobody intends to hit.

### Fixed — the benchmark harness was measuring stale code

⭐ Found while verifying the above, and the sharpest of the three.
`tests/prani.bcyr` benchmarks `dist/prani.cyr` — the generated bundle,
deliberately, since that is what a consumer gets. **So a stale bundle makes the
bench report on the last bundled code.** `cyrius audit` read **155 ns** for a
change that actually measured **114 ns**: the fix ran, the suite went green, and
the benchmark disagreed with reality by 36%.

CI's step order is corrected — bundle coherence now runs **before** the audit, so
staleness fails before anything measures it. The local hazard remains and is
documented: run `cyrius distlib` before believing a bench number after editing
`src/`.

### Filed — 2.0.11: the suite still has no baseline

All three of the above were caught by a human reading numbers. `cyrius audit`
reports `1 passed, 0 failed` for the bench because **all it checks is that the
harness ran** — it compares nothing. Filed with three options and a
recommendation (gate on deterministic `alloc_used()` first; treat timing gates as
having to earn their place against a noisy shared runner).

## [2.0.8] - `rust-old/` retired

**The 2.0.x arc's destination.** The frozen Rust oracle prani was ported from —
**26 files, 4,646 lines** — is gone from the working tree. Patch-level because it
changes nothing a consumer sees: no API, no behaviour, no bundle content. What it
changes is what prani's parity guarantee *rests on*. **From here, parity is
asserted by the suites alone.**

`cyrius audit` exit 0 (17 suites, **1900 assertions**), five examples clean, with
the directory absent.

### The gate, and what it caught

The arc was preserve-first: nothing the oracle held could be lost before it went.
Every item was checked, and two of them found problems:

- **2.0.4** — all 73 Rust tests audited against the suites; 41 shortfalls closed,
  zero behavioural defects. Suite 770 → 1200.
- **2.0.6** — five runnable examples, CI-gated; `streaming.cyr` drives the full
  FFI lifecycle, which is what consumer-green was standing in for.
- **2.0.7** — the Rust comparison, captured while the oracle still built. The
  arc's only hard ordering constraint.
- **Move-aside proof** — `mv rust-old ../`, then audit + examples + `distlib`, all
  green, **before** deleting. Nothing in the build, tests, or bundle read it.
- **Reference sweep** — every live citation converted to the tag-qualified form
  from [ADR-0004](docs/adr/0004-cite-the-oracle-by-tag.md).

### Fixed — the sweep's own search pattern was too narrow

⭐ The gate said *"83 citations across 42 files"*. The real figure was **297
across 53** — 2.0.4–2.0.7 added heavily-citing docs and the number was never
re-measured. That is the same drift that produced the wrong 176 figure 2.0.4
corrected, and the second time a gate number measured once has gone stale.

Worse, and only caught by the verification pass: **`git grep rust-old` was the
wrong search.** Citations like `voice.rs:227-231` name an oracle file with **no
`rust-old/` prefix at all**, so they were invisible to it and just as
unresolvable — 77 of them, including one in `src/voice.cyr`. Most turned out to
be module names in prose (`prani's rng.rs and svara's rng.rs are identical`),
which need nothing; the line-numbered ones were converted.

Also normalised: citations written `rust-old/…:904-915 @ 2.0.3` or
`2.0.3: rust-old/…` were tag-qualified but **not paste-able into `git show`** —
which, with the directory gone, is the entire point. All 33 are now the canonical
`2.0.3:rust-old/…`.

### How to read the oracle now

```sh
git show 2.0.3:rust-old/src/voice.rs           # port era (2.0.0+)
git show 1.1.0:src/voice.rs                    # Rust era — the Rust WAS src/
git grep 'fn vocalize' 2.0.3 -- rust-old/src   # grep the whole oracle
```

The path changed at 2.0.0 because the port created `rust-old/` by moving the Rust
aside, so the era decides which form resolves. `CHANGELOG.md` entries written
before this release cite bare paths and are **left as written** — a released
changelog records what was true at the time; a note at the head of this file
carries the incantation instead.

### ⚠ Follow-up — push the tags

The recovery rule names tags, and they could not be verified against `origin`
from the dev environment. **Run `git push --tags`.** The risk is bounded rather
than open: all three oracle-bearing tags are **ancestors of `main`**, and
`origin/main` matched local `main`, so the oracle's *content* is already on
origin — only the readable tag refs may be missing, and pushing them later works
against commits origin already has. Until then, citations resolve by hash only.

## [2.0.7] - The Rust comparison, and two published figures that were wrong

The measurement that had to happen before `rust-old/` could be retired: after
2.0.8 it can only be run from a tag, against three crates.io releases staying
published. Benchmarks **4 → 18** (14 mirroring the oracle's criterion suite
one-for-one), `cyrius audit` exit 0, 1900 assertions. `rust-old/` unmodified —
built with `CARGO_TARGET_DIR` outside the repo.

### The comparison

Both harnesses on one host, one sitting, 2026-08-31. AMD Ryzen 7 5800H, single
core. Same species, same durations, same 44100 Hz.

| | Rust oracle (f32) | Cyrius port (f64) | |
|---|---:|---:|---:|
| `wolf_howl_1s` | 1.39 ms | 21.9 ms | **15.8× slower** |
| median, 13 synthesis benchmarks | | | **15.7×** |
| range | | | 9.9× – 17.4× |

The band is *tight* — every apparatus, every duration. A uniform ratio across
unrelated code paths is the signature of something systemic in the substrate,
not one slow algorithm.

⚠ **This measures two stacks as shipped, not one algorithm in two languages.**
Three things differ and only one is float width: the oracle is prani 1.1.0 on
svara 1.0.0 / naad 1.0.0 compiled by LLVM `--release`; the port is 2.0.6 on
svara **3.5.4** / naad **2.2.2** compiled by cycc. svara 3.x added work 1.0.0
never did. **It is not evidence that Cyrius is 15× slower than Rust**, and the
honest prior is that codegen dominates and float width is second.

### Fixed — two figures this project has published since the port began

| Claim | Measured |
|---|---|
| oracle does *~1000× realtime* | **719×** |
| port does *~236× realtime* | **45.6×** |

⭐ The port's 236× was an **artifact of the 8 kHz benchmark**. Per-sample cost
barely moves with rate — 533 ns/sample at 0.05 s @ 8 kHz against 497 ns/sample at
1.0 s @ 44100 Hz — so a benchmark at one eighth the rate renders one eighth the
samples and looks eight times more real-time than the library is. Neither figure
had ever been checked. `docs/benchmarks.md` now quotes **ns/sample**, or realtime
with its rate attached.

### Added — every vocal apparatus is now benchmarked

Before this, only the laryngeal path was, so **a regression in syringeal,
stridulatory, vibratile or noise-only synthesis was invisible**. The spread is
real: noise-only costs 124 ns/sample against laryngeal's 497, having no glottal
source and no formant bank.

### Found — `emotion_evaluate` regressed 2.2×, and 2.0.5 did it

⭐ The only historical-series row that moved: **69 ns → 151 ns**, on a path this
file calls a *"per-frame game-AI call"*. Caused by 2.0.5's own input-range
guards. `evaluate` calls `select_vocalization` and `select_intent`, and **each
calls both zone functions**, so the same two fields are range-checked **four
times** per call, plus the smoothing check — five validations where one would do.

The lesson is not "fewer guards". It is that 2.0.5 added 109 of them and ran the
benchmark suite green, because **the suite never compared against a baseline** —
it only checked the harness ran. A guard on a per-frame path needs a benchmark
delta attached before it ships. Filed as roadmap **2.0.10** with the fix
(validate once at the entry point; split public checked wrappers from internal
unchecked paths, as 2.0.3 did for `species_params`).

### Note — the f32 constraint this port was built on is gone

`port-audit.md` has said since 2.0.0 that *"f32 → f64 everywhere … widening is
**forced**"*. **ganita 1.1.4 ships a 23-function f32 scalar tier and is already
vendored in `lib/`**, and prani calls nothing outside it — no `tanh`, `sinh`,
`cosh`, `asin` or `acos` anywhere in `src/`. Not forced; a choice nobody had
revisited.

prani still cannot convert alone: `svara_glottal_next_sample` and
`svara_tract_process_sample` are f64 on both sides of every per-sample call.
Filed as **P0 on naad and svara** (naad first — it is the bottom of the stack)
and as roadmap **2.1.0 Lane A** here. Both P0s carry the caveat above: measure
one hot path before converting a module, because a measured "no" closes the
question for everyone downstream.

## [2.0.6] - Runnable examples, and the two defects they found

Five worked programs in [`docs/examples/`](docs/examples/), built and run by CI on
every push via [`scripts/run-examples.sh`](scripts/run-examples.sh). Suite
**1894 → 1900 assertions / 17 suites**, `cyrius audit` exit 0.

`CLAUDE.md` had advertised *"[`docs/examples/`](docs/examples/) — Runnable
examples"* over a directory containing a `.gitkeep`. More to the point:
`rust-old/` is removed in 2.0.8, and after that **a worked example is the only
executable statement of how this API is driven.**

Each example includes **`dist/prani.cyr`** — the published bundle — rather than
`src/*.cyr`, so building them exercises prani exactly as a consumer does. That
makes the examples the closest thing the project has to a consumer integration
test, which matters because neither consumer has ported up the stack.

| Example | Shows |
|---|---|
| `basic.cyr` | a wolf howl end to end, and the vec-or-negative-code convention |
| `species_tour.cyr` | one call per vocal apparatus, with parameters read back at runtime |
| `error_handling.cyr` | every failure shape, and the ADR divergences behind them |
| `streaming.cyr` | the real-time path, the allocation rule, **and the full FFI lifecycle** |
| `sequencing.cyr` | bouts, phrases, and a multi-voice chorus |

### Note — the runner is POSIX `sh`, deliberately

`scripts/run-examples.sh` is written for POSIX `sh`, not bash. CI invokes it as
`sh scripts/run-examples.sh`, which is **dash** on `ubuntu-latest`, where
`set -o pipefail` is an *error* rather than a no-op — the first version used it
(plus arrays and `shopt`) and failed CI at line 11 before running anything. It is
now free of bashisms and its shebang says `#!/bin/sh` so both invocations agree.
The trap is that `/bin/sh` on a typical dev box is a symlink to bash, so the
script passes locally and fails only in CI; there is a comment in the file saying
so. Both failure paths are verified: a build failure and a non-zero exit each
fail the gate, and the runner reports every failing example rather than stopping
at the first.

### The FFI surface has now been driven end to end

`streaming.cyr` runs `prani_ffi_voice_create` → `_stream_start` → `_stream_fill`
to completion → `_stream_is_finished` → both destroys, checking every return.
**That lifecycle is the condition the 2026-08-31 decision to drop consumer-green
from 2.0.8's gate rests on**, and nothing had ever driven it before.

It also proves the FFI is a thin adapter rather than a second implementation:
the FFI drain and the Cyrius-level drain produce **sample-for-sample identical
audio** — 0 of 2205 samples differ — for the same parameters.

### Fixed — an out-of-range species tag rendered audio and reported success

⭐ **Found by `error_handling.cyr`**, which is exactly the job a worked example is
supposed to do. 2.0.5 guarded the `voc` and `intent` tags for precisely this
reason — the oracle's enums made an out-of-range value unrepresentable, and the
port carries them as `i64` — and **missed `species`**. `species_params` falls
through to the FANTASY defaults for any unknown tag (deliberately, to preserve
totality), so:

```
crvoice_new(99)                              -> a valid voice, no error
crvoice_vocalize(that, HOWL, 44100.0, 0.05)  -> 2205 samples, no error
stream_new(that, HOWL, IDLE, 44100.0, 0.05)  -> a valid stream, no error
```

`crvoice_new` returns a pointer with no error channel, so the guard is at the
**point of use** — `crvoice_vocalize_with_intent` and `stream_new` — exactly as
[ADR-0006](docs/adr/0006-reject-non-finite-numeric-input.md) describes for the
builders. Pinned as **F13** in `tests/hardening.tcyr`, with controls at
`PRANI_SP_FANTASY` (the last valid tag) so a guard that rejected everything
could not pass.

### Filed — `stream_fill_buffer` allocates 8,800 bytes per call

Found by `streaming.cyr`. At 44100 Hz with 512-sample blocks that is 86
callbacks a second — **~757 KB/s retained for the life of the process**, on an
allocator that never frees; a one-minute creature loop retains ~45 MB. It
contradicts what `src/stream.cyr`'s own header advertises the module for:
*"suitable for real-time audio callbacks … (Wwise, FMOD, Godot, JACK)."*

Three per-call sources, all the **same pattern 2.0.3 already fixed twice** (F7,
F8). Filed as roadmap **2.0.9** rather than fixed here, to keep an examples
release from carrying a `stream.cyr` rework. It does not gate `rust-old/`'s
retirement — the oracle allocated per call too and simply had an allocator that
freed.

### Also noted

The examples surfaced two ergonomic gaps, both recorded rather than fixed
(adding public API is a minor bump, and 2.1.0 is where that belongs): there are
**no name helpers** for the species / vocalization / intent tags to match
`prani_err_name`, so every example and host log line hand-writes the same string
tables; and the **FFI has no `total_samples` getter**, so a host cannot learn the
length it should expect, and cannot account for an intent's `duration_scale`.

## [2.0.5] - Input-range validation, and a crash in the whole low-sample-rate band

Closes [ADR-0002](docs/adr/0002-deserializers-report-parse-failure.md)'s deferral
and audit findings **F10** and **F11**. Suite **1219 → 1894 assertions / 17
suites**, `cyrius audit` exit 0. **109 guards across 11 modules**, with the
accepted range of every numeric parameter written down in
[`docs/architecture/input-ranges.md`](docs/architecture/input-ranges.md) —
something the library never had.

### Fixed — CRITICAL: every sample rate in (1000, 7500] aborted the process

⭐ **The headline, and it was not prani's bug.** `crvoice_vocalize(voice, HOWL,
4000.0, …)` — an ordinary low sample rate, reachable from
`prani_ffi_stream_start` where a host passes the rate straight in — **killed the
process**. Bisected: 1000 rejected cleanly, **1001–7500 aborted**, 7501+ fine.

`crtract_new` was doing everything right. It checked `svara_tract_new`'s return
exactly as [ADR-0001](docs/adr/0001-check-svara-tract-constructor.md) requires,
and the return was a **valid pointer** — the tract simply could not survive its
own next call, for two independent reasons inside svara:

1. Every svara vowel target carries F5 = 3750 Hz, so at or below 7500 Hz svara
   fell back to a **single** 500 Hz formant and stored that one-element vec as
   the tract's record — then wrote five formants into it.
2. `svara_tract_new` stored two **unchecked naad error codes as filter
   pointers**; its fixed 600 Hz subglottal bandpass needs a rate above 1200 Hz.

Both are repaired in **svara 3.5.4**, found from here and fixed there rather than
worked around: a prani-side floor would have left the defect live for every other
svara consumer *and* rejected rates that actually work. prani bumps its pin
3.5.3 → 3.5.4 and adds an **F12** group to `tests/hardening.tcyr` that aborts on
the old tree.

**The working range widened.** 1201–7500 Hz never worked and now renders, via the
warn-then-continue path — which is the oracle's own semantics, so parity improves.

This is ADR-0001's lesson one level deeper. That ADR made the *constructor's*
return checkable; here a **later** call aborted instead of returning a code, and
no amount of checking at the constructor could see it coming.

### Fixed — HIGH: NaN was rendered as audio and returned as success

`f64_clamp(NaN, lo, hi)` is NaN — both comparisons are false — so a NaN passed
every clamp in the tree. The audit filed this as *"a silent empty buffer where an
error code belongs"*. Measured, the worst case is considerably worse:

> `crvoice_vocalize` on a **syringeal, stridulatory or vibratile** species
> returned a **full-length, all-NaN buffer as a success**. A host plays that.

It hid because the species you reach for first is the one that reports: on a
**laryngeal** species svara refuses the NaN f0 and it surfaces as an error.

Two of the audit's own notes were wrong and are corrected, both now pinned as
assertions in `tests/error.tcyr`: **infinities *are* bounded by `f64_clamp`** (so
the clamp-site hazard is NaN specifically), and **all three** non-finite values
convert to i64::MIN — not just NaN, and not saturating high.

### Fixed — F10: the chorus allocation, understated by eight orders of magnitude

The audit filed `base_samples + max_offset_samples * 2` as an i64 overflow at a
`timing_spread` around 1e14 seconds. Measured, a spread of **1e6 seconds** —
11.6 days, eight orders of magnitude below that — already demands over **700 GB**.
The reachable failure was arena exhaustion; a guard against the overflow alone
would have missed nearly the whole range. Separately, a NaN spread gave i64::MIN
and `i64::MIN * 2` **wraps to 0**, so the chorus quietly returned a
correct-length buffer — a silent *success*, not the silent-empty on file.

### Fixed — three more live crashes

- `crvoice_apply_nasal_antiformant`: `nasal_len = f64_to(len * nasal_fraction)`
  with no `min(len)` clamp — a fraction above 1.0 indexed past the buffer and
  terminated the process.
- `preset_from_json_str`: `str_data(bayan_json_v_str(…))` on a missing or
  non-string `name` dereferenced address 0.
- Five functions documented *"never fails"* became fallible, and **every in-tree
  call site was audited**: `crtract_synthesize_purr`'s two callers used the
  result as a pointer (`vec_len` on a negative code), and four `crvoice_apply_*`
  post-processing steps **discarded** their returns, silently leaving a buffer
  un-post-processed and returning it as success. All now propagate — their
  arguments cannot fail today, but "the caller already proved it" is exactly the
  assumption ADR-0001 was written about.

### Fixed — ADR-0002's deferral: deserializers reject nonsense that parses

All four hand-written codecs now range-check every field. A document with
`size_scale` 0 (it divides in `effective_f0`), a species tag out of range, or
`f0_min > f0_max` is rejected instead of producing a struct. prani's own
`to_json` output always satisfies the ranges — asserted as a control, because a
guard that rejected prani's own documents would be a worse bug than the one it
fixed.

### Filed upstream — `#derive(Serialize)`'s generated deserializer

Five of prani's codecs are **generated**, and the generated
`<Name>_from_json_str` opens with `load8(json + _p)` and no null guard, so
`X_from_json_str(0)` SIGSEGVs in any Cyrius project, and malformed input returns
the `memset`-zeroed struct as success — ADR-0002's exact defect, in codecs
ADR-0002 could not reach because they are emitted rather than written. Filed as a
cycc issue with a reproducer; not fixable from prani.

Found alongside it: **`cyrius test` reports `1 passed, 0 failed` for a `.tcyr`
whose process dies of SIGSEGV**, which is why these survived both the 2.0.3 sweep
and the 2.0.4 parity audit.

### Deferred — the builders, to 2.1.0

The 7 `crvoice_with_*` builders return `self` with no error channel, so rejecting
a NaN there is an API break. Roadmap 2.1.0 must already put a fallible return on
every constructor for the allocation contract, so the signature change happens
once, there. Until then a NaN smuggled in through a builder is caught at the
**point of use**. Written down rather than left to be rediscovered.

## [2.0.4] - Parity re-verification: the oracle's tests, not just its source

The port was verified module-by-module against `rust-old/`'s **source**. It had
never been verified against its **tests** — a different question, and the one
that decides whether 2.0.8 can safely remove the directory, because after that
parity is asserted by prani's own suites alone. This release asks it, closes what
it found, and writes down how to read the oracle once it is gone. Suite
**770 → 1200 assertions / 17 suites**; `cyrius audit` exits 0. **No file under
`src/` changed.**

### The result: zero behavioural defects

All **73** Rust `#[test]` blocks audited against the 17 Cyrius suites, one row
each, in [`docs/development/rust-test-parity.md`](docs/development/rust-test-parity.md).

| | |
|---|--:|
| ✅ Covered | 30 |
| 🟡 Partial | 30 |
| 🔴 Gap | 11 |
| ⬜ N/A (Rust-only language properties) | 2 |

The roadmap said a gap that turns out to be a real behavioural difference is a
defect and outranks everything else in the arc. **None was found.** Every
shortfall was a test gap: the port does the right thing, nothing asserted it.
Where reading both implementations was not conclusive, the verifying pass built a
throwaway probe and *measured* the port at the oracle's exact arguments.

### Fixed — the suite could not tell audio from silence

⭐ **The headline finding.** Every synthesis assertion in all 17 suites was
*non-error + exact length + all-finite*. **An all-zero buffer satisfied all
three.** No suite computed a peak, an RMS or an energy over a `crvoice_vocalize`
result — `f64_abs` appeared only inside tolerance comparisons on scalar helpers.
The oracle checks `max_amp > 0.001`, `near_energy > 10 * far_energy`,
Alarm-louder-than-Idle and shout-louder-than-whisper; none of it survived the
port. `t_max_abs` / `t_energy` helpers now exist and the oracle's own bars are
asserted.

This is the same shape as 2.0.3's lesson one level up: there, *a contract derived
from a dependency is a measurement*. Here — **an assertion that cannot fail is
not a test**, and 770 of them had been counted as if they were.

### Fixed — 6 of 13 species and 6 vocalizations never synthesized

Dog, Crow, Raptor, Crocodilian, Bee and Fantasy never reached `crvoice_vocalize`;
several never constructed a `CreatureTract` at all. SCREECH, RUMBLE, GROWL, BARK
and CHIRP were never synthesized, and the Cat+HOWL pair that drives the nasal
anti-formant and the cat formant-transition table was never built. All 13 species
now synthesize under the oracle's own selection ladder, asserted 13/13 on four
counters plus support-matrix cells so a wrong matrix cannot silently reroute a
species and still count as a success.

### Fixed — four serde roundtrips that could not fail, and eleven prefix compares

bayan's value accessors are null-safe by design (`bayan_json_v_int(0)` returns
0), so **a roundtrip asserted on a discriminant-0 value passes even if the derive
dropped the key entirely.** `CallBout` was round-tripped only with `HOWL` (= 0);
`VocalApparatus` only as Laryngeal (= 0), 1 of 5 variants. The oracle uses `Bark`
and `Dragon`. Now so does prani.

Chasing that, the systemic version: **all 11 idempotency checks were prefix
compares** — `memeq(json, json2, strlen(json))` — so a re-serialization that
*appended* a field passed. Each now asserts length equality first. This is the
same null-safe composition behind 2.0.3's HIGH finding
([ADR-0002](docs/adr/0002-deserializers-report-parse-failure.md)), surfacing in
the tests this time instead of the source.

### Fixed — edge inputs the oracle probed and the suites did not

Zero duration was never passed to any synthesis entry point. No stream was ever
drained in more than two `fill_buffer` calls, leaving the `t > 0.85`
release-boundary arm **dead in both `voice.cyr` and `stream.cyr`**. `fatigue` was
never read after more than one `record_call`, so an implementation that
**assigned instead of accumulated** would have passed. `emotion_update` was never
called on a non-default smoothing, so one that ignored the field would have
passed. `crvoice_with_jitter` and `crvoice_with_shimmer` had **zero test
callers**. No preset was ever synthesized; only 3 of 13 bout templates were even
constructed; every chorus mixed two bit-identical Wolves.

### Added — [ADR-0005](docs/adr/0005-serialized-tract-rebuilds-dsp-state.md): a serialized `CreatureTract` rebuilds svara's state

The one genuine oracle divergence in the tree that no ADR covered — it lived only
in source comments. The Rust derived `Serialize` over the whole struct, svara's
`VocalTract` included, and resumed seamlessly. svara hands prani an **opaque
handle with no state accessors**, so the port serializes eight fields and
*rebuilds* the svara tract and naad biquad from `params`. A tract serialized
mid-call resumes with **cleared filter memory**. `tests/tract.tcyr` now pins this
in both directions: the deserialized tract is bit-identical to a fresh one
carrying the restored scalars (proving rebuild), *and* differs from the
original's own continuation (proving the first assertion is not vacuous). Same
root cause as [ADR-0001](docs/adr/0001-check-svara-tract-constructor.md).

### Added — [ADR-0004](docs/adr/0004-cite-the-oracle-by-tag.md): cite the oracle by tag

2.0.8 removes `rust-old/` from the working tree, and a bare
`rust-old/src/x.rs:NN` in a comment stops resolving that day. The rule, in force
now: **a citation names a tag.** The path changed at 2.0.0 — the port *created*
`rust-old/` — so the recovery incantation differs by era, and both halves are
verified:

```sh
git show 2.0.3:rust-old/src/voice.rs   # port era (2.0.0+)
git show 1.1.0:src/voice.rs            # Rust era — the Rust WAS src/
```

### Fixed — two release-gate numbers were wrong

Both were measurements 2.0.8 depends on. The `rust-old/` citation count read
**176 across 45 files**; it had counted the vendored `lib/` bundles prani must
not modify (143 citations belonging to naad, goonj, svara and hisab) and
under-counted by exactly the 56 in `lib/naad.cyr`. The real figure is **83 across
42** maintained files. The Rust benchmark count read **10**; `criterion_group!`
registers **14**. Every number in the roadmap's facts table now carries the
command that reproduces it.

A third, found while correcting the second: `crvoice_vocalize
wolf_howl_0.05s@8k` **is not comparable** to the oracle's `wolf_howl_1s` — 400
samples against 44,100, at a sample rate that changes which svara path runs. 2.0.7
must add a matching row or drop wolf howl from the comparison.

### Fixed — CI ran three gates of five, and the contributor doc was still Rust

CI ran `deps` → `build` → `test`, so **fmt, lint, docs and bench were enforced
nowhere but contributors' machines**. It now runs `cyrius audit`, plus a **bundle
coherence** gate (`cyrius distlib` then `git diff --exit-code -- dist/`) — `dist/`
is a tracked build product that consumers build against, and it is deterministic,
so any diff is a stale commit.

`CONTRIBUTING.md` still told contributors to run `cargo fmt`, `cargo clippy` and
`cargo audit` and to install Rust 1.89+. It is now a Cyrius document.
`scripts/bench-history.sh` ran `cargo bench` into `benches/history.csv`, a path
that does not exist in this tree; it now drives `cyrius bench`, records the
measured timer floor alongside each figure, and refuses to write on a parse of
zero benchmarks.

### Decided — consumer-green does not gate `rust-old/`'s retirement

svara treated it as a hard gate. For prani it would block 2.0.8 indefinitely on
kiran/joshua ports that have no date. **Not a hard gate**, on one condition, now
a hard requirement on 2.0.6: `streaming.cyr` must drive the full FFI lifecycle
(create → start → fill to completion → is_finished → destroy). That lifecycle is
what consumer-green was standing in for, and an example runs in CI on every push,
which a consumer port does not.

### Note — one ledger row was wrong, and closing it caught that

The audit proposed asserting the cricket inter-chirp silence gap through a 0.2 s
`crvoice_vocalize` call. **That is impossible in the port and the oracle alike**:
`pos_in_chirp` is the index *within one synthesize call*, and vocalize renders in
882-sample blocks, so it never reaches `chirp_active` = 5880. The oracle's own
test passes on the per-syllable envelope, not the gap its comment names. The port
now carries the oracle's bar verbatim, and the silence arm is asserted where it
is reachable — a direct 12000-sample `crtract_synthesize_stridulatory` call, with
controls. A faithful port of an oracle quirk, and a reminder that a suggested fix
is a hypothesis until the code runs.

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

### Changed — the roadmap is open work only, in two arcs

A sweep for deferment language across the tree (source, tests, docs, CI, manifest
— `lib/` and `rust-old/` excluded) found **no TODO, FIXME or deferral marker in
any source file**; every open item lives in prose. Each was checked against the
tree rather than taken at its word, and three had gone stale:

- `port-audit.md` marked **sequence** 🟡 partial with *"synthesize parity test
  lands with voice/stream"*. It landed — `tests/sequence.tcyr` covers
  `CallBout::synthesize`, its error propagation, `CallPhrase::synthesize` and
  `synthesize_chorus`. Now ✅ 33, and **no partial or pending rows remain**.
- `port-audit.md`'s tract row still cited `filter_biquad_new(FILTER_BANDPASS,…)`,
  renamed by naad 2.2.0 and absorbed in 2.0.2.
- `port-audit.md`'s *"Deferred / follow-up work"* section listed two items that
  were both completed at port close-out (the distlib bundle, the 1.1.0 → 2.0.0
  bump).

One gap surfaced that nothing had recorded: **`CLAUDE.md` advertises
`docs/examples/` as "Runnable examples" and the directory holds a `.gitkeep` and
nothing else** — the same gap svara closed before its own oracle retirement.

[`roadmap.md`](docs/development/roadmap.md) is rewritten to carry **open work
only** — completed milestones are not restated there, they are in this file — and
**2.0.x is now one arc with one destination: retiring `rust-old/`** at 2.0.8,
under the preserve-first gate svara and goonj both used. **2.0.4 is a port parity
re-verification**, because the port was verified module-by-module against the
oracle's *source* and has never been checked against its *tests* — two different
questions, and svara found five real gaps when it asked the second one. Every gap
2.0.4 finds becomes a new 2.0.x item. The previously-listed items move back
behind it: input-range validation (2.0.5), runnable examples (2.0.6), benchmark
breadth and the Rust comparison (2.0.7). **2.x** keeps what changes the public
surface (the allocation-failure contract from audit F9) or is blocked outside
this repo (consumer-green), and four **watch items** carry the condition that
would schedule each.

The arc rests on facts measured rather than assumed: **73** Rust `#[test]` blocks
(72 of them in one `tests/integration.rs`) against 770 Cyrius assertions; **10**
Rust benchmarks against 4, and the Rust ten happen to cover every vocal apparatus
the Cyrius four miss; **176** `rust-old/…` citations across 45 tracked files that
become unresolvable on removal; and a recovery path that **splits by era** —
`git show 2.0.3:rust-old/src/voice.rs` works, but 1.1.0 and earlier predate the
port and carry the Rust at `src/` instead, so `git show 1.1.0:src/voice.rs` is
the form there. Both were run. `rust-old` still builds (`cargo fetch --locked`
resolves `svara 1.0.0` / `naad 1.0.0` / `hisab 1.2.0` from crates.io), which is
what makes 2.0.7's comparison possible — and is a window rather than a guarantee,
so 2.0.7 is the one item that must land before 2.0.8.

Every entry cites its source in the tree, and `state.md` now points at the
roadmap instead of duplicating it.

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
