# Deterministic fixture catalog

All files contain 1,001 samples at dt=0.02 s over 0–20 s. Model parameters and initial states are given in docs/physics.md. Tolerances accompany each fixture in a matching TOML file.

| ID                  | Construction                                                        | Expected result                                                         |
| ------------------- | ------------------------------------------------------------------- | ----------------------------------------------------------------------- |
| analytic            | Exact COM/relative-coordinate solution                              | E and P pass to floating-point precision                                |
| euler               | Explicit Euler; all updates use old state                           | Energy grows and fails; momentum passes                                 |
| verlet              | Velocity Verlet, recomputed position accelerations                  | Bounded nonzero energy error passes the declared 1% tolerance; P passes |
| energy-corruption   | Analytic, at index 250 add +0.2 to v1 and −0.1 to v2                | Energy fails at t=5 s, row 252; P passes                                |
| momentum-corruption | Analytic, at index 250 subtract twice COM velocity from both v1/v2  | Momentum reverses; energy preserved; only P fails                       |
| damped              | Analytic underdamped relative solution with c=0.2 kg/s and free COM | E declines; E conservation N/A; non-increase and P pass                 |
| driven              | Analytic relative motion plus common 0.1 m/s² acceleration          | Diagnostic only; both conservation checks N/A; UNVERIFIED               |

Regenerate into a fresh directory with `cargo run -p conservationci-cli -- fixtures generate --output /tmp/conservationci-fixtures`. The generator refuses to overwrite existing files. For intentional fixture updates, review fresh output before replacing tracked files. The server embeds the checked-in fixture bytes at compile time; rebuild after changing them.

Generator outputs are deterministic on the same build/platform. System sin/cos/exp implementations can differ in final bits across platforms; fixed checked-in bytes are the cross-platform test inputs. None of these fixtures validates arbitrary simulator trajectories or identifies an integrator from observed errors.
