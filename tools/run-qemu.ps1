# =============================================================================
# LITRIC OS - QEMU Launcher (Windows Host)
# Launches Litric in QEMU with WHPX acceleration
#
# Prerequisites:
#   - QEMU for Windows (https://qemu.weilnetz.de/w64/), or: winget install SoftwareFreedomConservancy.QEMU
#   - litric.qcow2 in build/ (see tools/build-image.sh - needs a Linux host with mkosi)
#   - Firmware: build/OVMF_CODE_4M.fd + build/OVMF_VARS_4M.fd if you have them;
#     otherwise the split edk2 images QEMU ships are used, so nothing to download.
#
# Usage:
#   .\tools\run-qemu.ps1                    # Normal boot (headless: serial on this console)
#   .\tools\run-qemu.ps1 -Gui               # Real window, YOUR pointer and keyboard
#   .\tools\run-qemu.ps1 -Gui -Display gtk  # Same, through GTK instead of SDL
#   .\tools\run-qemu.ps1 -Gui -NoSound      # Window without the audio device
#   .\tools\run-qemu.ps1 -Debug             # GDB stub (frozen at start)
#   .\tools\run-qemu.ps1 -Tcg               # TCG fallback (WHPX SMP bug #3178)
#   .\tools\run-qemu.ps1                    # Uses all host logical processors
#   .\tools\run-qemu.ps1 -Smp 1             # Single core
#   .\tools\run-qemu.ps1 -Memory 8G         # 8 GB RAM
#
# -Gui is what you want to use the desktop rather than measure it: the window is
# a real QEMU display, the pointer is a USB tablet (absolute - clicks land where
# you aim) and the keyboard is a USB keyboard, both grabbed by the window exactly
# like tools/run-qemu-gui.sh does under WSLg. The guest console moves off this
# console to build/gui-serial.log, and the QEMU monitor is reachable at
# 127.0.0.1:4444 (`nc 127.0.0.1 4444`).
#
# SAFETY: Never touches host physical disks. All paths relative to project dir.
# =============================================================================

