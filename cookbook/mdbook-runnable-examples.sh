#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/rust-bitcoin-cookbook-mdbook-runnable-examples-target}"
exec cargo run --quiet --manifest-path "$SCRIPT_DIR/../tools/mdbook-runnable-examples/Cargo.toml" -- "$@"
