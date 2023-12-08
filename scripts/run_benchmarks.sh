#!/bin/bash
set -eou pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
cd ../
cargo build --locked --features runtime-benchmarks

IMBUE=./target/debug/imbue
ERR_FILE="benchmarking_err.txt"

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
    
    OUTPUT=$(
        $IMBUE benchmark pallet \
        --chain="imbue-dev" \
        --steps=50 \
        --repeat=20 \
        --pallet="$PALLET" \
        --extrinsic="*" \
        --output="$WEIGHT_FILE" \
        --template="./scripts/frame-weight-template.hbs" 2>&1
    )

    if [ $? -ne 0 ]; then
      echo "$OUTPUT" >> "$ERR_FILE"
      echo "[-] Failed to benchmark $PALLET. Error written to $ERR_FILE; continuing..."
    fi
done

exit 0