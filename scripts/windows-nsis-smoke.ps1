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

try {
  $install = Start-Process -FilePath $Installer -ArgumentList @('/S', "/D=$installDirectory") -Wait -PassThru
  if ($install.ExitCode -ne 0) { throw "NSIS installer exited with $($install.ExitCode)" }

  $executable = Join-Path $installDirectory 'racoon-app.exe'
  if (-not (Test-Path -LiteralPath $executable -PathType Leaf)) {
    throw "Installed executable was not found: $executable"
  }

  function Start-And-Stop-App {
    $logPath = Join-Path $workspace 'app.log'
    $process = Start-Process -FilePath $executable -PassThru -RedirectStandardOutput $logPath -RedirectStandardError "$logPath.err"
    # First launch on a clean runner initializes WebView2, which can take well
    # over the previous 8s window before the app's setup() creates data.db.
    Start-Sleep -Seconds 30
    if ($process.HasExited) {
      $out = if (Test-Path -LiteralPath $logPath) { Get-Content -LiteralPath $logPath -Raw } else { '' }
      $err = if (Test-Path -LiteralPath "$logPath.err") { Get-Content -LiteralPath "$logPath.err" -Raw } else { '' }
      throw "Application exited during startup with $($process.ExitCode). stdout=[$out] stderr=[$err]"
    }
    Stop-Process -Id $process.Id -ErrorAction Stop
    $process.WaitForExit()
  }

  Start-And-Stop-App
  if (-not (Test-Path -LiteralPath $database -PathType Leaf)) {
    $out = if (Test-Path -LiteralPath (Join-Path $workspace 'app.log')) { Get-Content -LiteralPath (Join-Path $workspace 'app.log') -Raw } else { '' }
    $err = if (Test-Path -LiteralPath (Join-Path $workspace 'app.log.err')) { Get-Content -LiteralPath (Join-Path $workspace 'app.log.err') -Raw } else { '' }
    # Enumerate every data dir that was actually created so the failure shows
    # where the app really wrote its state (Roaming vs Local, and subpaths).
    $roaming = Join-Path $appData 'com.racoon.typper'
    $localAppData = Join-Path $env:LOCALAPPDATA 'com.racoon.typper'
    $existing = @()
    foreach ($candidate in @($roaming, $localAppData)) {
      if (Test-Path -LiteralPath $candidate) {
        $files = Get-ChildItem -LiteralPath $candidate -Recurse -File -ErrorAction SilentlyContinue | ForEach-Object { $_.FullName }
        $existing += "DIR=$candidate FILES=[$($files -join ';')]"
      }
    }
    throw "First launch did not create the expected application database: $database stdout=[$out] stderr=[$err] found=[$($existing -join ' | ')]"
  }
  Start-And-Stop-App
  Write-Output 'Windows NSIS smoke passed: installed, launched twice, and retained application data.'
} finally {
  Remove-Item -LiteralPath $workspace -Recurse -Force -ErrorAction SilentlyContinue
  Remove-Item -LiteralPath $appDataDir -Recurse -Force -ErrorAction SilentlyContinue
}
