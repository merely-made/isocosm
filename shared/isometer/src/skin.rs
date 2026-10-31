// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The creature skin: the presentation half of the body adapter
//! (phenotype-contract P5; cases 1035, 1049), shared by every product.
//!
//! A body's look is derived from what it is. Each living part is drawn as the
//! existing voxel grammar's form for its declared extent, never outside it;
//! the form's colour comes from the part's matter, mixed by amount through the
//! rules' colour for each matter key; the part's functions become tissue
//! marks; and the body's seed shifts colour only. The skin is presentation:
//! it returns copies of the documents that differ only in their volume
//! references, and nothing here is read back as geometry or anatomy.
//!
//! The inputs are product-neutral ([`PartSkin`], in `isometer-core`):
//! Cosmograph reads its entities into them (`attention::spatial::skin_parts`).

use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};

pub use isometer_core::PartSkin;
use isometer_core::{BodyDocument, PartId, classify};
use isometer_mesh::{ContentEntry, ContentError, Volume};
use isometer_render::PartMaterial;

use crate::bodies::{PaletteColour, SubjectKey};
use crate::volumes::DeclaredExtentVolumes;

/// The colours a skin draws with. A matter key the rules do not name takes a
/// colour drawn from the key itself, inside `bounds`, so open and generated
/// matter keys still read apart.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SkinRules {
    pub matter: BTreeMap<String, [u8; 3]>,
    /// The lowest and highest value each channel of a drawn colour takes.
    pub bounds: ([u8; 3], [u8; 3]),
    /// A part with no matter.
    pub empty: [u8; 3],
    /// The seeded shift each channel may take, either way.
    pub jitter: u8,
}

impl Default for SkinRules {
    /// Muted, earthy bounds, so an unnamed matter never reads as a signal
    /// colour.
    fn default() -> Self {
        Self {
            matter: BTreeMap::new(),
            bounds: ([70, 60, 50], [200, 185, 160]),
            empty: [128, 120, 110],
            jitter: 10,
        }
    }
}

/// One body to skin.
#[derive(Clone, Copy, Debug)]
pub struct SkinBody<'a> {
    pub subject: SubjectKey,
    pub document: &'a BodyDocument,
    pub parts: &'a [PartSkin],
    /// The individual's seed, for its colour variation.
    pub seed: u64,
}

/// Skinned bodies: presentation copies of the documents, their forms, each
/// body's colour table and its function marks.
#[derive(Clone, Debug, Default)]
pub struct Skin {
    pub documents: BTreeMap<SubjectKey, BodyDocument>,
    pub volumes: DeclaredExtentVolumes,
    pub palettes: BTreeMap<SubjectKey, Vec<PaletteColour>>,
    pub marks: BTreeMap<SubjectKey, Vec<PartMaterial>>,
}

/// Skin every body. A body whose document cannot be drawn is an error, as
/// [`DeclaredExtentVolumes::generated_from_documents`] would refuse it.
pub fn skin<'a>(
    bodies: impl IntoIterator<Item = SkinBody<'a>>,
    rules: &SkinRules,
) -> Result<Skin, ContentError> {
    let mut out = Skin::default();
    for body in bodies {
        let mut document = body.document.clone();
        // Entry 0 is the empty material; each distinct part colour after it.
        let mut palette: Vec<PaletteColour> = vec![[0, 0, 0]];
        let skins: BTreeMap<PartId, &PartSkin> = body.parts.iter().map(|p| (p.part, p)).collect();
        for part in document.parts.iter_mut().filter(|part| !part.severed) {
            let colour = shift(part_colour(skins.get(&part.id).copied(), rules), body.seed, rules.jitter);
            let slot = match palette.iter().position(|held| *held == colour) {
                Some(slot) => slot,
                None => {
                    palette.push(colour);
                    palette.len() - 1
                },
            };
            let slot = u8::try_from(slot).unwrap_or(u8::MAX);
            let volume = form(part.half_extent, slot)?;
            let reference = volume.content_ref();
            out.volumes.insert(reference, volume);
            part.volume = reference;
        }
        out.marks.insert(body.subject, function_marks(body.parts));
        out.palettes.insert(body.subject, palette);
        out.documents.insert(body.subject, document);
    }
    Ok(out)
}

