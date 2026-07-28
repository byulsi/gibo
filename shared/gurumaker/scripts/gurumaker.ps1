$ErrorActionPreference = "Stop"

$SkillDirectory = Split-Path -Parent $PSScriptRoot
$VersionFile = Join-Path $SkillDirectory "runtime/version"
$ReleaseBaseUrl = if ($env:GURUMAKER_RELEASE_BASE_URL) {
    $env:GURUMAKER_RELEASE_BASE_URL.TrimEnd("/")
} else {
    "https://github.com/monarchjuno/guru-maker/releases/download"
}

if (-not (Test-Path -LiteralPath $VersionFile -PathType Leaf)) {
    throw "Guru Maker runtime error: missing runtime version file"
}
$RuntimeVersion = (Get-Content -LiteralPath $VersionFile -Raw).Trim()
if (-not $RuntimeVersion) {
    throw "Guru Maker runtime error: runtime version is empty"
}

function Get-GuruMakerTarget {
    switch ($env:PROCESSOR_ARCHITECTURE.ToUpperInvariant()) {
        "AMD64" { return "x86_64-pc-windows-msvc" }
        default {
            throw "Guru Maker runtime error: unsupported Windows architecture: $env:PROCESSOR_ARCHITECTURE"
        }
    }
}

function Get-GuruMakerRuntimeHome {
    if ($env:GURUMAKER_RUNTIME_HOME) {
        return $env:GURUMAKER_RUNTIME_HOME
    }
    $localData = [Environment]::GetFolderPath("LocalApplicationData")
    if (-not $localData) {
        throw "Guru Maker runtime error: cannot resolve runtime home"
    }
    return (Join-Path $localData "GuruMaker/runtimes")
}

function Test-GuruMakerVersion([string]$Path) {
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        return $false
    }
    try {
        $reported = (& $Path --version 2>$null | Out-String).Trim()
        return $LASTEXITCODE -eq 0 -and (
            $reported -eq $RuntimeVersion -or $reported -eq "gurumaker $RuntimeVersion"
        )
    } catch {
        return $false
    }
}

$Target = Get-GuruMakerTarget
$RuntimeHome = Get-GuruMakerRuntimeHome
$ManagedRuntime = Join-Path $RuntimeHome "$RuntimeVersion/$Target/gurumaker.exe"

function Find-GuruMakerRuntime {
    if ($env:GURUMAKER_RUNTIME_BIN) {
        if (-not (Test-GuruMakerVersion $env:GURUMAKER_RUNTIME_BIN)) {
            throw "Guru Maker runtime error: GURUMAKER_RUNTIME_BIN is not Guru Maker runtime $RuntimeVersion"
        }
        return $env:GURUMAKER_RUNTIME_BIN
    }
    if (Test-GuruMakerVersion $ManagedRuntime) {
        return $ManagedRuntime
    }
    $pathRuntime = Get-Command "gurumaker.exe" -CommandType Application -ErrorAction SilentlyContinue
    if ($pathRuntime -and (Test-GuruMakerVersion $pathRuntime.Source)) {
        return $pathRuntime.Source
    }
    return $null
}

function Receive-GuruMakerFile([string]$Uri, [string]$Destination) {
    $parsed = [Uri]$Uri
    if ($parsed.IsFile) {
        Copy-Item -Force -LiteralPath $parsed.LocalPath -Destination $Destination
        return
    }
    Invoke-WebRequest -UseBasicParsing -Uri $Uri -OutFile $Destination
}

function Install-GuruMakerRuntime {
    $artifact = "gurumaker-$Target.exe"
    $releaseUrl = "$ReleaseBaseUrl/v$RuntimeVersion"
    $destinationDirectory = Split-Path -Parent $ManagedRuntime
    New-Item -ItemType Directory -Force -Path $destinationDirectory | Out-Null

    $nonce = "$PID-$([Guid]::NewGuid().ToString('N'))"
    $downloadPath = Join-Path $destinationDirectory ".gurumaker.download.$nonce.exe"
    $checksumsPath = Join-Path $destinationDirectory ".SHA256SUMS.$nonce"
    try {
        [Console]::Error.WriteLine(
            "Installing Guru Maker runtime $RuntimeVersion for $Target..."
        )
        Receive-GuruMakerFile "$releaseUrl/SHA256SUMS" $checksumsPath

        $expected = $null
        foreach ($line in Get-Content -LiteralPath $checksumsPath) {
            if ($line -match "^([0-9a-fA-F]{64})\s+\*?(.+)$" -and $Matches[2] -eq $artifact) {
                $expected = $Matches[1].ToLowerInvariant()
                break
            }
        }
        if (-not $expected) {
            throw "Guru Maker runtime error: SHA256SUMS does not contain $artifact"
        }

        Receive-GuruMakerFile "$releaseUrl/$artifact" $downloadPath
        $actual = (Get-FileHash -Algorithm SHA256 -LiteralPath $downloadPath).Hash.ToLowerInvariant()
        if ($actual -ne $expected) {
            throw "Guru Maker runtime error: checksum mismatch for $artifact"
        }

        if (-not (Test-GuruMakerVersion $downloadPath)) {
            throw "Guru Maker runtime error: downloaded runtime did not report version $RuntimeVersion"
        }
        Move-Item -Force -LiteralPath $downloadPath -Destination $ManagedRuntime
        [Console]::Error.WriteLine("Installed Guru Maker runtime at $ManagedRuntime")
        return $ManagedRuntime
    } finally {
        Remove-Item -Force -ErrorAction SilentlyContinue -LiteralPath $downloadPath, $checksumsPath
    }
}

if ($args.Count -gt 0 -and $args[0] -eq "--runtime-status") {
    $runtime = Find-GuruMakerRuntime
    if ($runtime) {
        Write-Output "ready`t$RuntimeVersion`t$runtime"
        exit 0
    }
    Write-Output "missing`t$RuntimeVersion`t$Target"
    exit 1
}

if ($args.Count -gt 0 -and $args[0] -eq "--runtime-install") {
    $runtime = Find-GuruMakerRuntime
    if (-not $runtime) {
        $runtime = Install-GuruMakerRuntime
    }
    Write-Output $runtime
    exit 0
}

$runtime = Find-GuruMakerRuntime
if (-not $runtime) {
    $runtime = Install-GuruMakerRuntime
}
& $runtime @args
exit $LASTEXITCODE
