# prani — Claude Code Instructions

> **Core rule**: this file is **preferences, process, and procedures** —
> durable rules that change rarely. Volatile state (current version,
> module line counts, port progress, test counts, consumers) lives in
> [`docs/development/state.md`](docs/development/state.md).
> Do not inline state here.

## Project Identity

**prani** — Cyrius port of a Rust project (3527 lines of Rust, preserved in git history — see [Scaffolding](#scaffolding)).

- **Type**: Port (Rust → Cyrius)
- **License**: GPL-3.0-only
- **Language**: Cyrius (toolchain pinned in `cyrius.cyml [package].cyrius`)
- **Version**: `VERSION` at the project root is the source of truth — do not inline the number here
- **Standards**: [First-Party Standards](https://github.com/MacCracken/agnosticos/blob/main/docs/development/applications/first-party-standards.md) · [First-Party Documentation](https://github.com/MacCracken/agnosticos/blob/main/docs/development/applications/first-party-documentation.md)

## Goal

prani (प्राणी — "living being / creature") **owns non-human vocal synthesis for
AGNOS**: species-specific vocal tract models, call-pattern generators, and
behavioral vocalization mapping for animals, fantasy beings, and alien
creatures. It builds on **svara**'s glottal/formant engine (and naad's filters)
to give each species its own vocal apparatus — laryngeal, syringeal,
stridulatory, vibratile, or noise-only — driven by emotion, fatigue, and
call-intent state. Serves kiran (game engine) and joshua (game manager) and any
AGNOS component needing creature voices.

## Current State

> Volatile state lives in [`docs/development/state.md`](docs/development/state.md) —
> port progress, surface parity, in-flight work. Refreshed every release.

This file (`CLAUDE.md`) is durable rules.

## Scaffolding

Project was scaffolded with `cyrius port`. The original Rust is the reference oracle — cross-check the port against it.

The oracle is not in the working tree: roadmap **2.0.8** retired `rust-old/`, so the Rust is read out of git history instead. The rule for citing it is [ADR-0004](docs/adr/0004-cite-the-oracle-by-tag.md):

```sh
git show 2.0.3:rust-old/src/voice.rs   # port era (2.0.0+)
git show 1.1.0:src/voice.rs            # Rust era (1.1.0 and earlier) — the Rust WAS src/
git grep 'fn vocalize' 2.0.3 -- rust-old/src   # grep across the whole oracle
```

## Quick Start

```sh
cyrius deps                              # resolve dependencies
cyrius build src/main.cyr build/prani    # compile
cyrius test                              # run tests/*.tcyr
```

## Key Principles

- **Cross-check against the Rust oracle** (`git show 2.0.3:rust-old/…`) — the port's correctness bar is "matches what Rust did". Diverge only with an ADR.
- **Correctness over cleverness** — if the Cyrius behavior diverges silently from Rust, the bugs win
- Test after every change, not after the feature is "done"
- ONE change at a time — never bundle unrelated changes
- Build with `cyrius build`, not raw `cat file | cc5` — the manifest auto-resolves deps
- Source files only need project includes — stdlib auto-resolves from `cyrius.cyml`
- `var buf[N]` = N **bytes**, not N entries

## Rules (Hard Constraints)

- **Do not commit or push** — the user handles all git operations
- **Never use `gh` CLI** — use `curl` to the GitHub API if needed
- **The oracle lives in git history, not on disk** — read it with `git show 2.0.3:rust-old/<path>` ([ADR-0004](docs/adr/0004-cite-the-oracle-by-tag.md)). Do not restore it to the working tree
- **Cite the oracle by tag, not a bare path** — write `2.0.3:rust-old/src/tract.rs:104`, never `rust-old/src/tract.rs:104`. Since 2.0.8 a bare path resolves against nothing ([ADR-0004](docs/adr/0004-cite-the-oracle-by-tag.md))
- Do not skip tests before claiming changes work
- Do not modify `lib/` files (vendored stdlib / dep symlinks)
- Do not hardcode toolchain versions in CI YAML — `cyrius = "X.Y.Z"` in `cyrius.cyml` is the source of truth

## Documentation

- [`docs/adr/`](docs/adr/) — Architecture Decision Records (*why X over Y?*)
- [`docs/audit/`](docs/audit/) — P(-1) sweep reports (`YYYY-MM-DD-audit.md`): findings with severity, file, line, and disposition
- [`docs/architecture/`](docs/architecture/) — Non-obvious constraints
- [`docs/guides/`](docs/guides/) — Task-oriented how-tos
- [`docs/examples/`](docs/examples/) — Runnable examples
- [`docs/development/state.md`](docs/development/state.md) — Live state
- [`docs/development/roadmap.md`](docs/development/roadmap.md) — **Open work only**. **2.0.x** is the preserve-first arc ending in `rust-old/`'s removal (2.0.8); **2.x** is surface-changing or externally blocked work. Completed milestones live in `CHANGELOG.md`, not here

