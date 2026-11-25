// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! What a body makes its living by, read off the parts it feeds with
//! (Mesocosm's DC1.5 and DC4, re-expressed by wing ruling 755): a producer
//! holds a fixing plate up in the light, a consumer bears a mouth under its
//! head, and a decomposer is neither, absorbing across its surface. Fixing
//! comes first, so a body with both is read a producer. A reading beside the
//! kingdom an entity asserts, never stored, and named apart from it so
//! *kingdom* keeps one sense, a lineage's playstyle (cases 1219, 1222).
//! Geometry is read through isometer's document (674).

use crate::schema::*;
use isometer_core::{BodyDocument, Role, classify};

/// The three ways of making a living: a body's trophic reading.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
pub enum Trophic {
    Producer,
    Consumer,
    Decomposer,
}

pub mod canopy_light;

const FIX: &str = "function:fix";

/// A part's height: its pivot's `y` in body space, which no yaw moves.
fn height(doc: &BodyDocument, id: PartId) -> i32 {
    doc.world_pivot(id).map_or(0, |at| at[1])
}

/// The plates in a canopy position: hung above what they grow from (a
/// positive `y` offset, no larger lateral), with nothing on the body
/// standing over them (no living part's lowest voxel above the plate's
/// highest).
pub fn canopy_parts(doc: &BodyDocument) -> Vec<PartId> {
    canopy(doc).collect()
}

fn canopy(doc: &BodyDocument) -> impl Iterator<Item = PartId> + '_ {
    let overhead = doc
        .living()
        .map(|p| height(doc, p.id) - p.half_extent[1].abs())
        .max()
        .unwrap_or(0);
    doc.living()
        .filter(move |p| {
            let Some(at) = p.attachment else { return false };
            let top = height(doc, p.id) + p.half_extent[1].abs();
            classify(p.half_extent) == Role::Plate
                && at.offset[1] > 0
                && at.offset[1] >= at.offset[0].abs()
                && top >= overhead
        })
        .map(|p| p.id)
}

/// The mouth: a living part borne under the head (attached to the root,
/// hung below it), a limb-classed jaw before a mass-classed bulk mouth.
pub fn mouth_part(doc: &BodyDocument) -> Option<PartId> {
    let mut found = None;
    for p in doc.living() {
        let under = p
            .attachment
            .is_some_and(|at| at.parent == doc.root && at.offset[1] < 0);
        match (under, classify(p.half_extent)) {
            (true, Role::Limb) => return Some(p.id),
            (true, Role::Mass) => found = Some(p.id),
            _ => {},
        }
    }
    found
}

/// First exposed plate with live fixing cells, in document order. Read from
/// the current body so severing, regrowth and changes to tissue take effect.
pub fn fixing_plate(e: &Entity) -> Option<PartId> {
    canopy(e.body.as_ref()?).find(|id|
        e.parts.get(id).is_some_and(|p| p.cells.get(FIX).copied().unwrap_or(0) > 0))
}

/// (Exposed fixing cells, cells backed by local plant biomass). A unit of
/// tissue can support one cell in this rate law. A bite affects the part it took.
pub fn fixing_tissue(e: &Entity, tissue: &str) -> (u64, u64) {
    let Some(doc) = &e.body else { return (0, 0) };
    canopy(doc).filter_map(|id| e.parts.get(&id)).fold((0, 0), |(exposed, active), p| {
        let cells = u64::from(p.cells.get(FIX).copied().unwrap_or(0));
        (exposed + cells, active + cells.min(p.matter.get(tissue).copied().unwrap_or(0)))
    })
}

/// Integer pulse density. At constant exposure k of B, exactly k of every B
/// scheduled opportunities are admitted, each still paying full light and soil.
pub fn fixes_this_turn(active: u64, budget: u32, turn: u64) -> bool {
    let budget = u64::from(budget);
    if budget == 0 { return false; }
    let phase = turn % budget;
    let units = active.min(budget);
    (phase + 1) * units / budget != phase * units / budget
}

/// The trophic reading `e`'s body makes; none for a body without geometry. A
/// canopy plate counts only where its cells fix.
pub fn of(e: &Entity) -> Option<Trophic> {
    let doc = e.body.as_ref()?;
    Some(if fixing_plate(e).is_some() {
        Trophic::Producer
    } else if mouth_part(doc).is_some_and(|id| e.lives(id)) {
        Trophic::Consumer
    } else {
        Trophic::Decomposer
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::{Body, Sketch};

    fn sketch(parent: Option<u32>, half: [i32; 3], offset: [i32; 3], fix: u32) -> Sketch {
        let mut part = Part::default();
        if fix > 0 {
            part.cells.insert(FIX.into(), fix);
            part.functions.insert(FIX.into());
        }
        Sketch {
            parent,
            half_extent: half,
            offset,
            part,
            ..Default::default()
        }
    }

    fn read(parts: Vec<Sketch>) -> Option<Trophic> {
        let mut e = crate::probe::BodyFounding::default()
            .generate()
            .unwrap()
            .genesis
            .population
            .groups
            .into_values()
            .next()
            .unwrap()
            .entity.as_ref().clone();
        let body = Body::sketch(parts.into_iter().enumerate().map(|(i, s)| (i as u32, s)));
        e.embody(body);
        of(&e)
    }

    #[test]
    fn a_lit_fixing_plate_makes_a_producer_and_a_mouth_a_consumer() {
        let root = || sketch(None, [2, 2, 2], [0; 3], 0);
        let frond = sketch(Some(0), [4, 1, 4], [0, 3, 0], 4);
        let mouth = sketch(Some(0), [2, 1, 1], [0, -3, 0], 0);
        assert_eq!(read(vec![root(), frond]), Some(Trophic::Producer));
        assert_eq!(read(vec![root(), mouth]), Some(Trophic::Consumer));
        assert_eq!(read(vec![root()]), Some(Trophic::Decomposer));
        // A plate on the flank covers rather than fixes.
        let shell = sketch(Some(0), [1, 4, 4], [3, 1, 0], 4);
        assert_eq!(read(vec![root(), shell]), Some(Trophic::Decomposer));
    }
}
