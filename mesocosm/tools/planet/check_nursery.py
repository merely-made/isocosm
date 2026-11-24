# Copyright 2026 Mark Alan Boykin
# SPDX-License-Identifier: MPL-2.0
"""Advance a matched nursery pair with the field camera elsewhere; reopen and measure it."""
import argparse
from concurrent.futures import ThreadPoolExecutor
import json
from pathlib import Path
import sqlite3

from check_soil import touching
from field_server import FieldTrip
from recruitment import inspect


def matched(near, far, site):
    def genesis(directory):
        return json.loads((directory / "founding.json").read_bytes())["native"]["genesis"]
    a, b = genesis(near), genesis(far)
    left, right = (g["rules"]["soil_beds"][str(site)]["colonies"] for g in (a, b))
    shifted = [id_ for id_ in left if left[id_] != right.get(id_)]
    if len(shifted) != 1:
        raise ValueError("Expected one displaced nurse")
    nurse = shifted[0]
    x, y, z = left[nurse]["at"]
    if right[nurse]["at"] != [x, y, z + 40]:
        raise ValueError("Expected the same nurse 40 units off the receiving patch")
    right[nurse]["at"] = [x, y, z]
    if a != b or a["world"]["base_unit_micrometres"] != 1000:
        raise ValueError("Foundings differ outside the nurse displacement or have different units")
    return int(nurse)


def advance(directory, binary, watch, through):
    trip = FieldTrip(directory, binary)
    try:
        head = trip.frame()["head"]
        if head > through:
            raise ValueError("The stored head is past the requested horizon")
        while head < through:
            head = trip.advance(head, watch)["head"]
            if head == through // 2:
                trip.disconnect()
    finally:
        trip.close()


def measure(directory, binary, site, watch, parent, nurse):
    family = inspect(directory, binary, parent, site)
    daughters = {m["id"]: dict(m, landing=None, contact=[], purchases=[], growth=0, awakeInterval=None)
                 for m in family["family"] if m["parent"] == parent}
    origin, previous_contacts = None, set()
    awake_since = {}
    parent_gone = False
    with sqlite3.connect((directory / "field.sqlite3").as_uri() + "?mode=ro", uri=True) as db:
        for tick, raw in db.execute("SELECT tick,data FROM frames WHERE tick<=? ORDER BY tick", (family["tick"],)):
            frame = json.loads(raw)
            if tick and frame["fieldSite"] != watch:
                raise ValueError(f"Camera left the control site at week {tick}")
            poses = {p["id"]: p for p in frame["poses"] if p["site"] == site}
            patch = frame["sites"][site]
            if any(g["first"] <= parent < g["first"] + g["count"]
                   for i, other in enumerate(frame["sites"]) if i != site for g in other["residents"]):
                raise ValueError(f"Parent visited another site at week {tick}")
            if parent in poses:
                if parent_gone:
                    raise ValueError(f"Parent returned from an unobserved gap at week {tick}")
                at = poses[parent]["pose"]["at"]
                if origin is None:
                    origin = at
                if at != origin:
                    raise ValueError(f"Parent moved at week {tick}")
            elif any(g["first"] <= parent < g["first"] + g["count"] for g in patch["residents"]):
                raise ValueError(f"Resident parent lost its root position at week {tick}")
            else:
                parent_gone = True
            contacts = set()
            for root in patch["soil"]["roots"]:
                id_ = root["id"]
                if id_ not in daughters:
                    continue
                daughter = daughters[id_]
                resident = next(g for g in patch["residents"] if g["first"] <= id_ < g["first"] + g["count"])
                if resident.get("dormant"):
                    awake_since.pop(id_, None)
                else:
                    first = awake_since.setdefault(id_, tick)
                    if tick - first >= family["interval"] and daughter["awakeInterval"] is None:
                        daughter["awakeInterval"] = [first, tick]
                at = poses[id_]["pose"]["at"]
                if daughter["landing"] is None:
                    daughter["landing"] = [tick, at]
                elif daughter["landing"][1] != at:
                    raise ValueError(f"Rooted daughter #{id_} moved")
                if nurse in poses and any(touching(box[1:3], root["bounds"]) for box in poses[nurse]["boxes"]):
                    contacts.add(id_)
                    daughter["contact"].append(tick)
            for flow in patch["flows"]:
                id_ = flow["to"]
                if id_ not in daughters:
                    continue
                if flow["process"] == "ecology:root-production" and flow["debit"] == "world:soil":
                    daughters[id_]["growth"] += flow["amount"]
                if (flow["process"] != "ecology:root-exchange" or flow["from"] != nurse
                        or flow["debit"] != "soil:phosphate"):
                    continue
                paid = sum(f["amount"] for f in patch["flows"] if f["process"] == flow["process"]
                           and f["from"] == id_ and f["to"] == nurse and f["debit"].startswith("matter:"))
                if id_ not in previous_contacts or paid != flow["amount"]:
                    raise ValueError(f"Trade without prior physical contact and payment at week {tick}")
                daughters[id_]["purchases"].append([tick, paid])
            previous_contacts = contacts
    p = family["family"][0]
    if p["journey"] or (p["site"] is not None and p["site"] != site):
        raise ValueError("Parent left its bed")
    return dict(branch=family["branch"], hash=family["reopenedHash"], week=family["tick"],
                watched=watch, site=site, parent=parent, origin=origin, nurse=nurse,
                parentLastSeen=p["lastSeen"], interval=family["interval"], daughters=list(daughters.values()))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=Path)
    parser.add_argument("near", type=Path)
    parser.add_argument("far", type=Path)
    parser.add_argument("--site", type=int, default=1)
    parser.add_argument("--watch", type=int, default=98)
    parser.add_argument("--parent", type=int, default=174)
    parser.add_argument("--through", type=int, required=True)
    args = parser.parse_args()
    if args.site == args.watch or args.through <= 0:
        parser.error("Choose a different camera site and a positive horizon")
    near, far, binary = (p.resolve() for p in (args.near, args.far, args.binary))
    nurse = matched(near, far, args.site)
    def run(directory):
        advance(directory, binary, args.watch, args.through)
        return measure(directory, binary, args.site, args.watch, args.parent, nurse)
    with ThreadPoolExecutor(max_workers=2) as pool:
        a, b = pool.map(run, (near, far))
    print(json.dumps(dict(near=a, far=b), indent=2))


if __name__ == "__main__":
    main()
