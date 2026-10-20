// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A generated world's declared spawn and arrival templates (ruling 889).
//! Stand cells are derived once from the founding terrain; a game binds them to
//! actual bodies and journeys. Nothing here moves a body or invents a pose later.
//! Movement stays template-driven: this declares no step, terrain or move process.

use super::*;
use crate::{simulation::Genesis, terrain::View};
use isometer_space::{places::{Places, Rules as PlaceRules}, volume::Volume};

pub const VERSION: u32 = 1;
/// Stands declared per site, in the order spawning members take them.
pub const STANDS: usize = 4;
/// Columns kept between stands, and the lattice and reach the search takes.
const APART: i64 = 4;
const STRIDE: i64 = 2;
const REACH: i64 = 12;
/// Half the window lifted around a site's centre, in base columns, and the
/// rock held under it: the same bounded window Mesocosm's placing lifts.
const WINDOW_HALF: i64 = 16;
const DEPTH: i64 = 8;
/// Root pivot to eye, in cells.
const EYE: [i64; 3] = [0, 1, 0];

/// A root pivot standing on solid ground, and its facing, in the site's base cells.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stand {
    pub at: [i64; 3], pub yaw: Yaw,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor: Option<Id>,
}

/// The stand a member takes on arriving by this route.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Crossing { pub from: Id, pub to: Id, pub stand: Stand }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Templates {
    pub version: u32,
    pub window: Window,
    pub places: isometer_space::places::Rules,
    /// Root pivot to eye, for every stand and arrival.
    pub eye: [i64; 3],
    pub stands: BTreeMap<Id, Vec<Stand>>,
    pub crossings: Vec<Crossing>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub foraging: BTreeMap<Key, super::foraging::Foraging>,
}

impl Templates {
    pub fn validate(&self, genesis: &Genesis) -> Result<()> {
        if !matches!(self.version, VERSION | 2 | 3) || self.window.max[0] <= self.window.min[0]
            || self.window.max[1] <= self.window.min[1] || self.window.depth <= 0 {
            return Err("unsupported or empty spatial templates".into());
        }
        let inside = |stand: &Stand| (0..2).all(|k| stand.at[2 * k] >= self.window.min[k] && stand.at[2 * k] < self.window.max[k]);
        if self.stands.iter().any(|(site, stands)| !genesis.sites.contains_key(site)
            || stands.len() > STANDS || !stands.iter().all(inside)) {
            return Err("spatial template stands name an absent site or leave the window".into());
        }
        let mut actors = BTreeSet::new();
        for (&site, stands) in &self.stands { for stand in stands {
            if self.version == 1 && stand.actor.is_some() { return Err("legacy stand has an actor binding".into()); }
            if self.version >= 2 && stand.actor.is_none_or(|id| !actors.insert(id)
                || genesis.population.get(id).is_none_or(|e| e.body.is_none() || e.resident_site() != Some(site))) {
                return Err("whole-body stand has no unique native resident".into());
            }
        } }
        super::foraging::validate(self, &genesis.rules)?;
        for crossing in &self.crossings {
            let routed = genesis.sites.get(&crossing.from)
                .is_some_and(|site| site.routes.iter().any(|r| r.to == crossing.to));
            if !routed || self.stands.get(&crossing.to).is_none_or(|stands| !stands.contains(&crossing.stand)) {
                return Err("spatial template arrival names no native route or stand".into());
            }
        }
        Ok(())
    }

    /// Derives the world's templates from its realized terrain and declares
    /// them in its rules. A world without a map and bodies has no realized
    /// terrain or physical bodies, and declares none.
    pub fn declare(genesis: &mut Genesis) -> Result<()> {
        let Some(founding) = genesis.founding.as_ref() else { return Ok(()); };
        if founding.map.is_none() || founding.bodies.is_none() { return Ok(()); }
        let seed = genesis.seed;
        let places = PlaceRules::default();
        let view = View::of(genesis)?;
        let side = view.footprint.side as i64;
        let (mid, half) = (side / 2, WINDOW_HALF.min(side / 2).max(1));
        let window = Window { min: [(mid - half).max(0); 2], max: [(mid + half).min(side); 2], depth: DEPTH };
        let mut stands = BTreeMap::new();
        for &site in genesis.sites.keys() {
            let volume = Volume::lift(&view, site, window.min, window.max, window.depth, &[])?;
            stands.insert(site, stands_of(&volume, places, seed, site)?);
        }
        let crossings = genesis.sites.iter().flat_map(|(&from, site)| site.routes.iter()
            .filter_map(|r| stands.get(&r.to).and_then(|s| s.first())
                .map(|stand| Crossing { from, to: r.to, stand: stand.clone() }))
            .collect::<Vec<_>>()).collect();
        genesis.rules.spatial_templates = Some(Templates { version: VERSION, window, places, eye: EYE,
            stands, crossings, foraging: BTreeMap::new() });
        Ok(())
    }
}

/// Root cells on solid ground whose root and eye are clear in one realized
/// patch, nearest the window centre first and spaced apart. A site whose
/// terrain admits none declares none.
fn stands_of(volume: &Volume, rules: PlaceRules, seed: u64, site: Id) -> Result<Vec<Stand>> {
    let m = volume.materials;
    let clear = |cell: [i64; 3]| volume.holds(cell[0], cell[2]) && cell[1] >= volume.datum
        && cell[1] < volume.ceiling() && volume.material(cell).is_some_and(|c| c == m.air);
    // The first non-air cell down a column decides it: solid ground carries a
    // root above it, anything else (water) carries none.
    let root_in = |x, z| -> Option<[i64; 3]> {
        let top = (volume.datum..volume.ceiling()).rev().find(|&y| volume.material([x, y, z]) != Some(m.air))?;
        let ground = volume.material([x, top, z]).filter(|&c| c != m.water)?;
        (ground != m.air && top + 1 + EYE[1] < volume.ceiling()).then_some([x, top + 1, z])
    };
    let mid = (volume.min[0] + volume.max[0]) / 2;
    let mut places: Option<Places> = None;
    let mut found: Vec<Stand> = Vec::new();
    for ring in (0..=REACH.min((volume.max[0] - volume.min[0]) / 2)).step_by(STRIDE as usize) {
        for x in (mid - ring..=mid + ring).step_by(STRIDE as usize) {
            for z in (mid - ring..=mid + ring).step_by(STRIDE as usize) {
                if found.len() >= STANDS { return Ok(found); }
                if (x - mid).abs().max((z - mid).abs()) != ring || !volume.holds(x, z)
                    || found.iter().any(|s| (s.at[0] - x).abs().max((s.at[2] - z).abs()) < APART) { continue; }
                let Some(root) = root_in(x, z) else { continue; };
                let eye = [root[0] + EYE[0], root[1] + EYE[1], root[2] + EYE[2]];
                if !clear(root) || !clear(eye) { continue; }
                // Places are derived only for a site that has ground to stand on.
                if places.is_none() { places = Some(Places::derive(volume, rules)?); }
                let patch = places.as_ref().and_then(|p| p.place_at(root));
                if patch.is_some() && patch == places.as_ref().and_then(|p| p.place_at(eye)) {
                    let quarters = crate::draw(seed, "stand-yaw", &[site, (x as u64) << 20 ^ z as u64]) % 4;
                    found.push(Stand { at: root, yaw: (0..quarters).fold(Yaw::Zero, |y, _| y.compose(Yaw::Quarter)), actor: None });
                }
            }
        }
    }
    Ok(found)
}
