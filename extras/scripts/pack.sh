#!/bin/bash
# Build release + .deb into target/deb/. Run: extras/scripts/pack.sh
set -euo pipefail
cd "$(dirname "$0")/../.."
cargo build --release --workspace
cargo deb -p latch-app --no-build --output target/deb/
ls -l target/deb/*.deb
