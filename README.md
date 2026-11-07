# ConservationCI

**Catch broken physics before it ships.**

ConservationCI independently audits exported simulation trajectories against model-specific conservation expectations, configured tolerances, and traceable evidence. It includes a Rust engine and CLI, a local HTTP API, and a React dashboard.

**GitHub repository description:** Audit physics simulation trajectories against declared conservation laws. Local Rust CLI, API, and React dashboard with CI-ready reports.

**Suggested GitHub topics:** `physics`, `computational-physics`, `scientific-computing`, `simulation`, `conservation-laws`, `rust`, `continuous-integration`, `trajectory-analysis`

ConservationCI audits exported physics trajectories against the invariants justified by a declared physical model. It provides a reusable Rust engine, standalone CLI, localhost HTTP API, and React diagnostic dashboard. No accounts, databases, external datasets, or network services are needed after installation.

## What it catches

An exported trajectory can look plausible while its energy grows or its total momentum jumps. ConservationCI evaluates **every sample**, identifies the first threshold crossing and source CSV row, and exports reproducible evidence for debugging and CI. It does not integrate your trajectory or identify a simulator's root cause.

Try the built-in analytic, Euler, Verlet, corruption, damping, and driven examples in the dashboard. The energy-only and momentum-only corruptions occur at **t=5 s, source row 252**. The analytic reference begins with **0.15 J** and **0.2 kg m/s**. These values are independently tested.

## Supported physical model

Two freely moving masses joined by a labeled, oriented ideal spring in one dimension:

```text
extension = (x2 - x1) - L0
K = (m1*v1² + m2*v2²)/2
U = k*extension²/2
E = K + U
P = m1*v1 + m2*v2
```

This is a signed-displacement spring, not an absolute-distance or collision model. A fixed-wall oscillator is different: the wall supplies an external force, so the moving mass alone generally does not conserve momentum. See [the derivation](docs/physics.md).

| Declared regime        | Mechanical energy                                                 | Total momentum     |
| ---------------------- | ----------------------------------------------------------------- | ------------------ |
| `closed_conservative`  | Conservation check                                                | Conservation check |
| `internal_dissipation` | Conservation not applicable; optional adjacent non-increase check | Conservation check |
| `externally_driven`    | Diagnostic only                                                   | Diagnostic only    |

Enabling a check never overrides regime applicability. Passive internal dissipation is a declared assumption; the auditor cannot establish it from CSV alone.

## Installation

Requires current stable Rust (edition 2024), Node.js 22+, and pnpm 12.3.4 (pinned in `web/package.json`). Run from the repository root:

```bash
cargo build --release --locked
pnpm --dir web install --frozen-lockfile
pnpm --dir web run build
```

The executable is `target/release/conservationci`. Optionally install it with `cargo install --path crates/cli --locked`. The dashboard assets remain in `web/dist`; pass `--web /absolute/path/to/web/dist` when launching elsewhere. The standalone CLI needs no frontend build or server.

## CLI quick start

Fixtures are already included. Generate a new copy without overwriting them:

```bash
cargo run -p conservationci-cli -- fixtures generate --output /tmp/conservationci-fixtures

# Pass: exit 0
cargo run -p conservationci-cli -- check \
  --input examples/fixtures/analytic.csv \
  --config examples/models/conservative.toml

# Energy failure: exit 1; momentum still passes
cargo run -p conservationci-cli -- check \
  --input examples/fixtures/euler.csv \
  --config examples/models/conservative.toml \
  --report-json /tmp/conservationci-report.json \
  --report-md /tmp/conservationci-report.md \
  --samples-csv /tmp/conservationci-samples.csv \
  --github-annotations

# Contract validation, independently of physical failures
cargo run -p conservationci-cli -- validate \
  --input examples/fixtures/euler.csv \
  --config examples/models/conservative.toml
```

`fixtures generate` without `--output` targets `examples/fixtures`. Existing files cause an error; it never overwrites them. Use a new directory for each generation. `--help` and `--version` are available.

| Exit | Meaning                                                                           |
| ---- | --------------------------------------------------------------------------------- |
| 0    | All applicable checks passed, or valid `UNVERIFIED` run with no applicable checks |
| 1    | At least one enabled physical check failed                                        |
| 2    | Invalid input/configuration or missing input file                                 |
| 3    | Execution failure, including report write or server errors                        |

A zero exit code for `UNVERIFIED` is an operational policy, not scientific verification. JSON and Markdown reports are written even for physical failures. Validation errors appear as JSON on stderr.

## Dashboard quick start

After building the frontend:

```bash
cargo run -p conservationci-cli -- serve --port 8787
```

