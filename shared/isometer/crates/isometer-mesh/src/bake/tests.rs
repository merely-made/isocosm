// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The bake lane's own suite, carried over from `isometry-voxel` with its
//! literal pins intact, plus the sheet-byte pins that gate the merge.

use super::*;
use crate::voxel::Voxels;

#[test]
fn bakes_a_nonempty_facing() {
    let (hero, pal) = demo::hero();
    let sheet = bake_facing(&hero, &pal, 0, &BakeParams::default());
    assert!(sheet.w > 0 && sheet.h > 0);
    // A 10x24x8 figure at half_w 5 should cover a healthy pixel count.
    assert!(sheet.opaque_pixels() > 500, "got {}", sheet.opaque_pixels());
}

#[test]
fn facings_differ() {
    let (hero, pal) = demo::hero();
    let p = BakeParams::default();
    let f0 = bake_facing(&hero, &pal, 0, &p);
    let f1 = bake_facing(&hero, &pal, 1, &p);
    // Same figure, different view: pixel data must differ.
    assert!(
        f0.rgba != f1.rgba || f0.w != f1.w,
        "facings 0 and 1 look identical"
    );
}

#[test]
fn palette_swap_keeps_silhouette_changes_color() {
    let (hero, base) = demo::hero();
    // A recolour: shift every entry toward blue.
    let recolor = Palette::new(base.0.iter().map(|c| [c[0] / 3, c[1] / 3, 255]).collect());
    let p = BakeParams::default();
    let a = bake_facing(&hero, &base, 0, &p);
    let b = bake_facing(&hero, &recolor, 0, &p);
    assert_eq!(
        a.alpha_mask(),
        b.alpha_mask(),
        "recolour must not move the silhouette"
    );
    assert!(a.rgba != b.rgba, "recolour must change pixels");
}

#[test]
fn png_data_uri_is_wellformed() {
    let (hero, pal) = demo::hero();
    let sheet = bake_facing(&hero, &pal, 0, &BakeParams::default());
    let png = sheet.to_png();
    // PNG signature.
    assert_eq!(&png[..8], &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);
    // IHDR width (offset 16 = 8 sig + 4 len + 4 type) matches the sheet.
    let w = u32::from_be_bytes([png[16], png[17], png[18], png[19]]);
    assert_eq!(w as i32, sheet.w);
    // Ends with IEND.
    assert_eq!(&png[png.len() - 8..png.len() - 4], b"IEND");
    let uri = sheet.to_png_data_uri();
    assert!(uri.starts_with("data:image/png;base64,"));
    assert!(uri.len() > 100);
}

#[cfg(feature = "vox")]
// Build a spec-valid `.vox` in memory: MAIN { SIZE, XYZI, RGBA }.
fn synth_vox() -> Vec<u8> {
    fn chunk(id: &[u8; 4], content: &[u8], children: &[u8]) -> Vec<u8> {
        let mut c = Vec::new();
        c.extend_from_slice(id);
        c.extend_from_slice(&(content.len() as u32).to_le_bytes());
        c.extend_from_slice(&(children.len() as u32).to_le_bytes());
        c.extend_from_slice(content);
        c.extend_from_slice(children);
        c
    }
    let mut size = Vec::new();
    for d in [2u32, 2, 3] {
        size.extend_from_slice(&d.to_le_bytes()); // mv x=2, y=2, z=3 (up)
    }
    let voxels = [(0u8, 0u8, 0u8, 1u8), (1, 1, 2, 2)];
    let mut xyzi = (voxels.len() as u32).to_le_bytes().to_vec();
    for (x, y, z, i) in voxels {
        xyzi.extend_from_slice(&[x, y, z, i]);
    }
    let mut rgba = vec![0u8; 256 * 4];
    rgba[0..4].copy_from_slice(&[220, 60, 60, 255]); // palette slot for index 1
    rgba[4..8].copy_from_slice(&[60, 200, 90, 255]); // palette slot for index 2
    let mut children = chunk(b"SIZE", &size, &[]);
    children.extend(chunk(b"XYZI", &xyzi, &[]));
    children.extend(chunk(b"RGBA", &rgba, &[]));
    let mut out = b"VOX ".to_vec();
    out.extend_from_slice(&150u32.to_le_bytes());
    out.extend(chunk(b"MAIN", &[], &children));
    out
}

#[cfg(feature = "vox")]
#[test]
fn loads_and_bakes_a_vox_model() {
    let (vox, pal) = load_vox(&synth_vox()).expect("parse .vox");
    // mv 2x2x3 (Z up) remaps to ours dx=2, dy=3 (height), dz=2 (depth).
    assert_eq!((vox.dx, vox.dy, vox.dz), (2, 3, 2));
    assert_eq!(vox.filled(), 2);
    // Index 1 resolves to the red we placed at file slot 0 (MagicaVoxel's
    // i -> palette[i-1], handled by the rotate in load_vox).
    assert_eq!(pal.color(1), [220, 60, 60]);
    assert_eq!(pal.color(2), [60, 200, 90]);
    let sheet = bake_facing(&vox, &pal, 0, &BakeParams::default());
    assert!(sheet.opaque_pixels() > 0, "a .vox bakes");
}

