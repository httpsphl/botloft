#!/usr/bin/env python3
"""Calls Botloft as the owner, for the server tests (spec 11): one
connection, the requests in order, each result printed as a JSON line. A
string `$N.path` in a later request's params is the value at that path in
the result of the Nth request (from 1).

    python3 rpc.py <port> <token-file> <method> <params-json> [<method> <params-json>]...

Python's standard library only, so a fresh server image runs it."""

import base64
import json
import os
import socket
import struct
import sys
import time


def connect(port):
    sock = socket.create_connection(("127.0.0.1", port), timeout=30)
    key = base64.b64encode(os.urandom(16)).decode()
    sock.sendall(
        (
            "GET /rpc HTTP/1.1\r\nHost: 127.0.0.1\r\nUpgrade: websocket\r\n"
            "Connection: Upgrade\r\nSec-WebSocket-Version: 13\r\n"
            f"Sec-WebSocket-Key: {key}\r\n\r\n"
        ).encode()
    )
    head = b""
    while b"\r\n\r\n" not in head:
        head += sock.recv(1)
    if b" 101 " not in head.split(b"\r\n")[0]:
        sys.exit(f"no WebSocket: {head.splitlines()[0]!r}")
    return sock


def send(sock, text):
    data = text.encode()
    mask = os.urandom(4)
    size = len(data)
    if size < 126:
        header = struct.pack("!BB", 0x81, 0x80 | size)
    elif size < 65536:
        header = struct.pack("!BBH", 0x81, 0x80 | 126, size)
    else:
        header = struct.pack("!BBQ", 0x81, 0x80 | 127, size)
    sock.sendall(header + mask + bytes(b ^ mask[i % 4] for i, b in enumerate(data)))


def exactly(sock, count):
    data = b""
    while len(data) < count:
        part = sock.recv(count - len(data))
        if not part:
            sys.exit("Botloft closed the connection")
        data += part
    return data


def receive(sock):
    first, second = exactly(sock, 2)
    size = second & 0x7F
    if size == 126:
        size = struct.unpack("!H", exactly(sock, 2))[0]
    elif size == 127:
        size = struct.unpack("!Q", exactly(sock, 8))[0]
    return first & 0x0F, exactly(sock, size)


def call(sock, ident, method, params):
    send(sock, json.dumps({"jsonrpc": "2.0", "id": ident, "method": method, "params": params}))
    while True:
        opcode, data = receive(sock)
        if opcode != 1:
            continue
        answer = json.loads(data)
        if answer.get("id") != ident:
            continue
        if "error" in answer:
            sys.exit(f"{method}: {answer['error'].get('message')}")
        return answer.get("result")


def resolve(value, results):
    if isinstance(value, str) and value.startswith("$"):
        index, _, path = value[1:].partition(".")
        found = results[int(index) - 1]
        for part in filter(None, path.split(".")):
            found = found[part]
        return found
    if isinstance(value, dict):
        return {key: resolve(item, results) for key, item in value.items()}
    return value


def main():
    port, token_file, rest = int(sys.argv[1]), sys.argv[2], sys.argv[3:]
    with open(token_file, encoding="utf-8") as file:
        token = file.read().strip()
    sock = connect(port)
    # The protocol this build speaks (PROTOCOL_VERSION).
    hello = {"token": token, "client": {"name": "test", "version": "0"}, "protocol": 2}
    send(sock, json.dumps({"jsonrpc": "2.0", "id": 0, "method": "session.hello", "params": hello}))
    while True:
        opcode, data = receive(sock)
        if opcode == 1 and json.loads(data).get("id") == 0:
            break
    results = []
    for index in range(0, len(rest), 2):
        params = resolve(json.loads(rest[index + 1]), results)
        results.append(call(sock, index // 2 + 1, rest[index], params))
        print(json.dumps(results[-1]), flush=True)
    # Some requests last only while the connection does (a browser watched).
    time.sleep(float(os.environ.get("RPC_HOLD", "0")))


main()
