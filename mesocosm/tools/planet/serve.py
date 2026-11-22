# Copyright 2026 Mark Alan Boykin
# SPDX-License-Identifier: MPL-2.0
"""Serve a published founding and continue its native session on demand."""
import argparse
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
from pathlib import Path
import subprocess
from threading import Lock
from field_server import FieldTrip
from urllib.parse import urlsplit, parse_qs

HERE = Path(__file__).resolve().parent
STATIC = {"": ("index.html", "text/html"), "globe.js": ("globe.js", "text/javascript"),
          "ground.js": ("ground.js", "text/javascript"),
          "app.js": ("app.js", "text/javascript"), "style.css": ("style.css", "text/css")}


class Timeline:
    def __init__(self, directory, binary):
        self.frames = json.loads((directory / "frames.json").read_bytes())
        self.lock = Lock()
        self.binary, self.resume = binary, directory / "resume.json"
        self.child = None

    def resume_native(self):
        if self.child is not None:
            return
        self.child = subprocess.Popen([self.binary, "watch", self.resume],
                                      stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True)
        try:
            if self.read("now")["hash"] != self.frames[-1]["hash"]:
                raise RuntimeError("The resumed planet differs from the last preview tick")
        except Exception:
            self.close()
            raise

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
                self.resume_native()
                frame = self.read("next")
                if frame["tick"] != tick:
                    raise RuntimeError("The planet advanced to an unexpected tick")
                self.frames.append(frame)
            return self.frames[tick]

    def close(self):
        if self.child is not None:
            self.child.stdin.close()
            self.child.wait()
            self.child = None


def run():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    parser.add_argument("binary", type=Path)
    parser.add_argument("--port", type=int, default=0)
    parser.add_argument("--field", type=Path, help="an independent native field founding to visit")
    args = parser.parse_args()
    field = FieldTrip(args.field, args.binary) if args.field else None
    timeline = Timeline(args.directory, args.binary)

    class Handler(BaseHTTPRequestHandler):
        def send_data(self, data, kind):
            self.send_response(200)
            self.send_header("Content-Type", kind)
            self.send_header("Content-Length", str(len(data)))
            self.send_header("Cache-Control", "no-store")
            self.end_headers()
            self.wfile.write(data)

        def do_POST(self):
            if self.path != "/field/advance" or field is None:
                return self.send_error(404)
            if self.headers.get_content_type() != "application/json":
                return self.send_error(415)
            try:
                length = int(self.headers.get("Content-Length", 0))
                if not 0 < length <= 256:
                    return self.send_error(400)
                body = json.loads(self.rfile.read(length))
                data = field.advance(body["tick"], body["site"])
                self.send_data(json.dumps(data,separators=(",",":")).encode(), "application/json")
            except (ValueError, TypeError, KeyError) as err:
                self.send_error(400, str(err))
            except (BrokenPipeError, ConnectionResetError):
                pass
            except RuntimeError as err:
                self.send_error(503, str(err))

        def do_GET(self):
            target = urlsplit(self.path)
            key = target.path.removeprefix("/")
            try:
                if key in STATIC:
                    name, kind = STATIC[key]
                    data = (HERE / name).read_bytes()
                    if key == "" and field:
                        data = data.replace(b'id="field-link" hidden', b'id="field-link"')
                elif field and key in ("field", "field.js", "field-map.js", "soil-view.js", "field.css"):
                    name = "field.html" if key == "field" else key
                    kind = {"html":"text/html","js":"text/javascript","css":"text/css"}[name.split(".")[-1]]
                    data = (HERE / name).read_bytes()
                elif field and key == "field/terrain":
                    data, kind = (args.field / "terrain.json").read_bytes(), "application/json"
                elif field and key == "field/frame":
                    params = parse_qs(target.query)
                    tick = int(params["tick"][0]) if "tick" in params else None
                    data, kind = json.dumps(field.frame(tick),separators=(",",":")).encode(), "application/json"
                elif key in ("terrain.json", "frames.json", "founding.json"):
                    data, kind = (args.directory / key).read_bytes(), "application/json"
                elif key == "frame":
                    tick = int(parse_qs(target.query).get("tick", ["-1"])[0])
                    data = json.dumps(timeline.at(tick), separators=(",", ":")).encode()
                    kind = "application/json"
                else:
                    return self.send_error(404)
                self.send_data(data, kind)
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
        if field:
            field.close()


if __name__ == "__main__":
    run()
