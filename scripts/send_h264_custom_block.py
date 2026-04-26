#!/usr/bin/env python3
"""Publish an Annex-B H264 stream as RoboMaster CustomByteBlock messages.

Protocol checked against:
  RoboMaster 2026 机甲大师高校系列赛通信协议 V1.3.0（20260327）

Relevant protocol facts:
  - MQTT server: 192.168.12.1:3333
  - Custom client IP: 192.168.12.2
  - MQTT topic name: CustomByteBlock
  - Protobuf: message CustomByteBlock { optional bytes data = 1; }
  - CustomByteBlock max frequency: 50Hz
  - CustomByteBlock data max size: 2.4 kbit = 300 bytes
  - Highest allowed QoS: 1
"""

from __future__ import annotations

import argparse
import threading
import time
from pathlib import Path
from typing import Iterable

PROTOCOL_VERSION = "RoboMaster 2026 custom client protocol V1.3.0 (2026-03-27)"
OFFICIAL_HOST = "192.168.12.1"
OFFICIAL_PORT = 3333
TEST_HOST = "127.0.0.1"
TOPIC_CUSTOM_BYTE_BLOCK = "CustomByteBlock"
MAX_CUSTOM_BYTE_BLOCK_BYTES = 300
MAX_CUSTOM_BYTE_BLOCK_HZ = 50.0


def encode_varint(value: int) -> bytes:
    if value < 0:
        raise ValueError("varint value must be non-negative")
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
    if len(data) > MAX_CUSTOM_BYTE_BLOCK_BYTES:
        raise ValueError(f"CustomByteBlock data field exceeds {MAX_CUSTOM_BYTE_BLOCK_BYTES} bytes")
    # field 1, wire type 2: (1 << 3) | 2 = 0x0a
    return b"\x0a" + encode_varint(len(data)) + data


def chunks(data: bytes, size: int) -> Iterable[bytes]:
    for index in range(0, len(data), size):
        yield data[index : index + size]


def stream_chunks(data: bytes, size: int, should_loop: bool) -> Iterable[bytes]:
    while True:
        yield from chunks(data, size)
        if not should_loop:
            break


def find_start_code(data: bytes, start: int = 0) -> tuple[int, int] | None:
    index = start
    while index + 3 <= len(data):
        if data[index : index + 3] == b"\x00\x00\x01":
            return index, 3
        if index + 4 <= len(data) and data[index : index + 4] == b"\x00\x00\x00\x01":
            return index, 4
        index += 1
    return None


def split_annexb_nals(data: bytes) -> list[bytes]:
    first = find_start_code(data)
    if first is None:
        raise SystemExit("Input does not look like Annex-B H264: missing start code")
    start, _ = first
    if start > 0:
        data = data[start:]

    nals: list[bytes] = []
    current = 0
    while True:
        next_start = find_start_code(data, current + 4)
        if next_start is None:
            nals.append(data[current:])
            break
        nals.append(data[current : next_start[0]])
        current = next_start[0]
    return [nal for nal in nals if nal]


def nal_type(nal: bytes) -> int | None:
    if nal.startswith(b"\x00\x00\x00\x01"):
        offset = 4
    elif nal.startswith(b"\x00\x00\x01"):
        offset = 3
    else:
        return None
    if offset >= len(nal):
        return None
    return nal[offset] & 0x1F


def annexb_access_units(data: bytes) -> list[bytes]:
    access_units: list[bytes] = []
    current: list[bytes] = []
    has_vcl = False
    cached_sps: bytes | None = None
    cached_pps: bytes | None = None

    for nal in split_annexb_nals(data):
        kind = nal_type(nal)
        if kind == 7:
            cached_sps = nal
        elif kind == 8:
            cached_pps = nal

        starts_new_picture = kind in (1, 5) and has_vcl
        if starts_new_picture and current:
            access_units.append(b"".join(current))
            current = []
            has_vcl = False
            if cached_sps:
                current.append(cached_sps)
            if cached_pps:
                current.append(cached_pps)

        current.append(nal)
        if kind in (1, 5):
            has_vcl = True

    if current and has_vcl:
        access_units.append(b"".join(current))
    return access_units


