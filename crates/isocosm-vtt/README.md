# Isocosm: VTT

The game layer for **Isocosm: VTT**, the virtual tabletop of the Isocosm
family (formerly Isometry; renamed 2026-09-22).

A pixel-art isometric virtual tabletop: a peer-to-peer map editor and
turn-based play substrate for D&D, Pathfinder and other systems, over the
Isocosm simulator. This crate owns native campaign founding, the table's
game actions and its overlay contract. The optional `endpoint` feature
exposes the table's Graphshell panels; defaults keep that endpoint off.

The host, views and session layer remain in the `isometry-*` and `isonetry`
crates of <https://github.com/merely-made/isocosm>. The crate carries the
VTT's MIT OR Apache-2.0 licence.
