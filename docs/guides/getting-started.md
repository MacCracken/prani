# Getting started with prani

## Build

```sh
cyrius deps                              # resolve dependencies
cyrius build src/main.cyr build/prani    # compile
cyrius test                              # run tests/*.tcyr
```

## Layout

- `src/main.cyr` — entry point. Top-level `var r = main(); syscall(SYS_EXIT, r);`.
- `tests/` — test suite (`.tcyr` files, auto-discovered by `cyrius test`).
- The original Rust source — the reference oracle — is kept in git history rather than in the tree. Read it with `git show 2.0.3:rust-old/src/voice.rs`, or grep it with `git grep <pattern> 2.0.3 -- rust-old/`. See [ADR-0004](../adr/0004-cite-the-oracle-by-tag.md).

## Adding a feature

1. Edit `src/main.cyr` (or add a new module and `include` it).
2. Cross-check parity against the oracle at the `2.0.3` tag (`git show 2.0.3:rust-old/src/<module>.rs`).
3. Add a test case to `tests/prani.tcyr`.
4. Run `cyrius test`.
5. Bump `VERSION` and add a CHANGELOG entry before tagging.

See [`../adr/template.md`](../adr/template.md) when a non-trivial design choice deserves an ADR.
