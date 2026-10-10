#!/bin/bash
# Build release + .deb. Run: extras/scripts/pack.sh [output dir]  (default: target/deb)
set -euo pipefail
cd "$(dirname "$0")/../.."
cargo build --release --workspace
OUT=${1:-target/deb}
mkdir -p "$OUT"
cargo deb -p latch-app --no-build --output "$OUT/"
ls -l "$OUT"/*.deb
