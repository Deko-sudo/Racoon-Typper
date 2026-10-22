# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 Racoon Typper Contributors
param(
  [Parameter(Mandatory = $true)] [string]$Installer,
  [string]$Evidence = 'candidate-evidence/windows-runtime',
  [switch]$UiProbe
)
$ErrorActionPreference = 'Stop'
if ($env:GITHUB_ACTIONS -ne 'true' -or $env:RUNNER_ENVIRONMENT -ne 'github-hosted') {
  throw 'Use only on a fresh GitHub-hosted runner, never a personal desktop.'
}
if (-not (Test-Path -LiteralPath $Installer -PathType Leaf)) { throw 'NSIS installer not found' }
# Fail before installation/settings writes if any real application profile exists.
$profile = Join-Path ([Environment]::GetFolderPath('ApplicationData')) 'com.racoon.typper'
if (Test-Path -LiteralPath $profile) { throw 'Existing profile: refusing to modify it.' }
$workspace = Join-Path ([IO.Path]::GetTempPath()) ('racoon-nsis-' + [guid]::NewGuid())
$installDirectory = Join-Path $workspace 'install'
New-Item -ItemType Directory -Path $workspace | Out-Null
try {
  $install = Start-Process -FilePath $Installer -ArgumentList @('/S', "/D=$installDirectory") -Wait -PassThru
  if ($install.ExitCode -ne 0) { throw "NSIS installer exited with $($install.ExitCode)" }
  $exe = Join-Path $installDirectory 'racoon-app.exe'
  if (-not (Test-Path -LiteralPath $exe -PathType Leaf)) { throw 'Installed executable not found' }
  $probeArguments = @()
  if ($UiProbe) { $probeArguments = @('--ui-probe', 'scripts/windows-ui-probe.ps1') }
  python scripts/desktop-runtime-smoke.py --executable $exe --evidence $Evidence @probeArguments
  if ($LASTEXITCODE -ne 0) { throw 'Windows window/database restart smoke failed; see evidence logs.' }
} finally {
  Remove-Item -LiteralPath $workspace -Recurse -Force -ErrorAction SilentlyContinue
}
