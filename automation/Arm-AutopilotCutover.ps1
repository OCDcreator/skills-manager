[CmdletBinding()]
param(
    [string]$StatePath = "automation\\runtime\\autopilot-state.json",
    [string]$Profile = "windows",
    [string]$ConfigPath = "automation\\autopilot-config.json",
    [string]$ProfilePath = "",
    [string]$RestartProfile = "",
    [string]$RestartConfigPath = "",
    [string]$RestartStatePath = "",
    [string]$RestartProfilePath = "",
    [string]$RestartSyncRef = "",
    [string]$RestartOutputPath = "automation\\runtime\\autopilot-cutover.out",
    [string]$RestartPidPath = "automation\\runtime\\autopilot-cutover.pid",
    [double]$RefreshSeconds = 5,
    [int]$StopTimeoutSeconds = 30,
    [switch]$HardReset,
    [switch]$KeepWatchingIfStatusChanges
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Resolve-PythonCommand {
    if (Get-Command py -ErrorAction SilentlyContinue) {
        return @("py", "-3")
    }
    if (Get-Command python -ErrorAction SilentlyContinue) {
        return @("python")
    }
    throw "Neither 'py' nor 'python' was found in PATH."
}

function Remove-TransientAutopilotArtifacts {
    $paths = @(
        (Join-Path $PSScriptRoot "__pycache__"),
        (Join-Path $PSScriptRoot "runtime\\__pycache__")
    )

    foreach ($path in $paths) {
        if (Test-Path -LiteralPath $path) {
            Remove-Item -LiteralPath $path -Recurse -Force
            Write-Host "[cutover] removed transient artifact: $path"
        }
    }
}

$resolvedRestartProfile = if ($RestartProfile) { $RestartProfile } else { $Profile }
$resolvedRestartConfigPath = if ($RestartConfigPath) { $RestartConfigPath } else { $ConfigPath }
$resolvedRestartStatePath = if ($RestartStatePath) { $RestartStatePath } else { $StatePath }
$resolvedRestartProfilePath = if ($RestartProfilePath) { $RestartProfilePath } else { $ProfilePath }

$scriptPath = Join-Path $PSScriptRoot "autopilot.py"
$pythonCommand = Resolve-PythonCommand
$invocationArgs = @(
    $scriptPath,
    "restart-after-next-commit",
    "--profile", $Profile,
    "--config-path", $ConfigPath,
    "--state-path", $StatePath,
    "--restart-profile", $resolvedRestartProfile,
    "--restart-config-path", $resolvedRestartConfigPath,
    "--restart-state-path", $resolvedRestartStatePath,
    "--restart-output-path", $RestartOutputPath,
    "--restart-pid-path", $RestartPidPath,
    "--refresh-seconds", [string]$RefreshSeconds,
    "--stop-timeout-seconds", [string]$StopTimeoutSeconds
)

if ($ProfilePath) {
    $invocationArgs += @("--profile-path", $ProfilePath)
}

if ($resolvedRestartProfilePath) {
    $invocationArgs += @("--restart-profile-path", $resolvedRestartProfilePath)
}

if ($RestartSyncRef) {
    $invocationArgs += @("--restart-sync-ref", $RestartSyncRef)
}

if ($HardReset) {
    $invocationArgs += "--hard-reset"
}

if (-not $KeepWatchingIfStatusChanges) {
    $invocationArgs += "--stop-if-status-changes"
}

Write-Host "[cutover] state path: $StatePath"
Write-Host "[cutover] current profile/config: $Profile / $ConfigPath"
Write-Host "[cutover] restart profile/config/state: $resolvedRestartProfile / $resolvedRestartConfigPath / $resolvedRestartStatePath"
if ($RestartSyncRef) {
    Write-Host "[cutover] restart sync ref: $RestartSyncRef"
}
Write-Host "[cutover] restart output path: $RestartOutputPath"
Write-Host "[cutover] restart pid path: $RestartPidPath"

Remove-TransientAutopilotArtifacts

if ($pythonCommand.Length -eq 1) {
    & $pythonCommand[0] @invocationArgs
}
else {
    & $pythonCommand[0] $pythonCommand[1] @invocationArgs
}

exit $LASTEXITCODE
