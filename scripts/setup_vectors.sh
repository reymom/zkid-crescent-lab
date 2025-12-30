#!/usr/bin/env bash
# Generate Crescent circuits + test vectors for one or more parameter sets via circuit_setup/run_setup.sh.

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VENV_DIR="${ROOT_DIR}/.venv"

if [ "$#" -lt 1 ]; then
  echo "Usage: $0 <param1> [param2 ...]" >&2
  exit 2
fi

if [ ! -x "${VENV_DIR}/bin/python" ]; then
  echo "Missing venv at ${VENV_DIR}. Run:" >&2
  echo "  ./scripts/bootstrap_python.sh" >&2
  exit 1
fi

RUN_SETUP_SH="${ROOT_DIR}/vendor/crescent-credentials/circuit_setup/scripts/run_setup.sh"
if [ ! -x "${RUN_SETUP_SH}" ]; then
  echo "run_setup.sh not found/executable at:" >&2
  echo "  ${RUN_SETUP_SH}" >&2
  echo "Did you run: ./scripts/vendor_crescent.sh ?" >&2
  exit 1
fi

# Ensure the circuit_setup scripts resolve python3/pip packages from our repo-root venv.
export PATH="${VENV_DIR}/bin:${PATH}"

for param in "$@"; do
  echo
  echo "[setup_vectors] Generating vectors for: ${param}"
  (cd "$(dirname "${RUN_SETUP_SH}")" && ./run_setup.sh "${param}")
done

echo
echo "[setup_vectors] Done."