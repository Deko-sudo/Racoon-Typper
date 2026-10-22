# SPDX-License-Identifier: Apache-2.0
# Copyright 2026 Racoon Typper Contributors
# CI investigation only: UIA controls + OS-injected Unicode, not a hardware/DPI test.
param([Parameter(Mandatory=$true)][int]$AppProcessId,
      [Parameter(Mandatory=$true)][string]$Evidence)
$ErrorActionPreference = 'Stop'
if ($env:GITHUB_ACTIONS -ne 'true' -or $env:RUNNER_ENVIRONMENT -ne 'github-hosted') {
  throw 'Use only on a disposable GitHub-hosted runner, never a personal desktop.'
}
Add-Type -AssemblyName UIAutomationClient, UIAutomationTypes, System.Windows.Forms, System.Drawing
Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class NativeProbe {
  [StructLayout(LayoutKind.Sequential)] public struct KEY {
    public ushort vk, scan; public uint flags, time; public UIntPtr extra;
  }
  [StructLayout(LayoutKind.Explicit, Size=40)] public struct INPUT {
    [FieldOffset(0)] public uint type; [FieldOffset(8)] public KEY key;
  }
  [DllImport("user32.dll")] public static extern uint SendInput(uint count, INPUT[] inputs, int size);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr handle);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  public static void Type(char c) {
    INPUT down = new INPUT(); down.type=1; down.key.scan=c; down.key.flags=4;
    INPUT up=down; up.key.flags=6;
    if (SendInput(2, new INPUT[]{down,up},40)!=2) throw new Exception("SendInput was blocked");
  }
}
'@
$process = Get-Process -Id $AppProcessId
$handle = $process.MainWindowHandle
if ($handle -eq [IntPtr]::Zero) { throw 'No main window handle' }
$root = [System.Windows.Automation.AutomationElement]::FromHandle($handle)
$scope = [System.Windows.Automation.TreeScope]::Descendants
function Dump-Tree {
  $root.FindAll($scope,[System.Windows.Automation.Condition]::TrueCondition) | ForEach-Object {
    "$( $_.Current.ControlType.ProgrammaticName ) | $( $_.Current.Name )"
  } | Set-Content -Encoding UTF8 (Join-Path $Evidence 'uia-tree.txt')
}
function Find-Name([string]$name) {
  $condition = [System.Windows.Automation.PropertyCondition]::new([System.Windows.Automation.AutomationElement]::NameProperty,$name)
  $root.FindFirst($scope,$condition)
}
function Wait-Name([string]$name) {
  $deadline = [DateTime]::UtcNow.AddSeconds(20)
  do {
    $element = Find-Name $name
    if ($null -ne $element -and $element.Current.IsEnabled -and -not $element.Current.IsOffscreen) { return $element }
    Start-Sleep -Milliseconds 200
  } while ([DateTime]::UtcNow -lt $deadline)
  throw "UIA control unavailable: $name"
}
function Invoke-Name([string]$name) {
  $element = Wait-Name $name
  $pattern = $element.GetCurrentPattern([System.Windows.Automation.InvokePattern]::Pattern)
  $pattern.Invoke()
}
function Type-Test {
  $inputElement = Wait-Name 'Type the displayed text'
  [void][NativeProbe]::SetForegroundWindow($handle)
  $inputElement.SetFocus()
  if ([NativeProbe]::GetForegroundWindow() -ne $handle) { throw 'Runner cannot focus the application' }
  foreach ($character in 'asdf'.ToCharArray()) {
    [NativeProbe]::Type($character)
    Start-Sleep -Milliseconds 200
  }
  [void](Wait-Name 'Test Complete')
}
try {
  Invoke-Name 'Texts'
  Invoke-Name 'Start'
  Type-Test
  Invoke-Name 'Restart'
  Type-Test
  $gallery = Join-Path ([Environment]::GetFolderPath('MyPictures')) 'Raccoon Typer'
  $before = @(Get-ChildItem -LiteralPath $gallery -Filter '*.png' -ErrorAction SilentlyContinue | Select-Object -ExpandProperty FullName)
  [System.Windows.Forms.Clipboard]::Clear()
  Invoke-Name 'Save PNG'
  $deadline = [DateTime]::UtcNow.AddSeconds(30)
  do {
    $new = @(Get-ChildItem -LiteralPath $gallery -Filter '*.png' -ErrorAction SilentlyContinue | Where-Object { $_.FullName -notin $before })
    if ($new.Count -eq 1 -and [System.Windows.Forms.Clipboard]::ContainsImage()) { break }
    Start-Sleep -Milliseconds 200
  } while ([DateTime]::UtcNow -lt $deadline)
  if ($new.Count -ne 1 -or -not [System.Windows.Forms.Clipboard]::ContainsImage()) { throw 'PNG file or OS image clipboard missing' }
  $image = [System.Windows.Forms.Clipboard]::GetImage()
  try {
    if ($image.Width -lt 400 -or $image.Height -lt 200) { throw 'Unexpected clipboard image dimensions' }
    $image.Save((Join-Path $Evidence 'clipboard.png'),[System.Drawing.Imaging.ImageFormat]::Png)
    $dimensions = @($image.Width,$image.Height)
  } finally { $image.Dispose() }
  Copy-Item -LiteralPath $new[0].FullName -Destination (Join-Path $Evidence 'saved.png')
  Invoke-Name 'Restart'
  [void](Wait-Name 'Type the displayed text')
  Invoke-Name 'Abort'
  [void](Wait-Name 'Start')
  @{status='passed';input='OS SendInput Unicode asdf, twice';abort='returned to custom text list';
    png_file=$new[0].Name;clipboard_image_dimensions=$dimensions;hardware_layout_dpi_paste_verified=$false} |
    ConvertTo-Json | Set-Content -Encoding UTF8 (Join-Path $Evidence 'native-ui.json')
} finally {
  Dump-Tree
}
