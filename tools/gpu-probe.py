#!/usr/bin/env python3
"""Probe a real Linux GPU and record what the browser and VA-API can see.

This is meant to run on a machine with a physical GPU and a
``/dev/dri/renderD*`` node. It writes evidence under
``artifacts/gpu-probe/<UTC timestamp>/``; it never invents a result when a
device, driver or browser engine is missing.

What is collected
-----------------
* ``/dev/dri`` nodes, their character-device metadata, sysfs card/render
  mappings, driver names, PCI IDs and the loaded DRM modules.
* ``vulkaninfo --summary`` and the installed Vulkan ICDs.
* ``vainfo`` against the first render node (and a plain fallback run).
* ``litric-browserd``'s own ``browserd: gpu=...`` policy line, plus the
  LBP/1 HELLO answer that says whether Chromium actually started.
* A headless ``chrome://gpu`` dump, driven with the same GPU switches
  browserd uses. Its DOM is saved raw and as text so the graphics feature
  report can be inspected without trusting a summary.

The distinction matters: browserd's line says which policy it selected;
``chrome://gpu`` is Chromium's own report. This script records both. A
``gpu=hardware`` line alone is not proof that a frame was drawn on the GPU.

Usage
-----
    python3 tools/gpu-probe.py
    python3 tools/gpu-probe.py --out artifacts/gpu-probe/my-gpu
    python3 tools/gpu-probe.py --browserd src/target/release/litric-browserd \\
        --chromium /usr/bin/chromium

Exit status
-----------
0  PASS: render node, Chromium engine, hardware policy, Vulkan or VA-API,
   and Chromium's own report confirms both GPU and VA-API video decode.
1  PARTIAL: a render node and hardware policy exist, but a report/API probe
   was unavailable or did not confirm acceleration.
2  BLOCKED: no render node, no browserd/Chromium, or browserd selected the
   software path. The JSON says which condition failed.

No command in this script runs with ``sudo``, writes to a device, or weakens
Chromium's sandbox. ``--allow-no-sandbox`` exists only for a disposable test
host where Chromium's sandbox cannot start; it is off by default.
"""

from __future__ import annotations

import argparse
import json
import os
import platform
import re
import shlex
import shutil
import signal
import socket
import stat
import subprocess
import sys
import tempfile
import time
from datetime import datetime, timezone
from html.parser import HTMLParser
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DEFAULT_OUT = ROOT / "artifacts" / "gpu-probe"
CHROMIUM_CANDIDATES = (
    "/usr/bin/chromium",
    "/usr/bin/chromium-browser",
    "/usr/bin/google-chrome",
    "/usr/bin/google-chrome-stable",
    "/opt/chrome/chrome",
)
BROWSERD_CANDIDATES = (
    ROOT / "src" / "target" / "release" / "litric-browserd",
    ROOT / "src" / "target" / "debug" / "litric-browserd",
)
DRM_MODULE_HINTS = (
    "drm",
    "i915",
    "xe",
    "amdgpu",
    "radeon",
    "nouveau",
    "nvidia",
    "virtio_gpu",
    "v3d",
    "panfrost",
    "msm",
    "etnaviv",
    "bochs",
    "qxl",
    "vmwgfx",
)
LIMITATIONS = [
    "browserd: gpu=... reports the policy selected when Chromium starts; it is not per-frame GPU utilisation.",
    "chrome://gpu is a capability report, not a frame-time or throughput measurement.",
    "vainfo lists VA-API profiles; this probe does not decode an encoded video, so actual decode acceleration is not proven.",
    "The probe requires read access to /dev/dri. It does not write to a device or change its permissions.",
    "A physical display/KMS scanout is separate from browser compositing and is not exercised here.",
]


def utc_now() -> str:
    return datetime.now(timezone.utc).isoformat(timespec="seconds")


