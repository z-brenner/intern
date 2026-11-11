<#
.SYNOPSIS
What a loaded llama-server takes out of Windows' available physical memory, and
what stopping it gives back, set against the tier plan's arithmetic.

.DESCRIPTION
Laboratory tooling, run by the memory probe in .github/workflows/runtime-speed.yml;
the app does not run it. For each model, accurate first, it starts upstream
llama-server with the app's flags (server.rs arguments_for, threads by
default_threads) and samples:

  * the available physical memory: GlobalMemoryStatusEx ullAvailPhys, the figure
    tiers::available_memory_bytes reads on Windows;
  * the memory lists behind it (free and zero, standby, modified);
  * the server's working set, peak working set and private bytes, read with
    GetProcessMemoryInfo on the process handle as the app can read its own child,
    and its private working set from the performance counters;

before the load, after /health, after one recorded request, after IdleSeconds
idle, and 3 s and 15 s after the server exits. With BalloonMiB, a process that
holds that much memory in its working set (a browser's share, say) runs
throughout, so the machine starts with less available. Then it applies plan()'s
arithmetic (src-tauri/src/commands.rs) with the constants in tiers.rs, read from
the source, two ways. With a model loaded, the plan sees ullAvailPhys plus a
credit for what unloading that model gives back, and the accurate model fits when
that reaches MIN_AVAILABLE_FOR_ACCURATE, plus UPGRADE_MARGIN while the fast model is
the one loaded. The credit the app uses is the live
server's working set (tiers::memory_released_by); the estimate
tiers::unload_gives_back(tier, file size), which the app credited up to alpha.14,
is now only its fallback for a server it cannot read. The probe judges both.
#>
param(
    [Parameter(Mandatory = $true)] [string]$Server,
    [Parameter(Mandatory = $true)] [string]$Accurate,
    [Parameter(Mandatory = $true)] [string]$Fast,
    [Parameter(Mandatory = $true)] [string]$Requests,
    [Parameter(Mandatory = $true)] [string]$Tiers,
    [Parameter(Mandatory = $true)] [string]$Out,
    [string]$Request = "0001",
    [int]$IdleSeconds = 60,
    [int]$BalloonMiB = 0,
    [int]$Port = 18300
)

$ErrorActionPreference = "Stop"
New-Item -ItemType Directory -Force -Path $Out | Out-Null

Add-Type -TypeDefinition @'
using System;
using System.ComponentModel;
using System.Runtime.InteropServices;

public static class InternMemoryProbe
{
    [StructLayout(LayoutKind.Sequential)]
    public struct MEMORYSTATUSEX
    {
        public uint dwLength;
        public uint dwMemoryLoad;
        public ulong ullTotalPhys;
        public ulong ullAvailPhys;
        public ulong ullTotalPageFile;
        public ulong ullAvailPageFile;
        public ulong ullTotalVirtual;
        public ulong ullAvailVirtual;
        public ulong ullAvailExtendedVirtual;
    }

    [StructLayout(LayoutKind.Sequential)]
    public struct PROCESS_MEMORY_COUNTERS_EX
    {
        public uint cb;
        public uint PageFaultCount;
        public UIntPtr PeakWorkingSetSize;
        public UIntPtr WorkingSetSize;
        public UIntPtr QuotaPeakPagedPoolUsage;
        public UIntPtr QuotaPagedPoolUsage;
        public UIntPtr QuotaPeakNonPagedPoolUsage;
        public UIntPtr QuotaNonPagedPoolUsage;
        public UIntPtr PagefileUsage;
        public UIntPtr PeakPagefileUsage;
        public UIntPtr PrivateUsage;
    }

    [DllImport("kernel32.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool GlobalMemoryStatusEx(ref MEMORYSTATUSEX status);

    [DllImport("psapi.dll", SetLastError = true)]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool GetProcessMemoryInfo(IntPtr process, out PROCESS_MEMORY_COUNTERS_EX counters, uint size);

    public static MEMORYSTATUSEX Status()
    {
        var status = new MEMORYSTATUSEX();
        status.dwLength = (uint)Marshal.SizeOf(typeof(MEMORYSTATUSEX));
        if (!GlobalMemoryStatusEx(ref status)) throw new Win32Exception();
        return status;
    }

    public static PROCESS_MEMORY_COUNTERS_EX Counters(IntPtr process)
    {
        PROCESS_MEMORY_COUNTERS_EX counters;
        if (!GetProcessMemoryInfo(process, out counters, (uint)Marshal.SizeOf(typeof(PROCESS_MEMORY_COUNTERS_EX))))
            throw new Win32Exception();
        return counters;
    }
}
'@

function ConvertTo-MiB([double]$Bytes) { [Math]::Round($Bytes / 1MB, 1) }

# The constants plan() uses, from the source, so the verdict is the code's.
$TiersSource = Get-Content -LiteralPath $Tiers -Raw
function Get-MiBConstant([string]$Pattern, [string]$Name) {
    $Match = [regex]::Match($TiersSource, $Pattern)
    if (-not $Match.Success) { throw "tiers.rs no longer states $Name the way this probe reads it" }
    [int]($Match.Groups[1].Value -replace "_", "")
}
$Floor = Get-MiBConstant 'pub const MIN_AVAILABLE_FOR_ACCURATE: u64 = ([0-9_]+) \* MIB;' "MIN_AVAILABLE_FOR_ACCURATE"
$Margin = Get-MiBConstant 'pub const UPGRADE_MARGIN: u64 = ([0-9_]+) \* MIB;' "UPGRADE_MARGIN"
$ResidentPattern = 'pub const fn resident_estimate\(tier: ModelTier\) -> u64 \{\s*match tier \{\s*ModelTier::Fast => ([0-9_]+) \* MIB,\s*ModelTier::Accurate => ([0-9_]+) \* MIB,'
$ResidentMatch = [regex]::Match($TiersSource, $ResidentPattern)
if (-not $ResidentMatch.Success) { throw "tiers.rs no longer states resident_estimate the way this probe reads it" }
$Resident = @{
    fast     = [int]($ResidentMatch.Groups[1].Value -replace "_", "")
    accurate = [int]($ResidentMatch.Groups[2].Value -replace "_", "")
}

# server.rs default_threads: every logical processor, between 2 and 8.
$Threads = [Math]::Max(2, [Math]::Min(8, [Environment]::ProcessorCount))

function Get-MemoryLists {
    try {
        $Memory = Get-CimInstance Win32_PerfRawData_PerfOS_Memory
        [ordered]@{
            free_and_zero_mib = ConvertTo-MiB $Memory.FreeAndZeroPageListBytes
            standby_mib       = ConvertTo-MiB ([double]$Memory.StandbyCacheCoreBytes + [double]$Memory.StandbyCacheNormalPriorityBytes + [double]$Memory.StandbyCacheReserveBytes)
            modified_mib      = ConvertTo-MiB $Memory.ModifiedPageListBytes
            committed_mib     = ConvertTo-MiB $Memory.CommittedBytes
        }
    } catch {
        [ordered]@{ error = $_.Exception.Message }
    }
}

function Get-Sample([string]$Tier, [string]$Stage, $Process) {
    $Status = [InternMemoryProbe]::Status()
    $Sample = [ordered]@{
        tier               = $Tier
        stage              = $Stage
        at                 = (Get-Date).ToUniversalTime().ToString("o")
        avail_phys_mib     = ConvertTo-MiB $Status.ullAvailPhys
        total_phys_mib     = ConvertTo-MiB $Status.ullTotalPhys
        memory_load_pct    = [int]$Status.dwMemoryLoad
        avail_pagefile_mib = ConvertTo-MiB $Status.ullAvailPageFile
        lists              = Get-MemoryLists
    }
    if ($null -ne $Process -and -not $Process.HasExited) {
        $Counters = [InternMemoryProbe]::Counters($Process.Handle)
        $Sample.working_set_mib = ConvertTo-MiB $Counters.WorkingSetSize.ToUInt64()
        $Sample.peak_working_set_mib = ConvertTo-MiB $Counters.PeakWorkingSetSize.ToUInt64()
        $Sample.private_bytes_mib = ConvertTo-MiB $Counters.PrivateUsage.ToUInt64()
        $Sample.page_faults = [uint64]$Counters.PageFaultCount
        $Process.Refresh()
        $Sample.ps_working_set64_mib = ConvertTo-MiB $Process.WorkingSet64
        $Sample.ps_private_memory_size64_mib = ConvertTo-MiB $Process.PrivateMemorySize64
        $Sample.ps_peak_working_set64_mib = ConvertTo-MiB $Process.PeakWorkingSet64
        try {
            $Perf = Get-CimInstance Win32_PerfRawData_PerfProc_Process -Filter "IDProcess = $($Process.Id)"
            $Sample.private_working_set_mib = ConvertTo-MiB $Perf.WorkingSetPrivate
        } catch {
            $Sample.private_working_set_mib = $null
        }
    }
    $Line = "{0,-8} {1,-14} available {2,8} MiB" -f $Tier, $Stage, $Sample.avail_phys_mib
    if ($Sample.Contains("working_set_mib")) {
        $Line += "  working set {0,8} MiB (private {1,8}, peak {2,8})  private bytes {3,8} MiB" -f `
            $Sample.working_set_mib, $Sample.private_working_set_mib, $Sample.peak_working_set_mib, $Sample.private_bytes_mib
    }
    Write-Host $Line
    $Sample
}

# Another program's memory: BalloonMiB allocated and touched, page by page, in
# a process of its own, so it sits in that process's working set as an open
# browser's does. Chunks of 256 MiB, since a .NET array stops short of 2 GiB.
$Balloon = $null
if ($BalloonMiB -gt 0) {
    $BalloonScript = @'
$Chunks = [System.Collections.Generic.List[byte[]]]::new()
for ($Held = 0; $Held -lt SIZE; $Held += 256) {
    $Length = [Math]::Min(256, SIZE - $Held) * 1MB
    $Chunk = [byte[]]::new($Length)
    for ($i = 0; $i -lt $Length; $i += 4096) { $Chunk[$i] = 1 }
    $Chunks.Add($Chunk)
}
Start-Sleep -Seconds 7200
'@
    $BalloonScript = $BalloonScript.Replace("SIZE", [string]$BalloonMiB)
    $Encoded = [Convert]::ToBase64String([Text.Encoding]::Unicode.GetBytes($BalloonScript))
    $Balloon = Start-Process -FilePath (Get-Process -Id $PID).Path -ArgumentList @("-NoProfile", "-NonInteractive", "-EncodedCommand", $Encoded) `
        -PassThru -NoNewWindow
    $Deadline = (Get-Date).AddMinutes(3)
    while ($true) {
        $Balloon.Refresh()
        if ($Balloon.HasExited) { throw "The balloon process exited" }
        if ($Balloon.WorkingSet64 -ge [int64]$BalloonMiB * 1MB) { break }
        if ((Get-Date) -gt $Deadline) { throw "The balloon did not fill in time" }
        Start-Sleep -Seconds 1
    }
    Write-Host ("Balloon of {0} MiB holds a working set of {1} MiB" -f $BalloonMiB, (ConvertTo-MiB $Balloon.WorkingSet64))
}

$Samples = [System.Collections.Generic.List[object]]::new()
$Replies = [System.Collections.Generic.List[object]]::new()
$Models = [ordered]@{ accurate = (Resolve-Path $Accurate).Path; fast = (Resolve-Path $Fast).Path }
$FileMiB = @{}
$WireBody = [IO.File]::ReadAllBytes((Join-Path $Requests "$Request-model-wire.json"))
$ServerPath = (Resolve-Path $Server).Path

$Samples.Add((Get-Sample "none" "start" $null))
try {
foreach ($Tier in $Models.Keys) {
    $Model = $Models[$Tier]
    $FileMiB[$Tier] = ConvertTo-MiB (Get-Item -LiteralPath $Model).Length
    $Samples.Add((Get-Sample $Tier "before-load" $null))
    $ApiKey = -join (1..32 | ForEach-Object { "{0:x2}" -f (Get-Random -Maximum 256) })
    # server.rs arguments_for with no projector, then the tier's own flags
    # (tiers::tier_server_flags: --no-repack for the accurate model).
    $Arguments = @("--host", "127.0.0.1", "--port", "$Port", "--api-key", $ApiKey, "--model", $Model,
        "--parallel", "1", "--ctx-size", "8192", "--n-gpu-layers", "0",
        "--threads", "$Threads", "--threads-batch", "$Threads", "--jinja", "--no-webui",
        "--cache-ram", "0", "-ub", "256", "--prio", "-1", "--poll", "0", "--no-mmproj")
    if ($Tier -eq "accurate") { $Arguments += "--no-repack" }
    $Process = Start-Process -FilePath $ServerPath -ArgumentList $Arguments -WorkingDirectory (Split-Path $ServerPath) `
        -PassThru -NoNewWindow -RedirectStandardOutput (Join-Path $Out "$Tier-server.out.log") `
        -RedirectStandardError (Join-Path $Out "$Tier-server.log")
    # Hold the handle from the start: GetProcessMemoryInfo reads through it.
    $null = $Process.Handle
    # The app starts its sidecars below normal priority (process.rs sidecar_creation_flags).
    try { $Process.PriorityClass = [System.Diagnostics.ProcessPriorityClass]::BelowNormal } catch { }
    try {
        $Healthy = $false
        $Deadline = (Get-Date).AddMinutes(10)
        while ((Get-Date) -lt $Deadline) {
            if ($Process.HasExited) { break }
            try {
                $Health = Invoke-WebRequest -Uri "http://127.0.0.1:$Port/health" -Headers @{ Authorization = "Bearer $ApiKey" } -UseBasicParsing -TimeoutSec 2
                if ($Health.StatusCode -eq 200) { $Healthy = $true; break }
            } catch { }
            Start-Sleep -Milliseconds 250
        }
        if (-not $Healthy) {
            Get-Content (Join-Path $Out "$Tier-server.log") -Tail 40
            throw "The $Tier llama-server did not become healthy"
        }
        $Samples.Add((Get-Sample $Tier "after-health" $Process))

        $Started = Get-Date
        $Reply = Invoke-RestMethod -Uri "http://127.0.0.1:$Port/v1/chat/completions" -Method Post `
            -Headers @{ Authorization = "Bearer $ApiKey" } -ContentType "application/json" -Body $WireBody -TimeoutSec 1800
        $Replies.Add([ordered]@{
                tier         = $Tier
                request      = $Request
                wall_s       = [Math]::Round(((Get-Date) - $Started).TotalSeconds, 1)
                prompt_n     = $Reply.timings.prompt_n
                prompt_ms    = $Reply.timings.prompt_ms
                predicted_n  = $Reply.timings.predicted_n
                predicted_ms = $Reply.timings.predicted_ms
            })
        $Samples.Add((Get-Sample $Tier "after-request" $Process))

        Start-Sleep -Seconds $IdleSeconds
        $Samples.Add((Get-Sample $Tier "after-idle" $Process))
    } finally {
        if (-not $Process.HasExited) { Stop-Process -Id $Process.Id -Force }
        $null = $Process.WaitForExit(60000)
    }
    Start-Sleep -Seconds 3
    $Samples.Add((Get-Sample $Tier "exit+3s" $null))
    Start-Sleep -Seconds 12
    $Samples.Add((Get-Sample $Tier "exit+15s" $null))
}
} finally {
    if ($null -ne $Balloon -and -not $Balloon.HasExited) { Stop-Process -Id $Balloon.Id -Force }
}

function Find-Sample([string]$Tier, [string]$Stage) {
    $Samples | Where-Object { $_.tier -eq $Tier -and $_.stage -eq $Stage } | Select-Object -First 1
}

# plan() in commands.rs: with `tier` loaded, the plan sees what the machine
# reports available plus the server's working set (tiers::memory_released_by),
# or, when it cannot read that, the estimate tiers::unload_gives_back(tier, file
# size), that is resident_estimate(tier) - file size. The accurate model fits at
# the floor, and with the fast model loaded at the floor plus UPGRADE_MARGIN
# (tiers::memory_for_plan). Each verdict is made both ways. The estimate's
# verdicts, and the bands below, are the arithmetic up to alpha.14, which had
# no margin.
$Verdicts = [System.Collections.Generic.List[object]]::new()
$Lines = [System.Collections.Generic.List[string]]::new()
$Lines.Add("Windows memory probe: $([Environment]::ProcessorCount) logical processors, $Threads server threads, " +
    "$((Find-Sample 'none' 'start').total_phys_mib) MiB physical, a balloon of $BalloonMiB MiB; " +
    "MIN_AVAILABLE_FOR_ACCURATE $Floor MiB, UPGRADE_MARGIN $Margin MiB.")
$Lines.Add("")
$Lines.Add("| Loaded | Stage | Available | Working set | Estimate (fallback) | Plan sees | Accurate model | Credit = working set | Plan sees | Accurate model |")
$Lines.Add("| --- | --- | ---: | ---: | ---: | ---: | --- | ---: | ---: | --- |")
foreach ($Tier in $Models.Keys) {
    $Credit = [Math]::Max(0.0, $Resident[$Tier] - $FileMiB[$Tier])
    $Asked = if ($Tier -eq "fast") { $Floor + $Margin } else { $Floor }
    foreach ($Stage in @("after-health", "after-request", "after-idle")) {
        $Sample = Find-Sample $Tier $Stage
        $Sees = $Sample.avail_phys_mib + $Credit
        $SeesMeasured = $Sample.avail_phys_mib + $Sample.working_set_mib
        $Verdict = [ordered]@{
            tier                  = $Tier
            stage                 = $Stage
            avail_phys_mib        = $Sample.avail_phys_mib
            working_set_mib       = $Sample.working_set_mib
            estimate_credit_mib   = $Credit
            plan_sees_mib         = $Sees
            accurate_fits         = ($Sees -ge $Floor)
            measured_credit_mib   = $Sample.working_set_mib
            plan_sees_measured    = $SeesMeasured
            accurate_fits_measured = ($SeesMeasured -ge $Asked)
        }
        $Verdicts.Add($Verdict)
        $Word = { param($Fits) if ($Fits) { "fits" } else { "does not fit" } }
        $Lines.Add("| $Tier | $Stage | $($Sample.avail_phys_mib) | $($Sample.working_set_mib) | $Credit | $Sees | " +
            "$(& $Word $Verdict.accurate_fits) | $($Sample.working_set_mib) | $SeesMeasured | $(& $Word $Verdict.accurate_fits_measured) |")
    }
}

$Lines.Add("")
$Lines.Add("| Model | Available before load | After request | Taken by the load | Given back 3 s after exit | Estimate (fallback) | Under-credit |")
$Lines.Add("| --- | ---: | ---: | ---: | ---: | ---: | ---: |")
$Bands = @{}
foreach ($Tier in $Models.Keys) {
    $Before = (Find-Sample $Tier "before-load").avail_phys_mib
    $Loaded = (Find-Sample $Tier "after-request").avail_phys_mib
    $Idle = (Find-Sample $Tier "after-idle").avail_phys_mib
    $Exited = (Find-Sample $Tier "exit+3s").avail_phys_mib
    $Credit = [Math]::Max(0.0, $Resident[$Tier] - $FileMiB[$Tier])
    $Taken = [Math]::Round($Before - $Loaded, 1)
    $GivenBack = [Math]::Round($Exited - $Idle, 1)
    $Bands[$Tier] = [ordered]@{ taken_mib = $Taken; given_back_mib = $GivenBack; estimate_credit_mib = $Credit }
    $Lines.Add("| $Tier | $Before | $Loaded | $Taken | $GivenBack | $Credit | $([Math]::Round($GivenBack - $Credit, 1)) |")
}

# On a machine with A MiB available before any model is loaded, and these
# models taking what they took here after one document (a linear extrapolation
# from this one load: Windows may trim harder where less is available), and the
# estimate credited: the accurate model is loaded when A >= floor; with it loaded
# the plan stands it down when A - taken + credit < floor; with the fast one
# loaded it brings the accurate one back when A - taken + credit >= floor.
$StandsDown = [Math]::Round($Floor + $Bands.accurate.taken_mib - $Bands.accurate.estimate_credit_mib, 1)
$ComesBack = [Math]::Round($Floor + $Bands.fast.taken_mib - $Bands.fast.estimate_credit_mib, 1)
$Lines.Add("")
$Lines.Add("Credited only the estimate (the fallback; up to alpha.14 the only credit), a machine with A MiB available before any model is loaded:")
$Lines.Add("- loads the accurate model for the first document when A >= $Floor;")
$Lines.Add("- with the accurate model loaded, plans the fast one for the next document when A < $StandsDown;")
$Lines.Add("- with the fast model loaded, plans the accurate one again when A >= $ComesBack.")
$Low = [Math]::Max($Floor, $ComesBack)
if ($Low -lt $StandsDown) {
    $Lines.Add("So the models swap before every document for $Low <= A < $StandsDown MiB.")
} else {
    $Lines.Add("So there is no A for which the models swap before every document.")
}
if ($Floor -lt [Math]::Min($ComesBack, $StandsDown)) {
    $Lines.Add("And for $Floor <= A < $([Math]::Min($ComesBack, $StandsDown)) MiB the accurate model reads one document and the fast one the rest.")
}
$Here = (Find-Sample "accurate" "before-load").avail_phys_mib
$Lines.Add("This runner had A = $Here MiB before the first load.")

$Report = [ordered]@{
    threads          = $Threads
    logical          = [Environment]::ProcessorCount
    request          = $Request
    idle_seconds     = $IdleSeconds
    balloon_mib      = $BalloonMiB
    floor_mib        = $Floor
    upgrade_margin_mib = $Margin
    resident_mib     = $Resident
    file_mib         = $FileMiB
    samples          = $Samples
    replies          = $Replies
    verdicts         = $Verdicts
    bands            = $Bands
    stands_down_below_mib = $StandsDown
    comes_back_from_mib   = $ComesBack
    runner_available_before_mib = $Here
}
$Report | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath (Join-Path $Out "memory-probe.json") -Encoding utf8NoBOM
$Text = $Lines -join "`n"
$Text | Set-Content -LiteralPath (Join-Path $Out "summary.md") -Encoding utf8NoBOM
Write-Host ""
Write-Host $Text
if ($env:GITHUB_STEP_SUMMARY) { Add-Content -LiteralPath $env:GITHUB_STEP_SUMMARY -Value $Text -Encoding utf8NoBOM }
