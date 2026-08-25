# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 Racoon Typper Contributors
param(
  [Parameter(Mandatory = $true)]
  [string]$Installer,
  [string]$TauriDriverExe
)

$ErrorActionPreference = 'Stop'
if (-not (Test-Path -LiteralPath $Installer -PathType Leaf)) {
  throw "NSIS installer was not found: $Installer"
}
if (-not $TauriDriverExe -or -not (Test-Path -LiteralPath $TauriDriverExe -PathType Leaf)) {
  throw "Pinned tauri-driver.exe was not found: '$TauriDriverExe' (pass the build-windows artifact)"
}

$appData = $env:APPDATA
$appDataDir = Join-Path $appData 'com.racoon.typper'
$database = Join-Path $appDataDir 'data.db'

$workspace = Join-Path ([System.IO.Path]::GetTempPath()) ("racoon-typper-smoke-" + [guid]::NewGuid())
$installDirectory = Join-Path $workspace 'install'
New-Item -ItemType Directory -Force -Path $installDirectory, $appDataDir | Out-Null

# A clean profile opens the first-run onboarding gate instead of auto-starting
# a practice session; this smoke exercises the post-onboarding journey.
Set-Content -LiteralPath (Join-Path $appDataDir 'settings.toml') `
  -Value "onboarding_completed = true`n" -Encoding Ascii

function Get-LaunchDiagnostics {
  $logPath = Join-Path $workspace 'app.log'
  $out = if (Test-Path -LiteralPath $logPath) { Get-Content -LiteralPath $logPath -Raw } else { '' }
  $err = if (Test-Path -LiteralPath "$logPath.err") { Get-Content -LiteralPath "$logPath.err" -Raw } else { '' }
  return "stdout=[$out] stderr=[$err]"
}

function Invoke-SqlScalar([string]$Query) {
  & $sqlite $database $Query | Select-Object -First 1
}

function Wait-TestCountAtLeast([int]$Minimum, [int]$TimeoutSeconds) {
  # Hard gate: a completed practice session must persist a row in `tests`.
  $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
  while ((Get-Date) -lt $deadline) {
    if (Test-Path -LiteralPath $database -PathType Leaf) {
      $raw = Invoke-SqlScalar "SELECT COUNT(*) FROM tests;"
      $count = 0
      if ($null -ne $raw -and [int]::TryParse([string]$raw, [ref]$count) -and $count -ge $Minimum) {
        return $count
      }
    }
    Start-Sleep -Seconds 3
  }
  foreach ($table in @('session_ledger', 'session_completion_intents', 'session_finalizations', 'tests')) {
    Write-Host "stage $table = [$(Invoke-SqlScalar "SELECT COUNT(*) FROM $table;")]"
  }
  throw "Persisted test rows did not reach >= $Minimum within ${TimeoutSeconds}s. $(Get-LaunchDiagnostics)"
}

function Provision-MsEdgeDriver {
  # tauri-driver delegates to msedgedriver on Windows; the driver major must
  # match the installed WebView2 Runtime major or sessions hang on connect.
  $key = 'HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}'
  $runtime = (Get-ItemProperty -LiteralPath $key -ErrorAction Stop).pv
  if (-not $runtime) { throw 'WebView2 Evergreen runtime version not found in registry.' }
  $major = ($runtime -split '\.')[0]

  $driverDir = Join-Path $workspace 'msedgedriver'
  New-Item -ItemType Directory -Force -Path $driverDir | Out-Null
  # Microsoft serves a driver build for every shipped WebView2 runtime version;
  # requesting the exact runtime version eliminates major/minor skew entirely.
  Write-Host "WebView2 runtime $runtime -> msedgedriver $runtime (exact match)"
  $zip = Join-Path $driverDir 'edgedriver_win64.zip'
  Invoke-WebRequest -Uri "https://msedgedriver.microsoft.com/$runtime/edgedriver_win64.zip" -OutFile $zip
  Expand-Archive -LiteralPath $zip -DestinationPath $driverDir -Force
  $exe = Join-Path $driverDir 'msedgedriver.exe'
  if (-not (Test-Path -LiteralPath $exe -PathType Leaf)) { throw 'msedgedriver.exe missing after extraction.' }
  $env:PATH = "$driverDir;$env:PATH"
  Write-Host "msedgedriver provisioned at $exe"
}

try {
  # Evidence tooling only (read-only SELECT); never shipped with the app.
  if (-not (Get-Command sqlite3 -ErrorAction SilentlyContinue)) {
    choco install sqlite -y --no-progress | Out-Null
  }
  $sqliteCommand = Get-Command sqlite3 -ErrorAction SilentlyContinue
  if (-not $sqliteCommand) {
    throw 'sqlite3 CLI is unavailable even after chocolatey install; cannot verify saved tests.'
  }
  $sqlite = $sqliteCommand.Source

  $install = Start-Process -FilePath $Installer -ArgumentList @('/S', "/D=$installDirectory") -Wait -PassThru
  if ($install.ExitCode -ne 0) { throw "NSIS installer exited with $($install.ExitCode)" }

  $executable = Join-Path $installDirectory 'racoon-app.exe'
  if (-not (Test-Path -LiteralPath $executable -PathType Leaf)) {
    throw "Installed executable was not found: $executable"
  }

  Provision-MsEdgeDriver

  # tauri-driver serves WebDriver on :4444 and spawns the application itself
  # through the `tauri:options.application` capability supplied by the client.
  $driverLog = Join-Path $workspace 'tauri-driver.log'
  $tauriDriver = Start-Process -FilePath $TauriDriverExe -PassThru `
    -RedirectStandardOutput $driverLog -RedirectStandardError "$driverLog.err"
  Start-Sleep -Seconds 2
  if ($tauriDriver.HasExited) { throw 'tauri-driver exited immediately after start.' }

  try {
    $env:APP_PATH = $executable
    Push-Location $PSScriptRoot\..\frontend
    node ..\scripts\windows-ui-smoke.mjs
    $uiExit = $LASTEXITCODE
    Pop-Location
    if ($uiExit -ne 0) { throw "WebDriver UI smoke failed with exit code $uiExit." }
  } finally {
    if ((Get-Location).ProviderPath -like '*frontend') { Pop-Location }
  }

  # Hard gate restored: typed input reached the WebView, the time test expired,
  # and the durable finalization committed exactly one persisted row.
  $savedRows = Wait-TestCountAtLeast 1 90
  Write-Host "Typed practice session persisted: tests rows = $savedRows"

  # Restart proves the persisted state survives a clean process lifecycle.
  $relaunch = Start-Process -FilePath $executable -PassThru `
    -RedirectStandardOutput (Join-Path $workspace 'relaunch.log') `
    -RedirectStandardError (Join-Path $workspace 'relaunch.log.err')
  Start-Sleep -Seconds 10
  if ($relaunch.HasExited) {
    throw "Application exited during restart with $($relaunch.ExitCode). $(Get-LaunchDiagnostics)"
  }
  Stop-Process -Id $relaunch.Id -ErrorAction SilentlyContinue
  $relaunch.WaitForExit()

  Write-Host 'Windows NSIS smoke passed: installed, WebDriver-typed session persisted to SQLite, restart retained data.'
} finally {
  if ($tauriDriver -and -not $tauriDriver.HasExited) {
    Stop-Process -Id $tauriDriver.Id -ErrorAction SilentlyContinue
  }
  Remove-Item -LiteralPath $workspace -Recurse -Force -ErrorAction SilentlyContinue
  Remove-Item -LiteralPath $appDataDir -Recurse -Force -ErrorAction SilentlyContinue
}
