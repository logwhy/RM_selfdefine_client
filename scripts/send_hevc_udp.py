#!/usr/bin/env python3
"""Send an Annex-B HEVC/H265 stream using the RoboMaster UDP 3334 format.

UDP payload format:
  frame_id:          2 bytes, big-endian
  fragment_index:    2 bytes, big-endian
  frame_total_bytes: 4 bytes, big-endian
  payload:           HEVC bytes

By default the script groups NAL units into rough HEVC access units and prefixes
known VPS/SPS/PPS before VCL frames. This is more reliable for local decoder
validation than sending one NAL unit per decoder packet.
"""

from __future__ import annotations

import argparse
import socket
import struct
import time
from pathlib import Path

MAX_UDP_PAYLOAD = 1200
HEADER_SIZE = 8
HEVC_PARAM_SET_TYPES = {32, 33, 34}
HEVC_AUD_TYPE = 35


def find_start_codes(data: bytes) -> list[int]:
    positions: list[int] = []
    index = 0
    while index + 3 <= len(data):
        if data.startswith(b"\x00\x00\x00\x01", index):
            positions.append(index)
            index += 4
            continue
        if data.startswith(b"\x00\x00\x01", index):
            positions.append(index)
            index += 3
            continue
        index += 1
    return positions


def split_annexb_units(data: bytes) -> list[bytes]:
    starts = find_start_codes(data)
    if not starts:
        raise ValueError("input does not look like Annex-B HEVC: no 00 00 01 start code found")

    units: list[bytes] = []
    for idx, start in enumerate(starts):
        end = starts[idx + 1] if idx + 1 < len(starts) else len(data)
        unit = data[start:end]
        if len(unit) > 4:
            units.append(unit)
    return units


def start_code_len(unit: bytes) -> int:
    if unit.startswith(b"\x00\x00\x00\x01"):
        return 4
    if unit.startswith(b"\x00\x00\x01"):
        return 3
    return 0


def hevc_nal_type(unit: bytes) -> int | None:
    offset = start_code_len(unit)
    if offset == 0 or len(unit) < offset + 2:
        return None
    return (unit[offset] >> 1) & 0x3F


def is_vcl_nal(unit: bytes) -> bool:
    nal_type = hevc_nal_type(unit)
    return nal_type is not None and 0 <= nal_type <= 31


def first_slice_segment_flag(unit: bytes) -> bool:
    offset = start_code_len(unit)
    if len(unit) <= offset + 2:
        return False
    return (unit[offset + 2] & 0x80) != 0


def build_access_units(units: list[bytes], prefix_parameter_sets: bool) -> list[bytes]:
    access_units: list[bytes] = []
    current: list[bytes] = []
    parameter_sets: dict[int, bytes] = {}
    pending_non_vcl: list[bytes] = []
    current_has_vcl = False

    for unit in units:
        nal_type = hevc_nal_type(unit)
        if nal_type is None:
            continue

        if nal_type in HEVC_PARAM_SET_TYPES:
            parameter_sets[nal_type] = unit
            pending_non_vcl.append(unit)
            continue

        if nal_type == HEVC_AUD_TYPE:
            if current:
                access_units.append(b"".join(current))
                current = []
                current_has_vcl = False
            pending_non_vcl.append(unit)
            continue

        if is_vcl_nal(unit):
            if current_has_vcl and first_slice_segment_flag(unit):
                access_units.append(b"".join(current))
                current = []
                current_has_vcl = False

            if not current:
                if prefix_parameter_sets:
                    current.extend(parameter_sets[nal] for nal in sorted(parameter_sets))
                current.extend(pending_non_vcl)
                pending_non_vcl = []

            current.append(unit)
            current_has_vcl = True
            continue

        if current:
            current.append(unit)
        else:
            pending_non_vcl.append(unit)

    if current:
        access_units.append(b"".join(current))

    return [unit for unit in access_units if unit]


def send_unit(
    sock: socket.socket,
    target: tuple[str, int],
    frame_id: int,
    unit: bytes,
    fragment_payload_size: int,
    fragment_gap_seconds: float,
) -> int:
    sent = 0
    total = len(unit)
    for fragment_index, offset in enumerate(range(0, total, fragment_payload_size)):
        chunk = unit[offset : offset + fragment_payload_size]
        header = struct.pack(">HHI", frame_id & 0xFFFF, fragment_index & 0xFFFF, total)
        sock.sendto(header + chunk, target)
        sent += 1
        if fragment_gap_seconds > 0:
            time.sleep(fragment_gap_seconds)
    return sent


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--input", required=True, type=Path, help="Annex-B .h265/.hevc input")
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--port", type=int, default=3334)
    parser.add_argument("--fps", type=float, default=30.0, help="access-unit pacing rate")
    parser.add_argument("--mtu", type=int, default=MAX_UDP_PAYLOAD, help="UDP datagram payload bytes including 8-byte header")
    parser.add_argument("--fragment-gap-ms", type=float, default=0.0)
    parser.add_argument(
        "--packet-mode",
        choices=("access-unit", "nalu"),
        default="access-unit",
        help="decoder packet grouping mode",
    )
    parser.add_argument(
        "--no-prefix-parameter-sets",
        action="store_true",
        help="do not prepend cached VPS/SPS/PPS to access-unit packets",
    )
    parser.add_argument("--loop", action="store_true")
    args = parser.parse_args()

    if args.fps <= 0:
        raise SystemExit("--fps must be positive")
    if args.mtu <= HEADER_SIZE:
        raise SystemExit("--mtu must be greater than 8")

    data = args.input.read_bytes()
    units = split_annexb_units(data)
    packets = (
        build_access_units(units, not args.no_prefix_parameter_sets)
        if args.packet_mode == "access-unit"
        else units
    )
    if not packets:
        raise SystemExit("No HEVC packets were built from input")

    fragment_payload_size = args.mtu - HEADER_SIZE
    frame_interval = 1.0 / args.fps
    fragment_gap_seconds = args.fragment_gap_ms / 1000.0
    target = (args.host, args.port)

    nal_counts: dict[int, int] = {}
    for unit in units:
        nal_type = hevc_nal_type(unit)
        if nal_type is not None:
            nal_counts[nal_type] = nal_counts.get(nal_type, 0) + 1

    print(f"Loaded {len(units)} Annex-B NAL units from {args.input}")
    print(f"Built {len(packets)} {args.packet_mode} decoder packets")
    print(f"NAL type counts: {dict(sorted(nal_counts.items()))}")
    print(f"Sending to udp://{args.host}:{args.port}, mtu={args.mtu}, fps={args.fps}")

    sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
    frame_id = 0
    try:
        while True:
            for unit in packets:
                start = time.perf_counter()
                fragments = send_unit(sock, target, frame_id, unit, fragment_payload_size, fragment_gap_seconds)
                print(f"frame_id={frame_id} bytes={len(unit)} fragments={fragments}")
                frame_id = (frame_id + 1) & 0xFFFF

                elapsed = time.perf_counter() - start
                sleep_for = frame_interval - elapsed
                if sleep_for > 0:
                    time.sleep(sleep_for)

            if not args.loop:
                break
    finally:
        sock.close()


if __name__ == "__main__":
    main()
