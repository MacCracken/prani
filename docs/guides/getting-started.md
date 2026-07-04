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
- `rust-old/` — original Rust source preserved for parity checks. Do not modify; it's the reference oracle.

## Adding a feature

1. Edit `src/main.cyr` (or add a new module and `include` it).
2. Cross-check parity against `rust-old/`.
3. Add a test case to `tests/prani.tcyr`.
4. Run `cyrius test`.
5. Bump `VERSION` and add a CHANGELOG entry before tagging.

See [`../adr/template.md`](../adr/template.md) when a non-trivial design choice deserves an ADR.
