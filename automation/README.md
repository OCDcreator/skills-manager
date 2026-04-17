# Repository Autopilot

This folder contains a repo-local unattended Codex autopilot scaffold.

## Files

- `automation/autopilot.py`: cross-platform outer controller
- `automation/Arm-AutopilotCutover.ps1`: Windows post-commit cutover wrapper
- `automation/arm-autopilot-cutover.sh`: macOS post-commit cutover wrapper
- `automation/start-autopilot.sh`: macOS start wrapper with optional background mode
- `automation/watch-autopilot.sh`: macOS watch wrapper
- `automation/launchd/com.example.codex-autopilot.plist`: launchd example for repo-local background launches
- `automation/autopilot-config.json`: repo-specific objective, queue, validation, and runner settings
- `automation/profiles/windows.json`: machine-neutral Windows defaults
- `automation/profiles/mac.json`: machine-neutral macOS defaults
- `automation/round-prompt.md`: per-round runner prompt template
- `automation/round-result.schema.json`: structured final-response contract
- `automation/runtime/`: ignored runtime state, logs, prompts, and round results
- `docs/status/autopilot-master-plan.md`: strategy and lane priorities
- `docs/status/autopilot-lane-map.md`: quick entrypoints for the current queue
- `docs/status/autopilot-round-roadmap.md`: queued `[NEXT]` / `[QUEUED]` work items

## Core guarantees

- Every round uses `codex exec` in a new non-interactive session
- Loop control lives in the Python controller, not in a recursive prompt
- Runtime state is machine-readable JSON
- Failed rounds hard-reset the worktree to the round's starting `HEAD`
- Successful rounds must write a phase doc and create a commit
- A runtime lock prevents two machines from driving the same branch simultaneously

## Main commands

Before starting unattended rounds, commit the scaffolded autopilot files so the worktree is clean.

### Windows

```powershell
python .\automation\autopilot.py doctor --profile windows
python .\automation\autopilot.py start --profile windows
```

For a true no-window unattended launch on Windows, prefer the wrapper's background mode instead of starting `py.exe` in a new visible console. The wrapper should run through a hidden PowerShell host that launches `py` / `python` without depending on `pythonw` / `pyw` shims:

```powershell
.\automation\Start-Autopilot.ps1 -Background --% --profile windows
```

### macOS

```bash
python3 ./automation/autopilot.py doctor --profile mac
python3 ./automation/autopilot.py start --profile mac
```

For a repo-local macOS wrapper flow, prefer:

```bash
./automation/start-autopilot.sh -- --profile mac
./automation/watch-autopilot.sh --state-path automation/runtime/autopilot-state.json --tail 80
```

### Helpful modes

```text
python automation/autopilot.py status
python automation/autopilot.py watch
python automation/autopilot.py start --profile windows --dry-run --single-round
python automation/autopilot.py start --profile windows --single-round
python automation/autopilot.py restart-after-next-commit --profile windows
./automation/start-autopilot.sh --background -- --profile mac
```

## Deploy policy

Prefer `deploy_policy=targeted` or `deploy_policy=never` for most unattended repos. Deploying after every successful round is usually unnecessary churn.

- Use `deploy_policy=targeted` when only specific files or directories should trigger deploy
- Keep `deploy_required_paths` narrow and repo-relative
- Reserve `deploy_policy=always` for repos where every successful build must publish a runtime artifact

## Watching the right logs

If this repo ever accumulates multiple autopilot runs or old `round-*` directories, bind your operator commands to the exact state file you care about.

### Windows

```powershell
python .\automation\autopilot.py status --state-path automation\runtime\<state-file>.json
python .\automation\autopilot.py watch --runtime-path automation\runtime --state-path automation\runtime\<state-file>.json --tail 80
Get-Content automation\runtime\round-XYZ\progress.log -Wait -Tail 80
```

### macOS

```bash
python3 ./automation/autopilot.py status --state-path automation/runtime/<state-file>.json
python3 ./automation/autopilot.py watch --runtime-path automation/runtime --state-path automation/runtime/<state-file>.json --tail 80
tail -n 80 -F automation/runtime/round-XYZ/progress.log
```

The scaffolded `watch` output shows:

- `round`
- `phase`
- `queue progress`
- `status`
- `failures`
- `phase doc`
- `focus`
- the exact `progress.log` path being followed
- a default long prefix on every streamed detail line, for example `[completion=25% round=006 phase=005 status=active failures=0]`
- Vulture count and delta when `vulture_command` is configured

Use `python automation/autopilot.py watch --prefix-format short` if you prefer the compact form `[25% r006 p005 active f0]`.

When the watched state is `active`, the live progress log is usually `current_round + 1`. When the watched state is terminal, it is usually `current_round`.

For queue-driven backlog presets, a round result of `goal_complete` means the current `[NEXT]` slice was already done. If the roadmap still has another `[NEXT]` or any `[QUEUED]` work, the controller should resume as `active`, advance `next_phase_number`, and continue to the next queued slice instead of stopping the whole lane.

## Sentinel cutovers

Use `restart-after-next-commit` when you want the current unattended run to finish its next successful round, stop cleanly, and relaunch with replacement settings.

For routine operator handoffs, prefer the scaffolded wrappers:

