[CmdletBinding()]
param(
    [string]$KernelUrl = "",
    [ValidateSet("auto","url","prebuilt","cross","none")]
    [string]$Source = "auto"
)
$ErrorActionPreference = "Stop"
Write-Host "[AeroOS] Kernel (source=$Source)..." -ForegroundColor Cyan

$KernelPath = Join-Path $PSScriptRoot "..\kernel"
$OutputPath = Join-Path $KernelPath "output"
[System.IO.Directory]::CreateDirectory($OutputPath) | Out-Null

$k = Join-Path $OutputPath "aeroos-kernel"
$i = Join-Path $OutputPath "aeroos-initramfs"
if ((Test-Path $k) -and (Test-Path $i)) {
    Write-Host "[AeroOS] Kernel already present." -ForegroundColor Green
    exit 0
}

if ($Source -eq "none") {
    Write-Host "[AeroOS] Kernel skipped (-KernelSource none)." -ForegroundColor Yellow
    exit 0
}

if ($Source -eq "url" -or ($Source -eq "auto" -and $KernelUrl)) {
    if (-not $KernelUrl) {
        Write-Host "[AeroOS] URL not provided." -ForegroundColor Yellow
    } else {
        try {
            Write-Host "[AeroOS] Downloading $KernelUrl" -ForegroundColor Yellow
            $tmp = Join-Path $env:TEMP "aeroos-kernel.tar.gz"
            Invoke-WebRequest -Uri $KernelUrl -OutFile $tmp -UseBasicParsing
            $ex = Join-Path $env:TEMP "aeroos-k-extract"
            if (Test-Path $ex) { Remove-Item $ex -Recurse -Force }
            [System.IO.Directory]::CreateDirectory($ex) | Out-Null
            tar -xzf $tmp -C $ex
            Get-ChildItem $ex -Recurse -File | ForEach-Object {
                Copy-Item $_.FullName $OutputPath -Force
            }
            Write-Host "[AeroOS] Kernel downloaded." -ForegroundColor Green
            exit 0
        } catch {
            Write-Host "[AeroOS] Download failed: $_" -ForegroundColor Yellow
        }
    }
}

Write-Host ""
Write-Host "============================================" -ForegroundColor Yellow
Write-Host "Kernel NOT built." -ForegroundColor Yellow
Write-Host "Structure is ready. To get a kernel:" -ForegroundColor Yellow
Write-Host "  1. -KernelUrl <url>       (download)" -ForegroundColor Yellow
Write-Host "  2. -KernelSource prebuilt (search github)" -ForegroundColor Yellow
Write-Host "  3. -KernelSource cross    (MSYS2)" -ForegroundColor Yellow
Write-Host "  4. -KernelSource none     (skip, .exe still builds)" -ForegroundColor Yellow
Write-Host "  5. Place manually in kernel/output/" -ForegroundColor Yellow
Write-Host "============================================" -ForegroundColor Yellow
exit 0