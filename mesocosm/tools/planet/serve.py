# Copyright 2026 Mark Alan Boykin
# SPDX-License-Identifier: MPL-2.0
"""Serve a published founding and continue its native session on demand."""
import argparse
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
from pathlib import Path
import subprocess
from threading import Lock
from urllib.parse import urlsplit, parse_qs

HERE = Path(__file__).resolve().parent
STATIC = {"": ("index.html", "text/html"), "globe.js": ("globe.js", "text/javascript"),
          "app.js": ("app.js", "text/javascript"), "style.css": ("style.css", "text/css")}


class Timeline:
    def __init__(self, directory, binary):
        self.frames = json.loads((directory / "frames.json").read_bytes())
        self.lock = Lock()
        self.child = subprocess.Popen([binary, "watch", directory / "resume.json"],
                                      stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True)
        if self.read("now")["hash"] != self.frames[-1]["hash"]:
            self.close()
            raise RuntimeError("The resumed planet differs from the last preview tick")

    def read(self, command):
        try:
            self.child.stdin.write(command + "\n")
            self.child.stdin.flush()
        except BrokenPipeError as error:
            raise RuntimeError("The simulation has stopped") from error
        line = self.child.stdout.readline()
        if not line:
            raise RuntimeError("The simulation has stopped")
        return json.loads(line)

    def at(self, tick):
        with self.lock:
            if not 0 <= tick <= len(self.frames):
                raise ValueError("Request an observed tick or its immediate successor")
            if tick == len(self.frames):
                frame = self.read("next")
                if frame["tick"] != tick:
                    raise RuntimeError("The planet advanced to an unexpected tick")
                self.frames.append(frame)
            return self.frames[tick]

    def close(self):
        self.child.stdin.close()
        self.child.wait()


def run():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    parser.add_argument("binary", type=Path)
    parser.add_argument("--port", type=int, default=0)
    args = parser.parse_args()
    timeline = Timeline(args.directory, args.binary)

    class Handler(BaseHTTPRequestHandler):
        def do_GET(self):
            target = urlsplit(self.path)
            key = target.path.removeprefix("/")
            try:
                if key in STATIC:
                    name, kind = STATIC[key]
                    data = (HERE / name).read_bytes()
                elif key in ("terrain.json", "frames.json", "founding.json"):
                    data, kind = (args.directory / key).read_bytes(), "application/json"
                elif key == "frame":
                    tick = int(parse_qs(target.query).get("tick", ["-1"])[0])
                    data = json.dumps(timeline.at(tick), separators=(",", ":")).encode()
                    kind = "application/json"
                else:
                    return self.send_error(404)
                self.send_response(200)
                self.send_header("Content-Type", kind)
                self.send_header("Content-Length", str(len(data)))
                self.send_header("Cache-Control", "no-store")
                self.end_headers()
                self.wfile.write(data)
            except ValueError as err:
                self.send_error(400, str(err))
            except (BrokenPipeError, ConnectionResetError):
                pass
            except RuntimeError as err:
                self.send_error(503, str(err))

    try:
        with ThreadingHTTPServer(("127.0.0.1", args.port), Handler) as server:
            print(f"http://127.0.0.1:{server.server_port}/", flush=True)
            server.serve_forever()
    finally:
        timeline.close()


if __name__ == "__main__":
    run()
