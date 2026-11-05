"""Losslessly compress actual captures into the inline comparison."""
import base64
import json
import pathlib
import sys

# Keep the previous report's standalone compressor without executing its CLI.
source = pathlib.Path(__file__).parent.parent / "token_compare" / "report.py"
namespace = {}
exec(source.read_text().split("receipt, destination =")[0], namespace)
compress = namespace["compress_png"]
receipt, destination = map(pathlib.Path, sys.argv[1:3])
report = json.loads((receipt / "comparison.json").read_text())
images = {
    p.name: "data:image/png;base64," + base64.b64encode(compress(p.read_bytes())).decode("ascii")
    for p in receipt.glob("*.png") if p.name.startswith(("live-", "sprite-"))
}
confirmation = json.loads((pathlib.Path(sys.argv[3]) / "comparison.json").read_text()) if len(sys.argv) > 3 else None
fragment = pathlib.Path(__file__).with_name("viewer.html").read_text(encoding="utf-8").replace(
    "__CREATURE_COMPARISON_DATA__",
    json.dumps({"report": report, "confirmation": confirmation, "images": images}, separators=(",", ":")).replace("</", "<\\/")
)
assert len(fragment.encode()) < 1_000_000
assert "__CREATURE_COMPARISON_DATA__" not in fragment
destination.write_text(fragment, encoding="utf-8")
print(json.dumps({"viewer": str(destination), "bytes": len(fragment.encode()), "captures": len(images)}))
