# =============================================================================
# LITRIC OS — QEMU Launcher (Windows Host)
# Launches Litric in QEMU with WHPX acceleration
#
# Prerequisites:
#   - QEMU for Windows (https://qemu.weilnetz.de/w64/)
#   - OVMF_CODE_4M.fd and OVMF_VARS_4M.fd in build/
#   - litric.qcow2 in build/
#
# Usage:
#   .\tools\run-qemu.ps1                    # Normal boot
#   .\tools\run-qemu.ps1 -Debug             # GDB stub (frozen at start)
#   .\tools\run-qemu.ps1 -Tcg               # TCG fallback (WHPX SMP bug #3178)
#   .\tools\run-qemu.ps1 -Smp 1             # Single core
#   .\tools\run-qemu.ps1 -Memory 8G         # 8 GB RAM
#
# SAFETY: Never touches host physical disks. All paths relative to project dir.
# =============================================================================

param(
    [switch]$Debug,
    [switch]$Tcg,
    [int]$Smp = 4,
    [string]$Memory = "4G",
    [string]$Image = "",
    [int]$SshPort = 2222
)

$ErrorActionPreference = "Stop"

# =============================================================================
# SETUP
# =============================================================================

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path -Parent $ScriptDir
$BuildDir = Join-Path $ProjectRoot "build"
$LogsDir = Join-Path $ProjectRoot "tools\logs"

# Verify project root
if (!(Test-Path (Join-Path $ProjectRoot "00-MASTER-WORKFLOW.md"))) {
    Write-Error "Not running from project root. Expected 00-MASTER-WORKFLOW.md at $ProjectRoot"
    exit 1
}

# Image path
if ($Image -eq "") {
    $Image = Join-Path $BuildDir "litric.qcow2"
}

# Verify image exists (safety: must be inside project dir)
$ResolvedImage = Resolve-Path $Image -ErrorAction SilentlyContinue
if ($ResolvedImage -eq $null) {
    Write-Error "Image not found: $Image`nRun build-image.sh first to create the image."
    exit 1
}
if (!$ResolvedImage.Path.StartsWith($ProjectRoot)) {
    Write-Error "SAFETY VIOLATION: Image path must be inside project directory!`nImage: $($ResolvedImage.Path)`nProject: $ProjectRoot"
    exit 1
}

# OVMF firmware
$OvmfCode = Join-Path $BuildDir "OVMF_CODE_4M.fd"
$OvmfVars = Join-Path $BuildDir "OVMF_VARS_4M.fd"

if (!(Test-Path $OvmfCode) -or !(Test-Path $OvmfVars)) {
    Write-Error "OVMF firmware not found.`nDownload OVMF_CODE_4M.fd and OVMF_VARS_4M.fd to $BuildDir\"
    exit 1
}

# Create logs directory
New-Item -ItemType Directory -Force -Path $LogsDir | Out-Null

$Timestamp = Get-Date -Format "yyyyMMdd-HHmmss"
$LogFile = Join-Path $LogsDir "boot-$Timestamp.log"

# =============================================================================
# QEMU COMMAND
# =============================================================================

# Base QEMU arguments
$QemuArgs = @(
    "-machine", "q35,accel=whpx,kernel-irqchip=off"
    "-cpu", "max"
    "-smp", "$Smp"
    "-m", "$Memory"
    
    # Firmware (4 MB split OVMF)
    "-drive", "if=pflash,format=raw,unit=0,readonly=on,file=$OvmfCode"
    "-drive", "if=pflash,format=raw,unit=1,file=$OvmfVars"
    
    # Disk (virtio)
    "-drive", "if=virtio,format=qcow2,file=$ResolvedImage.Path"
    
    # Network (user-mode with SSH forwarding)
    "-device", "virtio-net-pci,netdev=n0"
    "-netdev", "user,id=n0,hostfwd=tcp::$SshPort-:22"
    
    # Serial console
    "-serial", "mon:stdio"
    
    # QEMU monitor
    "-monitor", "none"
)

# TCG fallback (WHPX SMP bug #3178)
if ($Tcg) {
    $QemuArgs += "-accel", "tcg"
    if ($Smp -eq 4) {
        $Smp = 1
        $QemuArgs[5] = "1"  # Update -smp value
    }
    Write-Host "[LITRIC] Using TCG fallback (SMP=$Smp)" -ForegroundColor Yellow
}

# Debug mode (GDB stub, frozen at start)
if ($Debug) {
    $QemuArgs += "-s", "-S"
    Write-Host "[LITRIC] Debug mode: GDB stub on :1234, VM frozen at start" -ForegroundColor Cyan
    Write-Host "[LITRIC] Connect with: gdb vmlinux -ex 'target remote :1234'" -ForegroundColor Cyan
    Write-Host "[LITRIC] Boot with nokaslr for stable breakpoints" -ForegroundColor Cyan
}

# =============================================================================
# LAUNCH
# =============================================================================

Write-Host "=========================================" -ForegroundColor Green
Write-Host "LITRIC OS — QEMU Launcher" -ForegroundColor Green
Write-Host "=========================================" -ForegroundColor Green
Write-Host "Image:   $ResolvedImage.Path"
Write-Host "Firmware: $OvmfCode"
Write-Host "SMP:     $Smp"
Write-Host "Memory:  $Memory"
Write-Host "SSH:     localhost:$SshPort -> guest:22"
Write-Host "Log:     $LogFile"
Write-Host "=========================================" -ForegroundColor Green

# Run QEMU and capture output to log
& qemu-system-x86_64 @QemuArgs 2>&1 | Tee-Object -FilePath $LogFile

Write-Host "`n[LITRIC] QEMU exited. Log: $LogFile" -ForegroundColor Yellow
