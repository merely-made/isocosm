# Copyright 2026 Mark Alan Boykin
# SPDX-License-Identifier: MPL-2.0
"""Measure a plant family from a saved field survey and its native parentage."""
import argparse
from bisect import bisect_left
from collections import Counter
import json
from pathlib import Path
import sqlite3
import subprocess
import tempfile

from field_server import FieldTrip


def reopened(binary, save, ancestor):
    with tempfile.NamedTemporaryFile() as checkpoint:
        checkpoint.write(save)
        checkpoint.flush()
        run = subprocess.run([binary, "watch", checkpoint.name],
                             input=f"now\nfamily {ancestor}\n", text=True,
                             capture_output=True, check=True)
    now, family = map(json.loads, run.stdout.splitlines())
    return now, family


def plant_lineage(founding, db, ancestor, end):
    for first, group in founding["population"]["groups"].items():
        if int(first) <= ancestor < int(first) + group["count"]:
            return group["entity"]["lineage"]
    # A later-born ancestor has a recorded provision under its parent's
    # cycle, even if the newborn immediately left the site.
    for (raw,) in db.execute("SELECT data FROM frames WHERE tick<=? ORDER BY tick", (end,)):
        for patch in json.loads(raw)["sites"]:
            for flow in patch["flows"]:
                if flow["to"] == ancestor and flow["process"].startswith("ecology:birth-"):
                    return "lineage:" + flow["process"].removeprefix("ecology:birth-")
    raise ValueError(f"No founding group or birth recorded for #{ancestor}")


