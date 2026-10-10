// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Places over the bricks (the place-graph plan's SP5, rulings 417 to 422):
//! a volume's base cells divided by how things move. Rooms are air cut off
//! from the sky; water bodies are places of their own; the outdoors floods
//! over walkable surface, splits where a step passes the world's climb, and
//! a patch wider than the world's cap is cut on the cap's grid. Every
//! passage records the clearances a body must fit (ruling 418), so one graph
//! serves every body. Derived from the volume and never stored: an edit
//! re-derives the places its dirty region reaches ([`Places::rederive`]).

mod cells;
mod flood;
mod join;
#[cfg(test)]
mod join_tests;
mod local;
mod neck;
mod passages;
mod route;
mod sight;
#[cfg(test)]
mod tests;

pub use join::join;
pub use route::{Body, route_over};
pub use sight::ray;

/// The stances a walker at stance `at` can step to across one column under
/// `rules`' climb, each with its rise; a drop of any depth within headroom
/// is a step down (ruling 702). What a game's walker reads between places.
pub fn steps(volume: &Volume, rules: Rules, at: [i64; 3]) -> Vec<([i64; 3], i64)> {
    let cells = Cells::new(volume, rules);
    let climbs = |rise: i64| rise <= 0 || cells.climbable(rise);
    cells.steps(at).into_iter().filter(|(_, rise)| climbs(*rise)).collect()
}

use crate::volume::Volume;
use crate::{Result, SiteId};
use cells::Cells;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A rise a walker takes per unit of run (ruling 419).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Climb {
    pub rise: u32,
    pub run: u32,
}

/// The largest outdoor patch before it is cut, x by z base units (ruling
/// 420): the stepped presets, or a founder's own.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Cap {
    ExtraSmall,
    Small,
    Medium,
    ExtraMedium,
    Large,
    ExtraLarge,
    Own([u32; 2]),
}

impl Cap {
    pub fn sides(self) -> [u32; 2] {
        match self {
            Cap::ExtraSmall => [16; 2],
            Cap::Small => [32; 2],
            Cap::Medium => [64; 2],
            Cap::ExtraMedium => [128; 2],
            Cap::Large => [256; 2],
            Cap::ExtraLarge => [512; 2],
            Cap::Own(sides) => sides.map(|s| s.max(1)),
        }
    }
}

/// Which air is cut off from the sky (ruling 417). A room is roofed air
/// (ruling 737), so an open-mouthed cave is a room; `Sealed` stays as the
/// literal reading, for comparison.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Cover {
    /// Air with no path through air to the open sky.
    Sealed,
    /// Air with something solid anywhere above it.
    #[default]
    Roofed,
}

/// The world rules places are derived under.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rules {
    pub climb: Climb,
    pub cap: Cap,
    pub cover: Cover,
    /// The walkable width, in base units, below which a patch splits at a
    /// neck (ruling 738), 2 by default (743); unset, no patch splits.
    pub neck: Option<u32>,
}

impl Rules {
    /// The ruled defaults, one up per one across (419), the large cap
    /// (421) and a neck of 2 (743), under the named cover.
    pub fn ruled(cover: Cover) -> Self {
        Self {
            climb: Climb { rise: 1, run: 1 },
            cap: Cap::Large,
            cover,
            neck: Some(2),
        }
    }
}

impl Default for Rules {
    /// The ruled defaults with rooms roofed (ruling 737).
    fn default() -> Self {
        Self::ruled(Cover::default())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Kind {
    Patch,
    Room,
    Water,
}

/// A place's identity: its site and its least base cell, so a place keeps
/// its id through re-derivation while it holds that cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PlaceId {
    pub site: SiteId,
    pub cell: [i64; 3],
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Place {
    pub id: PlaceId,
    pub kind: Kind,
    /// Its cells: walkable cells for a patch, air for a room, water for water.
    pub cells: u64,
    /// The mean of its cells, rounded down: where travel cost is measured.
    pub centre: [i64; 3],
    /// The least cell of the walkable component a patch was cut from.
    pub component: [i64; 3],
    /// The least and greatest coordinate its cells reach on each axis.
    pub bounds: [[i64; 3]; 2],
}

/// What a body must fit to cross: the widest and tallest it may be and the
/// step it must manage (ruling 418), with one crossing's two cells.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Clearance {
    pub width: u32,
    pub height: u32,
    pub step: u32,
    /// A crossing's cell in the lesser place and its cell in the greater.
    pub cells: [[i64; 3]; 2],
}

/// An edge between two places, the lesser first.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Passage {
    pub between: [PlaceId; 2],
    /// Whether it enters water: a body wades or swims it.
    pub wet: bool,
    /// None dominated by another.
    pub clearances: Vec<Clearance>,
    /// Base units between the places' centres, travelled.
    pub cost: u64,
}

/// A volume's places and the passages between them.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Places {
    pub site: SiteId,
    pub rules: Rules,
    pub places: BTreeMap<PlaceId, Place>,
    pub passages: BTreeMap<[PlaceId; 2], Passage>,
    /// Which place each labelled cell belongs to, keyed `[x, z, y]` so a
    /// column's cells sit together.
    labels: BTreeMap<[i64; 3], PlaceId>,
}

impl Places {
    /// Every place of `volume`, derived whole.
    pub fn derive(volume: &Volume, rules: Rules) -> Result<Self> {
        check(rules)?;
        let cells = Cells::new(volume, rules);
        let mut places = Self {
            site: volume.site,
            rules,
            places: BTreeMap::new(),
            passages: BTreeMap::new(),
            labels: BTreeMap::new(),
        };
        let found = flood::components(&cells, cells.everywhere());
        places.insert(&cells, found);
        Ok(places)
    }

    /// The place holding `cell`: its patch, room or water, or the patch
    /// under it for open air above the ground.
    pub fn place_at(&self, [x, y, z]: [i64; 3]) -> Option<PlaceId> {
        let below = self.labels.range([x, z, i64::MIN]..=[x, z, y]).next_back()?;
        Some(*below.1)
    }

    /// The place labelling exactly `cell`.
    pub(super) fn label(&self, [x, y, z]: [i64; 3]) -> Option<PlaceId> {
        self.labels.get(&[x, z, y]).copied()
    }

    /// The passages touching `id`.
    pub fn passages_of(&self, id: PlaceId) -> impl Iterator<Item = &Passage> {
        self.passages.values().filter(move |p| p.between.contains(&id))
    }

    fn insert(&mut self, cells: &Cells<'_>, found: Vec<flood::Found>) {
        for f in &found {
            for &[x, y, z] in &f.cells {
                self.labels.insert([x, z, y], f.place.id);
            }
            self.places.insert(f.place.id, f.place.clone());
        }
        for passage in passages::of(cells, self, &found) {
            self.passages.insert(passage.between, passage);
        }
    }
}

fn check(rules: Rules) -> Result<()> {
    if rules.climb.run == 0 {
        return Err("a climb with no run".into());
    }
    Ok(())
}
