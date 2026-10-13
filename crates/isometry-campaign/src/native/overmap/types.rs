// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use super::*;

pub type Color = [u8; 4];

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ViewSettings {
    pub biomes: bool,
    pub factions: bool,
    /// Absent inherits the world's explicitly supplied backdrop availability.
    pub thumbnails: Option<bool>,
    /// An exact retained disclosure entry; absent reads all accepted disclosures.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disclosure: Option<Key>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cell { pub at: [u32; 2], pub site: Id, pub disclosure: Key }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Biome { pub key: Key, pub name: String, pub tags: BTreeSet<Key>, pub color: Color }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Thumbnail { pub size: [u32; 2], pub pixels: Vec<Color> }

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Presentation {
    pub version: u32,
    pub key: Key,
    pub bounds: [u32; 2],
    pub cells: Vec<Cell>,
    /// Explicit display anchors, never native poses or patch coordinates.
    pub anchors: BTreeMap<Id, [u32; 2]>,
    pub biomes: Vec<Biome>,
    pub factions: BTreeMap<Id, Color>,
    pub settings: ViewSettings,
    pub thumbnail: Option<Thumbnail>,
    pub cell_color: Color,
    pub route_color: Color,
    pub route_width_milli: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OvermapInput { pub version: u32, pub presentation: Presentation }

impl Presentation {
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1 || self.key.trim().is_empty() || self.bounds.contains(&0)
            || self.bounds.iter().any(|side| *side > i32::MAX as u32)
            || self.route_width_milli == 0 {
            return Err("overmap presentation needs named explicit bounds and route width".into());
        }
        let mut positions = BTreeSet::new();
        let mut sites = BTreeSet::new();
        for cell in &self.cells {
            if cell.at[0] >= self.bounds[0] || cell.at[1] >= self.bounds[1]
                || !positions.insert(cell.at) || cell.disclosure.trim().is_empty() {
                return Err("overmap cells repeat or exceed their explicit display bounds".into());
            }
            sites.insert(cell.site);
        }
        if self.anchors.keys().copied().collect::<BTreeSet<_>>() != sites
            || self.anchors.iter().any(|(site, at)| !self.cells.iter()
                .any(|cell| cell.site == *site && cell.at == *at)) {
            return Err("every display site needs its exact authored anchor cell".into());
        }
        let mut biomes = BTreeSet::new();
        if self.biomes.iter().any(|biome| biome.key.trim().is_empty() || biome.name.trim().is_empty()
            || biome.tags.is_empty() || !biomes.insert(&biome.key)) {
            return Err("biome readings need distinct named explicit retained-place tag predicates".into());
        }
        if let Some(thumbnail) = &self.thumbnail {
            let pixels = u64::from(thumbnail.size[0]).checked_mul(u64::from(thumbnail.size[1]))
                .and_then(|count| usize::try_from(count).ok()).ok_or("thumbnail dimensions overflow")?;
            if thumbnail.size != self.bounds || thumbnail.pixels.len() != pixels {
                return Err("thumbnail pixels must exactly match the authored display-cell bounds".into());
            }
        }
        Ok(())
    }
}

pub(super) fn disclosure(session: &Session, key: &str) -> Result<isocosm::party::Disclosure, String> {
    let entry = session.entries.iter().find(|entry| entry.id == key)
        .ok_or("overmap source disclosure is absent")?;
    let isocosm::Command::Party(isocosm::party::Change::Disclose { party, .. }) = &entry.command
        else { return Err("overmap source is not an accepted native party disclosure".into()); };
    let map = isocosm::party::shared_map(session, party)?;
    if !map.records.iter().any(|id| id == key) { return Err("overmap disclosure is not retained".into()); }
    let isocosm::party::Outcome::Disclosed(record) = serde_json::from_str(&entry.outcome)
        .map_err(|error| error.to_string())? else { return Err("overmap disclosure outcome is absent".into()); };
    Ok(record)
}
