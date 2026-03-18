#Requires -Version 5.1
<#
.SYNOPSIS
    Installs ruv on Windows.
.DESCRIPTION
    Downloads and installs the ruv binary from GitHub Releases.
.PARAMETER Tag
    The release tag to install (default: latest).
.PARAMETER InstallDir
    The directory to install ruv into (default: %LOCALAPPDATA%\ruv\bin).
.EXAMPLE
    irm https://github.com/A-Fisk/ruv/releases/download/TAG/install.ps1 | iex
#>
param(
    [string]$Tag = "latest",
    [string]$InstallDir = ""
)

$ErrorActionPreference = "Stop"

$Repo = "A-Fisk/ruv"

if ($Tag -eq "latest") {
    Write-Host "Fetching latest release..."
    $releases = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases"
    $Tag = $releases[0].tag_name
    if (-not $Tag) {
        Write-Error "Error: Could not determine latest release tag"
        exit 1
    }
}

Write-Host "Installing ruv $Tag..."

# Only x86_64 is supported on Windows
$Target = "x86_64-pc-windows-msvc"

# Determine install directory
if (-not $InstallDir) {
    $InstallDir = Join-Path $env:LOCALAPPDATA "ruv\bin"
}
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null

# Download zip
$Url = "https://github.com/$Repo/releases/download/$Tag/ruv-$Target.zip"
Write-Host "Downloading from: $Url"

$TempDir = Join-Path $env:TEMP "ruv-install-$(Get-Random)"
New-Item -ItemType Directory -Force -Path $TempDir | Out-Null

try {
    $ZipPath = Join-Path $TempDir "ruv.zip"
    Invoke-WebRequest -Uri $Url -OutFile $ZipPath -UseBasicParsing

    Write-Host "Extracting..."
    Expand-Archive -Path $ZipPath -DestinationPath $TempDir -Force

    $BinaryPath = Join-Path $TempDir "ruv-$Target\bin\ruv.exe"
    if (-not (Test-Path $BinaryPath)) {
        Write-Error "Error: Binary not found at expected location: $BinaryPath"
        exit 1
    }

    Copy-Item $BinaryPath (Join-Path $InstallDir "ruv.exe") -Force
} finally {
    Remove-Item -Recurse -Force $TempDir -ErrorAction SilentlyContinue
}

Write-Host ""
Write-Host "ruv $Tag installed successfully!"
Write-Host ""
Write-Host "Binary location: $InstallDir\ruv.exe"
Write-Host ""

# Check if install dir is in PATH
$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
$PathDirs = $UserPath -split ";"
if ($PathDirs -contains $InstallDir) {
    Write-Host "$InstallDir is already in your PATH"
    Write-Host ""
    Write-Host "You can now run: ruv --help"
} else {
    Write-Host "WARNING: $InstallDir is NOT in your PATH"
    Write-Host ""
    Write-Host "Run the following command to add it to your user PATH:"
    Write-Host ""
    Write-Host "  [Environment]::SetEnvironmentVariable('Path', `$env:Path + ';$InstallDir', 'User')"
    Write-Host ""
    Write-Host "Then restart your terminal."
}
