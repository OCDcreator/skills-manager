#!/usr/bin/env bash
set -euo pipefail

BACKGROUND="0"
PID_PATH="automation/runtime/autopilot.pid"
OUTPUT_PATH="automation/runtime/autopilot-session.out"
ARGS=()

usage() {
  cat <<'EOF'
Usage: ./automation/start-autopilot.sh [options] [-- autopilot-args...]

Options:
  --background             Launch unattended work through nohup and return immediately
  --pid-path <path>        PID file to write in background mode
  --output-path <path>     Combined stdout/stderr file in background mode
  -h, --help               Show this help

Everything after `--` is passed to `automation/autopilot.py start`.
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --background) BACKGROUND="1"; shift ;;
    --pid-path) PID_PATH="$2"; shift 2 ;;
    --output-path) OUTPUT_PATH="$2"; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    --) shift; ARGS+=("$@"); break ;;
    *) ARGS+=("$1"); shift ;;
  esac
done

if command -v python3 >/dev/null 2>&1; then
  PYTHON_CMD=(python3)
elif command -v python >/dev/null 2>&1; then
  PYTHON_CMD=(python)
else
  echo "[start] neither python3 nor python was found in PATH" >&2
  exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
AUTOPILOT_PY="$SCRIPT_DIR/autopilot.py"
DEFAULT_ARGS=(start --profile mac)

if [[ "$BACKGROUND" == "1" ]]; then
  mkdir -p "$(dirname "$PID_PATH")"
  mkdir -p "$(dirname "$OUTPUT_PATH")"
  nohup "${PYTHON_CMD[@]}" "$AUTOPILOT_PY" "${DEFAULT_ARGS[@]}" "${ARGS[@]}" >>"$OUTPUT_PATH" 2>&1 &
  pid=$!
  echo "$pid" >"$PID_PATH"
  echo "[start] background autopilot pid: $pid"
  echo "[start] pid file: $PID_PATH"
  echo "[start] output file: $OUTPUT_PATH"
  exit 0
fi

exec "${PYTHON_CMD[@]}" "$AUTOPILOT_PY" "${DEFAULT_ARGS[@]}" "${ARGS[@]}"
