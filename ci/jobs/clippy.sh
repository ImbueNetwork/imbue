#!/bin/bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
cd ../..

echo "Starting clippy!"
RUSTFLAGS="-D warnings" cargo clippy --all --features runtime-benchmarks