#!/usr/bin/env bash
# zkid-crescent-lab/scripts/patch_crescent_rust.sh
# Purpose: patch Crescent's Rust workspace for a smooth local build.

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CREDS_TOML="${ROOT_DIR}/vendor/crescent-credentials/creds/Cargo.toml"

if [[ ! -f "${CREDS_TOML}" ]]; then
  echo "Missing ${CREDS_TOML}. Did you run vendor_crescent.sh?" >&2
  exit 1
fi

# Problem:
# Crescent uses the `base64-url` crate (module name `base64_url`) and calls `base64_url::decode(...)`.
# The error type coming from the underlying `base64` crate implements `std::error::Error` only when
# compiled with the `std` feature. If `base64-url` is pulled in without `std`, `?` won't coerce
# the error into `Box<dyn Error>`.
#
# Fix: ensure `base64-url` is compiled with `features = ["std"]`.
# Docs: https://docs.rs/base64-url shows the recommended feature flag. (Enable std features.)

if ! grep -qE '^base64-url\s*=.*' "${CREDS_TOML}"; then
  echo "No base64-url dependency line found in creds/Cargo.toml. Skipping patch." >&2
  exit 0
fi

echo "Patching base64-url dependency to enable std feature: ${CREDS_TOML}"

python3 - <<'PY'
import re
from pathlib import Path

path = Path("vendor/crescent-credentials/creds/Cargo.toml")
txt = path.read_text(encoding="utf-8")

def patch_line(line: str) -> str:
    # base64-url = "2.0.0"  -> base64-url = { version = "2.0.0", features = ["std"] }
    m = re.match(r'^(base64-url\s*=\s*)"([^"]+)"\s*$', line)
    if m:
        return f'{m.group(1)}{{ version = "{m.group(2)}", features = ["std"] }}'

    # base64-url = { ... } -> ensure features = ["std"] exists (add if missing)
    m = re.match(r'^(base64-url\s*=\s*\{)(.*)(\})\s*$', line)
    if not m:
        return line

    body = m.group(2)
    if re.search(r'\bfeatures\s*=\s*\[', body):
        # already has features; trust it
        return line

    # Insert before closing brace.
    body_stripped = body.strip()
    if not body_stripped:
        body2 = ' features = ["std"] '
    else:
        # Ensure trailing comma for TOML inline table.
        body2 = body.rstrip()
        if not body2.strip().endswith(','):
            body2 += ','
        body2 += ' features = ["std"]'

    return f'{m.group(1)}{body2} {m.group(3)}'

lines = txt.splitlines()
out = []
changed = False
for line in lines:
    if line.strip().startswith("base64-url"):
        new_line = patch_line(line)
        if new_line != line:
            changed = True
        out.append(new_line)
    else:
        out.append(line)

if changed:
    path.write_text("\n".join(out) + "\n", encoding="utf-8")
    print("Applied patch to base64-url dependency line.")
else:
    print("No changes needed (base64-url already has features or uses a different format).")
PY

echo "Rust patch step complete. If you still see base64_url DecodeError trait issues, run:" 
echo "  cd vendor/crescent-credentials/creds && cargo clean && cargo test --release"
