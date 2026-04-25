#!/usr/bin/env python3
"""Publish an Annex-B .h264 stream as CustomByteBlock protobuf messages.

The protobuf shape is:
  message CustomByteBlock { bytes data = 1; }
"""

from __future__ import annotations

import argparse
import time
from pathlib import Path

try:
    import paho.mqtt.client as mqtt
except ImportError as exc:
    raise SystemExit("Missing dependency: pip install paho-mqtt") from exc


def encode_varint(value: int) -> bytes:
    out = bytearray()
    while True:
        byte = value & 0x7F
        value >>= 7
        if value:
            out.append(byte | 0x80)
        else:
            out.append(byte)
            return bytes(out)


def encode_custom_byte_block(data: bytes) -> bytes:
    return b"\x0a" + encode_varint(len(data)) + data


def chunks(data: bytes, size: int):
    for index in range(0, len(data), size):
        yield data[index : index + size]


def stream_chunks(data: bytes, size: int, should_loop: bool):
    while True:
        yield from chunks(data, size)
        if not should_loop:
            break


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--port", type=int, default=1883)
    parser.add_argument("--topic", default="CustomByteBlock")
    parser.add_argument("--input", required=True, type=Path)
    parser.add_argument("--chunk-size", type=int, default=300)
    parser.add_argument("--rate-hz", type=float, default=50.0)
    parser.add_argument("--loop", action="store_true")
    args = parser.parse_args()

    payload = args.input.read_bytes()
    if not payload:
        raise SystemExit(f"Input file is empty: {args.input}")
    if args.chunk_size <= 0:
        raise SystemExit("--chunk-size must be positive")
    if args.rate_hz <= 0:
        raise SystemExit("--rate-hz must be positive")

    try:
        client = mqtt.Client(mqtt.CallbackAPIVersion.VERSION2)
    except AttributeError:
        client = mqtt.Client()
    client.connect(args.host, args.port, keepalive=10)
    client.loop_start()

    interval = 1.0 / args.rate_hz
    try:
        for chunk in stream_chunks(payload, args.chunk_size, args.loop):
            client.publish(args.topic, encode_custom_byte_block(chunk), qos=0)
            time.sleep(interval)
    finally:
        client.loop_stop()
        client.disconnect()


if __name__ == "__main__":
    main()
