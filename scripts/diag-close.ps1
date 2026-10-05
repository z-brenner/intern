[CmdletBinding()]
param([Parameter(Mandatory = $true)][string]$InstallerPath)
# Diagnostic only (scratch branch): why does the installed app not exit after CloseMainWindow?
$ErrorActionPreference = "Continue"
Add-Type @"
using System; using System.Text; using System.Runtime.InteropServices; using System.Collections.Generic;
public static class W {
  public delegate bool EnumProc(IntPtr h, IntPtr l);
  [DllImport("user32.dll")] public static extern bool EnumWindows(EnumProc f, IntPtr l);
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] public static extern bool IsWindowEnabled(IntPtr h);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetClassName(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] public static extern IntPtr GetWindow(IntPtr h, uint cmd);
  public static List<string> List(uint target) {
    var r = new List<string>();
    EnumWindows((h, l) => { uint pid; GetWindowThreadProcessId(h, out pid); if (pid == target) {
      var t = new StringBuilder(256); GetWindowText(h, t, 256); var c = new StringBuilder(256); GetClassName(h, c, 256);
      r.Add(String.Format("hwnd={0} class={1} title='{2}' visible={3} enabled={4} owner={5}", h, c, t, IsWindowVisible(h), IsWindowEnabled(h), GetWindow(h, 4))); }
      return true; }, IntPtr.Zero);
    return r;
  }
}
"@
$Data = Join-Path $env:LOCALAPPDATA "com.intern.app"
$Install = Start-Process -FilePath $InstallerPath -ArgumentList "/S" -Wait -PassThru
"installer exit $($Install.ExitCode)"
$App = Join-Path (Join-Path $env:LOCALAPPDATA "Intern") "Intern.exe"
"processes after install: $((Get-Process -Name intern -ErrorAction SilentlyContinue | % { "$($_.Id)" }) -join ',')"
function Dump($label, $p) {
  "--- $label (t=$([int]$sw.Elapsed.TotalMilliseconds)ms) HasExited=$($p.HasExited)"
  if (-not $p.HasExited) { $p.Refresh(); "Responding=$($p.Responding) MainWindowHandle=$($p.MainWindowHandle) Title='$($p.MainWindowTitle)'"; [W]::List([uint32]$p.Id) | % { "  $_" } }
  Get-ChildItem -LiteralPath $Data -Recurse -File -ErrorAction SilentlyContinue | % { "  file $($_.FullName.Substring($Data.Length)) $($_.Length)" }
  Get-ChildItem -LiteralPath (Join-Path $Data "logs") -File -ErrorAction SilentlyContinue | % { "  ==== $($_.Name)"; Get-Content -LiteralPath $_.FullName -Tail 40 | % { "    $_" } }
  "  intern processes: $((Get-Process -Name intern,msedgewebview2,intern-worker,llama-server -ErrorAction SilentlyContinue | % { "$($_.Name)#$($_.Id)" }) -join ', ')"
}
function Experiment($name, $delayMs) {
  "===================== $name (close delay $delayMs ms)"
  $script:sw = [Diagnostics.Stopwatch]::StartNew()
  $p = Start-Process -FilePath $App -PassThru
  for ($i = 0; $i -lt 120; $i++) { Start-Sleep -Milliseconds 250; $p.Refresh(); if ($p.HasExited -or $p.MainWindowHandle -ne 0) { break } }
  Dump "window ready" $p
  Start-Sleep -Milliseconds $delayMs
  Dump "before close" $p
  "CloseMainWindow -> $($p.CloseMainWindow())"
  if ($p.WaitForExit(20000)) { "EXITED code=$($p.ExitCode) t=$([int]$sw.Elapsed.TotalMilliseconds)ms"; Dump "after exit" $p; return }
  Dump "20s after close" $p
  "second CloseMainWindow -> $($p.CloseMainWindow())"
  if ($p.WaitForExit(20000)) { "EXITED after second close code=$($p.ExitCode)"; return }
  Dump "20s after second close" $p
  Get-Process -Name intern,msedgewebview2,intern-worker,llama-server -ErrorAction SilentlyContinue | Stop-Process -Force
  Start-Sleep -Seconds 2
}
Experiment "immediate" 0
Remove-Item -LiteralPath (Join-Path $Data "logs") -Recurse -Force -ErrorAction SilentlyContinue
Experiment "settled" 20000
Experiment "immediate-again" 0
