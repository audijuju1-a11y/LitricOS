import json
import socket
import sys

sock = "/root/litric-latest/build/gui-qmp.sock"
x = int(sys.argv[1])
y = int(sys.argv[2])
axis_max = 32767  # usb-tablet absolute range
s = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
s.connect(sock)
s.settimeout(10)
f = s.makefile("rwb")
f.readline()
f.write(b'{"execute":"qmp_capabilities"}\n')
f.flush()
f.readline()


def send(events):
    cmd = {"execute": "input-send-event", "arguments": {"events": events}}
    f.write((json.dumps(cmd) + "\n").encode())
    f.flush()
    return f.readline()[:160]


def absx(v):
    return {"type": "abs", "data": {"axis": "x", "value": v}}


def absy(v):
    return {"type": "abs", "data": {"axis": "y", "value": v}}


ax = int(x * axis_max / 1920)
ay = int(y * axis_max / 1080)
print(send([absx(ax), absy(ay)]))
print(send([{"type": "btn", "data": {"down": True, "button": "left"}}]))
print(send([{"type": "btn", "data": {"down": False, "button": "left"}}]))
