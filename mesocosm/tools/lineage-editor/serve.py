# Copyright 2026 Mark Alan Boykin
# SPDX-License-Identifier: MPL-2.0

"""Serve the Mesocosm founding editor on loopback, backed by native Cosmograph."""
import argparse
from functools import lru_cache
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
import os
import secrets
from pathlib import Path
import subprocess
import sys
import tempfile
from urllib.parse import urlsplit
from urllib.request import urlopen
import webbrowser

HERE = Path(__file__).resolve().parent
TEMPLATE = HERE.parents[2] / "shared/cosmograph/examples/optional-gland.toml"
ASSETS = {
    "/": ("index.html", "text/html; charset=utf-8"),
    "/pocket": ("pocket.html", "text/html; charset=utf-8"),
    "/pocket.js": ("pocket.js", "text/javascript"),
    "/fronds": ("fronds.html", "text/html; charset=utf-8"),
    "/fronds.js": ("fronds.js", "text/javascript"),
    "/style.css": ("style.css", "text/css"),
    "/app.js": ("app.js", "text/javascript"),
    "/anatomy.js": ("anatomy.js", "text/javascript"),
    "/survival.js": ("survival.js", "text/javascript"),
    "/drawing.js": ("drawing.js", "text/javascript"),
}


def native(args, data=None):
    result = subprocess.run(args, input=data, capture_output=True, timeout=90)
    if result.returncode:
        raise ValueError(result.stderr.decode().strip() or "Native founding failed")
    return result.stdout


class Editor:
    def __init__(self, directory, game=None, world_dir=None):
        self.game = game
        self.world_dir = world_dir
        self.directory = directory
        self.launch_key = secrets.token_urlsafe(24)
        self.port = None
        if game:
            if not game.is_file():
                raise SystemExit(f"Missing Mesocosm executable: {game}")
            if b"--enter-world " not in native([game, "--help"]):
                raise SystemExit("Rebuild Mesocosm with --enter-world support before connecting the editor")
        suffix = ".exe" if sys.platform == "win32" else ""
        self.observer = directory / "examples" / f"lineage_preview{suffix}"
        self.pocket_observer = directory / "examples" / f"pocket_preview{suffix}"
        self.bench = directory / f"cosmograph-bench{suffix}"
        for path in (self.observer, self.bench):
            if not path.is_file():
                raise SystemExit(f"Build the editor binaries first. Missing {path}")

    @lru_cache(maxsize=12)
    def preview(self, edits):
        options = json.loads(edits)
        template = TEMPLATE
        study = options.pop("study", None)
        if study == "pocket":
            branch = options.pop("branch", "stay")
            if branch not in ("stay", "leave"):
                raise ValueError("Unknown pocket timeline")
            data = self.pocket_preview(json.dumps(options, sort_keys=True))
            return {**data, "pocket_branch": branch,
                    "frames": data["frames"] if branch == "stay" else data["leave"]}
        if study == "fronds":
            template = TEMPLATE.with_name("frond-allocation.toml")
        return json.loads(native([self.observer, template], json.dumps(options).encode()))

    @lru_cache(maxsize=8)
    def pocket_preview(self, edits):
        return json.loads(native([self.pocket_observer], edits.encode()))

    def save(self, preview, tick, witness):
        frames = preview["frames"]
        if type(tick) is not int or not 0 <= tick < len(frames):
            raise ValueError("Tick outside the preview")
        if frames[tick]["hash"] != witness:
            raise ValueError("Selection belongs to a different preview")
        if "pocket_branch" in preview:
            return native([self.pocket_observer, "--save", preview["pocket_branch"], str(tick), witness],
                          json.dumps(preview["options"]).encode())
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "founding.toml"
            path.write_text(preview["source"], encoding="utf-8")
            save = path.with_suffix(".json")
            native([self.bench, "--founding", path, "--ticks", str(tick),
                    "--expect-state", witness, "--output", save])
            return save.read_bytes()


    def enter(self, preview, tick, witness):
        if not self.game:
            raise ValueError("Start the editor with a Mesocosm executable to enter a world")
        saved = self.save(preview, tick, witness)
        self.world_dir.mkdir(parents=True, exist_ok=True)
        folder = Path(tempfile.mkdtemp(prefix=f"seed-{preview['seed']}-tick-{tick}-", dir=self.world_dir))
        source = folder / "entry.json"
        source.write_bytes(saved)
        (folder / "founding.toml").write_text(preview["source"], encoding="utf-8")
        with (folder / "game.log").open("wb") as log:
            subprocess.Popen([self.game, "--enter-world", source, "--trace", folder / "world.json",
                              "--capture", folder / "scene.png", "--receipt", folder / "receipt.json"],
                             stdout=log, stderr=subprocess.STDOUT,
                             env={**os.environ, "MESOCOSM_EDITOR_BIN_DIR": str(self.directory),
                                  "MESOCOSM_EDITOR_PORT": str(self.port)})
        return {"folder": str(folder)}


