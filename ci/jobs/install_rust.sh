#!/bin/bash
set -euo pipefail
sudo apt -y install build-essential
sudo apt -y install --assume-yes git clang curl libssl-dev  
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustup -y update
#rustup -y toolchain install nightly-2023-05-22
#rustup -y target add wasm32-unknown-unknown --toolchain nightly-2023-05-22
#rustup -y component add rustfmt
#rustup -y component add clippy
