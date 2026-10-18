"""Run an MCP stdio server so that clients always use the handshake (initialize) protocol.

Why: Claude Code 2.1.274 sends a `server/discover` probe and an `initialize`
request back to back on the same stdio connection. FastMCP 4 servers (such as
OpenBB's) lock the connection to the 2026-07-28 protocol after answering the
probe, then reject the `initialize` with error -32022. This relay answers the
probe itself with "method not found", so the server only ever sees the
handshake. Everything else passes through unchanged.

Usage: python3 mcp-legacy-proxy.py <server command> [args...]
Remove this relay once Claude Code waits for the probe reply before initializing.
"""
import json
import subprocess
import sys
import threading

server = subprocess.Popen(sys.argv[1:], stdin=subprocess.PIPE, stdout=subprocess.PIPE)
client_out = sys.stdout.buffer
write_lock = threading.Lock()


def to_client(line):
    with write_lock:
        client_out.write(line)
        client_out.flush()


def relay_server_output():
    for line in iter(server.stdout.readline, b""):
        to_client(line)


threading.Thread(target=relay_server_output, daemon=True).start()

for line in iter(sys.stdin.buffer.readline, b""):
    try:
        message = json.loads(line)
    except ValueError:
        message = None
    if isinstance(message, dict) and message.get("method") == "server/discover":
        if "id" in message:
            reply = {"jsonrpc": "2.0", "id": message["id"],
                     "error": {"code": -32601, "message": "Method not found"}}
            to_client((json.dumps(reply) + "\n").encode())
        continue
    server.stdin.write(line)
    server.stdin.flush()

server.stdin.close()
server.wait()
