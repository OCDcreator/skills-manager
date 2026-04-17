#!/usr/bin/env bash
set -euo pipefail

if command -v python3 >/dev/null 2>&1; then
  PYTHON_CMD=(python3)
elif command -v python >/dev/null 2>&1; then
  PYTHON_CMD=(python)
else
  echo "[watch] neither python3 nor python was found in PATH" >&2
  exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
AUTOPILOT_PY="$SCRIPT_DIR/autopilot.py"

exec "${PYTHON_CMD[@]}" "$AUTOPILOT_PY" watch "$@"
