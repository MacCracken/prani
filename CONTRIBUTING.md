# Contributing to prani

Thank you for your interest in contributing to prani.

## Getting Started

1. Fork the repository
2. Clone your fork
3. Create a feature branch: `git checkout -b feature/your-feature`
4. Make your changes following the guidelines below
5. Submit a pull request

## Development Requirements

- Rust 1.89+ (stable)
- cargo-deny (`cargo install cargo-deny`)
- cargo-audit (`cargo install cargo-audit`)

## Code Quality Requirements

Before submitting a PR, ensure all checks pass:

```sh
cargo fmt --check
cargo clippy --all-features --all-targets -- -D warnings
cargo test --all-features
cargo audit
cargo deny check
RUSTDOCFLAGS="-D warnings" cargo doc --all-features --no-deps
```

## Code Standards

- `#[non_exhaustive]` on all public enums
- `#[must_use]` on all pure functions
- `#[inline]` on hot-path sample processing functions
- Zero `unwrap`/`panic` in library code — use `Result` or safe defaults
- All public types must derive `Serialize`, `Deserialize`, `Debug`, `Clone`
- All new types require serde roundtrip tests
- Use `tracing` for structured logging (not `println!`)

## Adding New Species

1. Add the variant to `Species` enum in `species.rs`
2. Add bioacoustically-informed `SpeciesParams` in `Species::params()`
3. Update `supports_vocalization()` for valid species/vocalization combinations
4. Add integration tests (synthesis, serde roundtrip)
5. Add a benchmark

## Benchmarks

All performance-related changes must include benchmark results. Run:

```sh
cargo bench
# or
./scripts/bench-history.sh
```

## License

By contributing, you agree that your contributions will be licensed under GPL-3.0.
