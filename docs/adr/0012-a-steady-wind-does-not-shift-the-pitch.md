# 0012 — A steady wind does not shift the pitch

**Status**: Accepted
**Date**: 2026-11-17

## Context

Audit C10, deferred from the 2.2.1 sweep as roadmap 2.3.x. The Rust has
`doppler_velocity_from_wind(wind_speed, angle)`, which "converts wind speed and bearing
to an effective radial velocity for Doppler shift", where angle 0 is wind blowing from
the source toward the listener (`2.0.3:rust-old/src/bridge.rs:115-128`). It returned
the wind's component along the axis, `wind × cos(angle)`. `spatial_apply_doppler_shift`
reads a positive velocity as a receding source (`2.0.3:rust-old/src/spatial.rs:46`).
Composed as documented, a 10 m/s tailwind lowered the pitch: 1000 samples came back as
1029, and a headwind raised it, 1000 to 970.

The Rust defines what prani does, here a wind term for the radial velocity a Doppler
shift sees. It does not define the term's value. The bar for that is accuracy.

The physics settles it. In the medium's frame, with sound speed c, wind w, and unit
vector n̂ from source to listener, the received frequency is

    f_r / f_s = (c + v_r') / (c − v_s')

where v_r' and v_s' are the listener's and the source's velocities relative to the air,
projected on the axis. With both at rest on the ground, v_r' = w·n̂ and v_s' = −w·n̂, so
f_r / f_s = (c + w·n̂) / (c + w·n̂) = 1. The wind changes how fast the sound travels and
so how long it takes, but every wavefront emitted is received, and the pitch is
unchanged. A wind matters to a Doppler shift only through a moving source or listener,
and then as a second-order change to their shift, which a function of wind alone cannot
compute.

## Decision

**`bridge_doppler_velocity_from_wind` returns 0 for every finite input: the radial
velocity a steady wind adds to the Doppler shift between a source and a listener at
rest.** The function keeps its name and signature, so the Rust's functionality, a wind
term to compose with spatial's Doppler, is still there, and now composes to the correct
result: no shift. A non-finite wind or bearing returns NaN. It is passed on, not read
as a still day.

## Consequences

- **Positive.** A host that composes wind with spatial's Doppler, as documented, gets
  what physics gives: no pitch shift. `tests/hardening.tcyr` renders that composition,
  1000 samples in and 1000 out, bit for bit, with a moving source as the control. It
  fails on 2.3.0.
- **Negative.** A host that wanted wind to bend the pitch, for effect, no longer gets
  that from prani. It can compute `wind × cos(angle)` itself, knowing it is a stylistic
  choice and not a Doppler shift.
- **Neutral.** What a wind does audibly, to level downwind, to arrival time, and to
  refraction, is a propagation model prani does not have. That would be new
  functionality, not this function's.

## Alternatives considered

- **Negate the sign**, so that a tailwind raises the pitch, as the Rust's documentation
  seems to intend. Either sign is a shift that does not happen.
- **Keep the Rust's sign.** It was the documented contradiction that raised the issue,
  and it is still a shift that does not happen.
- **Remove the function.** That loses the Rust's functionality, and breaks a caller in a
  patch release, when the accurate value can simply be returned.
