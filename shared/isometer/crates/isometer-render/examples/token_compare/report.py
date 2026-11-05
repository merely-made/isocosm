"""Embed actual benchmark captures and metrics in the comparison viewer.

PNG compression changes serialization only; pixel bytes stay unchanged.
"""
import base64
import json
import pathlib
import struct
import sys
import zlib


def compress_png(raw):
    pos, chunks, image = 8, [], bytearray()
    while pos < len(raw):
        size = struct.unpack_from(">I", raw, pos)[0]
        kind, data = raw[pos + 4:pos + 8], raw[pos + 8:pos + 8 + size]
        if kind == b"IDAT":
            image.extend(data)
        else:
            chunks.append((kind, data))
        pos += size + 12
    pixels = zlib.decompress(image)
    encoded = zlib.compress(pixels, 9)
    assert zlib.decompress(encoded) == pixels
    chunks.insert(-1, (b"IDAT", encoded))
    return b"\x89PNG\r\n\x1a\n" + b"".join(
        struct.pack(">I", len(data)) + kind + data
        + struct.pack(">I", zlib.crc32(kind + data) & 0xffffffff)
        for kind, data in chunks
    )


receipt, destination = map(pathlib.Path, sys.argv[1:3])
report = json.loads((receipt / "comparison.json").read_text())
confirmation_path = receipt.with_name(receipt.name + "-confirmation") / "comparison.json"
confirmation = json.loads(confirmation_path.read_text()) if confirmation_path.exists() else None
images = {}
for path in receipt.glob("*.png"):
    if path.name.startswith(("live-", "sprite-")):
        encoded = compress_png(path.read_bytes())
        images[path.name] = "data:image/png;base64," + base64.b64encode(encoded).decode("ascii")
template = pathlib.Path(__file__).with_name("viewer.html").read_text(encoding="utf-8")
data = json.dumps({"report": report, "confirmation": confirmation, "images": images}, separators=(",", ":")).replace("</", "<\\/")
fragment = template.replace("__TOKEN_COMPARISON_DATA__", data)
assert len(fragment.encode("utf-8")) < 1_000_000
assert "__TOKEN_COMPARISON_DATA__" not in fragment
destination.parent.mkdir(parents=True, exist_ok=True)
destination.write_text(fragment, encoding="utf-8")
print(json.dumps({"viewer": str(destination), "bytes": len(fragment.encode("utf-8")), "images": len(images)}))
