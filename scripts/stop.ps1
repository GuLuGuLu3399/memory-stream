# stop.ps1 — 强制停止残留 dev 进程

$root = Split-Path -Parent $PSScriptRoot
$pidFile = Join-Path $root ".dev-pids"

if (Test-Path $pidFile) {
    Write-Host "[!] Killing stray dev processes..." -ForegroundColor Yellow
    Get-Content $pidFile | ForEach-Object {
        Stop-Process -Id $_ -Force -ErrorAction SilentlyContinue
    }
    Remove-Item -Force $pidFile -ErrorAction SilentlyContinue
    Write-Host "[+] Done" -ForegroundColor Green
} else {
    Write-Host "[i]  No dev PID file found"
}
