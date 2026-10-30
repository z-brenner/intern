[CmdletBinding()]
param(
    [string]$CacheDirectory = "",
    [switch]$KeepWorkDirectory
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$RepositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$ManifestPath = Join-Path $RepositoryRoot "src-tauri/resources/runtime-assets.json"
$Manifest = Get-Content -LiteralPath $ManifestPath -Raw | ConvertFrom-Json
$WorkDirectory = if ($CacheDirectory) {
    [System.IO.Path]::GetFullPath($CacheDirectory)
} else {
    Join-Path ([System.IO.Path]::GetTempPath()) ("intern-assets-" + [guid]::NewGuid().ToString("N"))
}
$OwnsWorkDirectory = -not $CacheDirectory
$BinariesDirectory = Join-Path $RepositoryRoot "src-tauri/binaries"
$ResourcesDirectory = Join-Path $RepositoryRoot "src-tauri/resources"
$NativeDirectory = Join-Path $ResourcesDirectory "native"
# Upstream llama.cpp's server and every DLL it loads, in a folder of their
# own: they share file names with the PrismML fork's at the install root, and
# Windows, like ggml looking for its CPU backends, looks first beside the
# executable a process was started from.
$LlamaUpstreamDirectory = Join-Path $NativeDirectory "llama-upstream"
$TessdataDirectory = Join-Path $ResourcesDirectory "tessdata"
$OcrModelsDirectory = Join-Path $ResourcesDirectory "ocr-models"
$LicensesDirectory = Join-Path $ResourcesDirectory "licenses"
$TargetTriple = "x86_64-pc-windows-msvc"
$RuntimePackages = @{}

function Assert-ExactFile {
    param([string]$Path, [long]$Size, [string]$Sha256, [string]$Label)
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) { throw "$Label is missing: $Path" }
    $ActualSize = (Get-Item -LiteralPath $Path).Length
    if ($ActualSize -ne $Size) { throw "$Label size mismatch: expected $Size, got $ActualSize" }
    $ActualHash = (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($ActualHash -ne $Sha256.ToLowerInvariant()) { throw "$Label SHA-256 mismatch: expected $Sha256, got $ActualHash" }
}

function Get-PinnedDownload {
    param([pscustomobject]$Download)
    $Destination = Join-Path $WorkDirectory $Download.archive
    # These come from GitHub release and CDN endpoints that return 503 often
    # enough to have failed two consecutive CI runs on the first request, eleven
    # seconds in. Retry with backoff, and discard a partial file before retrying
    # so a truncated response cannot be mistaken for a cache hit. Every attempt
    # still has to satisfy the pinned size and SHA-256 below, so retrying widens
    # no trust boundary.
    $Attempts = 0
    while (-not (Test-Path -LiteralPath $Destination -PathType Leaf)) {
        $Attempts += 1
        try {
            Invoke-WebRequest -Uri $Download.url -OutFile $Destination -MaximumRedirection 5
        } catch {
            if (Test-Path -LiteralPath $Destination) { Remove-Item -LiteralPath $Destination -Force }
            if ($Attempts -ge 4) { throw "Failed to download $($Download.id) after $Attempts attempts: $($_.Exception.Message)" }
            Write-Warning "Download of $($Download.id) failed on attempt ${Attempts}: $($_.Exception.Message)"
            Start-Sleep -Seconds (15 * $Attempts)
        }
    }
    Assert-ExactFile -Path $Destination -Size $Download.size -Sha256 $Download.sha256 -Label $Download.id
    return $Destination
}

function Copy-RuntimeFile {
    param([string]$Source, [string]$Destination)
    $Parent = Split-Path -Parent $Destination
    New-Item -ItemType Directory -Path $Parent -Force | Out-Null
    Copy-Item -LiteralPath $Source -Destination $Destination -Force
}

function Register-RuntimePackage {
    param([Parameter(Mandatory = $true)][string]$Path, [Parameter(Mandatory = $true)][string]$Name, [Parameter(Mandatory = $true)][string]$Version)
    if (-not $RuntimePackages.ContainsKey($Path)) { $RuntimePackages[$Path] = [System.Collections.Generic.List[object]]::new() }
    if (-not ($RuntimePackages[$Path] | Where-Object { $_.name -eq $Name -and $_.version -eq $Version })) {
        $RuntimePackages[$Path].Add([ordered]@{ name = $Name; version = $Version })
    }
}

# Stages a DLL into one folder under native/: sidecars/, whose files install
# beside the executables, unless another folder is named. Two files of one
# name in one folder must be the same bytes; each folder is checked on its own.
function Copy-SidecarDll {
    param([string]$Source, [Parameter(Mandatory = $true)][string]$Package, [Parameter(Mandatory = $true)][string]$Version, [string]$Folder = "sidecars")
    $Destination = Join-Path $NativeDirectory (Join-Path $Folder (Split-Path -Leaf $Source))
    if (Test-Path -LiteralPath $Destination -PathType Leaf) {
        $SourceHash = (Get-FileHash -LiteralPath $Source -Algorithm SHA256).Hash
        $DestinationHash = (Get-FileHash -LiteralPath $Destination -Algorithm SHA256).Hash
        if ($SourceHash -ne $DestinationHash) { throw "Sidecar DLL name collision with different bytes: $Folder/$(Split-Path -Leaf $Source)" }
        Register-RuntimePackage $Destination $Package $Version
        return
    }
    Copy-RuntimeFile $Source $Destination
    Register-RuntimePackage $Destination $Package $Version
}

function Assert-SafeArchivePath {
    param([Parameter(Mandatory = $true)][string]$Entry, [Parameter(Mandatory = $true)][string]$Archive)
    $Normalized = $Entry.Replace("\", "/")
    if ([string]::IsNullOrWhiteSpace($Normalized) -or $Normalized.StartsWith("/") -or
        $Normalized -match '^[A-Za-z]:' -or ($Normalized.Split("/") -contains "..")) {
        throw "Unsafe archive entry in $Archive`: $Entry"
    }
}

function Assert-SafeZipArchive {
    param([Parameter(Mandatory = $true)][string]$Path)
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    $Zip = [System.IO.Compression.ZipFile]::OpenRead($Path)
    try {
        foreach ($Entry in $Zip.Entries) { Assert-SafeArchivePath $Entry.FullName $Path }
    } finally { $Zip.Dispose() }
}

function Assert-SafeTarArchive {
    param([Parameter(Mandatory = $true)][string]$Path)
    $Entries = @(& tar -tzf $Path)
    if ($LASTEXITCODE -ne 0) { throw "Failed to inspect pinned archive: $Path" }
    foreach ($Entry in $Entries) { Assert-SafeArchivePath $Entry $Path }
}

function Copy-UpstreamLicense {
    param([Parameter(Mandatory = $true)][string]$Root, [Parameter(Mandatory = $true)][string]$DestinationName)
    $Candidates = @(Get-ChildItem -LiteralPath $Root -Recurse -File | Where-Object { $_.Name -match '^LICEN[CS]E(?:\..+)?$' } | Sort-Object @{ Expression = { $_.FullName.Length } }, FullName)
    if ($Candidates.Count -eq 0) { throw "Pinned upstream archive does not contain a complete license text: $Root" }
    Copy-RuntimeFile $Candidates[0].FullName (Join-Path $LicensesDirectory $DestinationName)
}

# Stages one pinned llama.cpp Windows CPU release. Its zip is flat and carries
# every CLI tool, their private impl DLLs and an RPC backend and server beside
# llama-server. Only llama-server is taken, to $ServerDestination, with exactly
# the DLLs of its import closure that the archive carries ($Required) and every
# CPU backend variant, into native/$Folder. ggml loads the backend at start-up
# by picking the best one for the processor (AVX2, AVX-512 and so on); without
# the right one a laptop runs on the slowest or none. All of it is owned by
# llama.cpp at the download's version. Returns the folder the archive keeps
# llama-server in, for anything else a caller takes from beside it.
function Install-LlamaServer {
    param(
        [Parameter(Mandatory = $true)][string]$Archive,
        [Parameter(Mandatory = $true)][string]$Version,
        [Parameter(Mandatory = $true)][string]$Extract,
        [Parameter(Mandatory = $true)][string]$ServerDestination,
        [Parameter(Mandatory = $true)][string]$Folder,
        [Parameter(Mandatory = $true)][string[]]$Required
    )
    if (Test-Path -LiteralPath $Extract) { Remove-Item -LiteralPath $Extract -Recurse -Force }
    Assert-SafeZipArchive $Archive
    Expand-Archive -LiteralPath $Archive -DestinationPath $Extract
    $Server = @(Get-ChildItem -LiteralPath $Extract -Recurse -File -Filter "llama-server.exe")
    if ($Server.Count -ne 1) { throw "Expected exactly one llama-server.exe in the pinned llama.cpp $Version archive" }
    Copy-RuntimeFile $Server[0].FullName $ServerDestination
    Register-RuntimePackage $ServerDestination "llama.cpp" $Version
    $Directory = $Server[0].Directory.FullName
    foreach ($Name in $Required) {
        $Source = Join-Path $Directory $Name
        if (-not (Test-Path -LiteralPath $Source -PathType Leaf)) { throw "The pinned llama.cpp $Version archive has no $Name" }
        Copy-SidecarDll $Source "llama.cpp" $Version -Folder $Folder
    }
    $CpuBackends = @(Get-ChildItem -LiteralPath $Directory -File -Filter "ggml-cpu-*.dll")
    if ($CpuBackends.Count -eq 0) { throw "The pinned llama.cpp $Version archive carries no CPU backend" }
    foreach ($Backend in $CpuBackends) { Copy-SidecarDll $Backend.FullName "llama.cpp" $Version -Folder $Folder }
    return $Directory
}

function Get-PackagedRuntimePath {
    param([Parameter(Mandatory = $true)][string]$Path)
    $Relative = [System.IO.Path]::GetRelativePath($RepositoryRoot, $Path).Replace("\", "/")
    if ($Relative.StartsWith("src-tauri/binaries/")) {
        return (Split-Path -Leaf $Path).Replace("-$TargetTriple", "")
    }
    if ($Relative.StartsWith("src-tauri/resources/tessdata/")) { return "tessdata/$(Split-Path -Leaf $Path)" }
    if ($Relative.StartsWith("src-tauri/resources/ocr-models/")) { return "ocr-models/$(Split-Path -Leaf $Path)" }
    # Before the rest of native/, which installs flat beside the executables.
    if ($Relative.StartsWith("src-tauri/resources/native/llama-upstream/")) { return "llama-upstream/$(Split-Path -Leaf $Path)" }
    if ($Relative.StartsWith("src-tauri/resources/native/")) { return (Split-Path -Leaf $Path) }
    throw "No package mapping for staged runtime file: $Relative"
}

function Get-RuntimePackages {
    param([Parameter(Mandatory = $true)][string]$Path)
    $Leaf = Split-Path -Leaf $Path
    if ($Leaf -eq "intern-worker-$TargetTriple.exe") { return @([ordered]@{ name = "Intern"; version = "0.1.0-alpha.12" }) }
    if ($RuntimePackages.ContainsKey($Path)) { return @($RuntimePackages[$Path]) }
    throw "No package identity for staged runtime file: $Path"
}

try {
    New-Item -ItemType Directory -Path $WorkDirectory -Force | Out-Null
    New-Item -ItemType Directory -Path $BinariesDirectory -Force | Out-Null
    New-Item -ItemType Directory -Path $ResourcesDirectory -Force | Out-Null
    if (Test-Path -LiteralPath $NativeDirectory) { Remove-Item -LiteralPath $NativeDirectory -Recurse -Force }
    if (Test-Path -LiteralPath $TessdataDirectory) { Remove-Item -LiteralPath $TessdataDirectory -Recurse -Force }
    if (Test-Path -LiteralPath $LicensesDirectory) { Remove-Item -LiteralPath $LicensesDirectory -Recurse -Force }
    if (Test-Path -LiteralPath $OcrModelsDirectory) { Remove-Item -LiteralPath $OcrModelsDirectory -Recurse -Force }
    # The native wipe above takes llama-upstream with it.
    New-Item -ItemType Directory -Path $NativeDirectory, $LlamaUpstreamDirectory, $TessdataDirectory, $LicensesDirectory, $OcrModelsDirectory -Force | Out-Null

    $Downloads = @{}
    foreach ($Download in $Manifest.downloads) { $Downloads[$Download.id] = Get-PinnedDownload $Download }

    # Two llama.cpp builds ship, each staged as Install-LlamaServer describes,
    # with the same import closure: llama-server.exe -> llama-server-impl.dll
    # -> llama-common, mtmd, llama, ggml, ggml-base.
    #
    # The PrismML fork serves the fast tier: its PQ2_0 kernels are what the
    # Bonsai model needs. Its server is the llama-server sidecar at the install
    # root, and its DLLs install beside it from sidecars/. Its zip also carries
    # an OpenMP runtime (libomp140.x86_64.dll) that nothing in its CPU build
    # imports, so that is left behind with the CLI tools.
    $PrismVersion = [string]($Manifest.downloads | Where-Object id -eq "llama.cpp").version
    $PrismRequired = @("llama-server-impl.dll", "llama-common.dll", "mtmd.dll", "llama.dll", "ggml.dll", "ggml-base.dll")
    $PrismServer = Join-Path $BinariesDirectory "llama-server-$TargetTriple.exe"
    $null = Install-LlamaServer -Archive $Downloads["llama.cpp"] -Version $PrismVersion -Extract (Join-Path $WorkDirectory "llama") -ServerDestination $PrismServer -Folder "sidecars" -Required $PrismRequired
    # Upstream llama.cpp (ggml-org) serves the accurate tier. Everything of it
    # installs in llama-upstream\, the server included, so its DLLs are the
    # ones found beside it and never the fork's.
    $UpstreamVersion = [string]($Manifest.downloads | Where-Object id -eq "llama.cpp-upstream").version
    $UpstreamRequired = @("llama-server-impl.dll", "llama-common.dll", "mtmd.dll", "llama.dll", "ggml.dll", "ggml-base.dll")
    $UpstreamServer = Join-Path $LlamaUpstreamDirectory "llama-server.exe"
    $UpstreamDirectory = Install-LlamaServer -Archive $Downloads["llama.cpp-upstream"] -Version $UpstreamVersion -Extract (Join-Path $WorkDirectory "llama-upstream") -ServerDestination $UpstreamServer -Folder "llama-upstream" -Required $UpstreamRequired
    # Unlike the fork's, upstream's ggml-base.dll and every one of its CPU
    # backends import libomp.dll, LLVM's OpenMP runtime, which its zip carries
    # with its license. Without it beside them the server cannot start at all.
    # It is LLVM's code, not llama.cpp's, and is owned as such.
    $OpenMpRuntime = Join-Path $UpstreamDirectory "libomp.dll"
    if (-not (Test-Path -LiteralPath $OpenMpRuntime -PathType Leaf)) { throw "The pinned llama.cpp $UpstreamVersion archive has no libomp.dll" }
    Copy-SidecarDll $OpenMpRuntime "llvm-openmp" $UpstreamVersion -Folder "llama-upstream"
    $OpenMpLicense = Join-Path $UpstreamDirectory "LICENSE-LLVM-OpenMP"
    if (-not (Test-Path -LiteralPath $OpenMpLicense -PathType Leaf)) { throw "The pinned llama.cpp $UpstreamVersion archive has no LICENSE-LLVM-OpenMP" }
    Copy-RuntimeFile $OpenMpLicense (Join-Path $LicensesDirectory "llvm-openmp-LICENSE.txt")
    # Neither build's Windows release ships llama.cpp's own license text inside
    # its archive, so the fork's MIT license is pinned as its own download from
    # the fork's tag. Upstream's LICENSE at b11946 is byte for byte the same
    # file, so this one copy covers both builds.
    Copy-RuntimeFile $Downloads["llama.cpp-license"] (Join-Path $LicensesDirectory "llama.cpp-LICENSE.txt")

    $PdfiumExtract = Join-Path $WorkDirectory "pdfium"
    if (Test-Path -LiteralPath $PdfiumExtract) { Remove-Item -LiteralPath $PdfiumExtract -Recurse -Force }
    New-Item -ItemType Directory -Path $PdfiumExtract | Out-Null
    Assert-SafeTarArchive $Downloads["pdfium"]
    & tar -xzf $Downloads["pdfium"] -C $PdfiumExtract
    if ($LASTEXITCODE -ne 0) { throw "Failed to extract pinned PDFium archive" }
    $PdfiumDll = @(Get-ChildItem -LiteralPath $PdfiumExtract -Recurse -File -Filter "pdfium.dll")
    if ($PdfiumDll.Count -ne 1) { throw "Expected exactly one pdfium.dll in the pinned archive" }
    $PdfiumDestination = Join-Path $NativeDirectory "pdfium/pdfium.dll"
    Copy-RuntimeFile $PdfiumDll[0].FullName $PdfiumDestination
    Register-RuntimePackage $PdfiumDestination "PDFium" ([string]($Manifest.downloads | Where-Object id -eq "pdfium").version)
    Copy-UpstreamLicense $PdfiumExtract "PDFium-LICENSE.txt"

    $EngDestination = Join-Path $TessdataDirectory "eng.traineddata"
    $OsdDestination = Join-Path $TessdataDirectory "osd.traineddata"
    Copy-RuntimeFile $Downloads["eng.traineddata"] $EngDestination
    Copy-RuntimeFile $Downloads["osd.traineddata"] $OsdDestination
    $TessdataVersion = [string]($Manifest.downloads | Where-Object id -eq "eng.traineddata").version
    Register-RuntimePackage $EngDestination "tessdata_fast" $TessdataVersion
    Register-RuntimePackage $OsdDestination "tessdata_fast" $TessdataVersion

    # ONNX Runtime for PP-OCR: Microsoft's own CPU build from its NuGet
    # package, which is a zip of every platform's build. Only the x64 runtime
    # DLL and the package's license texts are taken out of it, and the DLL is
    # checked against its own pin as well as the package's. It is staged
    # beside the executables like every other sidecar DLL; the worker loads
    # it by absolute path from the install directory.
    $OnnxDownload = $Manifest.downloads | Where-Object id -eq "onnxruntime"
    $OnnxExtract = Join-Path $WorkDirectory "onnxruntime"
    if (Test-Path -LiteralPath $OnnxExtract) { Remove-Item -LiteralPath $OnnxExtract -Recurse -Force }
    New-Item -ItemType Directory -Path $OnnxExtract | Out-Null
    Assert-SafeZipArchive $Downloads["onnxruntime"]
    $OnnxPackage = [System.IO.Compression.ZipFile]::OpenRead($Downloads["onnxruntime"])
    try {
        foreach ($EntryName in @($OnnxDownload.extract.path, "LICENSE", "ThirdPartyNotices.txt")) {
            $Entry = $OnnxPackage.GetEntry($EntryName)
            if ($null -eq $Entry) { throw "The pinned ONNX Runtime package has no $EntryName" }
            [System.IO.Compression.ZipFileExtensions]::ExtractToFile($Entry, (Join-Path $OnnxExtract (Split-Path -Leaf $EntryName)), $true)
        }
    } finally { $OnnxPackage.Dispose() }
    $OnnxDll = Join-Path $OnnxExtract (Split-Path -Leaf $OnnxDownload.extract.path)
    Assert-ExactFile -Path $OnnxDll -Size $OnnxDownload.extract.size -Sha256 $OnnxDownload.extract.sha256 -Label "onnxruntime extracted DLL"
    # "onnxruntime", the upstream project's own name: a package name with a
    # space breaks the application SBOM (see scripts/generate-sbom.ps1).
    Copy-SidecarDll $OnnxDll "onnxruntime" ([string]$OnnxDownload.version)
    Copy-RuntimeFile (Join-Path $OnnxExtract "LICENSE") (Join-Path $LicensesDirectory "onnxruntime-LICENSE.txt")
    Copy-RuntimeFile (Join-Path $OnnxExtract "ThirdPartyNotices.txt") (Join-Path $LicensesDirectory "onnxruntime-ThirdPartyNotices.txt")

    # The PP-OCR models, each pinned to a Hugging Face revision, under the
    # file names the worker looks for. They carry no license file of their
    # own; their Apache-2.0 text is pinned from the PaddleOCR repository.
    $OcrModelPackages = [ordered]@{
        "ocr-text-detection" = "PaddlePaddle/PP-OCRv5_mobile_det_onnx"
        "ocr-text-recognition" = "PaddlePaddle/en_PP-OCRv5_mobile_rec_onnx"
        "ocr-page-orientation" = "PaddlePaddle/PP-LCNet_x1_0_doc_ori_onnx"
    }
    foreach ($ModelId in $OcrModelPackages.Keys) {
        $ModelDownload = $Manifest.downloads | Where-Object id -eq $ModelId
        $ModelDestination = Join-Path $OcrModelsDirectory $ModelDownload.archive
        Copy-RuntimeFile $Downloads[$ModelId] $ModelDestination
        Register-RuntimePackage $ModelDestination $OcrModelPackages[$ModelId] ([string]$ModelDownload.version)
    }
    Copy-RuntimeFile $Downloads["paddleocr-license"] (Join-Path $LicensesDirectory "PaddleOCR-models-LICENSE.txt")

    $VcpkgDirectory = Join-Path $WorkDirectory "vcpkg"
    if (-not (Test-Path -LiteralPath (Join-Path $VcpkgDirectory ".git"))) {
        & git clone --filter=blob:none $Manifest.vcpkg.repository $VcpkgDirectory
        if ($LASTEXITCODE -ne 0) { throw "Failed to clone vcpkg" }
    }
    & git -C $VcpkgDirectory fetch --depth 1 origin $Manifest.vcpkg.baseline
    if ($LASTEXITCODE -ne 0) { throw "Failed to fetch pinned vcpkg baseline" }
    & git -C $VcpkgDirectory -c advice.detachedHead=false checkout --force $Manifest.vcpkg.baseline
    if ($LASTEXITCODE -ne 0) { throw "Failed to check out pinned vcpkg baseline" }
    $CheckedOutBaseline = (& git -C $VcpkgDirectory rev-parse HEAD).Trim()
    if ($CheckedOutBaseline -ne $Manifest.vcpkg.baseline) { throw "vcpkg checkout did not resolve to the pinned baseline" }
    & (Join-Path $VcpkgDirectory "bootstrap-vcpkg.bat") -disableMetrics
    if ($LASTEXITCODE -ne 0) { throw "Failed to bootstrap pinned vcpkg" }
    $InstallRoot = Join-Path $WorkDirectory "vcpkg-installed"
    # vcpkg fetches each port's sources from upstream release URLs, and those
    # transiently return 503. One such response after twenty minutes of building
    # fails the whole job, so retry the install: it resumes from what it already
    # built rather than starting over, and every source it downloads is still
    # checked against the pinned baseline's own hashes.
    $BuildAttempts = 0
    while ($true) {
        $BuildAttempts += 1
        & (Join-Path $VcpkgDirectory "vcpkg.exe") install "tesseract:$($Manifest.vcpkg.triplet)" "--x-install-root=$InstallRoot" --clean-after-build --disable-metrics
        if ($LASTEXITCODE -eq 0) { break }
        if ($BuildAttempts -ge 3) { throw "Failed to build pinned Tesseract runtime" }
        Write-Warning "vcpkg install failed on attempt $BuildAttempts; retrying"
        Start-Sleep -Seconds (30 * $BuildAttempts)
    }
    $Installed = (& (Join-Path $VcpkgDirectory "vcpkg.exe") list "--x-install-root=$InstallRoot") -join "`n"
    if ($Installed -notmatch "(?m)^tesseract:$([regex]::Escape($Manifest.vcpkg.triplet))\s+$([regex]::Escape($Manifest.vcpkg.packages.tesseract))\b") {
        throw "Pinned vcpkg baseline did not produce Tesseract $($Manifest.vcpkg.packages.tesseract)"
    }
    $RuntimeBin = Join-Path $InstallRoot "$($Manifest.vcpkg.triplet)/bin"
    $VcpkgMetadataPath = Join-Path $WorkDirectory "vcpkg-runtime-metadata.json"
    & node (Join-Path $PSScriptRoot "vcpkg-runtime-metadata.mjs") "--install-root=$InstallRoot" "--triplet=$($Manifest.vcpkg.triplet)" "--output=$VcpkgMetadataPath"
    if ($LASTEXITCODE -ne 0) { throw "Failed to derive vcpkg runtime ownership metadata" }
    $VcpkgMetadata = Get-Content -LiteralPath $VcpkgMetadataPath -Raw | ConvertFrom-Json
    $VcpkgOwners = @{}
    foreach ($Property in $VcpkgMetadata.owners.PSObject.Properties) {
        $VcpkgOwners[$Property.Name] = $Property.Value
    }
    function Get-VcpkgOwner {
        param([Parameter(Mandatory = $true)][string]$Path)
        $Relative = [IO.Path]::GetRelativePath($InstallRoot, $Path).Replace("\", "/")
        if (-not $VcpkgOwners.ContainsKey($Relative)) { throw "No vcpkg package metadata owns runtime file $Relative" }
        return $VcpkgOwners[$Relative]
    }
    $TesseractExe = @(Get-ChildItem -LiteralPath (Join-Path $InstallRoot $Manifest.vcpkg.triplet) -Recurse -File -Filter "tesseract.exe")
    if ($TesseractExe.Count -ne 1) { throw "Expected exactly one tesseract.exe from the pinned vcpkg build" }
    $TesseractDestination = Join-Path $BinariesDirectory "tesseract-$TargetTriple.exe"
    Copy-RuntimeFile $TesseractExe[0].FullName $TesseractDestination
    $TesseractOwner = Get-VcpkgOwner $TesseractExe[0].FullName
    Register-RuntimePackage $TesseractDestination $TesseractOwner.name $TesseractOwner.version
    Get-ChildItem -LiteralPath $RuntimeBin -File -Filter "*.dll" | ForEach-Object {
        $Owner = Get-VcpkgOwner $_.FullName
        Copy-SidecarDll $_.FullName $Owner.name $Owner.version
    }

    $VcpkgCopyrights = @(Get-ChildItem -LiteralPath (Join-Path $InstallRoot "$($Manifest.vcpkg.triplet)/share") -Recurse -File -Filter "copyright" | Sort-Object FullName)
    if ($VcpkgCopyrights.Count -eq 0) { throw "vcpkg did not stage copyright files for the Tesseract runtime closure" }
    foreach ($Copyright in $VcpkgCopyrights) {
        $Package = Split-Path -Leaf (Split-Path -Parent $Copyright.FullName)
        Copy-RuntimeFile $Copyright.FullName (Join-Path $LicensesDirectory "vcpkg/$Package.txt")
    }
    Copy-RuntimeFile (Join-Path $VcpkgDirectory "LICENSE.txt") (Join-Path $LicensesDirectory "vcpkg-LICENSE.txt")
    $TesseractCopyright = Join-Path $InstallRoot "$($Manifest.vcpkg.triplet)/share/tesseract/copyright"
    if (-not (Test-Path -LiteralPath $TesseractCopyright -PathType Leaf)) { throw "Tesseract copyright file is missing" }
    Copy-RuntimeFile $TesseractCopyright (Join-Path $LicensesDirectory "tessdata-Apache-2.0.txt")

    $BundledFiles = @(
        Get-ChildItem -LiteralPath $BinariesDirectory, $NativeDirectory, $TessdataDirectory, $OcrModelsDirectory -Recurse -File |
            Where-Object { $_.Extension -in @(".exe", ".dll", ".traineddata", ".onnx") } |
            Sort-Object FullName |
            ForEach-Object {
                $Relative = [System.IO.Path]::GetRelativePath($RepositoryRoot, $_.FullName).Replace("\", "/")
                [ordered]@{
                    path = $Relative
                    install_path = Get-PackagedRuntimePath $_.FullName
                    packages = @(Get-RuntimePackages $_.FullName)
                    size = $_.Length
                    sha256 = (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
                }
            }
    )
    if ($BundledFiles.Count -lt 7) { throw "Runtime asset staging produced an unexpectedly small signed file set" }
    # Packaged-path collisions are rejected by verify-assets.mjs below, which
    # reads the install path as a real property and is covered by
    # verify-assets.test.ts. The check that used to live here grouped these
    # ordered hashtables by that key name; PowerShell cannot resolve a hashtable
    # key as a named property, so every file landed in one unnamed group and the
    # throw fired for any package holding more than one file. It never passed,
    # and it printed an empty collision list when it did fire.
    $LicenseFiles = @(
        Get-ChildItem -LiteralPath $LicensesDirectory -Recurse -File | Sort-Object FullName | ForEach-Object {
            $Relative = [System.IO.Path]::GetRelativePath($RepositoryRoot, $_.FullName).Replace("\", "/")
            $InstallRelative = [System.IO.Path]::GetRelativePath($LicensesDirectory, $_.FullName).Replace("\", "/")
            [ordered]@{
                path = $Relative
                install_path = "licenses/$InstallRelative"
                size = $_.Length
                sha256 = (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
            }
        }
    )
    if ($LicenseFiles.Count -lt 4) { throw "License staging produced an unexpectedly small inventory" }
    $Manifest.bundled_files = $BundledFiles
    $Manifest.license_files = $LicenseFiles
    $Manifest | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath $ManifestPath -Encoding utf8NoBOM
    & node (Join-Path $PSScriptRoot "verify-assets.mjs") --require-bundled
    if ($LASTEXITCODE -ne 0) { throw "Bundled runtime asset verification failed" }
    Write-Host "Pinned Windows runtime assets are staged and verified."
}
finally {
    if ($OwnsWorkDirectory -and -not $KeepWorkDirectory -and (Test-Path -LiteralPath $WorkDirectory)) {
        Remove-Item -LiteralPath $WorkDirectory -Recurse -Force
    }
}
