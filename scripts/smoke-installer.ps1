[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][string]$InstallerPath,
    [string]$InstallDirectory = (Join-Path $env:LOCALAPPDATA "Intern"),
    [string]$FixtureDirectory = (Join-Path (Resolve-Path (Join-Path $PSScriptRoot "..")).Path "fixtures/generated"),
    [string]$EvidencePath,
    [string]$Commit = $env:GITHUB_SHA,
    [string]$Workflow = $env:GITHUB_WORKFLOW,
    [string]$RunId = $env:GITHUB_RUN_ID,
    [string]$RunAttempt = $env:GITHUB_RUN_ATTEMPT
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$Installer = (Resolve-Path -LiteralPath $InstallerPath).Path
if ([System.IO.Path]::GetExtension($Installer) -ne ".exe") { throw "Installer must be an NSIS executable" }
$InstallerSha256 = (Get-FileHash -LiteralPath $Installer -Algorithm SHA256).Hash.ToLowerInvariant()
if ($EvidencePath -and (@($Commit, $Workflow, $RunId, $RunAttempt) | Where-Object { [string]::IsNullOrWhiteSpace($_) })) {
    throw "Evidence output requires commit, workflow, run id, and run attempt"
}
$UserDataDirectory = Join-Path $env:LOCALAPPDATA "com.intern.app"
$Sentinel = Join-Path $UserDataDirectory "installer-smoke-user-data.txt"
New-Item -ItemType Directory -Path $UserDataDirectory -Force | Out-Null
Set-Content -LiteralPath $Sentinel -Value "must survive uninstall" -Encoding utf8NoBOM

if (Test-Path -LiteralPath $InstallDirectory) { throw "Smoke install target already exists: $InstallDirectory" }
# "Send to > Intern" (src-tauri/windows/hooks.nsh). Checked before install so a
# shortcut some earlier run left behind cannot pass for this installer's.
$SendToShortcut = Join-Path ([Environment]::GetFolderPath([Environment+SpecialFolder]::SendTo)) "Intern.lnk"
if (Test-Path -LiteralPath $SendToShortcut) { throw "A Send to shortcut already exists before install: $SendToShortcut" }
$StartupErrorLog = Join-Path $UserDataDirectory "logs/startup-error.log"
if (Test-Path -LiteralPath $StartupErrorLog) { Remove-Item -LiteralPath $StartupErrorLog -Force }
$AppProcess = $null
$Install = Start-Process -FilePath $Installer -ArgumentList "/S" -Wait -PassThru
if ($Install.ExitCode -ne 0) { throw "NSIS installer exited with $($Install.ExitCode)" }

