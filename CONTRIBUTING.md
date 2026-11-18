# Contributing to prani

Thank you for your interest in contributing to prani.

## Getting Started

1. Fork the repository
2. Clone your fork
3. Create a feature branch: `git checkout -b feature/your-feature`
4. Make your changes following the guidelines below
5. Submit a pull request

## Development Requirements

**prani is a Cyrius project.** The Rust it was ported from is a frozen parity
oracle kept in git history, not the build — nothing in CI or the release runs
`cargo`.

- The Cyrius toolchain, at the version pinned in `cyrius.cyml` `[package].cyrius`.
  That pin is the single source of truth; never hardcode a version anywhere else.

  ```sh
  CYRIUS_VERSION="$(grep '^cyrius = ' cyrius.cyml | head -1 | sed 's/cyrius = "\(.*\)"/\1/')"
  curl -sSf "https://raw.githubusercontent.com/MacCracken/cyrius/$CYRIUS_VERSION/scripts/install.sh" | \
    CYRIUS_VERSION="$CYRIUS_VERSION" sh
  ```

  The installer is fetched at the pinned tag too, not from `main`, so an unchanged
  prani commit always installs the same way (CI does the same).

- A Rust toolchain is needed **only** to build the oracle — checked out from
  `2.0.3` — and to re-run the benchmark comparison against it. That comparison
  was made once, in 2.0.7; the numbers and the method are in
  [`docs/benchmarks.md`](docs/benchmarks.md). It is not needed to build, test,
  or release prani.

```sh
cyrius deps                              # resolve dependencies
cyrius build src/main.cyr build/prani    # compile (the f64 build)
cyrius test                              # every tests/*.tcyr, f64 build
cyrius test -D PRANI_F32                 # every tests/*.tcyr, f32 build
cyrius test tests/voice.tcyr             # one suite
sh scripts/gen-f64.sh                    # regenerate src/f64/ from src/ (after any src/ edit)
cyrius distlib                           # regenerate dist/prani.cyr from [lib].modules
```

### Two float formats (ADR-0010)

prani builds in either float format: f64 by default, f32 with `-D PRANI_F32` (a
consumer adds `defines = ["PRANI_F32"]` to its `[build]`). Both are consumer-facing,
so both are tested, and both must pass.

- **Write arithmetic over `src/real.cyr`'s primitives**, never the `f64_*` builtins:
  `pr_mul(a, b)`, not `f64_mul(a, b)`. A real is the build's float, and the
  primitives are what make one source compile in either. A constant is a pair:
  `#ifdef PRANI_F32` with its f32 bits, `#else` with its f64 bits, each from the same
  decimal.
- **Write a compare as a bare truth value**: `if (pr_lt(a, b))`, `if (!pr_ge(a, b))`.
  After an `#inline` replay, cycc 6.7.18 does not fuse `== 1` / `== 0` into the
  branch, which cost 2x on a clamp.
- **svara, naad, hisab and bayan are f64 in both builds.** Widen a real into them
  with `pr_to64`, and narrow what comes back with `pr_of64`. A missed crossing
  compiles: the f32 suites are what find it.
- **An oscillator's time and phase are f64** in the f32 build: `pr_time(n, sr64)` and
  `pr_sinw(w64, t)` in an `#ifdef PRANI_F32` arm, with the f64 arm left as it was.
- **The f64 build compiles `src/f64/`**, which `scripts/gen-f64.sh` writes from
  `src/`. Never edit `src/f64/`. Regenerate after every `src/` edit, then run
  `cyrius distlib`. A module with no arithmetic has no twin and needs no guard.
- **A test states an expected real as the f64 value it should be.** Compare a real
  with `assert_real(actual, expected, ulps, msg)` (`tests/realt.cyr`): exact in the
  f64 build, and within `ulps` in the f32 build, where `ulps` is the measured error
  of that computation, recorded beside it. Never widen a budget to make a case pass.
  Where the format itself decides an input or a count, state both with
  `t_per(a64, a32)`.

## Code Quality Requirements

Before submitting a PR, one command must exit 0:

```sh
cyrius audit
```

