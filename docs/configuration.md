# Configuration contract (schema 1)

TOML is parsed strictly: unknown fields are errors, including unknown regimes. Required fields: schema_version=1, model="two_mass_spring_1d", units="SI", regime, masses.m1, masses.m2, spring.stiffness, spring.rest_length. Masses and stiffness must be finite and positive; rest length must be finite and nonnegative. The regime is a declaration the tool cannot independently establish.

Regimes:

- `closed_conservative`: energy and momentum conservation are physically applicable.
- `internal_dissipation`: energy conservation is skipped; momentum is applicable. Declares equal/opposite passive internal damping. Optional `[checks.energy_monotonicity]` tests energy non-increase.
- `externally_driven`: energy and momentum conservation are skipped. Enabling their tolerances cannot force applicability.

`[checks.energy]` defaults to absolute_tolerance=1e-6 J and relative_tolerance=0.01. `[checks.momentum]` defaults to absolute_tolerance=1e-8 kg m/s and relative_tolerance=1e-6. Both accept `enabled=false`. A monotonicity table defaults to absolute_tolerance=1e-8 J and relative_tolerance=1e-6 when present; without that table it is disabled. Monotonicity outside internal_dissipation is rejected.

All tolerances are finite and nonnegative. `reference_scale`, if present, must be finite and positive, in the checked quantity's units.

```text
allowed = absolute_tolerance + relative_tolerance * max(abs(reference), reference_scale or 0)
```

Conservation uses the initial sample reference; failure requires abs(Qᵢ−Q₀)>allowed. Equality passes. Monotonicity uses Eᵢ−Eᵢ₋₁>allowed(Eᵢ₋₁); the first sample has no preceding transition. A declining energy never fails that check. The optional reference scale is explicit: there is no automatic scale of one.

Column mapping defaults to:

```toml
[columns]
time = "t"
x1 = "x1"
v1 = "v1"
x2 = "x2"
v2 = "v2"
```

Names must be nonempty, distinct, and occur exactly once in the CSV. Partial tables inherit defaults. The dashboard offers mapping selectors and the complete TOML editor. Tolerance and status arithmetic always happens in Rust before presentation rounding.
