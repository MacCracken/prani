# Contributing to prani

Thank you for your interest in contributing to prani.

## Getting Started

1. Fork the repository
2. Clone your fork
3. Create a feature branch: `git checkout -b feature/your-feature`
4. Make your changes following the guidelines below
5. Submit a pull request

## Development Requirements

**prani is a Cyrius project.** The Rust at `rust-old/` is a frozen parity oracle,
not the build — nothing in CI or the release runs `cargo`.

- The Cyrius toolchain, at the version pinned in `cyrius.cyml` `[package].cyrius`.
  That pin is the single source of truth; never hardcode a version anywhere else.

  ```sh
  CYRIUS_VERSION="$(grep '^cyrius = ' cyrius.cyml | head -1 | sed 's/cyrius = "\(.*\)"/\1/')"
  curl -sSf https://raw.githubusercontent.com/MacCracken/cyrius/main/scripts/install.sh | \
    CYRIUS_VERSION="$CYRIUS_VERSION" sh
  ```

- A Rust toolchain is needed **only** to build the oracle for the roadmap 2.0.7
  benchmark comparison. It is not needed to build, test, or release prani.

```sh
cyrius deps                              # resolve dependencies
cyrius build src/main.cyr build/prani    # compile
cyrius test                              # every tests/*.tcyr
cyrius test tests/voice.tcyr             # one suite
cyrius distlib                           # regenerate dist/prani.cyr from [lib].modules
```

## Code Quality Requirements

Before submitting a PR, one command must exit 0:

```sh
cyrius audit
```

It runs **fmt · lint · docs · tests · bench** — the same five gates CI runs. Two
things to know about it:

- Its **fmt gate is stricter** than `cyrius fmt <file> --check`. The per-file
  check accepts 4-space continuation indents; the audit gate requires the
  canonical 2 spaces per open paren (cyrfmt became paren-aware in 6.5.28). Trust
  the audit, not the per-file check.
- Every `cyrius …` call re-resolves dependencies and races on `cyrius.lock`. If
  you run more than one at a time, serialize them: `flock /tmp/prani-build.lock cyrius …`.

## Code Standards

- **Fallible functions return a negative `PRANI_ERR_*`** from `src/error.cyr`, never
  a sentinel the caller can mistake for data. Callers check with `prani_is_err`.
  Constructors that can fail return "pointer or negative code" — see
  [ADR-0001](docs/adr/0001-check-svara-tract-constructor.md).
- **Every top-level symbol is `prani_` / `PRANI_` / `Pr`-prefixed.** The distlib
  bundle is one flat namespace shared with `svara_*` and naad's `validate_*` /
  `ERR_NONE`. Front-load collision avoidance.
- **Every serializable type gets `#derive(Serialize)` + a `*_to_json` /
  `*_from_json_str` codec + a roundtrip test.** f64 fields are typed `i64` so they
  serialize as exact bit patterns — the derive's 6-digit float writer is lossy.
  Deserializers must reject input that does not parse
  ([ADR-0002](docs/adr/0002-deserializers-report-parse-failure.md)).
- **No panics, no unchecked dereferences.** `alloc()` returns 0 on exhaustion and
  `vec_push` returns -1; the library-wide contract for that is roadmap 2.1.0, but
  do not add new unchecked sites in the meantime.
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

## Benchmarks

All performance-related changes must include benchmark results. Run:

```sh
cyrius bench tests/prani.bcyr
```

`./scripts/bench-history.sh` runs the same harness and appends each figure to
`benches/history.csv` with a timestamp and git rev, so a regression is visible
across commits. Note that `lib/bench.cyr` measures and subtracts the timer floor
(cyrius 6.5.19+); a criterion run does not, which matters for any comparison
against the oracle. Reference numbers and the method live in
[`docs/benchmarks.md`](docs/benchmarks.md).

## Reading the Rust oracle

prani is a port. The original Rust is preserved as the parity oracle, and roadmap
**2.0.8** removes it from the working tree — after that, you read it out of git
history rather than off disk. The rule is [ADR-0004](docs/adr/0004-cite-the-oracle-by-tag.md):
**a citation to the oracle names a tag.**

```sh
git show 2.0.3:rust-old/src/voice.rs           # port era (2.0.0+) — the oracle at rust-old/
git show 1.1.0:src/voice.rs                    # Rust era (1.1.0 and earlier) — the Rust WAS src/
git show 2.0.3:rust-old/tests/integration.rs   # the 73 #[test] blocks
git grep 'fn vocalize' 2.0.3 -- rust-old/src   # grep across the whole oracle
```

The path changed at 2.0.0 because the port created `rust-old/` by moving the Rust
aside. Use `2.0.3` — the last release before the removal arc, and the state every
parity claim in this tree was checked against.

When you write a parity comment, cite `2.0.3:rust-old/src/tract.rs:104`, not
`rust-old/src/tract.rs:104`. A bare path with a line number stops resolving the
day the directory goes, and a reviewer cannot tell whether it was ever right.

If `git show` cannot find the tag, your clone is shallow — `git fetch --tags --unshallow`.

## License

By contributing, you agree that your contributions will be licensed under GPL-3.0.