def encode_packetized_frame_blocks(data: bytes, data_budget: int) -> list[bytes]:
    if data_budget <= 8:
        raise SystemExit("packetized-frame mode requires --chunk-size > 8")
    payload_budget = data_budget - 8
    blocks: list[bytes] = []
    for frame_id, frame in enumerate(annexb_access_units(data)):
        if frame_id > 0xFFFF:
            break
        for fragment_index, fragment in enumerate(chunks(frame, payload_budget)):
            if fragment_index > 0xFFFF:
                raise SystemExit("frame has too many fragments for uint16 fragment_index")
            header = (
                frame_id.to_bytes(2, "big")
                + fragment_index.to_bytes(2, "big")
                + len(frame).to_bytes(4, "big")
            )
            blocks.append(header + fragment)
    return blocks


def data_blocks(data: bytes, size: int, mode: str) -> list[bytes]:
    if mode == "raw-annexb":
        return list(chunks(data, size))
    if mode == "packetized-frame":
        return encode_packetized_frame_blocks(data, size)
    raise ValueError(f"unknown stream mode: {mode}")


def stream_blocks(blocks: list[bytes], should_loop: bool) -> Iterable[bytes]:
    while True:
        yield from blocks
        if not should_loop:
            break


def load_mqtt_client():
    try:
        import paho.mqtt.client as mqtt
    except ImportError as exc:
        raise SystemExit("Missing dependency for real publish: python -m pip install paho-mqtt") from exc
    return mqtt


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Send H264 Annex-B bytes over MQTT topic CustomByteBlock for local/client testing.",
    )
    parser.add_argument("--input", required=True, type=Path, help="Annex-B .h264 input file")
    parser.add_argument("--host", default=TEST_HOST, help=f"MQTT broker host, default local test {TEST_HOST}")
    parser.add_argument("--port", type=int, default=OFFICIAL_PORT, help=f"MQTT broker port, default {OFFICIAL_PORT}")
    parser.add_argument("--official", action="store_true", help=f"use official server {OFFICIAL_HOST}:{OFFICIAL_PORT}")
    parser.add_argument("--topic", default=TOPIC_CUSTOM_BYTE_BLOCK, help="MQTT topic, protocol default CustomByteBlock")
    parser.add_argument("--client-id", default="custombyteblock-h264-test")
    parser.add_argument("--chunk-size", type=int, default=MAX_CUSTOM_BYTE_BLOCK_BYTES)
    parser.add_argument("--rate-hz", type=float, default=MAX_CUSTOM_BYTE_BLOCK_HZ)
    parser.add_argument(
        "--stream-mode",
        choices=["raw-annexb", "packetized-frame"],
        default="raw-annexb",
        help="must match the client CustomByteBlock parser mode",
    )
    parser.add_argument("--qos", type=int, choices=[0, 1], default=0, help="QoS 0 for stream testing, QoS 1 max by protocol")
    parser.add_argument("--connect-timeout", type=float, default=5.0, help="seconds to wait for MQTT CONNACK")
    parser.add_argument("--no-wait-publish", action="store_true", help="do not wait for each MQTT publish to leave the client")
    parser.add_argument("--loop", action="store_true")
    parser.add_argument("--dry-run", action="store_true", help="encode and print send plan without connecting MQTT")
    parser.add_argument("--limit", type=int, default=0, help="maximum chunks to send/inspect; 0 means all")
    return parser


def mqtt_reason_success(reason: object) -> bool:
    is_failure = getattr(reason, "is_failure", None)
    if isinstance(is_failure, bool):
        return not is_failure
    try:
        return int(reason) == 0
    except (TypeError, ValueError):
        return str(reason).lower() in {"0", "success"}


