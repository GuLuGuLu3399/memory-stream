# dev.ps1 — Memory Stream dev launcher (single terminal mode)
# Usage:
#   scripts\dev.ps1                # start all services with multiplexed logs
#   scripts\dev.ps1 -DryRun        # validate environment, print plan only
#   scripts\dev.ps1 -ForceStopOld  # stop old .dev-pids before launching

param(
    [switch] $DryRun,
    [switch] $ForceStopOld
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
$pidFile = Join-Path $root ".dev-pids"
$logDir = Join-Path $root ".dev-logs"

# 主进程输出切到 UTF-8，减少乱码（失败时忽略，不影响后续流程）
$utf8NoBom = New-Object System.Text.UTF8Encoding($false)
[Console]::InputEncoding = $utf8NoBom 2>$null
[Console]::OutputEncoding = $utf8NoBom 2>$null
$OutputEncoding = $utf8NoBom
chcp 65001 > $null 2>&1

function Assert-CommandExists {
    param([Parameter(Mandatory = $true)] [string] $Name)
    if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
        throw "Required command not found: $Name"
    }
}

function Get-PortOwners {
    param([int[]] $Ports)
    Get-NetTCPConnection -State Listen -ErrorAction SilentlyContinue |
    Where-Object { $Ports -contains $_.LocalPort } |
    Select-Object LocalPort, OwningProcess -Unique |
    Sort-Object LocalPort
}

function Stop-ProcessTree {
    param([int] $ProcessId)
    if ($ProcessId -le 0) { return }
    taskkill /PID $ProcessId /T /F | Out-Null
}

function Stop-OldPids {
    if (-not (Test-Path $pidFile)) { return }
    Write-Host "[i] Found existing .dev-pids, stopping old processes..." -ForegroundColor Yellow
    Get-Content $pidFile | ForEach-Object {
        if ($_ -match '^\d+$') {
            Stop-ProcessTree -ProcessId ([int]$_)
        }
    }
    Remove-Item -Force $pidFile -ErrorAction SilentlyContinue
}

function New-LogFile {
    param([string] $Name)
    if (-not (Test-Path $logDir)) {
        New-Item -ItemType Directory -Path $logDir | Out-Null
    }
    $file = Join-Path $logDir ("{0}.log" -f $Name.ToLower())
    Set-Content -Path $file -Value "" -Encoding UTF8
    return $file
}

function Start-DevService {
    param(
        [Parameter(Mandatory = $true)] [string] $Name,
        [Parameter(Mandatory = $true)] [string] $WorkingDir,
        [Parameter(Mandatory = $true)] [string] $Command
    )

    $logFile = New-LogFile -Name $Name
    $script = @"
`$env:FORCE_COLOR='0'
`$env:NO_COLOR='1'
[Console]::OutputEncoding=[System.Text.UTF8Encoding]::new(`$false)
`$OutputEncoding=[System.Text.UTF8Encoding]::new(`$false)
chcp 65001 > `$null
Set-Location -LiteralPath '$WorkingDir'
& { $Command } 2>&1
"@

    $proc = Start-Process "powershell.exe" `
        -ArgumentList "-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", $script `
        -WindowStyle Hidden `
        -RedirectStandardOutput $logFile `
        -PassThru

    [PSCustomObject]@{
        Name       = $Name
        WorkingDir = $WorkingDir
        Command    = $Command
        Process    = $proc
        LogFile    = $logFile
        LastLine   = 0
    }
}

function Pump-Logs {
    param([object[]] $Services)
    foreach ($svc in $Services) {
        if (-not (Test-Path $svc.LogFile)) { continue }
        $all = Get-Content -Path $svc.LogFile -ErrorAction SilentlyContinue
        if (-not $all) { continue }
        $newCount = $all.Count - $svc.LastLine
        if ($newCount -le 0) { continue }
        $newLines = $all | Select-Object -Skip $svc.LastLine
        foreach ($line in $newLines) {
            Write-Host ("[{0}] {1}" -f $svc.Name, $line)
        }
        $svc.LastLine = $all.Count
    }
}

function Cleanup-Children {
    param([object[]] $Services)
    foreach ($svc in $Services) {
        if ($null -ne $svc.Process) {
            Stop-ProcessTree -ProcessId $svc.Process.Id
        }
    }
    Remove-Item -Force $pidFile -ErrorAction SilentlyContinue
    Remove-Item -Force (Join-Path $logDir "*.log") -ErrorAction SilentlyContinue
}

$goDir = Join-Path $root "go-server"
$webDir = Join-Path $root "frontend-workspace\apps\web-reader"
$tauriDir = Join-Path $root "frontend-workspace\apps\admin-tauri"

$serviceDefs = @(
    @{ Name = "GO"; WorkingDir = $goDir; Command = "go run ./cmd/api" },
    @{ Name = "WEB"; WorkingDir = $webDir; Command = "pnpm dev" },
    @{ Name = "TAURI"; WorkingDir = $tauriDir; Command = "pnpm tauri dev" }
)

Write-Host "[*] Starting Memory Stream development (single terminal logs)..." -ForegroundColor Cyan
Write-Host "    Go Server:   http://localhost:8080"
Write-Host "    Web Reader:  http://localhost:5173"
Write-Host "    Tauri App:   launching..."
Write-Host ""

Assert-CommandExists -Name "go"
Assert-CommandExists -Name "pnpm"
Assert-CommandExists -Name "powershell"

if ($ForceStopOld) {
    Stop-OldPids
}
else {
    Remove-Item -Force $pidFile -ErrorAction SilentlyContinue
}

$portOwners = Get-PortOwners -Ports @(8080, 5173, 1420)
if ($portOwners) {
    Write-Host "[!] Detected listening ports before launch:" -ForegroundColor Yellow
    $portOwners | ForEach-Object {
        Write-Host ("    port={0}, pid={1}" -f $_.LocalPort, $_.OwningProcess) -ForegroundColor Yellow
    }
    Write-Host "[i] If this is from a previous run, execute: make stop" -ForegroundColor DarkGray
    Write-Host ""
}

if ($DryRun) {
    Write-Host "[dry-run] launch plan:" -ForegroundColor Cyan
    $serviceDefs | ForEach-Object {
        Write-Host ("  [{0}] cd {1}; {2}" -f $_.Name, $_.WorkingDir, $_.Command)
    }
    exit 0
}

$services = @()
try {
    foreach ($def in $serviceDefs) {
        $svc = Start-DevService -Name $def.Name -WorkingDir $def.WorkingDir -Command $def.Command
        $services += $svc
        Write-Host "[+] Started $($svc.Name) (PID=$($svc.Process.Id))" -ForegroundColor Green
    }

    $services | ForEach-Object { "$($_.Process.Id)" | Out-File $pidFile -Encoding utf8 -Append }

    Write-Host ""
    Write-Host "[i] Logs are multiplexed in this terminal." -ForegroundColor DarkGray
    Write-Host "[i] Ctrl+C will stop all services started by this launcher." -ForegroundColor DarkGray
    Write-Host ""

    while ($true) {
        Pump-Logs -Services $services

        $exited = $services | Where-Object { $_.Process.HasExited }
        if ($exited.Count -gt 0) {
            foreach ($svc in $exited) {
                Write-Host "[!] Process exited: $($svc.Name) PID=$($svc.Process.Id), Code=$($svc.Process.ExitCode)" -ForegroundColor Yellow
            }
            break
        }

        Start-Sleep -Milliseconds 300
    }
}
finally {
    Cleanup-Children -Services $services
}