try {
    $App = Join-Path $InstallDirectory "Intern.exe"
    if (-not (Test-Path -LiteralPath $App -PathType Leaf)) { throw "Installed application is missing: $App" }
    if (-not (Test-Path -LiteralPath $SendToShortcut -PathType Leaf)) { throw "Send to shortcut is missing after install: $SendToShortcut" }
    $SendToTarget = (New-Object -ComObject WScript.Shell).CreateShortcut($SendToShortcut).TargetPath
    if (-not [string]::Equals([IO.Path]::GetFullPath($SendToTarget), [IO.Path]::GetFullPath($App), [StringComparison]::OrdinalIgnoreCase)) {
        throw "Send to shortcut points at $SendToTarget, not $App"
    }

    $ManifestFiles = @(Get-ChildItem -LiteralPath $InstallDirectory -Recurse -File -Filter "runtime-assets.json")
    if ($ManifestFiles.Count -ne 1) { throw "Expected exactly one installed runtime-assets.json, got $($ManifestFiles.Count)" }
    $Manifest = Get-Content -LiteralPath $ManifestFiles[0].FullName -Raw | ConvertFrom-Json
    if ($Manifest.schema_version -ne 1 -or $Manifest.bundled_files.Count -eq 0 -or $Manifest.license_files.Count -eq 0) {
        throw "Installed runtime manifest is incomplete"
    }
    $Seen = @{}
    foreach ($Entry in @($Manifest.bundled_files) + @($Manifest.license_files)) {
        $Relative = [string]$Entry.install_path
        if ([string]::IsNullOrWhiteSpace($Relative) -or $Relative.Contains("\") -or $Relative.Contains(":") -or ($Relative.Split("/") -contains "..")) {
            throw "Unsafe installed manifest path: $Relative"
        }
        if ($Seen.ContainsKey($Relative)) { throw "Duplicate installed manifest path: $Relative" }
        $Seen[$Relative] = $true
        $Packaged = [IO.Path]::GetFullPath((Join-Path $InstallDirectory $Relative))
        $RootPrefix = [IO.Path]::GetFullPath($InstallDirectory).TrimEnd([IO.Path]::DirectorySeparatorChar) + [IO.Path]::DirectorySeparatorChar
        if (-not $Packaged.StartsWith($RootPrefix, [StringComparison]::OrdinalIgnoreCase)) { throw "Installed manifest path escapes root: $Relative" }
        if (-not (Test-Path -LiteralPath $Packaged -PathType Leaf)) { throw "Signed packaged file is missing: $Relative" }
        $File = Get-Item -LiteralPath $Packaged
        if ($File.Length -ne [long]$Entry.size) { throw "Installed size mismatch: $Relative" }
        $Digest = (Get-FileHash -LiteralPath $Packaged -Algorithm SHA256).Hash.ToLowerInvariant()
        if ($Digest -ne [string]$Entry.sha256) { throw "Installed SHA-256 mismatch: $Relative" }
    }
    foreach ($Required in @("intern-worker.exe", "llama-server.exe", "tesseract.exe", "pdfium.dll", "tessdata/eng.traineddata", "tessdata/osd.traineddata", "onnxruntime.dll", "ocr-models/text-detection.onnx", "ocr-models/text-recognition.onnx", "ocr-models/page-orientation.onnx")) {
        if (-not $Seen.ContainsKey($Required)) { throw "Installed manifest omits required runtime: $Required" }
    }
    if (-not (Get-ChildItem -LiteralPath $InstallDirectory -Recurse -File -Filter "THIRD_PARTY_NOTICES.md")) { throw "Third-party notices are missing from the installation" }
    if (-not (Get-ChildItem -LiteralPath (Join-Path $InstallDirectory "licenses") -Recurse -File -Filter "*.txt")) { throw "Complete license texts are missing from the installation" }
    if (Get-ChildItem -LiteralPath $InstallDirectory -Recurse -File -Filter "*.gguf") { throw "Model files must not be bundled in the installer" }

    & (Join-Path $PSScriptRoot "smoke-worker.ps1") -WorkerPath (Join-Path $InstallDirectory "intern-worker.exe") -RuntimeDirectory $InstallDirectory -FixtureDirectory $FixtureDirectory

    # Intern's own window, found by its class. Process.MainWindowHandle is the
    # first visible top-level window the process owns, and the main window is
    # created hidden, so until setup shows it that is the single-instance
    # plugin's message window ("com.intern.app-siw"). CloseMainWindow closed
    # that one: the app rightly kept running, and this step reported a hang.
    Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;
using System.Text;
public static class InternSmokeWindow {
    delegate bool EnumWindowsProc(IntPtr hwnd, IntPtr lParam);
    [DllImport("user32.dll")] static extern bool EnumWindows(EnumWindowsProc callback, IntPtr lParam);
    [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr hwnd, out uint processId);
    [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr hwnd);
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern int GetClassName(IntPtr hwnd, StringBuilder name, int capacity);
    [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr hwnd, uint message, IntPtr wParam, IntPtr lParam);
    public static IntPtr Visible(int processId) {
        IntPtr found = IntPtr.Zero;
        EnumWindows((hwnd, lParam) => {
            uint owner;
            GetWindowThreadProcessId(hwnd, out owner);
            if (owner != (uint)processId || !IsWindowVisible(hwnd)) { return true; }
            StringBuilder name = new StringBuilder(64);
            GetClassName(hwnd, name, name.Capacity);
            if (name.ToString() != "Tauri Window") { return true; }
            found = hwnd;
            return false;
        }, IntPtr.Zero);
        return found;
    }
}
"@
    # The window is shown before initialization when the launch is going to
    # show it at all, so its appearing does not mean setup finished. A failed
    # start shows it too, behind a dialog saying why; the startup error log,
    # checked again once the app has exited, is what tells the two apart.
    $AppProcess = Start-Process -FilePath $App -PassThru
    $AppWindow = [IntPtr]::Zero
    for ($Attempt = 0; $Attempt -lt 60; $Attempt += 1) {
        Start-Sleep -Milliseconds 500
        $AppProcess.Refresh()
        if ($AppProcess.HasExited) { throw "Installed Intern.exe exited before its window became ready" }
        $AppWindow = [InternSmokeWindow]::Visible($AppProcess.Id)
        if ($AppWindow -ne [IntPtr]::Zero) { break }
    }
    if ($AppWindow -eq [IntPtr]::Zero) { throw "Installed Intern.exe did not show its main window" }
    if (Test-Path -LiteralPath $StartupErrorLog) {
        throw "Installed Intern.exe could not start: $(Get-Content -LiteralPath $StartupErrorLog -Raw)"
    }
    # WM_CLOSE, which is what CloseMainWindow posts, to the right window.
    if (-not [InternSmokeWindow]::PostMessage($AppWindow, 0x0010, [IntPtr]::Zero, [IntPtr]::Zero)) {
        throw "Installed Intern.exe rejected a normal window close request"
    }
    # A WebView2 app on a shared CI runner can take well over fifteen seconds to
    # tear its browser process down, and a timeout here reads as "the app hangs on
    # close" when the truth is "the runner was busy". Sixty seconds still fails a
    # genuine hang, and a slow-but-clean shutdown costs only the time it needs.
    if (-not $AppProcess.WaitForExit(60000)) {
        # Say what is still alive. Without this the failure names no process and
        # gives no way to tell a hung app from a hung child.
        $Surviving = @(Get-Process -Name "intern", "intern-worker", "llama-server", "msedgewebview2" -ErrorAction SilentlyContinue |
            ForEach-Object { "$($_.Name)#$($_.Id)" })
        $Children = @(Get-CimInstance Win32_Process -Filter "ParentProcessId = $($AppProcess.Id)" -ErrorAction SilentlyContinue |
            ForEach-Object { "$($_.Name)#$($_.ProcessId)" })
        throw ("Installed Intern.exe did not shut down cleanly within 60s. " +
            "Main process HasExited=$($AppProcess.HasExited). " +
            "Children: $($Children -join ', '). Live Intern processes: $($Surviving -join ', ')")
    }
    if ($AppProcess.ExitCode -ne 0) { throw "Installed Intern.exe exited with $($AppProcess.ExitCode)" }
    if (Test-Path -LiteralPath $StartupErrorLog) {
        throw "Installed Intern.exe could not start: $(Get-Content -LiteralPath $StartupErrorLog -Raw)"
    }

    $Uninstaller = Get-ChildItem -LiteralPath $InstallDirectory -File -Filter "uninstall*.exe" | Select-Object -First 1
    if (-not $Uninstaller) { throw "NSIS uninstaller is missing" }
    $Uninstall = Start-Process -FilePath $Uninstaller.FullName -ArgumentList "/S" -Wait -PassThru
    if ($Uninstall.ExitCode -ne 0) { throw "NSIS uninstaller exited with $($Uninstall.ExitCode)" }
    Start-Sleep -Seconds 2
    if (Test-Path -LiteralPath $App) { throw "Application binary remains after uninstall" }
    if (Test-Path -LiteralPath $SendToShortcut) { throw "Send to shortcut remains after uninstall: $SendToShortcut" }
    if ((Test-Path -LiteralPath $InstallDirectory) -and (Get-ChildItem -LiteralPath $InstallDirectory -Recurse -Force | Select-Object -First 1)) {
        throw "Installation files remain after uninstall: $InstallDirectory"
    }
    if (-not (Test-Path -LiteralPath $Sentinel -PathType Leaf)) { throw "Uninstall removed user data" }
    if ($EvidencePath) {
        $EvidenceDirectory = Split-Path -Parent $EvidencePath
        if ($EvidenceDirectory) { New-Item -ItemType Directory -Path $EvidenceDirectory -Force | Out-Null }
        [ordered]@{
            schema_version = 1
            status = "accepted"
            commit = $Commit
            workflow = $Workflow
            run_id = $RunId
            run_attempt = $RunAttempt
            installer_sha256 = $InstallerSha256
            checks = [ordered]@{
                app_launched = $true
                clean_shutdown = $true
                runtime_inventory_verified = $true
                installed_worker_core_path = $true
                uninstall_succeeded = $true
                user_data_retained = $true
            }
        } | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath $EvidencePath -Encoding utf8NoBOM
    }
    Write-Host "Per-user NSIS app launch, clean shutdown, signed runtime, worker PDF/OCR, Send to shortcut, and install/uninstall smoke passed."
}
finally {
    if ($AppProcess -and -not $AppProcess.HasExited) {
        Stop-Process -Id $AppProcess.Id -Force -ErrorAction SilentlyContinue
    }
    if (Test-Path -LiteralPath $InstallDirectory) {
        Write-Warning "Installer smoke directory remains for inspection: $InstallDirectory"
    }
}
