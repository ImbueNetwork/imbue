#!/bin/bash
set -eou pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
cd ../
cargo clean
cargo build --release --locked --features runtime-benchmarks

IMBUE=./target/release/imbue

EXCLUDED_PALLETS=( 
    "frame_benchmarking"
    "frame_system"
    "pallet_balances"
    "pallet_timestamp"
)

ALL_PALLETS=($(
  $IMBUE benchmark pallet --list --chain=imbue-dev |\
    tail -n+2 |\
    cut -d',' -f1 |\
    sort |\
    uniq
));

PALLETS=($({ printf '%s\n' "${ALL_PALLETS[@]}" "${EXCLUDED_PALLETS[@]}"; } | sort | uniq -u))

echo "Benchmarking ${#PALLETS[@]} Imbue pallets."
for PALLET in "${PALLETS[@]}"; do

    FOLDER="$(echo "${PALLET#*_}" | tr '_' '-')";
    WEIGHT_FILE="./pallets/${FOLDER}/src/weights.rs"
    echo "Benchmarking $PALLET with weight file $WEIGHT_FILE";
    
    $IMBUE benchmark pallet \
    --chain="imbue-dev" \
    --steps=50 \
    --repeat=20 \
    --pallet="$PALLET" \
    --extrinsic="*" \
    --output="$WEIGHT_FILE" \
    --template="./scripts/frame-weight-template.hbs" 2>&1

done
exit 0