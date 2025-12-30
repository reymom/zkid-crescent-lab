#!/usr/bin/env bash
# Clone Microsoft's crescent-credentials repo into ./vendor and initialize required submodules.

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VENDOR_DIR="$ROOT_DIR/vendor"
CRESCENT_DIR="$VENDOR_DIR/crescent-credentials"

mkdir -p "$VENDOR_DIR"

if [ -d "$CRESCENT_DIR" ]; then
  echo "OK: Crescent already present at $CRESCENT_DIR"
else
  git clone https://github.com/microsoft/crescent-credentials "$CRESCENT_DIR"
fi

(
  cd "$CRESCENT_DIR"
  git submodule update --init --recursive
)

echo "OK: Crescent vendored."
echo "Next:"
echo "  ./scripts/bootstrap_python.sh"
echo "  source .venv/bin/activate"
echo "  ./scripts/setup_vectors.sh rs256-sd rs256-db"
