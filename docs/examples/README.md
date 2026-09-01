# prani — runnable examples

Five worked programs. **CI builds and runs every one on each push**
(`scripts/run-examples.sh`), so they cannot rot against the API.

```sh
sh scripts/run-examples.sh                 # all five
cyrius build docs/examples/basic.cyr build/examples/basic && ./build/examples/basic
```

Each example includes the dependency bundles and **`dist/prani.cyr`** — the
published bundle — rather than `src/*.cyr`. That is deliberate: building them
exercises prani exactly as a consumer does, so this doubles as the closest thing
the project has to a consumer integration test.

| Example | Read it for |
|---|---|
| [`basic.cyr`](basic.cyr) | Start here. A wolf howl end to end, and **the vec-or-negative-code convention** every fallible function in prani uses. |
| [`species_tour.cyr`](species_tour.cyr) | One call per vocal apparatus — laryngeal, syringeal, stridulatory, vibratile, noise-only — plus reading a species' parameters back at runtime. |
| [`error_handling.cyr`](error_handling.cyr) | Every failure shape, and the ADR divergences behind them: a rejected sample rate ([0001](../adr/0001-check-svara-tract-constructor.md)), rejected JSON ([0002](../adr/0002-deserializers-report-parse-failure.md)), a retired stream ([0003](../adr/0003-failed-fill-reports-zero-and-retires.md)), and refused out-of-range input ([0006](../adr/0006-reject-non-finite-numeric-input.md)). |
| [`streaming.cyr`](streaming.cyr) | The real-time path: one reused buffer, drained to completion — **and the full FFI lifecycle** a non-Cyrius host drives. |
| [`sequencing.cyr`](sequencing.cyr) | Call patterns above single vocalizations: bouts, phrases, and a multi-voice chorus. |

## Two things the examples are load-bearing for

**`streaming.cyr` drives the whole FFI surface** — `voice_create` → `stream_start`
→ `stream_fill` to completion → `is_finished` → destroy. Nothing else has ever
driven it end to end, which is why it stood in for the lifecycle check when
consumer-green was dropped from 2.0.8's gate. Consumer-green itself stays open on
the [roadmap](../development/roadmap.md), blocked on kiran and joshua — an example
is not a host. It
also proves the FFI and Cyrius-level drains produce **sample-for-sample identical
audio**, so the FFI is a thin adapter and not a second implementation.

**They find real defects.** Writing these five surfaced two that the 2.0.3 audit,
the 2.0.4 parity sweep and the 2.0.5 range survey all missed: an out-of-range
species tag rendered audio and reported success (fixed in 2.0.6, pinned as F13 in
`tests/hardening.tcyr`), and `stream_fill_buffer` retained **8,800 bytes per
call** on the path advertised for audio callbacks — ~757 KB/s at 44100 Hz with
512-sample blocks, retained for the life of the process (fixed in 2.0.10, down to
≤512 B and budgeted in `tests/allocbudget.tcyr`). That is the argument for
examples as a gate rather than a nicety.
