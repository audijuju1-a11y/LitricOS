# LITRIC — Kernel Debugging Guide
**Document:** DEV-01-debug.md | **Phase:** 01 | **Status:** DESIGN-ONLY

---

## 1. Prerequisites

- QEMU with `-s -S` support (gdbstub)
- GDB with Python support (`gdb-add-auto-load-safe-path`)
- `vmlinux` (unstripped kernel image from build)
- `nokaslr` kernel parameter for stable breakpoints

## 2. Quick Start

### Terminal 1: Start QEMU (frozen at start)

```bash
./tools/run-qemu.sh --debug
# or on Windows:
.\tools\run-qemu.ps1 -Debug
```

This adds `-s -S` flags:
- `-s` = GDB server on TCP port 1234
- `-S` = Freeze CPU at start (wait for GDB to continue)

### Terminal 2: Connect GDB

```bash
cd /path/to/linux-source
gdb vmlinux
```

Inside GDB:
```gdb
# Load Litric's GDB scripts (for pretty-printing)
add-auto-load-safe-path /path/to/linux/scripts/gdb/vmlinux-gdb.py

# Connect to QEMU
target remote :1234

# Set a hardware breakpoint (software BP misses early boot code)
hbreak start_kernel

# Continue until breakpoint hits
c

# Now you're at the start of the kernel
```

## 3. Important GDB Commands

```gdb
# Hardware breakpoints (use for early boot, before software BP works)
hbreak <function>
hbreak *0x<address>

# Software breakpoints (normal use after early boot)
break <function>

# Single step
si          # one instruction
ni          # one source line
step        # step into function
next        # step over function

# Examine memory
x/10i $pc   # 10 instructions at PC
x/16xg $rsp # 16 64-bit values at stack
x/s <addr>  # string at address

# Registers
info registers
info registers rax rbx rcx

# Thread info
info threads

# Kernel-specific
p/d variable         # print decimal
p/x variable         # print hex
p *variable          # dereference pointer
list                 # show source
```

## 4. Debugging Tips

### Boot failures
1. Use `nokaslr` to get stable addresses
2. Set `hbreak start_kernel` and step through
3. Check `earlyprintk` output on serial console
4. Use `dmesg` from another terminal via SSH

### Kernel panic analysis
```bash
# In GDB after panic:
bt                          # backtrace
p current_stack_trace()     # stack trace (if CONFIG_STACKTRACE)
info dmesg                  # kernel log via GDB
```

### Memory debugging
```bash
# Enable KASAN (if compiled in)
echo 1 > /sys/kernel/debug/kasan/report

# Check for memory leaks
cat /proc/meminfo
slabtop -o -s c
```

### Scheduler debugging (sched_ext)
```bash
# Check scheduler state
cat /sys/kernel/debug/sched/debug

# BPF program state
bpftool prog list
bpftool map dump
```

## 5. Common Issues

| Symptom | Cause | Fix |
|---------|-------|-----|
| GDB can't connect | QEMU not started with `-s` | Add `--debug` flag |
| Software BP missed | Early boot code | Use `hbreak` instead of `break` |
| KASLR randomizes addresses | Normal security feature | Boot with `nokaslr` for debugging |
| QEMU hangs at start | Normal with `-S` | Press `c` in GDB to continue |
| No serial output | Missing `console=ttyS0` | Add to kernel cmdline |

## 6. Serial Console

All kernel messages go to the serial console (`-serial mon:stdio`).

To capture logs:
```bash
# Boot with earlyprintk for earliest messages
# Kernel cmdline: earlyprintk=serial console=ttyS0,115200

# Save log to file
./tools/run-qemu.sh 2>&1 | tee tools/logs/debug-$(date +%s).log
```

## 7. Debug Build vs Release

| Feature | Debug Build | Release Build |
|---------|-------------|---------------|
| KASAN | Enabled | Disabled |
| KASLR | Optional | Enabled |
| DEBUG_INFO | Full | Reduced |
| DEBUG_KERNEL | Yes | No |
| Panic timeout | 0 (hang) | Configurable |

Use debug builds during development. Switch to release for measurements.
