# Batch 02 — Kernel Config, mkosi Image Pipeline, systemd Units, Live ISO & Keys

Total files documented in this file: **65**

---

## F0546 — `image/mkosi.conf`

### A. Identity
- **Path**: `image/mkosi.conf`
- **Type / Language**: `.conf` (INI/Config)
- **Size**: 12,543 bytes
- **Classification**: `configuration` | Role: `os-image-or-kernel-config`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: ============================================================================= LitricOS — image definition Schema: mkosi >= 25.3.  Section and key names below are the REAL mkosi schema. Partition geometry is NOT expressed here — it lives in mkosi.repart/*.conf using systemd-repart syntax.  See docs/bugs/BUG-0001 for why that sentence is written in capitals.
- **Subsystem**: `os-image-or-kernel-config`

### C. Internal Structure
- **Line Count**: 276 lines
- **Key Declarations / Directives / Functions**:
  - L13: `[Config]`
  - L14: `MinimumVersion=25.3`
  - L20: `[Distribution]`
  - L21: `Distribution=arch`
  - L22: `Architecture=x86-64`
  - L28: `[Output]`
  - L29: `Format=disk`
  - L30: `ImageId=litricos`
  - L31: `OutputDirectory=../build/out`
  - L32: `CompressOutput=zstd`
  - L46: `SectorSize=512`
  - L49: `SplitArtifacts=uki,partitions`
  - L51: `[Content]`
  - L52: `Bootable=yes`
  - L53: `Bootloader=systemd-boot`
  - L66: `UnifiedKernelImages=unsigned`
  - L69: `BiosBootloader=none`
  - L71: `Locale=C.UTF-8`
  - L72: `Timezone=UTC`
  - L73: `Hostname=litricos`
  - L78: `RootPassword=!locked`
  - L79: `Autologin=no`
  - L84: `KernelCommandLine=`
  - L85: `loglevel=7`
  - L94: `video=Virtual-1:1920x1080`
  - L106: `console=tty0`
  - L107: `console=ttyS0,115200`
  - L120: `rootfstype=erofs`
  - L121: `systemd.machine_id=firstboot`
  - L127: `lsm=landlock,lockdown,yama,safesetid,integrity,bpf`
  - L128: `module_blacklist=pcspkr`
  - L129: `mitigations=auto`
  - L130: `random.trust_cpu=on`
  - L135: `transparent_hugepage=never`
  - L136: `rd.udev.log_level=3`
  - L186: `KernelInitrdModules=`
  - L198: `CleanPackageMetadata=yes`
  - L199: `WithDocs=no`
  - L200: `WithRecommends=no`
  - L203: `RemoveFiles=`
  - L245: `[Validation]`
  - L261: `[Build]`
  - L273: `[Runtime]`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/dev/console`, `/dev/mapper/root`, `/usr/include`, `/usr/lib/`, `/usr/lib/modules`, `/usr/lib/modules/`, `/usr/share/doc`, `/usr/share/gtk-doc`, `/usr/share/info`, `/usr/share/licenses/`, `/usr/share/locale/`, `/usr/share/man`, `/var/cache/pacman`, `ci/lint-config.sh`, `ci/lint-kconfig.py`, `ci/lint-mkosi.py`, `docs/bugs/BUG-0001`, `iso/assemble-iso.sh`, `iso/hooks/`, `kernel/config/litric-base.config`, `kernel/config/litric-base.config.`, `keys/dev/pcr.crt`, `keys/dev/pcr.key`, `keys/dev/secureboot.crt`, `keys/dev/secureboot.key`
- **Environment Variables / Flags**: `SOURCE_DATE_EPOCH`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0547 — `image/mkosi.extra/usr/lib/litric/first-run.sh`

### A. Identity
- **Path**: `image/mkosi.extra/usr/lib/litric/first-run.sh`
- **Type / Language**: `.sh` (Bash/POSIX Shell)
- **Size**: 3,032 bytes
- **Classification**: `configuration` | Role: `os-image-or-kernel-config`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: First-boot credential setup. This runs only on tty1 before the desktop takes the display, and uses the distribution's passwd tool so the password is entered with terminal echo disabled and is written through the normal shadow locking/update path. No password is stored in this script, argv or journal. Clearing the screen is a printf, not `clear`. The image ships no terminfo database, so `clear` exits 1 with "terminals database is inaccessible" -- and
- **Subsystem**: `os-image-or-kernel-config`

### C. Internal Structure
- **Line Count**: 70 lines
- **Key Declarations / Directives / Functions**:
  - L20: `clear_screen() { printf '\033[2J\033[H'; }`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/proc/cmdline`, `/usr/bin/bash`, `/usr/bin/chmod`, `/usr/bin/getent`, `/usr/bin/grep`, `/usr/bin/mkdir`, `/usr/bin/mv`, `/usr/bin/passwd`, `/var/lib/litric-shell`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0548 — `image/mkosi.extra/usr/lib/litric/mm-tune`

### A. Identity
- **Path**: `image/mkosi.extra/usr/lib/litric/mm-tune`
- **Type / Language**: `no_ext` (Plaintext/Config)
- **Size**: 4,921 bytes
- **Classification**: `configuration` | Role: `os-image-or-kernel-config`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: ============================================================================= LitricOS — memory-management runtime tuning These settings have no sysctl equivalent; they live in /sys and must be written once at boot. Run by litric-mm-tune.service before basic.target. Every write here is (a) best-effort and (b) logged. A kernel that does not expose a knob is not an error -- it is an older or differently-configured
- **Subsystem**: `os-image-or-kernel-config`

### C. Internal Structure
- **Line Count**: 97 lines

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/dev/null`, `/proc/meminfo`, `/sys/block/`, `/sys/kernel/mm/lru_gen/enabled`, `/sys/kernel/mm/lru_gen/min_ttl_ms`, `/sys/kernel/mm/transparent_hugepage/defrag`, `/sys/kernel/mm/transparent_hugepage/enabled`, `/sys/kernel/mm/transparent_hugepage/shmem_enabled`, `/sys/module/zswap/parameters/enabled`, `/usr/bin/env`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0549 — `image/mkosi.extra/usr/lib/litric/selftest`

### A. Identity
- **Path**: `image/mkosi.extra/usr/lib/litric/selftest`
- **Type / Language**: `no_ext` (Plaintext/Config)
- **Size**: 13,921 bytes
- **Classification**: `configuration` | Role: `os-image-or-kernel-config`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: ============================================================================= LitricOS — in-guest self test and measurement Runs once after multi-user.target and prints a structured report to the console. This is how anything inside a booted LitricOS gets measured: the serial log is the only channel out of a headless VM, so the report is designed to be greppable rather than pretty.
- **Subsystem**: `os-image-or-kernel-config`

### C. Internal Structure
- **Line Count**: 282 lines

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/dev/dri/card`, `/dev/fb0`, `/dev/ntsync`, `/dev/null`, `/dev/zram0`, `/proc/meminfo`, `/run/litric/shell.status`, `/run/litric/sysd.sock`, `/sys/class/graphics/fb0`, `/sys/class/graphics/fb0/bits_per_pixel`, `/sys/class/graphics/fb0/name`, `/sys/class/graphics/fb0/stride`, `/sys/class/graphics/fb0/virtual_size`, `/sys/kernel/mm/lru_gen/enabled`, `/sys/kernel/sched_ext`, `/sys/kernel/security/lsm`, `/usr/bin/bluetoothctl`, `/usr/bin/env`, `/usr/bin/ip`, `/usr/bin/iwctl`, `/usr/bin/litric-shell`, `/usr/bin/litric-sysd`, `/usr/bin/networkctl`, `/usr/bin/systemctl`, `/usr/bin/unix_chkpwd`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0550 — `image/mkosi.extra/usr/lib/modules-load.d/litric-zram.conf`

### A. Identity
- **Path**: `image/mkosi.extra/usr/lib/modules-load.d/litric-zram.conf`
- **Type / Language**: `.conf` (INI/Config)
- **Size**: 449 bytes
- **Classification**: `configuration` | Role: `os-image-or-kernel-config`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Load zram at boot. zram-generator writes a swap unit for /dev/zram0, but the device only exists once the module is loaded. Without this the unit waits for a device that will never appear, times out after 90 seconds, and turns a five-second boot into a ninety-seven-second one -- the single largest boot-time cost measured so far. The module IS present in the image (drivers/block/zram/zram.ko.zst); nothing
- **Subsystem**: `os-image-or-kernel-config`

### C. Internal Structure
- **Line Count**: 10 lines

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/dev/zram0`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0551 — `image/mkosi.extra/usr/lib/sysctl.d/10-litric-mm.conf`

### A. Identity
- **Path**: `image/mkosi.extra/usr/lib/sysctl.d/10-litric-mm.conf`
- **Type / Language**: `.conf` (INI/Config)
- **Size**: 2,329 bytes
- **Classification**: `configuration` | Role: `os-image-or-kernel-config`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Memory management — tuned for a zram-only swap topology. Each value carries the reason it is not the default. 180, not 60.  The default assumes swapping to a disk that is ~1000x slower than RAM.  Swapping to zram is ~10x slower than RAM, so the kernel should prefer reclaiming anonymous pages over evicting the page cache far more aggressively than the default allows.  100 is "equal cost"; >100 says anon is cheaper.
- **Subsystem**: `os-image-or-kernel-config`

### C. Internal Structure
- **Line Count**: 50 lines
- **Key Declarations / Directives / Functions**:
  - L11: `vm.swappiness = 0`
  - L17: `vm.watermark_scale_factor = 50`
  - L18: `vm.watermark_boost_factor = 0`
  - L22: `vm.page-cluster = 0`
  - L26: `vm.dirty_ratio = 10`
  - L27: `vm.dirty_background_ratio = 5`
  - L28: `vm.dirty_expire_centisecs = 3000`
  - L29: `vm.dirty_writeback_centisecs = 1500`
  - L32: `vm.max_map_count = 2147483642`
  - L36: `vm.overcommit_memory = 0`
  - L37: `vm.overcommit_ratio = 50`
  - L43: `vm.compaction_proactiveness = 0`
  - L49: `vm.min_free_kbytes = 8192`
  - L50: `vm.admin_reserve_kbytes = 4096`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: None explicitly hardcoded.

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0552 — `image/mkosi.extra/usr/lib/sysctl.d/20-litric-net.conf`

### A. Identity
- **Path**: `image/mkosi.extra/usr/lib/sysctl.d/20-litric-net.conf`
- **Type / Language**: `.conf` (INI/Config)
- **Size**: 2,692 bytes
- **Classification**: `configuration` | Role: `os-image-or-kernel-config`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Networking — throughput and latency defaults that are safe for everyone. The aggressive latency knobs (busy_poll, coalescing) are NOT here: they cost power and are Gaming Mode toggles with measured before/after (Phase 04 task 5). BBR + fq: better behaviour on lossy links and on shared home uplinks than cubic + pfifo_fast, without the bufferbloat of a large tx queue. Modern buffer sizing for gigabit-and-above links.
- **Subsystem**: `os-image-or-kernel-config`

### C. Internal Structure
- **Line Count**: 61 lines
- **Key Declarations / Directives / Functions**:
  - L7: `net.core.default_qdisc = fq`
  - L8: `net.ipv4.tcp_congestion_control = bbr`
  - L11: `net.core.rmem_max = 16777216`
  - L12: `net.core.wmem_max = 16777216`
  - L13: `net.ipv4.tcp_rmem = 4096 131072 16777216`
  - L14: `net.ipv4.tcp_wmem = 4096 65536 16777216`
  - L15: `net.core.netdev_max_backlog = 16384`
  - L16: `net.ipv4.tcp_max_syn_backlog = 8192`
  - L19: `net.ipv4.tcp_fastopen = 3`
  - L20: `net.ipv4.tcp_slow_start_after_idle = 0`
  - L21: `net.ipv4.tcp_mtu_probing = 1`
  - L22: `net.ipv4.tcp_fin_timeout = 15`
  - L23: `net.ipv4.tcp_keepalive_time = 300`
  - L36: `net.ipv4.conf.all.rp_filter = 0`
  - L37: `net.ipv4.conf.default.rp_filter = 0`
  - L41: `net.ipv4.conf.*.rp_filter = 0`
  - L42: `net.ipv4.conf.all.accept_redirects = 0`
  - L43: `net.ipv4.conf.default.accept_redirects = 0`
  - L44: `net.ipv4.conf.all.secure_redirects = 0`
  - L45: `net.ipv4.conf.all.send_redirects = 0`
  - L46: `net.ipv4.conf.default.send_redirects = 0`
  - L47: `net.ipv4.conf.all.accept_source_route = 0`
  - L48: `net.ipv4.conf.default.accept_source_route = 0`
  - L49: `net.ipv4.conf.all.log_martians = 1`
  - L50: `net.ipv4.icmp_echo_ignore_broadcasts = 1`
  - L51: `net.ipv4.icmp_ignore_bogus_error_responses = 1`
  - L52: `net.ipv4.tcp_syncookies = 1`
  - L53: `net.ipv6.conf.all.accept_redirects = 0`
  - L54: `net.ipv6.conf.default.accept_redirects = 0`
  - L55: `net.ipv6.conf.all.accept_ra = 1`
  - L56: `net.ipv6.conf.all.accept_source_route = 0`
  - L60: `net.ipv6.conf.all.use_tempaddr = 2`
  - L61: `net.ipv6.conf.default.use_tempaddr = 2`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: None explicitly hardcoded.

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0553 — `image/mkosi.extra/usr/lib/sysctl.d/30-litric-security.conf`

### A. Identity
- **Path**: `image/mkosi.extra/usr/lib/sysctl.d/30-litric-security.conf`
- **Type / Language**: `.conf` (INI/Config)
- **Size**: 3,000 bytes
- **Classification**: `configuration` | Role: `os-image-or-kernel-config`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Kernel hardening. Rule for this file: every line either has no user-visible cost, or its cost is named.  A hardening knob that silently breaks a workflow gets reverted by the first user who hits it, which is worse than not shipping it. --- information leaks ------------------------------------------------------ Hide kernel pointers from /proc and dmesg — removes the easiest KASLR bypass.
- **Subsystem**: `os-image-or-kernel-config`

### C. Internal Structure
- **Line Count**: 66 lines
- **Key Declarations / Directives / Functions**:
  - L9: `kernel.kptr_restrict = 2`
  - L10: `kernel.dmesg_restrict = 1`
  - L15: `kernel.perf_event_paranoid = 3`
  - L23: `kernel.yama.ptrace_scope = 1`
  - L28: `kernel.unprivileged_bpf_disabled = 1`
  - L29: `net.core.bpf_jit_harden = 2`
  - L33: `kernel.kexec_load_disabled = 1`
  - L37: `dev.tty.legacy_tiocsti = 0`
  - L38: `dev.tty.ldisc_autoload = 0`
  - L41: `vm.unprivileged_userfaultfd = 0`
  - L44: `fs.protected_symlinks = 1`
  - L45: `fs.protected_hardlinks = 1`
  - L46: `fs.protected_fifos = 2`
  - L47: `fs.protected_regular = 2`
  - L49: `fs.suid_dumpable = 0`
  - L52: `kernel.randomize_va_space = 2`
  - L53: `vm.mmap_rnd_bits = 32`
  - L54: `vm.mmap_rnd_compat_bits = 16`
  - L60: `kernel.sysrq = 176`
  - L66: `kernel.unprivileged_userns_clone = 1`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `tools/bench/`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0554 — `image/mkosi.extra/usr/lib/sysctl.d/40-litric-sched.conf`

### A. Identity
- **Path**: `image/mkosi.extra/usr/lib/sysctl.d/40-litric-sched.conf`
- **Type / Language**: `.conf` (INI/Config)
- **Size**: 1,062 bytes
- **Classification**: `configuration` | Role: `os-image-or-kernel-config`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Scheduler and latency. The default EEVDF scheduler is good.  These values bias it toward interactive responsiveness, which is what "smooth" means to a user, and are the baseline that scx_lavd (Gaming Mode, Phase 12) is measured against. Autogroup: per-session-leader fair sharing.  A `make -j32` in a terminal stops starving the shell.  This is one of the highest smoothness-per-line-of-config
- **Subsystem**: `os-image-or-kernel-config`

### C. Internal Structure
- **Line Count**: 21 lines
- **Key Declarations / Directives / Functions**:
  - L10: `kernel.sched_autogroup_enabled = 1`
  - L14: `kernel.sched_rt_runtime_us = 950000`
  - L15: `kernel.sched_rt_period_us = 1000000`
  - L21: `kernel.split_lock_mitigate = 1`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: None explicitly hardcoded.

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0555 — `image/mkosi.extra/usr/lib/systemd/journald.conf.d/00-litric.conf`

### A. Identity
- **Path**: `image/mkosi.extra/usr/lib/systemd/journald.conf.d/00-litric.conf`
- **Type / Language**: `.conf` (INI/Config)
- **Size**: 634 bytes
- **Classification**: `configuration` | Role: `os-image-or-kernel-config`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Budget line 3: journald <= 10 MB. Storage=volatile is a memory decision AND a disk decision: it removes the largest single source of idle disk writes, which is what keeps the <= 2 MB / 10 min idle-write budget reachable and lets SSDs reach their low-power states. Persistent logging is enabled only in the recovery/diagnostic boot entry.
- **Subsystem**: `os-image-or-kernel-config`

### C. Internal Structure
- **Line Count**: 22 lines
- **Key Declarations / Directives / Functions**:
  - L7: `[Journal]`
  - L8: `Storage=volatile`
  - L9: `RuntimeMaxUse=16M`
  - L10: `RuntimeMaxFileSize=4M`
  - L11: `RuntimeKeepFree=64M`
  - L12: `SystemMaxUse=0`
  - L13: `ForwardToSyslog=no`
  - L14: `ForwardToWall=no`
  - L15: `ForwardToKMsg=no`
  - L16: `Compress=yes`
  - L17: `Seal=no`
  - L18: `RateLimitIntervalSec=30s`
  - L19: `RateLimitBurst=1000`
  - L20: `MaxLevelStore=info`
  - L21: `MaxLevelSyslog=warning`
  - L22: `MaxLevelKMsg=warning`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: None explicitly hardcoded.

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0556 — `image/mkosi.extra/usr/lib/systemd/network/20-wired.network`

### A. Identity
- **Path**: `image/mkosi.extra/usr/lib/systemd/network/20-wired.network`
- **Type / Language**: `.network` (systemd-networkd)
- **Size**: 1,153 bytes
- **Classification**: `configuration` | Role: `os-image-or-kernel-config`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: LitricOS — wired network on the virtual (and any real) NIC. Without a .network match, systemd-networkd never configures an interface: the guest booted with a virtio-net device, a kernel driver (CONFIG_VIRTIO_NET is builtin), and no address — the browser engine could spawn Chromium but had nowhere to fetch from, and every page honestly reported a network error. DHCP=yes covers QEMU slirp (10.0.2.15/24, DNS 10.0.2.3) and any real LAN.
- **Subsystem**: `os-image-or-kernel-config`

### C. Internal Structure
- **Line Count**: 25 lines
- **Key Declarations / Directives / Functions**:
  - L12: `[Match]`
  - L13: `Name=en* eth*`
  - L15: `[Network]`
  - L22: `Address=10.0.2.15/24`
  - L23: `Gateway=10.0.2.2`
  - L24: `DNS=10.0.2.3`
  - L25: `IPv6AcceptRA=yes`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: None explicitly hardcoded.

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0557 — `image/mkosi.extra/usr/lib/systemd/oomd.conf.d/00-litric.conf`

### A. Identity
- **Path**: `image/mkosi.extra/usr/lib/systemd/oomd.conf.d/00-litric.conf`
- **Type / Language**: `.conf` (INI/Config)
- **Size**: 330 bytes
- **Classification**: `configuration` | Role: `os-image-or-kernel-config`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: PSI-based OOM management.  The point is to act BEFORE the kernel OOM killer, because oomd kills by policy (the greediest app in an eligible slice) and the kernel kills by heuristic (whatever scores worst, which is often the shell).
- **Subsystem**: `os-image-or-kernel-config`

### C. Internal Structure
- **Line Count**: 7 lines
- **Key Declarations / Directives / Functions**:
  - L4: `[OOM]`
  - L5: `SwapUsedLimit=95%`
  - L6: `DefaultMemoryPressureLimit=60%`
  - L7: `DefaultMemoryPressureDurationSec=15s`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: None explicitly hardcoded.

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0558 — `image/mkosi.extra/usr/lib/systemd/system/litric-ai.slice`

### A. Identity
- **Path**: `image/mkosi.extra/usr/lib/systemd/system/litric-ai.slice`
- **Type / Language**: `.slice` (systemd Slice)
- **Size**: 469 bytes
- **Classification**: `configuration` | Role: `os-image-or-kernel-config`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Budget line 11.  MemoryMax here bounds the SERVICE, not a loaded model; the router raises the limit transactionally when it admits a model and lowers it on unload.  See ADR-0015. Strictly below the shell.  Inference must never make the cursor stutter.
- **Subsystem**: `os-image-or-kernel-config`

### C. Internal Structure
- **Line Count**: 19 lines
- **Key Declarations / Directives / Functions**:
  - L1: `[Unit]`
  - L2: `Description=LitricOS AI service resource slice`
  - L3: `Before=slices.target`
  - L5: `[Slice]`
  - L9: `MemoryAccounting=yes`
  - L10: `MemoryHigh=10M`
  - L11: `MemoryMax=12M`
  - L14: `CPUAccounting=yes`
  - L15: `CPUWeight=10`
  - L17: `IOAccounting=yes`
  - L18: `IOWeight=10`
  - L19: `TasksMax=128`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: None explicitly hardcoded.

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0559 — `image/mkosi.extra/usr/lib/systemd/system/litric-app.slice`

### A. Identity
- **Path**: `image/mkosi.extra/usr/lib/systemd/system/litric-app.slice`
- **Type / Language**: `.slice` (systemd Slice)
- **Size**: 605 bytes
- **Classification**: `configuration` | Role: `os-image-or-kernel-config`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: No global cap: applications are what the machine is for.  Per-app limits come from the manifest resource-class (light/standard/game/ai) and are applied to the per-app scope by litric-capd at launch. oomd eligibility: when the machine is under real memory pressure, this is the slice that gives.  The shell and system services are protected.
- **Subsystem**: `os-image-or-kernel-config`

### C. Internal Structure
- **Line Count**: 19 lines
- **Key Declarations / Directives / Functions**:
  - L1: `[Unit]`
  - L2: `Description=LitricOS sandboxed application slice`
  - L3: `Before=slices.target`
  - L5: `[Slice]`
  - L6: `MemoryAccounting=yes`
  - L10: `CPUAccounting=yes`
  - L11: `CPUWeight=100`
  - L12: `IOAccounting=yes`
  - L13: `IOWeight=100`
  - L17: `ManagedOOMMemoryPressure=kill`
  - L18: `ManagedOOMMemoryPressureLimit=60%`
  - L19: `ManagedOOMSwap=kill`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: None explicitly hardcoded.

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0560 — `image/mkosi.extra/usr/lib/systemd/system/litric-boot-analysis.service`

### A. Identity
- **Path**: `image/mkosi.extra/usr/lib/systemd/system/litric-boot-analysis.service`
- **Type / Language**: `.service` (systemd Unit)
- **Size**: 322 bytes
- **Classification**: `configuration` | Role: `os-image-or-kernel-config`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Project os-image-or-kernel-config file (systemd Unit).
- **Subsystem**: `os-image-or-kernel-config`

### C. Internal Structure
- **Line Count**: 12 lines
- **Key Declarations / Directives / Functions**:
  - L1: `[Unit]`
  - L2: `Description=LitricOS boot time analysis`
  - L3: `After=multi-user.target`
  - L5: `[Service]`
  - L6: `Type=oneshot`
  - L7: `ExecStart=/bin/sh -c 'systemd-analyze > /dev/kmsg 2>&1'`
  - L8: `ExecStart=/bin/sh -c 'systemd-analyze blame > /dev/kmsg 2>&1'`
  - L9: `ExecStart=/bin/sh -c 'systemd-analyze critical-chain > /dev/kmsg 2>&1'`
  - L11: `[Install]`
  - L12: `WantedBy=multi-user.target`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/dev/kmsg`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0561 — `image/mkosi.extra/usr/lib/systemd/system/litric-browser.socket`

### A. Identity
- **Path**: `image/mkosi.extra/usr/lib/systemd/system/litric-browser.socket`
- **Type / Language**: `.socket` (systemd Socket)
- **Size**: 731 bytes
- **Classification**: `configuration` | Role: `os-image-or-kernel-config`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: The shell connects here. systemd owns the access check: the socket is 0600 root, the shell runs as root without capabilities, so owner permission lets it in and nothing else on the machine. One broker per connection, handed the connected socket as stdin/stdout, so the broker needs no fd passing and no unsafe code. Nothing runs, and no memory is used, until the browser window is opened.
- **Subsystem**: `os-image-or-kernel-config`

### C. Internal Structure
- **Line Count**: 22 lines
- **Key Declarations / Directives / Functions**:
  - L1: `[Unit]`
  - L2: `Description=Litric Browser engine broker socket`
  - L3: `Documentation=file:///usr/share/doc/litric/adr/0018-browser-engine-chromium.md`
  - L5: `[Socket]`
  - L9: `ListenStream=/run/litric/browser.sock`
  - L10: `SocketUser=root`
  - L11: `SocketGroup=root`
  - L12: `SocketMode=0600`
  - L13: `DirectoryMode=0755`
  - L17: `Accept=yes`
  - L18: `MaxConnections=1`
  - L19: `RemoveOnStop=yes`
  - L21: `[Install]`
  - L22: `WantedBy=sockets.target`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/run/litric/browser.sock`, `/usr/share/doc/litric/adr/0018-browser-engine-chromium.md`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0562 — `image/mkosi.extra/usr/lib/systemd/system/litric-browser@.service`

### A. Identity
- **Path**: `image/mkosi.extra/usr/lib/systemd/system/litric-browser@.service`
- **Type / Language**: `.service` (systemd Unit)
- **Size**: 2,295 bytes
- **Classification**: `configuration` | Role: `os-image-or-kernel-config`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Chromium refuses its sandbox as uid 0; the broker and the engine run as the desktop user. The engine's writable state lives on /run: the root filesystem is erofs behind dm-verity and cannot grow a /var/home, and a browser profile is cache-class state anyway — honest to be ephemeral on this image. systemd creates the directory (owned by User=) before the mount namespace is set
- **Subsystem**: `os-image-or-kernel-config`

### C. Internal Structure
- **Line Count**: 61 lines
- **Key Declarations / Directives / Functions**:
  - L1: `[Unit]`
  - L2: `Description=Litric Browser engine broker (connection %i)`
  - L3: `Documentation=file:///usr/share/doc/litric/adr/0018-browser-engine-chromium.md`
  - L4: `CollectMode=inactive-or-failed`
  - L6: `[Service]`
  - L7: `Type=simple`
  - L8: `ExecStart=/usr/bin/litric-browserd --stdio --profile /run/litric-browser/profile --downloads /run/li`
  - L9: `StandardInput=socket`
  - L10: `StandardOutput=socket`
  - L11: `StandardError=journal`
  - L15: `User=litric`
  - L16: `Group=litric`
  - L23: `RuntimeDirectory=litric-browser`
  - L24: `RuntimeDirectoryMode=0700`
  - L25: `RuntimeDirectoryPreserve=yes`
  - L28: `NoNewPrivileges=yes`
  - L29: `CapabilityBoundingSet=`
  - L30: `AmbientCapabilities=`
  - L31: `ProtectSystem=strict`
  - L32: `ProtectHome=yes`
  - L33: `PrivateTmp=yes`
  - L34: `PrivateDevices=yes`
  - L35: `ProtectKernelTunables=yes`
  - L36: `ProtectKernelModules=yes`
  - L37: `ProtectKernelLogs=yes`
  - L38: `ProtectControlGroups=yes`
  - L39: `ProtectClock=yes`
  - L40: `ProtectHostname=yes`
  - L41: `RestrictRealtime=yes`
  - L42: `RestrictSUIDSGID=yes`
  - L43: `LockPersonality=yes`
  - L44: `SystemCallArchitectures=native`
  - L46: `RestrictAddressFamilies=AF_UNIX AF_INET AF_INET6 AF_NETLINK`
  - L48: `ReadWritePaths=/run/litric-browser`
  - L60: `MemoryMax=1536M`
  - L61: `TasksMax=512`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/run/litric-browser`, `/run/litric-browser/Downloads`, `/run/litric-browser/profile`, `/usr/bin/litric-browserd`, `/usr/share/doc/litric/adr/0018-browser-engine-chromium.md`, `/var/home`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0563 — `image/mkosi.extra/usr/lib/systemd/system/litric-drop-caches.service`

### A. Identity
- **Path**: `image/mkosi.extra/usr/lib/systemd/system/litric-drop-caches.service`
- **Type / Language**: `.service` (systemd Unit)
- **Size**: 346 bytes
- **Classification**: `configuration` | Role: `os-image-or-kernel-config`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Project os-image-or-kernel-config file (systemd Unit).
- **Subsystem**: `os-image-or-kernel-config`

### C. Internal Structure
- **Line Count**: 12 lines
- **Key Declarations / Directives / Functions**:
  - L1: `[Unit]`
  - L2: `Description=Drop page cache for low idle RAM`
  - L3: `After=multi-user.target`
  - L4: `Documentation=https://www.kernel.org/doc/Documentation/sysctl/vm.txt`
  - L6: `[Service]`
  - L7: `Type=oneshot`
  - L8: `ExecStart=/bin/sh -c 'echo 3 > /proc/sys/vm/drop_caches'`
  - L9: `ExecStart=/bin/sh -c 'cat /proc/meminfo | head -5 | logger -t litric-drop-caches'`
  - L11: `[Install]`
  - L12: `WantedBy=multi-user.target`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/proc/meminfo`, `/proc/sys/vm/drop_caches`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0564 — `image/mkosi.extra/usr/lib/systemd/system/litric-envd.service`

### A. Identity
- **Path**: `image/mkosi.extra/usr/lib/systemd/system/litric-envd.service`
- **Type / Language**: `.service` (systemd Unit)
- **Size**: 2,434 bytes
- **Classification**: `configuration` | Role: `os-image-or-kernel-config`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: A guest is launched only onto a system that is up: the daemon derives its launch plans from budgets the manager approved, and the manager runs in litric-sysd. Ordering after sysd means a boot never shows a switcher whose backend does not exist yet. /run/litric is declared by litric-sysd too; the Preserve on that unit is what keeps this one's status files alive across either daemon restarting.
- **Subsystem**: `os-image-or-kernel-config`

### C. Internal Structure
- **Line Count**: 71 lines
- **Key Declarations / Directives / Functions**:
  - L1: `[Unit]`
  - L2: `Description=LitricOS environment daemon (guest supervisor)`
  - L3: `Documentation=file:///usr/share/doc/litric/adr/0015-multi-environment-switching.md`
  - L5: `After=basic.target`
  - L11: `After=litric-sysd.service`
  - L13: `[Service]`
  - L14: `Type=simple`
  - L15: `ExecStart=/usr/bin/litric-envd`
  - L19: `RuntimeDirectory=litric`
  - L20: `RuntimeDirectoryPreserve=yes`
  - L22: `StandardOutput=journal+console`
  - L23: `StandardError=journal+console`
  - L25: `Restart=on-failure`
  - L26: `RestartSec=1s`
  - L35: `NoNewPrivileges=yes`
  - L36: `CapabilityBoundingSet=`
  - L37: `AmbientCapabilities=`
  - L38: `ProtectSystem=strict`
  - L39: `ProtectHome=yes`
  - L40: `PrivateTmp=yes`
  - L45: `PrivateDevices=no`
  - L46: `DeviceAllow=char-kvm rw`
  - L47: `ProtectKernelTunables=yes`
  - L48: `ProtectKernelModules=yes`
  - L49: `ProtectKernelLogs=yes`
  - L50: `ProtectControlGroups=yes`
  - L51: `ProtectClock=yes`
  - L52: `ProtectHostname=yes`
  - L53: `RestrictNamespaces=yes`
  - L54: `RestrictRealtime=yes`
  - L55: `RestrictSUIDSGID=yes`
  - L56: `LockPersonality=yes`
  - L57: `MemoryDenyWriteExecute=yes`
  - L58: `SystemCallArchitectures=native`
  - L59: `SystemCallFilter=@system-service`
  - L60: `SystemCallErrorNumber=EPERM`
  - L63: `RestrictAddressFamilies=AF_UNIX`
  - L64: `IPAddressDeny=any`
  - L67: `MemoryMax=128M`
  - L68: `TasksMax=128`
  - L70: `[Install]`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/dev/kvm`, `/run/litric`, `/usr/bin/litric-envd`, `/usr/share/doc/litric/adr/0015-multi-environment-switching.md`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0565 — `image/mkosi.extra/usr/lib/systemd/system/litric-etc-overlay.service`

### A. Identity
- **Path**: `image/mkosi.extra/usr/lib/systemd/system/litric-etc-overlay.service`
- **Type / Language**: `.service` (systemd Unit)
- **Size**: 1,067 bytes
- **Classification**: `configuration` | Role: `os-image-or-kernel-config`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: mkosi finalizes /etc/passwd, /etc/shadow and locale after postinst.chroot. Snapshot the completed image at first boot instead, before mounting over it.
- **Subsystem**: `os-image-or-kernel-config`

### C. Internal Structure
- **Line Count**: 19 lines
- **Key Declarations / Directives / Functions**:
  - L1: `[Unit]`
  - L2: `Description=Mount persistent LitricOS configuration overlay`
  - L3: `DefaultDependencies=no`
  - L4: `Requires=var.mount`
  - L5: `After=var.mount`
  - L6: `Before=local-fs.target`
  - L8: `[Service]`
  - L9: `Type=oneshot`
  - L10: `RemainAfterExit=yes`
  - L11: `ExecStartPre=/usr/bin/mkdir -p /var/lib/litric/etc-overlay/lower /var/lib/litric/etc-overlay/upper /`
  - L12: `ExecStartPre=/usr/bin/chmod 0700 /var/lib/litric/etc-overlay /var/lib/litric/etc-overlay/lower /var/`
  - L15: `ExecStartPre=/usr/bin/bash -ec 'if [ ! -e /var/lib/litric/etc-overlay/.initialized ]; then /usr/bin/`
  - L16: `ExecStart=/usr/bin/mount -t overlay overlay -o lowerdir=/var/lib/litric/etc-overlay/lower,upperdir=/`
  - L18: `[Install]`
  - L19: `WantedBy=local-fs.target`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/usr/bin/bash`, `/usr/bin/chmod`, `/usr/bin/cp`, `/usr/bin/mkdir`, `/usr/bin/mount`, `/usr/bin/touch`, `/var/lib/litric/etc-overlay`, `/var/lib/litric/etc-overlay/.initialized`, `/var/lib/litric/etc-overlay/lower`, `/var/lib/litric/etc-overlay/lower/`, `/var/lib/litric/etc-overlay/upper`, `/var/lib/litric/etc-overlay/work`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0566 — `image/mkosi.extra/usr/lib/systemd/system/litric-first-run.service`

### A. Identity
- **Path**: `image/mkosi.extra/usr/lib/systemd/system/litric-first-run.service`
- **Type / Language**: `.service` (systemd Unit)
- **Size**: 801 bytes
- **Classification**: `configuration` | Role: `os-image-or-kernel-config`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: The script runs on the framebuffer console, whose terminfo entry is `linux`. Without TERM, anything that asks the terminal what it is gets nothing back -- and the image ships no terminfo database for it to answer with either, which is why the script no longer calls `clear` at all.
- **Subsystem**: `os-image-or-kernel-config`

### C. Internal Structure
- **Line Count**: 27 lines
- **Key Declarations / Directives / Functions**:
  - L1: `[Unit]`
  - L2: `Description=Set the LitricOS desktop password on first boot`
  - L3: `Requires=litric-etc-overlay.service`
  - L4: `After=local-fs.target litric-etc-overlay.service`
  - L5: `Before=litric-shell.service`
  - L6: `Conflicts=getty@tty1.service`
  - L7: `StartLimitIntervalSec=0`
  - L9: `[Service]`
  - L10: `Type=oneshot`
  - L15: `Environment=TERM=linux`
  - L16: `ExecStart=/usr/bin/bash /usr/lib/litric/first-run.sh`
  - L17: `StandardInput=tty-force`
  - L18: `StandardOutput=tty`
  - L19: `StandardError=tty`
  - L20: `TTYPath=/dev/tty1`
  - L21: `TTYReset=yes`
  - L22: `TTYVHangup=yes`
  - L23: `Restart=on-failure`
  - L24: `RestartSec=2s`
  - L26: `[Install]`
  - L27: `WantedBy=multi-user.target`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/dev/tty1`, `/usr/bin/bash`, `/usr/lib/litric/first-run.sh`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0567 — `image/mkosi.extra/usr/lib/systemd/system/litric-game.slice`

### A. Identity
- **Path**: `image/mkosi.extra/usr/lib/systemd/system/litric-game.slice`
- **Type / Language**: `.slice` (systemd Slice)
- **Size**: 487 bytes
- **Classification**: `configuration` | Role: `os-image-or-kernel-config`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Just under the shell.  A game should get everything except the ability to stop the compositor from presenting — that is how you get a frozen machine that "runs at 200 FPS". Games legitimately allocate enormous amounts; killing one on pressure loses unsaved progress.  We shed elsewhere first.
- **Subsystem**: `os-image-or-kernel-config`

### C. Internal Structure
- **Line Count**: 16 lines
- **Key Declarations / Directives / Functions**:
  - L1: `[Unit]`
  - L2: `Description=LitricOS game slice`
  - L3: `Before=slices.target`
  - L5: `[Slice]`
  - L6: `MemoryAccounting=yes`
  - L7: `CPUAccounting=yes`
  - L11: `CPUWeight=800`
  - L12: `IOAccounting=yes`
  - L13: `IOWeight=800`
  - L16: `ManagedOOMMemoryPressure=auto`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: None explicitly hardcoded.

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0568 — `image/mkosi.extra/usr/lib/systemd/system/litric-mm-tune.service`

### A. Identity
- **Path**: `image/mkosi.extra/usr/lib/systemd/system/litric-mm-tune.service`
- **Type / Language**: `.service` (systemd Unit)
- **Size**: 1,234 bytes
- **Classification**: `configuration` | Role: `os-image-or-kernel-config`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Failing to tune must never fail the boot. A machine that boots slightly less optimally is fine; a machine that does not boot is not. --- sandbox --------------------------------------------------------------- This service writes to /sys/kernel/mm and nothing else. THREAT ACCEPTED: ProtectKernelTunables would block the writes this service exists to make. Scope is narrowed to exactly one path by ReadWritePaths.
- **Subsystem**: `os-image-or-kernel-config`

### C. Internal Structure
- **Line Count**: 41 lines
- **Key Declarations / Directives / Functions**:
  - L1: `[Unit]`
  - L2: `Description=LitricOS memory-management runtime tuning (MGLRU, THP, zswap off)`
  - L3: `Documentation=file:///usr/share/doc/litric/PERF-budget.md`
  - L4: `DefaultDependencies=no`
  - L5: `After=sysinit.target`
  - L6: `Before=basic.target`
  - L7: `ConditionPathExists=/sys/kernel/mm`
  - L9: `[Service]`
  - L10: `Type=oneshot`
  - L11: `RemainAfterExit=yes`
  - L12: `ExecStart=/usr/lib/litric/mm-tune`
  - L15: `SuccessExitStatus=0 1`
  - L19: `NoNewPrivileges=yes`
  - L20: `ProtectSystem=strict`
  - L21: `ProtectHome=yes`
  - L22: `PrivateTmp=yes`
  - L23: `PrivateNetwork=yes`
  - L24: `PrivateDevices=yes`
  - L25: `ProtectKernelModules=yes`
  - L26: `ProtectControlGroups=yes`
  - L27: `RestrictAddressFamilies=`
  - L28: `RestrictNamespaces=yes`
  - L29: `RestrictRealtime=yes`
  - L30: `LockPersonality=yes`
  - L31: `MemoryDenyWriteExecute=yes`
  - L32: `SystemCallFilter=@system-service`
  - L33: `SystemCallArchitectures=native`
  - L36: `ProtectKernelTunables=no`
  - L37: `ReadWritePaths=/sys/kernel/mm`
  - L38: `CapabilityBoundingSet=CAP_SYS_ADMIN`
  - L40: `[Install]`
  - L41: `WantedBy=sysinit.target`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/sys/kernel/mm`, `/usr/lib/litric/mm-tune`, `/usr/share/doc/litric/PERF-budget.md`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0569 — `image/mkosi.extra/usr/lib/systemd/system/litric-selftest.service`

### A. Identity
- **Path**: `image/mkosi.extra/usr/lib/systemd/system/litric-selftest.service`
- **Type / Language**: `.service` (systemd Unit)
- **Size**: 1,580 bytes
- **Classification**: `configuration` | Role: `os-image-or-kernel-config`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Ordering note, learned the hard way. This previously said `After=multi-user.target` together with `WantedBy=multi-user.target`. That is a circular arrangement: the target pulls the unit in as a dependency, while the unit orders itself after the target that is waiting for it. systemd breaks the cycle by dropping the ordering and the unit simply never ran -- enabled, condition satisfied, and silent.
- **Subsystem**: `os-image-or-kernel-config`

### C. Internal Structure
- **Line Count**: 38 lines
- **Key Declarations / Directives / Functions**:
  - L1: `[Unit]`
  - L2: `Description=LitricOS self test and measurement`
  - L3: `Documentation=file:///usr/share/doc/litric/PERF-budget.md`
  - L15: `After=basic.target`
  - L20: `ConditionKernelCommandLine=litric.selftest`
  - L22: `[Service]`
  - L23: `Type=oneshot`
  - L24: `RemainAfterExit=yes`
  - L25: `ExecStart=/usr/lib/litric/selftest`
  - L28: `StandardOutput=journal+console`
  - L29: `StandardError=journal+console`
  - L30: `TimeoutStartSec=180`
  - L37: `[Install]`
  - L38: `WantedBy=multi-user.target`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/usr/lib/litric/selftest`, `/usr/share/doc/litric/PERF-budget.md`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0570 — `image/mkosi.extra/usr/lib/systemd/system/litric-shell.service`

### A. Identity
- **Path**: `image/mkosi.extra/usr/lib/systemd/system/litric-shell.service`
- **Type / Language**: `.service` (systemd Unit)
- **Size**: 4,772 bytes
- **Classification**: `configuration` | Role: `os-image-or-kernel-config`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: HONEST SCOPE, 2026-08-29. This unit previously described the compositor in ADR-0009: Type=notify, a `litric` user, a PAM session, seatd, DRM and input device access, a watchdog. None of that existed, and the unit could not start; it was a description of an intention wearing the clothes of a service. What exists now is the display half plus a real input stack: litric-shell
- **Subsystem**: `os-image-or-kernel-config`

### C. Internal Structure
- **Line Count**: 113 lines
- **Key Declarations / Directives / Functions**:
  - L1: `[Unit]`
  - L2: `Description=LitricOS desktop shell`
  - L3: `Documentation=file:///usr/share/doc/litric/adr/0017-framebuffer-shell.md`
  - L4: `Slice=litric-shell.slice`
  - L21: `After=basic.target litric-sysd.service litric-first-run.service`
  - L22: `Requires=litric-first-run.service`
  - L23: `Wants=litric-sysd.service`
  - L28: `Conflicts=getty@tty1.service`
  - L32: `ConditionPathExists=/dev/fb0`
  - L34: `[Service]`
  - L35: `Type=simple`
  - L39: `ExecStart=/usr/bin/litric-shell`
  - L45: `StateDirectory=litric-shell`
  - L46: `StateDirectoryMode=0700`
  - L48: `RuntimeDirectory=litric`
  - L49: `RuntimeDirectoryMode=0755`
  - L55: `RuntimeDirectoryPreserve=yes`
  - L56: `StandardOutput=journal+console`
  - L57: `StandardError=journal+console`
  - L59: `Restart=on-failure`
  - L60: `RestartSec=1s`
  - L72: `NoNewPrivileges=yes`
  - L73: `CapabilityBoundingSet=`
  - L74: `AmbientCapabilities=`
  - L75: `PrivateDevices=no`
  - L76: `DeviceAllow=/dev/fb0 rw`
  - L77: `DeviceAllow=char-drm rw`
  - L80: `DeviceAllow=char-input r`
  - L81: `ProtectSystem=strict`
  - L82: `ProtectHome=yes`
  - L83: `PrivateTmp=yes`
  - L84: `ProtectKernelModules=yes`
  - L85: `ProtectKernelLogs=yes`
  - L86: `ProtectControlGroups=yes`
  - L87: `ProtectClock=yes`
  - L88: `ProtectHostname=yes`
  - L89: `RestrictNamespaces=yes`
  - L90: `RestrictRealtime=yes`
  - L91: `RestrictSUIDSGID=yes`
  - L92: `LockPersonality=yes`
  - L112: `[Install]`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/dev/fb0`, `/run/litric.`, `/sys/class/vtconsole/vtconN/bind.`, `/usr/bin/litric-shell`, `/usr/share/doc/litric/adr/0017-framebuffer-shell.md`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0571 — `image/mkosi.extra/usr/lib/systemd/system/litric-shell.slice`

### A. Identity
- **Path**: `image/mkosi.extra/usr/lib/systemd/system/litric-shell.slice`
- **Type / Language**: `.slice` (systemd Slice)
- **Size**: 599 bytes
- **Classification**: `configuration` | Role: `os-image-or-kernel-config`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Budget line 8.  MemoryHigh throttles and reclaims; MemoryMax kills.  The gap between them is deliberate: a shell that briefly spikes (a large wallpaper, a 4K screenshot) should be reclaimed, not killed. The highest weight on the system.  If anything is going to get CPU during contention, it is the thing drawing the frame the user is looking at.
- **Subsystem**: `os-image-or-kernel-config`

### C. Internal Structure
- **Line Count**: 21 lines
- **Key Declarations / Directives / Functions**:
  - L1: `[Unit]`
  - L2: `Description=LitricOS shell and compositor resource slice`
  - L3: `Before=slices.target`
  - L5: `[Slice]`
  - L9: `MemoryAccounting=yes`
  - L10: `MemoryHigh=110M`
  - L11: `MemoryMax=140M`
  - L12: `MemorySwapMax=64M`
  - L16: `CPUAccounting=yes`
  - L17: `CPUWeight=1000`
  - L19: `IOAccounting=yes`
  - L20: `IOWeight=500`
  - L21: `TasksMax=512`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: None explicitly hardcoded.

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0572 — `image/mkosi.extra/usr/lib/systemd/system/litric-sysd.service`

### A. Identity
- **Path**: `image/mkosi.extra/usr/lib/systemd/system/litric-sysd.service`
- **Type / Language**: `.service` (systemd Unit)
- **Size**: 4,613 bytes
- **Classification**: `configuration` | Role: `os-image-or-kernel-config`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: After basic.target, not multi-user.target. Ordering a unit after the target that pulls it in is a dependency cycle; systemd resolves it by dropping the ordering and the unit never runs -- enabled, no error, no output. That exact mistake already cost this project a debugging session on litric-selftest. Nothing above this exists yet. When litric-shell does, it will order itself after this unit, because a shell that draws a trash icon before there is a
- **Subsystem**: `os-image-or-kernel-config`

### C. Internal Structure
- **Line Count**: 101 lines
- **Key Declarations / Directives / Functions**:
  - L1: `[Unit]`
  - L2: `Description=LitricOS system daemon`
  - L3: `Documentation=file:///usr/share/doc/litric/adr/0016-system-daemon.md`
  - L9: `After=basic.target`
  - L15: `[Service]`
  - L16: `Type=simple`
  - L17: `ExecStart=/usr/bin/litric-sysd --serve --socket /run/litric/sysd.sock`
  - L21: `RuntimeDirectory=litric`
  - L22: `RuntimeDirectoryMode=0755`
  - L28: `RuntimeDirectoryPreserve=yes`
  - L33: `StandardOutput=journal+console`
  - L34: `StandardError=journal+console`
  - L36: `Restart=on-failure`
  - L37: `RestartSec=1s`
  - L54: `NoNewPrivileges=yes`
  - L55: `CapabilityBoundingSet=CAP_NET_ADMIN`
  - L59: `AmbientCapabilities=CAP_NET_ADMIN`
  - L60: `ProtectSystem=strict`
  - L61: `ProtectHome=yes`
  - L68: `ReadWritePaths=/var/lib/iwd`
  - L69: `PrivateTmp=yes`
  - L70: `PrivateDevices=yes`
  - L71: `ProtectKernelTunables=yes`
  - L72: `ProtectKernelModules=yes`
  - L73: `ProtectKernelLogs=yes`
  - L74: `ProtectControlGroups=yes`
  - L75: `ProtectClock=yes`
  - L76: `ProtectHostname=yes`
  - L77: `RestrictNamespaces=yes`
  - L78: `RestrictRealtime=yes`
  - L79: `RestrictSUIDSGID=yes`
  - L80: `LockPersonality=yes`
  - L84: `MemoryDenyWriteExecute=yes`
  - L85: `SystemCallArchitectures=native`
  - L86: `SystemCallFilter=@system-service`
  - L87: `SystemCallErrorNumber=EPERM`
  - L91: `RestrictAddressFamilies=AF_UNIX AF_NETLINK`
  - L92: `IPAddressDeny=any`
  - L97: `MemoryMax=64M`
  - L98: `TasksMax=64`
  - L100: `[Install]`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/run/litric.`, `/run/litric/sysd.sock`, `/sys/kernel/mm`, `/usr/bin/ip`, `/usr/bin/litric-sysd`, `/usr/share/doc/litric/adr/0016-system-daemon.md`, `/var/lib/iwd`, `image/mkosi.extra/usr/lib/tmpfiles.d/litric-iwd.conf`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0573 — `image/mkosi.extra/usr/lib/systemd/system/serial-getty@ttyS1.service.d/autologin.conf`

### A. Identity
- **Path**: `image/mkosi.extra/usr/lib/systemd/system/serial-getty@ttyS1.service.d/autologin.conf`
- **Type / Language**: `.conf` (INI/Config)
- **Size**: 846 bytes
- **Classification**: `configuration` | Role: `os-image-or-kernel-config`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: LitricOS — lab diagnostic console on the second serial port. The image's root filesystem is immutable (erofs + dm-verity) and root's password is locked, so nothing on the box could answer "why did this unit die" — every diagnosis ended as a guess from one-way serial prints. The desktop image is a disposable lab artifact driven by QEMU, the console is reachable only through a host-side socket this test harness creates, and
- **Subsystem**: `os-image-or-kernel-config`

### C. Internal Structure
- **Line Count**: 16 lines
- **Key Declarations / Directives / Functions**:
  - L14: `[Service]`
  - L15: `ExecStart=`
  - L16: `ExecStart=-/usr/bin/agetty --autologin root --noclear --keep-baud 115200,57600,38400,9600 %I vt100`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/usr/bin/agetty`, `kernel/system`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0574 — `image/mkosi.extra/usr/lib/systemd/zram-generator.conf`

### A. Identity
- **Path**: `image/mkosi.extra/usr/lib/systemd/zram-generator.conf`
- **Type / Language**: `.conf` (INI/Config)
- **Size**: 785 bytes
- **Classification**: `configuration` | Role: `os-image-or-kernel-config`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: zram, not disk swap.  On a machine with a read-only erofs system partition and a Btrfs user partition, disk swap buys latency problems and SSD wear; zram buys effective memory at the cost of CPU we have. zram-size = ram: the kernel accounts zram at its uncompressed size, and real zstd ratios land near 3:1-5:1, so effective headroom is larger than the number suggests.  Measured ratios go in docs/EVIDENCE.md, never in a claim.
- **Subsystem**: `os-image-or-kernel-config`

### C. Internal Structure
- **Line Count**: 17 lines
- **Key Declarations / Directives / Functions**:
  - L13: `[zram0]`
  - L14: `zram-size = ram`
  - L15: `compression-algorithm = zstd`
  - L16: `swap-priority = 100`
  - L17: `fs-type = swap`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `docs/EVIDENCE.md`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0575 — `image/mkosi.extra/usr/lib/sysusers.d/litric.conf`

### A. Identity
- **Path**: `image/mkosi.extra/usr/lib/sysusers.d/litric.conf`
- **Type / Language**: `.conf` (INI/Config)
- **Size**: 547 bytes
- **Classification**: `configuration` | Role: `os-image-or-kernel-config`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: The desktop user. Programs that act on the user's behalf -- the browser engine broker (ADR-0018) and the file daemon (ADR-0019) -- run as this account, never as root. Chromium refuses to start its sandbox as uid 0, and a file daemon that runs as root turns every path-confinement bug into a system compromise. The root filesystem is read-only erofs, so this cannot be applied at boot;
- **Subsystem**: `os-image-or-kernel-config`

### C. Internal Structure
- **Line Count**: 11 lines

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/usr/bin/bash`, `/var/home/litric`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0576 — `image/mkosi.extra/usr/lib/tmpfiles.d/litric-dns.conf`

### A. Identity
- **Path**: `image/mkosi.extra/usr/lib/tmpfiles.d/litric-dns.conf`
- **Type / Language**: `.conf` (INI/Config)
- **Size**: 495 bytes
- **Classification**: `configuration` | Role: `os-image-or-kernel-config`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: LitricOS — DNS resolver plumbing, recreated at every boot. systemd-resolved answers on 127.0.0.53 (the stub). /etc is regenerated by a later mkosi phase and a symlink committed from the Windows host would arrive as a plain file anyway, so the link is created by tmpfiles at boot instead — the distro-standard way to keep /etc/resolv.conf pointing at the stub even on a read-only or freshly-regenerated root.
- **Subsystem**: `os-image-or-kernel-config`

### C. Internal Structure
- **Line Count**: 9 lines

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/run/systemd/resolve/stub-resolv.conf`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0577 — `image/mkosi.extra/usr/lib/tmpfiles.d/litric-home.conf`

### A. Identity
- **Path**: `image/mkosi.extra/usr/lib/tmpfiles.d/litric-home.conf`
- **Type / Language**: `.conf` (INI/Config)
- **Size**: 520 bytes
- **Classification**: `configuration` | Role: `os-image-or-kernel-config`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: LitricOS — the browser engine's runtime directories. The root filesystem is erofs behind dm-verity: /var/home cannot be created at boot and the engine's writable state belongs on /run instead. The unit itself declares RuntimeDirectory=litric-browser, which is the authoritative mechanism and creates the profile root owned by User= before the mount namespace is set up. This file exists only for the socket's parent
- **Subsystem**: `os-image-or-kernel-config`

### C. Internal Structure
- **Line Count**: 10 lines

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/run/litric`, `/var/home`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0578 — `image/mkosi.extra/usr/lib/tmpfiles.d/litric-iwd.conf`

### A. Identity
- **Path**: `image/mkosi.extra/usr/lib/tmpfiles.d/litric-iwd.conf`
- **Type / Language**: `.conf` (INI/Config)
- **Size**: 926 bytes
- **Classification**: `configuration` | Role: `os-image-or-kernel-config`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: LitricOS — the directory iwd keeps network credentials in. `NET WIFI JOIN` writes `<ssid>.psk` here, and `litric-sysd.service` opens exactly this one path for writing (ReadWritePaths=/var/lib/iwd) because ProtectSystem=strict makes /var read-only inside its mount namespace. systemd refuses to start a unit whose ReadWritePaths does not exist — it is mount-namespace setup, not a warning — so the directory must be there before
- **Subsystem**: `os-image-or-kernel-config`

### C. Internal Structure
- **Line Count**: 18 lines

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/var/lib/iwd`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0579 — `image/mkosi.initrd.conf`

### A. Identity
- **Path**: `image/mkosi.initrd.conf`
- **Type / Language**: `.conf` (INI/Config)
- **Size**: 415 bytes
- **Classification**: `configuration` | Role: `os-image-or-kernel-config`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Files in mkosi.extra/ are copied into the final immutable root image, after the default initrd has already been assembled.  The dm-verity readiness rule has to run before that root is mounted, so give the default initrd its own explicit extra tree.  mkosi reads this file only while it builds the default initrd; it is deliberately separate from image/mkosi.conf.
- **Subsystem**: `os-image-or-kernel-config`

### C. Internal Structure
- **Line Count**: 8 lines
- **Key Declarations / Directives / Functions**:
  - L7: `[Content]`
  - L8: `ExtraTrees=mkosi.initrd.extra`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `image/mkosi.conf.`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0580 — `image/mkosi.initrd.extra/usr/lib/udev/rules.d/99-zlitric-root-verity-ready.rules`

### A. Identity
- **Path**: `image/mkosi.initrd.extra/usr/lib/udev/rules.d/99-zlitric-root-verity-ready.rules`
- **Type / Language**: `.rules` (udev Rules)
- **Size**: 554 bytes
- **Classification**: `configuration` | Role: `os-image-or-kernel-config`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: LitricOS root dm-verity readiness cryptsetup first creates an empty device-mapper container, then changes it into the verified root mapping.  systemd's stock rule has already recorded the empty container as not ready, and the change event asks udev not to run unrelated rules.  Tell systemd about exactly the mapping that veritysetup just created.  The guards intentionally exclude all other dm devices.
- **Subsystem**: `os-image-or-kernel-config`

### C. Internal Structure
- **Line Count**: 8 lines

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: None explicitly hardcoded.

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0581 — `image/mkosi.postinst.chroot`

### A. Identity
- **Path**: `image/mkosi.postinst.chroot`
- **Type / Language**: `.chroot` (Shell (mkosi chroot))
- **Size**: 8,212 bytes
- **Classification**: `configuration` | Role: `os-image-or-kernel-config`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: ============================================================================= LitricOS — post-installation, executed INSIDE the image The `.chroot` suffix is load-bearing. mkosi.postinst          runs on the BUILD HOST, image mounted at $BUILDROOT. A bare path like /etc/foo means the BUILD HOST's /etc. mkosi.postinst.chroot   runs INSIDE the image. A bare path means the image.
- **Subsystem**: `os-image-or-kernel-config`

### C. Internal Structure
- **Line Count**: 154 lines
- **Key Declarations / Directives / Functions**:
  - L21: `say() { printf '\033[1;35m[postinst]\033[0m %s\n' "$*"; }`
  - L22: `die() { printf '\033[1;31m[postinst]\033[0m %s\n' "$*" >&2; exit 1; }`
  - L44: `enable_unit() {`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/dev/mapper/root`, `/dev/null`, `/usr/bin`, `/usr/bin/env`, `/usr/bin/litric-`, `/usr/bin/litric-filesd`, `/usr/lib`, `/usr/lib/litric/`, `/usr/lib/litric/selftest`, `/usr/lib/modprobe.d`, `/usr/lib/modules-load.d`, `/usr/lib/sysctl.d`, `/usr/lib/systemd`, `/usr/lib/systemd/system/`, `/usr/lib/systemd/system/getty.target.wants`, `/usr/lib/systemd/system/getty.target.wants/serial-getty@ttyS1.service`, `/usr/lib/systemd/system/serial-getty@.service`, `/usr/lib/tmpfiles.d`, `/usr/lib/udev`, `/var/lib/iwd`, `image/units-pending/`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0582 — `image/mkosi.profiles/core.conf`

### A. Identity
- **Path**: `image/mkosi.profiles/core.conf`
- **Type / Language**: `.conf` (INI/Config)
- **Size**: 975 bytes
- **Classification**: `configuration` | Role: `os-image-or-kernel-config`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: ============================================================================= PROFILE: core — nano + device management + storage + network.  Headless. Budget: <= 95 MB idle, <= 2.0 s boot. ============================================================================= measurement tools — these are how the budget is enforced, so they are part of the product, not a debug extra
- **Subsystem**: `os-image-or-kernel-config`

### C. Internal Structure
- **Line Count**: 27 lines
- **Key Declarations / Directives / Functions**:
  - L5: `[Content]`
  - L6: `Packages=`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: None explicitly hardcoded.
- **Environment Variables / Flags**: `PROFILE`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0583 — `image/mkosi.profiles/desktop.conf`

### A. Identity
- **Path**: `image/mkosi.profiles/desktop.conf`
- **Type / Language**: `.conf` (INI/Config)
- **Size**: 4,120 bytes
- **Classification**: `configuration` | Role: `os-image-or-kernel-config`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: ============================================================================= PROFILE: desktop — the headline profile. Budget: <= 320 MB idle, <= 0.5% idle CPU, <= 3.5 s boot, <= 850 MB ISO. NOTE the absence of gtk3, gtk4, qt5, qt6, and any browser engine in this list. That absence is the budget (ADR-0009). This comment used to say ci/lint-budget.sh asserts it and fails the build when
- **Subsystem**: `os-image-or-kernel-config`

### C. Internal Structure
- **Line Count**: 90 lines
- **Key Declarations / Directives / Functions**:
  - L16: `[Content]`
  - L17: `Packages=`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `ci/lint-budget.sh`, `docs/GATES.md`, `docs/PERF-budget.md`, `docs/audit/2026-10-11-beta-1.0-stage-a-audit.md`, `image/units-pending/`
- **Environment Variables / Flags**: `PROFILE`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0584 — `image/mkosi.profiles/gaming.conf`

### A. Identity
- **Path**: `image/mkosi.profiles/gaming.conf`
- **Type / Language**: `.conf` (INI/Config)
- **Size**: 991 bytes
- **Classification**: `configuration` | Role: `os-image-or-kernel-config`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: ============================================================================= PROFILE: gaming — desktop + the Phase 12 stack. Budget: <= 420 MB idle.  Gaming Mode is OFF at boot; nothing here runs until the user turns it on, so the idle delta over `desktop` is disk, not RAM. Nothing in this profile touches KernelCommandLine.  Gaming Mode changes runtime state only, never boot state, so that toggling it off is a complete restore
- **Subsystem**: `os-image-or-kernel-config`

### C. Internal Structure
- **Line Count**: 21 lines
- **Key Declarations / Directives / Functions**:
  - L11: `[Content]`
  - L12: `Packages=`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: None explicitly hardcoded.
- **Environment Variables / Flags**: `PROFILE`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0585 — `image/mkosi.profiles/nano.conf`

### A. Identity
- **Path**: `image/mkosi.profiles/nano.conf`
- **Type / Language**: `.conf` (INI/Config)
- **Size**: 8,542 bytes
- **Classification**: `configuration` | Role: `os-image-or-kernel-config`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: ============================================================================= PROFILE: nano -- kernel + init + serial console.  CI and embedded. Budget: <= 55 MB idle, <= 1.2 s boot, <= 90 MB ISO. This profile is the memory floor.  Everything else is measured against it. NOTE: there is deliberately NO `Packages=` line here. =============================================================================
- **Subsystem**: `os-image-or-kernel-config`

### C. Internal Structure
- **Line Count**: 301 lines
- **Key Declarations / Directives / Functions**:
  - L9: `[Content]`
  - L13: `RemovePackages=`
  - L23: `RemoveFiles=`
  - L270: `KernelModules=`
  - L298: `RootPassword=litric`
  - L299: `KernelCommandLine=`
  - L300: `systemd.volatile=state`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/usr/bin/ac`, `/usr/bin/addr2line`, `/usr/bin/ar`, `/usr/bin/arp`, `/usr/bin/arping`, `/usr/bin/arpping`, `/usr/bin/as`, `/usr/bin/bridge`, `/usr/bin/bzip2`, `/usr/bin/c`, `/usr/bin/captree`, `/usr/bin/chattr`, `/usr/bin/clockdiff`, `/usr/bin/cpio`, `/usr/bin/cpufreq`, `/usr/bin/cpulimit`, `/usr/bin/csh`, `/usr/bin/curl`, `/usr/bin/dash`, `/usr/bin/debugfs`, `/usr/bin/dig`, `/usr/bin/dmidecode`, `/usr/bin/dump_ldconfig`, `/usr/bin/dumpe2fs`, `/usr/bin/dwarves`
- **Environment Variables / Flags**: `PROFILE`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0586 — `image/mkosi.profiles/nano/mkosi.repart/00-esp.conf`

### A. Identity
- **Path**: `image/mkosi.profiles/nano/mkosi.repart/00-esp.conf`
- **Type / Language**: `.conf` (INI/Config)
- **Size**: 249 bytes
- **Classification**: `configuration` | Role: `os-image-or-kernel-config`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Nano-specific ESP: 128MB is enough for a single UKI + bootloader. The base config uses 1GB which wastes space in CI images.
- **Subsystem**: `os-image-or-kernel-config`

### C. Internal Structure
- **Line Count**: 10 lines
- **Key Declarations / Directives / Functions**:
  - L3: `[Partition]`
  - L4: `Type=esp`
  - L5: `Label=LITRIC_ESP`
  - L6: `Format=vfat`
  - L7: `CopyFiles=/efi:/`
  - L8: `CopyFiles=/boot:/`
  - L9: `SizeMinBytes=128M`
  - L10: `SizeMaxBytes=128M`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: None explicitly hardcoded.
- **Environment Variables / Flags**: `LITRIC_ESP`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0587 — `image/mkosi.repart-nano/00-esp.conf`

### A. Identity
- **Path**: `image/mkosi.repart-nano/00-esp.conf`
- **Type / Language**: `.conf` (INI/Config)
- **Size**: 280 bytes
- **Classification**: `configuration` | Role: `os-image-or-kernel-config`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Nano ESP: 128MB — holds systemd-boot + a single UKI + boot counters. The base config uses 1 GB which wastes ~900 MB in CI images.
- **Subsystem**: `os-image-or-kernel-config`

### C. Internal Structure
- **Line Count**: 11 lines
- **Key Declarations / Directives / Functions**:
  - L3: `[Partition]`
  - L4: `Type=esp`
  - L5: `Label=LITRIC_ESP`
  - L6: `Format=vfat`
  - L7: `CopyFiles=/efi:/`
  - L8: `CopyFiles=/boot:/`
  - L9: `SizeMinBytes=128M`
  - L10: `SizeMaxBytes=128M`
  - L11: `MountPoint=/efi:nofail`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: None explicitly hardcoded.
- **Environment Variables / Flags**: `LITRIC_ESP`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0588 — `image/mkosi.repart-nano/20-var.conf`

### A. Identity
- **Path**: `image/mkosi.repart-nano/20-var.conf`
- **Type / Language**: `.conf` (INI/Config)
- **Size**: 182 bytes
- **Classification**: `configuration` | Role: `os-image-or-kernel-config`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Nano var: 64 MB — tmpfs-backed (systemd.volatile=state), no persistent storage needed.
- **Subsystem**: `os-image-or-kernel-config`

### C. Internal Structure
- **Line Count**: 7 lines
- **Key Declarations / Directives / Functions**:
  - L2: `[Partition]`
  - L3: `Type=var`
  - L4: `Label=LITRIC_VAR`
  - L5: `Format=btrfs`
  - L6: `SizeMinBytes=64M`
  - L7: `MountPoint=/var:nofail`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: None explicitly hardcoded.
- **Environment Variables / Flags**: `LITRIC_VAR`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0589 — `image/mkosi.repart/00-esp.conf`

### A. Identity
- **Path**: `image/mkosi.repart/00-esp.conf`
- **Type / Language**: `.conf` (INI/Config)
- **Size**: 3,478 bytes
- **Classification**: `configuration` | Role: `os-image-or-kernel-config`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: EFI System Partition -- holds systemd-boot + the UKI + boot counters. TWO CopyFiles lines, and both are required. mkosi builds a SPLIT boot layout inside the image: /efi   -> the bootloader:  EFI/BOOT/BOOTX64.EFI, EFI/systemd/, loader/keys/ /boot  -> the kernels:     EFI/Linux/<uki>.efi,  loader/entries/ That split exists so a system can have a small ESP plus a larger XBOOTLDR
- **Subsystem**: `os-image-or-kernel-config`

### C. Internal Structure
- **Line Count**: 69 lines
- **Key Declarations / Directives / Functions**:
  - L17: `[Partition]`
  - L18: `Type=esp`
  - L19: `Label=LITRIC_ESP`
  - L20: `Format=vfat`
  - L21: `CopyFiles=/efi:/`
  - L22: `CopyFiles=/boot:/`
  - L32: `SizeMinBytes=512M`
  - L33: `SizeMaxBytes=512M`
  - L69: `MountPoint=/efi:nofail`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: None explicitly hardcoded.
- **Environment Variables / Flags**: `LITRIC_ESP`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0590 — `image/mkosi.repart/10-root.conf`

### A. Identity
- **Path**: `image/mkosi.repart/10-root.conf`
- **Type / Language**: `.conf` (INI/Config)
- **Size**: 666 bytes
- **Classification**: `configuration` | Role: `os-image-or-kernel-config`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: System partition — erofs, read-only, dm-verity protected. erofs over squashfs: random-read latency is materially better (erofs does fixed-size compressed blocks with direct addressing rather than squashfs's sequential block chains), which is what page-fault-driven binary loading actually does.  Over btrfs: no CoW metadata, no free-space tracking, no write path at all — nothing in this partition is ever dirtied, so every page
- **Subsystem**: `os-image-or-kernel-config`

### C. Internal Structure
- **Line Count**: 18 lines
- **Key Declarations / Directives / Functions**:
  - L9: `[Partition]`
  - L10: `Type=root-x86-64`
  - L11: `Label=LITRIC_SYS_A`
  - L12: `Format=erofs`
  - L13: `CopyFiles=/`
  - L14: `Minimize=best`
  - L15: `Verity=data`
  - L16: `VerityMatchKey=root`
  - L17: `SplitName=system`
  - L18: `ReadOnly=yes`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: None explicitly hardcoded.
- **Environment Variables / Flags**: `LITRIC_SYS_A`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0591 — `image/mkosi.repart/11-root-verity.conf`

### A. Identity
- **Path**: `image/mkosi.repart/11-root-verity.conf`
- **Type / Language**: `.conf` (INI/Config)
- **Size**: 170 bytes
- **Classification**: `configuration` | Role: `os-image-or-kernel-config`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: dm-verity Merkle tree for the partition above.
- **Subsystem**: `os-image-or-kernel-config`

### C. Internal Structure
- **Line Count**: 8 lines
- **Key Declarations / Directives / Functions**:
  - L2: `[Partition]`
  - L3: `Type=root-x86-64-verity`
  - L4: `Label=LITRIC_SYSVER_A`
  - L5: `Verity=hash`
  - L6: `VerityMatchKey=root`
  - L7: `Minimize=best`
  - L8: `SplitName=verity`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: None explicitly hardcoded.
- **Environment Variables / Flags**: `LITRIC_SYSVER_A`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0592 — `image/mkosi.repart/12-root-verity-sig.conf`

### A. Identity
- **Path**: `image/mkosi.repart/12-root-verity-sig.conf`
- **Type / Language**: `.conf` (INI/Config)
- **Size**: 386 bytes
- **Classification**: `configuration` | Role: `os-image-or-kernel-config`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Signed verity root hash.  The kernel refuses to mount a system partition whose root hash does not match the signature chained to the Secure Boot key, so a byte flipped in LITRIC_SYS_A fails the boot closed rather than continuing silently (Phase 03, task 2).
- **Subsystem**: `os-image-or-kernel-config`

### C. Internal Structure
- **Line Count**: 10 lines
- **Key Declarations / Directives / Functions**:
  - L5: `[Partition]`
  - L6: `Type=root-x86-64-verity-sig`
  - L7: `Label=LITRIC_SYSSIG_A`
  - L8: `Verity=signature`
  - L9: `VerityMatchKey=root`
  - L10: `SplitName=verity-sig`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: None explicitly hardcoded.
- **Environment Variables / Flags**: `LITRIC_SYSSIG_A`, `LITRIC_SYS_A`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0593 — `image/mkosi.repart/20-var.conf`

### A. Identity
- **Path**: `image/mkosi.repart/20-var.conf`
- **Type / Language**: `.conf` (INI/Config)
- **Size**: 963 bytes
- **Classification**: `configuration` | Role: `os-image-or-kernel-config`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Writable /var. The `nano` profile gets away with `systemd.volatile=state` (tmpfs /var) because it is the memory floor and holds no persistent state. Everything from `core` upward needs real storage: without it, every service declaring StateDirectory= fails at startup with "Failed to set up special execution directory in /var/lib: Read-only file system", which is how systemd-logind and
- **Subsystem**: `os-image-or-kernel-config`

### C. Internal Structure
- **Line Count**: 22 lines
- **Key Declarations / Directives / Functions**:
  - L17: `[Partition]`
  - L18: `Type=var`
  - L19: `Label=LITRIC_VAR`
  - L20: `Format=btrfs`
  - L21: `SizeMinBytes=64M`
  - L22: `MountPoint=/var:nofail`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/var/lib`
- **Environment Variables / Flags**: `LITRIC_VAR`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0594 — `image/units-pending/README.md`

### A. Identity
- **Path**: `image/units-pending/README.md`
- **Type / Language**: `.md` (Markdown)
- **Size**: 3,204 bytes
- **Classification**: `configuration` | Role: `os-image-or-kernel-config`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Project os-image-or-kernel-config file (Markdown).
- **Subsystem**: `os-image-or-kernel-config`

### C. Internal Structure
- **Line Count**: 41 lines
- **Major Sections / Headings**:
  - L1: `# Units waiting for a daemon that exists`
  - L24: `## The one that was worse than dead`
  - L32: `## What must exist before a unit returns here`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/usr/bin/litric-capd`, `/usr/bin/litric-defender`, `/usr/bin/litric-filesd`, `ci/check-memory-budget.py`, `ci/lint-unit-binaries.py`, `docs/PERF-budget.md`, `docs/adr/0016-system-daemon.md`, `docs/adr/0019-litric-files.md`, `docs/audit/2026-10-11-beta-1.0-stage-a-audit.md`, `image/mkosi.extra/usr/lib/systemd/system/`, `image/mkosi.postinst.chroot`, `src/main.rs`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0595 — `image/units-pending/litric-capd.service`

### A. Identity
- **Path**: `image/units-pending/litric-capd.service`
- **Type / Language**: `.service` (systemd Unit)
- **Size**: 1,649 bytes
- **Classification**: `configuration` | Role: `os-image-or-kernel-config`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: The broker is the authority for every capability decision on the system. If it is down, the correct behaviour is deny-by-default, which is what clients see when the socket has no listener. It is never "allow because the broker is gone". --- sandbox --------------------------------------------------------------- A privileged decision-maker gets the strictest profile that still lets it work. THREAT ACCEPTED: AF_UNIX only. The broker must never speak to a network, so a
- **Subsystem**: `os-image-or-kernel-config`

### C. Internal Structure
- **Line Count**: 55 lines
- **Key Declarations / Directives / Functions**:
  - L1: `[Unit]`
  - L2: `Description=LitricOS capability broker (litric.caps)`
  - L3: `Documentation=file:///usr/share/doc/litric/API-00-contracts.md`
  - L4: `Requires=litric-capd.socket`
  - L5: `After=litric-capd.socket dbus-broker.service`
  - L7: `[Service]`
  - L8: `Type=notify`
  - L9: `ExecStart=/usr/bin/litric-capd --systemd-socket`
  - L10: `Restart=always`
  - L11: `RestartSec=200ms`
  - L15: `WatchdogSec=30s`
  - L17: `Slice=system.slice`
  - L18: `MemoryAccounting=yes`
  - L19: `MemoryMax=24M`
  - L20: `TasksMax=32`
  - L24: `User=litric-capd`
  - L25: `Group=litric-capd`
  - L26: `NoNewPrivileges=yes`
  - L27: `ProtectSystem=strict`
  - L28: `ProtectHome=yes`
  - L29: `PrivateTmp=yes`
  - L30: `PrivateDevices=yes`
  - L31: `ProtectKernelTunables=yes`
  - L32: `ProtectKernelModules=yes`
  - L33: `ProtectKernelLogs=yes`
  - L34: `ProtectControlGroups=yes`
  - L35: `ProtectClock=yes`
  - L36: `ProtectHostname=yes`
  - L37: `ProtectProc=invisible`
  - L38: `ProcSubset=pid`
  - L39: `RestrictNamespaces=yes`
  - L40: `RestrictRealtime=yes`
  - L41: `RestrictSUIDSGID=yes`
  - L42: `LockPersonality=yes`
  - L43: `MemoryDenyWriteExecute=yes`
  - L44: `SystemCallArchitectures=native`
  - L45: `SystemCallFilter=@system-service`
  - L46: `SystemCallErrorNumber=EPERM`
  - L49: `RestrictAddressFamilies=AF_UNIX`
  - L50: `IPAddressDeny=any`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/usr/bin/litric-capd`, `/usr/share/doc/litric/API-00-contracts.md`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0596 — `image/units-pending/litric-capd.socket`

### A. Identity
- **Path**: `image/units-pending/litric-capd.socket`
- **Type / Language**: `.socket` (systemd Socket)
- **Size**: 608 bytes
- **Classification**: `configuration` | Role: `os-image-or-kernel-config`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: 0666 is correct and deliberate: every process on the system must be able to ASK. Authorisation happens inside the broker using SO_PEERCRED plus the caller's cgroup-derived application identity, never by filesystem permission on the socket. Relying on socket mode for authorisation is exactly the confused-deputy bug that RUNTIME section 37 exists to prevent.
- **Subsystem**: `os-image-or-kernel-config`

### C. Internal Structure
- **Line Count**: 17 lines
- **Key Declarations / Directives / Functions**:
  - L1: `[Unit]`
  - L2: `Description=LitricOS capability broker socket`
  - L3: `Documentation=file:///usr/share/doc/litric/API-00-contracts.md`
  - L5: `[Socket]`
  - L6: `ListenStream=/run/litric/capd.sock`
  - L7: `SocketMode=0666`
  - L13: `Accept=no`
  - L14: `RemoveOnStop=yes`
  - L16: `[Install]`
  - L17: `WantedBy=sockets.target`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/run/litric/capd.sock`, `/usr/share/doc/litric/API-00-contracts.md`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0597 — `image/units-pending/litric-defender.service`

### A. Identity
- **Path**: `image/units-pending/litric-defender.service`
- **Type / Language**: `.service` (systemd Unit)
- **Size**: 1,928 bytes
- **Classification**: `configuration` | Role: `os-image-or-kernel-config`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: A boot entry that sets litric.defender=off is the documented escape hatch for "the defender is blocking a system binary" (Phase 07 failure mode). --- sandbox --------------------------------------------------------------- fanotify permission events require CAP_SYS_ADMIN. There is no way around that: it is the capability that lets a process gate other processes' file opens. THREAT ACCEPTED: a compromise of litric-defender is effectively root.
- **Subsystem**: `os-image-or-kernel-config`

### C. Internal Structure
- **Line Count**: 54 lines
- **Key Declarations / Directives / Functions**:
  - L1: `[Unit]`
  - L2: `Description=LitricOS Defender (on-access scanning)`
  - L3: `Documentation=file:///usr/share/doc/litric/SEC-00-threatmodel.md`
  - L4: `After=local-fs.target litric-capd.service`
  - L7: `ConditionKernelCommandLine=!litric.defender=off`
  - L9: `[Service]`
  - L10: `Type=notify`
  - L11: `ExecStart=/usr/bin/litric-defender --rules /usr/lib/litric/yara --cache /var/cache/litric/defender`
  - L12: `Restart=always`
  - L13: `RestartSec=1s`
  - L14: `WatchdogSec=60s`
  - L16: `Slice=litric-defender.slice`
  - L29: `CapabilityBoundingSet=CAP_SYS_ADMIN CAP_DAC_READ_SEARCH`
  - L30: `AmbientCapabilities=CAP_SYS_ADMIN CAP_DAC_READ_SEARCH`
  - L31: `NoNewPrivileges=yes`
  - L32: `ProtectSystem=strict`
  - L33: `ProtectHome=read-only`
  - L34: `PrivateTmp=yes`
  - L35: `PrivateNetwork=yes`
  - L36: `RestrictAddressFamilies=AF_UNIX`
  - L37: `IPAddressDeny=any`
  - L38: `ProtectKernelModules=yes`
  - L39: `ProtectKernelLogs=yes`
  - L40: `ProtectClock=yes`
  - L41: `ProtectHostname=yes`
  - L42: `RestrictNamespaces=yes`
  - L43: `RestrictRealtime=yes`
  - L44: `RestrictSUIDSGID=yes`
  - L45: `LockPersonality=yes`
  - L46: `MemoryDenyWriteExecute=yes`
  - L47: `SystemCallArchitectures=native`
  - L48: `SystemCallFilter=@system-service @privileged`
  - L49: `SystemCallErrorNumber=EPERM`
  - L50: `CacheDirectory=litric/defender`
  - L51: `StateDirectory=litric/defender`
  - L53: `[Install]`
  - L54: `WantedBy=multi-user.target`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/usr/bin/litric-defender`, `/usr/lib/litric/yara`, `/usr/share/doc/litric/SEC-00-threatmodel.md`, `/var/cache/litric/defender`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0598 — `image/units-pending/litric-defender.slice`

### A. Identity
- **Path**: `image/units-pending/litric-defender.slice`
- **Type / Language**: `.slice` (systemd Slice)
- **Size**: 660 bytes
- **Classification**: `configuration` | Role: `os-image-or-kernel-config`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Budget line 10.  This is a HARD cap, not a target: if the scanner leaks or a YARA rule set explodes, the defender dies and restarts rather than pushing the desktop into swap.  A security tool that degrades the system it protects gets turned off by users, and then protects nothing. Below the shell in CPU weight.  A full-tree scan must never cost a frame.
- **Subsystem**: `os-image-or-kernel-config`

### C. Internal Structure
- **Line Count**: 24 lines
- **Key Declarations / Directives / Functions**:
  - L1: `[Unit]`
  - L2: `Description=LitricOS Defender resource slice`
  - L3: `Documentation=file:///usr/share/doc/litric/PERF-budget.md`
  - L4: `Before=slices.target`
  - L6: `[Slice]`
  - L11: `MemoryAccounting=yes`
  - L12: `MemoryHigh=56M`
  - L13: `MemoryMax=64M`
  - L14: `MemorySwapMax=0`
  - L17: `CPUAccounting=yes`
  - L18: `CPUWeight=20`
  - L19: `CPUQuota=60%`
  - L21: `IOAccounting=yes`
  - L22: `IOWeight=20`
  - L24: `TasksMax=64`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/usr/share/doc/litric/PERF-budget.md`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0599 — `image/units-pending/litric-files.socket`

### A. Identity
- **Path**: `image/units-pending/litric-files.socket`
- **Type / Language**: `.socket` (systemd Socket)
- **Size**: 463 bytes
- **Classification**: `configuration` | Role: `os-image-or-kernel-config`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Same shape as litric-browser.socket: 0600 root, so only the shell can ask the daemon to touch the user's files, and one daemon process per connection.
- **Subsystem**: `os-image-or-kernel-config`

### C. Internal Structure
- **Line Count**: 18 lines
- **Key Declarations / Directives / Functions**:
  - L1: `[Unit]`
  - L2: `Description=Litric Files daemon socket`
  - L3: `Documentation=file:///usr/share/doc/litric/adr/0019-litric-files.md`
  - L5: `[Socket]`
  - L8: `ListenStream=/run/litric/files.sock`
  - L9: `SocketUser=root`
  - L10: `SocketGroup=root`
  - L11: `SocketMode=0600`
  - L12: `DirectoryMode=0755`
  - L13: `Accept=yes`
  - L14: `MaxConnections=8`
  - L15: `RemoveOnStop=yes`
  - L17: `[Install]`
  - L18: `WantedBy=sockets.target`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/run/litric/files.sock`, `/usr/share/doc/litric/adr/0019-litric-files.md`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0600 — `image/units-pending/litric-files@.service`

### A. Identity
- **Path**: `image/units-pending/litric-files@.service`
- **Type / Language**: `.service` (systemd Unit)
- **Size**: 1,139 bytes
- **Classification**: `configuration` | Role: `os-image-or-kernel-config`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: --- sandbox --------------------------------------------------------------- It reads and writes the user's home and nothing else, needs no network, no devices, no namespaces and no generated code.
- **Subsystem**: `os-image-or-kernel-config`

### C. Internal Structure
- **Line Count**: 45 lines
- **Key Declarations / Directives / Functions**:
  - L1: `[Unit]`
  - L2: `Description=Litric Files daemon (connection %i)`
  - L3: `Documentation=file:///usr/share/doc/litric/adr/0019-litric-files.md`
  - L4: `CollectMode=inactive-or-failed`
  - L6: `[Service]`
  - L7: `Type=simple`
  - L8: `ExecStart=/usr/bin/litric-filesd --stdio --root /var/home/litric`
  - L9: `StandardInput=socket`
  - L10: `StandardOutput=socket`
  - L11: `StandardError=journal`
  - L12: `User=litric`
  - L13: `Group=litric`
  - L18: `NoNewPrivileges=yes`
  - L19: `CapabilityBoundingSet=`
  - L20: `AmbientCapabilities=`
  - L21: `ProtectSystem=strict`
  - L22: `ProtectHome=yes`
  - L23: `ReadWritePaths=/var/home/litric`
  - L24: `PrivateTmp=yes`
  - L25: `PrivateDevices=yes`
  - L26: `PrivateNetwork=yes`
  - L27: `IPAddressDeny=any`
  - L28: `RestrictAddressFamilies=AF_UNIX`
  - L29: `ProtectKernelTunables=yes`
  - L30: `ProtectKernelModules=yes`
  - L31: `ProtectKernelLogs=yes`
  - L32: `ProtectControlGroups=yes`
  - L33: `ProtectClock=yes`
  - L34: `ProtectHostname=yes`
  - L35: `RestrictNamespaces=yes`
  - L36: `RestrictRealtime=yes`
  - L37: `RestrictSUIDSGID=yes`
  - L38: `LockPersonality=yes`
  - L39: `MemoryDenyWriteExecute=yes`
  - L40: `SystemCallArchitectures=native`
  - L41: `SystemCallFilter=@system-service`
  - L42: `SystemCallErrorNumber=EPERM`
  - L44: `MemoryMax=256M`
  - L45: `TasksMax=16`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/usr/bin/litric-filesd`, `/usr/share/doc/litric/adr/0019-litric-files.md`, `/var/home/litric`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0601 — `iso/assemble-iso.sh`

### A. Identity
- **Path**: `iso/assemble-iso.sh`
- **Type / Language**: `.sh` (Bash/POSIX Shell)
- **Size**: 13,773 bytes
- **Classification**: `configuration` | Role: `os-image-or-kernel-config`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: ============================================================================= LitricOS — live ISO assembly Takes the split artifacts produced by mkosi and wraps them into a bootable UEFI ISO 9660 image with a GPT-appended EFI System Partition. INPUTS  (from build/out/, produced by `mkosi --split-artifacts`) litricos_<ver>.efi              signed Unified Kernel Image
- **Subsystem**: `os-image-or-kernel-config`

### C. Internal Structure
- **Line Count**: 317 lines
- **Key Declarations / Directives / Functions**:
  - L46: `log()  { printf '\033[1;32m[iso]\033[0m %s\n' "$*"; }`
  - L47: `warn() { printf '\033[1;33m[iso]\033[0m %s\n' "$*" >&2; }`
  - L48: `die()  { printf '\033[1;31m[iso]\033[0m %s\n' "$*" >&2; exit 1; }`
  - L53: `preflight() {`
  - L98: `restamp_uki() {`
  - L165: `stage_tree() {`
  - L223: `build_efiboot() {`
  - L258: `master_iso() {`
  - L307: `main() {`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/dev/null`, `/usr/bin/env`, `/usr/lib/systemd/boot/efi/systemd-bootx64.efi`, `docs/PERF-budget.md`, `image/mkosi.extra/usr/lib/os-release`, `iso/assemble-iso.sh`, `keys/dev`, `tests/budget/`, `tools/build.sh`, `tools/gen-dev-keys.sh.`
- **Environment Variables / Flags**: `LITRIC_BUILD_EPOCH`, `LITRIC_EFI`, `LITRIC_LIVE`, `LITRIC_PROFILE`, `LITRIC_ROOTHASH`, `LITRIC_VERSION`, `PROFILE`, `SOURCE_DATE_EPOCH`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0602 — `iso/hooks/litric-live-root.service`

### A. Identity
- **Path**: `iso/hooks/litric-live-root.service`
- **Type / Language**: `.service` (systemd Unit)
- **Size**: 733 bytes
- **Classification**: `configuration` | Role: `os-image-or-kernel-config`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Fail closed: if root assembly fails we land in the initrd emergency shell with the printed reason, rather than switch-rooting into something unverified.
- **Subsystem**: `os-image-or-kernel-config`

### C. Internal Structure
- **Line Count**: 22 lines
- **Key Declarations / Directives / Functions**:
  - L1: `[Unit]`
  - L2: `Description=LitricOS live root assembly`
  - L3: `Documentation=file:///usr/share/doc/litric/adr/0010-iso-delivery.md`
  - L4: `DefaultDependencies=no`
  - L5: `ConditionKernelCommandLine=litric.live=1`
  - L6: `After=systemd-udev-settle.service dev-mapper-litric\x2dsys.device`
  - L7: `Wants=systemd-udev-settle.service`
  - L8: `Before=initrd-root-fs.target sysroot.mount`
  - L9: `Conflicts=shutdown.target initrd-switch-root.target`
  - L11: `[Service]`
  - L12: `Type=oneshot`
  - L13: `RemainAfterExit=yes`
  - L14: `ExecStart=/usr/lib/litric/live-root`
  - L15: `StandardOutput=journal+console`
  - L16: `StandardError=journal+console`
  - L19: `FailureAction=none`
  - L21: `[Install]`
  - L22: `WantedBy=initrd-root-fs.target`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/usr/lib/litric/live-root`, `/usr/share/doc/litric/adr/0010-iso-delivery.md`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0603 — `iso/hooks/litric-live-root.sh`

### A. Identity
- **Path**: `iso/hooks/litric-live-root.sh`
- **Type / Language**: `.sh` (Bash/POSIX Shell)
- **Size**: 6,812 bytes
- **Classification**: `configuration` | Role: `os-image-or-kernel-config`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: ============================================================================= LitricOS — live root assembly (runs inside the initrd) Turns the ISO payload into a mounted, writable, integrity-verified /sysroot that systemd can switch-root into. ISO (iso9660, ro) └─ /litric/system.erofs   ──loop──┐
- **Subsystem**: `os-image-or-kernel-config`

### C. Internal Structure
- **Line Count**: 149 lines
- **Key Declarations / Directives / Functions**:
  - L36: `log()  { printf 'litric-live: %s\n' "$*" > /dev/kmsg 2>/dev/null || printf 'litric-live: %s\n' "$*"; }`
  - L37: `fail() {`
  - L46: `kcmd() {`
  - L68: `find_medium() {`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/dev/kmsg`, `/dev/mapper/`, `/dev/mapper/litric-sys`, `/dev/null`, `/proc/cmdline`, `/run/litric`, `/run/litric/live`, `/run/litric/live-session`, `/run/litric/lower`, `/run/litric/upper`, `/usr/bin/env`, `iso/assemble-iso.sh`
- **Environment Variables / Flags**: `LITRIC_LIVE`, `LITRIC_MEDIUM`, `LITRIC_ROOTHASH`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0604 — `iso/loader/entries/10-litric-live.conf`

### A. Identity
- **Path**: `iso/loader/entries/10-litric-live.conf`
- **Type / Language**: `.conf` (INI/Config)
- **Size**: 90 bytes
- **Classification**: `configuration` | Role: `os-image-or-kernel-config`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Project os-image-or-kernel-config file (INI/Config).
- **Subsystem**: `os-image-or-kernel-config`

### C. Internal Structure
- **Line Count**: 3 lines

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: None explicitly hardcoded.

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0605 — `iso/loader/entries/20-litric-live-safe.conf`

### A. Identity
- **Path**: `iso/loader/entries/20-litric-live-safe.conf`
- **Type / Language**: `.conf` (INI/Config)
- **Size**: 495 bytes
- **Classification**: `configuration` | Role: `os-image-or-kernel-config`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Phase 11 failure mode: "no display on boot (GPU driver) -> the image must always ship a nomodeset/basic-framebuffer boot entry." This is that entry. litric.shell=software forces the compositor onto llvmpipe rather than letting it fail to find a DRM device and exit.
- **Subsystem**: `os-image-or-kernel-config`

### C. Internal Structure
- **Line Count**: 8 lines
- **Key Declarations / Directives / Functions**:
  - L8: `options  nomodeset litric.shell=software litric.live=1 litric.volid=@VOLID@ roothash=@ROOTHASH@ syst`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: None explicitly hardcoded.

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0606 — `iso/loader/entries/30-litric-live-noscan.conf`

### A. Identity
- **Path**: `iso/loader/entries/30-litric-live-noscan.conf`
- **Type / Language**: `.conf` (INI/Config)
- **Size**: 351 bytes
- **Classification**: `configuration` | Role: `os-image-or-kernel-config`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Phase 07 failure mode: a false positive that blocks a system binary must have an escape hatch that does not require another machine to fix.
- **Subsystem**: `os-image-or-kernel-config`

### C. Internal Structure
- **Line Count**: 6 lines
- **Key Declarations / Directives / Functions**:
  - L6: `options  litric.defender=off litric.live=1 litric.volid=@VOLID@ roothash=@ROOTHASH@ systemd.verity=y`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: None explicitly hardcoded.

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0607 — `iso/loader/entries/40-litric-recovery.conf`

### A. Identity
- **Path**: `iso/loader/entries/40-litric-recovery.conf`
- **Type / Language**: `.conf` (INI/Config)
- **Size**: 464 bytes
- **Classification**: `configuration` | Role: `os-image-or-kernel-config`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Boots litric-recovery.target: shell, storage tools, network, log extraction. No compositor, no AI, no defender. Persistent journald so that whatever went wrong is still readable after the next reboot.
- **Subsystem**: `os-image-or-kernel-config`

### C. Internal Structure
- **Line Count**: 7 lines
- **Key Declarations / Directives / Functions**:
  - L7: `options  systemd.unit=litric-recovery.target systemd.journald.storage=persistent litric.live=1 litri`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: None explicitly hardcoded.

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0608 — `iso/loader/loader.conf`

### A. Identity
- **Path**: `iso/loader/loader.conf`
- **Type / Language**: `.conf` (INI/Config)
- **Size**: 381 bytes
- **Classification**: `configuration` | Role: `os-image-or-kernel-config`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: systemd-boot configuration for the LitricOS live ISO. editor=no: the boot-entry editor lets anyone with physical access append init=/bin/sh and get a root shell, bypassing every userspace control. Disk encryption is the answer to physical access, not an editable cmdline.
- **Subsystem**: `os-image-or-kernel-config`

### C. Internal Structure
- **Line Count**: 10 lines

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: None explicitly hardcoded.

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0609 — `kernel/config/litric-base.config`

### A. Identity
- **Path**: `kernel/config/litric-base.config`
- **Type / Language**: `.config` (Kconfig)
- **Size**: 12,181 bytes
- **Classification**: `configuration` | Role: `os-image-or-kernel-config`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: LITRIC OS — Kernel Configuration Fragment Based on: x86_64 defconfig + Litric-specific overlay Target: QEMU virt + modern PC (x86-64 first-class) Usage: apply this fragment on top of defconfig make defconfig scripts/kconfig/merge_config.sh -m .config kernel/config/litric-base.config
- **Subsystem**: `os-image-or-kernel-config`

### C. Internal Structure
- **Line Count**: 387 lines
- **Key Declarations / Directives / Functions**:
  - L15: `# SECTION 1: LITRIC CORE FEATURES (mandatory per MASTER §3)`
  - L19: `CONFIG_LRU_GEN=y`
  - L20: `CONFIG_LRU_GEN_ENABLED=y`
  - L23: `CONFIG_SCHED_CLASS_EXT=y`
  - L26: `CONFIG_NTSYNC=y`
  - L29: `CONFIG_FANOTIFY=y`
  - L30: `CONFIG_FANOTIFY_ACCESS_PERMISSIONS=y`
  - L33: `CONFIG_SECURITY_LANDLOCK=y`
  - L36: `CONFIG_BPF_LSM=y`
  - L39: `CONFIG_DM_VERITY=y`
  - L42: `CONFIG_EROFS_FS=y`
  - L45: `CONFIG_BTRFS_FS=y`
  - L48: `CONFIG_ZRAM=y`
  - L49: `CONFIG_ZRAM_MULTI_COMP=y`
  - L50: `CONFIG_CRYPTO_ZSTD=y`
  - L51: `CONFIG_CRYPTO_LZ4=y`
  - L54: `CONFIG_PREEMPT=y`
  - L55: `CONFIG_HZ_1000=y`
  - L56: `CONFIG_NO_HZ_IDLE=y`
  - L59: `CONFIG_EFI_STUB=y`
  - L62: `CONFIG_GDB_SCRIPTS=y`
  - L65: `# SECTION 2: BOOT / EFI`
  - L68: `CONFIG_EFI=y`
  - L70: `CONFIG_EFI_PARTITION=y`
  - L71: `CONFIG_DMI=y`
  - L72: `CONFIG_DMIID=y`
  - L75: `# SECTION 3: VIRTIO (QEMU virtualization)`
  - L78: `CONFIG_VIRTIO=y`
  - L79: `CONFIG_VIRTIO_PCI=y`
  - L80: `CONFIG_VIRTIO_MMIO=y`
  - L81: `CONFIG_VIRTIO_BLK=y`
  - L82: `CONFIG_VIRTIO_NET=y`
  - L83: `CONFIG_VIRTIO_CONSOLE=y`
  - L84: `CONFIG_VIRTIO_GPU=y`
  - L85: `CONFIG_VIRTIO_INPUT=y`
  - L86: `CONFIG_VIRTIO_FS=m`
  - L87: `CONFIG_9P_FS=m`
  - L88: `CONFIG_NET_9P=m`
  - L89: `CONFIG_NET_9P_VIRTIO=m`
  - L92: `# SECTION 4: STORAGE`
  - L95: `CONFIG_BLOCK=y`
  - L96: `CONFIG_BLK_DEV=y`
  - L97: `CONFIG_BLK_DEV_LOOP=y`
  - L98: `CONFIG_BLK_DEV_RAM=y`
  - L99: `CONFIG_BLK_DEV_RAM_SIZE=65536`

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `/dev/mem`, `ci/lint-kconfig.py`, `image/mkosi.conf.`, `kernel/config/litric-base.config`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---

## F0610 — `keys/.gitignore`

### A. Identity
- **Path**: `keys/.gitignore`
- **Type / Language**: `no_ext` (Plaintext/Config)
- **Size**: 215 bytes
- **Classification**: `secret-sensitive` | Role: `key-management`
- **Inspection Status**: `VERIFIED` (full file inspected)

### B. Purpose
- **Summary from File Header/Docstring**: Development keys are generated locally by tools/gen-dev-keys.sh and are never committed. A committed signing key -- even a development one -- is a key that will eventually be used by mistake.
- **Subsystem**: `key-management`

### C. Internal Structure
- **Line Count**: 6 lines

### D. Dependencies & Relationships
- **Referenced Files & System Paths**: `tools/gen-dev-keys.sh`

### E. Operational Behavior & Security
- **Execution/Loading Context**: Loaded or executed during build (`tools/build.sh`), CI gate verification (`ci/verify.sh`), initrd/systemd boot (`image/`), or developer session orchestration.
- **Security & Invariants**: Enforces LitricOS invariants (G1 host-untouched path guards, LF-only line endings, least-privilege systemd sandboxes, dm-verity/UKI signing).

### F. Change-Impact Analysis
- Modifying or removing this file impacts downstream gates (`ci/verify.sh`, `ci/lint-doc-refs.py`, `ci/lint-mkosi.py`, `ci/lint-kconfig.py`, `ci/lint-unit-binaries.py`, `ci/check-memory-budget.py`) or image assembly (`tools/build.sh`, `iso/assemble-iso.sh`).

---
