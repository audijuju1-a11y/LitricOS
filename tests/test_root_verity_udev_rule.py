"""Regression guard for the initrd dm-verity readiness rule.

The meaningful proof is the UEFI QEMU boot test.  This small test prevents a
future packaging cleanup from deleting or broadening the narrow udev rule that
allows systemd to notice its already verified root device.
"""

from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
RULE = (
    ROOT
    / "image"
    / "mkosi.initrd.extra"
    / "usr"
    / "lib"
    / "udev"
    / "rules.d"
    / "99-zlitric-root-verity-ready.rules"
)
INITRD_CONFIG = ROOT / "image" / "mkosi.initrd.conf"


def test_root_verity_ready_rule_is_narrow_and_runs_after_systemd_rule() -> None:
    config = INITRD_CONFIG.read_text(encoding="utf-8")
    assert "[Content]" in config
    assert "ExtraTrees=mkosi.initrd.extra" in config

    text = RULE.read_text(encoding="utf-8")

    # udev applies lexical order.  The transition has to happen after the
    # stock 99-systemd rule records the first empty dm container as unready.
    assert RULE.name > "99-systemd.rules"
    assert 'ACTION=="change"' in text
    assert 'SUBSYSTEM=="block"' in text
    assert 'KERNEL=="dm-[0-9]*"' in text
    assert 'ENV{DM_NAME}=="root"' in text
    assert 'ENV{DM_UUID}=="CRYPT-VERITY-*"' in text
    assert 'ENV{SYSTEMD_READY}="1"' in text

    # A generic dm-* ready rule could make an unrelated mapping look like the
    # root device.  The only assignment in this file must carry both guards.
    assignments = [line for line in text.splitlines() if "SYSTEMD_READY" in line and not line.startswith("#")]
    assert assignments == [
        'ACTION=="change", SUBSYSTEM=="block", KERNEL=="dm-[0-9]*", ENV{DM_NAME}=="root", ENV{DM_UUID}=="CRYPT-VERITY-*", ENV{SYSTEMD_READY}="1"'
    ]
