[CmdletBinding()]
param(
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$Args
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

$pythonCommand = Resolve-PythonCommand
$scriptPath = Join-Path $PSScriptRoot "autopilot.py"
if ($pythonCommand.Length -eq 1) {
    & $pythonCommand[0] $scriptPath watch @Args
}
else {
    & $pythonCommand[0] $pythonCommand[1] $scriptPath watch @Args
}
exit $LASTEXITCODE