### Windows

```powershell
.\automation\Arm-AutopilotCutover.ps1 `
  -StatePath automation\runtime\<state-file>.json `
  -Profile windows `
  -ConfigPath automation\<config>.json `
  -RestartSyncRef <cutover-ref>
```

### macOS

```bash
./automation/arm-autopilot-cutover.sh \
  --state-path automation/runtime/<state-file>.json \
  --profile mac \
  --config-path automation/<config>.json \
  --restart-sync-ref <cutover-ref>
```

### Config/profile/state cutover

Use this when you only need to swap config, profile, output, or state paths.

#### Windows

```powershell
python .\automation\autopilot.py restart-after-next-commit `
  --profile windows `
  --state-path automation\runtime\<state-file>.json `
  --restart-profile windows `
  --restart-config-path automation\<new-config>.json `
  --restart-state-path automation\runtime\<new-state-file>.json `
  --restart-profile-path C:\Users\you\.config\codex-autopilot\windows.profile.json `
  --restart-output-path automation\runtime\<new-run>.out `
  --restart-pid-path automation\runtime\<new-run>.pid
```

#### macOS

```bash
python3 ./automation/autopilot.py restart-after-next-commit \
  --profile mac \
  --state-path automation/runtime/<state-file>.json \
  --restart-profile mac \
  --restart-config-path automation/<new-config>.json \
  --restart-state-path automation/runtime/<new-state-file>.json \
  --restart-profile-path /Users/you/.config/codex-autopilot/mac.profile.json \
  --restart-output-path automation/runtime/<new-run>.out \
  --restart-pid-path automation/runtime/<new-run>.pid
```

### Code/prompt cutover via git ref

Use this when the replacement run should resume from a prepared commit on another ref, such as a cutover worktree branch.

1. Prepare the replacement commit on a sibling ref.
2. Launch the sentinel against the currently active state line.
3. Pass `--restart-sync-ref <cutover-ref>` so the controller waits for that ref, fast-forwards to it, and relaunches unattended work.

#### Windows

```powershell
python .\automation\autopilot.py restart-after-next-commit `
  --profile windows `
  --state-path automation\runtime\<state-file>.json `
  --restart-sync-ref <cutover-ref> `
  --restart-profile windows `
  --restart-config-path automation\<config>.json `
  --restart-state-path automation\runtime\<state-file>.json
```

#### macOS

```bash
python3 ./automation/autopilot.py restart-after-next-commit \
  --profile mac \
  --state-path automation/runtime/<state-file>.json \
  --restart-sync-ref <cutover-ref> \
  --restart-profile mac \
  --restart-config-path automation/<config>.json \
  --restart-state-path automation/runtime/<state-file>.json
```

Prefer this built-in sentinel flow over ad-hoc shell loops. Only fall back to a custom local script when the cutover must run machine-local actions that cannot be expressed through git refs and restart arguments.

The built-in cutover path and both wrapper scripts remove known transient Python bytecode directories before relaunching:

- `automation/__pycache__/`
- `automation/runtime/__pycache__/`

This prevents generated bytecode from making the replacement `start` fail its clean-worktree guard.

## Profile overrides

Committed profile files stay machine-neutral. Put local absolute paths in an external profile JSON and pass it with `--profile-path`.

Example override fields:

```json
{
  "runner_additional_dirs": [
    "C:\\\\absolute\\\\path\\\\to\\\\extra\\\\workspace"
  ],
  "deploy_verify_path": ""
}
```

Typical usage:

```powershell
python .\automation\autopilot.py start --profile windows --profile-path C:\Users\you\.config\codex-autopilot\windows.profile.json
```

```bash
python3 ./automation/autopilot.py start --profile mac --profile-path /Users/you/.config/codex-autopilot/mac.profile.json
```

## macOS convenience scripts

- `automation/start-autopilot.sh` (foreground by default, `--background` for nohup launches)
- `automation/watch-autopilot.sh`
- `automation/arm-autopilot-cutover.sh`

To turn the scaffold into a LaunchAgent:

1. Copy `automation/launchd/com.example.codex-autopilot.plist` to `~/Library/LaunchAgents/com.<repo>.codex-autopilot.plist`
2. Replace every `/ABSOLUTE/PATH/TO/REPO` placeholder
3. Load it with:

```bash
launchctl bootstrap gui/$(id -u) ~/Library/LaunchAgents/com.<repo>.codex-autopilot.plist
launchctl kickstart -k gui/$(id -u)/com.<repo>.codex-autopilot
```

To stop and unload it later:

```bash
launchctl bootout gui/$(id -u) ~/Library/LaunchAgents/com.<repo>.codex-autopilot.plist
```

## Windows convenience scripts

- `automation/New-AutopilotWorktree.ps1`
- `automation/Start-Autopilot.ps1` (interactive by default, `-Background` for no-window launches)
- `automation/Watch-Autopilot.ps1`

These are thin Windows wrappers around the Python CLI. `autopilot.py` already hides child `cmd.exe` / `pwsh.exe` subprocess windows, and `Start-Autopilot.ps1 -Background` should use a hidden PowerShell host so the top-level launcher also stays windowless even when `pythonw` / `pyw` shims are unreliable.


