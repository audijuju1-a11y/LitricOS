<#
.SYNOPSIS
    Prepare a Windows machine to build LitricOS.

.DESCRIPTION
    LitricOS is a Linux-based operating system. Its image is produced by mkosi,
    systemd-repart and erofs-utils, none of which run on Windows. This script
    installs WSL2 and a build distribution, then provisions the toolchain inside
    it, so that `tools/build.sh` has somewhere to run.

    This is the blocker that keeps GATE 01 at BLOCKED. Nothing about LitricOS can
    be measured, booted or trusted until an image exists, and no image can exist
    without a Linux build host.

.NOTES
    SAFETY

    This script modifies the host: it enables Windows optional features and
    installs a WSL distribution, and it may require a reboot. It NEVER touches a
    physical disk, never repartitions anything, and never writes outside the
    project directory and WSL's own storage. MASTER invariant G1 (the host OS is
    untouched) refers to LitricOS installs; a development toolchain is a
    deliberate exception.

    To reverse it: `wsl --unregister <Distro>` removes the distribution, and the
    two optional features can be disabled from "Turn Windows features on or off".
    This script does not automate that, because a script that can uninstall
    Windows features is a script that can do so by accident.

    It requires Administrator. It will say so and stop rather than half-running.

.PARAMETER Distro
    WSL distribution to install. Default archlinux.

    Arch is not a preference, it is a requirement of the build: mkosi bootstraps
    the LitricOS image by driving pacman on the HOST, and pacman only exists on
    Arch. A Debian or Ubuntu build host would have to run the entire build inside
    an Arch container. See tools/provision-builder.sh.

.PARAMETER SkipProvision
    Install WSL only; do not install the build toolchain inside it.

.PARAMETER CheckOnly
    Report what is present and what is missing, change nothing. Safe to run
    without Administrator.

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File tools\bootstrap-windows.ps1 -CheckOnly
    powershell -ExecutionPolicy Bypass -File tools\bootstrap-windows.ps1
#>

[CmdletBinding()]
param(
    [string] $Distro = 'archlinux',
    [switch] $SkipProvision,
    [switch] $CheckOnly
)

$ErrorActionPreference = 'Stop'

$ProjectRoot = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)

function Write-Step { param([string] $Message) Write-Host "[bootstrap] $Message" -ForegroundColor Green }
function Write-Warn { param([string] $Message) Write-Host "[bootstrap] $Message" -ForegroundColor Yellow }
function Write-Err  { param([string] $Message) Write-Host "[bootstrap] $Message" -ForegroundColor Red }

