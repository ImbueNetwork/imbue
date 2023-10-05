#!/bin/bash
set -euo pipefail
sudo apt install build-essential
sudo apt install --assume-yes git clang curl libssl-dev protobuf-compiler
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustup update
rustup toolchain install nightly-2023-05-22
rustup target add wasm32-unknown-unknown --toolchain nightly-2023-05-22
rustup component add rustfmt
rustup component add clippy
