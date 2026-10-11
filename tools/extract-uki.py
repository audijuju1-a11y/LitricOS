#!/usr/bin/env python3
"""Extract sections from a UKI (PE/COFF binary) and rebuild with trimmed initrd."""
import struct
import sys
import subprocess
import os

UKI_PATH = sys.argv[1]
INITRD_PATH = sys.argv[2]
OUTPUT_PATH = sys.argv[3]
WORK_DIR = sys.argv[4]

os.makedirs(WORK_DIR, exist_ok=True)

# Read the UKI PE binary
with open(UKI_PATH, 'rb') as f:
    data = f.read()

# Find PE signature
pe_off = data.find(b'PE\x00\x00')
if pe_off < 0:
    print("ERROR: Not a PE binary")
    sys.exit(1)

print(f"PE signature at offset {pe_off}")

# COFF header
num_sections = struct.unpack_from('<H', data, pe_off + 6)[0]
opt_hdr_size = struct.unpack_from('<H', data, pe_off + 20)[0]
print(f"Sections: {num_sections}")

# Parse section headers
sec_off = pe_off + 24 + opt_hdr_size
sections = {}
for i in range(num_sections):
    hdr = data[sec_off + i * 40 : sec_off + (i + 1) * 40]
    name = hdr[0:8].rstrip(b'\x00').decode('ascii', errors='replace')
    vsize = struct.unpack_from('<I', hdr, 8)[0]
    vaddr = struct.unpack_from('<I', hdr, 12)[0]
    raw_size = struct.unpack_from('<I', hdr, 16)[0]
    raw_ptr = struct.unpack_from('<I', hdr, 20)[0]
    sections[name] = {
        'vaddr': vaddr, 'vsize': vsize,
        'raw_ptr': raw_ptr, 'raw_size': raw_size
    }
    print(f"  {name:10s} raw_ptr={raw_ptr:#010x} raw_size={raw_size:#010x} vsize={vsize:#010x}")

# Extract .linux (kernel)
if '.linux' not in sections:
    print("ERROR: .linux section not found")
    sys.exit(1)

s = sections['.linux']
kernel = data[s['raw_ptr']:s['raw_ptr'] + s['raw_size']]
kernel_path = os.path.join(WORK_DIR, 'vmlinuz')
with open(kernel_path, 'wb') as f:
    f.write(kernel)
print(f"Kernel extracted: {len(kernel)} bytes")

# Extract .cmdline
cmdline = b''
if '.cmdline' in sections:
    s = sections['.cmdline']
    cmdline = data[s['raw_ptr']:s['raw_ptr'] + s['raw_size']].rstrip(b'\x00')
    print(f"Cmdline: {cmdline.decode('utf-8', errors='replace')[:100]}")

# Extract .osrel
osrel_path = os.path.join(WORK_DIR, 'os-release')
if '.osrel' in sections:
    s = sections['.osrel']
    osrel = data[s['raw_ptr']:s['raw_ptr'] + s['raw_size']].rstrip(b'\x00')
    with open(osrel_path, 'wb') as f:
        f.write(osrel)
    print(f"OS release extracted: {len(osrel)} bytes")
else:
    # Fallback: write a minimal os-release
    with open(osrel_path, 'w') as f:
        f.write("NAME=\"Arch Linux\"\nVERSION=\"rolling\"\nID=arch\nPRETTY_NAME=\"Arch Linux\"\n")
    print("Minimal os-release created")

# Rebuild UKI with trimmed initrd
print("Rebuilding UKI...")
cmd = [
    'ukify', 'build',
    f'--linux={kernel_path}',
    f'--initrd={INITRD_PATH}',
    f'--os-release={osrel_path}',
    f'--output={OUTPUT_PATH}'
]
if cmdline:
    cmd.insert(3, f'--cmdline={cmdline.decode()}')

# Add signing keys if available
key_path = '/root/litricos/keys/dev/secureboot.key'
cert_path = '/root/litricos/keys/dev/secureboot.crt'
pcr_key_path = '/root/litricos/keys/dev/pcr.key'
pcr_cert_path = '/root/litricos/keys/dev/pcr.crt'
if os.path.exists(key_path):
    cmd.extend([f'--secureboot-private-key={key_path}', f'--secureboot-certificate={cert_path}', '--sign-kernel'])
if os.path.exists(pcr_key_path):
    cmd.extend([
        f'--pcrpkey={pcr_key_path}',
        f'--pcr-private-key={pcr_key_path}',
        f'--pcr-certificate={pcr_cert_path}',
        '--sign-profile=DEFAULT'
    ])

print(f"Running: {' '.join(cmd[:5])}...")
result = subprocess.run(cmd, capture_output=True, text=True)
if result.returncode != 0:
    print(f"ukify FAILED: {result.stderr}")
    sys.exit(1)

new_size = os.path.getsize(OUTPUT_PATH)
print(f"New UKI: {new_size} bytes ({new_size // 1024}KB)")
print(f"Old UKI: {len(data)} bytes ({len(data) // 1024}KB)")
print(f"Saved: {(len(data) - new_size) // 1024}KB")
