param(
    [switch]$VerifyOnly
)

$ErrorActionPreference = "Stop"

$Root = Split-Path -Parent $MyInvocation.MyCommand.Path
$rootWasm = Join-Path $Root "star-crusher.wasm"
$targetWasm = Join-Path $Root "target\wasm32-unknown-unknown\release\star-crusher.wasm"
$buildInfoPath = Join-Path $Root "star-crusher.wasm.buildinfo.json"

$expectedMarker = "Tap a destination"
$staleMarker = "Choose a destination"

function Get-FileSha256([string]$Path) {
    return (Get-FileHash -LiteralPath $Path -Algorithm SHA256).Hash.ToLowerInvariant()
}

function Test-WasmContainsString([string]$Path, [string]$Text) {
    if (-not (Test-Path -LiteralPath $Path)) {
        return $false
    }
    $needle = [System.Text.Encoding]::UTF8.GetBytes($Text)
    $bytes = [System.IO.File]::ReadAllBytes($Path)
    if ($bytes.Length -lt $needle.Length) {
        return $false
    }
    for ($i = 0; $i -le ($bytes.Length - $needle.Length); $i++) {
        $match = $true
        for ($j = 0; $j -lt $needle.Length; $j++) {
            if ($bytes[$i + $j] -ne $needle[$j]) {
                $match = $false
                break
            }
        }
        if ($match) {
            return $true
        }
    }
    return $false
}

function Test-WasmMarkers([string]$Path) {
    if (Test-WasmContainsString $Path $staleMarker) {
        throw "Stale WASM marker found in '$Path': '$staleMarker'. Rebuild with build-wasm.ps1."
    }
    if (-not (Test-WasmContainsString $Path $expectedMarker)) {
        throw "Expected WASM marker '$expectedMarker' not found in '$Path'."
    }
}

function Write-BuildInfo([string]$Hash, [string]$SourcePath) {
    $gitSha = "unknown"
    try {
        $gitSha = (git -C $Root rev-parse HEAD 2>$null).Trim()
        if (-not $gitSha) { $gitSha = "unknown" }
    } catch {
        $gitSha = "unknown"
    }

    $info = @{
        hash = $Hash
        builtAt = (Get-Date).ToUniversalTime().ToString("o")
        gitSha = $gitSha
        sourcePath = "target/wasm32-unknown-unknown/release/star-crusher.wasm"
        rootWasm = "star-crusher.wasm"
    }
    $info | ConvertTo-Json | Set-Content -LiteralPath $buildInfoPath -Encoding UTF8
}

function Assert-RootWasmFresh() {
    if (-not (Test-Path -LiteralPath $rootWasm)) {
        throw "Missing root WASM artifact: star-crusher.wasm. Run build-wasm.ps1 first."
    }

    if (Test-Path -LiteralPath $targetWasm) {
        $rootHash = Get-FileSha256 $rootWasm
        $targetHash = Get-FileSha256 $targetWasm
        if ($rootHash -ne $targetHash) {
            throw "Root WASM hash does not match cargo output. Run build-wasm.ps1 to copy the fresh artifact."
        }
    }

    if (Test-Path -LiteralPath $buildInfoPath) {
        $info = Get-Content -LiteralPath $buildInfoPath -Raw | ConvertFrom-Json
        $rootHash = Get-FileSha256 $rootWasm
        if ($info.hash -and ($info.hash.ToLowerInvariant() -ne $rootHash)) {
            throw "star-crusher.wasm hash differs from buildinfo. Rebuild with build-wasm.ps1."
        }
    }

    Test-WasmMarkers $rootWasm
}

if ($VerifyOnly) {
    Assert-RootWasmFresh
    Write-Output "OK: star-crusher.wasm is fresh and contains '$expectedMarker'."
    exit 0
}

if ($env:CARGO_TARGET_DIR) {
    Write-Warning "Removing CARGO_TARGET_DIR so cargo writes to project target/."
}
Remove-Item Env:CARGO_TARGET_DIR -ErrorAction SilentlyContinue

Set-Location $Root

Write-Output "Building wasm32-unknown-unknown release..."
& cargo build --target wasm32-unknown-unknown --release
if ($LASTEXITCODE -ne 0) {
    throw "cargo build failed with exit code $LASTEXITCODE"
}

if (-not (Test-Path -LiteralPath $targetWasm)) {
    throw "Expected cargo output missing: target/wasm32-unknown-unknown/release/star-crusher.wasm"
}

Copy-Item -LiteralPath $targetWasm -Destination $rootWasm -Force

$rootHash = Get-FileSha256 $rootWasm
$targetHash = Get-FileSha256 $targetWasm
if ($rootHash -ne $targetHash) {
    throw "Post-copy hash mismatch between root WASM and cargo output."
}

Test-WasmMarkers $rootWasm
Write-BuildInfo $rootHash $targetWasm

Write-Output "OK: built star-crusher.wasm"
Write-Output "HASH=$rootHash"
