"""The headed half of the paging receipts: B5's frame profile, taken again.

Each host binary runs two sessions of the shipping host, each left to run 25
seconds and stopped: `headed-demo`, the demo board under the flag as B5
measured it, and `headed-256`, the 256 by 256 stress board
(`ISOMETRY_SYNTH=256`). Both arm the overlay self-test, so the host keeps
drawing a still board until the test fires at three seconds. Hosts built at
several commits run back to back and interleaved, so a before and an after
share one window of the machine's load; the demo session is each run's own
control for what that load was.

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
    """Both sessions of one host binary, into `out/<session>`."""
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
        env.pop("ISOMETRY_SCENE_HEADROOM", None)
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


def hosts(out: pathlib.Path) -> list[pathlib.Path]:
    """The host runs under `out`, in the order they ran."""
    return sorted(path for path in out.glob("host-*") if path.is_dir())


def section(out: pathlib.Path) -> str:
    runs = [host for host in hosts(out) if any((host / s / "stderr.log").exists() for s in SESSIONS)]
    if not runs:
        return ""
    lines = [
        "",
        "## Headed: a steady frame, host against host",
        "",
        "The shipping host in the dev build at device scale 2, interface zoom",
        "0.917 and render scale 2, as B5 measured (`headed.py` has the window).",
        "B5's scene board read 12.50 ms over 183 frames of the demo board. The",
        "hosts ran back to back in the order listed, each more than once and",
        "interleaved, so every host has samples from across one window of the",
        "machine's load, and each session's demo control shows what that load was.",
        "",
        "| host | session | n | total, ms | p25 | p75 | p95 | range | emit | a11y | raster"
        " | producer |",
        "| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |",
    ]
    for host in runs:
        for name in SESSIONS:
            log = host / name / "stderr.log"
            if not log.exists():
                continue
            run = profile(log)
            m = run["median"]
            lines.append(
                f"| {host.name.removeprefix('host-')} | {name} | {run['n']} | {m['total']:.2f} "
                f"| {run['p25']:.2f} | {run['p75']:.2f} | {run['p95']:.2f} "
                f"| {run['low']:.2f} to {run['high']:.2f} | {m['emit']:.2f} | {m['a11y']:.2f} "
                f"| {m['raster']:.2f} | {m['producer']:.2f} |"
            )
    lines += ["", "What the ground cost in each session, as the host printed it:", ""]
    for host in runs:
        for name in SESSIONS:
            log = host / name / "stderr.log"
            if log.exists():
                run = profile(log)
                label = f"{host.name.removeprefix('host-')} {name}"
                lines.append(f"- `{label}`, first frame: {run['first']}")
                lines.append(f"- `{label}`, the self-test's overlays: {run['overlays']}")
    return NEWLINE.join(lines) + NEWLINE


def checks(out: pathlib.Path) -> list[pathlib.Path]:
    """Extra 256 sessions run beside the hosts, such as a setting's control."""
    return sorted(path for path in (out / "headroom-check").glob("*") if path.is_dir())


def files(out: pathlib.Path) -> list[pathlib.Path]:
    found = []
    runs = [host / name for host in hosts(out) for name in SESSIONS] + checks(out)
    for run in runs:
        for leaf in ("stderr.log", "isometry_capture.png", "source.txt"):
            path = run / leaf
            if path.exists():
                found.append(path)
    return found


def pictures(out: pathlib.Path) -> str:
    """Which 256 captures are the same picture. The self-test hovers the
    first of the tiles tied on `col + row` that a `HashMap` yields, which moves
    between processes, so captures are compared within one hover target."""
    runs = [host / "headed-256" for host in hosts(out)] + checks(out)
    rows = []
    for run in runs:
        capture, log = run / "isometry_capture.png", run / "stderr.log"
        if not (capture.exists() and log.exists()):
            continue
        text = log.read_text(encoding="utf-8", errors="replace")
        target = re.search(r"path to (\(\d+, \d+\))", text)
        asked = re.search(r"scene board headroom (\d+) layers", text)
        commit = (run / "source.txt").read_text(encoding="utf-8").split()[1]
        headroom = asked.group(1) if asked else ("none" if commit in PRE_HEADROOM else "1")
        digest = hashlib.sha256(capture.read_bytes()).hexdigest()[:16]
        rows.append((target.group(1) if target else "?", headroom, run, digest))
    if not rows:
        return ""
    lines = [
        "",
        "## The same picture",
        "",
        "Each 256 capture with its self-test's hover target and the headroom it",
        "drew with (`none` predates the setting). The self-test hovers the first",
        "of the tiles tied on `col + row` that a `HashMap` yields, which differs",
        "between processes, so pictures compare within one target. Identical",
        "pixels encode to identical bytes here, so equal hashes are equal pictures;",
        "where PIL is present, the pixels that differ from the target's first",
        "capture are counted.",
        "",
        "| target | headroom | run | capture sha256 | pixels unlike the first |",
        "| --- | --- | --- | --- | --- |",
    ]
    reference = {}
    for target, headroom, run, digest in sorted(rows, key=lambda r: (r[0], r[1] != "none", str(r[2]))):
        first = reference.setdefault(target, run / "isometry_capture.png")
        label = run.relative_to(out).as_posix()
        lines.append(
            f"| {target} | {headroom} | `{label}` | `{digest}` "
            f"| {unlike(first, run / 'isometry_capture.png')} |"
        )
    return NEWLINE.join(lines) + NEWLINE


PRE_HEADROOM = {"31370bf", "c6fb846"}


def unlike(first: pathlib.Path, other: pathlib.Path) -> str:
    try:
        import numpy
        from PIL import Image
    except ImportError:
        return "-"
    load = lambda path: numpy.asarray(Image.open(path).convert("RGB")).astype(int)  # noqa: E731
    return f"{int((numpy.abs(load(first) - load(other)).sum(axis=2) > 0).sum()):,}"
