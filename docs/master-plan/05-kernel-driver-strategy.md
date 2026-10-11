# 05 — Kernel and driver strategy

## Decision

Litric ships a supported Linux LTS kernel with a reviewable Litric config and
minimal downstream patches. The goal is not the smallest theoretical config; it
is a bootable, secure, updateable kernel whose enabled features are evidenced on
the actual image and HCL.

## Configuration policy

- `kernel/config/litric-base.config` is an input, not proof. CI must retain the
  final `.config`, kernel release, enabled LSM order, module list, config diff,
  and boot evidence as release artifacts.
- Each enabled symbol declares: feature owner, dependent user-facing capability,
  memory/attack-surface cost, test, and removal consequence.
- Kernel features needed for sandboxing and resource management include cgroup
  v2, PSI, seccomp, Landlock, namespaces, IOMMU/VFIO where supported, fs-verity,
  dm-verity, Btrfs, eBPF policy only where audited, and MGLRU where measured.
- Maintain a production config and a debug/fuzz config. Sanitizers and KUnit are
  not shipped in production by default.

## Driver and firmware policy

- Prefer upstream in-tree drivers and vendor firmware packages with provenance.
- No out-of-tree driver is a supported default without an explicit security,
  maintenance, rollback, and HCL ADR.
- Track BIOS/UEFI and device firmware version in hardware evidence. Firmware
  update failure must leave an operator-visible recovery path.
- Maintain device status as `SUPPORTED`, `PARTIAL`, `EXPERIMENTAL`, `BLOCKED`,
  or `UNTESTED`; never infer support from PCI IDs alone.

## Hardware enablement order

1. UEFI/NVMe/USB/input/virtio for CI and installer.
2. Intel and AMD integrated graphics, internal panel, HDMI/DP, audio and Wi-Fi.
3. Bluetooth, camera, fingerprint, docks, card readers and external GPUs.
4. Discrete GPU, HDR/VRR, USB4/Thunderbolt, NPU and passthrough features only
   after explicit test matrices.

## Test gates

Build and boot kernel on each HCL system; run selected kselftests for cgroups,
Landlock, seccomp, filesystem, networking and power; run KUnit/debug kernels in
CI; test module signing/load refusal; record dmesg warnings; and require a
reproducer plus regression test for every kernel-facing defect.

