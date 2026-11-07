# CI integration

Build this workspace or install from a pinned source revision, then point the CLI at your simulation export and matching model. The CLI runs independently of Node and HTTP. Never absorb a failed check with `|| true` in the actual gate.

```yaml
name: Physics audit
on: [push, pull_request]
permissions:
  contents: read
jobs:
  physics:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo build --release --locked -p conservationci-cli
      - name: Audit exported trajectory
        run: |
          target/release/conservationci check \
            --input examples/fixtures/analytic.csv \
            --config examples/models/conservative.toml \
            --report-json report.json --report-md report.md \
            --samples-csv samples.csv --github-annotations
      - uses: actions/upload-artifact@v4
        if: always()
        with:
          name: conservation-evidence
          path: |
            report.json
            report.md
            samples.csv
```

Replace the input with your simulator's output and set explicit justified tolerances. `--github-annotations` emits one error per failed check at its first failing CSV source line. Filenames/messages escape workflow-command metacharacters. The exit code remains authoritative: 0 passed or UNVERIFIED, 1 physical failure, 2 invalid input/config, 3 execution error. Decide separately whether your project permits an UNVERIFIED status.

The repository's CI workflow also exercises an expected failing Euler run, asserts exit 1, and uploads its reports. A local workflow-equivalent run does not establish that hosted GitHub Actions has executed.
