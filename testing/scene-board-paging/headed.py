"""The headed half of the paging receipts: B5's frame profile, taken again.

Two sessions of the shipping host, each left to run 25 seconds and stopped:
`headed-demo`, the demo board under the flag as B5 measured it, which is the
same-session control against B5's 12.50 ms; and `headed-256`, the 256 by 256
stress board (`ISOMETRY_SYNTH=256`). Both arm the overlay self-test, so the
host keeps drawing a still board until the test fires at three seconds.

The window is B5's: the frames before the self-test fires, less the first
three and any frame that captured. Each session keeps its `stderr.log`, its
capture and a `source.txt` naming the commit, the binary and the switches.
"""

import hashlib
import os
import pathlib
import re
import statistics
import subprocess
import time

NEWLINE = chr(10)
SESSIONS = {"headed-demo": None, "headed-256": "256"}
FIELD = re.compile(r"([a-z][a-z0-9-]*)=(\d+)(?:us)?")
PHASES = ["total", "emit", "a11y", "raster", "relayout", "present", "producer"]


def source(directory: pathlib.Path, exe: pathlib.Path, commit: str, synth) -> None:
    digest = hashlib.sha256(exe.read_bytes()).hexdigest()
    switches = "ISOMETRY_SCENE_BOARD=1 ISOMETRY_PROFILE=1 ISOMETRY_OVERLAY_SELFTEST=1"
    if synth:
        switches += f" ISOMETRY_SYNTH={synth}"
    (directory / "source.txt").write_text(
        NEWLINE.join([
            f"commit {commit}",
            f"binary {exe.name} sha256 {digest}",
            f"switches {switches} ISOMETRY_CAPTURE_DIR=<this directory>",
            "run 25 s, then stopped",
            "",
        ]),
        encoding="utf-8",
        newline=NEWLINE,
    )


def run(exe: pathlib.Path, out: pathlib.Path, commit: str, cwd: pathlib.Path) -> None:
    for name, synth in SESSIONS.items():
        directory = out / name
        directory.mkdir(parents=True, exist_ok=True)
        env = dict(os.environ)
        env.update({
            "ISOMETRY_SCENE_BOARD": "1",
            "ISOMETRY_PROFILE": "1",
            "ISOMETRY_OVERLAY_SELFTEST": "1",
            "ISOMETRY_CAPTURE_DIR": str(directory),
        })
        env.pop("ISOMETRY_SYNTH", None)
        if synth:
            env["ISOMETRY_SYNTH"] = synth
        with open(directory / "stderr.log", "wb") as err, open(directory / "stdout.log", "wb") as log:
            process = subprocess.Popen([str(exe)], cwd=cwd, env=env, stdout=log, stderr=err)
            time.sleep(25)
            process.kill()
            process.wait()
        source(directory, exe, commit, synth)


def profile(log: pathlib.Path) -> dict:
    frames, grounds, fired = [], [], False
    for line in log.read_text(encoding="utf-8", errors="replace").splitlines():
        if "overlay selftest" in line:
            fired = True
        elif "scene board ground:" in line:
            grounds.append((fired, line.split("scene board ground: ", 1)[1]))
        elif "frame total=" in line and not fired:
            frames.append({key: int(value) for key, value in FIELD.findall(line)})
    steady = [frame for frame in frames[3:] if frame.get("capture", 0) == 0]
    totals = sorted(frame["total"] / 1e3 for frame in steady)
    quarters = statistics.quantiles(totals, n=4)
    twentieths = statistics.quantiles(totals, n=20)
    return {
        "n": len(steady),
        "p25": quarters[0],
        "p75": quarters[2],
        "p95": twentieths[18],
        "low": totals[0],
        "high": totals[-1],
        "median": {p: statistics.median(f.get(p, 0) / 1e3 for f in steady) for p in PHASES},
        "stages": max(frame.get("stages", 0) for frame in steady),
        "first": next((line for was_fired, line in grounds if not was_fired), None),
        "overlays": next((line for was_fired, line in grounds if was_fired), None),
    }


def section(out: pathlib.Path) -> str:
    runs = {name: out / name / "stderr.log" for name in SESSIONS}
    runs = {name: log for name, log in runs.items() if log.exists()}
    if not runs:
        return ""
    lines = [
        "",
        "## Headed: a steady frame, beside B5",
        "",
        "The shipping host in the dev build at device scale 2, interface zoom",
        "0.917 and render scale 2, as B5 measured (`headed.py` has the window).",
        "B5's scene board read 12.50 ms (p25 11.82, p75 13.91, p95 16.77) over",
        "183 frames of the demo board; the demo session here is its control.",
        "",
        "| session | n | total, ms | p25 | p75 | p95 | range | emit | a11y | raster | producer"
        " | stages |",
        "| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |",
    ]
    for name, log in runs.items():
        run = profile(log)
        m = run["median"]
        lines.append(
            f"| {name} | {run['n']} | {m['total']:.2f} | {run['p25']:.2f} | {run['p75']:.2f} "
            f"| {run['p95']:.2f} | {run['low']:.2f} to {run['high']:.2f} | {m['emit']:.2f} "
            f"| {m['a11y']:.2f} | {m['raster']:.2f} | {m['producer']:.2f} | {run['stages']} |"
        )
    lines += ["", "What the ground cost in each session, as the host printed it:", ""]
    for name, log in runs.items():
        run = profile(log)
        lines.append(f"- `{name}`, first frame: {run['first']}")
        lines.append(f"- `{name}`, the self-test's overlays: {run['overlays']}")
    return NEWLINE.join(lines) + NEWLINE


def files(out: pathlib.Path) -> list[pathlib.Path]:
    found = []
    for name in SESSIONS:
        for leaf in ("stderr.log", "isometry_capture.png", "source.txt"):
            path = out / name / leaf
            if path.exists():
                found.append(path)
    return found