Open [the local dashboard](http://127.0.0.1:8787). Choose a real bundled fixture, or import/drop a CSV, edit its TOML, map columns, and run analysis. Inspect energy, momentum, phase space, tolerance boundaries, source samples, and failure evidence. Play/scrub the imported positions; reduced-motion preferences disable playback. Downloads come from the Rust backend and retain the analyzed input even if the editor changes afterward.

For frontend development, run the Rust server in one terminal and `pnpm --dir web run dev` in another. Vite proxies `/api` to port 8787. The API always binds `127.0.0.1`, with no permissive CORS or browser-controlled filesystem paths.

## CSV schema

```csv
t,x1,v1,x2,v2
0,0,0.4,1.2,-0.1
```

This excerpt is just one initial state; analysis needs at least two samples. SI units: t in seconds; positions in meters; velocities in m/s; masses in kg; k in N/m; energy in J; momentum in kg m/s. Time must be finite and strictly increasing; irregular intervals are supported. Invalid rows are rejected, never discarded, reordered, or repaired. Extra columns are reported as ignored. Default limits: **32 MiB**, **250,000 samples**. Source rows are physical CSV start lines, including for quoted multiline records. See [CSV format](docs/csv-format.md).

## TOML model and tolerances

Use [conservative.toml](examples/models/conservative.toml) as a starting point:

```toml
schema_version = 1
model = "two_mass_spring_1d"
regime = "closed_conservative"
units = "SI"
[masses]
m1 = 1.0
m2 = 2.0
[spring]
stiffness = 3.0
rest_length = 1.0
[checks.energy]
absolute_tolerance = 0.000001
relative_tolerance = 0.01
[checks.momentum]
absolute_tolerance = 0.00000001
relative_tolerance = 0.000001
```

The initial sampled quantity Q₀ is the reference. A violation means:

```text
abs(Qᵢ − Q₀) > absolute_tolerance + relative_tolerance * abs(Q₀)
```

Equality passes. An optional positive `reference_scale` changes the relative scale to `max(abs(Q₀), reference_scale)`. No universal dimensionful scale is inserted. At zero reference, absolute tolerance works normally. With zero allowed error, a nonzero error fails; its normalized ratio is null with an explicit `zero_tolerance_exceeded` flag.

These defaults are illustrative, not universal accuracy guarantees. Choose tolerances for your model, units, integrator, and export precision. Monotonicity compares adjacent energies using a local tolerance based on the preceding energy; it is not energy conservation. [Configuration reference](docs/configuration.md).

## Generated examples

All seven deterministic Rust-generated fixture families, their numerical methods, parameters, and expectations are documented in [examples/README.md](examples/README.md). The API embeds the actual checked-in CSV/TOML files; no browser simulation or fabricated result is used.

## GitHub Actions

[The repository workflow](.github/workflows/ci.yml) builds, checks formatting and Clippy, tests Rust and React, generates fresh fixtures, runs the CLI with annotations, and uploads reports even on failure. Use `check` directly in your simulation's workflow; its exit code fails the job. See [CI integration](docs/ci-integration.md) for a copyable example.

## Reports

Versioned JSON includes tool version, input SHA-256, canonical parsed-config SHA-256, regime, applicability, configured tolerances, summary metrics, source locations, and deterministic explanations. No generation timestamp enters the canonical report. JSON keeps at most 1,000 violation events while retaining full counts; the full-sample CSV includes every analyzed sample. [Report schema](docs/report-schema.md).

Diagnostics include first crossing, peak/time, final signed error, RMS, first/last 20% RMS, signed OLS slope, and longest consecutive violation streak. They describe error behavior without automatically identifying an integrator, declaring monotonic drift, or assigning a root cause.

## Testing

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
pnpm --dir web run typecheck
pnpm --dir web run lint
pnpm --dir web test
pnpm --dir web run build
```

Run all checks with `./scripts/verify.sh`.

Rust tests cover independent known-value physics assertions, every fixture family, tolerances, invalid input, deterministic reports, 100,000 samples, subprocess exit codes, exports, and API/core equivalence. Frontend interaction tests use a mocked API and chart renderer; real charts should also be exercised in a browser. Keep rustc and rustdoc from the same stable toolchain if your PATH includes multiple installations.

## Architecture

`crates/core` owns validation, physics, fixtures, analysis, presentation reduction, and reports. `crates/cli` owns commands and exit policy. `crates/server` adapts the same engine to a bounded, stateless Axum API and serves the frontend. `web` renders backend results with React, TypeScript, Recharts, and original SVG. [Architecture and API](docs/architecture.md).

Analysis uses bounded O(n) memory. Charts use bucket extrema and the first 1,000 threshold boundaries; the full analysis is never reduced. Dense crossing evidence is available in full CSV exports; charts can omit later crossings. The visible event list shows the first 100 events; selecting an event requests its exact original sample. Playback traverses plotted samples and is an inspection aid, not real-time simulation playback.

## Scientific limitations

- Conservation does not prove overall physical correctness, and violations do not uniquely identify a bug.
- Findings depend on declared model assumptions, adequate exported state, SI units, and chosen tolerances.
- Different integration methods have different finite-step errors. Bounded energy error is not exact conservation.
- f64 and CSV export precision limit accuracy; extreme arithmetic ranges are rejected.
- Export sampling may miss violations between samples.
- The first exported state is a reference, not independently verified ground truth.
- Dissipation needs different expectations; external driving needs work/impulse accounting for balance-law checks, which this version does not support.
- The oriented spring model has no collisions or absolute-distance behavior. Only this one model is implemented.
- The analyzer does not verify equations of motion and cannot replace model validation.
- Cross-platform transcendental rounding in fixture generation can differ at the last bits; checked-in fixture bytes provide a shared oracle. Reports are deterministic for identical bytes, config, and build/platform.

## Contributing and license

See [CONTRIBUTING.md](CONTRIBUTING.md). MIT licensed; see [LICENSE](LICENSE).
