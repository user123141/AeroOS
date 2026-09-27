[CmdletBinding()]
param([switch]$Portable, [switch]$Encryption)
$ErrorActionPreference = "Stop"
Write-Host "[AeroOS] Building Rust..." -ForegroundColor Cyan

$RootPath = Join-Path $PSScriptRoot ".."
$BuildPath = Join-Path $RootPath "build"
[System.IO.Directory]::CreateDirectory($BuildPath) | Out-Null

Push-Location $RootPath
try {
    Write-Host "[AeroOS] Syncing Cargo.lock..." -ForegroundColor Yellow
    & cargo generate-lockfile
    if ($LASTEXITCODE -ne 0) { throw "cargo generate-lockfile failed" }

    $feats = @()
    if ($Portable)   { $feats += "ape" }
    if ($Encryption) { $feats += "encryption" }

    $cargoArgs = @("build", "--release")
    if ($feats.Count -gt 0) { $cargoArgs += @("--features", ($feats -join ",")) }

    Write-Host "[AeroOS] cargo $($cargoArgs -join ' ')" -ForegroundColor DarkGray
    & cargo @cargoArgs
    if ($LASTEXITCODE -ne 0) {
        Write-Host "[AeroOS][ERROR] cargo build failed" -ForegroundColor Red
        throw "cargo build failed"
    }

    $B = Join-Path $RootPath "target/release/aeroos.exe"
    if (Test-Path $B) { Copy-Item $B $BuildPath -Force }
    Copy-Item (Join-Path $RootPath "config/aeroos.config.toml") $BuildPath -Force -EA SilentlyContinue
    $L = Join-Path $RootPath "kernel/output"
    if (Test-Path $L) {
        $BL = Join-Path $BuildPath "kernel"
        [System.IO.Directory]::CreateDirectory($BL) | Out-Null
        Copy-Item "$L\*" $BL -Force -EA SilentlyContinue
    }
    Write-Host "[AeroOS] Rust built." -ForegroundColor Green
} finally { Pop-Location }