#!/usr/bin/env bash
set -euo pipefail

STATE_PATH="automation/runtime/autopilot-state.json"
PROFILE="mac"
CONFIG_PATH="automation/autopilot-config.json"
PROFILE_PATH=""
RESTART_PROFILE=""
RESTART_CONFIG_PATH=""
RESTART_STATE_PATH=""
RESTART_PROFILE_PATH=""
RESTART_SYNC_REF=""
RESTART_OUTPUT_PATH="automation/runtime/autopilot-cutover.out"
RESTART_PID_PATH="automation/runtime/autopilot-cutover.pid"
REFRESH_SECONDS="5"
STOP_TIMEOUT_SECONDS="30"
HARD_RESET="0"
STOP_IF_STATUS_CHANGES="0"

usage() {
  cat <<'EOF'
Usage: ./automation/arm-autopilot-cutover.sh [options]

Options:
  --state-path <path>              Current state file to watch
  --profile <name>                 Current profile name
  --config-path <path>             Current config path
  --profile-path <path>            Current external profile path
  --restart-profile <name>         Replacement profile name
  --restart-config-path <path>     Replacement config path
  --restart-state-path <path>      Replacement state path
  --restart-profile-path <path>    Replacement external profile path
  --restart-sync-ref <ref>         Replacement git ref to fast-forward before restart
  --restart-output-path <path>     Output log for replacement run
  --restart-pid-path <path>        PID file for replacement run
  --refresh-seconds <n>            Polling interval while waiting for next commit
  --stop-timeout-seconds <n>       Graceful stop timeout before force-kill
  --hard-reset                     Reset tracked changes to HEAD before restart
  --stop-if-status-changes         Abort if watched state stops being active before a new commit
  -h, --help                       Show this help
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --state-path) STATE_PATH="$2"; shift 2 ;;
    --profile) PROFILE="$2"; shift 2 ;;
    --config-path) CONFIG_PATH="$2"; shift 2 ;;
    --profile-path) PROFILE_PATH="$2"; shift 2 ;;
    --restart-profile) RESTART_PROFILE="$2"; shift 2 ;;
    --restart-config-path) RESTART_CONFIG_PATH="$2"; shift 2 ;;
    --restart-state-path) RESTART_STATE_PATH="$2"; shift 2 ;;
    --restart-profile-path) RESTART_PROFILE_PATH="$2"; shift 2 ;;
    --restart-sync-ref) RESTART_SYNC_REF="$2"; shift 2 ;;
    --restart-output-path) RESTART_OUTPUT_PATH="$2"; shift 2 ;;
    --restart-pid-path) RESTART_PID_PATH="$2"; shift 2 ;;
    --refresh-seconds) REFRESH_SECONDS="$2"; shift 2 ;;
    --stop-timeout-seconds) STOP_TIMEOUT_SECONDS="$2"; shift 2 ;;
    --hard-reset) HARD_RESET="1"; shift ;;
    --stop-if-status-changes) STOP_IF_STATUS_CHANGES="1"; shift ;;
    -h|--help) usage; exit 0 ;;
    *)
      echo "[cutover] unknown argument: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

if command -v python3 >/dev/null 2>&1; then
  PYTHON_CMD=(python3)
elif command -v python >/dev/null 2>&1; then
  PYTHON_CMD=(python)
else
  echo "[cutover] neither python3 nor python was found in PATH" >&2
  exit 1
fi

if [[ -z "$RESTART_PROFILE" ]]; then
  RESTART_PROFILE="$PROFILE"
fi

if [[ -z "$RESTART_CONFIG_PATH" ]]; then
  RESTART_CONFIG_PATH="$CONFIG_PATH"
fi

if [[ -z "$RESTART_STATE_PATH" ]]; then
  RESTART_STATE_PATH="$STATE_PATH"
fi

if [[ -z "$RESTART_PROFILE_PATH" ]]; then
  RESTART_PROFILE_PATH="$PROFILE_PATH"
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
AUTOPILOT_PY="$SCRIPT_DIR/autopilot.py"

remove_transient_autopilot_artifacts() {
  for path in "$SCRIPT_DIR/__pycache__" "$SCRIPT_DIR/runtime/__pycache__"; do
    if [[ -e "$path" ]]; then
      rm -rf "$path"
      echo "[cutover] removed transient artifact: $path"
    fi
  done
}

ARGS=(
  "$AUTOPILOT_PY"
  restart-after-next-commit
  --profile "$PROFILE"
  --config-path "$CONFIG_PATH"
  --state-path "$STATE_PATH"
  --restart-profile "$RESTART_PROFILE"
  --restart-config-path "$RESTART_CONFIG_PATH"
  --restart-state-path "$RESTART_STATE_PATH"
  --restart-output-path "$RESTART_OUTPUT_PATH"
  --restart-pid-path "$RESTART_PID_PATH"
  --refresh-seconds "$REFRESH_SECONDS"
  --stop-timeout-seconds "$STOP_TIMEOUT_SECONDS"
)

if [[ -n "$PROFILE_PATH" ]]; then
  ARGS+=(--profile-path "$PROFILE_PATH")
fi

if [[ -n "$RESTART_PROFILE_PATH" ]]; then
  ARGS+=(--restart-profile-path "$RESTART_PROFILE_PATH")
fi

if [[ -n "$RESTART_SYNC_REF" ]]; then
  ARGS+=(--restart-sync-ref "$RESTART_SYNC_REF")
fi

if [[ "$HARD_RESET" == "1" ]]; then
  ARGS+=(--hard-reset)
fi

if [[ "$STOP_IF_STATUS_CHANGES" == "1" ]]; then
  ARGS+=(--stop-if-status-changes)
fi

echo "[cutover] state path: $STATE_PATH"
echo "[cutover] current profile/config: $PROFILE / $CONFIG_PATH"
echo "[cutover] restart profile/config/state: $RESTART_PROFILE / $RESTART_CONFIG_PATH / $RESTART_STATE_PATH"
if [[ -n "$RESTART_SYNC_REF" ]]; then
  echo "[cutover] restart sync ref: $RESTART_SYNC_REF"
fi
echo "[cutover] restart output path: $RESTART_OUTPUT_PATH"
echo "[cutover] restart pid path: $RESTART_PID_PATH"

remove_transient_autopilot_artifacts

exec "${PYTHON_CMD[@]}" "${ARGS[@]}"
