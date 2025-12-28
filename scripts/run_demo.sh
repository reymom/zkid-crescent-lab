#!/usr/bin/env bash
# zkid-crescent-lab/scripts/run_demo.sh
# Purpose: run Crescent's CLI steps for a given param set (assumes circuit setup already ran)

set -euo pipefail

PARAM="${1:-rs256-sd}"
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CREDS_DIR="${ROOT_DIR}/vendor/crescent-credentials/creds"

if [[ ! -d "${CREDS_DIR}" ]]; then
  echo "Missing vendor Crescent. Run: ./scripts/vendor_crescent.sh"
  exit 1
fi

cd "${CREDS_DIR}"

cargo run --bin crescent --release --features print-trace zksetup --name "${PARAM}"
cargo run --bin crescent --release --features print-trace prove   --name "${PARAM}"
cargo run --bin crescent --release --features print-trace show    --name "${PARAM}" --presentation-message "demo:${PARAM}:$(date -Iseconds)"
cargo run --bin crescent --release --features print-trace verify  --name "${PARAM}" --presentation-message "demo:${PARAM}:$(date -Iseconds)"
