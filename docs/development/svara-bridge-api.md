# svara / naad bridge API — for porting tract.rs & voice.rs

The prani modules `tract` and `voice` wrap svara's vocal-synthesis primitives
(and naad's filters under `naad-backend`). This maps the Rust API those modules
call to the **ported Cyrius symbol names** in `dist/svara.cyr` / `dist/naad.cyr`
(resolved via cyrius.cyml). Verify against `../svara/src/{glottal,formant,tract}.cyr`
and `../naad/src/filter.cyr` if a signature looks off.

## Return convention (svara)

svara constructors return **a heap pointer on success or a negative
`SVARA_ERR_*` code on failure** (pointers are large positives; errors are −1..−6).
Detect failure with `svara_is_err(result)` or `result < 0`. prani maps any svara
failure to its own code via `prani_from_svara(result)` (→ `PRANI_ERR_SVARA`) or,
where the Rust used `PraniError::SynthesisFailed`, to `PRANI_ERR_SYNTHESIS_FAILED`.

## GlottalSource  (svara::glottal::GlottalSource)

| Rust | Cyrius |
|---|---|
| `GlottalSource::new(f0, sr)? ` | `svara_glottal_new(f0, sr)` → ptr or `SVARA_ERR_INVALID_PITCH`/`_FORMANT` (valid f0 ∈ [20,2000], sr>0 finite) |
| `g.set_breathiness(a)` | `svara_glottal_set_breathiness(g, a)` |
| `g.set_jitter(j)` | `svara_glottal_set_jitter(g, j)` |
| `g.set_shimmer(s)` | `svara_glottal_set_shimmer(g, s)` |
| `g.next_sample()` | `svara_glottal_next_sample(g)` |

## VocalTract  (svara::tract::VocalTract)

| Rust | Cyrius |
|---|---|
| `VocalTract::new(sr)` | `svara_tract_new(sr)` → ptr |
| `t.set_formants_from_target(&target)?` | `svara_tract_set_formants_from_target(t, target)` → `SVARA_ERR_NONE`(0) or negative |
| `t.process_sample(x)` | `svara_tract_process_sample(t, x)` |
| `t.synthesize(&mut g, n)` → Vec | `svara_tract_synthesize(t, g, n)` → **raw `alloc(n*8)` buffer** (NOT a stdlib vec; read with `load64(buf+i*8)`). Internals = per-sample loop `store64(out+i*8, process_sample(t, glottal_next_sample(g)))`. **Prefer inlining that loop to push into a `vec` directly**, avoiding the raw-buffer/vec mismatch when you then DC-block. |
| `t.reset()` | `svara_tract_reset(t)` |

## Formant / VowelTarget / FormantFilter  (svara::formant)

| Rust | Cyrius |
|---|---|
| `Formant::new(freq, bw, amp)` | `svara_formant_new(freq, bw, amp)` → ptr (SvFormant) |
| `VowelTarget::with_bandwidths([f1..f5],[b1..b5])` | `svara_vowel_target_with_bandwidths(f1,f2,f3,f4,f5, b1,b2,b3,b4,b5)` — **10 scalar args, not arrays** |
| `FormantFilter::new(&formants, sr)?` | `svara_formant_filter_new(formants_vec, sr)` → ptr or `SVARA_ERR_INVALID_FORMANT`. `formants_vec` is a stdlib `vec` of SvFormant ptrs (build with vec_new/vec_push). |
| `ff.process_sample(x)` | `svara_formant_filter_process_sample(ff, x)` |

## naad BiquadFilter  (naad::filter, `naad-backend` only)

| Rust | Cyrius |
|---|---|
| `BiquadFilter::new(FilterType::BandPass, sr, freq, q)?` | `filter_biquad_new(filter_type, sr, freq, q)` → ptr (or error sentinel — check). FilterType consts live in `lib/naad.cyr` (grep `FILTER_` / bandpass). |
| `f.process_sample(x)` | `filter_biquad_process_sample(f, x)` |

**naad-backend note:** prani's default features include `naad-backend`. In tract,
NoiseOnly species pre-build a naad `filter_biquad_new(BANDPASS, sr, formants[0], 2.0)`
for noise shaping; the svara `FormantFilter` path is the fallback. The Cyrius port
should keep the naad bandpass path (that's the default-feature behaviour) and MAY
drop the fallback (documented deviation) or keep both.

## math note for tract/voice

`crate::math::f32::sin/cos(x)` → `f64_sin`/`f64_cos` builtins; `core::f32::consts::TAU`
→ `F64_TAU`, `::PI` → `F64_PI`. Phase accumulators, AM/FM carriers, pulse envelopes
port directly (f32→f64). `x as usize` truncation → `f64_to_i64`/`f64_trunc` per the
stdlib (check math.cyr); `i as f32` → `f64_from(i)`.
