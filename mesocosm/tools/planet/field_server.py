# Copyright 2026 Mark Alan Boykin
# SPDX-License-Identifier: MPL-2.0
"""Keep the field trip's observations and its resumable native head in one transaction."""

import json
import sqlite3
import subprocess
import tempfile
from threading import Lock


def encode(value):
    return json.dumps(value, separators=(",", ":")).encode()


class FieldTrip:
    def __init__(self, directory, binary):
        self.binary = binary
        self.lock, self.child, self.child_tick = Lock(), None, None
        self.db = sqlite3.connect(directory / "field.sqlite3", timeout=60, check_same_thread=False)
        self.db.executescript("""
            CREATE TABLE IF NOT EXISTS frames(tick INTEGER PRIMARY KEY, data BLOB NOT NULL);
            CREATE TABLE IF NOT EXISTS head(id INTEGER PRIMARY KEY CHECK(id=1), tick INTEGER, save BLOB);
        """)
        first = (directory / "field.json").read_bytes()
        self.site_count = len(json.loads(first)["sites"])
        with self.db:
            self.db.execute("INSERT OR IGNORE INTO frames VALUES(0,?)", (first,))
            self.db.execute("INSERT OR IGNORE INTO head VALUES(1,0,?)", ((directory / "founding.json").read_bytes(),))
        if self.frame(0)["frame"]["hash"] != json.loads(first)["hash"]:
            raise ValueError("The field database belongs to a different founding")

    def frame(self, tick=None):
        with self.lock:
            head, = self.db.execute("SELECT tick FROM head").fetchone()
            item = self.db.execute("SELECT data FROM frames WHERE tick=?", (head if tick is None else tick,)).fetchone()
            if not item:
                raise ValueError("That tick has not been observed")
            return {"head": head, "frame": json.loads(item[0])}

    def read(self, command):
        try:
            self.child.stdin.write(command + "\n")
            self.child.stdin.flush()
        except BrokenPipeError as error:
            raise RuntimeError("The field simulation stopped") from error
        result = self.child.stdout.readline()
        if not result:
            raise RuntimeError("The field simulation stopped")
        return json.loads(result)

    def disconnect(self):
        if self.child:
            try:
                self.child.stdin.close()
            except BrokenPipeError:
                pass
            self.child.wait()
            self.child.stdout.close()
            self.child = self.child_tick = None

    def advance(self, start, site):
        if type(start) is not int or start < 0 or type(site) is not int or not 0 <= site < self.site_count:
            raise ValueError("Choose a valid site and tick")
        with self.lock, self.db:
            self.db.execute("BEGIN IMMEDIATE")
            head, = self.db.execute("SELECT tick FROM head").fetchone()
            if start < head:
                data, = self.db.execute("SELECT data FROM frames WHERE tick=?", (start + 1,)).fetchone()
                return {"frame": json.loads(data), "head": head}
            if start != head:
                raise ValueError("The requested starting tick is ahead of this world")
            try:
                if self.child_tick != head:
                    self.disconnect()
                    saved, = self.db.execute("SELECT save FROM head").fetchone()
                    published, = self.db.execute("SELECT data FROM frames WHERE tick=?", (head,)).fetchone()
                    with tempfile.NamedTemporaryFile() as checkpoint:
                        checkpoint.write(saved)
                        checkpoint.flush()
                        self.child = subprocess.Popen([self.binary, "watch", checkpoint.name],
                            stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True)
                        if self.read("now")["hash"] != json.loads(published)["hash"]:
                            raise RuntimeError("The reopened field differs from its recorded head")
                result = self.read("visit " + str(site))
                frame = result["frame"]
                if frame["tick"] != head + 1 or frame["fieldSite"] != site:
                    raise RuntimeError("Unexpected native field observation")
                self.db.execute("INSERT INTO frames VALUES(?,?)", (head + 1, encode(frame)))
                self.db.execute("UPDATE head SET tick=?, save=? WHERE id=1", (head + 1, encode(result["save"])))
                self.db.commit()
                self.child_tick = head + 1
                return {"frame": frame, "head": head + 1}
            except Exception:
                self.disconnect()
                raise

    def close(self):
        with self.lock:
            self.disconnect()
            self.db.close()
