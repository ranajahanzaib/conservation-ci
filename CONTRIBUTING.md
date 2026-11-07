# Contributing

Use current stable Rust, Node.js 22+, and pnpm. Build the workspace, install web dependencies with `pnpm --dir web install --frozen-lockfile`, and follow the README's development commands. Keep PLAN.md unchanged.

Before submitting, run Rust formatting, warning-denied Clippy, all workspace tests, frontend typecheck/lint/tests, and the production build. Keep rustdoc and rustc from the same toolchain.

Changes to numerical calculations need independent known-value assertions and edge-case tests. Respect model applicability, SI units, signed spring orientation, zero-reference tolerance semantics, and deterministic report ordering. Do not infer causal defects from conservation evidence or add frontend copies of physics logic. New report fields need documentation; breaking changes require a schema version decision.

Keep dependencies restrained. Do not commit target, node_modules, dist, transient reports, credentials, or large datasets. Generate proposed fixture changes into a new directory and review their effects on all checks. Include the trigger, expected/actual behavior, and exact commands/results in a pull request. The project follows CODE_OF_CONDUCT.md and uses the MIT license.
