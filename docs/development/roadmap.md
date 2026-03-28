# Roadmap

## v1.1.0 — Game Integration

C FFI and real-time API for kiran/joshua and middleware (Wwise, FMOD, Godot).

- C FFI buffer-callback API (fills `&mut [f32]` on demand — maps to Wwise source plugin, FMOD DSP, Godot `AudioStreamGenerator`)
- RTPC-style continuous parameter interface (body size, arousal, valence, vocal effort, urgency — what game AI systems expect to drive)
- Streaming synthesis iterator (pull-based, no full-buffer allocation)
- Creature emotion state machine (valence/arousal model driving vocalization selection and parameter blending)
- Lombard effect (involuntary vocal effort increase in response to ambient noise level)
- Vocal effort parameter (continuous whisper-to-shout: modulates subglottal pressure, spectral slope, formant bandwidth, HNR)
- Fatigue/habituation (prolonged calling shows pitch drift, increased breathiness; repeated alarm without threat reduces intensity)

## v1.2.0 — Species Expansion

Priority real-world species for games, plus fantasy archetypes.

### Real Animals
- Bear (deep growls, woofs, jaw-popping — large mammalian tract, high subglottal pressure)
- Elephant (infrasonic rumbles 10-35 Hz, trumpets to 5 kHz — ~2m vocal tract)
- Horse (neighs with rapid FM, nickers, snorts)
- Frog/Toad (vocal sac as Helmholtz resonator, AM pulses, species-specific rates)
- Owl (low hoots via syrinx, screeches, bill clacking)
- Bat (echolocation FM sweeps 20-200 kHz, social calls)
- Deer/Elk (bugling FM sweep 1-4 kHz, barks, grunts)
- Seal/Sea Lion (barking, growling, aquatic resonance)

### Fantasy Archetypes
- Wyvern (smaller dragon variant: higher screeches, wing membrane vibration)
- Griffin (hybrid: eagle screeches layered with lion growls)
- Undead (degraded vocal tract: reduced formant clarity, gurgling, rattling)
- Elementals — fire (crackling roar), water (gurgling resonance), earth (grinding rumble), air (aeolian whistling)
- Golem (stone/metal resonance, grinding, no biological mechanism)
- Giant Spider (spiracle hissing, leg stridulation, mandible clicks)

### Non-Vocal Sound Production
- `VocalApparatus::Mechanical` for non-biological resonators (golems, elementals)
- Bill/mandible clicking synthesis
- Wing membrane vibration (distinct from Vibratile — aerodynamic flutter)

## v1.3.0 — Environmental Realism

Contextual audio that reacts to the world. Integrates with AGNOS science crates for physics.

- Underwater propagation (frequency-dependent absorption, speed-of-sound via water science crate — whale/dolphin species need this)
- Atmospheric vocalization (gas density, molecular weight, temperature affect f0 and formant spacing — via atmosphere crate)
- Frequency-dependent terrain absorption (foliage, cave reverb, open field)
- Environmental masking (energetic + informational — creatures adjust calls when SNR is low)
- Temporal coordination / turn-taking (anti-phase calling, call-and-response, leader-follower chorus dynamics)
- Ontogenetic variation (age parameter: juveniles get higher pitch, less stable formants, simpler syllables)

### New Species (aquatic)
- Whale (humpback song: hierarchical units/phrases/themes, FM sweeps, nonlinear dynamics)
- Dolphin (echolocation click trains with adaptive ICI, signature whistles)
- Kraken/Sea monster (deep underwater vocalizations with aquatic propagation)

## v2.0.0 — Architecture

Breaking changes for performance, flexibility, and scale.

- Zero-allocation streaming core (replace `Vec<f32>` returns with buffer-write API)
- SIMD-accelerated formant filter banks (`f32x4` parallel biquad evaluation, SoA data layout)
- Pluggable excitation sources (trait-based — allow custom source models beyond GlottalSource)
- Multi-creature scene graph (spatial mixing, distance culling, priority system — game voice budget typically 6-12 creature voices)
- Offline ML parameter extraction pipeline (use DDSP/RAVE externally to fit synthesis params from recordings, export as presets)