def read_text(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8", errors="replace").strip()
    except OSError:
        return ""


def write_text(path: Path, text: str) -> None:
    path.write_text(text, encoding="utf-8")


def command_meta(record: dict) -> dict:
    return {
        "argv": record["argv"],
        "returncode": record["returncode"],
        "timed_out": record["timed_out"],
        "error": record["error"],
    }


def run_command(
    argv: list[str],
    output_path: Path,
    timeout: float,
    env: dict[str, str] | None = None,
) -> dict:
    """Run one command without a shell and save its complete output."""
    argv = [str(item) for item in argv]
    record = {
        "argv": argv,
        "returncode": None,
        "timed_out": False,
        "error": None,
        "stdout": "",
        "stderr": "",
    }
    try:
        proc = subprocess.run(
            argv,
            capture_output=True,
            text=True,
            errors="replace",
            timeout=timeout,
            env=env,
            check=False,
        )
        record["returncode"] = proc.returncode
        record["stdout"] = proc.stdout or ""
        record["stderr"] = proc.stderr or ""
    except FileNotFoundError:
        record["error"] = "not found"
    except subprocess.TimeoutExpired as exc:
        record["timed_out"] = True
        record["error"] = f"timed out after {timeout:g}s"
        record["stdout"] = _coerce_text(exc.stdout)
        record["stderr"] = _coerce_text(exc.stderr)
    except OSError as exc:
        record["error"] = str(exc)

    header = f"$ {shlex.join(argv)}\n"
    if record["timed_out"]:
        header += f"[timed out after {timeout:g}s]\n"
    elif record["error"]:
        header += f"[error: {record['error']}]\n"
    else:
        header += f"[exit {record['returncode']}]\n"
    body = header
    if record["stdout"]:
        body += "[stdout]\n" + record["stdout"].rstrip() + "\n"
    if record["stderr"]:
        body += "[stderr]\n" + record["stderr"].rstrip() + "\n"
    write_text(output_path, body)
    return record


def _coerce_text(value: object) -> str:
    if isinstance(value, bytes):
        return value.decode("utf-8", "replace")
    return value if isinstance(value, str) else ""


def usable_executable(path: Path) -> bool:
    return path.is_file() and os.access(path, os.X_OK)


def find_browserd(explicit: str | None) -> Path | None:
    if explicit:
        path = Path(explicit).expanduser()
        return path if usable_executable(path) else None
    for candidate in BROWSERD_CANDIDATES:
        if usable_executable(candidate):
            return candidate
    found = shutil.which("litric-browserd")
    return Path(found) if found else None


def find_chromium(explicit: str | None) -> Path | None:
    if explicit:
        path = Path(explicit).expanduser()
        if not path.is_absolute():
            found = shutil.which(explicit)
            path = Path(found) if found else path
        path = path.resolve()
        return path if usable_executable(path) else None
    from_env = os.environ.get("BROWSERD_CHROMIUM")
    if from_env:
        path = Path(from_env)
        if usable_executable(path):
            return path
    for candidate in CHROMIUM_CANDIDATES:
        path = Path(candidate)
        if usable_executable(path):
            return path
    for name in ("chromium", "chromium-browser", "google-chrome", "google-chrome-stable"):
        found = shutil.which(name)
        if found:
            return Path(found)
    return None


def make_output_dir(explicit: str | None) -> Path:
    if explicit:
        out = Path(explicit).expanduser()
    else:
        stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
        out = DEFAULT_OUT / stamp
        suffix = 1
        while out.exists():
            suffix += 1
            out = DEFAULT_OUT / f"{stamp}-{suffix}"
    out.mkdir(parents=True, exist_ok=True)
    return out


def git_head() -> str:
    try:
        proc = subprocess.run(
            ["git", "-C", str(ROOT), "rev-parse", "HEAD"],
            capture_output=True,
            text=True,
            timeout=5,
            check=False,
        )
        return proc.stdout.strip() or "unknown"
    except (OSError, subprocess.TimeoutExpired):
        return "unknown"


def host_info() -> dict:
    uname = platform.uname()
    return {
        "uname": {
            "system": uname.system,
            "node": uname.node,
            "release": uname.release,
            "version": uname.version,
            "machine": uname.machine,
        },
        "python": platform.python_version(),
        "uid": os.getuid() if hasattr(os, "getuid") else None,
        "cwd": str(Path.cwd()),
        "os_release": read_text(Path("/etc/os-release")),
        "cmdline": read_text(Path("/proc/cmdline")),
        "git_head": git_head(),
    }


def describe_node(path: Path) -> dict:
    entry: dict[str, object] = {"name": path.name, "path": str(path)}
    try:
        info = path.stat()
    except OSError as exc:
        entry["error"] = str(exc)
        return entry
    entry["mode"] = f"{stat.S_IMODE(info.st_mode):04o}"
    entry["character_device"] = stat.S_ISCHR(info.st_mode)
    entry["rdev"] = info.st_rdev
    return entry


def sysfs_info(name: str) -> dict:
    base = Path("/sys/class/drm") / name
    result: dict[str, object] = {}
    if not base.exists():
        return result
    result["sysfs"] = str(base)
    driver = base / "device" / "driver"
    if driver.exists():
        try:
            result["driver"] = driver.resolve().name
        except OSError:
            result["driver"] = read_text(driver)
    for key in ("vendor", "device"):
        value = read_text(base / "device" / key)
        if value:
            result[key] = value
    uevent = read_text(base / "device" / "uevent")
    if uevent:
        result["uevent"] = uevent
    status = read_text(base / "status")
    if status:
        result["status"] = status
    return result


def collect_dri(out: Path, timeout: float) -> dict:
    dev = Path("/dev/dri")
    sys_class = Path("/sys/class/drm")
    dev_lines: list[str] = []
    entries: list[dict] = []
    render_nodes: list[str] = []
    cards: list[str] = []
    by_path: list[str] = []

    if dev.is_dir():
        for child in sorted(dev.iterdir()):
            entry = describe_node(child)
            entries.append(entry)
            dev_lines.append(
                f"{child.name}: mode={entry.get('mode')} "
                f"char={entry.get('character_device')} "
                f"rdev={entry.get('rdev')}"
            )
            if re.fullmatch(r"renderD\d+", child.name) and entry.get("character_device"):
                render_nodes.append(str(child))
            if re.fullmatch(r"card\d+", child.name) and entry.get("character_device"):
                cards.append(str(child))
        by_path_dir = dev / "by-path"
        if by_path_dir.is_dir():
            for link in sorted(by_path_dir.iterdir()):
                try:
                    by_path.append(f"{link.name} -> {link.resolve()}")
                except OSError:
                    by_path.append(link.name)
    else:
        dev_lines.append("/dev/dri: absent")

    for entry in entries:
        entry.update(sysfs_info(str(entry["name"])))

    sys_lines: list[str] = []
    if sys_class.is_dir():
        for child in sorted(sys_class.iterdir()):
            try:
                target = child.resolve()
            except OSError:
                target = Path("?")
            sys_lines.append(f"{child.name} -> {target}")
            if re.fullmatch(r"(card|renderD)\d+", child.name):
                for rel in (
                    "device/driver",
                    "device/vendor",
                    "device/device",
                    "device/uevent",
                    "status",
                ):
                    value = read_text(child / rel)
                    if value:
                        sys_lines.append(f"  {rel}: {value.splitlines()[0]}")
    else:
        sys_lines.append("/sys/class/drm: absent")

    if by_path:
        sys_lines.extend(["", "by-path:", *by_path])

    modules: list[str] = []
    for line in read_text(Path("/proc/modules")).splitlines():
        name = line.split(maxsplit=1)[0] if line.strip() else ""
        if any(hint in name for hint in DRM_MODULE_HINTS):
            modules.append(line)

    write_text(out / "dev-dri.txt", "\n".join(dev_lines) + "\n")
    write_text(out / "drm-sysfs.txt", "\n".join(sys_lines) + "\n")

    lspci = shutil.which("lspci")
    lspci_record = None
    if lspci:
        lspci_record = run_command(
            [lspci, "-nnk"], out / "lspci.txt", timeout=min(timeout, 15)
        )
    else:
        write_text(out / "lspci.txt", "SKIP: lspci is not installed\n")

    nvidia_version = Path("/proc/driver/nvidia/version")
    nvidia_smi = shutil.which("nvidia-smi")
    nvidia_record = None
    if nvidia_version.exists():
        write_text(out / "nvidia-version.txt", read_text(nvidia_version) + "\n")
    if nvidia_smi:
        nvidia_record = run_command(
            [
                nvidia_smi,
                "--query-gpu=name,driver_version,memory.total",
                "--format=csv,noheader",
            ],
            out / "nvidia-smi.txt",
            timeout=min(timeout, 15),
        )

    return {
        "present": dev.is_dir(),
        "sysfs_present": sys_class.is_dir(),
        "render_nodes": render_nodes,
        "cards": cards,
        "entries": entries,
        "by_path": by_path,
        "modules": modules,
        "lspci": command_meta(lspci_record) if lspci_record else None,
        "nvidia_smi": command_meta(nvidia_record) if nvidia_record else None,
    }


def collect_vulkan(out: Path, timeout: float) -> dict:
    tool = shutil.which("vulkaninfo")
    icd_dirs = [Path("/usr/share/vulkan/icd.d"), Path("/etc/vulkan/icd.d")]
    icd_lines: list[str] = []
    for directory in icd_dirs:
        icd_lines.append(f"{directory}:")
        if not directory.is_dir():
            icd_lines.append("  absent")
            continue
        for entry in sorted(directory.iterdir()):
            icd_lines.append(f"  {entry.name}")
    write_text(out / "vulkan-icds.txt", "\n".join(icd_lines) + "\n")

    if not tool:
        write_text(out / "vulkaninfo.txt", "SKIP: vulkaninfo is not installed\n")
        return {"available": False, "tool": None, "command": None, "summary": ""}

    record = run_command(
        [tool, "--summary"], out / "vulkaninfo.txt", timeout=min(timeout, 30)
    )
    text = record["stdout"] + "\n" + record["stderr"]
    available = record["returncode"] == 0 and bool(
        re.search(r"\b(GPU\d|deviceName|Discrete GPU|Integrated GPU|VkPhysicalDevice)\b", text, re.I)
    )
    excerpt = "\n".join(
        line
        for line in text.splitlines()
        if re.search(r"(deviceName|driverName|driverInfo|apiVersion|GPU\d|GPU id)", line, re.I)
    )[:8000]
    return {
        "available": available,
        "tool": tool,
        "command": command_meta(record),
        "summary": excerpt,
    }


def collect_vaapi(out: Path, render_nodes: list[str], timeout: float) -> dict:
    tool = shutil.which("vainfo")
    if not tool:
        write_text(out / "vainfo.txt", "SKIP: vainfo is not installed\n")
        return {"available": False, "tool": None, "runs": [], "profiles": []}

    runs: list[dict] = []
    attempts: list[tuple[str, list[str]]] = []
    if render_nodes:
        attempts.append(
            (
                "drm",
                [tool, "--display", "drm", "--device", render_nodes[0]],
            )
        )
    attempts.append(("default", [tool]))

    chosen_text = ""
    profiles: list[str] = []
    for label, argv in attempts:
        record = run_command(
            argv, out / f"vainfo-{label}.txt", timeout=min(timeout, 20)
        )
        text = record["stdout"] + "\n" + record["stderr"]
        found = sorted(set(re.findall(r"VAProfile[A-Za-z0-9_]+", text)))
        runs.append(
            {
                "label": label,
                "command": command_meta(record),
                "profiles": found,
            }
        )
        if record["returncode"] == 0 and found and not profiles:
            profiles = found
            chosen_text = text

    write_text(out / "vainfo.txt", chosen_text or "SKIP: no VA-API profile was reported\n")
    return {
        "available": bool(profiles),
        "tool": tool,
        "runs": runs,
        "profiles": profiles,
    }


def stop_process_group(proc: subprocess.Popen) -> None:
    if proc.poll() is not None:
        return
    try:
        if os.name == "posix":
            os.killpg(os.getpgid(proc.pid), signal.SIGTERM)
        else:
            proc.terminate()
    except OSError:
        return
    try:
        proc.wait(timeout=5)
        return
    except subprocess.TimeoutExpired:
        pass
    try:
        if os.name == "posix":
            os.killpg(os.getpgid(proc.pid), signal.SIGKILL)
        else:
            proc.kill()
    except OSError:
        pass
    try:
        proc.wait(timeout=5)
    except subprocess.TimeoutExpired:
        pass


def connect_broker(sock_path: Path, proc: subprocess.Popen, timeout: float) -> socket.socket | None:
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if proc.poll() is not None:
            return None
        if sock_path.exists():
            stream = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
            stream.settimeout(0.5)
            try:
                stream.connect(str(sock_path))
                return stream
            except OSError:
                stream.close()
        time.sleep(0.05)
    return None


def parse_browserd_log(text: str) -> dict:
    gpu_lines = [
        line for line in text.splitlines() if "browserd: gpu=" in line
    ]
    engine_lines = [
        line for line in text.splitlines() if "browserd: engine = chromium" in line
    ]
    policy = ""
    if gpu_lines:
        policy = gpu_lines[-1].split("browserd: gpu=", 1)[1].strip()
    match = re.search(
        r"gpu=hardware\s+backend=(\S+)\s+render=(\S+)", policy
    )
    return {
        "gpu_policy": f"gpu={policy}" if policy else None,
        "gpu_backend": match.group(1) if match else None,
        "gpu_render": match.group(2) if match else None,
        "engine_log": engine_lines[-1].strip() if engine_lines else None,
    }


def run_browserd(
    browserd: Path,
    chromium: Path | None,
    out: Path,
    timeout: float,
) -> dict:
    run_dir = out / "browserd"
    run_dir.mkdir(parents=True, exist_ok=True)
    log_path = run_dir / "browserd.log"
    hello_path = run_dir / "hello.txt"
    result: dict = {
        "started": False,
        "hello": None,
        "engine": "unknown",
        "gpu_policy": None,
        "gpu_backend": None,
        "gpu_render": None,
        "engine_log": None,
        "error": None,
        "log": str(log_path),
    }

    with tempfile.TemporaryDirectory(prefix="litric-gpu-probe-") as temporary:
        temp = Path(temporary)
        sock_path = temp / "browser.sock"
        env = dict(os.environ)
        env["BROWSERD_SOCK"] = str(sock_path)
        if chromium:
            env["BROWSERD_CHROMIUM"] = str(chromium)
        argv = [
            str(browserd),
            "--profile",
            str(temp / "profile"),
            "--downloads",
            str(temp / "downloads"),
        ]
        with open(log_path, "wb") as log:
            try:
                proc = subprocess.Popen(
                    argv,
                    env=env,
                    stdout=log,
                    stderr=subprocess.STDOUT,
                    start_new_session=True,
                )
            except OSError as exc:
                result["error"] = str(exc)
                write_text(hello_path, f"could not start browserd: {exc}\n")
                return result
            result["started"] = True
            stream = None
            try:
                stream = connect_broker(sock_path, proc, min(timeout, 30))
                if stream is None:
                    result["error"] = (
                        "browserd did not create its socket before the timeout"
                    )
                else:
                    stream.settimeout(min(timeout, 10))
                    stream.sendall(b"HELLO 1\n")
                    reader = stream.makefile("rb")
                    line = reader.readline()
                    hello = line.decode("utf-8", "replace").strip()
                    result["hello"] = hello
                    if hello.startswith("HELLO 1 engine=chromium"):
                        result["engine"] = "chromium"
                    elif hello.startswith("ERR engine-unavailable"):
                        result["engine"] = "missing"
                    else:
                        result["engine"] = "unknown"
                    try:
                        stream.sendall(b"QUIT\n")
                    except OSError:
                        pass
            except OSError as exc:
                result["error"] = f"broker connection: {exc}"
            finally:
                if stream is not None:
                    stream.close()
                if proc.poll() is None:
                    try:
                        proc.wait(timeout=5)
                    except subprocess.TimeoutExpired:
                        stop_process_group(proc)
                else:
                    stop_process_group(proc)
        log_text = read_text(log_path)
        result.update(parse_browserd_log(log_text))
        if result["hello"] is None:
            result["hello"] = "no HELLO reply"
        write_text(
            hello_path,
            f"hello: {result['hello']}\n"
            f"engine: {result['engine']}\n"
            f"gpu_policy: {result['gpu_policy']}\n",
        )
    return result


class TextExtractor(HTMLParser):
    def __init__(self) -> None:
        super().__init__(convert_charrefs=True)
        self.parts: list[str] = []

    def handle_data(self, data: str) -> None:
        if data.strip():
            self.parts.append(data.strip())


def html_to_text(markup: str) -> str:
    parser = TextExtractor()
    try:
        parser.feed(markup)
    except Exception:
        return re.sub(r"\s+", " ", re.sub(r"<[^>]+>", " ", markup))
    return "\n".join(parser.parts)


def feature_status(text: str, label: str) -> str | None:
    flat = re.sub(r"\s+", " ", text).lower()
    match = re.search(
        rf"{re.escape(label)}\s*:?\s*(hardware accelerated|software only|disabled|unavailable)",
        flat,
    )
    return match.group(1) if match else None


def run_chromium_gpu(
    chromium: Path | None,
    policy: str | None,
    backend: str | None,
    out: Path,
    timeout: float,
    allow_no_sandbox: bool,
) -> dict:
    result: dict = {
        "ran": False,
        "returncode": None,
        "timed_out": False,
        "report_available": False,
        "hardware_accelerated": False,
        "video_decode_hardware_accelerated": False,
        "features": {},
        "error": None,
        "html": None,
        "text": None,
    }
    if chromium is None:
        write_text(out / "chromium-gpu.txt", "SKIP: no Chromium binary found\n")
        result["error"] = "no Chromium binary"
        return result

    with tempfile.TemporaryDirectory(prefix="litric-gpu-chrome-") as temporary:
        temp = Path(temporary)
        argv = [
            str(chromium),
            "--headless=new",
            "--disable-dev-shm-usage",
            "--no-first-run",
            "--no-default-browser-check",
            f"--user-data-dir={temp / 'profile'}",
            "--dump-dom",
            "chrome://gpu",
        ]
        if policy and policy.startswith("gpu=hardware"):
            argv.extend(
                [
                    "--enable-gpu",
                    "--enable-gpu-rasterization",
                    "--enable-zero-copy",
                    "--enable-features=VaapiVideoDecoder,VaapiVideoEncoder",
                ]
            )
            if backend:
                argv.append(f"--use-angle={backend}")
        else:
            argv.append("--disable-gpu")
        if allow_no_sandbox:
            argv.append("--no-sandbox")
        record = run_command(
            argv, out / "chromium-gpu-command.txt", timeout=min(timeout, 60)
        )
        result["ran"] = True
        result["returncode"] = record["returncode"]
        result["timed_out"] = record["timed_out"]
        if record["error"]:
            result["error"] = record["error"]
        html = record["stdout"]
        if html.strip():
            result["report_available"] = True
            html_path = out / "chromium-gpu.html"
            write_text(html_path, html)
            text = html_to_text(html)
            text_path = out / "chromium-gpu.txt"
            write_text(text_path, text + "\n")
            result["html"] = html_path.name
            result["text"] = text_path.name
            labels = (
                "canvas",
                "rasterization",
                "webgl",
                "webgl2",
                "video decode",
                "video encode",
            )
            features = {label: feature_status(text, label) for label in labels}
            result["features"] = features
            result["hardware_accelerated"] = any(
                value == "hardware accelerated" for value in features.values()
            )
            result["video_decode_hardware_accelerated"] = (
                features.get("video decode") == "hardware accelerated"
            )
        else:
            write_text(out / "chromium-gpu.txt", "SKIP: Chromium produced no DOM\n")
            result["error"] = result["error"] or "Chromium produced no DOM"
    return result


def make_summary(
    out: Path,
    checks: dict[str, bool],
    status: str,
    dri: dict,
    vulkan: dict,
    vaapi: dict,
    browser: dict,
) -> dict:
    return {
        "schema": 1,
        "generated_at": utc_now(),
        "status": status,
        "exit_code": {"PASS": 0, "PARTIAL": 1, "BLOCKED": 2}[status],
        "host": host_info(),
        "dri": dri,
        "vulkan": vulkan,
        "vaapi": vaapi,
        "browser": browser,
        "checks": checks,
        "limitations": LIMITATIONS,
        "artifacts": str(out),
    }


def human_summary(result: dict) -> str:
    browser = result["browser"]
    vulkan = result["vulkan"]
    vaapi = result["vaapi"]
    chromium = browser.get("chromium_gpu", {})
    lines = [
        f"GPU probe: {result['status']}",
        f"host: {result['host']['uname']['machine']} kernel {result['host']['uname']['release']}",
        f"render nodes: {', '.join(result['dri']['render_nodes']) or 'none'}",
        f"cards: {', '.join(result['dri']['cards']) or 'none'}",
        f"browserd: {browser.get('browserd_path') or 'not found'}",
        f"browser engine: {browser.get('engine', 'unknown')}",
        f"browser GPU policy: {browser.get('gpu_policy') or 'not reported'}",
        f"vulkaninfo: {'available' if vulkan.get('available') else 'unavailable'}",
        f"vainfo: {'available' if vaapi.get('available') else 'unavailable'} "
        f"({len(vaapi.get('profiles', []))} profiles)",
        f"chrome://gpu: "
        f"{'hardware accelerated' if chromium.get('hardware_accelerated') else 'not confirmed'}"
        f"{' (no report)' if not chromium.get('report_available') else ''}",
        f"VA-API video decode report: "
        f"{'hardware accelerated' if chromium.get('video_decode_hardware_accelerated') else 'not confirmed'}",
        f"artifacts: {result['artifacts']}",
    ]
    return "\n".join(lines) + "\n"


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Probe a real Linux GPU, VA-API and Litric browser GPU policy."
    )
    parser.add_argument("--out", help="output directory (default: artifacts/gpu-probe/<UTC timestamp>)")
    parser.add_argument("--browserd", help="path to litric-browserd")
    parser.add_argument("--chromium", help="Chromium binary for browserd and chrome://gpu")
    parser.add_argument("--timeout", type=float, default=30.0, help="per-stage timeout in seconds")
    parser.add_argument(
        "--allow-no-sandbox",
        action="store_true",
        help="pass --no-sandbox to the Chrome probe; only for a disposable test host",
    )
    args = parser.parse_args()
    if args.timeout <= 0:
        parser.error("--timeout must be positive")

    out = make_output_dir(args.out)
    browserd = find_browserd(args.browserd)
    chromium = find_chromium(args.chromium)
    write_text(out / "host.txt", json.dumps(host_info(), indent=2, sort_keys=True) + "\n")

    dri = collect_dri(out, args.timeout)
    vulkan = collect_vulkan(out, args.timeout)
    vaapi = collect_vaapi(out, dri["render_nodes"], args.timeout)

    if browserd is None:
        browser = {
            "browserd_path": None,
            "browserd_present": False,
            "engine": "missing",
            "hello": None,
            "gpu_policy": None,
            "gpu_backend": None,
            "gpu_render": None,
            "engine_log": None,
            "error": "litric-browserd was not found",
        }
    else:
        browser = {
            "browserd_path": str(browserd),
            "browserd_present": True,
        }
        browser.update(run_browserd(browserd, chromium, out, args.timeout))

    chromium_gpu = run_chromium_gpu(
        chromium, browser.get("gpu_policy"), browser.get("gpu_backend"),
        out, args.timeout, args.allow_no_sandbox,
    )
    browser["chromium_path"] = str(chromium) if chromium else None
    browser["chromium_gpu"] = chromium_gpu

    checks = {
        "render_node": bool(dri["render_nodes"]),
        "dri_card": bool(dri["cards"]),
        "browserd_present": bool(browser.get("browserd_present")),
        "browser_engine_chromium": browser.get("engine") == "chromium",
        "browser_gpu_policy_hardware": bool(
            browser.get("gpu_policy") and browser["gpu_policy"].startswith("gpu=hardware")
        ),
        "vulkan": bool(vulkan.get("available")),
        "vaapi": bool(vaapi.get("available")),
        "chromium_gpu_report": bool(chromium_gpu.get("report_available")),
        "chromium_gpu_hardware": bool(chromium_gpu.get("hardware_accelerated")),
        "vaapi_browser_decode": bool(
            chromium_gpu.get("video_decode_hardware_accelerated")
        ),
    }

    if not checks["render_node"]:
        status = "BLOCKED"
    elif not checks["browserd_present"]:
        status = "BLOCKED"
    elif not checks["browser_engine_chromium"]:
        status = "BLOCKED"
    elif not checks["browser_gpu_policy_hardware"]:
        status = "BLOCKED"
    elif not (checks["vulkan"] or checks["vaapi"]):
        status = "PARTIAL"
    elif not checks["chromium_gpu_hardware"]:
        status = "PARTIAL"
    elif not checks["vaapi"] or not checks["vaapi_browser_decode"]:
        status = "PARTIAL"
    else:
        status = "PASS"

    result = make_summary(out, checks, status, dri, vulkan, vaapi, browser)
    text = human_summary(result)
    write_text(out / "summary.txt", text)
    write_text(out / "result.json", json.dumps(result, indent=2, sort_keys=True) + "\n")
    print(text, end="")
    return result["exit_code"]


if __name__ == "__main__":
    sys.exit(main())