class Handler(BaseHTTPRequestHandler):
    def send(self, status, content_type, data):
        self.send_response(status)
        self.send_header("Content-Type", content_type)
        self.send_header("Content-Length", str(len(data)))
        self.send_header("Cache-Control", "no-store")
        self.end_headers()
        try:
            self.wfile.write(data)
        except (BrokenPipeError, ConnectionResetError):
            pass

    def do_GET(self):
        port = self.server.server_port
        if self.headers.get("Host") not in {f"127.0.0.1:{port}", f"localhost:{port}"}:
            return self.send_error(403)
        if self.path == "/workshop":
            return self.send(200, "application/json", json.dumps({
                "editor": "mesocosm-lineage", "game": str(self.server.editor.game or "")
            }).encode())
        item = ASSETS.get(urlsplit(self.path).path)
        if item:
            name, kind = item
            self.send(200, kind, (HERE / name).read_bytes())
        else:
            self.send_error(404)

    def do_POST(self):
        if self.path not in ("/preview", "/save", "/enter"):
            return self.send_error(404)
        port = self.server.server_port
        local_hosts = {f"{host}:{port}" for host in ("localhost", "127.0.0.1")}
        if self.headers.get("Host") not in local_hosts:
            return self.send_error(403)
        origin = self.headers.get("Origin")
        if self.headers.get_content_type() != "application/json" or (
            origin and origin not in {f"http://{host}:{self.server.server_port}"
                                      for host in ("127.0.0.1", "localhost")}
        ):
            return self.send_error(403)
        try:
            size = int(self.headers.get("Content-Length", 0))
            if not 0 < size <= 2048:
                raise ValueError("Invalid editor request size")
            req = json.loads(self.rfile.read(size))
            if not isinstance(req, dict) or not isinstance(req.get("edits"), dict):
                raise ValueError("Expected a founding edit")
            editor = self.server.editor
            if self.path == "/enter" and not secrets.compare_digest(
                str(req.get("launch_key", "")), editor.launch_key
            ):
                return self.send_error(403)
            preview = editor.preview(json.dumps(req["edits"], sort_keys=True))
            if self.path == "/save":
                data = editor.save(preview, req.get("tick"), req.get("hash"))
            elif self.path == "/enter":
                data = json.dumps(editor.enter(preview, req.get("tick"), req.get("hash"))).encode()
            else:
                data = json.dumps({**preview, "launch_key": editor.launch_key if editor.game else None}).encode()
            self.send(200, "application/json", data)
        except (ValueError, subprocess.TimeoutExpired, OSError) as exc:
            self.send(422, "text/plain; charset=utf-8", str(exc).encode())


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--bin-dir", type=Path, required=True,
                        help="Cargo release directory containing the bench and preview example")
    parser.add_argument("--from-menu", action="store_true",
                        help="open a running game-enabled editor or start one")
    parser.add_argument("--game", type=Path, help="Mesocosm executable used by Enter world")
    parser.add_argument("--world-dir", type=Path, default=Path.home() / "Mesocosm" / "worlds",
                        help="directory for worlds entered from the editor")
    parser.add_argument("--port", type=int, default=8765)
    parser.add_argument("--open", action="store_true", help="open the editor in your browser")
    args = parser.parse_args()
    game = args.game.resolve() if args.game else None
    if args.from_menu:
        args.port = int(os.environ.get("MESOCOSM_EDITOR_PORT", args.port))
        if not game:
            parser.error("--from-menu requires --game")
        url = f"http://127.0.0.1:{args.port}"
        try:
            with urlopen(url + "/workshop", timeout=1) as response:
                metadata = json.load(response)
            if metadata == {"editor": "mesocosm-lineage", "game": str(game)}:
                webbrowser.open(url)
                return
        except (OSError, ValueError):
            pass
    editor = Editor(args.bin_dir.resolve(), game, args.world_dir.resolve())
    if args.from_menu:
        # The game needs a quick success/failure result. A fresh service prints
        # its bound URL once. Only the menu launcher opens the browser.
        for port in dict.fromkeys((args.port, 0)):
            child = subprocess.Popen([sys.executable, __file__, "--bin-dir", editor.directory,
                "--game", game, "--world-dir", editor.world_dir, "--port", str(port)],
                stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True)
            with child.stdout:
                line = child.stdout.readline().strip()
            prefix = "Mesocosm lineage editor at "
            if line.startswith(prefix):
                webbrowser.open(line[len(prefix):])
                return
            child.wait()
        raise SystemExit("Could not start the local lineage editor")
    with ThreadingHTTPServer(("127.0.0.1", args.port), Handler) as server:
        server.editor = editor
        editor.port = server.server_port
        url = f"http://127.0.0.1:{server.server_port}/"
        print(f"Mesocosm lineage editor at {url}", flush=True)
        if args.open:
            webbrowser.open(url)
        try:
            server.serve_forever()
        except KeyboardInterrupt:
            pass


if __name__ == "__main__":
    main()
