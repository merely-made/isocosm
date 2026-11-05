"""Pack losslessly compressed GPU captures and the two measurement receipts."""
import base64
import hashlib
import json
import pathlib
import sys

source = pathlib.Path(__file__).parents[2] / "token_compare" / "report.py"
namespace = {}
exec(source.read_text(encoding="utf-8").split("receipt, destination =")[0], namespace)
compress = namespace["compress_png"]
receipt, confirmation, destination = map(pathlib.Path, sys.argv[1:4])
names, blobs = {}, {}
for path in receipt.glob("*.png"):
    if not path.name.startswith(("live-", "mixed-")):
        continue
    png = compress(path.read_bytes())
    key = hashlib.sha256(png).hexdigest()[:16]
    names[path.name] = key
    blobs[key] = "data:image/png;base64," + base64.b64encode(png).decode("ascii")
payload = {
    "report": json.loads((receipt / "hybrid.json").read_text(encoding="utf-8")),
    "confirmation": json.loads((confirmation / "hybrid.json").read_text(encoding="utf-8")),
    "names": names,
    "blobs": blobs,
}
fragment = pathlib.Path(__file__).with_name("viewer.html").read_text(encoding="utf-8").replace(
    "__HYBRID_COMPARISON_DATA__", json.dumps(payload, separators=(",", ":")).replace("</", "<\\/")
)
assert len(fragment.encode("utf-8")) < 1_000_000
destination.write_text(fragment, encoding="utf-8")
print(json.dumps({"bytes": len(fragment.encode("utf-8")), "capture_names": len(names), "unique_images": len(blobs)}))
