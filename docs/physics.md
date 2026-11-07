# Physics and independent interpretation

For the signed, labeled spring, r=x₂−x₁−L₀, U=kr²/2, F₁=kr and F₂=−kr. K=(m₁v₁²+m₂v₂²)/2 and P=m₁v₁+m₂v₂. Hence dP/dt=F₁+F₂=0. Also dK/dt=kr(v₁−v₂) and dU/dt=kr(v₂−v₁), so dE/dt=0 in the declared closed conservative model.

Define M=m₁+m₂, μ=m₁m₂/M, X=(m₁x₁+m₂x₂)/M and V=P/M. Then X=X₀+V₀t, and r satisfies r″+(k/μ)r=0. With ω=√(k/μ):

```text
r(t) = r₀ cos(ωt) + (u₀/ω) sin(ωt)
u(t) = −r₀ω sin(ωt) + u₀ cos(ωt)
x₁ = X − (m₂/M)(L₀+r), x₂ = X + (m₁/M)(L₀+r)
v₁ = V − (m₂/M)u, v₂ = V + (m₁/M)u
E = M V²/2 + μ u²/2 + k r²/2
```

The analytic fixture uses m₁=1 kg, m₂=2 kg, k=3 N/m, L₀=1 m, x₁=0, x₂=1.2 m, v₁=0.4 m/s and v₂=−0.1 m/s. Independently: K=0.09 J, U=0.06 J, E=0.15 J, P=0.2 kg m/s, X=0.8 m, V=1/15 m/s. Tests compare to these constants, not just generator output.

Internal passive damping F₁=kr+cu, F₂=−F₁ gives dE/dt=−cu²≤0 for c≥0 while P stays constant. Our fixture solves r″+0.3r′+4.5r=0 analytically (c=0.2 kg/s). It retains the same free COM motion. Optional energy non-increase checking is an adjacent-sample monotonicity test, not a full thermal budget.

The driven example adds a common acceleration of 0.1 m/s² to both masses. It has nonzero net external force; no work or impulse budget is provided. Constant E and P are therefore not tested. A wall-attached oscillator similarly exchanges momentum with its wall and is not a momentum-conserving single-body reference.

Explicit Euler uses the old positions and velocities for every update. Velocity Verlet uses old acceleration for positions and average old/new acceleration for velocities. At finite steps, Verlet's energy generally oscillates; these fixtures do not establish exact conservation or universal integrator stability.

The analyzer evaluates exported state only. It never integrates a trajectory, estimates missing external budgets, or infers the integrator. Finite f64 precision, export rounding, and unobserved intervals remain limitations. Conserved quantities are necessary model-specific tests, not sufficient physical validation.

Further reading: [Hairer, Lubich and Wanner, Geometric Numerical Integration](https://www.unige.ch/~hairer/gni.html); [OpenStax University Physics, conservation of momentum](https://openstax.org/books/university-physics-volume-1/pages/9-3-conservation-of-linear-momentum). These are conceptual references; this implementation has no runtime dependency on them.
