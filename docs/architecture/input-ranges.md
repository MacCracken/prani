# prani — input ranges

> **The artifact of milestone 2.0.5.** Every public entry point that takes numeric
> input, the range it accepts, and the `PRANI_ERR_*` it returns when you go outside
> it. Generated from the 2.0.5 guard sweep and kept with the code.

## The rule

**prani reports bad numeric input; it does not fabricate a plausible answer.** That is
[ADR-0001](../adr/0001-check-svara-tract-constructor.md)'s rule applied to parameters
rather than to a dependency: *"a host that asks for 10 Hz has a bug, and silently giving
it 4000 Hz audio hides that bug."*

There is **no generic "bad parameter" code**, deliberately. The Rust oracle's five
variants already carry the range meaning by category — `PraniError::InvalidTract` is
documented as *"a vocal tract parameter is out of valid range"* — so guards map onto
them and the port does not diverge from the oracle's enum:

| Code | Used for |
|---|---|
| `PRANI_ERR_INVALID_TRACT` | sample rates, f0, tract and filter parameters |
| `PRANI_ERR_INVALID_VOCALIZATION` | durations, sample counts, vocalization parameters |
| `PRANI_ERR_INVALID_SPECIES` | species tags, voice configuration, deserialized documents |

## What was actually wrong (measured, not assumed)

```
f64_clamp(NaN,  lo, hi) -> NaN     NaN passes every clamp in the tree
f64_clamp(+inf, lo, hi) -> hi      infinities ARE bounded by clamp
f64_clamp(-inf, lo, hi) -> lo
f64_to(NaN) == f64_to(+inf) == f64_to(-inf) == i64::MIN
```

Two corrections to the 2.0.3 audit's notes, both pinned as assertions in
`tests/error.tcyr`: the hazard at a clamp site is **NaN specifically**, not
non-finiteness in general; and **all three** non-finite values convert to i64::MIN, not
just NaN — which is why a loop bound derived from one goes negative and the symptom is a
silently **empty** buffer rather than a crash or a runaway allocation.

## Guards

### `src/dsp.cyr`

| Function | Parameter | Accepted range | Rejects with |
|---|---|---|---|
| `prani_vec_push_zeros` | `n` | [0, i64::MAX] — n == 0 is VALID and unchanged; n < 0 is rejected | `PRANI_ERR_INVALID_VOCALIZATION` |

<details><summary>Deliberate divergences from the Rust oracle in this module</summary>

- **`prani_vec_push_zeros` / `n`** — Not a divergence in accepted values — a state the oracle could not represent. Rust open-coded this loop over a `usize` count, and a usize cannot be negative; the negative count is an artifact of the port's i64 (f64_to saturating to i64::MIN, and i64 arithmetic wrapping). Every value the oracle could actually pass is still accepted, zero included.

</details>

### `src/emotion.cyr`

| Function | Parameter | Accepted range | Rejects with |
|---|---|---|---|
| `emotion_valence_zone` | `valence (read from the state)` | [-1.0, 1.0] inclusive, exact (PRE_NEG_1_0 .. PRE_1_0) | `PRANI_ERR_INVALID_VOCALIZATION` |
| `emotion_arousal_zone` | `arousal (read from the state)` | [0.0, 1.0] inclusive, exact (PRE_0_0 .. PRE_1_0) | `PRANI_ERR_INVALID_VOCALIZATION` |
| `emotion_select_vocalization` | `valence + arousal (via the two zone calls)` | valence [-1.0, 1.0], arousal [0.0, 1.0] | `PRANI_ERR_INVALID_VOCALIZATION (propagated from the zone call)` |
| `emotion_select_intent` | `valence + arousal (via the two zone calls)` | valence [-1.0, 1.0], arousal [0.0, 1.0] | `PRANI_ERR_INVALID_VOCALIZATION (propagated from the zone call)` |
| `emotion_evaluate` | `smoothing (checked directly), plus valence + arousal (propagated from the selectors)` | smoothing [0.0, 0.95] inclusive exact (PRE_0_0 .. PRE_0_95); valence [-1.0, 1.0]; arousal [0.0, 1.0] | `PRANI_ERR_INVALID_VOCALIZATION` |

<details><summary>Deliberate divergences from the Rust oracle in this module</summary>

- **`emotion_valence_zone` / `valence (read from the state)`** — Yes. valence_zone is private in rust-old and infallible; Rust's f32::clamp propagates NaN identically, so the oracle also stores and classifies a NaN. No oracle test covers it (all six emotion tests in `git show 2.0.3:rust-old/tests/integration.rs` lines 723-773 use in-range values).
- **`emotion_arousal_zone` / `arousal (read from the state)`** — Yes, same class as valence_zone: private and infallible in rust-old, NaN propagates through f32::clamp there too.
- **`emotion_select_vocalization` / `valence + arousal (via the two zone calls)`** — Yes: private and infallible in rust-old (`fn select_vocalization(&self) -> Vocalization`).
- **`emotion_select_intent` / `valence + arousal (via the two zone calls)`** — Yes: private and infallible in rust-old (`fn select_intent(&self) -> CallIntent`).
- **`emotion_evaluate` / `smoothing (checked directly), plus valence + arousal (propagated from the selectors)`** — Yes, and this is the signature-shaped one: rust-old's `EmotionState::evaluate()` returns EmotionOutput, not Result. It is the same call ADR-0001 made for crtract_new - "pointer or negative code" - and it is what lets the module report at all. serde's derived Deserialize in the oracle likewise accepts any float for any field, so every state rejected here is one the oracle accepted.

</details>

### `src/fatigue.cyr`

| Function | Parameter | Accepted range | Rejects with |
|---|---|---|---|
| `fatigue_record_call` | `duration` | [0.0, 1e9] seconds, inclusive at both ends (PR_FAT_MAX_DURATION_S = 0x41cdcd6500000000) | `NONE — record_call has no error channel. Guard fires prani_log_error("prani: fatigue record_call duration out of range, call not recorded") and drops the whole call event with an early `return 0;`; state left bit-identical. No signature change (callers already discard the return).` |
| `fatigue_rest` | `duration` | [0.0, 1e9] seconds, inclusive at both ends (same PR_FAT_MAX_DURATION_S) | `NONE — rest has no error channel. Guard fires prani_log_error("prani: fatigue rest duration out of range, rest not recorded") and drops the rest event with an early `return 0;`. No signature change.` |
| `fatigue_modifiers_make` | `pitch_offset, breathiness_delta, amplitude_scale, jitter_scale (all four)` | finite (prani_finite2 twice) — REPORTED ONLY, not enforced | `NONE, and none possible without a contract change. modifiers_make returns a POINTER with no error channel; giving it one is roadmap 2.1.0's constructor-contract work, so per decision 2 it is left alone. It fires prani_log_error("prani: fatigue modifiers built from a non-finite value") and returns the values it was given, UNALTERED.` |

<details><summary>Deliberate divergences from the Rust oracle in this module</summary>

- **`fatigue_record_call` / `duration`** — YES. rust-old/src/fatigue.rs @2.0.3 record_call(duration: f32, ...) accepted every one of NaN, ±inf, negative and 1e300 and poisoned itself identically — f32 could represent all of them. Rejecting them is deliberate. Dropping the event rather than substituting a value is not ADR-0001 fabrication: no value is invented, and the log line reports the failure through the channel prani already has (precedent: tract.cyr:117, tract.cyr:136).
- **`fatigue_rest` / `duration`** — YES. The f32 oracle's rest(duration: f32) accepted NaN, ±inf and negatives with the same consequences. Rejecting them is deliberate.
- **`fatigue_modifiers_make` / `pitch_offset, breathiness_delta, amplitude_scale, jitter_scale (all four)`** — No behavioural divergence — the returned struct is bit-identical to the oracle's in every case. Only a log line is added.

