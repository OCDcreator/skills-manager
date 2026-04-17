[CmdletBinding()]
param(
    [string]$WorktreePath = "..\\repo-autopilot",
    [string]$Branch = "autopilot/main"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

$repoRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot ".."))
$resolvedWorktreePath = if ([System.IO.Path]::IsPathRooted($WorktreePath)) {
    [System.IO.Path]::GetFullPath($WorktreePath)
}
else {
    [System.IO.Path]::GetFullPath((Join-Path $repoRoot $WorktreePath))
}

if (Test-Path $resolvedWorktreePath) {
    throw "Worktree path already exists: $resolvedWorktreePath"
}

& git -C $repoRoot show-ref --verify --quiet "refs/heads/$Branch"
$branchExists = ($LASTEXITCODE -eq 0)

if ($branchExists) {
    & git -C $repoRoot worktree add $resolvedWorktreePath $Branch
}
else {
    & git -C $repoRoot worktree add -b $Branch $resolvedWorktreePath HEAD
}

if ($LASTEXITCODE -ne 0) {
    throw "Failed to create worktree at $resolvedWorktreePath"
}

Write-Host "Created worktree at $resolvedWorktreePath on branch $Branch"
Write-Host "Next: cd `"$resolvedWorktreePath`" and run .\\automation\\Start-Autopilot.ps1"
