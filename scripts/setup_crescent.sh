#!/usr/bin/env bash
# Purpose: make Crescent's setup scripts + Rust crate compile on a typical dev machine.

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CRESCENT_DIR="${ROOT_DIR}/vendor/crescent-credentials"

if [ ! -d "${CRESCENT_DIR}" ]; then
  echo "Crescent not found at ${CRESCENT_DIR}. Run: ./scripts/vendor_crescent.sh" >&2
  exit 1
fi

echo "[setup_crescent] Updating submodules (circomlib, etc.)"
git -C "${CRESCENT_DIR}" submodule update --init --recursive

# Optional but recommended: force the Crescent repo to use the lab toolchain when you run cargo inside it.
if command -v rustup >/dev/null 2>&1; then
  if rustup toolchain list | grep -q "^1.88.0"; then
    echo "[setup_crescent] Setting rustup override 1.88.0 inside vendor/crescent-credentials"
    (cd "${CRESCENT_DIR}" && rustup override set 1.88.0) >/dev/null
  else
    echo "[setup_crescent] rustup toolchain 1.88.0 not installed (skipping override)."
    echo "  Install with: rustup toolchain install 1.88.0"
  fi
fi

# Local patch used by the lab (kept as a script so it's explicit + reviewable).
echo "[setup_crescent] Applying local Rust patch (lab-only)"
"${ROOT_DIR}/scripts/patch_crescent_rust.sh"

echo "[setup_crescent] Done."