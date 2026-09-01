# prani — Rust test parity ledger (2.0.4)

> **The artifact of milestone 2.0.4** (complete — see
> [`CHANGELOG.md`](../../CHANGELOG.md); the roadmap carries open work only).
> One row per Rust `#[test]` block, against the 17 Cyrius suites. Companion to
> [`port-audit.md`](port-audit.md), which is per-*module*; this is per-*test*.

The port was verified module-by-module against the oracle's **source**. It had
never been verified against the oracle's **tests** — a different question, and
the one that decides whether [2.0.8](roadmap.md#208--retire-rust-old) can
safely drop the oracle from the working tree. Parity after the removal is
asserted by these suites alone, so anything the Rust asserts that the Cyrius
does not is lost the day the directory goes.

**Method**: 73 Rust tests mapped in 13 parallel passes; every claimed gap then
handed to an independent pass instructed to *refute* it by finding the covering
assertion, and to check the two implementations against each other for a real
behavioural difference. 55 agents, 0 errors. One claimed gap was refuted
(`test_serde_roundtrip_intent_modifiers`, PARTIAL → COVERED).

Oracle citations follow [ADR-0004](../adr/0004-cite-the-oracle-by-tag.md): read
any of them with `git show 2.0.3:rust-old/tests/integration.rs`. The Ledger
table's **Oracle** column is the one abbreviation: a cell reading
`integration.rs:6-13` means `2.0.3:rust-old/tests/integration.rs:6-13`, and the
43 rows that carry a shortfall entry below spell it out in full there.
Everywhere outside that column the tag is written out.

## Result

| | Count | Meaning |
|---|---:|---|
| ✅ **Covered** | 30 | Every property the Rust asserts is asserted in Cyrius at equal or greater strength. |
| 🟡 **Partial** | 30 | Some properties covered, at least one dropped — usually a narrower species/parameter set, or a property with no Cyrius analogue yet. |
| 🔴 **Gap** | 11 | No Cyrius assertion covers the Rust test's code path at all. |
| ⬜ **N/A** | 2 | A Rust language property with no Cyrius analogue. Not a gap. |
| | **73** | |

### 🟢 Zero behavioural defects

**No row turned out to be a real difference between `src/*.cyr` and
`2.0.3:rust-old/src/*.rs`.** The roadmap said a gap that is actually a
behavioural difference is a defect and takes priority over everything else in
the arc — none was found. Every finding below is a *test* gap: the port does
the right thing, nothing asserts it. Where the outcome was not obvious from
reading both implementations, the verifying pass built a throwaway probe and
measured the port at the oracle's exact arguments rather than reasoning about it.

The one exception is [`test_serde_roundtrip_creature_tract`](#test_serde_roundtrip_creature_tract),
which *is* a deliberate divergence — the port's `CreatureTract` JSON carries a
narrower field set than the oracle's, because svara exposes its tract as an
opaque handle with no state accessors. That wants an ADR, not a test.

## Closure — done 2026-08-31

All seven themes are closed. The suite went **770 → 1200 assertions across 17
suites**, and `cyrius audit` exits 0 (fmt · lint · docs · tests · bench).

| Suite | Before | After |
|---|--:|--:|
| `tests/voice.tcyr` | 73 | **222** |
| `tests/species.tcyr` | 66 | **135** |
| `tests/preset.tcyr` | 99 | **117** |
| `tests/emotion.tcyr` | 72 | **89** |
| `tests/bridge.tcyr` | 75 | **81** |
| `tests/fatigue.tcyr` | 61 | **76** |
| `tests/vocalization.tcyr` | 21 | **76** |
| `tests/stream.tcyr` | 35 | **69** |
| `tests/tract.tcyr` | 52 | **64** |
| `tests/sequence.tcyr` | 33 | **61** |
| `tests/spatial.tcyr` | 24 | **37** |
| `tests/prani.tcyr` | 16 | **29** |
| `tests/dsp.tcyr` | 13 | **14** |
| unchanged | `error` 21 · `ffi` 37 · `hardening` 53 · `rng` 19 | |

**No file under `src/` was modified.** That was the instruction and it held: the
audit found zero defects, and closing the gaps required no source change.

Two things worth carrying forward:

- **Negative controls were run, not assumed.** The `voice.tcyr` pass temporarily
  inverted 10 of its highest-value new assertions — the Alarm-vs-Idle energy
  comparison, shout-vs-whisper, the audibility floors, the 13/13 species count,
  the purr size comparison — confirmed all 10 **fail**, then restored and re-ran
  green. The theme that motivated this whole milestone was assertions that cannot
  fail; the closure had to prove it did not add more.
- **Expected values are hand-derived, not captured.** Lengths from
  `duration × sample_rate` (2205, 1543 = trunc(0.05·0.7·44100), 3307, 8820,
  13230), effective-f0 values from the species table (Dog 300/0.5 = 600, Dragon
  70/3 = 23.33 → clamps up to f0_min 30, deserialized (400+50)/1.2 = 375), and
  every audibility floor taken from the oracle's own bar (0.001, 1e-4, 0.01).

### One ledger row was wrong, and the closure caught it

The entry below for [`test_cricket_pulse_train`](#test_cricket_pulse_train)
originally proposed asserting the inter-chirp silence gap through a 0.2 s
`crvoice_vocalize` call. **That is impossible, in the port and in the oracle
alike**, and the claim has been corrected in place.

`pos_in_chirp` is `i % chirp_period` over the index **within one synthesize
call** (`src/tract.cyr:364-368`, porting `2.0.3:rust-old/src/tract.rs:256-258` —
structurally identical). `crvoice_vocalize` renders in blocks of
`sample_rate × 0.02` = 882 samples at 44100, each a fresh call starting at `i =
0`, so `pos_in_chirp` never exceeds 881 while `chirp_active` is 5880. **The
silence arm is unreachable through the public API at any duration.**

The oracle's own `test_cricket_pulse_train` passes anyway — on the per-syllable
pulse envelope, not on the inter-chirp gap its comment names. So the port carries
the same bar (`near_silent > 10`, verbatim), and the silence arm is now asserted
where it is actually reachable: a **direct `crtract_synthesize_stridulatory` call
of 12000 samples** in `tests/tract.tcyr`, asserting `[10000, 12000)` is silent to
the last sample (2000/2000), with two controls proving the active chirp is not.

This is a faithful port of an oracle quirk, not a defect — but it is a reminder
that a suggested fix is a hypothesis until the code runs.

## What the 41 shortfalls are, grouped

The 30 partials and 11 gaps are not 41 unrelated omissions — they are **seven
themes**, and two of them account for most of the list. Every non-covered row
below appears in at least one theme; the counts overlap because a row can sit in
two.

### T1 — Nothing measures the magnitude of synthesized audio

*8 rows, 2 of them hard gaps*

**The largest structural gap, and the one that makes the others matter.** Every synthesis assertion in all 17 suites is *non-error + exact length + all-finite*. **An all-zero buffer satisfies all three.** No suite computes a peak, an RMS, or an energy over a `crvoice_vocalize` result — `f64_abs` appears only inside tolerance comparisons on scalar helpers. The oracle checks `max_amp > 0.001` (wolf howl), `> 0.0001` (cat purr), and `near_energy > 10 * far_energy` (attenuation), and asserts that Alarm carries more energy than Idle and a shout more than a whisper. None of that survives the port. Two `t_max_abs(v)` / `t_energy(v)` helpers beside the existing `t_all_finite` (`tests/voice.tcyr:66-75`) unlock every row here.

<sub>🟡 [`test_wolf_howl`](#test_wolf_howl) · 🟡 [`test_intent_modifies_output`](#test_intent_modifies_output) · 🟡 [`test_all_species_synthesize`](#test_all_species_synthesize) · 🟡 [`test_cat_purr_special_synthesis`](#test_cat_purr_special_synthesis) · 🔴 [`test_crocodilian_rumble_with_subharmonics`](#test_crocodilian_rumble_with_subharmonics) · 🟡 [`test_distance_attenuation`](#test_distance_attenuation) · 🔴 [`test_vocal_effort_whisper_vs_shout`](#test_vocal_effort_whisper_vs_shout) · 🟡 [`test_cricket_pulse_train`](#test_cricket_pulse_train)</sub>

### T2 — 6 of 13 species and 6 vocalizations are never synthesized

*16 rows, 6 of them hard gaps*

Only **Wolf, Lion, Songbird, Snake, Cat, Cricket, Dragon** ever reach `crvoice_vocalize`. **Dog, Crow, Raptor, Crocodilian, Bee and Fantasy never synthesize anywhere** — several never construct a `CreatureTract` at all. On the vocalization axis, **SCREECH, RUMBLE, GROWL, BARK and CHIRP are never synthesized**, and the Cat+HOWL pair — which is what drives the nasal anti-formant and the cat formant-transition table — is never built. The oracle covers all of it. This is also the theme that makes 2.0.7's benchmark breadth and these tests one piece of work: the same species list is missing from both.

<sub>🟡 [`test_individual_variation`](#test_individual_variation) · 🟡 [`test_all_species_synthesize`](#test_all_species_synthesize) · 🟡 [`test_bee_buzz`](#test_bee_buzz) · 🔴 [`test_crow_screech`](#test_crow_screech) · 🔴 [`test_crocodilian_rumble_with_subharmonics`](#test_crocodilian_rumble_with_subharmonics) · 🔴 [`test_raptor_screech`](#test_raptor_screech) · 🟡 [`test_subharmonics_are_finite`](#test_subharmonics_are_finite) · 🔴 [`test_cat_howl_formant_transitions`](#test_cat_howl_formant_transitions) · 🔴 [`test_cat_nasal_resonance`](#test_cat_nasal_resonance) · 🟡 [`test_call_bout`](#test_call_bout) · 🟡 [`test_call_phrase`](#test_call_phrase) · 🔴 [`test_spectral_envelope_per_vocalization`](#test_spectral_envelope_per_vocalization) · 🟡 [`test_non_stationary_perturbation`](#test_non_stationary_perturbation) · 🟡 [`test_emotion_state_drives_synthesis`](#test_emotion_state_drives_synthesis) · 🟡 [`test_stream_finishes`](#test_stream_finishes) · 🟡 [`test_stream_next_block`](#test_stream_next_block)</sub>

### T3 — Composite paths are never driven end to end

*4 rows, 2 of them hard gaps*

**No preset-built voice is ever synthesized** — all 11 presets are asserted field-by-field and then never called. **Only 3 of the 13 bout templates are constructed**, and none is synthesized, so a template naming a vocalization its own species rejects would not be caught. Every chorus assertion mixes two bit-identical default-size Wolves, so the `voice_count * 7919` seed path and the size-heterogeneous mix the oracle exercises are untested. And nothing feeds an `emotion_evaluate` result into a voice — the emotion→synthesis composition the library exists to provide.

<sub>🔴 [`test_voice_presets`](#test_voice_presets) · 🔴 [`test_bout_template_all_species`](#test_bout_template_all_species) · 🟡 [`test_chorus_synthesis`](#test_chorus_synthesis) · 🟡 [`test_emotion_state_drives_synthesis`](#test_emotion_state_drives_synthesis)</sub>

### T4 — Four serde roundtrips cannot fail

*4 rows, 0 of them hard gaps*

A methodology defect, not a coverage one. bayan's value accessors are null-safe by design — `bayan_json_v_int(0)` returns 0 — so **a roundtrip asserted on a value whose discriminant is 0 passes even if the derive dropped the key entirely.** `CallBout` is round-tripped only with `PRANI_VOC_HOWL` (= 0); `VocalApparatus` only as Laryngeal (= 0), 1 of 5 variants. The oracle uses `Bark` and `Dragon` — nonzero — and would catch it. This is the same null-safe composition that produced 2.0.3's HIGH finding ([ADR-0002](../adr/0002-deserializers-report-parse-failure.md)), showing up in the tests this time instead of the source. Fix by round-tripping nonzero discriminants.

<sub>🟡 [`test_serde_roundtrip_species`](#test_serde_roundtrip_species) · 🟡 [`test_serde_roundtrip_vocal_apparatus`](#test_serde_roundtrip_vocal_apparatus) · 🟡 [`test_serde_roundtrip_call_bout`](#test_serde_roundtrip_call_bout) · 🟡 [`test_serde_roundtrip_creature_voice`](#test_serde_roundtrip_creature_voice)</sub>

> **Found alongside T4, and the same class: every serde idempotency assertion in
> the project is a prefix compare.** All 11 of them are
> `memeq(json, json2, strlen(json))` — so a re-serialization that **appended** a
> field passes. The oracle compared whole strings (`assert_eq!(json, json2)`).
> Sites: `tests/dsp.tcyr:76`, `tests/vocalization.tcyr:70`,
> `tests/emotion.tcyr:226,249`, `tests/fatigue.tcyr:246,262`,
> `tests/voice.tcyr:296`, `tests/sequence.tcyr:157`, `tests/preset.tcyr:261`,
> `tests/species.tcyr:133`, and `tests/tract.tcyr:255` (**fixed** — it now asserts
> `strlen(json2) == strlen(json)` first). The remaining 10 are a one-line fix each.

### T5 — Edge and boundary inputs the oracle probes and the suites do not

*7 rows, 2 of them hard gaps*

**Zero duration is never passed to any synthesis entry point.** No stream is ever drained in more than two `fill_buffer` calls, so the `t > 0.85` release-boundary arm is dead in *both* `voice.cyr` and `stream.cyr` — no assertion in the suite reaches it. `fatigue` is never read after more than one `record_call`, so an implementation that **assigned instead of accumulated** would pass. `emotion_update` is never called on a non-default smoothing, so one that ignored the field and used the hardcoded 0.1 would pass. `crvoice_with_jitter` and `crvoice_with_shimmer` have **zero test callers**. And the cricket buffers are all shorter than one chirp period, so the pulse-train gap the species exists to produce is never rendered.

<sub>🔴 [`test_zero_duration_synthesis`](#test_zero_duration_synthesis) · 🔴 [`test_cat_purr_size_variation`](#test_cat_purr_size_variation) · 🟡 [`test_emotion_state_smooth_update`](#test_emotion_state_smooth_update) · 🟡 [`test_fatigue_accumulates`](#test_fatigue_accumulates) · 🟡 [`test_stream_produces_same_length_as_batch`](#test_stream_produces_same_length_as_batch) · 🟡 [`test_parameter_clamping`](#test_parameter_clamping) · 🟡 [`test_cricket_pulse_train`](#test_cricket_pulse_train)</sub>

### T6 — Table and matrix breadth on pure lookups

*5 rows, 0 of them hard gaps*

Cheap rows, worth closing because they are nearly free. Two intents (`TERRITORIAL`, `SUBMISSIVE`) have their modifiers asserted nowhere and the pairwise-distinctness property the oracle checks has no analogue; the support matrix misses Wolf+Growl and Cricket+Chirp; every rejection assertion in the tree uses the single pair Snake+Howl; and both of the oracle's `size_from_body_mass` probe points (0.03 kg, 50 kg) lie outside the range the suite exercises, so the assertions there extrapolate rather than interpolate.

<sub>🟡 [`test_species_valid_vocalizations`](#test_species_valid_vocalizations) · 🟡 [`test_all_intents_modify_differently`](#test_all_intents_modify_differently) · 🟡 [`test_dragon_individual_variation`](#test_dragon_individual_variation) · 🟡 [`test_bridge_size_from_body_mass`](#test_bridge_size_from_body_mass) · 🟡 [`test_invalid_species_vocalization_rejected`](#test_invalid_species_vocalization_rejected)</sub>

### T7 — One real divergence — wants an ADR, not a test

*1 rows, 0 of them hard gaps*

The port's `CreatureTract` JSON is a strictly **narrower field set** than the oracle's: it emits sample_rate, phase, rng state, DC-blocker state and params, and rebuilds the svara `VocalTract` and naad biquad state rather than restoring them. That is not a bug and not fixable in prani — svara exposes the tract as an opaque handle with no state accessors, the same root cause as [ADR-0001](../adr/0001-check-svara-tract-constructor.md). It is a **documented behavioural difference in the serialization contract** and belongs beside ADR-0001–0004.

<sub>🟡 [`test_serde_roundtrip_creature_tract`](#test_serde_roundtrip_creature_tract)</sub>
## Ledger

| # | Rust test | Oracle | Verdict | Covering suites | Shortfall |
|--:|---|---|---|---|---|
| 1 | [`test_wolf_howl`](#test_wolf_howl) | `integration.rs:6-13` | 🟡 partial | `hardening`, `prani`, `voice` | Nothing in any of the 17 Cyrius suites asserts anything about the MAGNITUDE of synthesized output. `rg -n 'f64_abs\|f64_gt\|assert_gt' tests/*.tcyr` r… |
| 2 | `test_cat_purr` | `integration.rs:16-21` | ✅ covered | `stream`, `tract`, `voice` | — |
| 3 | `test_snake_hiss` | `integration.rs:24-29` | ✅ covered | `species`, `stream`, `tract`, `voice` | — |
| 4 | `test_cricket_stridulate` | `integration.rs:32-39` | ✅ covered | `species`, `tract`, `voice` | — |
| 5 | `test_lion_roar` | `integration.rs:42-47` | ✅ covered | `voice` | — |
| 6 | `test_dragon_roar` | `integration.rs:50-55` | ✅ covered | `voice` | — |
| 7 | `test_songbird_trill` | `integration.rs:57-63` | ✅ covered | `stream`, `voice` | — |
| 8 | [`test_individual_variation`](#test_individual_variation) | `integration.rs:65-72` | 🟡 partial | `preset`, `voice` | Species::Dog is never exercised on this path — no Cyrius suite calls crvoice_effective_f0 or crvoice_effective_tract_scale on a Dog voice, and no Cyri… |
| 9 | [`test_intent_modifies_output`](#test_intent_modifies_output) | `integration.rs:74-92` | 🟡 partial | `hardening`, `prani`, `vocalization`, `voice` | The central property — property 3, that Alarm output carries strictly more energy than Idle output — is asserted NOWHERE. No Cyrius suite computes an… |
| 10 | [`test_all_species_synthesize`](#test_all_species_synthesize) | `integration.rs:94-125` | 🟡 partial | `ffi`, `prani`, `species`, `tract`, `voice` | Only 7 of the 13 species ever reach crvoice_vocalize: Wolf, Lion, Songbird, Snake, Cat, Cricket, Dragon (tests/voice.tcyr:197-236). Dog, Crow, Raptor,… |
| 11 | [`test_serde_roundtrip_species`](#test_serde_roundtrip_species) | `integration.rs:127-132` | 🟡 partial | `hardening`, `preset`, `voice` | Two distinct shortfalls. (a) The value the oracle actually round-trips — Species::Dragon — is never round-tripped in any Cyrius suite. Species survive… |
| 12 | `test_serde_roundtrip_vocalization` | `integration.rs:134-139` | ✅ covered | `sequence`, `species`, `vocalization`, `voice` | — |
| 13 | `test_serde_roundtrip_call_intent` | `integration.rs:142-146` | ✅ covered | `emotion`, `sequence` | — |
| 14 | [`test_serde_roundtrip_creature_voice`](#test_serde_roundtrip_creature_voice) | `integration.rs:149-159` | 🟡 partial | `hardening`, `voice` | Two of the three Rust assertions have no counterpart. (1) Nothing asserts crvoice_effective_f0 on a DESERIALIZED voice — the closest assertions (tests… |
| 15 | [`test_invalid_species_vocalization_rejected`](#test_invalid_species_vocalization_rejected) | `integration.rs:162-174` | 🟡 partial | `prani`, `sequence`, `species`, `stream`, `voice` | Only ONE of the three species/vocalization pairs is asserted end-to-end through crvoice_vocalize: Snake+Howl. Cricket+Roar and Wolf+Stridulate are ass… |
| 16 | [`test_species_valid_vocalizations`](#test_species_valid_vocalizations) | `integration.rs:177-190` | 🟡 partial | `species` | 5 of the 7 asserted pairs are covered exactly; 2 are not. (a) Wolf+Growl (integration.rs:180) is asserted nowhere — no suite calls species_supports_vo… |
| 17 | [`test_parameter_clamping`](#test_parameter_clamping) | `integration.rs:193-203` | 🟡 partial | `voice` | Three of the five properties are unasserted, including the only explicit assertion in the Rust test. (1) crvoice_with_jitter (src/voice.cyr:191-196) h… |
| 18 | `test_serde_roundtrip_species_params` | `integration.rs:206-212` | ✅ covered | `species`, `tract` | — |
| 19 | [`test_serde_roundtrip_vocal_apparatus`](#test_serde_roundtrip_vocal_apparatus) | `integration.rs:214-229` | 🟡 partial | `species` | Only 1 of the 5 apparatus variants is ever round-tripped through the codec, and it is the one variant that cannot detect a failure. The port has no st… |
| 20 | [`test_serde_roundtrip_error`](#test_serde_roundtrip_error) | `integration.rs:231-237` | ⬜ n/a | `error` | — |
| 21 | `test_serde_roundtrip_intent_modifiers` | `integration.rs:239-248` | ✅ covered | `vocalization` | — |
| 22 | [`test_serde_roundtrip_creature_tract`](#test_serde_roundtrip_creature_tract) | `integration.rs:250-258` | 🟡 partial | `hardening`, `species`, `tract` | The port's CreatureTract JSON is a strictly NARROWER field set than the oracle's, so the oracle's `json == json2` covers state the port's idempotency… |
| 23 | [`test_zero_duration_synthesis`](#test_zero_duration_synthesis) | `integration.rs:260-265` | 🔴 gap | — | Nothing in any of the 17 suites calls a synthesis entry point with a zero duration. I enumerated every `crvoice_vocalize` / `crvoice_vocalize_with_int… |
| 24 | `test_high_frequency_syringeal_path` | `integration.rs:267-277` | ✅ covered | `species`, `stream`, `tract`, `voice` | — |
| 25 | [`test_bee_buzz`](#test_bee_buzz) | `integration.rs:279-285` | 🟡 partial | `ffi`, `species`, `tract`, `vocalization` | Nothing anywhere in the 17 suites calls crvoice_vocalize (or crvoice_vocalize_with_intent) on a Bee voice — PRANI_SP_BEE appears exactly once in the w… |
| 26 | [`test_crow_screech`](#test_crow_screech) | `integration.rs:287-295` | 🔴 gap | `bridge`, `emotion`, `preset`, `tract` | None of the Rust test's assertions is reproduced. crvoice_vocalize is never called with PRANI_SP_CROW (the only Crow voice built anywhere, tests/prese… |
| 27 | [`test_all_intents_modify_differently`](#test_all_intents_modify_differently) | `integration.rs:297-320` | 🟡 partial | `ffi`, `vocalization` | The distinctness property itself — the only thing this Rust test asserts — has no Cyrius counterpart anywhere: I grepped all 17 suites for prani_inten… |
| 28 | [`test_crocodilian_rumble_with_subharmonics`](#test_crocodilian_rumble_with_subharmonics) | `integration.rs:322-330` | 🔴 gap | `bridge`, `emotion`, `preset`, `voice` | Two independent holes. (a) SPECIES: crvoice_vocalize is never called on a Crocodilian voice, and PRANI_VOC_RUMBLE is never synthesized by any test. So… |
| 29 | [`test_raptor_screech`](#test_raptor_screech) | `integration.rs:332-340` | 🔴 gap | `preset`, `tract`, `voice` | None of the Rust test's assertions is reproduced on the same code path. crvoice_vocalize is never called with PRANI_SP_RAPTOR — the sole Raptor voice… |
| 30 | [`test_dragon_individual_variation`](#test_dragon_individual_variation) | `integration.rs:342-354` | 🟡 partial | `hardening`, `preset`, `voice` | Two properties are unasserted. (a) The Dragon-specific ORDERING assertion has no counterpart. The inverse size/pitch relation is pinned exactly but on… |
| 31 | [`test_cat_purr_special_synthesis`](#test_cat_purr_special_synthesis) | `integration.rs:358-368` | 🟡 partial | `hardening`, `stream`, `tract`, `voice` | The audibility property (max \|sample\| > 0.0001) is not asserted anywhere. Ripgrep for max_amp / max_abs / audib / amplitude / f64_abs across all 17… |
| 32 | [`test_cat_purr_size_variation`](#test_cat_purr_size_variation) | `integration.rs:370-379` | 🔴 gap | `tract`, `voice` | No Cyrius test combines a size-scaled voice with a purr. Grepping crvoice_with_size across all 17 suites returns exactly voice.tcyr:103, 112, 114, 263… |
| 33 | [`test_subharmonics_are_finite`](#test_subharmonics_are_finite) | `integration.rs:381-405` | 🟡 partial | `vocalization`, `voice` | Three distinct holes. (a) CROCODILIAN IS NEVER SYNTHESIZED ANYWHERE. Grepping PRANI_SP_CROCODILIAN across all 17 suites hits only bridge.tcyr:152-158… |
| 34 | `test_wolf_howl_formant_transitions` | `integration.rs:407-414` | ✅ covered | `hardening`, `prani`, `stream`, `voice` | — |
| 35 | [`test_cat_howl_formant_transitions`](#test_cat_howl_formant_transitions) | `integration.rs:416-423` | 🔴 gap | `voice` | No Cyrius suite ever synthesizes a cat howl. I grepped PRANI_SP_CAT and PRANI_VOC_HOWL across all 17 .tcyr files: every PRANI_SP_CAT synthesis is a PU… |
| 36 | [`test_cricket_pulse_train`](#test_cricket_pulse_train) | `integration.rs:425-441` | 🟡 partial | `species`, `tract`, `voice` | The pulse-train / silence-gap property is not asserted, and the code that produces it is never even reached. At 44100 Hz src/tract.cyr:360-362 compute… |
| 37 | `test_wolf_biphonation` | `integration.rs:445-452` | ✅ covered | `hardening`, `prani`, `voice` | — |
| 38 | [`test_cat_nasal_resonance`](#test_cat_nasal_resonance) | `integration.rs:454-461` | 🔴 gap | `hardening`, `voice` | No assertion in any of the 17 Cyrius suites calls crvoice_vocalize (or crvoice_vocalize_with_intent, or stream_new) with the pair (PRANI_SP_CAT, PRANI… |
| 39 | `test_doppler_shift` | `integration.rs:463-476` | ✅ covered | `prani`, `spatial`, `voice` | — |
| 40 | [`test_distance_attenuation`](#test_distance_attenuation) | `integration.rs:478-493` | 🟡 partial | `spatial`, `voice` | Two distinct properties are dropped. (1) THE AGGREGATE ENERGY RATIO. The oracle sums s² over a full 22050-sample vocalization at 1 m and at 50 m and a… |
| 41 | [`test_call_bout`](#test_call_bout) | `integration.rs:495-517` | 🟡 partial | `sequence`, `species`, `vocalization`, `voice` | The structural properties are covered — at greater strength than the oracle — but only on a different configuration: Wolf / HOWL / Social / count 2 /… |
| 42 | [`test_call_phrase`](#test_call_phrase) | `integration.rs:519-546` | 🟡 partial | `sequence`, `voice` | Two properties are dropped. (1) NO FINITENESS ASSERTION ON PHRASE OUTPUT. tests/sequence.tcyr:89-97 asserts only prani_is_err == 0 and vec_len == 1600… |
| 43 | [`test_chorus_synthesis`](#test_chorus_synthesis) | `integration.rs:548-565` | 🟡 partial | `prani`, `sequence` | No chorus assertion anywhere uses more than 2 voices, and none uses voices with differing size scales — both Cyrius chorus tests mix two bit-identical… |
| 44 | [`test_voice_presets`](#test_voice_presets) | `integration.rs:567-592` | 🔴 gap | `prani`, `preset`, `species` | Nothing in any of the 17 suites ever SYNTHESIZES from a preset-built voice: grep of crvoice_vocalize / crvoice_vocalize_with_intent across tests/*.tcy… |
| 45 | [`test_serde_roundtrip_call_bout`](#test_serde_roundtrip_call_bout) | `integration.rs:594-608` | 🟡 partial | `hardening`, `sequence` | The CallBout roundtrip is asserted only with PRANI_VOC_HOWL, whose discriminant is 0 (src/vocalization.cyr:16). bayan's value accessors are null-safe,… |
| 46 | `test_serde_roundtrip_voice_preset` | `integration.rs:610-617` | ✅ covered | `hardening`, `preset` | — |
| 47 | [`test_bout_template_all_species`](#test_bout_template_all_species) | `integration.rs:619-651` | 🔴 gap | `sequence`, `species` | Three separate holes. (a) Only 3 of the 13 templates are constructed by any test — Cat, Songbird, Crow, Raptor, Snake, Crocodilian, Cricket, Bee, Drag… |
| 48 | [`test_spectral_envelope_per_vocalization`](#test_spectral_envelope_per_vocalization) | `integration.rs:653-663` | 🔴 gap | `stream`, `voice` | Neither Growl nor Screech is ever synthesized end-to-end by any suite. Every crvoice_vocalize / crvoice_vocalize_with_intent call in tests/*.tcyr uses… |
| 49 | `test_source_filter_coupling_birds` | `integration.rs:665-674` | ✅ covered | `stream`, `tract`, `voice` | — |
| 50 | [`test_non_stationary_perturbation`](#test_non_stationary_perturbation) | `integration.rs:676-689` | 🟡 partial | `hardening`, `sequence`, `species`, `vocalization`, `voice` | Two things this Rust test asserts are asserted nowhere in the Cyrius suites. (1) Vocalization::Bark is never synthesized: a grep of every synthesis ca… |
| 51 | [`test_vocal_effort_whisper_vs_shout`](#test_vocal_effort_whisper_vs_shout) | `integration.rs:693-707` | 🔴 gap | `bridge`, `fatigue`, `vocalization`, `voice` | Nothing in any of the 17 Cyrius suites measures the ENERGY, RMS, peak amplitude or any loudness proxy of a synthesized buffer. I grepped tests/*.tcyr… |
| 52 | `test_vocal_effort_default_is_normal` | `integration.rs:709-713` | ✅ covered | `hardening`, `voice` | — |
| 53 | `test_vocal_effort_set_mutably` | `integration.rs:715-720` | ✅ covered | `ffi`, `voice` | — |
| 54 | `test_emotion_state_default` | `integration.rs:722-727` | ✅ covered | `emotion` | — |
| 55 | `test_emotion_state_evaluate_high_arousal_negative` | `integration.rs:729-735` | ✅ covered | `emotion` | — |
| 56 | `test_emotion_state_evaluate_low_arousal_positive` | `integration.rs:737-744` | ✅ covered | `emotion` | — |
| 57 | [`test_emotion_state_smooth_update`](#test_emotion_state_smooth_update) | `integration.rs:746-753` | 🟡 partial | `emotion` | No assertion in any of the 17 suites calls emotion_update on a state whose smoothing was changed from the 0.1 default (grep for emotion_update across… |
| 58 | [`test_emotion_state_drives_synthesis`](#test_emotion_state_drives_synthesis) | `integration.rs:755-765` | 🟡 partial | `emotion`, `hardening`, `prani`, `voice` | Three properties are unasserted anywhere in the 17 suites. (1) The emotion -> voice composition: no test feeds an emotion_evaluate result into crvoice… |
| 59 | `test_serde_roundtrip_emotion_state` | `integration.rs:767-774` | ✅ covered | `emotion` | — |
| 60 | `test_lombard_effect` | `integration.rs:776-787` | ✅ covered | `bridge`, `ffi`, `voice` | — |
| 61 | [`test_fatigue_accumulates`](#test_fatigue_accumulates) | `integration.rs:789-804` | 🟡 partial | `fatigue` | Nothing in the Cyrius suite reads `fatigue` after MORE THAN ONE fatigue_record_call. Every fatigue-valued assertion (tests/fatigue.tcyr:33 single 10 s… |
| 62 | `test_fatigue_recovers_with_rest` | `integration.rs:806-816` | ✅ covered | `fatigue` | — |
| 63 | `test_habituation_alarm_calls` | `integration.rs:818-833` | ✅ covered | `fatigue` | — |
| 64 | `test_serde_roundtrip_fatigue_state` | `integration.rs:835-842` | ✅ covered | `fatigue` | — |
| 65 | [`test_stream_produces_same_length_as_batch`](#test_stream_produces_same_length_as_batch) | `integration.rs:844-866` | 🟡 partial | `hardening`, `stream`, `voice` | No Cyrius test ever drains a stream in more than TWO fill_buffer calls, and every fill in every suite starts at normalized time t = 0, t = 0.2322 (tes… |
| 66 | [`test_stream_next_block`](#test_stream_next_block) | `integration.rs:868-879` | 🟡 partial | `stream` | Two shortfalls, both mild. (1) The cat-purr special path is never driven through stream_next_block — tests/stream.tcyr's next_block group uses only Wo… |
| 67 | [`test_stream_finishes`](#test_stream_finishes) | `integration.rs:881-894` | 🟡 partial | `ffi`, `hardening`, `stream` | Two things the Rust test asserts are not asserted anywhere in the Cyrius suites. (1) Vocalization::Bark is never streamed: every stream_new in tests/s… |
| 68 | `test_stream_invalid_vocalization_rejected` | `integration.rs:896-902` | ✅ covered | `ffi`, `prani`, `species`, `stream` | — |
| 69 | [`test_bridge_size_from_body_mass`](#test_bridge_size_from_body_mass) | `integration.rs:904-915` | 🟡 partial | `bridge` | Properties 2 and 3 are unasserted, and both oracle probe points lie strictly OUTSIDE the range the Cyrius suite exercises — this is extrapolation, not… |
| 70 | `test_bridge_intent_from_threat` | `integration.rs:917-923` | ✅ covered | `bridge` | — |
| 71 | `test_bridge_vocal_effort_from_arousal` | `integration.rs:925-932` | ✅ covered | `bridge` | — |
| 72 | `test_bridge_lombard_boost` | `integration.rs:934-939` | ✅ covered | `bridge`, `voice` | — |
| 73 | [`public_types_are_send_sync`](#public_types_are_send_sync) | `lib.rs:88-103 (mod assert_traits, helper `fn _assert_send_sync<T: Send + Sync>() {}` at lib.rs:86)` | ⬜ n/a | `emotion`, `error`, `fatigue`, `preset`, `sequence`, `stream`, `tract` | — |

## Detail — every row that is not fully covered

Each entry gives what the Rust asserts that the Cyrius does not, and the
assertion that would close it. Line numbers are as of `2.0.3`.

### test_wolf_howl

**🟡 partial** · `2.0.3:rust-old/tests/integration.rs:6-13`

**Missing** — Nothing in any of the 17 Cyrius suites asserts anything about the MAGNITUDE of synthesized output. `rg -n 'f64_abs\|f64_gt\|assert_gt' tests/*.tcyr` returns amplitude assertions only for pure-arithmetic bridge/spatial/dsp helpers (e.g. tests/bridge.tcyr:65-72, tests/spatial.tcyr:54) — never for a `crvoice_vocalize` result. Every vocalize assertion is is-not-an-error / exact-length / all-finite, and an all-zero buffer satisfies all three. Rust's `max_amp > 0.001` audibility check is therefore unasserted on this path (and on every other synthesis path).

**Closes with** — Add a max-abs helper to tests/voice.tcyr alongside `t_all_finite` (tests/voice.tcyr:66-75) — `fn t_max_abs(v)` folding `f64_abs`/`f64_gt` — and add to the "vocalize -- non-error, length, finite" group, after tests/voice.tcyr:200: `assert_eq(f64_gt(t_max_abs(w_out), 0x3F50624DD2F1A9FC), 1, "wolf howl max_amp > 0.001 (audible)");` (0x3F50624DD2F1A9FC is the 1e-3 constant already used at tests/dsp.tcyr:34). Worth adding the same line for lion/dragon/snake/cricket in that group, since audibility is cheap and currently zero suites assert it.

**Checked for a defect** — Not a defect — and I did not settle this by reading alone, I settled it by running the code. READING: I diffed the entire wolf-howl chain line-for-line and found it a faithful port with no divergence that could attenuate output: - block loop + t computation: 2.0.3:rust-old/src/voice.rs:229-231 (`let t = rendered as f32 / num_samples.max(1) as f32`) vs src/voice.cyr:853 (`var t = f64_div(f64_from(rendered), f64_from(num_samples_max))`), with num_samples_max floored at 1 at src/voice.cyr:843-844 matching `.max(1)`. - boundary boost, all three branches: 2.0.3:rust-old/src/voice.rs:250-257 vs src/voice.cyr:884-893. The tail arm `1.0 + (t - 0.85) / 0.15 * 0.5` associates left-to-right as `((t-0.85)/0.15)*0.5`, and src/voice.cyr:891-892 encodes exactly that nesting: `f64_mul(f64_div(f64_sub(t, PR_V_0_85), PR_V_0_15), PR_V_0_5)`. - canid biphonation: 2.0.3:rust-old/src/voice.rs:329-355 vs src/voice.cyr:961-988,  …

### test_individual_variation

**🟡 partial** · `2.0.3:rust-old/tests/integration.rs:65-72`

**Missing** — Species::Dog is never exercised on this path — no Cyrius suite calls crvoice_effective_f0 or crvoice_effective_tract_scale on a Dog voice, and no Cyrius suite asserts Dog's numeric SpeciesParams row at all (tests/species.tcyr:78 asserts only Dog's resonance_seed; tests/species.tcyr:105-109 only its bout template). So the fact that Dog's f0_min/f0_max/f0_default/tract_scale are 100/2000/300/0.9 — which is what makes the oracle's unclamped ordering hold — is asserted by nothing. Size 0.5 exactly is also never used (the nearest sub-1 sizes are 0.4 via wolf_pup/kitten, both of which hit the f0_max clamp, and 0.8 via young_dragon). Finally, no Cyrius assertion is a direct ORDERING comparison between two voices: every covering assertion is an exact-value equality, which implies the ordering for the pairs tested but does not state the monotonicity as a property. Property 5 (builder independence) is not asserted anywhere either, and it is the one place the port's model genuinely differs: crvoice_with_size mutates in place and returns the same pointer (src/voice.cyr:178-181), so a caller who reuses one voice pointer for both sizes gets aliasing that Rust's by-value builder made impossible.

**Closes with** — Add to tests/voice.tcyr, in the "effective_f0 -- hand-computed, exact" group (voice.tcyr:99) and "effective_tract_scale" group (voice.tcyr:259): `var dog_s = crvoice_with_size(crvoice_new(PRANI_SP_DOG), T_0_5); var dog_l = crvoice_with_size(crvoice_new(PRANI_SP_DOG), T_2_0);` then `assert_eq(crvoice_effective_f0(dog_s), f64_from(600), "dog @ size 0.5 -> 600 (unclamped)"); assert_eq(crvoice_effective_f0(dog_l), f64_from(150), "dog @ size 2.0 -> 150 (unclamped)"); assert_eq(f64_gt(crvoice_effective_f0(dog_s), crvoice_effective_f0(dog_l)), 1, "smaller size -> higher effective_f0"); assert_eq(f64_lt(crvoice_effective_tract_scale(dog_s), crvoice_effective_tract_scale(dog_l)), 1, "smaller size -> shorter tract");` — the two ordering assertions state the monotonicity directly, which no current assertion does. Separately add a "params -- Dog (laryngeal, f0 100-2000/300, tract_scale 0.9)" group to tests/species.tcyr alongside the existing Wolf/Lion/Songbird/Snake groups, so a mis-transcribed Dog row is caught.

### test_intent_modifies_output

**🟡 partial** · `2.0.3:rust-old/tests/integration.rs:74-92`

**Missing** — The central property — property 3, that Alarm output carries strictly more energy than Idle output — is asserted NOWHERE. No Cyrius suite computes an energy, RMS, peak or mean-absolute value of any synthesized buffer: ripgrep for energy/max_amp/rms/abs over tests/*.tcyr returns only the f64_abs tolerance helpers in bridge/spatial/tract/fatigue/vocalization and the DC-blocker check at tests/dsp.tcyr:34. Every output-value assertion in voice.tcyr, stream.tcyr, tract.tcyr and sequence.tcyr is one of: non-error, exact length, all-finite, or bit-identical determinism. Consequently the entire amplitude stage (src/voice.cyr:1006-1011, the `amp = amplitude_scale * effort_amp` multiply over all samples) is unobserved end-to-end — the amplitude_scale constants are asserted as a lookup table (vocalization.tcyr:24, :32) but nothing asserts they change a single emitted sample. Property 2 is also uncovered: PRANI_INTENT_ALARM never reaches a successful crvoice_vocalize_with_intent call anywhere in the suites (its only appearance, hardening.tcyr:67, asserts an error return).

**Closes with** — Add a group to tests/voice.tcyr, e.g. after "vocalize_with_intent -- intent scales duration (Mating x2)" (voice.tcyr:239): a `t_energy(v)` helper alongside `t_all_finite` that returns the f64 sum of `f64_mul(s, s)` over the vec, then `var i_out = crvoice_vocalize_with_intent(crvoice_new(PRANI_SP_WOLF), PRANI_VOC_HOWL, PRANI_INTENT_IDLE, T_SR, T_DUR_005); var a_out = crvoice_vocalize_with_intent(crvoice_new(PRANI_SP_WOLF), PRANI_VOC_HOWL, PRANI_INTENT_ALARM, T_SR, T_DUR_005); assert_eq(prani_is_err(i_out), 0, "wolf howl idle non-error"); assert_eq(prani_is_err(a_out), 0, "wolf howl alarm non-error"); assert_eq(vec_len(a_out), 1543, "alarm (x0.7 duration) length"); assert_eq(f64_gt(t_energy(a_out), t_energy(i_out)), 1, "alarm carries more energy than idle");`. The same t_energy helper would also let tests/voice.tcyr recover the `max_amp > 0.001` audibility check that test_wolf_howl (integration.rs:11-12) asserts and the port likewise drops.

**Checked for a defect** — NOT a defect — a pure TEST gap. I diffed the two implementations at the amplitude stage and they are line-for-line identical in semantics. Rust, 2.0.3:rust-old/src/voice.rs:378-382: let amp = modifiers.amplitude_scale * effort_amp; for s in &mut samples { *s *= amp; } Cyrius, src/voice.cyr:1005-1011: var amp = f64_mul(IntentModifiers_amplitude_scale(modifiers), effort_amp); while (ai < total) { vec_set(samples, ai, f64_mul(vec_get(samples, ai), amp)); ai = ai + 1; } The three inputs to that expression are also faithful: 1. effort_amp — 2.0.3:rust-old/src/voice.rs:196 `0.3 + effort * 1.2` vs src/voice.cyr:792 `f64_add(PR_V_0_3, f64_mul(effort, PR_V_1_2))`. Same, and effort is identical for both renders (same voice, default 0.5). 2. amplitude_scale — the intent table matches exactly: Rust Alarm 1.5 / Idle 0.5 (2.0.3:rust-old/src/vocalization.rs:87, :111) vs Cyrius src/vocalization.cyr:105-107 (Alarm PR_F_1_ …

### test_all_species_synthesize

**🟡 partial** · `2.0.3:rust-old/tests/integration.rs:94-125`

**Missing** — Only 7 of the 13 species ever reach crvoice_vocalize: Wolf, Lion, Songbird, Snake, Cat, Cricket, Dragon (tests/voice.tcyr:197-236). Dog, Crow, Raptor, Crocodilian and Fantasy are NEVER synthesized and never even construct a CreatureTract anywhere — the complete set of crtract_new call sites in the suites is tests/tract.tcyr:79, 88, 100, 109, 118, 129, 143, 152, 163, 173, 174, 179, 180, 211, 229 and tests/hardening.tcyr:46-59, 133, covering only Wolf/Songbird/Snake/Cricket/Bee/Cat. Bee reaches only the tract layer (tract.tcyr:152-158), never crvoice_vocalize, so the VIBRATILE apparatus has zero voice-level coverage — no assertion anywhere runs a Vibratile species through the pitch contour, post-processing chain, tilt, amplitude and envelope. Fantasy is worse: it is reached only by the default fallthrough arm (src/species.cyr:317-323 'PRANI_SP_FANTASY (default / fallthrough)') and NO assertion in any suite reads a single Fantasy SpeciesParams field, so a wrong Fantasy row is invisible to the entire test suite. Property 5 is uncovered in full: GROWL and CHIRP are never passed to crvoice_vocalize or crvoice_vocalize_with_intent anywhere — GROWL appears only as a pitch-contour input (voice.tcyr:155-156, stream.tcyr:91) and a spectral-offset table lookup (voice.tcyr:162), CHIRP only in emotion selection (emotion.tcyr:109) and CallElement construction/serde (sequence.tcyr:27-28, :144). Property 2's dispatch outcomes that the oracle actually depends on are also unasserted: nothing asserts that Wolf/Crow/Raptor/Crocodilian/Dragon/Fantasy support Growl, that Cricket and Bee support Chirp, or that Bee and Cricket reject Growl — species.tcyr's matrix happens to check different cells.

**Closes with** — Add a group to tests/voice.tcyr mirroring the oracle's loop exactly — "vocalize -- every species synthesizes with an apparatus-compatible voc (all 13)": iterate `sp` over 0..PRANI_SP_COUNT, select `v = PRANI_VOC_GROWL` if `species_supports_vocalization(sp, PRANI_VOC_GROWL) == 1`, else `PRANI_VOC_HISS` if supported, else `PRANI_VOC_CHIRP` (the oracle's exact ladder), then `var o = crvoice_vocalize(crvoice_new(sp), v, T_SR, T_DUR_005); assert_eq(prani_is_err(o), 0, ...); assert_eq(vec_len(o), 2205, ...); assert_eq(t_all_finite(o), 1, ...);`. That single group closes properties 1, 3, 4 and 5 at once and is the highest-value addition in this batch. Additionally extend tests/species.tcyr:81's "supports_vocalization matrix" with the cells the oracle's dispatch turns on — `(PRANI_SP_BEE, PRANI_VOC_GROWL) == 0`, `(PRANI_SP_BEE, PRANI_VOC_CHIRP) == 1`, `(PRANI_SP_CRICKET, PRANI_VOC_GROWL) == 0`, `(PRANI_SP_CRICKET, PRANI_VOC_CHIRP) == 1`, `(PRANI_SP_CROCODILIAN, PRANI_VOC_GROWL) == 1`, `(PRANI_SP_FANTASY, PRANI_VOC_GROWL) == 1` — and add a "params -- Fantasy (fallthrough default)" group so the fallthrough arm's values are pinned.

### test_serde_roundtrip_species

**🟡 partial** · `2.0.3:rust-old/tests/integration.rs:127-132`

**Missing** — Two distinct shortfalls. (a) The value the oracle actually round-trips — Species::Dragon — is never round-tripped in any Cyrius suite. Species survives a JSON round trip only as Wolf (voice.tcyr:288, preset.tcyr:255) and Lion (hardening.tcyr:124); PRANI_SP_DRAGON appears in the suites only at preset.tcyr:145/:171/:216-218/:225 (preset table values and preset_build) and hardening.tcyr:160-162 (the FFI RTPC), never through a codec. (b) There is no standalone Species codec at all: grepping `fn .*_to_json\|fn .*_from_json_str` over src/*.cyr yields only crtract (tract.cyr:171/:200), preset (preset.cyr:83/:103), crvoice (voice.cyr:131/:152) and sequence_call_phrase (sequence.cyr:133/:154), plus the #derive(Serialize) auto codecs for SpeciesParams / IntentModifiers / FatigueModifiers / CallElement / CallBout. So the property is only ever observable through a container struct, never on the enum alone the way the oracle tests it.

**Closes with** — Cheapest closure of (a): in tests/preset.tcyr's "VoicePreset serde roundtrip" group (preset.tcyr:248), add a second round trip built from `preset_ancient_dragon()` and assert `VoicePreset_species(pre_d2) == PRANI_SP_DRAGON` plus json idempotence; or equivalently in tests/voice.tcyr's "CreatureVoice serde roundtrip" group (voice.tcyr:279) add `var vd = crvoice_new(PRANI_SP_DRAGON); ... assert_eq(crvoice_species(vd2), PRANI_SP_DRAGON, "Dragon species roundtrips")`. Shortfall (b) needs no fix — see notes.

**Checked for a defect** — Not a defect — a TEST gap, and a mild one, because the missing piece is a Cyrius representational non-analogue plus one untested discriminant. The standalone-codec half is structurally N/A rather than a divergence. In Rust, Species is a type (2.0.3:rust-old/src/species.rs:15) and serde attaches a codec to it, so `to_string(&Species::Dragon)` is meaningful. In the Cyrius port, Species is not a type at all — it is a block of plain i64 constants (src/species.cyr:26-42, `var PRANI_SP_DRAGON = 11`). There is no type to hang a codec on, and serializing an i64 standalone is `str_builder_add_int`, so a `Species_to_json` would be a no-op wrapper asserting nothing about prani. I cannot point at two divergent lines of behaviour here, because the Cyrius side has no corresponding code path — this is the same category as the Send/Sync and Display non-analogues. The value-level half IS a real, testable prope …

### test_serde_roundtrip_creature_voice

**🟡 partial** · `2.0.3:rust-old/tests/integration.rs:149-159`

**Missing** — Two of the three Rust assertions have no counterpart. (1) Nothing asserts crvoice_effective_f0 on a DESERIALIZED voice — the closest assertions (tests/voice.tcyr:101-107, tests/preset.tcyr:183-233, tests/hardening.tcyr:167) all compute it on freshly built or FFI-mutated voices, never on one that came back from JSON. (2) Nothing asserts crvoice_effective_tract_scale on a deserialized voice either (tests/voice.tcyr:261-264 asserts it only on fresh voices), and the nested SpeciesParams field it reads — tract_scale — is not asserted to roundtrip in ANY suite (tests/species.tcyr:126-130 covers apparatus/f0_default/formant0/spectral_tilt/resonance_seed; tests/tract.tcyr:240 covers formant0 only). Same for f0_min/f0_max, which effective_f0's clamp depends on. (3) The port's roundtrip never sets a non-default f0_offset, so tests/voice.tcyr:289 is the vacuous 0 == 0 — a codec that dropped f0_offset would still pass. The port compensates in a different direction (it also covers vocal_effort, which the Rust test does not).

**Closes with** — In tests/voice.tcyr's "CreatureVoice serde roundtrip" group: (a) add `crvoice_with_f0_offset(vser, f64_from(50));` to the builder chain so f0_offset is asserted at a non-default value; (b) add `assert_eq(crvoice_effective_f0(v2), crvoice_effective_f0(vser), "effective_f0 survives the roundtrip");` and `assert_eq(crvoice_effective_tract_scale(v2), crvoice_effective_tract_scale(vser), "effective_tract_scale survives the roundtrip");`. Optionally also add `assert_eq(SpeciesParams_tract_scale(sp2), SpeciesParams_tract_scale(sp), ...)` to tests/species.tcyr's SpeciesParams roundtrip group so tract_scale has a direct assertion too.

### test_invalid_species_vocalization_rejected

**🟡 partial** · `2.0.3:rust-old/tests/integration.rs:162-174`

**Missing** — Only ONE of the three species/vocalization pairs is asserted end-to-end through crvoice_vocalize: Snake+Howl. Cricket+Roar and Wolf+Stridulate are asserted only at the predicate level (tests/species.tcyr:88, :92); no assertion anywhere calls crvoice_vocalize (or crvoice_vocalize_with_intent) with those two pairs. Every crvoice_vocalize/stream_new/bout rejection assertion in the whole suite uses Snake+Howl. The composed property is covered by construction (src/voice.cyr:780-782 delegates to species_supports_vocalization for every species, and species.tcyr covers the matrix values), but the Rust test's point is that all three apparatus classes are rejected at the public entry point, and two of the three are never exercised there.

**Closes with** — In tests/voice.tcyr's "supports rejection -> error (snake howl)" group, add two assertions alongside the snake case: `assert_eq(crvoice_vocalize(crvoice_new(PRANI_SP_CRICKET), PRANI_VOC_ROAR, T_SR, T_DUR_005), PRANI_ERR_INVALID_VOCALIZATION, "cricket roar -> INVALID_VOCALIZATION");` and `assert_eq(crvoice_vocalize(crvoice_new(PRANI_SP_WOLF), PRANI_VOC_STRIDULATE, T_SR, T_DUR_005), PRANI_ERR_INVALID_VOCALIZATION, "wolf stridulate -> INVALID_VOCALIZATION");` (rename the group to drop "(snake howl)").

**Checked for a defect** — No defect. I read both sides of the two lines that decide this behaviour and they are equivalent. The gate is 2.0.3:rust-old/src/voice.rs:176-184 versus /home/macro/Repos/prani/src/voice.cyr:780-782 — in both, the supports_vocalization check is the FIRST statement of vocalize_with_intent and short-circuits to an error before sample_rate, duration, num_samples, or tract construction are used. The predicate it delegates to is likewise a faithful arm-for-arm port (2.0.3:rust-old/src/species.rs:302-322 vs /home/macro/Repos/prani/src/species.cyr:338-360): Laryngeal/Syringeal explicitly reject Stridulate and Buzz (species.cyr:357-358), and Stridulatory admits only Stridulate\|Buzz\|Chirp so Roar falls through to `return 0` (species.cyr:345-350). Wolf's apparatus is pinned Laryngeal at tests/species.tcyr:30 and both uncovered predicate values are pinned at tests/species.tcyr:88 and :92. So Cricket+Roar and …

### test_species_valid_vocalizations

**🟡 partial** · `2.0.3:rust-old/tests/integration.rs:177-190`

**Missing** — 5 of the 7 asserted pairs are covered exactly; 2 are not. (a) Wolf+Growl (2.0.3:rust-old/tests/integration.rs:180) is asserted nowhere — no suite calls species_supports_vocalization(PRANI_SP_WOLF, PRANI_VOC_GROWL). This is low-severity: it lands on the same laryngeal fall-through return as Wolf+Howl (src/species.cyr:359), so no untested branch is involved. (b) Cricket+Chirp (2.0.3:rust-old/tests/integration.rs:188) is asserted nowhere, and this one DOES leave an untested branch: the Stridulatory arm's Chirp case is a distinct literal test at src/species.cyr:348 that no assertion in any suite reaches (species.tcyr:87 exercises the Stridulate case at :346; nothing exercises the Buzz case at :347 or Chirp at :348). The Vibratile Chirp case at src/species.cyr:353 is likewise unexercised (species.tcyr:94 covers Buzz only).

**Closes with** — Add to tests/species.tcyr's "supports_vocalization matrix" group: `assert_eq(species_supports_vocalization(PRANI_SP_WOLF, PRANI_VOC_GROWL), 1, "wolf growls");` and `assert_eq(species_supports_vocalization(PRANI_SP_CRICKET, PRANI_VOC_CHIRP), 1, "cricket chirps");`. While there, the untested sibling arms are cheap to close too: `species_supports_vocalization(PRANI_SP_CRICKET, PRANI_VOC_BUZZ) == 1` (src/species.cyr:347) and `species_supports_vocalization(PRANI_SP_BEE, PRANI_VOC_CHIRP) == 1` (src/species.cyr:353).

**Checked for a defect** — Not a defect — a test gap. I read both implementations side by side and they are structurally identical, arm for arm: 2.0.3:rust-old/src/species.rs:303-320 dispatches on self.params().apparatus with NoiseOnly = Hiss\|Growl, Stridulatory = Stridulate\|Buzz\|Chirp, Vibratile = Buzz\|Chirp, Laryngeal\|Syringeal = !(Stridulate\|Buzz); src/species.cyr:339-359 does the same with early returns in the same order and the same membership sets. The two inputs the Rust asserts and the Cyrius never exercises would both return true in the port: Cricket maps to PRANI_APP_STRIDULATORY (src/species.cyr:297, matching 2.0.3:rust-old/src/species.rs:242), PRANI_VOC_CHIRP = 5 (src/vocalization.cyr:20) matches Rust's Chirp discriminant (2.0.3:rust-old/src/vocalization.rs:12-24 — Howl 0, Bark 1, Growl 2, Roar 3, Hiss 4, Chirp 5), and src/species.cyr:348 returns 1 for it; Wolf is PRANI_APP_LARYNGEAL (src/species.cyr:234) and Grow …

### test_parameter_clamping

**🟡 partial** · `2.0.3:rust-old/tests/integration.rs:193-203`

**Missing** — Three of the five properties are unasserted, including the only explicit assertion in the Rust test. (1) crvoice_with_jitter (src/voice.cyr:191-196) has ZERO test callers — grep for `crvoice_with_jitter` across /home/macro/Repos/prani/tests, /src and /docs returns only its own definition. Its clamp ceiling PR_V_0_05 is never checked. (2) crvoice_with_shimmer (src/voice.cyr:198-203) likewise has ZERO test callers; its [0, 0.1] clamp is never checked. Both are public API. (3) Nothing anywhere synthesizes from a voice whose builder parameters were pushed out of range: every crvoice_vocalize assertion in tests/voice.tcyr:195-236 and tests/prani.tcyr:38-45 uses a default-constructed voice, and the one test that does mutate a voice before vocalizing (tests/voice.tcyr:267-273) discards the result and asserts only that params were not mutated. So "clamped extremes still synthesize successfully" — the whole point of the Rust test — is asserted nowhere. Partial credit only for the breathiness and under-min size clamps.

**Closes with** — (a) Extend tests/voice.tcyr's "builder clamps" group with the two missing builders: `var v_ji = crvoice_with_jitter(crvoice_new(PRANI_SP_WOLF), T_1_0); assert_eq(SpeciesParams_jitter(CreatureVoice_params(v_ji)), PR_V_0_05, "with_jitter clamps 1.0 -> 0.05");` plus a 0-floor case, and the same shape for `crvoice_with_shimmer` clamping 1.0 -> PR_V_0_1; also assert `crvoice_with_size(v, f64_neg(T_1_0))` -> 0.1 to cover the negative (not merely zero) size. (b) Add a new group porting the oracle's headline assertion: build one Wolf voice with breathiness 5.0, size -1.0, jitter 1.0, shimmer 1.0 chained, then `var out = crvoice_vocalize(v, PRANI_VOC_HOWL, T_SR, T_0_3);` and assert prani_is_err(out) == 0, vec_len(out) == 13230, t_all_finite(out) == 1.

**Checked for a defect** — Not a defect — a pure TEST gap. I compared every relevant line pair and found no divergence to point at. The four clamp ranges are bit-for-bit the same (src/voice.cyr:178-203 vs 2.0.3:rust-old/src/voice.rs:63-88, with PR_V_0_05 = 0x3fa999999999999a = 0.05 at src/voice.cyr:79 and PR_V_0_1 = 0x3fb999999999999a = 0.1 at src/voice.cyr:52), and f64_clamp (lib/math.cyr:410-414) has the same semantics as Rust's f32::clamp for these inputs since f64_lt/f64_gt are IEEE-754 compiler builtins, not raw integer comparisons — so the oracle's negative -1.0 and the port test's 0 both take the `return lo` branch. The unasserted headline property ("clamped extremes still synthesize") also holds by construction: none of breathiness/size/jitter/shimmer feed crtract_new (src/tract.cyr:114-155 uses only sample_rate + formants), 44100 clears the ADR-0001 sample-rate check, the clamped Wolf f0 of 1200 Hz stays inside …

### test_serde_roundtrip_vocal_apparatus

**🟡 partial** · `2.0.3:rust-old/tests/integration.rs:214-229`

**Missing** — Only 1 of the 5 apparatus variants is ever round-tripped through the codec, and it is the one variant that cannot detect a failure. The port has no standalone VocalApparatus codec: apparatus survives serde only as field 0 of SpeciesParams (src/species.cyr:159-160, typed i64), written by `#derive(Serialize)` and read back by name at src/species.cyr:415 (`species_params_from_node`). The single covering assertion (tests/species.tcyr:126) uses Wolf, whose apparatus is PRANI_APP_LARYNGEAL == 0 — and bayan's accessors are null-safe by design (the premise of ADR-0002): `bayan_json_v_int(bayan_json_v_obj_get(node, "apparatus"))` returns 0 for a missing, misspelled or mistyped key. A deserializer that dropped the apparatus key entirely would pass that assertion. SYRINGEAL(1), STRIDULATORY(2), VIBRATILE(3) and NOISE_ONLY(4) are never round-tripped anywhere: the other four serde suites all use laryngeal species too — tests/voice.tcyr:280 (Wolf), tests/tract.tcyr:229 (Wolf), tests/hardening.tcyr:118 (Lion), and tests/preset.tcyr:248-261 (VoicePreset has no apparatus field).

**Closes with** — In tests/species.tcyr, extend the "SpeciesParams serde roundtrip" group (:120-133) to cover one species per apparatus, asserting the round-tripped value against the expected NON-ZERO constant so a dropped key cannot pass: Songbird -> `assert_eq(SpeciesParams_apparatus(SpeciesParams_from_json_str(j_bird)), PRANI_APP_SYRINGEAL, ...)`, Cricket -> PRANI_APP_STRIDULATORY, Bee -> PRANI_APP_VIBRATILE, Snake -> PRANI_APP_NOISE_ONLY, keeping the existing Wolf/LARYNGEAL case. Four extra `species_params` + `SpeciesParams_to_json` + `SpeciesParams_from_json_str` triples; no source change.

**Checked for a defect** — This is a TEST gap, not a behavioural difference — and I settled it empirically rather than by reading alone. Code reading first: the round trip is name-keyed and type-uniform. The writer is the `#derive(Serialize)` on `struct SpeciesParams { apparatus: i64; ... }` (src/species.cyr:157-176), which emits `apparatus` as a plain integer like every other i64 field. Both readers fetch it by key: `species_params_from_node` at src/species.cyr:415, and the derive-generated `SpeciesParams_from_json_str`. There is no per-value branch anywhere on the codec path, so no mechanism by which 0 could survive while 1..4 fail. I found no divergent pair of lines against 2.0.3:rust-old/src/species.rs:56-67 (Rust's `#[derive(Serialize, Deserialize)]` on the same enum) — only the documented format divergence (string variant name vs integer), which is not a behavioural defect. Empirical confirmation: I wrote a throwa …

### test_serde_roundtrip_error

**⬜ n/a** · `2.0.3:rust-old/tests/integration.rs:231-237`

N/A on both halves of the Rust assertion, and I want to be explicit about the reasoning rather than wave it through. (1) There is no `PraniError` TYPE in the port to round-trip. src/error.cyr:23-28 declares five plain i64 constants; src/error.cyr:6-8 records the decision in the module header: "Rust carried a String payload per variant; the Cyrius port uses integer codes -- fallible functions return PRANI_ERR_NONE (0) on success or a negative code on failure; diagnostic text via prani_err_name()". An i64 IS its own serialization — there is no encoder, no decoder, and nothing that a round-trip could lose, so `#derive(Serialize)` was (correctly) not applied to anything here. Note the contrast with the other three rows in this batch: SpeciesParams, IntentModifiers and CreatureTract all DID get their serde restored in 2.0.1 because they are structs with state; PraniError did not because it has none. (2) `err.to_string()` has no analogue — Cyrius has no `Display` trait, and the brief lists Display-string tests as deliberately dropped. `prani_err_name` (asserted at tests/error.tcyr:29-32) is the nearest thing and it is covered. The one genuine information loss versus the oracle is the per-error `String` context: `PraniError::Svara(format!("{e}"))` (2.0.3:rust-old/src/error.rs:31-34) carried svara's formatted message, and `prani_from_svara` (src/error.cyr:61-64) collapses every non-zero svara code to the bare PRANI_ERR_SVARA. That loss is real but it is a consequence of the integer-code design the whole port rests on, it is documented in the module header, it is not observable through any prani API (nothing returns a payload to compare), and tests/error.tcyr:36-38 asserts the collapse deliberately. So: nothing to assert, no gap.

### test_serde_roundtrip_creature_tract

**🟡 partial** · `2.0.3:rust-old/tests/integration.rs:250-258`

**Missing** — The port's CreatureTract JSON is a strictly NARROWER field set than the oracle's, so the oracle's `json == json2` covers state the port's idempotency assertion cannot reach — and the port's behaviour on that state genuinely differs. `crtract_to_json` (src/tract.cyr:171-192) emits exactly eight things: sample_rate, phase, rng_state, rng_inc, dcb_x_prev, dcb_y_prev, dcb_r, params. The svara `VocalTract` and the naad `noise_filter` are NOT serialized. `crtract_from_json_str` (src/tract.cyr:200-219) calls `crtract_new(params, sample_rate)` to REBUILD both from the deserialized SpeciesParams, then overwrites phase/rng/dc_blocker with the saved values. The oracle serialized svara's VocalTract in full and restored it. Consequence: a consumer that serializes a mid-call tract and restores it resumes with CLEARED svara filter memory (and, for a NoiseOnly species, a freshly-built naad bandpass), so the restored tract's continuation diverges from what the original would have produced — the oracle resumed seamlessly. Nothing in the suite asserts either the old behaviour or the new one; tests/tract.tcyr:241 only checks the rebuilt handle is non-null and :248 only that it produces a finite sample. Secondary, smaller test gap in the same group: dcb_x_prev and dcb_y_prev ARE serialized (src/tract.cyr:182-185) and ARE advanced by the purr at tests/tract.tcyr:230, but only `DcBlocker_r` is asserted to round-trip (:239) — and r is the one field the constructor sets to a constant, so it would survive even if the restore were dropped entirely.

**Closes with** — Three parts. (1) Record the divergence as an ADR alongside 0001-0003 — "a serialized CreatureTract restores prani-side state and REBUILDS the svara/naad DSP state" — stating that svara exposes the tract only as an opaque handle with no state accessors, so preserving it is not available to the port (same root cause as ADR-0001), and that the observable cost is a discontinuity on resume. (2) Add to tests/tract.tcyr's "CreatureTract serde roundtrip" group an assertion that PINS the new behaviour rather than leaving it unstated: advance `tser`, round-trip to `t2`, then compare `crtract_synthesize_purr(t2, 16, 27)` against a freshly-built tract given the same restored phase/rng/dc_blocker — they should be bit-identical (proving the svara side is rebuilt, not restored) — and note in the group comment that this is where the port parts company with the oracle. (3) Add `assert_eq(DcBlocker_x_prev(CreatureTract_dc_blocker(t2)), DcBlocker_x_prev(CreatureTract_dc_blocker(tser)), ...)` and the same for `_y_prev` next to tests/tract.tcyr:239.

### test_zero_duration_synthesis

**🔴 gap** · `2.0.3:rust-old/tests/integration.rs:260-265`

**Missing** — Nothing in any of the 17 suites calls a synthesis entry point with a zero duration. I enumerated every `crvoice_vocalize` / `crvoice_vocalize_with_intent` call site in tests/: tests/voice.tcyr:190, 197, 203, 209, 215, 221, 227, 233, 241, 250, 251, 254, 255, 272; tests/prani.tcyr:39, 48; tests/hardening.tcyr:64, 67, 71, 74, 220, 222. Every one passes T_DUR_005 (0.05 s), T_DUR_003 (0.03 s), F64_ONE or DUR — never 0. The stream and sequence entry points are the same story (tests/stream.tcyr:64, 73, 80, 96, 122, 139, 154, 167-168; tests/sequence.tcyr:54-59). So neither property is asserted: not "zero duration is non-error" and not "the output is empty". The nearest empty-output assertion in the tree is tests/stream.tcyr:134 (`vec_len(blk3) == 0`, "next_block on finished stream -> empty vec") — a different function reaching empty for a different reason (a drained stream), which does not exercise the zero-num_samples path through crvoice_vocalize_with_intent at all.

**Closes with** — Add a group to tests/voice.tcyr, next to the existing "vocalize -- non-error, length, finite" group (:195): test_group("zero duration -- non-error, empty output"); var z = crvoice_vocalize(crvoice_new(PRANI_SP_WOLF), PRANI_VOC_HOWL, T_SR, 0); assert_eq(prani_is_err(z), 0, "duration 0 is not an error"); assert_eq(vec_len(z), 0, "duration 0 -> empty output (num_samples.max(1) does not leak)"); Worth adding the cat-purr twin in the same group, since src/voice.cyr:800-804 routes PURR out through a SEPARATE exit (`crvoice_synthesize_cat_purr`) that the Wolf case never touches: var zp = crvoice_vocalize(crvoice_new(PRANI_SP_CAT), PRANI_VOC_PURR, T_SR, 0); assert_eq(prani_is_err(zp), 0, "cat purr duration 0 is not an error"); assert_eq(vec_len(zp), 0, "cat purr duration 0 -> empty output");

**Checked for a defect** — NOT a defect — I traced both implementations line by line and they agree on the zero-duration path. Rust (2.0.3:rust-old/src/voice.rs:152-159, 189, 227-231): vocalize forwards CallIntent::Idle; num_samples = (effective_duration * sample_rate) as usize = (0.0 * 44100.0) as usize = 0; `while rendered < num_samples` never iterates; `Vec::with_capacity(0)` stays empty; the num_samples.max(1) uses at :232, :285, :316, :339 are divisor guards only and never touch the length. Cyrius (src/voice.cyr:765-766, 788, 841-842, 848-857): crvoice_vocalize forwards PRANI_INTENT_IDLE (:766, matching Rust); num_samples = f64_to(f64_mul(effective_duration, sample_rate)) = f64_to(0.0) = 0 (:788); num_samples_max is a separate variable floored at 1 (:841-842) used ONLY as the normalized-time divisor at :852, :918, :947, :969 — exactly the oracle's max(1), and it does not feed the loop bound; `while (rendered < num_ …

### test_bee_buzz

**🟡 partial** · `2.0.3:rust-old/tests/integration.rs:279-285`

**Missing** — Nothing anywhere in the 17 suites calls crvoice_vocalize (or crvoice_vocalize_with_intent) on a Bee voice — PRANI_SP_BEE appears exactly once in the whole test tree, at tests/tract.tcyr:151, and only as an argument to species_params(). So none of the Rust test's three assertions is reproduced on the same code path: no non-error result, no non-empty/length assertion, and no all-finite assertion for crvoice_vocalize(PRANI_SP_BEE, PRANI_VOC_BUZZ, 44100, 0.3). The tract-level cover at tests/tract.tcyr:153-158 skips everything the voice layer adds: the 20 ms block loop and its per-block pitch-contour/perturbation arming (src/voice.cyr:867-908), the BUZZ arm of crvoice_vocalization_spectral_offset (falls through to 0 at src/voice.cyr:636) feeding crtract_apply_spectral_tilt, the intent amplitude scaling, and the BUZZ arm of crvoice_apply_vocalization_envelope (0.05/0.05, src/voice.cyr:694-695) — which is the only place the Buzz envelope constants are ever selected and is currently dead in test terms.

**Closes with** — Add to the "vocalize -- non-error, length, finite (multiple species)" group in tests/voice.tcyr (alongside the existing wolf/lion/songbird/snake/cat/cricket/dragon cases at lines 197-236): `var bee_out = crvoice_vocalize(crvoice_new(PRANI_SP_BEE), PRANI_VOC_BUZZ, T_SR, T_DUR_005);` then `assert_eq(prani_is_err(bee_out), 0, "bee buzz non-error"); assert_eq(vec_len(bee_out), 2205, "bee buzz length == 2205"); assert_eq(t_all_finite(bee_out), 1, "bee buzz all finite");`. The suite's existing shortened duration (0.05 s instead of the oracle's 0.3 s) is consistent with how every other species case there is written.

**Checked for a defect** — Not a defect — I compared every line the Bee/Buzz path touches and they match the oracle. 1. Species params: /home/macro/Repos/prani/src/species.cyr:303-308 vs 2.0.3:rust-old/src/species.rs:254-266 — apparatus Vibratile, f0 200/500/300, tract_scale 0.03, formants 300/600/900, bandwidths 100/200/300, breathiness 0.0, jitter 0.001, shimmer 0.003, tilt -1.0. Identical. 2. supports_vocalization Vibratile arm: /home/macro/Repos/prani/src/species.cyr:351-354 (Buzz\|Chirp -> 1) vs 2.0.3:rust-old/src/species.rs:313-316. Identical, so the Ok(..) precondition holds. 3. Apparatus dispatch: /home/macro/Repos/prani/src/tract.cyr:449-464 vs 2.0.3:rust-old/src/tract.rs:110-116 — Vibratile is the fall-through arm in both. 4. Vibratile synth: /home/macro/Repos/prani/src/tract.cyr:417-433 vs 2.0.3:rust-old/src/tract.rs:381-394 — mod_r …

### test_crow_screech

**🔴 gap** · `2.0.3:rust-old/tests/integration.rs:287-295`

**Missing** — None of the Rust test's assertions is reproduced. crvoice_vocalize is never called with PRANI_SP_CROW (the only Crow voice built anywhere, tests/preset.tcyr:220, is only inspected for species/tract_scale/effective_f0), and PRANI_VOC_SCREECH is never passed to any synthesis function in any of the 17 suites — its only two appearances (tests/emotion.tcyr:102,185) are emotion-to-vocalization selection. Consequently three Screech-specific implementation arms are never executed by any test: crvoice_pitch_contour's SCREECH points (src/voice.cyr:366-370), crvoice_vocalization_spectral_offset's SCREECH +1.5 (src/voice.cyr:632), and crvoice_apply_vocalization_envelope's SCREECH (0.03, 0.15) (src/voice.cyr:698-699). The Crow parameter set is likewise never driven through a tract: the only syringeal synthesis anywhere is Songbird (tests/tract.tcyr:102, 110 and tests/voice.tcyr:209-212).

**Closes with** — Add a Crow case to the "vocalize -- non-error, length, finite (multiple species)" group in tests/voice.tcyr:195-236: `var cw_out = crvoice_vocalize(crvoice_new(PRANI_SP_CROW), PRANI_VOC_SCREECH, T_SR, T_DUR_005);` with `assert_eq(prani_is_err(cw_out), 0, ...)`, `assert_eq(vec_len(cw_out), 2205, ...)`, `assert_eq(t_all_finite(cw_out), 1, ...)`. Also add the two missing table assertions to the existing groups: `assert_eq(crvoice_vocalization_spectral_offset(PRANI_VOC_SCREECH), <1.5 hex>, "screech offset +1.5")` in the "vocalization_spectral_offset -- table" group (tests/voice.tcyr:161-164), and SCREECH f0_at endpoints (base*1.5 at t=0, base*0.7 at t=1) in the "pitch_contour f0_at" group (tests/voice.tcyr:148-158), which currently only covers HOWL and FLAT.

### test_all_intents_modify_differently

**🟡 partial** · `2.0.3:rust-old/tests/integration.rs:297-320`

**Missing** — The distinctness property itself — the only thing this Rust test asserts — has no Cyrius counterpart anywhere: I grepped all 17 suites for prani_intent_modifiers and it appears only at tests/vocalization.tcyr:22, 30, 37, 39, 41, 57, always as a single-intent lookup, never as a pairwise comparison. Two of the seven intents are never asserted AT ALL: PRANI_INTENT_TERRITORIAL's modifiers (0.9/1.3/1.5/0.5) and PRANI_INTENT_SOCIAL's modifiers (1.0/0.8/1.0/0.3) have no assertion in any suite (the only other appearances of those constants are as arguments to stream_new / sequence_call_bout_new / emotion_select_intent, which never inspect the modifier tuple). Of the 28 field values in the table, only 9 are asserted (Alarm x4, Idle x3, Mating x1, Distress x1, Threat x1) — and Idle's duration_scale 1.0 is missing even though the group is titled as covering it. Because only Alarm has a full 4-tuple pinned, the exact values do not imply pairwise distinctness for the other 20 pairs.

**Closes with** — In tests/vocalization.tcyr, add a distinctness group that mirrors the oracle's double loop: a helper `fn t_mods_same(a, b) { return (IntentModifiers_pitch_scale(a) == IntentModifiers_pitch_scale(b)) & (IntentModifiers_amplitude_scale(a) == IntentModifiers_amplitude_scale(b)) & (IntentModifiers_duration_scale(a) == IntentModifiers_duration_scale(b)) & (IntentModifiers_urgency(a) == IntentModifiers_urgency(b)); }` then a nested `while` over intent 0..6 x j..6 asserting `assert_eq(t_mods_same(prani_intent_modifiers(i), prani_intent_modifiers(j)), 0, ...)` for all 21 pairs. Exact i64 bit-pattern equality is a legitimate and STRONGER substitute for the oracle's `< f32::EPSILON` test since all seven tuples are compile-time constants. Additionally pin the two missing tuples explicitly — Territorial (PR_F_0_9, PR_F_1_3, PR_F_1_5, PR_F_0_5) and Social (PR_F_1_0, PR_F_0_8, PR_F_1_0, PR_F_0_3) — and add Idle's duration_scale == PR_F_1_0.

### test_crocodilian_rumble_with_subharmonics

**🔴 gap** · `2.0.3:rust-old/tests/integration.rs:322-330`

**Missing** — Two independent holes. (a) SPECIES: crvoice_vocalize is never called on a Crocodilian voice, and PRANI_VOC_RUMBLE is never synthesized by any test. So the Rumble arms of crvoice_pitch_contour (src/voice.cyr:359-360, aliased to the ROAR points), crvoice_vocalization_spectral_offset (src/voice.cyr:630, -2.0), and crvoice_apply_vocalization_envelope (src/voice.cyr:682-683, 0.1/0.1) are never executed, and the Crocodilian parameter set (notably tract_scale 2.5 and the very low F1 = 200 with f0 = 60) is never driven through svara. (b) AUDIBILITY — the more serious one, and it is suite-wide, not just this test. `max_amp > 0.001` has NO analogue anywhere in the Cyrius suites. I grepped all 17 for f64_abs/max/amplitude assertions: every occurrence (tests/dsp.tcyr:34, tests/bridge.tcyr:29-136, tests/spatial.tcyr:31-128, tests/fatigue.tcyr:136, tests/vocalization.tcyr:65, tests/tract.tcyr:73) is a \|a-b\| < tolerance closeness check, not an output-magnitude floor. Every synthesis assertion in the port is non-error + exact length + all-finite + bit-identical determinism — all four of which an identically-zero buffer satisfies. This is exactly the strength regression the milestone brief warns about, and it silently drops the one property this Rust test was written to protect: that the post-tract subharmonic injection compensates for svara's formant attenuation of a sub-F1 fundamental.

**Closes with** — Two additions to tests/voice.tcyr. First, a max-amplitude helper next to t_all_finite (tests/voice.tcyr:67-75): `fn t_max_abs(v) { var m = 0; var i = 0; while (i < vec_len(v)) { var a = f64_abs(vec_get(v, i)); if (f64_gt(a, m) == 1) { m = a; } i = i + 1; } return m; }`. Second, a Crocodilian case in the "vocalize -- non-error, length, finite" group: `var cr_out = crvoice_vocalize(crvoice_new(PRANI_SP_CROCODILIAN), PRANI_VOC_RUMBLE, T_SR, T_DUR_005);` asserting prani_is_err == 0, vec_len == 2205, t_all_finite == 1, AND `assert_eq(f64_gt(t_max_abs(cr_out), <0.001 hex>), 1, "crocodilian rumble is audible (max_amp > 0.001)")`. Use a duration long enough that the subharmonic envelope's t_norm > 0.15 plateau is reached (0.05 s is fine — the envelope is normalized over num_samples, not wall time). Apply the same t_max_abs floor to the existing wolf-howl case (tests/voice.tcyr:197-200), which drops the identical assertion from 2.0.3:rust-old/tests/integration.rs:11-12, and to the cat-purr case at tests/voice.tcyr:220-224 (oracle floor there is 0.0001).

**Checked for a defect** — NOT a defect — I read both implementations line by line and the Crocodilian Rumble path is a faithful port. This is a pure TEST gap. Subharmonics arm, 2.0.3:rust-old/src/voice.rs:277-307 vs src/voice.cyr:912-940: identical species set (Lion\|Dragon\|Crocodilian — src/voice.cyr:1023-1028 vs voice.rs:279-282), identical sub_f0 = f0*0.5, identical seed `resonance_seed + 0xCA05` (voice.rs:280-281 vs voice.cyr:915), identical envelope breakpoints (t_norm<0.15 -> t/0.15; t_norm>0.8 -> (1-t)/0.2; else 1.0), identical sub_amp = 0.4*sub_env, identical `sin(TAU*sub_f0*t_sec)` add, identical chaos gate sub_env>0.6 with intensity (sub_env-0.6)/0.4 and noise*0.15. I verified the hardcoded seed constant rather than trusting it, since Cyrius is f64-only and a naive port would have hashed f64 bits. species.rs:108-112 computes `(f0_default.to_bits() as u64) ^ ((tract_scale.to_bits() as u64) << 32)`, wrapping_m …

### test_raptor_screech

**🔴 gap** · `2.0.3:rust-old/tests/integration.rs:332-340`

**Missing** — None of the Rust test's assertions is reproduced on the same code path. crvoice_vocalize is never called with PRANI_SP_RAPTOR — the sole Raptor voice in the suites (tests/preset.tcyr:215) is only inspected for species/tract_scale/effective_f0 — and PRANI_VOC_SCREECH is never synthesized anywhere (see the test_crow_screech row: its only appearances are emotion selection at tests/emotion.tcyr:102,185). The dual-source syringeal branch IS covered at the tract layer (tests/tract.tcyr:110-113) but only with Songbird's parameters, and the specific interaction this test targets — a screech contour sweeping ACROSS the 2000 Hz branch boundary within one call, so the tract switches sub-branches mid-synthesis while the source-filter coupling drags F1 by ~660 Hz — is not exercised by any assertion in any suite. There is also no Raptor-parameter synthesis at all: Raptor's F1 = 2100 with B1 = 400 and tract_scale 0.2 never reaches svara.

**Closes with** — Add a Raptor case to the "vocalize -- non-error, length, finite (multiple species)" group in tests/voice.tcyr:195-236: `var rp_out = crvoice_vocalize(crvoice_new(PRANI_SP_RAPTOR), PRANI_VOC_SCREECH, T_SR, T_DUR_005);` with `assert_eq(prani_is_err(rp_out), 0, "raptor screech non-error")`, `assert_eq(vec_len(rp_out), 2205, "raptor screech length == 2205")`, `assert_eq(t_all_finite(rp_out), 1, "raptor screech all finite")`. Pair it with the Crow case suggested in the test_crow_screech row so both syringeal f0 regimes (Crow's sub-2000 glottal, Raptor's above-2000 dual-source) are covered at the voice level. Optionally add the exact effective_f0 assertion `assert_eq(crvoice_effective_f0(crvoice_new(PRANI_SP_RAPTOR)), f64_from(2500), ...)` so the branch-selection precondition (f0 * 1.5 > 2000) is pinned rather than inferred.

### test_dragon_individual_variation

**🟡 partial** · `2.0.3:rust-old/tests/integration.rs:342-354`

**Missing** — Two properties are unasserted. (a) The Dragon-specific ORDERING assertion has no counterpart. The inverse size/pitch relation is pinned exactly but only for Wolf (tests/voice.tcyr:101, 104: 400 at size 1, 200 at size 2, neither clamped). For Dragon the suite has 150 (size 0.8, offset +50) and 30 (size 4.0, offset -30) at tests/preset.tcyr:208, 213 — two different offsets, so the comparison does not isolate size, and nothing asserts an inequality between any two effective_f0 values anywhere in the 17 suites. The oracle's specific pair (0.5 -> 140, 3.0 -> clamped 30) is never computed. (b) No synthesis at a non-default size, for any species. Every crvoice_vocalize / crvoice_vocalize_with_intent call in the suites (tests/voice.tcyr:190, 197, 203, 209, 215, 221, 227, 233, 241, 250-255, 272; tests/hardening.tcyr:64-75, 220-222; tests/prani.tcyr:39, 48) uses a freshly constructed voice at size_scale 1.0, and tests/preset.tcyr builds size-varied voices but never synthesizes with any of them. So "a size-3.0 Dragon, f0 clamped to the 30 Hz floor, still produces finite output" — the more interesting half of the oracle's pair, since 30 Hz sits far below Dragon's F1 of 170 at a 44100 Hz rate — is asserted nowhere.

**Closes with** — Add a group to tests/voice.tcyr next to the existing effective_f0 group (lines 99-107): `var dr_small = crvoice_with_size(crvoice_new(PRANI_SP_DRAGON), T_0_5);` `var dr_large = crvoice_with_size(crvoice_new(PRANI_SP_DRAGON), f64_from(3));` `assert_eq(crvoice_effective_f0(dr_small), f64_from(140), "dragon @ size 0.5 -> 70/0.5 = 140");` `assert_eq(crvoice_effective_f0(dr_large), f64_from(30), "dragon @ size 3.0 -> 70/3 = 23.3 clamps up to f0_min 30");` `assert_lt(crvoice_effective_f0(dr_large), crvoice_effective_f0(dr_small), "smaller dragon has the higher f0");` (assert_lt is already in use at tests/voice.tcyr:171, so the ordering assertion needs no new harness support.) Then add the two synthesis assertions the oracle makes: `var ds1 = crvoice_vocalize(dr_small, PRANI_VOC_ROAR, T_SR, T_DUR_003);` `var ds2 = crvoice_vocalize(dr_large, PRANI_VOC_ROAR, T_SR, T_DUR_003);` with `assert_eq(prani_is_err(ds1), 0, ...)` / `assert_eq(t_all_finite(ds1), 1, ...)` and the same for ds2. Note crvoice_with_size mutates in place, so build dr_small and dr_large from separate crvoice_new calls (as written above) rather than chaining off one voice.

### test_cat_purr_special_synthesis

**🟡 partial** · `2.0.3:rust-old/tests/integration.rs:358-368`

**Missing** — The audibility property (max \|sample\| > 0.0001) is not asserted anywhere. Ripgrep for max_amp / max_abs / audib / amplitude / f64_abs across all 17 .tcyr suites finds f64_abs used only inside tolerance comparisons (bridge.tcyr, fatigue.tcyr, spatial.tcyr, tract.tcyr:73, dsp.tcyr:34) and never over a synthesis buffer; no suite computes a peak or any energy measure of synthesized audio. crvoice_synthesize_cat_purr could return an all-zero buffer and every existing cat-purr assertion (non-error, length 2205, all finite, stream fills) would still pass. Also incidental: the Rust runs 1.0 s, the Cyrius 0.05 s.

**Closes with** — Add a `t_max_abs(v)` helper to tests/voice.tcyr beside t_all_finite (loop f64_abs(vec_get(v,i)), keep the max), then in the "vocalize -- non-error, length, finite (multiple species)" group add `assert_eq(f64_gt(t_max_abs(p_out), 0x3F1A36E2EB1C432D), 1, "cat purr audible (max \|s\| > 1e-4)");`. The same helper then serves the subharmonics row.

### test_cat_purr_size_variation

**🔴 gap** · `2.0.3:rust-old/tests/integration.rs:370-379`

**Missing** — No Cyrius test combines a size-scaled voice with a purr. Grepping crvoice_with_size across all 17 suites returns exactly voice.tcyr:103, 112, 114, 263, 281 (all Wolf) and hardening.tcyr:118, 162, 216 (Lion/Dragon/Wolf) — none is followed by a PURR vocalization. Consequently src/voice.cyr:744-745 `f64_clamp(f64_div(f64_from(27), CreatureVoice_size_scale(self)), f64_from(20), f64_from(35))` is only ever evaluated at size_scale == 1.0, where it returns 27 and NEITHER clamp rail is taken. All four asserted properties (that a size-0.5 and a size-2.0 cat purr synthesize at all, and that both are finite) are entirely unasserted in the port.

**Closes with** — Add a group to tests/voice.tcyr: `var pp_s = crvoice_vocalize(crvoice_with_size(crvoice_new(PRANI_SP_CAT), T_0_5), PRANI_VOC_PURR, T_SR, T_DUR_005);` and `var pp_l = crvoice_vocalize(crvoice_with_size(crvoice_new(PRANI_SP_CAT), T_2_0), PRANI_VOC_PURR, T_SR, T_DUR_005);` asserting for each `prani_is_err == 0`, `vec_len == 2205`, `t_all_finite == 1` (Rust parity). Then go one better than the oracle and pin the mapping the Rust comment describes but never checks: `assert_eq(t_vecs_equal(pp_s, pp_l), 0, "size 0.5 (purr_f0 35) and size 2.0 (purr_f0 20) differ");`

### test_subharmonics_are_finite

**🟡 partial** · `2.0.3:rust-old/tests/integration.rs:381-405`

**Missing** — Three distinct holes. (a) CROCODILIAN IS NEVER SYNTHESIZED ANYWHERE. Grepping PRANI_SP_CROCODILIAN across all 17 suites hits only bridge.tcyr:152-158 (bridge_species_from_f0 mapping) and preset.tcyr:152/172/231 (preset species id) — no crvoice_vocalize, no crvoice_vocalize_with_intent, no stream_new, no crtract_new. Properties 7/8/9 are wholly uncovered, and with them the Rumble vocalization on a subharmonic species and the Crocodilian arm of crvoice_species_has_subharmonics (src/voice.cyr:1023-1028). (b) CallIntent::Territorial never reaches synthesis: grep finds PRANI_INTENT_TERRITORIAL only at bridge.tcyr:49/53/54 (intent_from_threat_level) and species.tcyr:114 (bout template); it is never passed to crvoice_vocalize_with_intent, and tests/vocalization.tcyr never asserts its four modifier fields. Property 11 is uncovered on every species — Lion and Dragon are only ever run at Idle, whose amplitude_scale is 0.5. (c) The max \|sample\| > 0.01 threshold (properties 3/6/9) has no analogue anywhere: no suite computes a peak amplitude over a synthesis buffer (same finding as the cat-purr row).

**Closes with** — In tests/voice.tcyr's synthesis group add a Crocodilian rumble at Territorial: `var cr_out = crvoice_vocalize_with_intent(crvoice_new(PRANI_SP_CROCODILIAN), PRANI_VOC_RUMBLE, PRANI_INTENT_TERRITORIAL, T_SR, T_DUR_005);` asserting prani_is_err == 0, vec_len == (2205 * Territorial duration_scale), t_all_finite == 1; and add Territorial variants of the existing Lion and Dragon roars beside the Idle ones. With the t_max_abs helper from the cat-purr row, assert `f64_gt(t_max_abs(x), 0x3F847AE147AE147B) == 1` (> 0.01) for all three. Separately add a "intent modifiers -- Territorial" group to tests/vocalization.tcyr pinning its four fields, matching the existing Alarm/Idle groups.

**Checked for a defect** — Not a defect — a pure TEST gap, and I verified this empirically rather than by inspection alone. IMPLEMENTATION PARITY (read both sides, line for line): - Branch predicate: 2.0.3:rust-old/src/voice.rs:277-280 `matches!(self.species, Species::Lion \| Species::Dragon \| Species::Crocodilian)` vs src/voice.cyr:913 `if (crvoice_species_has_subharmonics(species) == 1)`, whose body at src/voice.cyr:1023-1028 returns 1 for exactly PRANI_SP_LION / PRANI_SP_DRAGON / PRANI_SP_CROCODILIAN. Same three arms. - Branch body: 2.0.3:rust-old/src/voice.rs:281-308 (sub_f0 = f0*0.5; chaos_rng seeded resonance_seed+0xCA05; sub_env ramp t_norm<0.15 / >0.8 with /0.2; sub_amp = 0.4*sub_env; sin(TAU*sub_f0*t_sec); chaos when sub_env>0.6 with (sub_env-0.6)/0.4 and *0.15) is reproduced statement for statement at src/voice.cyr:914-939. - Pipeline ORDER is the load-bearing thing for the max_amp assertion, and it matches: subhar …

### test_cat_howl_formant_transitions

**🔴 gap** · `2.0.3:rust-old/tests/integration.rs:416-423`

**Missing** — No Cyrius suite ever synthesizes a cat howl. I grepped PRANI_SP_CAT and PRANI_VOC_HOWL across all 17 .tcyr files: every PRANI_SP_CAT synthesis is a PURR (voice.tcyr:221, stream.tcyr:139, hardening.tcyr:71) and every PRANI_VOC_HOWL synthesis is a Wolf (voice.tcyr:197/250/251/272, prani.tcyr:39, stream.tcyr:64/96/122/167/168, hardening.tcyr:64/74/220/222) or a Snake rejection (voice.tcyr:190, prani.tcyr:48). Properties 1, 2 and 3 are unasserted outright. Properties 4 and 5 are covered only as static table lookups: the CAT_MEOW_HOWL keyframes are never fed to crtract_set_formant_blend and the 0.2 nasal fraction is never applied to a sample buffer. This is the only test in this batch whose entire code path is unexecuted by the port's suite.

**Closes with** — In tests/voice.tcyr's "vocalize -- non-error, length, finite (multiple species)" group, add beside the existing cat purr: `var ch_out = crvoice_vocalize(crvoice_new(PRANI_SP_CAT), PRANI_VOC_HOWL, T_SR, T_DUR_005);` with `assert_eq(prani_is_err(ch_out), 0, "cat howl non-error");`, `assert_eq(vec_len(ch_out), 2205, "cat howl length == 2205");`, `assert_eq(t_all_finite(ch_out), 1, "cat howl all finite");`. That restores Rust parity. Worth also adding it to the "determinism" group, since cat howl exercises formant transitions and the nasal path together.

### test_cricket_pulse_train

**🟡 partial** · `2.0.3:rust-old/tests/integration.rs:425-441`

**Missing** — The pulse-train / silence-gap property is not asserted, and the code that produces it is never even reached. At 44100 Hz src/tract.cyr:360-362 computes chirp_period = 44100/2.5 = 17640 and chirp_active = (44100/30)*4 = 5880. Both cricket call sites in the suite render buffers that fit entirely inside the first active chirp — voice.tcyr:227 renders 2205 samples, tract.tcyr:131 renders 128 — so `f64_lt(pos_in_chirp, chirp_active)` (src/tract.cyr:374) is true for every sample rendered by the entire test suite, and the else branch at src/tract.cyr:389-391 (`# inter-chirp silence` / `vec_push(output, 0)`) executes ZERO times. Separately, no suite counts near-silent samples or measures any amplitude of a synthesis buffer. Note tract.tcyr:136's exact-zero assertion is the pulse envelope at t=0 inside the chirp, not the silence branch — it looks like it covers this and does not.

**Closes with** — ⚠ **Item (1) below was wrong and is superseded — see
[the correction above](#one-ledger-row-was-wrong-and-the-closure-caught-it).**
A `crvoice_vocalize` call cannot reach the silence arm at *any* duration, because
`pos_in_chirp` is the index within one synthesize call and vocalize renders in
882-sample blocks. ~~(1) In tests/voice.tcyr, give the cricket case a 0.2 s
duration so it crosses the chirp boundary at sample 5880.~~ What was actually
done at the voice layer instead: the oracle's own `near_silent > 10` bar
verbatim (it passes on the per-syllable envelope, which *is* reachable), plus a
hand-derived per-syllable envelope check — energy over a plateau window exceeds
1.5x energy over an equal-width ramp window of the same block.

**(2) — done, and this is where the branch is genuinely covered.** In
tests/tract.tcyr, a direct `crtract_synthesize_stridulatory` call of 12000
samples (spanning one full active chirp then 6120 samples of gap), asserting
`[10000, 12000)` is near-silent to the last sample (2000/2000), with two
controls proving the active chirp is not silent and does carry real signal. Note
the arm pushes exact `0`, but `dcblocker_process_buffer` runs over the whole
buffer with pole 0.995, so the tail decays over ~1100 samples rather than
snapping to zero — which is why the window starts at 10000 and why the oracle
uses a `< 0.001` bar rather than an equality.

**Checked for a defect** — TEST gap, not a defect. I diffed the two implementations line for line and they are a faithful port, so the property the Rust asserts is almost certainly still true of the Cyrius — nothing asserts it. Stridulatory synthesis, 2.0.3:rust-old/src/tract.rs:246-281 vs src/tract.cyr:355-393: identical constants (syllable_rate 30.0, chirp_rate 2.5, pulses_per_chirp 4), identical derivations (chirp_period = sr/chirp_rate at tract.rs:252 / tract.cyr:360; syllable_period = sr/syllable_rate at tract.rs:253 / tract.cyr:361; chirp_active = syllable_period * 4 at tract.rs:254 / tract.cyr:362), identical guard (`if pos_in_chirp < chirp_active` at tract.rs:264 / `if (f64_lt(pos_in_chirp, chirp_active) == 1)` at tract.cyr:374), identical three-arm pulse envelope (tract.rs:268-275 / tract.cyr:379-386: <0.1 ramp, <0.5 unity, else 1-(frac-0.5)/0.5), identical else-branch push of exact 0.0 (tract.rs:277-278 / trac …

### test_cat_nasal_resonance

**🔴 gap** · `2.0.3:rust-old/tests/integration.rs:454-461`

**Missing** — No assertion in any of the 17 Cyrius suites calls crvoice_vocalize (or crvoice_vocalize_with_intent, or stream_new) with the pair (PRANI_SP_CAT, PRANI_VOC_HOWL). A grep for PRANI_SP_CAT across tests/*.tcyr returns only: purr synthesis (voice.tcyr:221, stream.tcyr:139), the sample-rate-10 rejection control (hardening.tcyr:70-71), an FFI handle check (hardening.tcyr:169-171), preset species fields (preset.tcyr:104-196), tract-level species_params (tract.tcyr:162), and the bridge f0->species mapping (bridge.tcyr:148). So none of the test's asserted properties has a covering assertion: cat howl is never shown to return Ok, never shown to be non-empty, and never shown to be all-finite. Concretely uncovered code: the 0.2 nasal fraction is never executed through crvoice_apply_nasal_antiformant (only wolf's 0.1 is), so the 8820-sample notch recursion at the cat fraction is unasserted; and the CAT_MEOW_HOWL keyframe table at src/voice.cyr:503-521 is never read at any t — voice.tcyr:176 only proves the pointer is non-null, while the analogous wolf assertions at voice.tcyr:184-185 do check blend and F1 values.

**Closes with** — Add to tests/voice.tcyr, in the "vocalize -- non-error, length, finite (multiple species)" group (alongside the cat-purr block at :221-224): var cat_howl = crvoice_vocalize(crvoice_new(PRANI_SP_CAT), PRANI_VOC_HOWL, T_SR, T_DUR_005); assert_eq(prani_is_err(cat_howl), 0, "cat howl non-error"); assert_eq(vec_len(cat_howl), 2205, "cat howl length == 2205"); assert_eq(t_all_finite(cat_howl), 1, "cat howl all finite"); and, to give the cat keyframe table the value coverage wolf already has, extend the "formant_transition_contour -- Some/None" group (mirroring :182-185): var cat_fc = crvoice_formant_transition_contour(PRANI_VOC_HOWL, PRANI_SP_CAT); var cfb0 = crvoice_formant_transition_at(cat_fc, 0); assert_eq(FormantBlend_blend(cfb0), T_0_8, "cat meow blend at t=0 == 0.8"); assert_eq(FormantBlend_f0(cfb0), f64_from(400), "cat meow F1 at t=0 == 400");

**Checked for a defect** — Not a defect -- a pure TEST gap. I diffed every element of the Cat+Howl path against the oracle and each matches: supports_vocalization (species.rs:317-320 vs species.cyr:356-359), the purr short-circuit and its position relative to the supports check and tract build (voice.rs:211-213 vs voice.cyr:800-804), nasal_phase_fraction = 0.2 (voice.rs:578 vs voice.cyr:556-558, PR_V_0_2 = 0x3fc999999999999a), the entire apply_nasal_antiformant IIR including the fade blend against the pre-write sample (voice.rs:589-625 vs voice.cyr:578-621), all 24 CAT_MEOW_HOWL keyframe values (voice.rs:539-546 vs voice.cyr:503-521), FormantTransitionContour::at (voice.rs:493-529 vs voice.cyr:419-466), and set_formant_blend with its deliberately discarded Result (tract.rs:336-376 / voice.rs:210 vs tract.cyr:508-531 / voice.cyr:866). The nasal filter is fraction-agnostic, so the Wolf 0.1 run at tests/voice.tcyr:19 …

### test_distance_attenuation

**🟡 partial** · `2.0.3:rust-old/tests/integration.rs:478-493`

**Missing** — Two distinct properties are dropped. (1) THE AGGREGATE ENERGY RATIO. The oracle sums s² over a full 22050-sample vocalization at 1 m and at 50 m and asserts near_energy > 10 * far_energy. The Cyrius substitutes a per-sample amplitude ordering on 1- and 2-element synthetic impulses at 48 kHz (spatial.tcyr:31 pins near out[0] == 0.7196..., spatial.tcyr:56-57 put far out[0] below 0.01 and below near out[0]). Those two together do imply an amplitude ratio > 71x at 100 m, but nothing in the suite computes an energy integral, runs the attenuator over a real multi-sample signal longer than 2 samples, or exercises distance 50 with its 10 kHz cutoff (only 0.5, 1 and 100 appear). (2) THE IMPLICIT AUDIBILITY ASSERTION, which is the more serious half. Because far_energy >= 0 always, near_energy > far_energy*10 forces near_energy > 0, so the Rust test fails outright if crvoice_vocalize ever returns an all-zero buffer. No assertion anywhere in the 17 Cyrius suites checks that any synthesized buffer is non-silent: every synthesis assertion is prani_is_err == 0, an exact vec_len, all-finite, or bit-identical determinism — all of which an all-zero buffer passes. (Grepping for amplitude-floor style assertions turns up only tests/dsp.tcyr:34, which asserts a DC value is driven BELOW 1e-3 — the opposite direction.) The port therefore has no regression guard at all against a silent-output regression in svara or in the amplitude/envelope stages.

**Closes with** — Two additions. (1) In tests/spatial.tcyr, add a group that reproduces the oracle's shape on a multi-sample buffer (spatial.tcyr has no voice deps by design, so build the input synthetically — e.g. a few hundred alternating-sign samples): run spatial_apply_distance_attenuation at (1.0, 1.0, 44100) and at (50.0, 1.0, 44100), sum the squares of each output, then assert_eq(f64_gt(near_energy, 0), 1, "near energy > 0") and assert_eq(f64_gt(near_energy, f64_mul(far_energy, f64_from(10))), 1, "near energy > 10x far energy") — this covers distance 50 and the energy integral in one group. (2) In tests/voice.tcyr, add a t_max_abs(v) helper next to t_all_finite (:67-75) and assert audibility on the synthesis paths, at minimum the wolf howl: assert_eq(f64_gt(t_max_abs(w_out), 0x3F50624DD2F1A9FC), 1, "wolf howl is audible (max\|s\| > 1e-3)") — this is the property the oracle got for free from the energy comparison and the port currently has nowhere.

**Checked for a defect** — Not a defect — established by execution, not inference. I compiled and ran a probe that reproduces the oracle body verbatim against the Cyrius port (Wolf HOWL, 44100 Hz, 0.5 s, attenuated at 1 m and 50 m, energy = sum of s^2). Every assertion the Rust test makes PASSES on the port, including the implicit audibility one: near_energy > 0 passes, near_energy > far_energy*10 passes, and in fact near_energy > far_energy*1000 passes. Bracketing puts the raw wolf-howl max \|sample\| in (0.05, 0.1) and near_energy in (10, 100) — comfortably audible, not a silent buffer. The source supports this. src/spatial.cyr:43-68 is a faithful transcription of 2.0.3:rust-old/src/spatial.rs:16-42: identical distance clamp, identical inverse-distance gain, identical cutoff formula max(20000/(1+d*0.02), 200) with the same 200 Hz floor, identical alpha derivation, identical per-sample one-pole recurrence. The amplitud …

### test_call_bout

**🟡 partial** · `2.0.3:rust-old/tests/integration.rs:495-517`

**Missing** — The structural properties are covered — at greater strength than the oracle — but only on a different configuration: Wolf / HOWL / Social / count 2 / 8 kHz instead of Dog / BARK / Alarm / count 3 / 44.1 kHz. Three things go unasserted as a result. (a) SPECIES AND VOCALIZATION: PRANI_SP_DOG is never passed to crvoice_vocalize, crvoice_vocalize_with_intent or stream_new in any of the 17 suites (its only appearances are the resonance-seed literal at species.tcyr:78 and the bout template at species.tcyr:105-109), and PRANI_VOC_BARK is never synthesized at all. So the BARK pitch contour (src/voice.cyr:389-393: 0.0->1.2, 0.1->1.0, 1.0->0.8) has neither execution coverage nor value coverage — tests/voice.tcyr:148-158 asserts f0_at only for HOWL and the FLAT/GROWL fallback — and the Bark/Yelp/Chirp 0.05/0.10 attack/release arm of the envelope table (src/voice.cyr:672-674) is never entered. (b) COUNT: count = 3 walks the `i < count - 1` guard over two interior boundaries and one terminal one; count = 2 gives only one of each. (c) INTENT: Alarm's 0.7 duration_scale never scales a real render anywhere — sequence.tcyr uses Social (1.0) throughout, and the only intent-scaled synthesis is Mating (x2) at voice.tcyr:241-244. So the bout-length arithmetic is never checked against an intent whose duration_scale is neither 1.0 nor a clean multiple.

**Closes with** — Extend the "CallBout::synthesize" group in tests/sequence.tcyr with the oracle's own configuration (note SR there is 8000; use a 44100 constant): var dogv = crvoice_new(PRANI_SP_DOG); var dbout = sequence_call_bout_new(PRANI_VOC_BARK, 3, 0x3fc999999999999a /*0.2*/, 0x3fd3333333333333 /*0.3*/, PRANI_INTENT_ALARM); var dbuf = sequence_call_bout_synthesize(dbout, dogv, T_SR_44100); assert_eq(prani_is_err(dbuf), 0, "dog bark bout non-error"); assert_eq(vec_len(dbuf), 44979, "3 alarm barks (6173 each) + 2 gaps of 13230"); <all-finite loop as at :64-70> The 44979 is measured, not guessed: Alarm's duration_scale 0.7 gives f64_to(0.2*0.7*44100) == 6173 per call (0.2*0.7 rounds to 0.13999999999999999 in f64, so it truncates to 6173, NOT 6174), plus 2 * f64_to(0.3*44100) == 2*13230. Separately, give the BARK contour value coverage in tests/voice.tcyr's "pitch_contour f0_at" group: f0_at(0) == base*1.2, f0_at(0.1) == base, f0_at(1.0) == base*0.8.

### test_call_phrase

**🟡 partial** · `2.0.3:rust-old/tests/integration.rs:519-546`

**Missing** — Two properties are dropped. (1) NO FINITENESS ASSERTION ON PHRASE OUTPUT. tests/sequence.tcyr:89-97 asserts only prani_is_err == 0 and vec_len == 1600 on phrase_buf. Its sibling groups both carry all-finite loops — the bout at :64-70 and the chorus at :110-116 — but the phrase group has none, so the oracle's `assert!(samples.iter().all(\|s\| s.is_finite()))` at :545 has no analogue anywhere for CallPhrase::synthesize. (2) THE PHRASE IS ONLY EVER HOMOGENEOUS. The only synthesized phrase is two IDENTICAL Wolf HOWL elements of equal duration under Social intent at 8 kHz. The oracle's distinguishing content — different vocalization types per element, different durations per element, a non-zero interior gap, and an intent whose duration_scale is 2.0 rather than 1.0 — is never rendered. Consequently PRANI_VOC_CHIRP is never synthesized by ANY suite (grep: it appears only in sequence construction/serde at sequence.tcyr:27-28, 42, 144 and in emotion selection at emotion.tcyr:109), so the Chirp branches have no execution coverage: the +1.5 spectral offset at src/voice.cyr:633 and the Bark/Yelp/CHIRP 0.05/0.10 envelope arm at src/voice.cyr:676-677. Songbird is likewise never used on the phrase path.

**Closes with** — In tests/sequence.tcyr's "CallPhrase::synthesize -- structure + finiteness" group (whose title already promises finiteness): (1) add the all-finite loop the bout group has at :64-70, over phrase_buf; (2) add the oracle's own phrase, which also brings CHIRP and Mating onto a synthesis path for the first time: var bvoice = crvoice_new(PRANI_SP_SONGBIRD); var bels = vec_new(); vec_push(bels, sequence_call_element_new(PRANI_VOC_CHIRP, 0x3fb999999999999a /*0.1*/, 0x3fa999999999999a /*0.05*/)); vec_push(bels, sequence_call_element_new(PRANI_VOC_TRILL, 0x3fd3333333333333 /*0.3*/, 0x3fb999999999999a /*0.1*/)); vec_push(bels, sequence_call_element_new(PRANI_VOC_CHIRP, 0x3fb999999999999a /*0.1*/, 0)); var bphrase = sequence_call_phrase_new(bels, PRANI_INTENT_MATING); var bbuf = sequence_call_phrase_synthesize(bphrase, bvoice, T_SR_44100); assert_eq(prani_is_err(bbuf), 0, "songbird chirp/trill/chirp phrase non-error"); assert_eq(vec_len(bbuf), 50715, "8820 + 2205 + 26460 + 4410 + 8820 (Mating x2 per element)"); <all-finite loop>

### test_chorus_synthesis

**🟡 partial** · `2.0.3:rust-old/tests/integration.rs:548-565`

**Missing** — No chorus assertion anywhere uses more than 2 voices, and none uses voices with differing size scales — both Cyrius chorus tests mix two bit-identical default-size Wolf voices (crvoice_new with no with_size), so the mix is effectively 2x one signal. The oracle's distinguishing content — a 4-voice mix of size-varied (0.8/1.0/1.2/1.4) individuals, hence the n=4 RNG seed (voice_count*7919, src/sequence.cyr:214) and the 1/sqrt(4) normalization (src/sequence.cyr:241) — has no analogue. crvoice_with_size is used in tests/voice.tcyr:103,112-115,263 and tests/hardening.tcyr:118,162,216, but never on the chorus path.

**Closes with** — In tests/sequence.tcyr's "synthesize_chorus" group, add a second chorus over FOUR voices built as crvoice_with_size(crvoice_new(PRANI_SP_WOLF), s) for s in {0.8, 1.0, 1.2, 1.4}, and assert prani_is_err == 0, the exact mix length, and all-finite (the same finiteness loop). This mirrors the oracle's voice heterogeneity and exercises the n=4 seed/normalization.

### test_voice_presets

**🔴 gap** · `2.0.3:rust-old/tests/integration.rs:567-592`

**Missing** — Nothing in any of the 17 suites ever SYNTHESIZES from a preset-built voice: grep of crvoice_vocalize / crvoice_vocalize_with_intent across tests/*.tcyr shows calls only on crvoice_new(...) voices (tests/voice.tcyr:190,197,203,209,215,221,227,233,241,250-255,272; tests/prani.tcyr:39,48; tests/hardening.tcyr:64,67,71,74,220,222). Properties 3 and 4 (each of the 11 presets produces a non-error, all-finite 0.3 s buffer at 44100) are therefore entirely unasserted, and with them the ONLY voice-level synthesis coverage the suite would have for Raptor, Crow and Crocodilian (never synthesized at any level), for Lion/Dragon/Cat Howl (voice.tcyr does Lion Roar, Dragon Roar, Cat Purr only) and for Cricket Chirp (voice.tcyr does Cricket Stridulate only). The oracle's supports_vocalization selection chain is likewise unasserted for those species.

**Closes with** — Add a group to tests/preset.tcyr that loops preset_all(): build each preset, pick the vocalization with the oracle's chain (PRANI_VOC_HOWL else ROAR else GROWL else CHIRP else BUZZ via species_supports_vocalization), call crvoice_vocalize(v, voc, 44100.0, 0.3) and assert non-error, non-empty and all-finite. Accumulate failures into a bitmask of preset indices and assert the mask == 0 so a failure names the preset.

### test_serde_roundtrip_call_bout

**🟡 partial** · `2.0.3:rust-old/tests/integration.rs:594-608`

**Missing** — The CallBout roundtrip is asserted only with PRANI_VOC_HOWL, whose discriminant is 0 (src/vocalization.cyr:16). bayan's value accessors are null-safe, so a derive that omitted or misspelled the "vocalization" key for CallBout would still yield 0 and tests/sequence.tcyr:138 would pass; the oracle's assert_eq!(b2.vocalization, Bark) uses a nonzero discriminant and would catch that. Secondary: the CallBout roundtrip has no json-idempotency assertion (CallElement/CallPhrase get one at tests/sequence.tcyr:157) and never checks the interval field (the oracle does not either, so that is not a parity gap). Everything else in the group is stronger than the oracle.

**Closes with** — In tests/sequence.tcyr's "sequence serde roundtrip" group, roundtrip the oracle's own bout — sequence_call_bout_new(PRANI_VOC_BARK, 3, 0.2, 0.3, PRANI_INTENT_ALARM) — and assert CallBout_vocalization(bo2) == PRANI_VOC_BARK plus CallBout_interval(bo2); optionally add the memeq to_json idempotency check the sibling structs already have.

### test_bout_template_all_species

**🔴 gap** · `2.0.3:rust-old/tests/integration.rs:619-651`

**Missing** — Three separate holes. (a) Only 3 of the 13 templates are constructed by any test — Cat, Songbird, Crow, Raptor, Snake, Crocodilian, Cricket, Bee, Dragon and Fantasy templates are never asserted at all, so their vocalization/count/call_duration/interval/intent are unverified against the oracle table (species.rs:330-424); even for the three covered, `interval` is never asserted and Lion's call_duration is not either. (b) The invariant "the template's vocalization is supported by its own species" is asserted for ZERO species — species.tcyr's supports matrix deliberately picks non-template pairs. (c) No template bout is ever synthesized: sequence.tcyr's bout synthesis uses a hand-built Wolf bout with different count/duration at sr 8000, so properties 3 and 4 have no coverage for any species, including the whole-apparatus sweep (Bee's vibratile path is exercised only at tract level, tests/tract.tcyr:150-158; Crow, Raptor, Crocodilian, Dog and Fantasy are never synthesized at any level).

**Closes with** — Add a loop group over s in 0..PRANI_SP_COUNT that (1) takes b = species_bout_template(s) and asserts species_supports_vocalization(s, CallBout_vocalization(b)) == 1, and (2) — in tests/sequence.tcyr, whose unit already includes voice.cyr — asserts sequence_call_bout_synthesize(b, crvoice_new(s), 44100.0) is non-error, non-empty and all-finite, accumulating failures into a per-species bitmask so a failure names the species. Separately, extend tests/species.tcyr's bout_template group with the remaining 10 templates' field values (including interval) transcribed from 2.0.3:rust-old/src/species.rs:330-424.

### test_spectral_envelope_per_vocalization

**🔴 gap** · `2.0.3:rust-old/tests/integration.rs:653-663`

**Missing** — Neither Growl nor Screech is ever synthesized end-to-end by any suite. Every crvoice_vocalize / crvoice_vocalize_with_intent call in tests/*.tcyr uses HOWL, ROAR, TRILL, HISS, PURR or STRIDULATE (tests/voice.tcyr:190,197,203,209,215,221,227,233,241,250-255,272; tests/prani.tcyr:39,48; tests/hardening.tcyr:64,67,71,74,220,222). All four of the oracle's properties are therefore unasserted: the growl branch is covered only as isolated unit lookups (its -2.0 offset and FLAT contour), and Screech has NO coverage anywhere — not even its +1.5 spectral offset, which the offset-table group omits.

**Closes with** — Add to tests/voice.tcyr's "vocalize -- non-error, length, finite" group a Wolf GROWL and a Wolf SCREECH call at T_SR/T_DUR_005 asserting prani_is_err == 0, vec_len == 2205 and t_all_finite == 1 (ideally from one shared voice, as the oracle does). Separately complete the offset table group with assert_eq(crvoice_vocalization_spectral_offset(PRANI_VOC_SCREECH), 1.5) and the remaining arms (CHIRP +1.5, ROAR -1.0, RUMBLE -2.0, TRILL +0.5) so the whole table is pinned against 2.0.3:rust-old/src/voice.rs:631-646.

**Checked for a defect** — Not a defect — a pure TEST gap. I read both implementations line-for-line on every branch this oracle test reaches, and each one matches: the Laryngeal supports-gate (2.0.3:rust-old/src/species.rs:317-319 vs src/species.cyr:355-357) lets Wolf produce both Growl and Screech in both ports; the spectral-offset table agrees (-2.0 at voice.rs:634 / voice.cyr:629, +1.5 at voice.rs:638 / voice.cyr:632); Screech's contour points are transcribed verbatim (voice.rs:462 vs voice.cyr:366-370) and Growl falls to FLAT in both (voice.rs:474 vs voice.cyr:376-379); the envelope fractions agree (voice.rs:682,687 vs voice.cyr:680-681,698-699); apply_am_pattern early-returns for both in each port (voice.rs:652-655 vs voice.cyr:645); and the post-processing chain runs in the same order (voice.rs:274-384 vs voice.cyr:906-1014). Neither Growl nor Screech hits any of the species-conditional effects (subharmonics, fir …

### test_non_stationary_perturbation

**🟡 partial** · `2.0.3:rust-old/tests/integration.rs:676-689`

**Missing** — Two things this Rust test asserts are asserted nowhere in the Cyrius suites. (1) Vocalization::Bark is never synthesized: a grep of every synthesis call site in tests/*.tcyr (crvoice_vocalize, crvoice_vocalize_with_intent, sequence_call_bout_synthesize, sequence_call_phrase_synthesize, sequence_synthesize_chorus, crtract_synthesize*) shows PRANI_VOC_BARK appears only in emotion.tcyr:104,112 (select_vocalization mapping) and species.tcyr:106 (bout-template field). So the BARK pitch contour built at src/voice.cyr:353-355 via crvoice_pitch_contour_bark (src/voice.cyr:390-394) is never driven through crvoice_pitch_contour_f0_at inside a real synthesis, and voice.tcyr's contour group (148-157) hand-checks only HOWL and the FLAT/GROWL fallback — never BARK. (2) No SUCCESSFUL Alarm-intent synthesis exists anywhere: the only crvoice_vocalize_with_intent(..., PRANI_INTENT_ALARM, ...) call is hardening.tcyr:66-68, which deliberately asserts an error at sample_rate 10 and therefore never reaches the block loop. Consequently perturbation_scale = boundary_boost + urgency with a NON-ZERO urgency (0.9, giving up to 2.4 at call boundaries) is never fed into crtract_synthesize; every synthesized buffer in the suites uses Idle (0.0), Social (0.3) or Mating (0.2) urgency. The Idle half of the Rust test (non-error + all-finite through the Idle branch) IS covered in substance by voice.tcyr:195-236, just not with Bark.

**Closes with** — Add to tests/voice.tcyr, next to the existing "vocalize_with_intent -- intent scales duration (Mating x2)" group: var pb_alarm = crvoice_vocalize_with_intent(crvoice_new(PRANI_SP_WOLF), PRANI_VOC_BARK, PRANI_INTENT_ALARM, T_SR, T_DUR_005); assert_eq(prani_is_err(pb_alarm), 0, "wolf alarm bark non-error"); assert_eq(vec_len(pb_alarm), 1543, "alarm (x0.7 duration) length"); assert_eq(t_all_finite(pb_alarm), 1, "alarm bark all finite"); then the same three assertions for PRANI_INTENT_IDLE on the SAME voice handle (reusing one crvoice_new(PRANI_SP_WOLF) for both calls, to reproduce the oracle's &self reuse) with vec_len == 2205. Recompute the two expected lengths from f64_to(duration * duration_scale * 44100) before committing them. Optionally also add a BARK entry to the voice.tcyr pitch_contour group (crvoice_pitch_contour(PRANI_VOC_BARK, T_1000): f0_at(0) == 1200, f0_at(0.1) == 1000, f0_at(1.0) == 800) so the bark contour table is pinned exactly.

### test_vocal_effort_whisper_vs_shout

**🔴 gap** · `2.0.3:rust-old/tests/integration.rs:693-707`

**Missing** — Nothing in any of the 17 Cyrius suites measures the ENERGY, RMS, peak amplitude or any loudness proxy of a synthesized buffer. I grepped tests/*.tcyr for energy, louder, whisper, shout, max_amp, amplitude and rms: every hit is a parameter-level scalar (bridge_amplitude_from_spl, FatigueModifiers_amplitude_scale, IntentModifiers_amplitude_scale) or a comment — none reads back a crvoice_vocalize result and reduces it. So the whole point of this test, 's_energy > w_energy * 2.0' (vocal effort audibly and monotonically scales the output), is entirely unasserted. Worse, the two operative synthesis calls do not exist at all: no test calls crvoice_vocalize on a voice at effort 0.0, and the single max-effort call (voice.tcyr:272) throws the buffer away, so even the weaker properties 'whisper synthesizes without error' and 'shout synthesizes without error / is finite' are unasserted. The extremes matter here beyond amplitude: effort 0.0 and 1.0 are the only inputs that drive effort_tilt_offset to its -3 / +3 dB/oct limits (src/voice.cyr:793) and effort_breathiness_delta to its 0.15 maximum (src/voice.cyr:794, clamped into the tract params at src/voice.cyr:824-826) — extreme-parameter paths through svara that nothing currently walks.

**Closes with** — Add a group to tests/voice.tcyr (it already has the t_all_finite helper at 60-69; add a sum-of-squares helper beside it, e.g. fn t_energy(v) { var acc = 0; var i = 0; while (i < vec_len(v)) { acc = f64_add(acc, f64_mul(vec_get(v,i), vec_get(v,i))); i = i + 1; } return acc; }). Then: test_group("vocal effort -- shout is much louder than whisper"); var wsp = crvoice_vocalize(crvoice_with_vocal_effort(crvoice_new(PRANI_SP_WOLF), 0), PRANI_VOC_HOWL, T_SR, T_DUR_005); var sht = crvoice_vocalize(crvoice_with_vocal_effort(crvoice_new(PRANI_SP_WOLF), T_1_0), PRANI_VOC_HOWL, T_SR, T_DUR_005); assert_eq(prani_is_err(wsp), 0, "whisper non-error"); assert_eq(prani_is_err(sht), 0, "shout non-error"); assert_eq(t_all_finite(wsp), 1, "whisper all finite"); assert_eq(t_all_finite(sht), 1, "shout all finite"); assert_eq(f64_gt(t_energy(sht), f64_mul(t_energy(wsp), T_2_0)), 1, "shout energy > 2x whisper energy"); and, to pin the implied audibility, assert_eq(f64_gt(t_energy(sht), 0), 1, "shout energy > 0"). f64_gt / f64_mul are already used throughout the suite (e.g. voice.tcyr:264, fatigue.tcyr:162).

### test_emotion_state_smooth_update

**🟡 partial** · `2.0.3:rust-old/tests/integration.rs:746-753`

**Missing** — No assertion in any of the 17 suites calls emotion_update on a state whose smoothing was changed from the 0.1 default (grep for emotion_update across tests/*.tcyr hits only tests/emotion.tcyr:74 and :79, both on emotion_new()). The composition with_smoothing -> update is therefore unasserted: if emotion_update ignored PrEmotion_smoothing(self) and used the hardcoded PRE_0_1, the entire Cyrius suite would still pass. Neither the specific halfway result at s=0.5 (valence 0.5, arousal 0.75) nor the chained with_values(...).with_smoothing(...) construction is covered.

**Closes with** — Add to tests/emotion.tcyr, in the "update" group (after line 82): `var eu3 = emotion_with_smoothing(emotion_with_values(T_0_0, T_0_5), T_0_5); emotion_update(eu3, T_1_0, T_1_0); assert_eq(emotion_valence(eu3), T_0_5, "s=0.5 -> valence halfway (0.5)"); assert_eq(emotion_arousal(eu3), 0x3fe8000000000000, "s=0.5 -> arousal 0.75");` — both expected values verified by direct execution (see notes).

**Checked for a defect** — Not a defect. I read both implementations side by side. src/emotion.cyr:117 does `var s = PrEmotion_smoothing(self);` — it reads the per-state smoothing field, exactly as 2.0.3:rust-old/src/emotion.rs:101 does `let s = self.smoothing;`. The blend expressions at src/emotion.cyr:119-120 are the same `x*s + target*(1-s)` as 2.0.3:rust-old/src/emotion.rs:102-103, with the same pre-clamping of targets (src/emotion.cyr:115-116 vs emotion.rs:99-100). emotion_with_smoothing (src/emotion.cyr:94-97) clamps to [0.0, 0.95] and returns the handle, matching emotion.rs:76-79, so the chained with_values(...).with_smoothing(...) construction composes correctly in the port. Hand-evaluating the Cyrius code for the Rust test's inputs (v=0.0, a=0.5, s=0.5, targets 1.0/1.0) yields valence = 0.0*0.5 + 1.0*0.5 = 0.5 and arousal = 0.5*0.5 + 1.0*0.5 = 0.75 — both inside the Rust assertion windows (0.4,0.6) and (0.7,0.8). The …

### test_emotion_state_drives_synthesis

**🟡 partial** · `2.0.3:rust-old/tests/integration.rs:755-765`

**Missing** — Three properties are unasserted anywhere in the 17 suites. (1) The emotion -> voice composition: no test feeds an emotion_evaluate result into crvoice_with_vocal_effort / crvoice_vocalize_with_intent; tests/prani.tcyr:34 stops at a finiteness check on pitch_scale. (2) The Wolf + BARK synthesis path: grep for PRANI_VOC_BARK across tests/*.tcyr hits only emotion mapping assertions (emotion.tcyr:104, :112) and the dog bout template (species.tcyr:106) — Bark is never synthesized by any species. (3) Synthesis at an emotion-derived non-default vocal effort producing a non-error, non-empty, all-finite result. Also unasserted: the -0.5 valence boundary (strict `<` at src/emotion.cyr:158, :191) that decides Bark-vs-Screech and Alarm-vs-Distress for exactly this state.

**Closes with** — Add a group to tests/prani.tcyr (bundle level, where emotion and voice are already both in scope) or a new group in tests/voice.tcyr: `var dst = emotion_with_values(0xbfe0000000000000, 0x3fe999999999999a);` (-0.5, 0.8) `var dout = emotion_evaluate(dst); assert_eq(PrEmotionOut_vocalization(dout), PRANI_VOC_BARK, "v=-0.5 boundary -> Bark not Screech"); assert_eq(PrEmotionOut_intent(dout), PRANI_INTENT_ALARM, "v=-0.5 -> Alarm not Distress"); assert_eq(PrEmotionOut_vocal_effort(dout), 0x3fe851eb851eb852, "effort 0.76"); var dv = crvoice_with_vocal_effort(crvoice_new(PRANI_SP_WOLF), PrEmotionOut_vocal_effort(dout)); var ds = crvoice_vocalize_with_intent(dv, PrEmotionOut_vocalization(dout), PrEmotionOut_intent(dout), T_SR, T_0_5); assert_eq(prani_is_err(ds), 0, ...); assert_eq(vec_len(ds), 15434, "0.5 s * Alarm duration_scale 0.7 * 44100"); assert_eq(t_all_finite(ds), 1, ...);` — all four expected values measured, see notes.

**Checked for a defect** — I read both implementations end to end on this exact path and found no divergent lines, so is_possible_defect = false. 1. Emotion mapping. src/emotion.cyr:82-88 (emotion_with_values, f64_clamp to [-1,1]/[0,1]) matches 2.0.3:rust-old/src/emotion.rs:67-73. Zones: src/emotion.cyr:131-145 (`f64_lt(v, -0.2)` / `f64_gt(v, 0.2)`, `f64_lt(a, 0.33)` / `f64_gt(a, 0.66)`) match 2.0.3:rust-old/src/emotion.rs:206-224 exactly, including strictness. Sub-branches: src/emotion.cyr:158 `if (f64_lt(v, PRE_NEG_0_5)) { return PRANI_VOC_SCREECH; } return PRANI_VOC_BARK;` matches 2.0.3:rust-old/src/emotion.rs:152-156; src/emotion.cyr:191 `if (f64_lt(v, PRE_NEG_0_7)) { DISTRESS } return ALARM;` matches 2.0.3:rust-old/src/emotion.rs:189-193. PRE_NEG_0_5 = 0xbfe0000000000000 is exactly -0.5 (src/emotion.cyr:34) and -0.5 is exactly representable in both f32 and f64, so the boundary decides identically in both: Bark + Alarm. evaluate(): s …

### test_fatigue_accumulates

**🟡 partial** · `2.0.3:rust-old/tests/integration.rs:789-804`

**Missing** — Nothing in the Cyrius suite reads `fatigue` after MORE THAN ONE fatigue_record_call. Every fatigue-valued assertion (tests/fatigue.tcyr:33 single 10 s call -> 0.15; :43 single 100 s call -> clamp 1.0; :71 single call then rest) is taken after exactly one record_call, so an implementation that ASSIGNED instead of accumulated — `FatigueState_set_fatigue(self, f64_mul(duration, PR_FAT_FATIGUE_RATE))` instead of the add at src/fatigue.cyr:95-97 — would pass the entire suite while failing the Rust oracle (30 x 1.0 s would give 0.015, not > 0.3). The same holds for bout_calling_time, which is only ever checked after one call (:34, :92). The habituation counterpart IS covered across calls (:49 0.1 after one alarm, :53 ~0.3 after three), so this is an asymmetry in the suite rather than a blanket omission. The port itself is correct: src/fatigue.cyr:96 reads `f64_add(FatigueState_fatigue(self), f64_mul(duration, PR_FAT_FATIGUE_RATE))`, matching 2.0.3:rust-old/src/fatigue.rs:80, so this is a test gap, not a defect.

**Closes with** — Add to tests/fatigue.tcyr, in (or next to) the "record_call(10, false) -- fatigue accumulation" group: build a fresh state, loop `fatigue_record_call(acc, F64_ONE, 0)` 30 times as the oracle does, then assert (a) `assert_eq(f64_gt(fatigue_fatigue(acc), 0x3fd3333333333333), 1, "30x1s calls -> fatigue > 0.3")` — the oracle's own bound — and (b) the bit-exact accumulator `assert_eq(FatigueState_bout_calling_time(acc), 0x403e000000000000, "bout time == 30.0 after 30 calls")` (30 x 1.0 is exact in f64, so this pins accumulate-vs-assign with no rounding argument). Optionally also assert the modifier signs off that accumulated state (pitch < 0, breathiness > 0, amplitude < 1.0) so the Rust test's exact shape is reproduced.

**Checked for a defect** — TEST GAP, not a defect — I read both implementations line by line and they agree. Rust oracle, 2.0.3:rust-old/src/fatigue.rs:73-80: pub fn record_call(&mut self, duration: f32, is_alarm: bool) { self.bout_calling_time += duration; // :74 let fatigue_rate = 0.015; // :79 self.fatigue = (self.fatigue + duration * fatigue_rate).clamp(0.0, 1.0); // :80 Cyrius port, src/fatigue.cyr:89-95: fn fatigue_record_call(self, duration, is_alarm) { FatigueState_set_bout_calling_time(self, f64_add(FatigueState_bout_calling_time(self), duration)); // :91 var f = f64_add(FatigueState_fatigue(self), f64_mul(duration, PR_FAT_FATIGUE_RATE)); // :94 FatigueState_set_fatigue(self, f64_clamp(f, 0, F64_ONE)); // :95 Both are read-add-write on the stored field, i.e. genuine accumulation. Supporting checks: - PR_FAT_FATIGUE_RATE (src/fatigue.cyr:24) is 0x3f8eb851eb851eb8, which decodes to exactly 0.015 and is exactly th …

### test_stream_produces_same_length_as_batch

**🟡 partial** · `2.0.3:rust-old/tests/integration.rs:844-866`

**Missing** — No Cyrius test ever drains a stream in more than TWO fill_buffer calls, and every fill in every suite starts at normalized time t = 0, t = 0.2322 (tests/stream.tcyr:112) or t = 0.4535 (:129). Consequently the release-boundary branch of stream_fill_buffer — `elif (f64_gt(t, PR_ST_0_85) == 1) { boundary_boost = 1 + (t-0.85)/0.15*0.5 }` in src/stream.cyr — is never executed by any assertion in the suite, while the oracle's 512-sample drain of a 22050-sample stream renders roughly 44 blocks and so asserts finiteness of output produced through both the attack (t < 0.1) and release (t > 0.85) perturbation branches. The oracle's 'all samples finite across the whole call' property is therefore only partially covered: it is asserted over the first block and over single-pass renders, never over late-call blocks. Related but lesser: repeated-fill continuation (many successive fills each advancing samples_rendered by a fixed buffer size) is only exercised twice, and batch-vs-stream length parity is established by two literal assertions in two different suites rather than by one equality — and never with SOCIAL intent on the batch side (the Wolf batch assertion uses crvoice_vocalize, i.e. Idle; both scale duration by 1.0, and the Songbird/MATING pair covers a non-unit scale on both sides). This is a TEST gap, not a defect: the boundary arithmetic in src/stream.cyr is transcribed verbatim from 2.0.3:rust-old/src/stream.rs:164-171, and total_samples in src/stream.cyr (`f64_to(f64_mul(effective_duration, sample_rate))`) is the identical expression to the batch path's num_samples at src/voice.cyr:788, so the lengths agree by construction.

**Closes with** — Add a group to tests/stream.tcyr that mirrors the oracle's drain: build a Wolf/HOWL/SOCIAL stream at T_SR with a longer duration (0.5 s -> 22050 samples), allocate one 512-sample buffer, then loop `while (stream_is_finished(sd) == 0) { var n = stream_fill_buffer(sd, buf); total = total + n; ... }` accumulating into a vec (or checking t_all_finite(buf) on each pass and counting), and assert (a) `total == stream_total_samples(sd)` and `total == 22050`, (b) all-finite held on every pass — which is what actually exercises the t < 0.1 and t > 0.85 perturbation branches — and (c) the loop terminated. Optionally add the missing direct equality by also calling crvoice_vocalize_with_intent(Wolf, HOWL, SOCIAL, T_SR, same duration) in that group and asserting `vec_len(batch) == total`, so batch/stream length parity lives in one assertion on the oracle's own species/intent instead of across two suites.

**Checked for a defect** — Not a defect — a TEST gap. I read both implementations line for line. Release branch, oracle (2.0.3:rust-old/src/stream.rs:172-174): } else if t > 0.85 { 1.0 + (t - 0.85) / 0.15 * 0.5 } Port (src/stream.cyr:226-229): } elif (f64_gt(t, PR_ST_0_85) == 1) { boundary_boost = f64_add(F64_ONE, f64_mul(f64_div(f64_sub(t, PR_ST_0_85), PR_ST_0_15), PR_ST_0_5)); Same associativity ((t-0.85)/0.15)*0.5, same comparison sense, same else-if ordering after the t < 0.1 arm. The constants are exact: src/stream.cyr:53-57 gives PR_ST_0_5 = 0x3fe0000000000000 (0.5), PR_ST_0_1 = 0x3fb999999999999a (0.1), PR_ST_0_85 = 0x3feb333333333333 (0.85), PR_ST_0_15 = 0x3fc3333333333333 (0.15); the voice.cyr twins at :49-53 carry the identical bit patterns, so the untested twin branch at voice.cyr:889-892 is not a typo either. The length-parity property is true by construction, not by luck: total_samples at src/stream.cyr:116 …

### test_stream_next_block

**🟡 partial** · `2.0.3:rust-old/tests/integration.rs:868-879`

**Missing** — Two shortfalls, both mild. (1) The cat-purr special path is never driven through stream_next_block — tests/stream.tcyr's next_block group uses only Wolf/HOWL/SOCIAL, and Cat/PURR/IDLE is exercised only through stream_fill_buffer (:145-148). The oracle deliberately picks Cat + Purr for its next_block test, so the species/vocalization pair on this path differs; coverage is only compositional (stream_next_block delegates to stream_fill_buffer at src/stream.cyr:285). (2) There is no direct `assert_eq(stream_is_finished(sn), 0, ...)` after a partial next_block; the property is inferred only from the fact that the following next_block(5000) returns 1205 rather than 0 (tests/stream.tcyr:130). The fill_buffer analogue is asserted directly at :103, so the omission is specific to next_block. Both are TEST gaps rather than defects: src/stream.cyr:275-288 matches 2.0.3:rust-old/src/stream.rs:196-207 exactly — saturating remaining, min against block_size, empty vec at 0, zero-filled buffer handed to fill_buffer — and the purr branch it reaches (src/stream.cyr:207-211, clamp(27.0, 20.0, 35.0) then crtract_synthesize_purr) matches 2.0.3:rust-old/src/stream.rs:150-156.

**Closes with** — Extend the "next_block -- sizing" group in tests/stream.tcyr with the oracle's own case: `var scp = stream_new(crvoice_new(PRANI_SP_CAT), PRANI_VOC_PURR, PRANI_INTENT_IDLE, T_SR, T_DUR_005);` then `var cblk = stream_next_block(scp, 256);` and assert `vec_len(cblk) == 256`, `t_all_finite(cblk) == 1`, `stream_samples_rendered(scp) == 256`, and — the missing direct assertion — `assert_eq(stream_is_finished(scp), 0, "not finished after a partial next_block")`. Add the same is_finished == 0 assertion after the existing next_block(1000) at line 126 so the Wolf path carries it too.

### test_stream_finishes

**🟡 partial** · `2.0.3:rust-old/tests/integration.rs:881-894`

**Missing** — Two things the Rust test asserts are not asserted anywhere in the Cyrius suites. (1) Vocalization::Bark is never streamed: every stream_new in tests/stream.tcyr and tests/hardening.tcyr uses HOWL, TRILL, PURR or HISS, so neither Wolf-accepts-Bark (tests/species.tcyr:90-92 only probes Wolf+HOWL/BUZZ/STRIDULATE) nor the BARK/YELP branch of stream_pitch_contour_at (src/stream.cyr:307-310 dispatching to stream_contour_bark, src/stream.cyr:371-375) is exercised by any assertion — tests/stream.tcyr:87-91 probes only the HOWL and FLAT/GROWL tables. The BARK table is the only contour with an early knee (0.1) and is dead code as far as the suite is concerned. (2) The exact combination the Rust asserts — fill_buffer returning 0 on a stream retired by next_block — is not asserted; the suite asserts next_block-on-finished -> empty vec (stream.tcyr:134) and fill_buffer-on-finished -> 0 (stream.tcyr:118) but never crosses them. There is also no actual `while (stream_is_finished(s) == 0)` drain loop on the success path; termination is inferred from two explicit next_block calls (hardening.tcyr:85 runs the loop-termination argument only for the ADR-0003 failure path).

**Closes with** — Add a group to tests/stream.tcyr mirroring the oracle: `var sfin = stream_new(crvoice_new(PRANI_SP_WOLF), PRANI_VOC_BARK, PRANI_INTENT_IDLE, T_SR, T_0_1);` assert prani_is_err == 0 and stream_total_samples == 4410; drain with a real loop `while (stream_is_finished(sfin) == 0) { stream_next_block(sfin, 1024); }`; assert stream_is_finished(sfin) == 1; then push 64 zeros into a vec and assert stream_fill_buffer(sfin, that vec) == 0. Separately extend the stream_pitch_contour_at group (tests/stream.tcyr:85-91) with the BARK table, hand-computed against base 400: t=0 -> 480, t=0.1 -> 400, t=0.05 -> 440 (1.2 + (1.0-1.2)*0.5), t=1.0 -> 320, plus one PRANI_VOC_YELP probe to pin that it shares the BARK table.

### test_bridge_size_from_body_mass

**🟡 partial** · `2.0.3:rust-old/tests/integration.rs:904-915`

**Missing** — Properties 2 and 3 are unasserted, and both oracle probe points lie strictly OUTSIDE the range the Cyrius suite exercises — this is extrapolation, not interpolation between pinned points. The smallest positive mass tested is 1.0 kg (-> 0.32183, tests/bridge.tcyr:34), so nothing asserts that a sub-kilogram mass lands below 0.2; the mouse case at 0.03 kg is also the one input where the allometric result (~0.1) collides numerically with the non-positive floor constant PR_B_0_1 (src/bridge.cyr:63), so no assertion distinguishes 'computed the cbrt' from 'fell through to the floor' at that magnitude. The largest mass tested is 240 kg (-> 2.0, tests/bridge.tcyr:30), so nothing asserts the multi-tonne end is uncapped and exceeds 4.0. This matters more than a normal regime hole because the port replaces Rust's .cbrt() with f64_pow(x, 0.3333333333333333) (src/bridge.cyr:66) — accuracy at extreme bases (0.001 and 166.67) is precisely what the untested regimes would exercise.

**Closes with** — Extend the size_from_body_mass group in tests/bridge.tcyr (after line 35) with the two oracle points: `assert_eq(f64_lt(bridge_size_from_body_mass(PR_B_0_03), PR_B_0_2), 1, "mouse 0.03 kg -> below 0.2");` (PR_B_0_03 = 0.03 already exists at src/bridge.cyr:26) and `assert_eq(f64_gt(bridge_size_from_body_mass(f64_from(5000)), f64_from(4)), 1, "elephant 5000 kg -> above 4.0");`. Prefer also pinning the values within TOL — 0.03 kg -> 0.10000000000000002 and 5000 kg -> 5.503212081491044 — and adding `assert_eq(f64_neq(bridge_size_from_body_mass(PR_B_0_03), PR_B_0_1), ...)`-style separation from the floor, or at minimum a comment, so the mouse case cannot silently pass via the non-positive branch.

### public_types_are_send_sync

**⬜ n/a** · `2.0.3:rust-old/src/lib.rs:88-103 (mod assert_traits, helper `fn _assert_send_sync<T: Send + Sync>() {}` at 2.0.3:rust-old/src/lib.rs:86)`

NA — a Rust language-level property with no Cyrius analogue, and I want to be explicit about why rather than wave at the briefing's N/A clause. (1) The property is inexpressible. Cyrius has no trait system at all: `grep -rn '\btrait\b' lib/*.cyr src/*.cyr` returns only three prose comments describing Rust traits collapsing to free functions (lib/goonj.cyr:8249, lib/naad.cyr:8090, lib/naad.cyr:8258) — no `trait` construct, no marker/auto traits, no generic bounds, no monomorphisation. `_assert_send_sync::<T>()` has no translation target. (2) The property is unobservable by the harness even if it were expressible. The Rust test is compile-time-only: 13 instantiations of an empty generic fn, zero runtime effect. A `.tcyr` suite is a runtime sakshi harness (`test_group` / `assert_eq` / `assert_streq`), so there is nothing for it to assert. A `cyrius test` run that passes proves the port links, not that it type-checks a thread-safety marker Cyrius does not have. (3) The property is structurally inapplicable, not merely untested. Every one of the 13 Cyrius analogues is either an untyped integer constant (PraniError -> PRANI_ERR_* src/error.cyr:23-28; Species -> PRANI_SP_* src/species.cyr:28-42; Vocalization -> PRANI_VOC_* src/vocalization.cyr:15-28; CallIntent -> PRANI_INTENT_* src/vocalization.cyr:35-41) or a bare `alloc()` handle passed as an i64 (CreatureTract src/tract.cyr:91/114, CreatureVoice src/voice.cyr:93/100, VoicePreset src/preset.cyr:59/66, PrEmotion src/emotion.cyr:63/72, PrEmotionOut src/emotion.cyr:224, FatigueState src/fatigue.cyr:52/62, FatigueModifiers src/fatigue.cyr:162/176, CallBout src/sequence.cyr:64/78, CallPhrase src/sequence.cyr:113/120). In Rust terms every handle type IS a raw pointer — exactly the shape the oracle test excluded from its list (2.0.3:rust-old/src/ffi.rs:31,34). So the port cannot preserve the property even in spirit; it is void, not violated, and no assertion could make it true. (4) It also guards nothing here, because prani ships no concurrency. `ls lib/thread*` -> no match (lib/thread.cyr is not vendored), and `grep -rniE 'thread\|send\|sync\|mutex\|atomic\|concurren' src/*.cyr tests/*.tcyr` returns ZERO hits across all 16 sources and all 17 suites. lib/atomic.cyr is vendored but prani never includes or calls it. There is no code path whose thread-safety this would protect. Partial recovery, worth recording: property 14 (the 13 types still exist and are reachable from outside their module) IS incidentally preserved, as a link-time rather than an assert-time check. All 13 analogues are constructed or read by the suites — see the nine citations above, which together touch every one of the 13. Deleting or renaming any of them breaks the corresponding suite's build. That is genuinely weaker than the Rust compile error (it is a build break in a test file, not a bound violation) but it is the same failure signal for the same edit, so I do not think a surface-existence assertion is worth adding. No possible-defect signal. Nothing in the port behaves differently from Rust here, because there is no behaviour to differ. I recommend recording this row in the 2.0.4 ledger as NA with the reasoning above, so a later reader does not re-open it as an untested public-surface item.

