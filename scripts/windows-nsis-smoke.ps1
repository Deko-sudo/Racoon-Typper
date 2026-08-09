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
$appData = Join-Path $workspace 'appdata'
New-Item -ItemType Directory -Force -Path $installDirectory, $appData | Out-Null

try {
  $install = Start-Process -FilePath $Installer -ArgumentList @('/S', "/D=$installDirectory") -Wait -PassThru
  if ($install.ExitCode -ne 0) { throw "NSIS installer exited with $($install.ExitCode)" }

  $executable = Join-Path $installDirectory 'racoon-app.exe'
  if (-not (Test-Path -LiteralPath $executable -PathType Leaf)) {
    throw "Installed executable was not found: $executable"
  }

  function Start-And-Stop-App {
    $previousAppData = $env:APPDATA
    $env:APPDATA = $appData
    try {
      $process = Start-Process -FilePath $executable -PassThru
      Start-Sleep -Seconds 8
      if ($process.HasExited) { throw "Application exited during startup with $($process.ExitCode)" }
      Stop-Process -Id $process.Id -ErrorAction Stop
      $process.WaitForExit()
    } finally {
      $env:APPDATA = $previousAppData
    }
  }

  Start-And-Stop-App
  $database = Join-Path $appData 'com.racoon.typper\data.db'
  if (-not (Test-Path -LiteralPath $database -PathType Leaf)) {
    throw "First launch did not create the expected application database: $database"
  }
  Start-And-Stop-App
  Write-Output 'Windows NSIS smoke passed: installed, launched twice, and retained application data.'
} finally {
  Remove-Item -LiteralPath $workspace -Recurse -Force -ErrorAction SilentlyContinue
}
