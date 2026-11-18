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
| **2.3.x** | [What the 2.2.1 sweep deferred](#23x--what-the-221-sweep-deferred) | can start today |
| **2.x.x** | [A raw C ABI](#2xx--a-raw-c-abi-conditional) | conditional on a host that needs it |
| ⛔ | [Blocked upstream](#-blocked-upstream) | waiting on other repos |

---

## 2.3.x — what the 2.2.1 sweep deferred

**Source**: [`docs/audit/2026-11-17-audit.md`](../audit/2026-11-17-audit.md). The
sweep confirmed 53 findings, and 2.2.1 repairs 49 of them. A7 is blocked upstream
(below), and B13 and C13 are kept, with the reasons in the report. What is left
here is C10, a second decision found while verifying, and a CI gap review D
suspected but could not exercise.

- [ ] **Decide what wind does to pitch (audit C10).** `bridge_doppler_velocity_from_wind`
      returns `+wind` for angle 0, which the oracle documents as wind blowing from the
      source toward the listener. `spatial_apply_doppler_shift` reads `+` as receding,
      so a tailwind lowers the pitch: 1,000 samples come back as 1,029. Both halves
      are the oracle's (`2.0.3:rust-old/src/bridge.rs:115-128`,
      `2.0.3:rust-old/src/spatial.rs:46`). The physics does not pick a sign: a steady
      wind shifts nothing when the source and listener are both at rest, because both
      move against the medium together. The options are to negate it (a tailwind
      raises the pitch), to keep the sign and say so, or to retire the function as a
      shift with no physical basis. **Needs an ADR either way**, since each is a
      divergence or a removal.
- [ ] **Decide whether a pitch at or above Nyquist is refused.** Found while measuring
      ADR-0009. A half-size cricket at 8 kHz has its carrier above 4 kHz, and it
      renders 0.1 s of numerical silence (RMS 3.3e-14) as success. Neither svara nor
      the oracle refuses such a pitch: `svara_glottal_new` checks its own f0 range,
      not the sample rate's. Refusing it would make prani stricter than svara for
      laryngeal voices too, so it is a contract decision (ADR-0001's "never fabricate"
      against svara's own floor) and not a guard to add quietly.
- [ ] **CI fetches the installer from cyrius `main`.** The toolchain version is pinned
      from `cyrius.cyml`, which is correct, but `install.sh` itself comes from the
      default branch. Fetch it at the pinned tag instead, reading the tag from
      `cyrius.cyml` as the version is read, never hardcoded (CLAUDE.md). Suspected
      by review D and not exercised: this tree cannot run CI.

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
