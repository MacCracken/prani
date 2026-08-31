# 0003 — A failed stream fill reports 0 written and retires the stream

**Status**: Accepted
**Date**: 2026-08-30

## Context

`rust-old/src/stream.rs:fill_buffer` guards only the **copy** with the synthesis
result:

```rust
if let Ok(block) = tract.synthesize(contour, to_render, &options) {
    buffer[..to_render].copy_from_slice(&block[..to_render]);
}
// ... spectral tilt and amplitude applied to buffer[..to_render] unconditionally
self.samples_rendered += to_render;
to_render
```

On the error arm the copy is skipped, but the function still advances
`samples_rendered` and still **returns `to_render`** — it tells the host "I wrote
N samples" having written none. The host then plays whatever the buffer held
before, which on a reused audio buffer is the previous block: an audible repeat,
attributed to prani, with no error anywhere.

The port inherited the shape. It also, separately, applies the tilt and amplitude
passes only when a block was produced, where the oracle applies them
unconditionally to the buffer — a divergence that had not been written down.

There is a real tension. Reporting 0 without advancing is honest, but a host
draining `while (stream_is_finished(s) == 0) { fill(s, buf); }` would then spin
forever, because the failure is a property of the stream's own configuration and
will repeat identically on every call.

## Decision

A failed fill returns **0**, and marks the stream finished by setting
`samples_rendered` to `total_samples`.

This applies to both failure paths: a `crtract_new` rejection (ADR-0001) and a
`crtract_synthesize` error. A zero-length caller buffer also returns 0 early,
since it can never advance the stream either — but it does **not** retire it,
because that is a caller mistake rather than a property of the stream.

The undocumented tilt/amplitude divergence is settled the same way: those passes
run only when there is a block, and the reason is now recorded here. Applying a
lowpass and a gain to stale audio, as the oracle does, makes the repeat quieter
and darker rather than absent, which is worse than leaving it untouched.

## Consequences

- **Positive** — a host can distinguish "stream ended" from "stream produced N
  samples", and can never be handed a count for audio that was not written. A
  drain loop terminates on failure instead of spinning.
- **Negative** — a stream that fails once is finished for good; there is no
  retry. Given both failure causes are configuration properties fixed at
  `stream_new` time, a retry would fail identically, so this costs nothing today.
  It would need revisiting if a transient failure mode is ever introduced.
- **Neutral** — the return value now means "samples actually written", which is
  what the name says but not what the oracle did. Consumers that summed the
  return to track position get a different (correct) total on the failure path.

## Alternatives considered

- **Keep the oracle's behaviour.** Rejected: it is the defect.
- **Return 0 without retiring the stream.** Rejected: a documented drain loop
  would never terminate. The failure is permanent, so the stream should say so.
- **Return a negative error code.** Rejected: the function returns a sample
  count, and its callers (including `prani_ffi_stream_fill`, whose C contract
  returns a count) add it to a position. A negative return would corrupt that
  arithmetic in exactly the hosts least able to notice.
