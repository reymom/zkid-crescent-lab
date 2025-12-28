#!/usr/bin/env bash
# zkid-crescent-lab/scripts/setup_python.sh
# Purpose: install Python deps required by Crescent's circuit_setup scripts in an isolated venv.

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CRESCENT_SCRIPTS_DIR="${ROOT_DIR}/vendor/crescent-credentials/circuit_setup/scripts"
VENV_DIR="${CRESCENT_SCRIPTS_DIR}/.venv"

if [[ ! -d "${CRESCENT_SCRIPTS_DIR}" ]]; then
  echo "Missing Crescent circuit_setup scripts dir at ${CRESCENT_SCRIPTS_DIR}. Did you run vendor_crescent.sh?" >&2
  exit 1
fi

if [[ ! -x "$(command -v python3)" ]]; then
  echo "python3 not found. Install Python 3.x first." >&2
  exit 1
fi

if [[ ! -d "${VENV_DIR}" ]]; then
  echo "Creating Python venv: ${VENV_DIR}"
  python3 -m venv "${VENV_DIR}"
fi

# shellcheck disable=SC1091
source "${VENV_DIR}/bin/activate"

python -m pip install --upgrade pip wheel setuptools

# Minimal set inferred from Crescent's jwk_gen.py import path:
#   import python_jwt as jwt, jwcrypto.jwk as jwk
# We also include cryptography to satisfy common backend requirements.
python -m pip install "python-jwt" "jwcrypto" "cryptography"

echo "Python deps installed. Important: run setup scripts with this venv activated:" 
echo "  cd vendor/crescent-credentials/circuit_setup/scripts"
echo "  source .venv/bin/activate"
echo "  ./run_setup.sh rs256-sd"
