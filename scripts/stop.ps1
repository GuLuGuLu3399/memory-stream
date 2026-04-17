# stop.ps1 — 强制停止残留 dev 进程（含进程树）

$root = Split-Path -Parent $PSScriptRoot
$pidFile = Join-Path $root ".dev-pids"
$killed = 0

function Stop-ProcessTree {
    param([int] $ProcessId)
    if ($ProcessId -le 0) { return }
    taskkill /PID $ProcessId /T /F | Out-Null
}

Write-Host "[!] Stopping dev processes..." -ForegroundColor Yellow

if (Test-Path $pidFile) {
    Get-Content $pidFile | ForEach-Object {
        if ($_ -match '^\d+$') {
            Stop-ProcessTree -ProcessId ([int]$_)
            $killed++
        }
    }
    Remove-Item -Force $pidFile -ErrorAction SilentlyContinue
}

# 兜底：处理 pid 文件缺失但残留的常见 dev 进程
$patterns = @(
    'go run ./cmd/api',
    'pnpm dev',
    'pnpm tauri dev'
)

Get-CimInstance Win32_Process -ErrorAction SilentlyContinue |
Where-Object {
    if ($_.Name -notmatch 'powershell|node|go|cargo') { return $false }
    $cmd = $_.CommandLine
    if ([string]::IsNullOrWhiteSpace($cmd)) { return $false }
    return ($patterns | Where-Object { $cmd -like "*$_*" }).Count -gt 0
} |
ForEach-Object {
    Stop-ProcessTree -ProcessId $_.ProcessId
    $killed++
}

if ($killed -gt 0) {
    Write-Host "[+] Stopped processes: $killed" -ForegroundColor Green
}
else {
    Write-Host "[i] No matching dev processes found" -ForegroundColor DarkGray
}
