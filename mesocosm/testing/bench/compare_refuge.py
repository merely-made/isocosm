"""Compare the independent Cosmograph experiment with Mesocosm's paired observations."""
import argparse
import json
from pathlib import Path


def read(path):
    return json.loads(Path(path).read_text(encoding="utf-8-sig"))


def expected_patch(sample, patch):
    plants = patch["plants"].values()
    return {
        "eaten_by_grazer": sample["eaten"][patch["site"]],
        "grazed": sum(patch["grazed"].values()),
        "emerged": sum(patch["recruited"].values()),
        "shoots": sum(plant["exposed"] for plant in plants),
        "roots": sum(plant["root"] for plant in plants),
        "seeds": sum(plant["dormant"] for plant in plants),
        "grazers": patch["grazers"],
    }


def check(oracle, pairs, nudged):
    def equal(label, left, right):
        if left != right:
            raise ValueError(f"{label}: expected {left!r}, got {right!r}")

    for column, native in [("alone", "control"), ("trial", "nudged" if nudged else "control")]:
        run = oracle[native]
        equal("number of observations", len(run["samples"]), len(pairs))
        equal("opening hash", oracle["fork_hash"], pairs[0][column]["hash"])
        equal("final hash", run["final_hash"], pairs[-1][column]["hash"])
        equal(column + " route refusals", run["blocked_route_minutes"], pairs[-1][column]["blocked_route_minutes"])
        for sample, pair in zip(run["samples"], pairs, strict=True):
            tick = sample["tick"]
            label = f"{column} at minute {tick}"
            equal(label, (tick, oracle["grazer"]), (pair["tick"], pair["grazer"]))
            obs = pair[column]
            equal(label, (sample["alive"], sample["grazer_site"]), (obs["alive"], obs["site"]))
            equal(label + " residence minutes", sample["minutes"], obs["minutes"])
            equal(label + " patches", [expected_patch(sample, p) for p in sample["patches"]], obs["patches"])
        if native == "control":
            equal(column + " nudge", "none", pairs[-1][column]["nudge"])
        else:
            equal(column + " nudge", "answered" if run["answer"] else "expired", pairs[-1][column]["nudge"])


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("oracle")
    parser.add_argument("product")
    parser.add_argument("choice", choices=["nudge", "alone"])
    args = parser.parse_args()
    check(read(args.oracle), read(args.product), args.choice == "nudge")
    print(f"{args.choice}: both histories and all recorded patch observations agree")
