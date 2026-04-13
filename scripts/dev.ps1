# dev.ps1 — Memory Stream dev launcher
# Usage: scripts\dev.ps1  |  Ctrl+C to stop all

$ErrorActionPreference = "SilentlyContinue"
$root = Split-Path -Parent $PSScriptRoot
$pidFile = Join-Path $root ".dev-pids"

Write-Host "[*] Starting Memory Stream development..." -ForegroundColor Cyan
Write-Host "    Go Server:   http://localhost:8080"
Write-Host "    Web Reader:  http://localhost:5173"
Write-Host "    Tauri App:   launching..."
Write-Host "    Ctrl+C to graceful shutdown"
Write-Host ""

Remove-Item -Force $pidFile -ErrorAction SilentlyContinue

# Start services in separate console windows
$p1 = Start-Process "cmd.exe" -ArgumentList "/c", "cd /d `"$root\go-server`" && go run ./cmd/api && pause" -PassThru
$p2 = Start-Process "cmd.exe" -ArgumentList "/c", "cd /d `"$root\frontend-workspace\apps\web-reader`" && pnpm dev && pause" -PassThru
$p3 = Start-Process "cmd.exe" -ArgumentList "/c", "cd /d `"$root\frontend-workspace\apps\admin-tauri`" && pnpm tauri dev && pause" -PassThru

$procs = @($p1, $p2, $p3)
$procs | ForEach-Object { "$($_.Id)" | Out-File $pidFile -Encoding utf8 -Append }

Write-Host "[+] 3 services launched in separate windows" -ForegroundColor Green
Write-Host "    use 'make stop' to kill them" -ForegroundColor DarkGray