It runs **fmt · lint · docs · tests · bench** — the same five gates CI runs. Two
things to know about it:

- Its **fmt gate agrees with `cyrius fmt <file> --check`**, since cyrius 6.7.12:
  both accept a 4-space continuation step as well as the canonical 2, and refuse
  anything deeper. `cyrius fmt <file>` rewrites to the canonical 2. Before 6.7.12
  the audit was the stricter of the two.
- Every `cyrius …` call re-resolves dependencies and races on `cyrius.lock`. If
  you run more than one at a time, serialize them: `flock /tmp/prani-build.lock cyrius …`.

CI runs gates **around** the audit that the audit itself does not, added across
2.0.4, 2.0.6, 2.0.11, 2.2.0 and 2.3.0. The audit runs the f64 build only. Run these
before you push, or the PR fails on something green locally:

```sh
cyrius test -D PRANI_F32                         # every suite, f32 build
sh scripts/gen-f64.sh --check                    # src/f64/ must match src/
cyrius distlib && git diff --exit-code -- dist/  # bundle coherence: dist/ must match src/
PRANI_FORMAT=f64 sh scripts/run-examples.sh      # fmt + lint + build + run all five docs/examples/
PRANI_FORMAT=f32 sh scripts/run-examples.sh      # and again, built for f32
sh scripts/bench-check.sh                        # timing delta vs benches/baseline.csv
sh scripts/check-alloc.sh                        # every allocation goes through prani's funnel
cyrius fuzz tests/prani.fcyr                     # the decoders' fuzz harness (2.2.1)
cyrius fuzz -D PRANI_F32 tests/prani.fcyr        # and in the f32 build
```

The fuzz harness feeds 1,500 mutated documents to all four hand-written decoders.
Since 2.3.0 it is a CI step, in both formats.

Bundle coherence runs **before** the audit in CI, deliberately — see the
benchmark warning below.

## Code Standards

- **Fallible functions return a negative `PRANI_ERR_*`** from `src/error.cyr`, never
  a sentinel the caller can mistake for data. Callers check with `prani_is_err`.
  Constructors that can fail return "pointer or negative code" — see
  [ADR-0001](docs/adr/0001-check-svara-tract-constructor.md).
- **Every top-level symbol is `prani_` / `PRANI_` / `Pr`-prefixed.** The distlib
  bundle is one flat namespace shared with `svara_*` and naad's `validate_*` /
  `ERR_NONE`. Front-load collision avoidance.
- **Every serializable type gets `#derive(Serialize)` + a `*_to_json` /
  `*_from_json_str` codec + a roundtrip test.** Real fields are typed `i64` so they
  serialize as exact bit patterns, and a document holds **f64** bit patterns in both
  builds (ADR-0010): the f32 build widens on write and narrows on read, through a
  64-bit shadow struct for a derived type and `pr_to64` / `pr_of64` in a
  hand-written codec.
  Deserializers must reject input that does not parse
  ([ADR-0002](docs/adr/0002-deserializers-report-parse-failure.md)).
- **No panics, no unchecked dereferences, no raw allocation.** Allocate only through
  `prani_alloc` / `prani_vec_new` / `prani_vec_push` (`src/error.cyr`), check the
  result, and pass `PRANI_ERR_ALLOC` on
  ([ADR-0008](docs/adr/0008-a-refused-allocation-is-reported.md)). Raw `alloc()`
  returns 0 on exhaustion and `vec_push` exits the process, and
  `scripts/check-alloc.sh` fails CI on either. A new allocating path also needs a
  case in `tests/allocrefused.tcyr`.
- **Structured logging via `src/logging.cyr`** (`prani_log_info` / `_warn` /
  `_error` / `_debug` / `_fatal`, sakshi-backed) — never a raw write.
- **Float literals**: integers via `f64_from(n)`; non-integers as a module-top
  `var NAME = 0x…;` holding the IEEE-754 hex bit pattern, with the decimal in a
  comment. Generate with
  `python3 -c "import struct;print(hex(struct.unpack('<Q',struct.pack('<d',V))[0]))"`.
