import json
import socket
import sys

sock = "/root/litric-latest/build/gui-qmp.sock"
out = sys.argv[1] if len(sys.argv) > 1 else "/root/litric-latest/build/latest.ppm"
s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
s.connect(sock)
s.settimeout(10)
f = s.makefile("rwb")
print(f.readline()[:120])
f.write(b'{"execute":"qmp_capabilities"}\n')
f.flush()
print(f.readline()[:120])
cmd = {"execute": "screendump", "arguments": {"filename": out}}
f.write((json.dumps(cmd) + "\n").encode())
f.flush()
print(f.readline()[:200])
f.write(b'{"execute":"query-status"}\n')
f.flush()
print(f.readline()[:200])
