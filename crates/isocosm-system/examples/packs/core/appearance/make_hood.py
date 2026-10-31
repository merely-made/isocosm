"""Writes hood.vox, the core pack's one .vox layer (original content).

The layer is authored here as box fills in the pack's own frame (x right,
y up, z depth, front at z = 0) and written as a MagicaVoxel file, whose axes
are x right, y depth, z up. Colour slot 10 draws palette index 9 (the
hood's cloth): a .vox's colour slot n draws palette index n - 1.

Run: python make_hood.py   (writes hood.vox beside this script)
"""
import os
import struct

CLOTH = 9 + 1  # palette index 9, as a .vox colour slot

# [x0, x1, y0, y1, z0, z1] half-open, in the pack frame.
FILLS = [
    (2, 8, 22, 24, 2, 7),   # crown
    (2, 3, 16, 22, 2, 7),   # left side
    (7, 8, 16, 22, 2, 7),   # right side
    (3, 7, 14, 22, 6, 7),   # back fall
    (1, 9, 15, 17, 4, 7),   # draped over the shoulders
    (3, 7, 22, 23, 1, 2),   # brim over the brow
]


def voxels():
    cells = {}
    for x0, x1, y0, y1, z0, z1 in FILLS:
        for x in range(x0, x1):
            for y in range(y0, y1):
                for z in range(z0, z1):
                    cells[(x, y, z)] = CLOTH
    return cells


def chunk(name, content, children=b""):
    return name + struct.pack("<ii", len(content), len(children)) + content + children


def write(path):
    cells = voxels()
    size = struct.pack("<iii", 10, 8, 24)  # MagicaVoxel x, y (depth), z (up)
    body = struct.pack("<i", len(cells))
    for (x, y, z), slot in sorted(cells.items()):
        body += struct.pack("<BBBB", x, z, y, slot)
    palette = [(0x4a, 0x5a, 0x3a, 0xff)] * 256
    rgba = b"".join(struct.pack("<BBBB", *c) for c in palette)
    children = chunk(b"SIZE", size) + chunk(b"XYZI", body) + chunk(b"RGBA", rgba)
    with open(path, "wb") as f:
        f.write(b"VOX " + struct.pack("<i", 150) + chunk(b"MAIN", b"", children))


if __name__ == "__main__":
    write(os.path.join(os.path.dirname(os.path.abspath(__file__)), "hood.vox"))