def inspect(directory, binary, ancestor=174, site=1):
    # Frames are append-only. Read the committed head once and bound every
    # query to it so a slow native replay never holds the viewer's write lock.
    with sqlite3.connect((directory / "field.sqlite3").as_uri() + "?mode=ro", uri=True) as db:
        end, save = db.execute("SELECT tick,save FROM head WHERE id=1").fetchone()
        founding = json.loads((directory / "founding.json").read_bytes())["native"]["genesis"]
        if str(site) not in founding["sites"]:
            raise ValueError(f"Unknown site {site}")
        native, family = reopened(binary, save, ancestor)
        rows = {m["id"]: dict(m, bites=[], children=[], lastSeen=None,
                              firstAwake=None, established=None, birth=None)
                for m in family["members"]}
        family_ids = sorted(rows)
        lineage = plant_lineage(founding, db, ancestor, end)
        if founding["lineages"][lineage]["kingdom"] != "kingdom:flora":
            raise ValueError("Recruitment measures a plant family")
        cycle = lineage.removeprefix("lineage:")
        birth = f"ecology:birth-{cycle}"
        interval = founding["rules"]["processes"][birth]["period"]
        focal_tissue = f"matter:{cycle}-0"
        exchange = Counter()
        plant_products = 0
        milestones = []
        population = []
        site_grazed = 0
        animal_feeds = {f"ecology:feed-{name.removeprefix('lineage:')}"
                        for name, line in founding["lineages"].items() if line["kingdom"] == "kingdom:fauna"}
        producer_keys = {
            "matter:" + name.removeprefix("lineage:") + "-0"
            for name, line in founding["lineages"].items() if line["kingdom"] == "kingdom:flora"
        }
        for expected, (tick, data) in enumerate(db.execute("SELECT tick,data FROM frames WHERE tick<=? ORDER BY tick", (end,))):
            frame = json.loads(data)
            if tick != expected or frame["tick"] != tick:
                raise ValueError("Gap in field observations")
            # Collect births before reading the newborn's first census.
            for i, patch in enumerate(frame["sites"]):
                for flow in patch["flows"]:
                    src, dest = flow["from"], flow["to"]
                    if flow["process"] == birth and src in rows and dest is not None:
                        if dest not in rows or rows[dest]["parent"] != src:
                            raise ValueError(f"Native parentage disagrees at birth of {dest}")
                        if rows[dest]["birth"] is None:
                            rows[dest]["birth"] = [tick, i]
                            rows[src]["children"].append(dest)
                    if i == site and flow["process"] in animal_feeds and flow["debit"] in producer_keys:
                        if src is not None and any(g["first"] <= src < g["first"] + g["count"] for g in patch["residents"]):
                            site_grazed += flow["amount"]
                    if flow["process"] in animal_feeds and src in rows and flow["debit"] == focal_tissue:
                        rows[src]["bites"].append([tick, i, dest, flow["amount"]])
                    if flow["process"] == "ecology:root-exchange" and ancestor in (src, dest):
                        exchange[flow["debit"]] += flow["amount"]
                    if flow["process"] == "ecology:root-production" and dest == ancestor and flow["debit"] == "world:soil":
                        plant_products += flow["amount"]
                for resident in patch["residents"]:
                    first = resident["first"]
                    lo = bisect_left(family_ids, first)
                    hi = bisect_left(family_ids, first + resident["count"], lo)
                    for member_id in family_ids[lo:hi]:
                        member = rows[member_id]
                        member["lastSeen"] = [tick, i, resident["tissue"]]
                        if resident.get("dormant"):
                            continue
                        if member["firstAwake"] is None:
                            member["firstAwake"] = [tick, i]
                        born = member["birth"]
                        if born and tick - born[0] >= interval and member["established"] is None:
                            member["established"] = [tick, i]
            calendar = frame.get("calendar")
            boundary = calendar and (4 * tick) % calendar["yearTicks"] == 0
            if tick in (0, end) or boundary:
                totals = {}
                for patch in frame["sites"]:
                    for resident in patch["residents"]:
                        group = totals.setdefault(resident["kingdom"], {"residents": 0, "tissue": 0})
                        group["residents"] += resident["count"]
                        group["tissue"] += resident["count"] * resident["tissue"]
                population.append(dict(tick=tick, kingdoms=totals))
            if boundary:
                patch = frame["sites"][site]
                milestones.append(dict(tick=tick, temperature=patch.get("temperature"),
                                       litter=patch["soil"]["remaining"]))
        if native["hash"] != frame["hash"] or family["tick"] != tick or tick != end:
            raise ValueError("Reopened family or world differs from the stored head")
        for m in rows.values():
            if m["parent"] is not None and m["birth"] is None:
                raise ValueError(f"No provision receipt for child {m['id']}")
            # A travelling body is absent from every resident list, but its
            # living state and journey are authoritative in the reopened head.
            if m["alive"] and m["birth"] and not m["dormant"] and end - m["birth"][0] >= interval and m["established"] is None:
                m["established"] = [end, m["site"]]
            seen = end if m["alive"] else m["lastSeen"][0] if m["lastSeen"] else -1
            # Animal ports can also take carrion. Keep transfers only where
            # this same plant is evidenced alive at or after the meal.
            m["bites"] = [bite for bite in m["bites"] if bite[0] <= seen]
            m["survivedBite"] = any(bite[0] < seen for bite in m["bites"])
        daughters = [m for m in rows.values() if m["parent"] == ancestor]
        return dict(branch=frame["branch"], tick=end, reopenedHash=native["hash"], interval=interval,
                    ancestor=ancestor, exchange=dict(exchange), rootSoilUsed=plant_products, seasons=milestones,
                    residentCommunity=population, confirmedGrazingOnSiteResidents=site_grazed,
                    daughters=len(daughters), established=sum(m["established"] is not None for m in daughters),
                    daughtersWithFullInterval=sum(end - m["birth"][0] >= interval for m in daughters),
                    daughtersGrazed=sum(bool(m["bites"]) for m in daughters),
                    daughtersReproduced=sum(bool(m["children"]) for m in daughters),
                    daughtersSurvivedBite=sum(m["survivedBite"] for m in daughters),
                    family=sorted(rows.values(), key=lambda m: (m["generation"], m["id"])))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("binary", type=Path)
    parser.add_argument("directory", type=Path)
    parser.add_argument("--ancestor", type=int, default=174)
    parser.add_argument("--site", type=int, default=1)
    parser.add_argument("--through", type=int, help="extend a field copy to this tick before measuring")
    args = parser.parse_args()
    directory, binary = args.directory.resolve(), args.binary.resolve()
    if args.through is not None:
        trip = FieldTrip(directory, binary)
        try:
            tick = trip.frame()["head"]
            if tick > args.through:
                parser.error("The saved head is past the requested horizon")
            while tick < args.through:
                tick = trip.advance(tick, args.site)["head"]
        finally:
            trip.close()
    print(json.dumps(inspect(directory, binary, args.ancestor, args.site), indent=2))


if __name__ == "__main__":
    main()
