# Copyright 2026 Mark Alan Boykin
# SPDX-License-Identifier: MPL-2.0
import json
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
from tempfile import TemporaryDirectory
from unittest import TestCase, main

from field_server import FieldTrip


class FieldTripPersistence(TestCase):
    def test_concurrent_visitors_share_one_round_and_resume_from_it(self):
        with TemporaryDirectory() as tmp:
            path = Path(tmp)
            founding = {"tick": 0, "fieldSite": 0, "sites": [{}, {}], "hash": "founding"}
            for name in ("field.json", "founding.json"):
                (path / name).write_text(json.dumps(founding))
            # Speak the same saved-head protocol as the native process.
            binary = path / "watch"
            binary.write_text("""#!/usr/bin/env python3
import json, sys
with open(sys.argv[2]) as saved:
    frame = json.load(saved)
for command in sys.stdin:
    words = command.split()
    if words[0] == "visit":
        tick = frame["tick"] + 1
        site = int(words[1])
        frame.update(tick=tick, fieldSite=site, hash=f"{frame['hash']}/{site}")
        result = {"save": frame, "frame": frame}
    else:
        result = frame
    print(json.dumps(result), flush=True)
""")
            binary.chmod(0o755)
            clients = [FieldTrip(path, binary) for _ in range(2)]
            try:
                with ThreadPoolExecutor(2) as pool:
                    calls = [pool.submit(client.advance, 0, site) for site, client in enumerate(clients)]
                    a, b = [call.result() for call in calls]
                self.assertEqual(a, b)
                self.assertEqual(a["head"], 1)
            finally:
                for client in clients:
                    client.close()
            reopened = FieldTrip(path, binary)
            try:
                self.assertEqual(reopened.frame(), a)
                self.assertEqual(reopened.advance(0, 1), a)
                self.assertEqual(reopened.frame(0)["frame"], founding)
                with self.assertRaises(ValueError):
                    reopened.advance(2, 1)
                following = reopened.advance(1, 0)
                self.assertEqual(following["frame"]["hash"], a["frame"]["hash"] + "/0")
                self.assertEqual(following["head"], 2)
            finally:
                reopened.close()


if __name__ == "__main__":
    main()
