#!/usr/bin/env bash
# Install Python deps required by Crescent's circuit_setup scripts in an isolated venv.

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VENV_DIR="${ROOT_DIR}/.venv"
REQ_FILE="${ROOT_DIR}/requirements.txt"

if ! command -v python3 >/dev/null 2>&1; then
  echo "python3 not found. Please install Python 3." >&2
  exit 1
fi

if [ ! -f "${REQ_FILE}" ]; then
  echo "Missing ${REQ_FILE} (expected at repo root)." >&2
  exit 1
fi

if [ ! -d "${VENV_DIR}" ]; then
  echo "[setup_python] Creating venv at ${VENV_DIR}"
  python3 -m venv "${VENV_DIR}"
else
  echo "[setup_python] Reusing existing venv at ${VENV_DIR}"
fi

# Always use the venv's interpreter + pip explicitly (no reliance on 'source')
PY="${VENV_DIR}/bin/python"
PIP="${VENV_DIR}/bin/pip"

echo "[setup_python] Upgrading pip/setuptools/wheel"
"${PY}" -m pip install --upgrade pip setuptools wheel >/dev/null

echo "[setup_python] Installing requirements from ${REQ_FILE}"
"${PIP}" install -r "${REQ_FILE}"

echo
echo "[setup_python] Done."
echo "Activate with:"
echo "  source ${VENV_DIR}/bin/activate"