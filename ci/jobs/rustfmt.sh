#!/bin/bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
cd ../..

echo "Starting cargo fmt!"
cargo fmt --check --all