function Test-Admin {
    $identity  = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($identity)
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

<#
.SYNOPSIS
    Run a native executable and return its exit code and combined output.

.DESCRIPTION
    Windows PowerShell 5.1 wraps every stderr line from a native executable in a
    NativeCommandError ErrorRecord. With $ErrorActionPreference = 'Stop' -- which
    this script sets, correctly, so that a genuine failure halts it -- that turns
    any program writing to stderr into a terminating error, even when it exits 0.

    `wsl --status` writes "The Windows Subsystem for Linux is not installed" to
    stderr. That is the normal, expected answer on a machine without WSL, and it
    was aborting this script with a wall of red text instead of being handled.

    So every native call goes through here: preference is relaxed locally, output
    is captured as plain strings, and the caller decides what the exit code
    means. WSL also emits UTF-16, which is why the output is de-nulled.
#>
function Invoke-Native {
    param(
        [Parameter(Mandatory)][string]   $Exe,
        [Parameter()][string[]]          $Arguments = @()
    )
    $prev = $ErrorActionPreference
    $ErrorActionPreference = 'Continue'
    try {
        $output = & $Exe @Arguments 2>&1 | ForEach-Object { ($_ | Out-String) -replace "`0", '' }
        $code = $LASTEXITCODE
    } finally {
        $ErrorActionPreference = $prev
    }
    return [pscustomobject]@{
        ExitCode = $code
        Output   = ($output -join '').Trim()
        Lines    = @($output | ForEach-Object { $_.Trim() } | Where-Object { $_ })
    }
}

# -----------------------------------------------------------------------------
# Preflight. Report everything before changing anything, so the user knows what
# they are agreeing to.
# -----------------------------------------------------------------------------
function Invoke-Preflight {
    Write-Step 'Checking prerequisites'
    $problems = @()

    $build = [int](Get-CimInstance Win32_OperatingSystem).BuildNumber
    Write-Host ("  Windows build      : {0}" -f $build)
    if ($build -lt 19041) {
        $problems += "Windows build $build is too old for WSL2 (need 19041+). Update Windows."
    }

    # WSL2 runs on the Hyper-V hypervisor, which needs virtualization enabled in
    # firmware. This is by far the most common reason `wsl --install` appears to
    # succeed and then fails to start a distribution.
    $virt = (Get-CimInstance Win32_ComputerSystem).HypervisorPresent
    Write-Host ("  Hypervisor present : {0}" -f $virt)
    if (-not $virt) {
        $cpu = Get-CimInstance Win32_Processor | Select-Object -First 1
        if (-not $cpu.VirtualizationFirmwareEnabled) {
            $problems += 'Hardware virtualization is disabled in firmware. Enable Intel VT-x / AMD-V (often called SVM) in the BIOS/UEFI setup, then re-run.'
        }
    }

    $wslPresent = $null -ne (Get-Command wsl.exe -ErrorAction SilentlyContinue)
    $wslWorking = $false
    if ($wslPresent) {
        # wsl.exe exists on every modern Windows even when WSL is not installed;
        # `wsl --status` is what actually distinguishes the two. It reports the
        # "not installed" case on stderr with a non-zero exit, which is a normal
        # answer here and must not be treated as a script failure.
        $status = Invoke-Native -Exe 'wsl.exe' -Arguments @('--status')
        $wslWorking = ($status.ExitCode -eq 0) -and ($status.Output -notmatch 'is not installed')
    }
    Write-Host ("  WSL installed      : {0}" -f $wslWorking)

    # Space requirement, measured by what the build actually consumes rather than
    # by a round number. LitricOS does not compile a kernel from source -- it
    # uses the distribution's packaged kernel (MASTER G5: integrate, do not
    # reinvent) -- so there is no 20 GB kernel tree in the budget.
    #
    #   WSL distribution base            ~1.5 GB
    #   build toolchain                  ~2.0 GB
    #   package cache, nano profile      ~2.0 GB   (base + systemd + linux)
    #   package cache, desktop profile  ~+6.0 GB   (mesa, pipewire, fonts, firmware)
    #   image output and intermediates   ~1.5 GB nano / ~4.0 GB desktop
    #
    # So: 10 GB gets you a bootable nano image. 25 GB gets you the full desktop.
    # Reporting one number for both would have blocked a machine that can
    # perfectly well build what the user needs today.
    $needNano    = 10
    $needDesktop = 25

    $drive  = (Get-Item $ProjectRoot).PSDrive.Name
    $freeGb = [math]::Round((Get-PSDrive -Name $drive).Free / 1GB, 1)
    Write-Host ("  Free space on {0}:   : {1} GB" -f $drive, $freeGb)

    if ($freeGb -lt $needNano) {
        $problems += ("Only $freeGb GB free on ${drive}:. Even the minimal 'nano' profile needs about $needNano GB (WSL distribution, toolchain, package cache, output image).")
    } elseif ($freeGb -lt $needDesktop) {
        Write-Warn ("$freeGb GB free is enough for the 'nano' and 'core' profiles (~$needNano GB).")
        Write-Warn ("The 'desktop' and 'gaming' profiles need about $needDesktop GB and will fail to build.")
        Write-Warn ("Build nano first -- that is what GATE 01 requires anyway -- and free space before the desktop profile.")
    }

    return [pscustomobject]@{
        Problems   = $problems
        WslWorking = $wslWorking
        Build      = $build
    }
}

# -----------------------------------------------------------------------------
$pre = Invoke-Preflight

if ($pre.Problems.Count -gt 0) {
    Write-Host ''
    Write-Err 'Cannot proceed:'
    foreach ($p in $pre.Problems) { Write-Host "  - $p" }
    Write-Host ''
    Write-Host 'None of these can be fixed by this script. Resolve them and re-run.'
    exit 1
}

if ($CheckOnly) {
    Write-Host ''
    Write-Step 'All prerequisites satisfied. Re-run without -CheckOnly to install.'
    exit 0
}

if (-not (Test-Admin)) {
    Write-Host ''
    Write-Err 'Administrator rights are required to install WSL.'
    Write-Host ''
    Write-Host 'Open PowerShell as Administrator and run:'
    Write-Host ("  powershell -ExecutionPolicy Bypass -File `"{0}\tools\bootstrap-windows.ps1`"" -f $ProjectRoot)
    Write-Host ''
    Write-Host 'This script does not elevate itself. A build tool that silently'
    Write-Host 'asks for Administrator is a build tool you cannot audit.'
    exit 1
}

# -----------------------------------------------------------------------------
# Install WSL
# -----------------------------------------------------------------------------
if (-not $pre.WslWorking) {
    Write-Step "Installing WSL2 and $Distro (this takes a few minutes)"
    Write-Warn 'A reboot will very likely be required afterwards.'

    $inst = Invoke-Native -Exe 'wsl.exe' -Arguments @('--install', '--distribution', $Distro)
    Write-Host $inst.Output
    $rc = $inst.ExitCode

    if ($rc -ne 0) {
        Write-Warn "wsl --install returned $rc; falling back to enabling the features directly."
        # The one-shot installer fails on some builds and in some managed
        # environments. The underlying operations are these two optional
        # features plus the kernel update, so do them explicitly rather than
        # reporting a failure the user cannot act on.
        $f1 = Invoke-Native -Exe 'dism.exe' -Arguments @('/online','/enable-feature','/featurename:Microsoft-Windows-Subsystem-Linux','/all','/norestart')
        Write-Host $f1.Output
        $f2 = Invoke-Native -Exe 'dism.exe' -Arguments @('/online','/enable-feature','/featurename:VirtualMachinePlatform','/all','/norestart')
        Write-Host $f2.Output
        Write-Host ''
        Write-Step 'Features enabled. Reboot, then run this script again.'
        exit 0
    }

    Write-Host ''
    Write-Step 'WSL install step finished.'
    Write-Warn 'Reboot now, then run this script again to provision the toolchain.'
    Write-Warn 'On first launch the distribution will ask you to create a username and password.'
    exit 0
}

Write-Step 'WSL is installed and working.'
 $null = Invoke-Native -Exe 'wsl.exe' -Arguments @('--set-default-version', '2')

# Confirm the target distribution is actually present. `wsl -l -q` emits UTF-16,
# which is why a naive string match against its output silently fails.
$listed = Invoke-Native -Exe 'wsl.exe' -Arguments @('-l', '-q')
$installed = $listed.Lines
Write-Host ("  Distributions      : {0}" -f ($installed -join ', '))

if ($installed -notcontains $Distro) {
    Write-Step "Installing distribution $Distro"
    $inst = Invoke-Native -Exe 'wsl.exe' -Arguments @('--install', '--distribution', $Distro)
    if ($inst.ExitCode -ne 0) {
        Write-Err "Could not install $Distro."
        Write-Host $inst.Output
        Write-Host 'Available distributions:'
        $avail = Invoke-Native -Exe 'wsl.exe' -Arguments @('--list', '--online')
        Write-Host $avail.Output
        exit 1
    }
    Write-Warn 'Launch the distribution once to create your user, then re-run this script.'
    exit 0
}

if ($SkipProvision) {
    Write-Step 'WSL ready. Skipping toolchain provisioning as requested.'
    exit 0
}

# -----------------------------------------------------------------------------
# Provision the build toolchain inside WSL
# -----------------------------------------------------------------------------
Write-Step "Provisioning the build toolchain inside $Distro"

# Translate the Windows project path to its WSL mount point. Doing this by hand
# (C:\ -> /mnt/c/) breaks on any drive that is not C: and on paths with spaces,
# so let wslpath do it.
$pathProbe = Invoke-Native -Exe 'wsl.exe' -Arguments @('-d', $Distro, '--', 'wslpath', '-a', $ProjectRoot.Replace('\', '/'))
$wslProject = if ($pathProbe.ExitCode -eq 0) { $pathProbe.Output } else { '' }
if (-not $wslProject) {
    $drive = $ProjectRoot.Substring(0, 1).ToLower()
    $rest  = $ProjectRoot.Substring(2).Replace('\', '/')
    $wslProject = "/mnt/$drive$rest"
    Write-Warn "wslpath unavailable; using derived path $wslProject"
}
$wslProject = $wslProject.Trim()
Write-Host ("  Project inside WSL : {0}" -f $wslProject)

# Provisioning output is streamed rather than captured: it is long, and a user
# watching a package install needs to see progress, not a wall of text at the end.
$prev = $ErrorActionPreference; $ErrorActionPreference = 'Continue'
& wsl.exe -d $Distro -u root -- bash -lc "cd '$wslProject' && bash tools/provision-builder.sh"
$provisionRc = $LASTEXITCODE
$ErrorActionPreference = $prev
if ($provisionRc -ne 0) {
    Write-Err "Toolchain provisioning failed (exit $provisionRc). The output above is from inside WSL."
    exit 1
}

Write-Host ''
Write-Step 'Build host is ready.'
Write-Host ''
Write-Host '  IMPORTANT: build from inside WSL, not from the /mnt/c path.'
Write-Host '  Building on a /mnt/c path is 5-20x slower (every file operation'
Write-Host '  crosses the 9p filesystem boundary) and loses Unix permissions,'
Write-Host '  which breaks the image tree. provision-builder.sh set up a clone'
Write-Host '  inside the WSL filesystem for this reason.'
Write-Host ''
Write-Host '  Next:'
Write-Host ("    wsl -d {0}" -f $Distro)
Write-Host '    cd ~/litricos'
Write-Host '    ./tools/build.sh --profile nano'
Write-Host '    ./tools/run-qemu.sh'
Write-Host ''
