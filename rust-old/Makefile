.PHONY: check fmt clippy test audit deny bench coverage build doc clean

check: fmt clippy test audit

fmt:
	cargo fmt --all -- --check

clippy:
	cargo clippy --all-features --all-targets -- -D warnings

test:
	cargo test --all-features

audit:
	cargo audit

deny:
	cargo deny check

bench:
	./scripts/bench-history.sh

coverage:
	cargo llvm-cov --all-features --html --output-dir coverage/

build:
	cargo build --release --all-features

doc:
	RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features

clean:
	cargo clean
