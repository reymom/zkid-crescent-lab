#!/usr/bin/env bash
# Create a repo-local Python venv with the deps needed by Crescent's circuit_setup scripts.

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VENV_DIR="$ROOT_DIR/.venv"
REQ_FILE="$ROOT_DIR/requirements.txt"

PYTHON_BIN="${PYTHON_BIN:-python3}"

if ! command -v "$PYTHON_BIN" >/dev/null 2>&1; then
  echo "ERROR: python not found. Set PYTHON_BIN=python3.11 (or similar) and retry." >&2
  exit 1
fi

if [ ! -d "$VENV_DIR" ]; then
  "$PYTHON_BIN" -m venv "$VENV_DIR"
fi

# shellcheck disable=SC1091
source "$VENV_DIR/bin/activate"

python -m pip install -U pip wheel
python -m pip install -r "$REQ_FILE"

echo "OK: venv ready."
echo "Next: source .venv/bin/activate"