</details>

### `src/ffi.cyr`

| Function | Parameter | Accepted range | Rejects with |
|---|---|---|---|
| `prani_ffi_voice_set_effort` | `effort` | finite (any IEEE-754 real); finite out-of-range still CLAMPS to [0,1] | `no error channel exists (Rust returned (), the port returns a constant 0) — rejection is expressed as a total no-op: the voice is byte-for-byte what the last accepted call left it, observable via crvoice_vocal_effort` |
| `prani_ffi_voice_set_size` | `size` | finite; finite out-of-range still CLAMPS to [0.1, 10] | `no error channel — no-op, voice unchanged; observable via CreatureVoice_size_scale and crvoice_effective_f0` |
| `prani_ffi_voice_apply_lombard` | `ambient_spl_db` | finite — ANY finite dB SPL is accepted, no magnitude bound | `no error channel — no-op, voice unchanged; observable via crvoice_vocal_effort` |
| `prani_ffi_stream_start` | `sample_rate` | (0, +inf) — finite and strictly positive | `0 (null handle) — this function's own documented error return, 'Returns a null pointer on error (invalid voice, unsupported vocalization, etc.)'. No PRANI_ERR_* is used: the FFI convention here is handle-or-0, not negative codes.` |
| `prani_ffi_stream_start` | `duration` | [0, +inf) — finite and non-negative; ZERO IS VALID | `0 (null handle) — same handle-or-0 convention as above` |
| `prani_ffi_stream_fill` | `buffer_len` | buffer_len >= 0 — a sample count cannot be negative | `0 (count written) — the same value the function already returns on a null handle, a null buffer, a zero length, or a finished stream. Per ADR-0003 the stream is NOT retired: a bad length is a caller mistake, not a property of the stream, so a corrected call still drains it (asserted).` |

<details><summary>Deliberate divergences from the Rust oracle in this module</summary>

- **`prani_ffi_stream_start` / `sample_rate`** — Rust's `as usize` cast saturates, so the oracle folded a negative or NaN rate into total_samples = 0 and returned a stream that produced an empty buffer. The port now returns null instead. Deliberate: the oracle could not represent the state (usize), i64 can, and f64_to wraps rather than saturates.
- **`prani_ffi_stream_start` / `duration`** — DELIBERATE: the oracle accepted a negative duration (usize saturation folded it into 0 and handed back a stream). The port rejects it with null. Justification: i64 can represent the state the oracle could not, and f64_to wraps rather than saturates, so the port would otherwise build a stream whose total_samples is large and negative and whose finished-ness is an accident of a signed compare rather than a real zero. A host that asks for -1 seconds has a bug (ADR-0001).

</details>

### `src/preset.cyr`

| Function | Parameter | Accepted range | Rejects with |
|---|---|---|---|
| `preset_from_json_str` | `name` | must be a JSON string node (present + string-typed) | `PRANI_ERR_INVALID_TRACT_PLACEHOLDER_NOT_USED -> PRANI_ERR_INVALID_SPECIES` |
| `preset_from_json_str` | `species` | integer node, [0, PRANI_SP_COUNT) i.e. 0..12 | `PRANI_ERR_INVALID_SPECIES` |
| `preset_from_json_str` | `size` | integer node, f64 in [0.1, 10.0] | `PRANI_ERR_INVALID_SPECIES` |
| `preset_from_json_str` | `f0_offset` | integer node, f64 in +/-(f0_max - f0_min) FOR THE DECLARED SPECIES (Wolf +/-1050, Cricket +/-5000, Lion +/-160, ...) | `PRANI_ERR_INVALID_SPECIES` |
| `preset_from_json_str` | `breathiness` | integer node, f64 in [0.0, 1.0] | `PRANI_ERR_INVALID_SPECIES` |

<details><summary>Deliberate divergences from the Rust oracle in this module</summary>

