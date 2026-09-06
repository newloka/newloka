# New Loka Local Release Packaging Script
# Generates ready-to-distribute release archive and SHA256 checksums.

param(
    [string]$Version = "0.1.0"
)

$ErrorActionPreference = "Stop"

$RootDir = Split-Path -Parent $PSScriptRoot
Set-Location $RootDir

Write-Host "==> Building release binaries with cargo..." -ForegroundColor Cyan
cargo build --release --workspace

$DistDir = Join-Path $RootDir "dist"
$StagingDir = Join-Path $DistDir "staging_windows_x86_64"

if (Test-Path $DistDir) {
    Remove-Item -Path $DistDir -Recurse -Force
}

New-Item -ItemType Directory -Force -Path $StagingDir | Out-Null

Write-Host "==> Staging binaries and documentation..." -ForegroundColor Cyan
Copy-Item "target\release\newloka-server.exe" -Destination $StagingDir
Copy-Item "target\release\newloka.exe" -Destination $StagingDir
Copy-Item "target\release\newloka-cli.exe" -Destination $StagingDir
Copy-Item "README.md" -Destination $StagingDir
if (Test-Path "LICENSE") {
    Copy-Item "LICENSE" -Destination $StagingDir
}

$ZipFile = Join-Path $DistDir "newloka-windows-x86_64.zip"
Write-Host "==> Compressing to $ZipFile..." -ForegroundColor Cyan
Compress-Archive -Path "$StagingDir\*" -DestinationPath $ZipFile -Force

Remove-Item -Path $StagingDir -Recurse -Force

Write-Host "==> Computing SHA256 checksum..." -ForegroundColor Cyan
$Hash = (Get-FileHash -Path $ZipFile -Algorithm SHA256).Hash.ToLower()
$ChecksumLine = "$Hash  newloka-windows-x86_64.zip"
$ChecksumFile = Join-Path $DistDir "SHA256SUMS.txt"
Set-Content -Path $ChecksumFile -Value $ChecksumLine

$SizeMB = [math]::Round(((Get-Item $ZipFile).Length / 1MB), 2)

Write-Host "`n[SUCCESS] New Loka Windows Release Package Created!" -ForegroundColor Green
Write-Host "  Package:  $ZipFile ($SizeMB MB)" -ForegroundColor White
Write-Host "  Checksum: $Hash" -ForegroundColor White
Write-Host "  Location: $DistDir`n" -ForegroundColor White