/// The grammar's form for `half`, every solid voxel in material `slot`.
fn form(half: [i32; 3], slot: u8) -> Result<Volume, ContentError> {
    let extent = half.map(|h| (h.max(0) as u32) * 2);
    if extent.contains(&0) {
        return Ok(Volume::empty(extent));
    }
    let mut volume = ContentEntry::generate(classify(half), 0, half)?.volume;
    let [dx, dy, dz] = volume.size;
    for z in 0..dz {
        for y in 0..dy {
            for x in 0..dx {
                if volume.get(x, y, z) != 0 {
                    volume.set(x, y, z, slot);
                }
            }
        }
    }
    Ok(volume)
}

/// A part's colour: its matter's colours mixed by amount.
fn part_colour(part: Option<&PartSkin>, rules: &SkinRules) -> [u8; 3] {
    let Some(part) = part else {
        return rules.empty;
    };
    let total: u128 = part.matter.values().map(|amount| u128::from(*amount)).sum();
    if total == 0 {
        return rules.empty;
    }
    let mut sum = [0u128; 3];
    for (key, amount) in &part.matter {
        let colour = rules.matter.get(key).copied().unwrap_or_else(|| drawn(key, rules));
        for channel in 0..3 {
            sum[channel] += u128::from(colour[channel]) * u128::from(*amount);
        }
    }
    sum.map(|channel| (channel / total) as u8)
}

/// The colour an unnamed matter key draws from itself, inside the bounds.
fn drawn(key: &str, rules: &SkinRules) -> [u8; 3] {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    key.hash(&mut hasher);
    let bits = hasher.finish();
    let (low, high) = rules.bounds;
    [0, 1, 2].map(|channel| {
        let span = u64::from(high[channel].saturating_sub(low[channel])) + 1;
        low[channel] + ((bits >> (channel * 16)) % span) as u8
    })
}

/// `colour` shifted by the seed, each channel within `jitter` either way.
fn shift(colour: [u8; 3], seed: u64, jitter: u8) -> [u8; 3] {
    if jitter == 0 {
        return colour;
    }
    let span = 2 * u64::from(jitter) + 1;
    let mut out = colour;
    for channel in 0..3 {
        let offset = ((seed >> (channel * 8 + 8)) % span) as i16 - i16::from(jitter);
        out[channel] = (i16::from(colour[channel]) + offset).clamp(0, 255) as u8;
    }
    out
}

/// A part's functions as the renderer's five tissue marks, each a share of
/// the part's capacity. Lifted from Mesocosm's own table (case 1049), so
/// every product marks function regions alike.
pub fn function_marks(parts: &[PartSkin]) -> Vec<PartMaterial> {
    let mut out = Vec::new();
    for part in parts {
        if part.capacity == 0 {
            continue;
        }
        let mut counts = [0u64; 5];
        for (function, &cells) in &part.functions {
            let channel = match function.as_str() {
                "function:contract" | "function:support" | "function:grip" | "function:adhesion" => 0,
                "function:intake" | "function:excrete" => 1,
                "function:sense" | "function:conduct" | "function:gate" | "function:circulate" => 2,
                "function:fix" | "function:respire" => 3,
                "function:store" | "function:reproduce" | "function:secrete" => 4,
                _ => continue,
            };
            counts[channel] += u64::from(cells);
        }
        for (channel, count) in counts.into_iter().enumerate().filter(|(_, n)| *n > 0) {
            out.push(PartMaterial {
                part: part.part,
                material: channel as u8,
                fraction: count as f32 / part.capacity as f32,
            });
        }
    }
    out
}

#[cfg(test)]
#[path = "skin_tests.rs"]
mod tests;
