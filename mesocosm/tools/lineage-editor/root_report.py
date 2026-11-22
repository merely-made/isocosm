# Copyright 2026 Mark Alan Boykin
# SPDX-License-Identifier: MPL-2.0

"""Render the native root_patch observations and link the matching fork saves."""
import argparse
import json
from pathlib import Path


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path, help="output directory from root_patch")
    directory = parser.parse_args().directory
    runs = json.loads((directory / "trials.json").read_text())
    data = json.dumps(runs, separators=(",", ":")).replace("<", r"\u003c")
    template = Path(__file__).with_name("root-report.html").read_text()
    (directory / "index.html").write_text(template.replace("DATA_HERE", data))


if __name__ == "__main__":
    main()