#[test]
fn compose_stacks_layers() {
    // Two disjoint single-voxel layers compose into a two-voxel volume.
    let mut a = Voxels::new(2, 1, 1);
    a.set(0, 0, 0, 0);
    let mut b = Voxels::new(2, 1, 1);
    b.set(1, 0, 0, 1);
    let out = compose(&[&a, &b]);
    assert_eq!(out.filled(), 2);
    assert_eq!(out.get(0, 0, 0), Some(0));
    assert_eq!(out.get(1, 0, 0), Some(1));
}

#[test]
fn appearance_round_trips_json() {
    let (_, pal) = demo::hero();
    let app = Appearance {
        layers: vec!["body".into(), "hair".into()],
        palette: pal,
        clips: vec![Clip {
            name: "idle".into(),
            frames: vec![0, 1, 2],
        }],
    };
    let json = serde_json::to_string(&app).unwrap();
    let back: Appearance = serde_json::from_str(&json).unwrap();
    assert_eq!(back.layers, app.layers);
    assert_eq!(back.clips[0].name, "idle");
}

// --- L2 identity pins -------------------------------------------------------
//
// The merge's done condition is pixel-identical sheets. These hash the bake's
// own output, at the parameters the real consumers use, against values
// captured from `isometry-voxel` at eb8b764 before the move. A change here is
// a change to every tileset sprite Isometry ships, and must be deliberate.
//
// `tower-beast.png`'s pin is the blake3 of the byte stream the
// `watchtower_sprites` example wrote before the move, so the encoder is
// covered as well as the projection.

fn sheet_digest(sheet: &Sheet) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(&sheet.w.to_le_bytes());
    hasher.update(&sheet.h.to_le_bytes());
    hasher.update(&sheet.rgba);
    hasher.finalize().to_hex().to_string()
}

/// The board's parameters: `isometry-views` bakes every token and prop here.
const BOARD: BakeParams = BakeParams {
    half_w: 2,
    cube_h: 2,
    facings: 4,
    margin: 2,
};

#[test]
fn the_demo_rig_bakes_to_the_same_pixels_as_before_the_merge() {
    let (hero, palette) = demo::hero();
    assert_eq!(
        sheet_digest(&bake_facing(&hero, &palette, 0, &BakeParams::default())),
        "1e2dd72ea82e3b56493782837c3868693400ff033e9b924941f3a3b890145e7d"
    );
    assert_eq!(
        sheet_digest(&bake_strip(&hero, &palette, &BOARD)),
        "4f7c002eabfbe5a87a3d975495aefff79aa4800e5f973cbc262d6152fc2e0698"
    );
    let (tile, palette) = demo::tile();
    assert_eq!(
        sheet_digest(&bake_facing(&tile, &palette, 0, &BOARD)),
        "a31f5ad1908d44c750e095b7974c949af7f5dcd3f33caac4bc13c3feaacb43bc"
    );
}

#[test]
fn the_watchtower_props_bake_to_the_same_pixels_as_before_the_merge() {
    let (beast, palette) = watchtower::tower_beast();
    assert_eq!(
        sheet_digest(&bake_strip(&beast, &palette, &BOARD)),
        "8a3a41bb21408b43fe0de38a9129293a6dd76189123a2ec9345f47639b388f0f"
    );
    let pins = [
        (
            "rubble",
            watchtower::rubble(),
            "92d813d9dccaf73b246127ee43f5bd9fcbbb369d816862c5b314aae235267987",
        ),
        (
            "bell",
            watchtower::bell(),
            "9b5bee5c78a6a6f2a480b896f8605ef44ddcf9e5e9e31d083d1409fe0a35fe52",
        ),
        (
            "nest",
            watchtower::nest(),
            "bd14745b82b2bc0603bb04811c6239061b7101bf6e371996aecf0b4647200ca6",
        ),
        (
            "forest_tree",
            watchtower::forest_tree(),
            "e75c1ffff0c94e7eda05e4b8703a5e632c7f3fe65787eeb0016be93b1fda7c86",
        ),
    ];
    for (name, (model, palette), pin) in pins {
        assert_eq!(
            sheet_digest(&bake_facing(&model, &palette, 0, &BOARD)),
            pin,
            "{name} moved"
        );
    }
}

#[test]
fn the_beast_sheet_encodes_to_the_same_png_bytes() {
    let (beast, palette) = watchtower::tower_beast();
    let png = bake_strip(&beast, &palette, &BOARD).to_png();
    assert_eq!(
        blake3::hash(&png).to_hex().to_string(),
        "3c8ccd30f1074b497ce7fc23ca15f3a8b520c3591d40220e0dc32acf6910c99a"
    );
}
