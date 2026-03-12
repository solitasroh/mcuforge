#!/usr/bin/env pwsh
# mcuforge installer for Windows
# Usage: irm https://raw.githubusercontent.com/solitasroh/mcuforge/main/install.ps1 | iex

$ErrorActionPreference = "Stop"

$repo = "solitasroh/mcuforge"
$installDir = "$env:USERPROFILE\.embtool\bin"

Write-Host "mcuforge installer" -ForegroundColor Cyan
Write-Host ""

# Get latest release
Write-Host "  Fetching latest release..." -ForegroundColor DarkGray
$release = Invoke-RestMethod "https://api.github.com/repos/$repo/releases/latest"
$tag = $release.tag_name
$version = $tag -replace '^v', ''

Write-Host "  Latest version: $tag" -ForegroundColor Green

# Find Windows binary asset
$asset = $release.assets | Where-Object { $_.name -match "windows" -and $_.name -match "\.exe$" }
if (-not $asset) {
    Write-Error "No Windows binary found in release $tag"
    exit 1
}

# Create install directory
if (-not (Test-Path $installDir)) {
    New-Item -ItemType Directory -Path $installDir -Force | Out-Null
}

$dest = Join-Path $installDir "mcuforge.exe"

# Download
Write-Host "  Downloading $($asset.name) ($([math]::Round($asset.size / 1MB, 1)) MB)..." -ForegroundColor DarkGray
Invoke-WebRequest -Uri $asset.browser_download_url -OutFile $dest

Write-Host "  Installed to: $dest" -ForegroundColor Green

# Check PATH
$userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($userPath -notlike "*$installDir*") {
    Write-Host ""
    Write-Host "  Adding $installDir to PATH..." -ForegroundColor Yellow
    [Environment]::SetEnvironmentVariable("PATH", "$userPath;$installDir", "User")
    $env:PATH = "$env:PATH;$installDir"
    Write-Host "  PATH updated. Restart terminal to take effect." -ForegroundColor Yellow
}

Write-Host ""
Write-Host "  mcuforge $version installed successfully!" -ForegroundColor Green
Write-Host ""
Write-Host "  Quick start:" -ForegroundColor Cyan
Write-Host "    mcuforge --version" -ForegroundColor White
Write-Host "    mcuforge new my-project --mcu k64 --claude" -ForegroundColor White
Write-Host "    mcuforge claude install" -ForegroundColor White