def main() -> None:
    parser = build_parser()
    args = parser.parse_args()

    if args.official:
        args.host = OFFICIAL_HOST
        args.port = OFFICIAL_PORT

    if args.topic != TOPIC_CUSTOM_BYTE_BLOCK:
        raise SystemExit(f"Topic mismatch: protocol topic is {TOPIC_CUSTOM_BYTE_BLOCK!r}, got {args.topic!r}")
    if args.chunk_size <= 0 or args.chunk_size > MAX_CUSTOM_BYTE_BLOCK_BYTES:
        raise SystemExit(f"--chunk-size must be 1..{MAX_CUSTOM_BYTE_BLOCK_BYTES}")
    if args.rate_hz <= 0 or args.rate_hz > MAX_CUSTOM_BYTE_BLOCK_HZ:
        raise SystemExit(f"--rate-hz must be >0 and <= {MAX_CUSTOM_BYTE_BLOCK_HZ}")

    payload = args.input.read_bytes()
    if not payload:
        raise SystemExit(f"Input file is empty: {args.input}")

    blocks = data_blocks(payload, args.chunk_size, args.stream_mode)
    if not blocks:
        raise SystemExit("No H264 data blocks generated")
    total_chunks = len(blocks)
    planned_chunks = min(total_chunks, args.limit) if args.limit > 0 else total_chunks
    first_block = encode_custom_byte_block(blocks[0])

    print(PROTOCOL_VERSION)
    print(f"topic={args.topic} host={args.host}:{args.port} qos={args.qos}")
    print(f"input={args.input} bytes={len(payload)} stream_mode={args.stream_mode} chunk_size={args.chunk_size} chunks={total_chunks}")
    print(f"first_protobuf_block_bytes={len(first_block)} first_data_bytes={len(blocks[0])}")

    if args.dry_run:
        print(f"dry-run: inspected {planned_chunks} chunk(s), no MQTT publish")
        return

    mqtt = load_mqtt_client()
    try:
        client = mqtt.Client(mqtt.CallbackAPIVersion.VERSION2, client_id=args.client_id)
    except AttributeError:
        client = mqtt.Client(client_id=args.client_id)

    connected = threading.Event()
    connect_error: list[str] = []
    disconnected: list[str] = []

    def on_connect(client, userdata, flags, reason_code, properties=None):
        if mqtt_reason_success(reason_code):
            connected.set()
            print(f"mqtt_connected reason={reason_code}")
        else:
            connect_error.append(str(reason_code))
            connected.set()

    def on_disconnect(client, userdata, *callback_args):
        if callback_args:
            disconnected.append(str(callback_args[-2] if len(callback_args) >= 2 else callback_args[-1]))

    client.on_connect = on_connect
    client.on_disconnect = on_disconnect

    connect_rc = client.connect(args.host, args.port, keepalive=10)
    if connect_rc != mqtt.MQTT_ERR_SUCCESS:
        raise SystemExit(f"MQTT TCP connect failed rc={connect_rc}")
    client.loop_start()
    if not connected.wait(args.connect_timeout):
        client.loop_stop()
        client.disconnect()
        raise SystemExit(f"MQTT CONNACK timeout after {args.connect_timeout:.1f}s: {args.host}:{args.port}")
    if connect_error:
        client.loop_stop()
        client.disconnect()
        raise SystemExit(f"MQTT CONNACK failed: {connect_error[-1]}")

    interval = 1.0 / args.rate_hz
    sent = 0
    try:
        for data_block in stream_blocks(blocks, args.loop):
            block = encode_custom_byte_block(data_block)
            result = client.publish(args.topic, block, qos=args.qos)
            if result.rc != mqtt.MQTT_ERR_SUCCESS:
                raise SystemExit(f"MQTT publish failed rc={result.rc} after sent={sent}")
            if not args.no_wait_publish:
                result.wait_for_publish()
            sent += 1
            if sent % 50 == 0:
                print(f"sent={sent}/{total_chunks if not args.loop else 'loop'}")
            if args.limit > 0 and sent >= args.limit:
                break
            time.sleep(interval)
    finally:
        client.loop_stop()
        client.disconnect()
        print(f"done sent={sent}")


if __name__ == "__main__":
    main()
