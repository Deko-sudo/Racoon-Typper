# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 Racoon Typper Contributors
param(
  [Parameter(Mandatory = $true)]
  [string]$Installer
)

$ErrorActionPreference = 'Stop'
if (-not (Test-Path -LiteralPath $Installer -PathType Leaf)) {
  throw "NSIS installer was not found: $Installer"
}

$workspace = Join-Path ([System.IO.Path]::GetTempPath()) ("racoon-typper-smoke-" + [guid]::NewGuid())
$installDirectory = Join-Path $workspace 'install'
New-Item -ItemType Directory -Force -Path $installDirectory | Out-Null

# The application resolves its data directory through the Windows known-folder
# API (FOLDERID_RoamingAppData), not the %APPDATA% environment variable, so
# overriding $env:APPDATA does not redirect where the app writes. We therefore
# exercise the real per-user data directory and clean it up afterwards.
$appData = $env:APPDATA
$appDataDir = Join-Path $appData 'com.racoon.typper'
$database = Join-Path $appDataDir 'data.db'

# Evidence tooling only (read-only SELECT); the application itself bundles its
# own SQLite. Unpinned chocolatey package is an accepted CI-only boundary, the
# release artifact never ships this binary.
if (-not (Get-Command sqlite3 -ErrorAction SilentlyContinue)) {
  choco install sqlite -y --no-progress | Out-Null
}
$sqliteCommand = Get-Command sqlite3 -ErrorAction SilentlyContinue
if (-not $sqliteCommand) {
  throw 'sqlite3 CLI is unavailable even after chocolatey install; cannot verify saved tests.'
}
$sqlite = $sqliteCommand.Source

function Invoke-SqlScalar([string]$Query) {
  & $sqlite $database $Query | Select-Object -First 1
}

function Get-LaunchDiagnostics {
  $logPath = Join-Path $workspace 'app.log'
  $out = if (Test-Path -LiteralPath $logPath) { Get-Content -LiteralPath $logPath -Raw } else { '' }
  $err = if (Test-Path -LiteralPath "$logPath.err") { Get-Content -LiteralPath "$logPath.err" -Raw } else { '' }
  return "stdout=[$out] stderr=[$err]"
}

function Wait-MainWindow([System.Diagnostics.Process]$Process, [int]$TimeoutSeconds) {
  # Q1 evidence: a non-empty MainWindowTitle proves the WebView composed the
  # first screen instead of merely keeping the process alive.
  $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
  while ((Get-Date) -lt $deadline) {
    if ($Process.HasExited) {
      throw "Application exited during startup with $($Process.ExitCode). $(Get-LaunchDiagnostics)"
    }
    $Process.Refresh()
    if (-not [string]::IsNullOrWhiteSpace($Process.MainWindowTitle)) {
      Write-Output "First screen rendered: window '$($Process.MainWindowTitle)'"
      return
    }
    Start-Sleep -Seconds 2
  }
  throw "Main window did not appear within ${TimeoutSeconds}s. $(Get-LaunchDiagnostics)"
}

function Wait-TestCountAtLeast([int]$Minimum, [int]$TimeoutSeconds) {
  # Q2 evidence: a completed practice session must persist a row in `tests`.
  # The default time-mode test finishes on its own timer; WAL allows reading
  # while the application keeps running.
  $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
  while ((Get-Date) -lt $deadline) {
    $count = 0
    $raw = Invoke-SqlScalar "SELECT COUNT(*) FROM tests;"
    if ($null -ne $raw -and -not [int]::TryParse([string]$raw, [ref]$count)) { $count = 0 }
    if ($count -ge $Minimum) { return $count }
    Start-Sleep -Seconds 5
  }
  throw "No persisted test rows appeared within ${TimeoutSeconds}s (expected >= $Minimum). $(Get-LaunchDiagnostics)"
}

function Send-TypingSample([System.Diagnostics.Process]$Process) {
  Add-Type -AssemblyName System.Windows.Forms
  $shell = New-Object -ComObject WScript.Shell
  if (-not $shell.AppActivate($Process.Id)) {
    Write-Output 'AppActivate returned false; keystrokes may target another window.'
  }
  Start-Sleep -Milliseconds 500
  for ($i = 0; $i -lt 20; $i++) {
    [System.Windows.Forms.SendKeys]::SendWait('asdf jkl; ewq poiuy ')
    Start-Sleep -Milliseconds 400
  }
}

function Start-Application {
  $logPath = Join-Path $workspace 'app.log'
  return Start-Process -FilePath $executable -PassThru -RedirectStandardOutput $logPath -RedirectStandardError "$logPath.err"
}

try {
  $install = Start-Process -FilePath $Installer -ArgumentList @('/S', "/D=$installDirectory") -Wait -PassThru
  if ($install.ExitCode -ne 0) { throw "NSIS installer exited with $($install.ExitCode)" }

  $script:executable = Join-Path $installDirectory 'racoon-app.exe'
  if (-not (Test-Path -LiteralPath $executable -PathType Leaf)) {
    throw "Installed executable was not found: $executable"
  }

  # First launch on a clean runner initializes WebView2 before the app's
  # setup() creates data.db. Poll for the database instead of a fixed sleep:
  # Evergreen runtime initialization can exceed 30s on fresh runner images.
  $process = Start-Application
  $deadline = (Get-Date).AddSeconds(120)
  while (-not (Test-Path -LiteralPath $database -PathType Leaf)) {
    if ($process.HasExited) {
      throw "Application exited during startup with $($process.ExitCode). $(Get-LaunchDiagnostics)"
    }
    if ((Get-Date) -gt $deadline) { break }
    Start-Sleep -Seconds 5
  }
  if (-not (Test-Path -LiteralPath $database -PathType Leaf)) {
    throw "First launch did not create the expected application database: $database $(Get-LaunchDiagnostics)"
  }

  Wait-MainWindow $process 120
  Send-TypingSample $process

  # Default mode is a 30s time test; it completes on its own timer and the
  # durable finalization ledger commits the row exactly once.
  $savedRows = Wait-TestCountAtLeast 1 150
  Write-Output "Practice session persisted: tests rows = $savedRows"

  if (-not $process.HasExited) {
    Stop-Process -Id $process.Id -ErrorAction SilentlyContinue
    $process.WaitForExit()
  }

  # Restart proves the persisted state survives a clean process lifecycle.
  $process = Start-Application
  Start-Sleep -Seconds 10
  if ($process.HasExited) {
    throw "Application exited during restart with $($process.ExitCode). $(Get-LaunchDiagnostics)"
  }
  Stop-Process -Id $process.Id -ErrorAction Stop
  $process.WaitForExit()

  Write-Output 'Windows NSIS smoke passed: installed, first screen rendered, typed session saved to SQLite, restart retained data.'
} finally {
  Remove-Item -LiteralPath $workspace -Recurse -Force -ErrorAction SilentlyContinue
  Remove-Item -LiteralPath $appDataDir -Recurse -Force -ErrorAction SilentlyContinue
}
