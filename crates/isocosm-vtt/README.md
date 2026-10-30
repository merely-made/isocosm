# Isocosm: VTT

The game layer for **Isocosm: VTT**, the virtual tabletop of the Isocosm
family (formerly Isometry; renamed 2026-09-22).

A pixel-art isometric virtual tabletop: a peer-to-peer map editor and
turn-based play substrate for D&D, Pathfinder and other systems, over the
Isocosm simulator. This crate owns native campaign founding, the table's
game actions and its overlay contract. The optional `endpoint` feature
exposes the table's Graphshell panels; defaults keep that endpoint off.

The endpoint includes **Character sheets**, an experimental D&D SRD 5.1
free-play path for Magic Weapon. It reuses the system plugin's casting,
weapon attack, concentration and expiry procedures and saves edition facts
beside the native world through the existing family authority. Equipment on
this page is labelled table equipment; it does not create a native physical
item. See the [supported loop and boundaries](../isometry-system/data/effects/README.md).

The host, views and session layer remain in the `isometry-*` and `isonetry`
crates of <https://github.com/merely-made/isocosm>. The crate carries the
VTT's MIT OR Apache-2.0 licence.
