set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CREDS_DIR="${ROOT_DIR}/vendor/crescent-credentials/creds"

if [ ! -d "${CREDS_DIR}" ]; then
  echo "Crescent creds dir not found at ${CREDS_DIR}." >&2
  echo "Run: ./scripts/vendor_crescent.sh" >&2
  exit 1
fi

cd "${CREDS_DIR}"

# Prefer the lab toolchain explicitly to avoid surprises.
if command -v cargo >/dev/null 2>&1; then
  if cargo +1.88.0 --version >/dev/null 2>&1; then
    exec cargo +1.88.0 test --release
  fi
fi

echo "cargo +1.88.0 not available. Install it with:" >&2
echo "  rustup toolchain install 1.88.0" >&2
exit 1