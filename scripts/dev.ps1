$ErrorActionPreference = "Stop"

$RootDir = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)

Write-Host "==> Starting Memory Stream dev environment..." -ForegroundColor Cyan
Write-Host "    Tauri  (Vite + Rust)"
Write-Host "    Go     (REST API on :8080)"
Write-Host ""

$goProc = Start-Process -FilePath "go" -ArgumentList "run", "./cmd/api" -WorkingDirectory "$RootDir\go-server" -PassThru -NoNewWindow
$tauriProc = Start-Process -FilePath "pnpm" -ArgumentList "--filter", "admin-tauri", "tauri", "dev" -WorkingDirectory "$RootDir\frontend-workspace" -PassThru -NoNewWindow

Write-Host "==> Go PID=$($goProc.Id), Tauri PID=$($tauriProc.Id)" -ForegroundColor Green
Write-Host "==> Press Ctrl+C to stop." -ForegroundColor Yellow
Write-Host ""

try {
    Wait-Process -Id $goProc.Id, $tauriProc.Id -Any
} finally {
    Write-Host ""
    Write-Host "==> Stopping all processes..." -ForegroundColor Yellow
    Stop-Process -Id $goProc.Id -ErrorAction SilentlyContinue
    Stop-Process -Id $tauriProc.Id -ErrorAction SilentlyContinue
    Write-Host "==> Done." -ForegroundColor Green
}
