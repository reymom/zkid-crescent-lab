#!/usr/bin/env bash
# zkid-crescent-lab/scripts/vendor_crescent.sh
# Purpose: fetch the Crescent Credentials repo into ./vendor for reproducible local runs

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VENDOR_DIR="${ROOT_DIR}/vendor"
CRESCENT_DIR="${VENDOR_DIR}/crescent-credentials"

mkdir -p "${VENDOR_DIR}"

if [[ -d "${CRESCENT_DIR}/.git" ]]; then
  echo "Crescent already vendored at: ${CRESCENT_DIR}"
  exit 0
fi

echo "Cloning Crescent into: ${CRESCENT_DIR}"
git clone --depth 1 https://github.com/microsoft/crescent-credentials "${CRESCENT_DIR}"

echo "Done."
echo "Next:"
echo "  1) ./scripts/setup_crescent.sh"
echo "  2) (cd vendor/crescent-credentials/circuit_setup/scripts && ./run_setup.sh rs256-sd)"
echo "  3) (cd vendor/crescent-credentials/creds && cargo test --release)"
echo "  4) cargo run --release -- run-all --param rs256-sd --show-iters 20"
