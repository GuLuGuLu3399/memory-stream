# scripts/dist.ps1 — Build distributable artifacts (Linux server + Web SPA + Tauri desktop)
$ErrorActionPreference = "Stop"

$root = $PSScriptRoot | Split-Path
Set-Location $root

$distDir = "dist"
$goDistDir = "$distDir/go"

# ── Linux server binary ──────────────────────────────────────────────
if (-not (Test-Path $distDir)) { New-Item -ItemType Directory -Path $distDir | Out-Null }
if (-not (Test-Path $goDistDir)) { New-Item -ItemType Directory -Path $goDistDir | Out-Null }

$env:GOOS = "linux"
$env:GOARCH = "amd64"
$env:CGO_ENABLED = "0"

Set-Location go-server
go build -ldflags="-s -w" -o "../$goDistDir/server" ./cmd/api
Set-Location ..

if (Test-Path "go-server/.env.production") {
    Copy-Item "go-server/.env.production" "$goDistDir/.env.example" -Force
    Write-Host "[+] Config template -> $goDistDir/.env.example"
}
Write-Host "[+] Linux binary -> $goDistDir/server"

# ── Web SPA ──────────────────────────────────────────────────────────
Set-Location frontend-workspace/apps/web-reader
npx vite build --mode production
Set-Location $root

if (-not (Test-Path "$distDir/web")) { New-Item -ItemType Directory -Path "$distDir/web" | Out-Null }
Get-ChildItem "$distDir/web" -Recurse | Remove-Item -Force -Recurse -ErrorAction SilentlyContinue
Copy-Item "frontend-workspace/apps/web-reader/dist/*" "$distDir/web/" -Recurse -Force
Write-Host "[+] Web SPA -> $distDir/web/"

# ── Tauri desktop bundle ─────────────────────────────────────────────
Set-Location frontend-workspace/apps/admin-tauri
pnpm tauri build
Set-Location $root

$tauriBundleDir = "frontend-workspace/apps/admin-tauri/src-tauri/target/release/bundle"
if (Test-Path $tauriBundleDir) {
    if (-not (Test-Path "$distDir/tauri")) { New-Item -ItemType Directory -Path "$distDir/tauri" | Out-Null }
    Get-ChildItem "$distDir/tauri" -Recurse | Remove-Item -Force -Recurse -ErrorAction SilentlyContinue
    Copy-Item "$tauriBundleDir/*" "$distDir/tauri/" -Recurse -Force
    Write-Host "[+] Tauri bundle -> $distDir/tauri/"
}
else {
    Write-Host "[!] Tauri bundle directory not found: $tauriBundleDir"
}
