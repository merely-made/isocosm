"""Recompute the driver's lineage count per world, isocosm::draw in Python,
to check the ecology worlds left identical by ruling 287 are those with one
lineage, a producer alone, which has no feeding process."""
import hashlib, struct


def draw(seed, domain, values):
    h = hashlib.sha256()
    h.update(struct.pack("<Q", seed))
    d = domain.encode()
    h.update(struct.pack("<Q", len(d)))
    h.update(d)
    for v in values:
        h.update(struct.pack("<Q", v))
    return struct.unpack("<Q", h.digest()[:8])[0]


for name, seed, worlds, identical in [
    ("fuzz 7", 7, 24, [1, 7, 15]),
    ("fuzz 11", 11, 60, [11, 17, 29, 51]),
    ("tight 13", 13, 48, [9, 11, 15, 25, 27, 39, 41]),
]:
    single = [w for w in range(worlds) if w % 2 == 1 and 1 + draw(seed, "lineages", [w]) % 4 == 1]
    print(name, "one-lineage ecology worlds", single, "identical", identical, single == identical)
