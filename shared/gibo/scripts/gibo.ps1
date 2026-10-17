$ErrorActionPreference = "Stop"

$SkillDirectory = Split-Path -Parent $PSScriptRoot
$VersionFile = Join-Path $SkillDirectory "runtime/version"
$ReleaseBaseUrl = if ($env:GIBO_RELEASE_BASE_URL) {
    $env:GIBO_RELEASE_BASE_URL.TrimEnd("/")
} else {
    "https://github.com/byulsi/gibo/releases/download"
}

if (-not (Test-Path -LiteralPath $VersionFile -PathType Leaf)) {
    throw "Gibo runtime error: missing runtime version file"
}
$RuntimeVersion = (Get-Content -LiteralPath $VersionFile -Raw).Trim()
if (-not $RuntimeVersion) {
    throw "Gibo runtime error: runtime version is empty"
}

function Get-GiboTarget {
    switch ($env:PROCESSOR_ARCHITECTURE.ToUpperInvariant()) {
        "AMD64" { return "x86_64-pc-windows-msvc" }
        default {
            throw "Gibo runtime error: unsupported Windows architecture: $env:PROCESSOR_ARCHITECTURE"
        }
    }
}

function Get-GiboRuntimeHome {
    if ($env:GIBO_RUNTIME_HOME) {
        return $env:GIBO_RUNTIME_HOME
    }
    $localData = [Environment]::GetFolderPath("LocalApplicationData")
    if (-not $localData) {
        throw "Gibo runtime error: cannot resolve runtime home"
    }
    return (Join-Path $localData "Gibo/runtimes")
}

function Test-GiboVersion([string]$Path) {
    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        return $false
    }
    try {
        $reported = (& $Path --version 2>$null | Out-String).Trim()
        return $LASTEXITCODE -eq 0 -and (
            $reported -eq $RuntimeVersion -or $reported -eq "gibo $RuntimeVersion"
        )
    } catch {
        return $false
    }
}

$Target = Get-GiboTarget
$RuntimeHome = Get-GiboRuntimeHome
$ManagedRuntime = Join-Path $RuntimeHome "$RuntimeVersion/$Target/gibo.exe"

function Find-GiboRuntime {
    if ($env:GIBO_RUNTIME_BIN) {
        if (-not (Test-GiboVersion $env:GIBO_RUNTIME_BIN)) {
            throw "Gibo runtime error: GIBO_RUNTIME_BIN is not Gibo runtime $RuntimeVersion"
        }
        return $env:GIBO_RUNTIME_BIN
    }
    if (Test-GiboVersion $ManagedRuntime) {
        return $ManagedRuntime
    }
    $pathRuntime = Get-Command "gibo.exe" -CommandType Application -ErrorAction SilentlyContinue
    if ($pathRuntime -and (Test-GiboVersion $pathRuntime.Source)) {
        return $pathRuntime.Source
    }
    return $null
}

function Receive-GiboFile([string]$Uri, [string]$Destination) {
    $parsed = [Uri]$Uri
    if ($parsed.IsFile) {
        Copy-Item -Force -LiteralPath $parsed.LocalPath -Destination $Destination
        return
    }
    Invoke-WebRequest -UseBasicParsing -Uri $Uri -OutFile $Destination
}

function Install-GiboRuntime {
    $artifact = "gibo-$Target.exe"
    $releaseUrl = "$ReleaseBaseUrl/v$RuntimeVersion"
    $destinationDirectory = Split-Path -Parent $ManagedRuntime
    New-Item -ItemType Directory -Force -Path $destinationDirectory | Out-Null

    $nonce = "$PID-$([Guid]::NewGuid().ToString('N'))"
    $downloadPath = Join-Path $destinationDirectory ".gibo.download.$nonce.exe"
    $checksumsPath = Join-Path $destinationDirectory ".SHA256SUMS.$nonce"
    try {
        [Console]::Error.WriteLine(
            "Installing Gibo runtime $RuntimeVersion for $Target..."
        )
        Receive-GiboFile "$releaseUrl/SHA256SUMS" $checksumsPath

        $expected = $null
        foreach ($line in Get-Content -LiteralPath $checksumsPath) {
            if ($line -match "^([0-9a-fA-F]{64})\s+\*?(.+)$" -and $Matches[2] -eq $artifact) {
                $expected = $Matches[1].ToLowerInvariant()
                break
            }
        }
        if (-not $expected) {
            throw "Gibo runtime error: SHA256SUMS does not contain $artifact"
        }

        Receive-GiboFile "$releaseUrl/$artifact" $downloadPath
        $actual = (Get-FileHash -Algorithm SHA256 -LiteralPath $downloadPath).Hash.ToLowerInvariant()
        if ($actual -ne $expected) {
            throw "Gibo runtime error: checksum mismatch for $artifact"
        }

        if (-not (Test-GiboVersion $downloadPath)) {
            throw "Gibo runtime error: downloaded runtime did not report version $RuntimeVersion"
        }
        Move-Item -Force -LiteralPath $downloadPath -Destination $ManagedRuntime
        [Console]::Error.WriteLine("Installed Gibo runtime at $ManagedRuntime")
        return $ManagedRuntime
    } finally {
        Remove-Item -Force -ErrorAction SilentlyContinue -LiteralPath $downloadPath, $checksumsPath
    }
}

if ($args.Count -gt 0 -and $args[0] -eq "--runtime-status") {
    $runtime = Find-GiboRuntime
    if ($runtime) {
        Write-Output "ready`t$RuntimeVersion`t$runtime"
        exit 0
    }
    Write-Output "missing`t$RuntimeVersion`t$Target"
    exit 1
}

if ($args.Count -gt 0 -and $args[0] -eq "--runtime-install") {
    $runtime = Find-GiboRuntime
    if (-not $runtime) {
        $runtime = Install-GiboRuntime
    }
    Write-Output $runtime
    exit 0
}

$runtime = Find-GiboRuntime
if (-not $runtime) {
    $runtime = Install-GiboRuntime
}
& $runtime @args
exit $LASTEXITCODE