- **`preset_from_json_str` / `size`** — PARTIAL divergence, deliberate. NaN/inf: restoration - JSON has no NaN/Infinity literal and serde_json rejects them, so the oracle could not carry a non-finite size through its codec at all (the port can, because the wire format is the f64 bit pattern as an integer). Finite out-of-range (e.g. 20.0): a real divergence - the oracle deserialized it and with_size clamped to 10.0. Rejected per decision 1 + ADR-0001.
- **`preset_from_json_str` / `f0_offset`** — Same split as size: non-finite is a restoration (unrepresentable in the oracle's JSON), finite-out-of-range is the deliberate divergence (the oracle clamped).
- **`preset_from_json_str` / `breathiness`** — Same split as size: non-finite restoration, finite-out-of-range deliberate divergence.

</details>

### `src/sequence.cyr`

| Function | Parameter | Accepted range | Rejects with |
|---|---|---|---|
| `sequence_call_element_new` | `duration` | finite, [0, 3600] s (PRANI_SEQ_MAX_SECONDS) | `PRANI_ERR_INVALID_VOCALIZATION` |
| `sequence_call_element_new` | `gap` | finite, [0, 3600] s | `PRANI_ERR_INVALID_VOCALIZATION` |
| `sequence_call_bout_new` | `count` | [0, 1024] (PRANI_SEQ_MAX_COUNT) | `PRANI_ERR_INVALID_VOCALIZATION` |
| `sequence_call_bout_new` | `call_duration` | finite, [0, 3600] s | `PRANI_ERR_INVALID_VOCALIZATION` |
| `sequence_call_bout_new` | `interval` | finite, [0, 3600] s | `PRANI_ERR_INVALID_VOCALIZATION` |
| `sequence_call_bout_synthesize` | `sample_rate` | finite, > 0 (no upper bound) | `PRANI_ERR_INVALID_TRACT` |
| `sequence_call_bout_synthesize` | `CallBout_count(self)` | [0, 1024] | `PRANI_ERR_INVALID_VOCALIZATION` |
| `sequence_call_bout_synthesize` | `CallBout_call_duration(self) / CallBout_interval(self)` | finite, [0, 3600] s each | `PRANI_ERR_INVALID_VOCALIZATION` |
| `sequence_call_bout_synthesize` | `count * (call_duration + interval) * sample_rate` | [0, 16777216] samples (PRANI_SEQ_MAX_SAMPLES = 2^24) | `PRANI_ERR_INVALID_VOCALIZATION` |
| `sequence_call_phrase_synthesize` | `sample_rate` | finite, > 0 | `PRANI_ERR_INVALID_TRACT` |
| `sequence_call_phrase_synthesize` | `vec_len(elements)` | [0, 1024] | `PRANI_ERR_INVALID_VOCALIZATION` |
| `sequence_call_phrase_synthesize` | `each element's duration and gap` | finite, [0, 3600] s each | `PRANI_ERR_INVALID_VOCALIZATION` |
| `sequence_call_phrase_synthesize` | `sum(duration + gap) * sample_rate` | [0, 16777216] samples | `PRANI_ERR_INVALID_VOCALIZATION` |
| `sequence_synthesize_chorus` | `sample_rate` | finite, > 0 | `PRANI_ERR_INVALID_TRACT` |
| `sequence_synthesize_chorus` | `duration` | finite, [0, 3600] s | `PRANI_ERR_INVALID_VOCALIZATION` |
| `sequence_synthesize_chorus` | `timing_spread` | finite, [0, 3600] s | `PRANI_ERR_INVALID_VOCALIZATION` |
| `sequence_synthesize_chorus` | `vec_len(voices)` | [0, 1024] | `PRANI_ERR_INVALID_VOCALIZATION` |
| `sequence_synthesize_chorus` | `(duration + 2 * timing_spread) * voice_count * sample_rate` | [0, 16777216] samples | `PRANI_ERR_INVALID_VOCALIZATION` |
| `sequence_call_phrase_from_json_str` | `intent` | [0, PRANI_INTENT_COUNT) = [0, 7) | `PRANI_ERR_INVALID_VOCALIZATION` |
| `sequence_call_phrase_from_json_str` | `each element's vocalization` | [0, PRANI_VOC_COUNT) = [0, 14) | `PRANI_ERR_INVALID_VOCALIZATION` |
| `sequence_call_phrase_from_json_str` | `each element's duration and gap` | finite, [0, 3600] s | `PRANI_ERR_INVALID_VOCALIZATION` |
| `sequence_call_phrase_from_json_str` | `elements array length` | [0, 1024] | `PRANI_ERR_INVALID_VOCALIZATION` |

<details><summary>Deliberate divergences from the Rust oracle in this module</summary>

- **`sequence_call_element_new` / `duration`** — Divergence. Rust's f32 duration accepted NaN, inf, negative and 1e9 alike; the saturating `as usize` turned them into 0 samples. The oracle could not report them because Self construction was infallible.
- **`sequence_call_element_new` / `gap`** — Same divergence as duration.
- **`sequence_call_bout_new` / `count`** — Negative count: the oracle's `count: u32` could not represent it at all. Upper bound 1024: no oracle counterpart — Rust would have looped 4 billion times.
- **`sequence_call_bout_new` / `call_duration`** — Same as element duration.
- **`sequence_call_bout_new` / `interval`** — Same as element gap.
- **`sequence_call_bout_synthesize` / `sample_rate`** — No divergence in outcome (an invalid rate was already an error); the code moved from whatever the tract layer reported to INVALID_TRACT, and the report now arrives before any synthesis.
- **`sequence_call_bout_synthesize` / `count * (call_duration + interval) * sample_rate`** — Divergence: Rust attempted the allocation and would OOM/abort. Nothing in the oracle bounded it.
- **`sequence_call_phrase_synthesize` / `vec_len(elements)`** — Divergence; the oracle's Vec<CallElement> had no length bound.
- **`sequence_synthesize_chorus` / `timing_spread`** — Divergence: Rust's f32 spread accepted NaN and 1e6 alike; `(NaN * sr) as usize` saturated to 0, so the oracle's chorus also dropped the spread silently. Reporting it is new.
- **`sequence_synthesize_chorus` / `(duration + 2 * timing_spread) * voice_count * sample_rate`** — Divergence: `vec![0.0f32; total_len]` in Rust would attempt the allocation and abort.
- **`sequence_call_phrase_from_json_str` / `intent`** — RESTORES the oracle: serde deserializing the CallIntent enum rejects an unknown discriminant. The port's bayan codec had silently accepted any integer.
- **`sequence_call_phrase_from_json_str` / `each element's vocalization`** — RESTORES the oracle (serde rejects an unknown Vocalization variant).
- **`sequence_call_phrase_from_json_str` / `each element's duration and gap`** — Divergence: serde would have accepted any f32, NaN included.
- **`sequence_call_phrase_from_json_str` / `elements array length`** — Divergence; serde had no length bound.

</details>

### `src/spatial.cyr`

| Function | Parameter | Accepted range | Rejects with |
|---|---|---|---|
| `spatial_apply_distance_attenuation` | `samples` | a live vec handle (> 0) | `PRANI_ERR_INVALID_VOCALIZATION` |
| `spatial_apply_distance_attenuation` | `distance` | finite, >= 0 metres (no upper bound) | `PRANI_ERR_INVALID_VOCALIZATION` |
| `spatial_apply_distance_attenuation` | `reference_distance` | finite, > 0 metres (no upper bound) | `PRANI_ERR_INVALID_VOCALIZATION` |
| `spatial_apply_distance_attenuation` | `sample_rate` | finite, > 0 Hz (no upper bound) | `PRANI_ERR_INVALID_TRACT` |
| `spatial_apply_distance_attenuation` | `sample_rate (derived: dt = 1/sample_rate)` | 1/sample_rate must be finite, i.e. sample_rate >~ 5.6e-309 | `PRANI_ERR_INVALID_TRACT` |
| `spatial_apply_doppler_shift` | `samples` | a live vec handle (> 0) | `PRANI_ERR_INVALID_VOCALIZATION` |
| `spatial_apply_doppler_shift` | `velocity` | ANY finite value -- explicitly not [-308.7, +308.7] | `PRANI_ERR_INVALID_VOCALIZATION` |

<details><summary>Deliberate divergences from the Rust oracle in this module</summary>

- **`spatial_apply_distance_attenuation` / `distance`** — Negative distance: the oracle's distance.max(reference_distance) silently clamped -5 m up to the reference and returned full-volume audio (measured bit-equal to the distance-1.0 result). Rejecting it is deliberate -- a negative distance is not a distance, and the clamp's documented job is 'don't blow up inside the reference radius', not 'accept sign errors'.
- **`spatial_apply_distance_attenuation` / `reference_distance`** — The f32 oracle accepted all four and produced those buffers. Deliberate divergence: none of them is the audio the caller asked for.
- **`spatial_apply_distance_attenuation` / `sample_rate`** — The oracle accepted 0 / negative / non-finite rates and returned the NaN, overshooting or silent buffers above.
- **`spatial_apply_distance_attenuation` / `sample_rate (derived: dt = 1/sample_rate)`** — f32's smallest subnormal is 1e-45 and 1/1e-45 also overflows, so the oracle had the same hole; it is unreachable in practice either way.
- **`spatial_apply_doppler_shift` / `velocity`** — f32 could hold NaN and inf too, so the oracle had the same empty-buffer and silent-saturation behaviour; reporting them is the divergence.

</details>

### `src/species.cyr`

| Function | Parameter | Accepted range | Rejects with |
|---|---|---|---|
| `species_params_in_range (via species_params_from_node / species_params_validate)` | `apparatus` | integer 0..4 (PRANI_APP_LARYNGEAL..PRANI_APP_NOISE_ONLY) | `PRANI_ERR_INVALID_SPECIES` |
| `species_params_in_range` | `f0_min` | finite, [0.0, 20000.0] Hz | `PRANI_ERR_INVALID_SPECIES` |
| `species_params_in_range` | `f0_max` | finite, [0.0, 20000.0] Hz | `PRANI_ERR_INVALID_SPECIES` |
| `species_params_in_range` | `f0_default` | finite, [0.0, 20000.0] Hz | `PRANI_ERR_INVALID_SPECIES` |
| `species_params_in_range` | `f0_min / f0_default / f0_max ordering` | f0_min <= f0_default <= f0_max | `PRANI_ERR_INVALID_SPECIES` |
| `species_params_in_range` | `tract_scale` | finite, (0.0, 100.0] - strictly positive | `PRANI_ERR_INVALID_SPECIES` |
| `species_params_in_range` | `formant0, formant1, formant2` | finite, (0.0, 20000.0] Hz each | `PRANI_ERR_INVALID_SPECIES` |
| `species_params_in_range` | `bandwidth0, bandwidth1, bandwidth2` | finite, (0.0, 20000.0] Hz each | `PRANI_ERR_INVALID_SPECIES` |
| `species_params_in_range` | `breathiness` | finite, [0.0, 1.0] | `PRANI_ERR_INVALID_SPECIES` |
| `species_params_in_range` | `jitter` | finite, [0.0, 0.05] | `PRANI_ERR_INVALID_SPECIES` |
| `species_params_in_range` | `shimmer` | finite, [0.0, 0.1] | `PRANI_ERR_INVALID_SPECIES` |
| `species_params_in_range` | `spectral_tilt` | finite only - NO magnitude bound, deliberately | `PRANI_ERR_INVALID_SPECIES` |
| `species_params_from_node` | `the whole parsed node (return contract)` | returns the SpeciesParams pointer, or NEGATIVE PRANI_ERR_INVALID_SPECIES when species_params_in_range fails | `PRANI_ERR_INVALID_SPECIES` |

<details><summary>Deliberate divergences from the Rust oracle in this module</summary>

- **`species_params_in_range` / `f0_min`** — The 20000 Hz ceiling is a DELIBERATE NARROWING - Rust f32 held any value and nothing rejected it. Non-finite and negative are not a divergence in spirit: the oracle's f32::clamp panics on a NaN bound.
- **`species_params_in_range` / `f0_max`** — Ceiling is a deliberate narrowing; see f0_min.
- **`species_params_in_range` / `f0_default`** — Ceiling is a deliberate narrowing; see f0_min.
- **`species_params_in_range` / `tract_scale`** — DELIBERATE DIVERGENCE. Rust f32 could hold 0.0 and negatives and nothing in the oracle rejected them - but the oracle also never constructed one, and 0.0 is the fabricated-zero case ADR-0002 named. NOTE: the task brief's premise that tract_scale is a divisor is INCORRECT and I did not rest the guard on it - I checked every f64_div in src/ and tract_scale is never a denominator; its only prani consumer multiplies it (voice.cyr:250).
- **`species_params_in_range` / `formant0, formant1, formant2`** — Both bounds are a deliberate narrowing - Rust's `formants: [f32; 3]` was unconstrained.
- **`species_params_in_range` / `bandwidth0, bandwidth1, bandwidth2`** — Deliberate narrowing; Rust's `bandwidths: [f32; 3]` was unconstrained.
- **`species_params_from_node` / `the whole parsed node (return contract)`** — Contract change in the ADR-0002 direction: Rust's serde_json returned Result here too, so 'pointer or negative code' RESTORES the oracle's fallibility rather than diverging. Both call sites already check - verified in the tree: src/tract.cyr:302-305 propagates prani_is_err(params) before crtract_new, and src/voice.cyr:251-252 routes through crvoice_params_in_range, which rejects p == 0 and prani_is_err(p) before any dereference.

</details>

### `src/stream.cyr`

| Function | Parameter | Accepted range | Rejects with |
|---|---|---|---|
| `stream_new` | `voice` | > 0 (a live heap handle) | `PRANI_ERR_INVALID_TRACT_NOT_USED__PRANI_ERR_INVALID_SPECIES` |
| `stream_new` | `sample_rate` | finite and > 0; NO upper bound and no lower bound of prani's own | `PRANI_ERR_INVALID_TRACT` |
| `stream_new` | `duration` | finite and >= 0 seconds (ZERO IS VALID) | `PRANI_ERR_INVALID_VOCALIZATION` |
| `stream_new` | `duration * duration_scale * sample_rate (the derived sample count)` | [0, 2^53] == [0, 9007199254740992] | `PRANI_ERR_INVALID_VOCALIZATION` |
| `stream_new` | `f0 / effort_amp / spectral_tilt (the voice snapshots)` | all three finite | `PRANI_ERR_INVALID_SPECIES` |
| `stream_fill_buffer` | `self, buffer (handles)` | both > 0 | `0 written, stream NOT retired` |
| `stream_fill_buffer` | `contour (point-of-use, derived from f0)` | finite | `0 written + stream retired (ADR-0003's arm verbatim)` |
| `stream_next_block` | `self (handle)` | > 0 | `PRANI_ERR_INVALID_VOCALIZATION` |
| `stream_next_block` | `block_size` | >= 0; no upper bound | `PRANI_ERR_INVALID_VOCALIZATION` |
| `stream_pitch_contour_at` | `t` | any finite value (clamped to [0,1] as before); NaN propagates as NaN | `none - NaN is propagated, NOT an error code` |

<details><summary>Deliberate divergences from the Rust oracle in this module</summary>

- **`stream_new` / `voice`** — Rust took CreatureVoice by value; a null/dangling voice was unrepresentable. Guard has no oracle counterpart and rejects nothing the oracle accepted.
- **`stream_new` / `duration`** — DELIBERATE: a NEGATIVE duration is now rejected. Rust's float->usize `as` cast saturates a negative product to 0, so the oracle returned a silent empty stream; Cyrius's f64_to keeps the sign, so the port would hand the host total_samples == -44100. Reporting beats both.
- **`stream_new` / `duration * duration_scale * sample_rate (the derived sample count)`** — The oracle could not reach this state the same way: `as usize` saturates to usize::MAX (a 64-bit unsigned count) rather than to a negative. Rejecting is a divergence from 'silently enormous' to 'reported'.
- **`stream_new` / `f0 / effort_amp / spectral_tilt (the voice snapshots)`** — f32 could carry NaN identically; the oracle had no guard, so this is a divergence from 'NaN audio' to 'reported'. It rejects only inputs the oracle would have turned into NaN samples.
- **`stream_fill_buffer` / `contour (point-of-use, derived from f0)`** — The oracle handed an inf f0 to svara and applied tilt+amp to a stale buffer while returning to_render - the exact defect ADR-0003 already fixed for the error arm.
- **`stream_next_block` / `block_size`** — DELIBERATE, and only possible in the port: the oracle's block_size was `usize`, so a negative one could not be expressed. Also a return-CONTRACT extension (no signature change): stream_next_block now returns 'vec handle or negative PRANI_ERR_*', which is the convention crvoice_vocalize already uses and tests already check with prani_is_err. Documented in the function's header comment. No caller anywhere in src/ or tests/ passes a negative block_size, and the FFI does not call it at all.
- **`stream_pitch_contour_at` / `t`** — Rust returned base_f0 * 0.6 for a NaN t as well (same fallthrough). Propagating NaN instead of a plausible f0 is a deliberate divergence for invalid input only.

</details>

### `src/tract.cyr`

| Function | Parameter | Accepted range | Rejects with |
|---|---|---|---|
| `crtract_synthesize_laryngeal` | `f0` | finite (any finite value; the existing clamp to [max(f0_min,20), min(f0_max,2000)] bounds it) | `PRANI_ERR_INVALID_TRACT` |
| `crtract_synthesize_laryngeal` | `num_samples` | >= 0 (0 is VALID) | `PRANI_ERR_INVALID_VOCALIZATION` |
| `crtract_synthesize_laryngeal` | `subharmonic_amp, perturbation_scale (SynthesisOptions)` | finite | `PRANI_ERR_INVALID_VOCALIZATION` |
| `crtract_synthesize_syringeal` | `f0` | finite | `PRANI_ERR_INVALID_TRACT` |
| `crtract_synthesize_syringeal` | `num_samples` | >= 0 (0 is VALID) | `PRANI_ERR_INVALID_VOCALIZATION` |
| `crtract_synthesize_syringeal` | `subharmonic_amp, perturbation_scale (SynthesisOptions)` | finite | `PRANI_ERR_INVALID_VOCALIZATION` |
| `crtract_synthesize_noise` | `num_samples` | >= 0 (0 is VALID) | `PRANI_ERR_INVALID_VOCALIZATION` |
| `crtract_synthesize_stridulatory` | `f0` | finite | `PRANI_ERR_INVALID_TRACT` |
| `crtract_synthesize_stridulatory` | `num_samples` | >= 0 (0 is VALID) | `PRANI_ERR_INVALID_VOCALIZATION` |
| `crtract_synthesize_vibratile` | `f0` | finite | `PRANI_ERR_INVALID_TRACT` |
| `crtract_synthesize_vibratile` | `num_samples` | >= 0 (0 is VALID) | `PRANI_ERR_INVALID_VOCALIZATION` |
| `crtract_synthesize_purr` | `purr_f0` | finite | `PRANI_ERR_INVALID_TRACT` |
| `crtract_synthesize_purr` | `num_samples` | >= 0 (0 is VALID) | `PRANI_ERR_INVALID_VOCALIZATION` |
| `crtract_apply_spectral_tilt` | `tilt_db` | finite | `PRANI_ERR_INVALID_TRACT` |
| `crtract_from_json_str` | `phase` | [0, 1] inclusive | `PRANI_ERR_INVALID_TRACT` |
| `crtract_from_json_str` | `dcb_x_prev, dcb_y_prev` | finite | `PRANI_ERR_INVALID_TRACT` |
| `crtract_from_json_str` | `dcb_r` | [0, 1) -- non-negative and strictly less than 1 | `PRANI_ERR_INVALID_TRACT` |
| `crtract_from_json_str` | `params (nested SpeciesParams node)` | propagates species.cyr's own 2.0.5 verdict | `propagated (PRANI_ERR_INVALID_SPECIES from species.cyr)` |

<details><summary>Deliberate divergences from the Rust oracle in this module</summary>

- **`crtract_synthesize_laryngeal` / `f0`** — Behaviourally none on this path: MEASURED pre-guard, a NaN f0 already came back as an error here (the clamp passes NaN through and svara_glottal_new refuses it). Only the code changes, PRANI_ERR_SVARA -> PRANI_ERR_INVALID_TRACT, which is the more accurate category (prani's own parameter, not a svara failure). f0 = 0 and negative f0 stay VALID as in the oracle.
- **`crtract_synthesize_laryngeal` / `num_samples`** — The oracle's num_samples was usize and could not be negative, so this is a state the port can reach and the Rust could not. Zero is NOT rejected: the oracle's test_zero_duration_synthesis asserts an empty buffer, not an error.
- **`crtract_synthesize_laryngeal` / `subharmonic_amp, perturbation_scale (SynthesisOptions)`** — The oracle accepted a NaN f32 here and produced the same silently-degraded block. Deliberate divergence: report rather than fabricate (ADR-0001's rule).
- **`crtract_synthesize_syringeal` / `num_samples`** — usize could not be negative; the port can. Zero stays valid.
- **`crtract_synthesize_syringeal` / `subharmonic_amp, perturbation_scale (SynthesisOptions)`** — Same as the laryngeal options guard: the oracle accepted NaN and degraded silently.
- **`crtract_synthesize_noise` / `num_samples`** — usize could not be negative.
- **`crtract_synthesize_stridulatory` / `num_samples`** — usize could not be negative.
- **`crtract_synthesize_vibratile` / `num_samples`** — usize could not be negative.
- **`crtract_synthesize_purr` / `purr_f0`** — CONTRACT CHANGE: this function was documented 'never fails' and now returns a negative code. The oracle's Result<Vec> shape accommodates it, but two in-tree call sites did not check -- both are provably safe today (see notes), and voice.cyr has since added upstream guards that make its arguments unreachable.
- **`crtract_synthesize_purr` / `num_samples`** — usize could not be negative; zero stays valid per test_zero_duration_synthesis.
- **`crtract_apply_spectral_tilt` / `tilt_db`** — The Rust returned () and had no error channel at all; the port already returned 0, so this uses the existing '0 or negative' return with no signature change. Rejecting +-inf is a deliberate divergence -- the oracle silently no-opped (+inf) or silently applied maximum lowpass (-inf).
- **`crtract_from_json_str` / `phase`** — serde_json would have accepted any f32 the document named; the oracle had no field validation at all (ADR-0002 deferred this to 2.0.5). KNOWN EDGE, documented in the source rather than hidden: a purr driven at purr_f0 > sample_rate (phase_inc > 1) can leave phase above 1, and such a tract's own json would be rejected on read. No in-tree path reaches it -- voice.cyr clamps purr_f0 to [20,35], stream.cyr passes the constant 27.
- **`crtract_from_json_str` / `dcb_x_prev, dcb_y_prev`** — The oracle validated nothing here.
- **`crtract_from_json_str` / `dcb_r`** — The oracle validated nothing here. Note a document that simply OMITS dcb_r reads as 0 and is accepted (r = 0 is stable); missing-field detection is field-presence, not range, and is out of this milestone's scope.

</details>

### `src/voice.cyr`

| Function | Parameter | Accepted range | Rejects with |
|---|---|---|---|
| `crvoice_vocalize_with_intent` | `voc` | integer in [0, PRANI_VOC_COUNT) | `PRANI_ERR_INVALID_VOCALIZATION` |
| `crvoice_vocalize_with_intent` | `intent` | integer in [0, PRANI_INTENT_COUNT) | `PRANI_ERR_INVALID_VOCALIZATION` |
| `crvoice_vocalize_with_intent` | `sample_rate` | finite and > 0 (NO upper or lower threshold) | `PRANI_ERR_INVALID_TRACT` |
| `crvoice_vocalize_with_intent` | `duration` | finite and >= 0 (zero is VALID) | `PRANI_ERR_INVALID_VOCALIZATION` |
| `crvoice_vocalize_with_intent` | `num_samples (derived: effective_duration * sample_rate)` | >= 0 | `PRANI_ERR_INVALID_VOCALIZATION` |
| `crvoice_vocalize_with_intent` | `self's params (all 16 SpeciesParams fields, via crvoice_params_in_range)` | apparatus [0,4]; f0_min/f0_max finite >= 0 with f0_min <= f0_max; f0_default in [f0_min, f0_max]; tract_scale finite > 0; formant0..2 and bandwidth0..2 finite > 0; breathiness [0,1]; jitter [0,0.05]; shimmer [0,0.1]; spectral_tilt finite | `PRANI_ERR_INVALID_SPECIES` |
| `crvoice_vocalize_with_intent` | `crvoice_effective_f0(self) and crvoice_effective_tract_scale(self)` | both finite | `PRANI_ERR_INVALID_TRACT` |
| `crvoice_synthesize_cat_purr` | `sample_rate` | finite and > 0 | `PRANI_ERR_INVALID_TRACT` |
| `crvoice_synthesize_cat_purr` | `num_samples` | >= 0 (zero is VALID) | `PRANI_ERR_INVALID_VOCALIZATION` |
| `crvoice_synthesize_cat_purr` | `self's size_scale` | finite and > 0 | `PRANI_ERR_INVALID_SPECIES` |
| `crvoice_apply_nasal_antiformant` | `iteration bound nasal_len (LIVE CRASH)` | min(nasal_len, len), clamped in f64 before the f64_to | `n/a -- parity repair, not a rejection` |
| `crvoice_apply_nasal_antiformant` | `sample_rate` | finite and > 0 | `PRANI_ERR_INVALID_TRACT` |
| `crvoice_apply_nasal_antiformant` | `nasal_fraction` | finite and >= 0 | `PRANI_ERR_INVALID_VOCALIZATION` |
| `crvoice_apply_am_pattern` | `sample_rate` | finite and > 0 | `PRANI_ERR_INVALID_TRACT` |
| `crvoice_from_json_str` | `species` | integer in [0, PRANI_SP_COUNT) | `PRANI_ERR_INVALID_SPECIES` |
| `crvoice_from_json_str` | `params (all 16 nested SpeciesParams fields, via crvoice_params_in_range)` | the same table as the vocalize guard above -- ONE helper serves both call sites so the deserializer's gate and the point-of-use gate cannot drift apart | `PRANI_ERR_INVALID_SPECIES` |
| `crvoice_from_json_str` | `f0_offset` | [-(f0_max - f0_min), +(f0_max - f0_min)], read off the params just validated | `PRANI_ERR_INVALID_SPECIES` |
| `crvoice_from_json_str` | `size_scale` | [0.1, 10.0] | `PRANI_ERR_INVALID_SPECIES` |
| `crvoice_from_json_str` | `vocal_effort` | [0, 1] | `PRANI_ERR_INVALID_SPECIES` |

<details><summary>Deliberate divergences from the Rust oracle in this module</summary>

- **`crvoice_vocalize_with_intent` / `voc`** — DELIBERATE. The oracle's Vocalization was an enum -- 99 was unrepresentable, so it never had to decide. The port carries the tag as i64. Rejecting is forced by the enum->int erasure.
- **`crvoice_vocalize_with_intent` / `intent`** — DELIBERATE, same enum->int erasure as voc.
- **`crvoice_vocalize_with_intent` / `duration`** — DELIBERATE. Rust's saturating `as usize` turned NaN and negatives into 0, so the oracle also returned an empty buffer as a success. We report instead of fabricating a plausible empty answer (ADR-0001's philosophy). For +inf the oracle saturated to usize::MAX and would have tried to render it; both behaviours are bugs.
- **`crvoice_vocalize_with_intent` / `num_samples (derived: effective_duration * sample_rate)`** — DELIBERATE, same class as the duration guard.
- **`crvoice_apply_nasal_antiformant` / `nasal_fraction`** — DELIBERATE but inert. Rust's saturating cast made NaN and negatives 0, so the oracle also no-opped. We report.

</details>

**109 guards across 11 modules.**

## Not guarded, and why

The 7 `crvoice_with_*` builders return `self` with no error channel, so rejecting a NaN
there means making them fallible — an API break every consumer must handle. Roadmap
**2.1.0** already has to put a fallible return on every constructor for the
allocation-failure contract, so the signature change happens once, there. Until then a
NaN smuggled in through a builder is caught at the **point of use**: the fallible
function that consumes it checks the voice's own derived values.

**`src/dsp.cyr`**

- dcblocker_process(b, x) — x: any f64 INCLUDING non-finite. NOT guarded, for two reasons. COST: this is the module's hot path at 19 ns/sample (docs/benchmarks.md); prani_is_finite is an f64_abs + f64_lt, so a per-sample guard would be a large fraction of the work it protects, on every sample of every synthesis — the brief rules this out and I agree. PLACE: x is never host data. It is prani's own tract output, so a non-finite x means a host number (sample_rate, f0, velocity) already got through an entry point upstream — that is where the range guard belongs. It also has no error channel to report through: every f64 bit pattern is a legal sample, so a negative return is not representable.
- dcblocker_process_buffer(b, samples) — no error return added. THE GUARD BELONGS AT THE CALLERS (crtract_synthesize* in tract.cyr, which take the host's sample_rate and f0 and already return value-or-negative-code). This function has never had an error channel, and inventing one would be the signature change 2.0.5 forbids — and it would be inert anyway, since I cannot touch tract.cyr to make anyone read it. A latch DETECTOR was added instead (see notes).
- prani_vec_extend(dst, src) — there is no range here to check. It copies opaque f64 bit patterns and never interprets one as a number. An O(n) non-finite scan would add a pass over the audio to the block-accumulation path (voice.cyr:905) to reject a value the entry point should have refused before any of it was synthesized: wrong layer, at a cost proportional to the buffer. Pinned with a test asserting a NaN element is copied VERBATIM, so the decision is visible rather than an omission.
- prani_vec_extend's OTHER hazard (src being a negative error code rather than a vec, which would deref a small negative address) — deliberately out of scope: that is F1's class, not an input range, and all three call sites check prani_is_err before extending.
- prani_vec_push_zeros — no upper bound on n. A cap would be an arbitrary magic number the oracle never had (Vec/usize), and F10's overflow is an arithmetic bug at its own site in sequence.cyr, not a range violation here. The negative-n guard already catches F10's observable symptom, because the overflow wraps negative.
- prani_map_naad_error(naad_code) — naad_code is a foreign error code, not a numeric range. Unchanged.
- DcBlocker_from_json_str (derive-generated) — noted, not guarded. It cannot validate r (an unstable pole) or a NaN field, but DcBlocker was pub(crate) in the oracle and this module is crate-private: the type is unreachable from the FFI and from every public entry point, and adding a validating wrapper would be new API surface, not a guard on an existing one.

**`src/emotion.cyr`**

- emotion_with_values(valence, arousal) - returns the struct, no error channel. Per decision 2 the builder is left alone (roadmap 2.1.0 constructor contract); the NaN it can smuggle in is caught at the point of use by all five guards above, asserted in tests/emotion.tcyr.
- emotion_with_smoothing(self, smoothing) - same: returns self. Its clamped NaN is caught by emotion_evaluate (asserted), and the one update() tick it survives turns valence NaN, which the zone guards then reject (also asserted).
- emotion_set(self, valence, arousal) and emotion_update(self, tv, ta) - neither has a return statement, so neither already returns "value or negative code". Adding one would be worse than the status quo: no caller checks a return here, so a rejection would silently become a no-op mutation - a new silent failure in place of the old one. Their clamped NaNs are caught at the point of use instead (a NaN via set() is asserted).
- emotion_valence(self) / emotion_arousal(self) - return a raw f64 bit pattern. Every i64 is a legal f64, and a negative return is a legitimate negative valence, so there is no error channel to use without a signature change.
- emotion_new() - takes no input.
- PrEmotion_from_json_str / PrEmotion_set_* - derive-generated, not written by me and not in my two files. They are the only doors an out-of-range (as opposed to NaN) value can come through, which is precisely why the range test lands at the consumer. A foreign document with valence 5.0 parsing unclamped and then being rejected by all four checkable entry points is asserted.

**`src/fatigue.cyr`**

- fatigue_fatigue / fatigue_alarm_habituation — getters returning a raw f64 bit pattern. Every i64 is a legal f64, so there is no value space left to carry a negative PRANI_ERR_*. Cannot be guarded without a signature change.
- fatigue_new / fatigue_reinforce_alarm / fatigue_clear_reinforcement / fatigue_reset — take no numeric input; nothing to range-check.
- fatigue_record_call's `is_alarm` — a 0/1 bool, not a float. `if (is_alarm != 0)` treats any nonzero as true, which is what the Rust `bool` parameter meant after conversion. Constraining it would be a divergence with no defect behind it.
- FatigueState_from_json_str / FatigueModifiers_from_json_str — derive-generated by #derive(Serialize) and not in a file I own. They accept ANY i64 per field, including NaN and inf bit patterns, so a document prani did not write can still rebuild a poisoned FatigueState (verified: a well-formed doc with fatigue = 9221120237041090560 parses, and modifiers() off it is still NaN). This is the deserializer-field-range half of milestone 2.0.5 and it belongs with the derive. Pinned as a known gap in the last test group so it is not believed fixed; the modifiers_make log line is the only report available from this module.
- fatigue_modifiers — cannot return a PRANI_ERR_* (pointer-only contract). Its poisoned-state path is covered by the modifiers_make log rather than by a second check, since every non-finite state field propagates into at least one of the four derived values (verified for NaN and for ±inf fatigue/habituation).

**`src/ffi.cyr`**

- FINITE out-of-range values on all three RTPC setters (effort 1.5, size 100, ambient 200 dB) — they still CLAMP, unchanged. This is the oracle's documented RTPC saturation, tests/ffi.tcyr already pins 'set_effort(1.5) clamps to 1.0' as an existing assertion, and hard rule 2 forbids changing valid-input behaviour. Rejecting a finite out-of-range builder argument is roadmap 2.1.0's builder/constructor-contract work.
- A finite, positive sample_rate BELOW svara's ~1000 Hz floor (e.g. 999.0) — deliberately still accepted by prani_ffi_stream_start. ADR-0001 rejected duplicating svara's threshold in prani because the duplicate is what goes stale. The rejection surfaces at the first fill per ADR-0003 (0 written, stream retires), and a new test group asserts that whole path end-to-end so the range guard cannot swallow it.
- A buffer_len that DISAGREES with vec_len(buffer) in either direction (e.g. buffer_len 64 against a 2205-element vec still writes and reports 2205; buffer_len 99999 against a 64-element vec writes and reports 64). This is not a range violation but a disagreement between two length sources, and the module header already documents the deviation that 'the vec's own length is the real bound' with buffer_len retained for C signature parity. Reconciling them would change that documented contract rather than validate a range — out of scope for 2.0.5. Worth noting for a future milestone: at a real C ABI the buffer_len < vec_len direction is the overrun case, and in the Cyrius port it is safe only because the vec is the allocation.
- species_index / voc_index / intent_index on prani_ffi_voice_create and prani_ffi_stream_start — already fully range-checked by the three *_from_index helpers (including the negative indices the oracle's u32 could not represent), with existing assertions.
- prani_ffi_stream_is_finished, prani_ffi_voice_destroy, prani_ffi_stream_destroy — no numeric parameters beyond the handle, which each already null-checks.

**`src/preset.cyr`**

- preset_make(name, species, size, f0_offset, breathiness) - returns a VoicePreset with NO error channel, the same shape as the crvoice_with_* builders it feeds. Guarding it would require inventing a channel, which decision 2 excludes and roadmap 2.1.0 owns. Its unchanged contract is PINNED in the test file (preset_make with a NaN size still returns a preset and stores the NaN verbatim) so 2.1.0 has to notice it is changing this.
- preset_build(self) - the point of use the brief flagged, but it returns a CreatureVoice and nothing else, so it also has no error channel; giving it one is the same 2.1.0 constructor-contract change. A preset from the codec can no longer carry an out-of-range field; a preset assembled by hand through preset_make still can, and its NaN passes every f64_clamp in the chain (F11). That remaining path to svara is closed at the fallible consumers downstream (crvoice_vocalize / the tract constructor, both of which return PRANI_ERR_*) - voice.cyr's and tract.cyr's files, not mine. Reasoning is written into the function's doc comment.
- preset_to_json(self, sb) - it does return a code (0), so a null-self/null-sb guard would fit without a signature change, but null pointers are not numeric-range input and no other *_to_json in the port guards them; guarding only preset's would make the four codecs inconsistent. Left for the same constructor-contract pass.
- The 11 built-in preset constructors and preset_all - trusted input, no caller-supplied numbers. Asserted to sit inside every range above (all 11 serialize and deserialize back bit-identically), which is what makes the guards 'only ever reject a document prani did not write' true rather than assumed.

**`src/sequence.cyr`**

- sequence_call_phrase_new — no numeric parameter to range. Its elements were ranged when built and are ranged again by the synthesize pre-pass (which is also where the element count is bounded); documented in the function comment.
- vocalization / intent discriminants at the CONSTRUCTORS (guarded only at the JSON codec). An unsupported vocalization is already reported at synthesis by species_supports_vocalization -> PRANI_ERR_INVALID_VOCALIZATION, and 'unknown intent -> neutral, preserving totality' is vocalization.cyr's stated port contract for a #[non_exhaustive] Rust enum — not this module's to redefine. The codec is different: that is where a nonsense discriminant arrives from outside prani, and serde rejected it there too.
- An UPPER bound on sample_rate. ADR-0001 refuses to restate svara's threshold in prani ('a constant derived from a dependency goes stale silently'); the MAX_SAMPLES product test catches an absurd rate through the thing it actually drives.
- The intent duration_scale (<= 2.0) inside the bout's allocation estimate — a deliberate <=2x under-estimate, documented, leaving the effective ceiling still 8x below the measured abort. Including it would couple sequence.cyr to prani_intent_modifiers for no safety gain.
- CallBout_from_json_str / CallElement_from_json_str — #derive(Serialize)-generated, so no guard can be written inside them. Covered instead by the point-of-use recheck in sequence_call_bout_synthesize / sequence_call_phrase_synthesize, which is every path from those codecs to svara.
- The RESULT of prani_vec_push_zeros is now checked at all three call sites, but as defense in depth only: after the guards its count cannot be negative. Flagged as unreachable in the comments rather than presented as a fix.

**`src/spatial.cyr`**

- _sample_rate on spatial_apply_doppler_shift -- the parameter is unused (the oracle carries it for signature parity only). Measured: a NaN there returns the same 100 finite identity samples as 48000 does. A guard would reject input that provably cannot reach the output, which is rejection without a defect behind it. Asserted as a control instead (3 assertions) so the non-guard is a decision, not an omission.
- An upper bound on distance, reference_distance or sample_rate. Once the guards hold, every derived value is finite by construction: max() floors dist at reference so gain = ref/dist is in (0,1] and cannot overflow; cutoff is floored at 200 Hz so rc <= 8e-4; only 1/sample_rate can overflow and that is checked where computed. Measured controls at 8.99e307 for both distance and sample_rate return finite output.
- \|velocity\| > 343 m/s (supersonic). The oracle's +/-0.9c clamp saturates them to a well-defined 10x decimation, and tests/spatial.tcyr already asserts v = +343 -> out_len 15; rejecting them would be a parity regression on valid input. Three controls now assert v = -308.7, -343 and -1e6 all give out_len 10 on a 100-sample buffer.
- distance == 0 and an empty samples buffer -- both VALID and asserted as controls (distance 0 clamps up to reference and still yields the gain-1.0 result; empty in still gives empty out with valid parameters).
- Output-side NaN (a NaN inside the samples buffer itself). It propagates to the output, but scanning every sample is an O(n) cost on an audio path and the buffer is prani's own product, not a range parameter; the milestone is about input ranges.

**`src/species.cyr`**

- resonance_seed - it is a PCG seed and every one of the 2^64 bit patterns is legal; the table itself carries five negative seeds. Asserted accepted at i64::MIN and -1 so a later 'tidying' guard cannot be mistaken for a fix.
- species_params_make / species_params_make_into - builders that return the struct with no error channel. Per the milestone rule these are 2.1.0 constructor-contract work; guarding them needs a signature change. Their only in-tree callers are the 13-row table (valid by construction) and crvoice_clone_params (a field-for-field copy of already-validated params), so nothing can smuggle a bad value in through them that species_params_validate at the point of use does not catch.
- species_params / species_params_into - table-driven, take only a species discriminant, and every row is asserted in range by the 13-species sweep. An out-of-range species falls through to the Fantasy arm, which is the oracle's #[non_exhaustive] totality behaviour, not a defect.
- species_params_resonance_seed / species_supports_vocalization / species_bout_template - no numeric input; the first two take an already-built params or an integer discriminant, and bout_template returns constants.
- spectral_tilt magnitude - finite-checked only, on purpose. See the guard entry: its consumer already saturates.

**`src/stream.cyr`**

- voc / intent (stream_new): enum-shaped i64 constants, not numeric ranges. species_supports_vocalization already rejects an unsupported voc (unchanged, still the first check and still PRANI_ERR_INVALID_VOCALIZATION), and prani_intent_modifiers is documented total - an unknown intent falls back to the neutral 1.0/1.0/1.0/0.0 arm, preserving the oracle's #[non_exhaustive] behaviour. Rejecting an out-of-enum intent would diverge from that documented totality.
- amplitude_scale (stream_new's fourth snapshot): comes from prani_intent_modifiers, a fixed table of finite constants on every arm including the fallback. There is no input path that makes it non-finite, so a guard would be unreachable code. f0/effort_amp/spectral_tilt ARE guarded because they mix in voice state the caller controls.
- stream_is_finished / stream_total_samples / stream_samples_rendered: pure accessors with no error channel. Returning 0 or 1 for a null handle is exactly the 'fabricated plausible answer' ADR-0001 forbids - a host cannot tell it from a real empty stream. The FFI already null-checks at its own boundary (prani_ffi_stream_is_finished returns 1 for null, matching the Rust None arm), and a real fix is 2.1.0's constructor-contract work. Unchanged from today, where they also deref.
- sample_rate upper bound: none. svara owns tract-side validation (ADR-0001) and the total-samples range already bounds the only arithmetic in this module that a huge rate can break. Inventing a ceiling here would be a constant derived from a dependency, which is what ADR-0001 rejected.
- block_size upper bound: none - it is clamped to `remaining`, which stream_new bounded at 2^53. Asserted with block_size 100000 clamping to the remaining 2205.
- purr_f0 in the cat-purr path: f64_clamp(27.0, 20.0, 35.0) over module constants, no caller input reaches it.
- t inside stream_fill_buffer: computed as f64_from(rendered) / f64_from(max(total,1)), both bounded i64s, so it is finite by construction. Only the contour it feeds needed the point-of-use guard.

**`src/tract.cyr`**

- crtract_new's sample_rate -- svara owns that threshold and crtract_new already reports its rejection (ADR-0001). Restating the constant here is exactly what ADR-0001 refused, because the duplicate is the thing that goes stale. Pinned: a document naming a 10 Hz sample rate is still rejected, by svara, via crtract_new.
- crtract_set_formant_blend's blend / target / target_bw -- MEASURED on the pre-guard tree: svara ALREADY rejects a NaN blend and a NaN target formant (both come back as PRANI_ERR_SVARA through prani_from_svara), and the tract is left synthesizing finite audio afterwards with its old formants. A guard here would restate a dependency's rule. Pinned as four assertions so that if svara ever stops refusing, prani's suite fails and says it now needs its own guard.
- crtract_apply_spectral_tilt's sample_rate -- the function never reads it (the coefficient comes from tilt_db alone; the parameter exists for oracle signature parity). Rejecting a value a function does not read reports a failure that cannot happen. Pinned: a NaN sample_rate is accepted and changes nothing (y[0] is still 0.5 for a -6 dB/oct tilt).
- options on the stridulatory path -- never read there (the oracle writes `let _ = options;`), so there is nothing with a range. Pinned: NaN options with a cricket still succeed.
- f0 on the noise path -- crtract_synthesize_noise takes no f0, so a snake's synthesis never looks at it. This is why the guards live in the five apparatus paths rather than in the crtract_synthesize dispatcher: a dispatcher-level guard would reject a NaN f0 for a snake, a failure that cannot happen. Pinned, so moving the guard up fails the suite and says why.
- from_json_str rng_state / rng_inc -- every 64-bit pattern is a reachable PCG32 state; neither is a number with a range. An EVEN `inc` is the one arguable case (PCG32 wants it odd, and prani_rng_new always emits odd), but that is generator quality, not a numeric range, and it produces no NaN, no empty buffer and no crash.
- An upper bound on num_samples -- no measured overflow exists (nothing does arithmetic on it here; the port pushes into a growable vec), so any ceiling would be invented rather than derived. Allocation failure stays the allocator's contract.
- Null/short-vec checks on apply_spectral_tilt's `samples` and set_formant_blend's 3-slot vecs -- pointer and length validity, not numeric range. Out of this milestone's scope; flagged rather than silently skipped.
- Missing-field detection in from_json_str -- bayan returns 0 for an absent key, so a document omitting `phase` or `dcb_r` reads as 0.0 and passes the ranges. That is field PRESENCE, which ADR-0002 scoped to parse-level and 2.0.5 scopes to ranges; it belongs with the constructor-contract work, not here.

**`src/voice.cyr`**

- crvoice_new, crvoice_with_f0_offset, crvoice_with_size, crvoice_with_breathiness, crvoice_with_jitter, crvoice_with_shimmer, crvoice_with_vocal_effort, crvoice_set_vocal_effort, crvoice_apply_lombard_effect, crvoice_reset_individual -- they return self or nothing, so there is no error channel to report into. Guarding them means changing signatures, which is roadmap 2.1.0's constructor-contract work. Every value they can smuggle in is instead reported at the point of use (params_in_range plus the two derived values at the top of vocalize_with_intent, and size_scale in synthesize_cat_purr), which covers every path to svara with zero API change.
- crvoice_effective_f0, crvoice_effective_tract_scale -- return an f64 audio quantity; a negative return is a legal value, not a code. They are the things GUARDED, at their consumer.
- crvoice_pitch_contour, crvoice_pitch_contour_f0_at, crvoice_formant_transition_contour, crvoice_formant_transition_at, crvoice_make_cat_meow_howl, crvoice_make_wolf_howl, crvoice_nasal_phase_fraction, crvoice_vocalization_spectral_offset, crvoice_lerp_kf, crvoice_contour_push, crvoice_keyframe_push, crvoice_clone_params, crvoice_species_has_subharmonics, crvoice_species_is_canid, crvoice_voc_is_howl_or_whine -- pure table lookups and interpolation returning a pointer or an f64, with no error channel. Their only inputs are a tag and a normalized t that f64_clamp already bounds; the tag is validated by the caller.
- crvoice_apply_vocalization_envelope -- its sample_rate parameter is UNUSED (kept for signature parity with the oracle), and attack_len/release_len are already min-clamped against len by the existing a_lim/r_lim logic, so it has neither a divisor nor an unbounded index. Nothing to guard.
- crvoice_to_json -- serializing has no range hazard; it writes whatever the struct holds, and the deserializer is where the contract is enforced.
- The species tag stored on a CreatureVoice, at vocalize time. species_params has a DOCUMENTED fallthrough to the Fantasy row for unknown tags ('preserves totality since the Rust enum is #[non_exhaustive]'), so an unknown species is a deliberate design choice of src/species.cyr, not an accident. Overriding it from voice.cyr would silently contradict another module's stated contract; it belongs to species.cyr's owner. The tag IS range-checked in crvoice_from_json_str, where the deserializer contract (decision 1) applies.
- SpeciesParams.resonance_seed -- an RNG seed. Every i64 bit pattern is a legal one, and pinned as accepted (a -1 seed round-trips) so a future tightening cannot creep in unnoticed.
- Nyquist upper bounds on formant0..2 / bandwidth0..2 in the deserializer. The sample rate is not known at deserialize time, so any bound would be a guess; they are checked finite and > 0 only.
- An upper bound on duration itself. A long call is a legitimate request; the only real failure mode is the num_samples overflow, which is guarded directly on the derived product rather than by inventing a maximum call length.

