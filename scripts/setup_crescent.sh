#!/usr/bin/env bash
# zkid-crescent-lab/scripts/setup_crescent.sh
# Purpose: make Crescent's setup scripts + Rust crate compile on a typical dev machine.

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CRESCENT_DIR="${ROOT_DIR}/vendor/crescent-credentials"

if [[ ! -d "${CRESCENT_DIR}/.git" ]]; then
  echo "Missing vendor Crescent at ${CRESCENT_DIR}. Run: ./scripts/vendor_crescent.sh" >&2
  exit 1
fi

"${ROOT_DIR}/scripts/setup_python.sh"
"${ROOT_DIR}/scripts/patch_crescent_rust.sh"

echo "Crescent local setup complete. Next:" 
echo "  (cd vendor/crescent-credentials/circuit_setup/scripts && ./run_setup.sh rs256-sd)"
echo "  (cd vendor/crescent-credentials/creds && cargo test --release)"
