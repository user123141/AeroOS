[CmdletBinding()]
param(
    [switch]$SkipKernel, [switch]$SkipNative,
    [switch]$Portable, [switch]$Encryption, [switch]$SelfTest,
    [string]$KernelUrl = "",
    [ValidateSet("auto","url","prebuilt","cross","none")]
    [string]$KernelSource = "auto"
)
$ErrorActionPreference = "Stop"
$S = $PSScriptRoot
$Root = Join-Path $S ".."

Write-Host "============================================" -ForegroundColor Magenta
Write-Host "   AeroOS Build System v1.3" -ForegroundColor Magenta
Write-Host "============================================" -ForegroundColor Magenta

if ($SelfTest) {
    $req = @("Cargo.toml","build.rs","src/main.rs","src/config.rs","src/security.rs",
             "src/hypervisor.rs","src/virtio.rs","src/snapshot.rs","src/ipc.rs","src/web.rs",
             "src/data_folder.rs","src/crypto.rs","src/boot.rs","src/terminal.rs","src/tap.rs",
             "vendor/whpx/Cargo.toml","vendor/whpx/src/lib.rs",
             "vendor/conpty/Cargo.toml","vendor/conpty/src/lib.rs",
             "web/index.html","web/css/style.css","web/js/main.js",
             "config/aeroos.config.toml",
             "kernel/aeroos_defconfig","kernel/virtio.fragment","kernel/aeroos.fragment",
             "scripts/build-all.ps1","scripts/build-kernel.ps1","scripts/build-native.ps1",
             "README.md",".gitignore")
    $miss = @()
    foreach ($f in $req) { if (-not (Test-Path (Join-Path $Root $f))) { $miss += $f } }
    if ($miss.Count -gt 0) {
        Write-Host "FAILED. Missing:" -ForegroundColor Red
        $miss | ForEach-Object { Write-Host "  - $_" -ForegroundColor Red }
        exit 1
    }
    Write-Host "Self-test PASSED ($($req.Count) files)." -ForegroundColor Green
    exit 0
}

if (-not $SkipKernel) {
    $p = @{ Source = $KernelSource }
    if ($KernelUrl) { $p.KernelUrl = $KernelUrl }
    & (Join-Path $S "build-kernel.ps1") @p
}
if (-not $SkipNative) {
    $p = @{}
    if ($Portable)   { $p.Portable = $true }
    if ($Encryption) { $p.Encryption = $true }
    & (Join-Path $S "build-native.ps1") @p
    if ($LASTEXITCODE -ne 0) { throw "Native build failed" }
}

# --- Structure dump (ASCII, PS 5.1 safe) ---
Write-Host ""
Write-Host "============================================" -ForegroundColor Magenta
Write-Host "   AeroOS STRUCTURE" -ForegroundColor Magenta
Write-Host "============================================" -ForegroundColor Magenta
$exclude = @('target','buildroot','.git','.vs','node_modules','snapshots')
function Walk($p, $prefix) {
    $items = Get-ChildItem -Path $p -Force -EA SilentlyContinue |
             Where-Object { $_.Name -notin $exclude } |
             Sort-Object @{E={-not $_.PSIsContainer}}, Name
    for ($i = 0; $i -lt $items.Count; $i++) {
        $item = $items[$i]
        $isLast = ($i -eq $items.Count - 1)
        $conn = if ($isLast) { "|-- " } else { "|-- " }
        $suf = if ($item.PSIsContainer) { "/" } else { "" }
        Write-Host "$prefix$conn$($item.Name)$suf" -ForegroundColor Gray
        if ($item.PSIsContainer) {
            $np = $prefix + "|   "
            Walk $item.FullName $np
        }
    }
}
Write-Host "AeroOS/" -ForegroundColor White
Walk $Root ""

Write-Host ""
Write-Host "============================================" -ForegroundColor Green
Write-Host "   AeroOS build completed!" -ForegroundColor Green
Write-Host "============================================" -ForegroundColor Green
Write-Host "Result: $Root\build" -ForegroundColor Cyan