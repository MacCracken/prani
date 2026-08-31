# prani — Roadmap

> Milestone plan. State lives in [`state.md`](state.md); this file is the
> sequencing — what ships, in what order, against what dependency gates.

## 2.0.0 criteria — ✅ met

- [x] Rust → Cyrius surface parity verified (function-for-function vs `rust-old/`)
- [x] Test coverage adequate for the surface area (627 assertions, 15 suites)
- [x] Benchmarks captured in [`docs/benchmarks.md`](../benchmarks.md)
- [x] `dist/prani.cyr` bundle assembled + smoke-linked (`src/main.cyr`)
- [x] CHANGELOG complete for the 2.0.0 port
- [ ] At least one downstream consumer green (kiran/joshua — pending their port)
- [x] Security audit pass — done in 2.0.3, [`../audit/2026-08-30-audit.md`](../audit/2026-08-30-audit.md)

## Milestones

### M0 — Port scaffold — ✅ shipped 2026-07-03

- `cyrius port` scaffold; Rust source frozen at `rust-old/`; doc tree.

### M1 — Rust → Cyrius surface parity (2.0.0) — ✅ shipped 2026-07-03

- All 15 modules ported (L0 foundation → FFI), each cross-checked against the
  frozen 3,527-line oracle. `math.rs` folded into `f64_*` builtins; `lib.rs`
  carries no independent logic.
- Delivered solo (foundation + keystone vocalization/sequence) + dependency-ordered
  parallel workflow waves (leaves → tract+bridge → voice → preset+stream → ffi),
  each integrated and independently re-verified against `rust-old/`.
- `dist/prani.cyr` bundle (collision-audited to zero) + hot-path benchmarks.

### M2 — Hardening & consumer-green (2.x)

- Broaden hot-path benchmarks; capture a like-for-like Rust-vs-Cyrius comparison.
- [x] Security/deep-review work-loop pass (correctness / memory-safety /
  performance / refactor, cross-checked against `rust-old/`) →
  [`../audit/2026-08-30-audit.md`](../audit/2026-08-30-audit.md). 11 findings,
  8 repaired in 2.0.3, 3 accepted; three oracle divergences in `docs/adr/`.
- Close the three accepted findings: unchecked `alloc()`/`vec_push` returns (F9),
  input-range validation beyond parse success (F10/F11, deferred by ADR-0002).
- Pin dep bundles for reproducible release builds; wire CI. (Pins current as
  of 2.0.2; CI derives the toolchain from `cyrius.cyml`.)
- Consumer-green: kiran (game engine) + joshua (game manager) once they port up
  the stack.

## Out of scope (for 2.0.0)

- A C ABI compatible with the old Rust `extern "C"` surface — the `ffi` module is
  re-expressed in Cyrius conventions (handles = pointers, buffers = vecs); a raw
  C-ABI shim is a later concern if a non-Cyrius host needs it.
  (serde/JSON persistence IS supported — restored in 2.0.1 via
  `#derive(Serialize)`+bayan on all 14 types that had it.)