- **Module files do not `include` each other.** The build entry and each test
  harness include them in dependency order. A new module must be added to
  `cyrius.cyml` `[lib].modules` in that order, or it will not reach the bundle.
- **`var buf[N]` is N *bytes*, not N entries.**
- **One change at a time.** Never bundle unrelated changes into one PR.
- **Cite the Rust oracle by tag** — see below.

## Adding New Species

1. Add the `PRANI_SP_*` constant to `src/species.cyr` (append — the discriminants
   are declaration order and are asserted) and bump `PRANI_SP_COUNT`.
2. Add bioacoustically-informed params to `species_params()`.
3. Update `species_supports_vocalization()` for the valid species/vocalization pairs.
4. Add a `species_bout_template()` entry.
5. Add assertions to `tests/species.tcyr` (params, the support matrix, the
   discriminant) **and** a synthesis assertion covering its vocal apparatus.
6. Add a benchmark to `tests/prani.bcyr`.
7. If the species reaches the FFI index tables, extend `tests/ffi.tcyr`'s
   out-of-range cases.
8. Extend `ex_species_name()` in `docs/examples/species_tour.cyr` — it sweeps to
   `PRANI_SP_COUNT` over a hand-written name table, and CI builds and runs it.

## Benchmarks

All performance-related changes must include benchmark results. Run:

```sh
cyrius bench tests/prani.bcyr
```

⚠ **Run `cyrius distlib` before believing a bench number after editing `src/`.**
`tests/prani.bcyr` benchmarks `dist/prani.cyr` — the generated bundle,
deliberately, since that is what a consumer gets — so a stale bundle reports the
*last bundled* code. 2.0.10 lost time to exactly this: the audit read 155 ns for
a change that actually measured 114. CI runs bundle coherence before the audit;
locally nothing stops you.

`sh scripts/bench-check.sh` compares a run against `benches/baseline.csv` and
prints a delta per benchmark. It is report-only (`BENCH_GATE=1.5` opts into
failing past a factor). Allocation budgets, which are deterministic, are
hard-gated instead in `tests/allocbudget.tcyr`. The baseline is a claim about one
host and one toolchain, so re-record it after every toolchain or dependency bump,
in a commit of its own: `sh scripts/bench-check.sh --write` takes each benchmark's
fastest of three runs and writes the host, toolchain and tree into the header. A
compare warns when the baseline's toolchain is not the one running.

`./scripts/bench-history.sh` runs the same harness and appends each figure to
`benches/history.csv` with a timestamp and git rev, so a regression is visible
across commits. Note that `lib/bench.cyr` measures and subtracts the timer floor
(cyrius 6.5.19+); a criterion run does not, which matters for any comparison
against the oracle. Reference numbers and the method live in
[`docs/benchmarks.md`](docs/benchmarks.md).

## Reading the Rust oracle

prani is a port. The original Rust is preserved as the parity oracle in git
history — roadmap **2.0.8** retired `rust-old/` from the working tree, so you
read the oracle with `git show` rather than off disk. The rule is
[ADR-0004](docs/adr/0004-cite-the-oracle-by-tag.md):
**a citation to the oracle names a tag.**

```sh
git show 2.0.3:rust-old/src/voice.rs           # port era (2.0.0+) — the oracle at rust-old/
git show 1.1.0:src/voice.rs                    # Rust era (1.1.0 and earlier) — the Rust WAS src/
git show 2.0.3:rust-old/tests/integration.rs   # 72 of the oracle's 73 #[test] blocks
git grep 'fn vocalize' 2.0.3 -- rust-old/src   # grep across the whole oracle
```

The path changed at 2.0.0 because the port created `rust-old/` by moving the Rust
aside. Use `2.0.3` — the last release before the removal arc, and the state every
parity claim in this tree was checked against.

When you write a parity comment, cite `2.0.3:rust-old/src/tract.rs:104`, not
`rust-old/src/tract.rs:104`. The directory is gone from the tree: a bare path
with a line number resolves against nothing, and a reviewer cannot tell whether
it was ever right.

If `git show` cannot find the tag, your clone is shallow — `git fetch --tags --unshallow`.

## License

By contributing, you agree that your contributions will be licensed under GPL-3.0.
