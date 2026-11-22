# Copyright 2026 Mark Alan Boykin
# SPDX-License-Identifier: MPL-2.0
"""Verify the seed 9, site 1 litter trial using the actual field saves."""
import argparse
from concurrent.futures import ThreadPoolExecutor
import json
from pathlib import Path
import subprocess

from field_server import FieldTrip


def touching(first, second):
    overlaps = [min(first[1][i], second[1][i]) - max(first[0][i], second[0][i]) for i in range(3)]
    return min(overlaps) >= 0 and sum(n > 0 for n in overlaps) >= 2


def check_buried(directory, bounds):
    terrain = json.loads((directory / "terrain.json").read_bytes())
    ground = terrain["sites"][1]["ground"]
    n = ground["samples"]
    side = terrain["side"]
    heights = ground["heights"]
    lo, hi = bounds
    # Same fixed-point native relief samples used by the section. Check the
    # entire top of this small deposit at the field's 1 mm resolution.
    for x in range(lo[0], hi[0] + 1):
        i, a = divmod(x * n, side)
        for z in range(lo[2], hi[2] + 1):
            j, b = divmod(z * n, side)
            offset = j * (n + 1) + i
            north = (side - a) * heights[offset] + a * heights[offset + 1]
            south = (side - a) * heights[offset + n + 1] + a * heights[offset + n + 2]
            assert (side - b) * north + b * south > hi[1] * side * side * 16, "litter exposed above soil"


def verify(directory, binary, ticks, near):
    trip = FieldTrip(directory, binary)
    try:
        frames = []
        for tick in range(ticks + 1):
            packet = trip.frame(0) if tick == 0 else trip.advance(tick - 1, 1)
            frames.append(packet["frame"])
            if tick == 2:
                trip.close()
                trip = FieldTrip(directory, binary)
    finally:
        trip.close()

    # The uninterrupted run must reproduce geometry, transfers and the native
    # state hash at every tick, including the first tick after the reopen.
    replay = subprocess.run([binary, "watch", directory / "founding.json"],
                            input="now\n" + "next\n" * ticks, text=True,
                            capture_output=True, check=True)
    assert [json.loads(line) for line in replay.stdout.splitlines()] == frames
    initial = frames[0]["sites"][1]["soil"]
    assert len(initial["colonies"]) == 2 and all(c["parts"] == 1 and c["host"] for c in initial["colonies"])
    remaining = initial["remaining"]
    previous_parts = {c["id"]: c["parts"] for c in initial["colonies"]}
    first_contact = first_trade = None
    delivered = incorporated = carbon = grazed = 0
    stored = {c["id"]: c["p"] for c in initial["colonies"]}
    previous_contact = set()
    for frame in frames:
        local = frame["sites"][1]
        bed = local["soil"]
        tick = frame["tick"]
        assert len(bed["colonies"]) == len({c["id"] for c in bed["colonies"]})
        roots = {r["id"]: r["bounds"] for r in bed["roots"]}
        receipt = [f for f in local["flows"] if f["process"] == "ecology:root-exchange"]
        uptake = sum(f["amount"] for f in local["flows"] if f["process"] == "ecology:recover-phosphate")
        touching_litter = {c["id"] for c in bed["colonies"] if c["contact"]}
        for f in local["flows"]:
            if f["process"] == "ecology:recover-phosphate":
                assert f["to"] in touching_litter | previous_contact
        assert remaining - bed["remaining"] == uptake
        remaining = bed["remaining"]
        for c in bed["colonies"]:
            assert c["parts"] <= previous_parts[c["id"]] + 1
            previous_parts[c["id"]] = c["parts"]
            poses = [p for p in frame["poses"] if p["id"] == c["id"]]
            assert len(poses) == 1 and poses[0]["soil"]
            bounds = [b[1:3] for b in poses[0]["boxes"]]
            contact = any(touching(b, bed["litter"]) for b in bounds)
            assert len(bounds) == c["parts"] and contact == c["contact"]
            if c.get("host") is not None:
                assert any(touching(b, roots[c["host"]]) for b in bounds)
            if contact and first_contact is None:
                first_contact = tick
        if uptake or receipt:
            assert first_contact is not None and near
        if receipt:
            if first_trade is None:
                first_trade = tick
            give = {(f["from"], f["to"]): f["amount"] for f in receipt if f["debit"] == "soil:phosphate"}
            pay = {(f["to"], f["from"]): f["amount"] for f in receipt if f["debit"].startswith("matter:")}
            assert give == pay and len(give) > 0
            delivered += sum(give.values())
            carbon += sum(pay.values())
        # The tracked founders retain mobile phosphate or exchange it. This
        # also catches an exchange hidden by a plant leaving in the same tick.
        for c in bed["colonies"]:
            ident = c["id"]
            received = sum(f["amount"] for f in local["flows"] if f["process"] == "ecology:recover-phosphate" and f["to"] == ident)
            sent = sum(f["amount"] for f in receipt if f["debit"] == "soil:phosphate" and f["from"] == ident)
            assert c["p"] == stored[ident] + received - sent, f"unaccounted colony phosphate at tick {tick}"
            stored[ident] = c["p"]
        previous_contact = touching_litter
        incorporated = max(incorporated, sum(r["used"] for r in bed["roots"]))
        grazed += sum(f["amount"] for f in local["flows"] if f["process"] == "ecology:feed-1")
    assert all(parts > 2 for parts in previous_parts.values())
    if near:
        assert first_contact is not None and first_trade is not None, "missing physical contact or trade"
        assert 0 < first_contact <= first_trade <= ticks and delivered > 0
        assert remaining < initial["remaining"]
    else:
        assert first_contact is None and first_trade is None and remaining == initial["remaining"]
    return dict(ticks=ticks, first_contact=first_contact, first_trade=first_trade,
                final_litter=remaining, delivered=delivered, carbon_paid=carbon,
                peak_incorporated_at_roots=incorporated, grazed=grazed,
                final_hash=frames[-1]["hash"])


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=Path)
    parser.add_argument("near", type=Path)
    parser.add_argument("far", type=Path)
    parser.add_argument("--ticks", type=int, default=12)
    args = parser.parse_args()
    origins = []
    for path in [args.near, args.far]:
        data = json.loads((path / "founding.json").read_bytes())["native"]["genesis"]
        box = data["rules"]["soil_beds"]["1"].pop("litter")
        check_buried(path, box)
        origins.append((data, box))
    (a, near), (b, far) = origins
    assert a == b
    assert [[x, y, z + 40] for x, y, z in near] == far
    binary = args.binary.resolve()
    with ThreadPoolExecutor(max_workers=2) as pool:
        near_run = pool.submit(verify, args.near, binary, args.ticks, True)
        far_run = pool.submit(verify, args.far, binary, args.ticks, False)
        print(json.dumps({"near": near_run.result(), "far": far_run.result()}, indent=2))


if __name__ == "__main__":
    main()
