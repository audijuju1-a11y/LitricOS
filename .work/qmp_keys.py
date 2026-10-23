import json
import socket
import sys

sock = "/root/litric-latest/build/gui-qmp.sock"
keys = sys.argv[1:] or ["2", "ret"]
s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
s.connect(sock)
s.settimeout(10)
f = s.makefile("rwb")
f.readline()
f.write(b'{"execute":"qmp_capabilities"}\n')
f.flush()
f.readline()
for k in keys:
    cmd = {"execute": "send-key", "arguments": {"keys": [{"type": "qcode", "data": k}]}}
    f.write((json.dumps(cmd) + "\n").encode())
    f.flush()
    print(k, f.readline()[:120])
