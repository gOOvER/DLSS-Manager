# DLSS Manager (Rust + Tauri v2) - Windows Installer Build Script
# Usage: pwsh build_rust_installer.ps1

$ErrorActionPreference = 'Stop'

# Ensure Cargo bin directory is on PATH
$cargoBin = Join-Path $env:USERPROFILE ".cargo\bin"
if (-not (Test-Path $cargoBin)) {
    New-Item -ItemType Directory -Force -Path $cargoBin | Out-Null
}
if ($env:PATH -notlike "*$cargoBin*") {
    $env:PATH = "$cargoBin;$env:PATH"
}

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "cargo not found in $cargoBin - installing Rust toolchain automatically via rustup-init..." -ForegroundColor Yellow
    $rustupExe = Join-Path $env:TEMP "rustup-init.exe"
    Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile $rustupExe
    & $rustupExe -y --default-toolchain stable --profile minimal
    if ($LASTEXITCODE -ne 0) {
        Write-Host "ERROR: rustup-init failed with exit code $LASTEXITCODE" -ForegroundColor Red
        exit $LASTEXITCODE
    }
}

Write-Host "`n=== Rust Toolchain Ready ===" -ForegroundColor Green
rustc --version
cargo --version

Write-Host "`n=== Checking tauri-cli ===" -ForegroundColor Cyan
if (-not (Get-Command cargo-tauri -ErrorAction SilentlyContinue)) {
    Write-Host "Downloading prebuilt cargo-tauri v2 binary..." -ForegroundColor Yellow
    $tauriZip = Join-Path $env:TEMP "cargo-tauri.zip"
    try {
        Invoke-WebRequest -Uri "https://github.com/tauri-apps/tauri/releases/latest/download/cargo-tauri-x86_64-pc-windows-msvc.zip" -OutFile $tauriZip
        Expand-Archive -Path $tauriZip -DestinationPath $cargoBin -Force
        Remove-Item -Path $tauriZip -Force -ErrorAction SilentlyContinue
        Write-Host "Installed prebuilt cargo-tauri.exe into $cargoBin" -ForegroundColor Green
    } catch {
        Write-Host "Prebuilt binary download failed, falling back to cargo install..." -ForegroundColor Yellow
        cargo install tauri-cli --version "^2.0.0" --locked
    }
}

Write-Host "`n=== Building DLSS Manager (Rust Release + MSI/NSIS Installer) ===" -ForegroundColor Cyan
Get-Process -Name "dlss-manager" -ErrorAction SilentlyContinue | Stop-Process -Force -ErrorAction SilentlyContinue
Start-Sleep -Milliseconds 300
Push-Location (Join-Path $PSScriptRoot "src-tauri")
try {
    cargo tauri build
    if ($LASTEXITCODE -ne 0) {
        Write-Host "cargo tauri build exited with code $LASTEXITCODE" -ForegroundColor Red
        exit $LASTEXITCODE
    }
} finally {
    Pop-Location
}

Write-Host "`n=== Build Complete! ===" -ForegroundColor Green
Get-ChildItem -Path "src-tauri\target\release\bundle" -Include "*.msi", "*.exe" -Recurse |
    ForEach-Object {
        Write-Host ("  Installer: {0} ({1:N2} MB)" -f $_.FullName, ($_.Length / 1MB)) -ForegroundColor Green
    }
