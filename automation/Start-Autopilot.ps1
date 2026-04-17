[CmdletBinding()]
param(
    [switch]$Background,
    [string]$PidPath = "",
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$Args
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Resolve-PythonCommand {
    if (Get-Command py -ErrorAction SilentlyContinue) {
        return ,@("py", "-3")
    }
    if (Get-Command python -ErrorAction SilentlyContinue) {
        return ,@("python")
    }
    throw "Neither 'py' nor 'python' was found in PATH."
}

function Resolve-BackgroundHost {
    if (Get-Command pwsh -ErrorAction SilentlyContinue) {
        return (Get-Command pwsh -ErrorAction Stop).Source
    }

    $windowsPowerShell = Join-Path $env:SystemRoot "System32\WindowsPowerShell\v1.0\powershell.exe"
    if (Test-Path -LiteralPath $windowsPowerShell) {
        return $windowsPowerShell
    }

    throw "Background mode requires 'pwsh' or Windows PowerShell."
}

function Quote-PowerShellLiteral {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Value
    )

    return "'" + $Value.Replace("'", "''") + "'"
}

$pythonCommand = Resolve-PythonCommand
$scriptPath = Join-Path $PSScriptRoot "autopilot.py"

if ($Background) {
    $backgroundHost = Resolve-BackgroundHost
    $workingDirectory = (Get-Location).Path
    $pythonExecutable = (Get-Command $pythonCommand[0] -ErrorAction Stop).Source
    $pythonArguments = New-Object System.Collections.Generic.List[string]
    if ($pythonCommand.Length -gt 1) {
        for ($index = 1; $index -lt $pythonCommand.Length; $index++) {
            $null = $pythonArguments.Add($pythonCommand[$index])
        }
    }
    $null = $pythonArguments.Add($scriptPath)
    $null = $pythonArguments.Add("start")
    foreach ($arg in $Args) {
        $null = $pythonArguments.Add($arg)
    }

    if ($PidPath) {
        $pidAbsolutePath = [System.IO.Path]::GetFullPath((Join-Path $workingDirectory $PidPath))
        $sessionBasePath = [System.IO.Path]::ChangeExtension($pidAbsolutePath, $null).TrimEnd('.')
    }
    else {
        $sessionBasePath = Join-Path $workingDirectory "automation\runtime\autopilot-session"
    }
    $stdoutPath = $sessionBasePath + ".out"
    $stderrPath = $sessionBasePath + ".err"
    $wrapperPath = $sessionBasePath + ".launcher.ps1"

    $pythonArgsJson = ConvertTo-Json ([string[]]$pythonArguments.ToArray()) -Compress
    $pythonArgsBase64 = [Convert]::ToBase64String([System.Text.Encoding]::UTF8.GetBytes($pythonArgsJson))
    $wrapperScript = @"
`$ErrorActionPreference = 'Stop'
Set-Location -LiteralPath $(Quote-PowerShellLiteral $workingDirectory)
`$stdoutPath = $(Quote-PowerShellLiteral $stdoutPath)
`$stderrPath = $(Quote-PowerShellLiteral $stderrPath)
`$pythonExecutable = $(Quote-PowerShellLiteral $pythonExecutable)
`$pythonArgsJson = [System.Text.Encoding]::UTF8.GetString([System.Convert]::FromBase64String($(Quote-PowerShellLiteral $pythonArgsBase64)))
`$pythonArgs = @()
foreach (`$item in (ConvertFrom-Json -InputObject `$pythonArgsJson)) {
    `$pythonArgs += [string]`$item
}
New-Item -ItemType Directory -Force -Path ([System.IO.Path]::GetDirectoryName(`$stdoutPath)) | Out-Null
& `$pythonExecutable @pythonArgs 1>> `$stdoutPath 2>> `$stderrPath
exit `$LASTEXITCODE
"@

    Set-Content -LiteralPath $wrapperPath -Value $wrapperScript -Encoding UTF8

    $process = Start-Process `
        -FilePath $backgroundHost `
        -ArgumentList @("-NoLogo", "-NoProfile", "-File", $wrapperPath) `
        -WorkingDirectory $workingDirectory `
        -WindowStyle Hidden `
        -PassThru

    if ($PidPath) {
        Set-Content -LiteralPath $PidPath -Value $process.Id
    }

    Write-Output ("Started background autopilot. PID={0}" -f $process.Id)
    if ($PidPath) {
        Write-Output ("PID file: {0}" -f $PidPath)
    }

    exit 0
}

if ($pythonCommand.Length -eq 1) {
    & $pythonCommand[0] $scriptPath start @Args
}
else {
    & $pythonCommand[0] $pythonCommand[1] $scriptPath start @Args
}
exit $LASTEXITCODE
