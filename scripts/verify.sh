#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
# Keep documentation tests on the same toolchain even when PATH contains another rustdoc.
if command -v rustup >/dev/null 2>&1; then
  export RUSTDOC
  RUSTDOC="$(rustup which rustdoc)"
fi
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
pnpm --dir web run typecheck
pnpm --dir web run lint
pnpm --dir web test
pnpm --dir web run build
