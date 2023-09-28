#!/bin/bash
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")"
echo "Cleaning swap and apt"

sudo swapoff -a
sudo rm -f /swapfile
sudo apt clean