# prani — Roadmap

> **What is still to do**, in release order. State lives in [`state.md`](state.md),
> and everything shipped lives in [`../../CHANGELOG.md`](../../CHANGELOG.md). The port
> ledger is [`port-audit.md`](port-audit.md).
>
> ⛔ marks work blocked outside this repo, with the blocker named. Everything else can
> start today. Every item carries its provenance; an item with no source in the tree
> is not on this roadmap.

| Release | Lane | Status |
|---|---|---|
| **2.x.x** | [A raw C ABI](#2xx--a-raw-c-abi-conditional) | conditional on a host that needs it |
| ⛔ | [Blocked upstream](#-blocked-upstream) | waiting on other repos |

---

## 2.x.x — a raw C ABI (conditional)

**Source**: the 2.0.0 roadmap's "Out of scope (for 2.0.0)" note.

- [ ] **Only if a non-Cyrius host needs one.** The `ffi` module is re-expressed in
      Cyrius conventions (handles are pointers, buffers are vecs), so it is not
      ABI-compatible with the old Rust `extern "C"` surface. No such host exists
      today. Recorded so the decision is not rediscovered.

---

## ⛔ Blocked upstream

- [ ] ⛔ **Consumer-green: kiran and joshua.** The one 2.0.0 criterion still open.
      Nothing has yet driven `prani_ffi_stream_start` / `_fill` / `_is_finished` from a
      host callback, which is where the stream contract (ADR-0003, and since 2.2.0
      `prani_ffi_stream_last_error`) would meet threading, callback timing and lifetime
      across frames. 2.0.6's `docs/examples/streaming.cyr` stands in for the lifecycle;
      an example is not a host. **Blocked on kiran and joshua porting up the stack.**
- [ ] ⛔ **svara's own allocations stop at the null page.** prani reports every
      refusal of its own allocations (ADR-0008), and what svara reports it passes on.
      Refusals of svara's own fixed-size allocations end the process inside svara, and
      `svara_tract_new` uses a `vec_push` that exits. That is svara's ADR-0009.
      **Blocked on svara reporting more.** When it does, pass the codes on, and add the
      windows to `tests/allocrefused.tcyr`'s naad section.
- [ ] ⛔ **A vec cannot be created at its final size.** A render whose length is known
      up front (`crvoice_vocalize_with_intent`'s output, the purr path) still grows
      by doubling, because the stdlib has no `vec_with_capacity` and svara declined
      to hand-build vec headers rather than depend on the layout. Found as audit A7.
      **Blocked on a stdlib `vec_with_capacity`.** When it lands, pre-size through
      `prani_vec_new`'s funnel, so the refusal is still reported (ADR-0008).
- [ ] ⛔ **The derive-generated decoders take the last of two equal keys and accept
      `05`.** `#derive(Serialize)` is the compiler's, so `PrRng`, `DcBlocker`, the
      fatigue and emotion types, `IntentModifiers`, `SpeciesParams`, `CallElement` and
      `CallBout` answer a duplicate key differently from prani's four hand-written
      decoders, which bayan 1.11.1 made strict. Found at 2.1.1. **Blocked on cyrius's
      derive.** When it changes, add the case to `tests/hardening.tcyr`'s duplicate-key
      group.
- ⛔ **Codegen.** The gap to the Rust is cycc against LLVM plus svara 3.x's extra work,
  and it belongs to cyrius. 2.3.0 measured it f32 against f32: 10.2× for prani's f32
  build and 9.3× for its f64 build (`docs/benchmarks.md`), so float width does not
  explain it. Part of the f32 build's own cost is cycc's `#inline` copying its
  arguments (ADR-0010).

---

## Watch items — not scheduled, but written down

Each is deliberately left alone, with the condition that would change that.

- **Two duplications held at two instances.** `crtract_synthesize_stridulatory`'s bee
  branch is byte-identical to `crtract_synthesize_vibratile`, and the `boundary_boost`
  block is duplicated between `voice.cyr` and `stream.cyr`. CLAUDE.md waits for the
  third instance. **Extract on the third, not before.**
- **ADR-0003's retirement rule.** A failed fill retires the stream; 2.2.0 kept that
  for a refused allocation, because the bump arena frees nothing, so a refusal is not
  one a later fill could wait out. **Revisit if prani ever allocates from something
  that can free**, an arena a host can reset for instance.
- **svara's sample-rate floor is a measurement.** It moved once, from >1000 Hz to
  >1200 Hz at svara 3.5.4. `tests/hardening.tcyr`'s F12 group pins 1200/1201 and
  7500/7501 with controls. **If those controls fail, re-measure; do not hardcode the
  floor in prani** (ADR-0001).
- **prani's excitation gain is solved against svara's pulse.** `PR_TR_EXC_GAIN`
  (ADR-0007) divides svara's `SV_G_ROSENBERG_GAIN` back out, so a recalibration of that
  constant flows through, but a change to the pulse's shape does not.
  `tests/voice.tcyr` pins the calibration and `tests/tract.tcyr` the svara facts it
  rests on. **If either fails on a svara bump, re-solve the gain by the method in
  `src/tract.cyr`. Do not nudge it.**
- **Re-running the Rust comparison depends on crates.io.** The tree at tag `2.0.3`
  has no `Cargo.lock`. 2.3.0 built it offline from the local cargo cache, which
  resolved svara 1.0.0, naad 1.2.5 and hisab 1.4.0, and recorded the f32-vs-f32
  comparison in `docs/benchmarks.md`. **Revisit only for a fresh comparison.**
- **The f64 build compiles generated twins because `#inline` copies its arguments**
  (ADR-0010). Through the primitives, the f64 build measured 1.09× 2.2.1. **If cycc's
  `#inline` stops copying its arguments, or cyrius gains a type alias,** measure
  again, and retire `src/f64/` and `scripts/gen-f64.sh` if the primitives cost
  nothing.