param(
    [switch]$Debug,
    [switch]$Tcg,
    [switch]$Gui,
    [string]$Display = "sdl",
    [switch]$NoSound,
    [int]$MonitorPort = 4444,
    [int]$Smp = 0,
    [string]$Memory = "4G",
    [string]$Image = "",
    [ValidateSet("ide", "virtio")]
    [string]$DiskBus = "ide",
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

# Default to the host's full logical CPU count. Pass -Smp to choose a smaller
# guest allocation explicitly.
if ($Smp -le 0) {
    $Smp = (Get-CimInstance Win32_ComputerSystem).NumberOfLogicalProcessors
    if (!$Smp -or $Smp -lt 1) { $Smp = 1 }
}

# Verify project root
if (!(Test-Path (Join-Path $ProjectRoot "00-MASTER-WORKFLOW.md"))) {
    Write-Error "Not running from project root. Expected 00-MASTER-WORKFLOW.md at $ProjectRoot"
    exit 1
}

# The QEMU binary: resolved by name, then by the paths the Windows installer
# uses. It is not always on PATH - the MSI puts it in Program Files and only a
# fresh shell sees the PATH entry - so a baked-in fallback saves a confusing
# "not recognized as an internal or external command" before anything runs.
$QemuExe = (Get-Command qemu-system-x86_64 -ErrorAction SilentlyContinue).Source
if (!$QemuExe) {
    foreach ($candidate in @(
        (Join-Path $env:ProgramFiles "qemu\qemu-system-x86_64.exe"),
        (Join-Path ${env:ProgramFiles(x86)} "qemu\qemu-system-x86_64.exe")
    )) {
        if ($candidate -and (Test-Path $candidate)) { $QemuExe = $candidate; break }
    }
}
if (!$QemuExe) {
    Write-Error "qemu-system-x86_64 not found.`nInstall it with: winget install SoftwareFreedomConservancy.QEMU`nThen reopen this shell (the installer only updates PATH for new shells)."
    exit 1
}

# Image path: the qcow2 the image build produces, or the raw image the GUI
# recipe in tools/run-qemu-gui.sh boots, so one launcher serves both.
if ($Image -eq "") {
    $Image = Join-Path $BuildDir "litric.qcow2"
    if (!(Test-Path $Image)) {
        $rawImage = Join-Path $BuildDir "out\litricos.raw"
        if (Test-Path $rawImage) { $Image = $rawImage }
    }
}

# Verify image exists (safety: must be inside project dir)
$ResolvedImage = Resolve-Path $Image -ErrorAction SilentlyContinue
if ($ResolvedImage -eq $null) {
    Write-Error @"
Image not found: $Image

There is no bootable Litric image in this checkout yet. Build one with:
    ./tools/build-image.sh          # needs a Linux host (or WSL2) with mkosi
or copy a prebuilt image to build/litric.qcow2 (raw is fine too: -Image build/out/litricos.raw).
Nothing here can build it on Windows alone: the image is a UEFI/mkosi Linux system.
"@
    exit 1
}
if (!$ResolvedImage.Path.StartsWith($ProjectRoot)) {
    Write-Error "SAFETY VIOLATION: Image path must be inside project directory!`nImage: $($ResolvedImage.Path)`nProject: $ProjectRoot"
    exit 1
}

# A plain string from here on. `"$ResolvedImage.Path"` inside a double-quoted
# string interpolates the object's string form and then appends the literal text
# ".Path", which is how the disk path used to reach QEMU as
# `...\litric.qcow2.Path` - a file that does not exist.
$ImagePath = $ResolvedImage.Path
$IsIso = $ImagePath -match '\.iso$'
$DiskFormat = if ($ImagePath -match '\.raw$') { "raw" } else { "qcow2" }

# Firmware: the project's own OVMF if it is there, otherwise the split edk2
# images QEMU ships (code + a vars template whose size makes up a 4 MiB pflash
# together), so a fresh install boots UEFI with nothing to download. The vars
# half must be writable - UEFI stores its boot variables in it - so the template
# is copied into build/ once and reused across boots.
$OvmfCode = Join-Path $BuildDir "OVMF_CODE_4M.fd"
$OvmfVars = Join-Path $BuildDir "OVMF_VARS_4M.fd"

if (!(Test-Path $OvmfCode) -or !(Test-Path $OvmfVars)) {
    $QemuShare = Join-Path (Split-Path -Parent $QemuExe) "share"
    $shareCode = Join-Path $QemuShare "edk2-x86_64-code.fd"
    $shareVars = Join-Path $QemuShare "edk2-i386-vars.fd"
    if ((Test-Path $shareCode) -and (Test-Path $shareVars)) {
        $OvmfCode = $shareCode
        $OvmfVars = Join-Path $BuildDir "OVMF_VARS_win.fd"
        if (!(Test-Path $OvmfVars)) {
            Copy-Item $shareVars $OvmfVars
        }
        Write-Host "[LITRIC] Using the edk2 firmware shipped with QEMU (no OVMF download needed)" -ForegroundColor Cyan
    } else {
        Write-Error "OVMF firmware not found.`nDownload OVMF_CODE_4M.fd and OVMF_VARS_4M.fd to $BuildDir\"
        exit 1
    }
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
    
    # Network (user-mode with SSH forwarding)
    "-device", "virtio-net-pci,netdev=n0"
    "-netdev", "user,id=n0,hostfwd=tcp::$SshPort-:22"
)

# q35 exposes its ICH9 SATA controller to OVMF, while the QEMU-bundled
# firmware here has no VirtIO block boot driver. Give the SATA disk an explicit
# UEFI boot index so firmware finds the ESP. The Litric initrd includes AHCI.
# VirtIO remains available for firmware that supplies VirtIO DXE drivers.
if ($IsIso) {
    # Live ISO carries its own EFI system partition as an appended GPT partition.
    # Attach it as an IDE CD-ROM so OVMF can boot the El Torito UEFI entry.
    $QemuArgs += @(
        "-drive", "if=none,id=litriccd,media=cdrom,format=raw,readonly=on,file=$ImagePath"
        "-device", "ide-cd,drive=litriccd,bus=ide.1,bootindex=1"
        "-boot", "order=d"
    )
} elseif ($DiskBus -eq "virtio") {
    $QemuArgs += @("-drive", "if=virtio,format=$DiskFormat,file=$ImagePath")
    $QemuArgs += @("-boot", "order=c")
} else {
    $QemuArgs += @(
        "-drive", "if=none,id=litricdisk,format=$DiskFormat,file=$ImagePath"
        "-device", "ide-hd,drive=litricdisk,bus=ide.0,bootindex=1"
    )
    $QemuArgs += @("-boot", "order=c")
}

# Display and input. Headless keeps the serial console on this console, which is
# how the verification runs read a boot; -Gui swaps that for the real thing and
# moves the console into a log file so the window owns the screen.
if ($Gui) {
    $GuiSerialLog = Join-Path $LogsDir "gui-serial-$Timestamp.log"
    $QemuArgs += @(
        # Match the kernel's `video=Virtual-1:1920x1080` display selection. QEMU's default
        # Bochs VGA produced a healthy framebuffer log while the SDL window stayed
        # black; virtio-vga exposes the scanout as Virtual-1 to the guest.
        "-vga", "virtio"
        "-display", $Display
        # An absolute pointer and a real keyboard. q35 has no default USB bus,
        # so -usb adds the machine's own controller first; without it QEMU exits
        # with "No 'usb-bus' bus found for device 'usb-tablet'" before drawing
        # anything. The guest measures the tablet's axis range with EVIOCGABS and
        # litric-input defaults to the same constant, so a click lands where it
        # was aimed.
        "-usb"
        "-device", "usb-tablet"
        "-device", "usb-kbd"
        "-serial", "file:$GuiSerialLog"
        "-monitor", "tcp:127.0.0.1:$MonitorPort,server,nowait"
    )
    if (!$NoSound) {
        # Sound only when the host has an output device: a machine with audio
        # disabled would otherwise fail to start QEMU at all.
        $hasSound = $false
        try { $hasSound = @(Get-CimInstance Win32_SoundDevice -ErrorAction Stop).Count -gt 0 } catch { $hasSound = $false }
        if ($hasSound) {
            $QemuArgs += @(
                "-audiodev", "dsound,id=snd0"
                "-device", "intel-hda"
                "-device", "hda-duplex,audiodev=snd0"
            )
        } else {
            Write-Host "[LITRIC] No host sound device: booting without audio" -ForegroundColor Yellow
        }
    }
} else {
    $QemuArgs += @(
        # Serial console
        "-serial", "mon:stdio"
        # QEMU monitor
        "-monitor", "none"
    )
}

# TCG fallback (WHPX SMP bug #3178)
if ($Tcg) {
    $QemuArgs[1] = "q35,accel=tcg"
    $QemuArgs[3] = "max,x2apic=off"
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
Write-Host "LITRIC OS - QEMU Launcher" -ForegroundColor Green
Write-Host "=========================================" -ForegroundColor Green
Write-Host "Image:   $ImagePath"
Write-Host "Firmware: $OvmfCode"
Write-Host "SMP:     $Smp"
Write-Host "Memory:  $Memory"
Write-Host "Disk bus: $(if ($IsIso) { 'CD-ROM (ISO)' } else { $DiskBus })"
Write-Host "SSH:     localhost:$SshPort -> guest:22"
if ($Gui) {
    Write-Host "Display: $Display (window: your pointer and keyboard)" -ForegroundColor Green
    Write-Host "Console: $GuiSerialLog"
    Write-Host "Monitor: 127.0.0.1:$MonitorPort"
} else {
    Write-Host "Display: none (headless: serial on this console)"
}
Write-Host "QEMU:    $QemuExe"
Write-Host "Log:     $LogFile"
Write-Host "=========================================" -ForegroundColor Green

# Run QEMU and capture output to log
& $QemuExe @QemuArgs 2>&1 | Tee-Object -FilePath $LogFile

Write-Host "`n[LITRIC] QEMU exited. Log: $LogFile" -ForegroundColor Yellow
