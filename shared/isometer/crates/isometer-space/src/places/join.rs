// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Passages across a border: places of two sites joined through the frame
//! relation, read cell by cell along their shared side. A stance steps into
//! the neighbour's edge column as it would within a site; a stance meets
//! water across the border as it would beside it; water meets water and a
//! swimmer crosses. Both sites read heights in one frame, so a step is the
//! same number from either side.

use super::cells::{Cell, Cells, HEADROOM};
use super::passages::{Crossings, file, passage};
use super::{Kind, Passage, Place, PlaceId, Places};
use crate::border::{across_cell, edge_cell};
use crate::volume::Volume;
use crate::{Atlas, Result};

/// The passages between `here`'s places and `there`'s across every border
/// their sites share, as far as both windows reach.
pub fn join<A: Atlas + ?Sized>(
    a: &A,
    here: (&Volume, &Places),
    there: (&Volume, &Places),
) -> Result<Vec<Passage>> {
    let length = a.footprint().side as i64;
    let mut crossings = Crossings::new();
    for (near, far) in [(here, there), (there, here)] {
        let (cn, cf) = (Cells::new(near.0, near.1.rules), Cells::new(far.0, far.1.rules));
        for side in 0..a.footprint().sides {
            let Some((to, b)) = a.border(near.0.site, side).filter(|(to, _)| *to == far.0.site) else {
                continue;
            };
            let _ = to;
            let (_, _, n) = crate::edit::frame(side, length);
            let (_, _, m) = crate::edit::frame(b.enters, length);
            let ways = [[-n[0], -n[1]], [-m[0], -m[1]]];
            for t in 0..length {
                let [x, z] = edge_cell(side, t, length);
                let [fx, _, fz] = across_cell([x - n[0], 0, z - n[1]], b, length);
                if !near.0.holds(x, z) || !far.0.holds(fx, fz) {
                    continue;
                }
                for y in cn.floor..=cn.top {
                    let at = [x, y, z];
                    let Some(p) = near.1.label(at) else { continue };
                    if cn.cell(at) == Cell::Water {
                        let w = [fx, y, fz];
                        if let Some(q) = far.1.label(w).filter(|_| cf.cell(w) == Cell::Water) {
                            let open = (0..HEADROOM)
                                .take_while(|d| matches!(cn.cell([x, y + d, z]), Cell::Water | Cell::Air))
                                .count() as i64;
                            file(&mut crossings, [p, q], [at, w], ways, open, 0);
                        }
                        continue;
                    }
                    if !cn.stance(at) {
                        continue;
                    }
                    let room = cn.headroom(at);
                    for (bc, rise) in cn.step_into(at, room, &cf, [fx, fz]) {
                        if let Some(q) = far.1.label(bc) {
                            file(&mut crossings, [p, q], [at, bc], ways, room.min(cf.headroom(bc)), rise);
                        }
                    }
                    let mut w = [fx, y, fz];
                    while w[1] > cf.floor && cf.air(w) {
                        w[1] -= 1;
                    }
                    if let Some(q) = far.1.label(w).filter(|_| cf.cell(w) == Cell::Water) {
                        file(&mut crossings, [p, q], [at, w], ways, room, w[1] - y);
                    }
                }
            }
        }
    }
    let place = |id: &PlaceId| -> Option<&Place> {
        [here.1, there.1].into_iter().find_map(|p| p.places.get(id).filter(|_| p.site == id.site))
    };
    Ok(crossings
        .into_iter()
        .map(|(between, set)| {
            let centre = |k: usize| place(&between[k]).map_or(between[k].cell, |p| p.centre);
            let mut far = centre(1);
            if between[1].site != between[0].site {
                let border = (0..a.footprint().sides)
                    .find_map(|k| a.border(between[1].site, k).filter(|(to, _)| *to == between[0].site));
                if let Some((_, b)) = border {
                    far = across_cell(far, b, length);
                }
            }
            let wet = between.iter().any(|id| place(id).is_some_and(|p| p.kind == Kind::Water));
            passage(between, set, [centre(0), far], wet)
        })
        .collect())
}
