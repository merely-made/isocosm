// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Compact world-time plants (1115), founded above their store's ground
//! plane. Each has one exposed head and four to sixteen lit fronds.

use super::*;

pub(super) fn bounds(b: &Bodies) -> Bodies {
    Bodies {
        sensing: true,
        grazing: true,
        hazard: [0, 0],
        wound: [0, 0],
        ..b.clone()
    }
}

pub(super) fn recipe(seed: u64, index: u64, kinds: &mut BTreeMap<Key, Template>) -> Result<Recipe> {
    // A positive solid root carries the store cells. Its geometry, rather
    // than those cells, places it entirely below the local ground plane.
    let root = "kind:root-store".to_string();
    kinds.insert(
        root.clone(),
        Template {
            half_extent: [2, 2, 2],
            cells: BTreeMap::from([(anatomy::STORE.into(), anatomy::capacity([2, 2, 2]))]),
            shape: String::new(),
            port: None,
        },
    );
    let head = "kind:rooted-head".to_string();
    kinds.insert(
        head.clone(),
        kinds
            .get(&kind("block"))
            .ok_or("no plant head kind")?
            .clone(),
    );
    let frond = "kind:rooted-frond".to_string();
    let mut lit = kinds.get(&kind("frond")).ok_or("no frond kind")?.clone();
    // This is an explicit new founding route, as for named body sheets:
    // native volume geometry needs positive half-extents.
    lit.half_extent = lit.half_extent.map(|h| h.max(1));
    kinds.insert(frond.clone(), lit);
    let mut root_tagma = bare(1, "root-store");
    root_tagma.segment = root;
    let mut head_tagma = bare(1, "rooted-head");
    head_tagma.segment = head;
    head_tagma.parent = Some(0);
    head_tagma.facing = Facing::Above;
    let mut tagmata = vec![root_tagma, head_tagma];
    let stretches = 1 + crate::draw(seed, "recipe-stretches", &[index, 0]) % 2;
    for s in 0..stretches {
        let segments = 4 + crate::draw(seed, "recipe-segments", &[index, s]) % 5;
        let mut run = bare(segments as u8, "rooted-frond");
        run.segment = frond.clone();
        run.facing = Facing::Above;
        // Sheets are on the lit side of the underground store; the first
        // stretch branches from the exposed head, subsequent ones continue.
        if s == 0 {
            run.parent = Some(1);
        }
        run.socket = Facing::Above;
        tagmata.push(run);
    }
    Ok(Recipe {
        tagmata,
        variance: 0,
        absence: [0, 1],
        riff: [0, 1],
        vary: [0, 1],
    })
}
