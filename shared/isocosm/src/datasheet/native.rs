// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The sheets lowered onto native kinds and recipes (wing ruling 758).
//! A palette's shapes become kinds, `kind:<name>`, every cell given to its
//! bank's one function (511): a mass bank's to intake, a limb's to
//! contract, a plate's to fix, a sensor's to sense. A sheet tagma becomes a
//! native tagma: its segment a mass shape, its appendage a borne kind at
//! Mesocosm's socket, a mouth below, a held plate above, a worn plate, a
//! limb, a vane or a feeler on the flank, mirrored where the plan is
//! bilateral. *Reading:* a shape of no extent, which no native part can
//! hold cells in, lowers to its bank's default, as the eye stands in for a
//! speck.

use std::collections::BTreeMap;

use super::schema::{AnchorWord, AppendageWord, FacingWord, MouthWord, TagmaEntry, WornWord};
use super::{Bank, Banks, Shape, Sheets, sheets};
use crate::anatomy;
use crate::rules::{Anchor, Facing, Recipe, Tagma, Template};
use crate::schema::Key;

/// Mesocosm's odds that a borne kind develops absent: one in twelve.
const ABSENCE: [u32; 2] = [1, 12];

fn function(bank: Bank) -> &'static str {
    match bank {
        Bank::Mass => "function:intake",
        Bank::Limb => "function:contract",
        Bank::Plate => "function:fix",
        Bank::Sensor => "function:sense",
    }
}

fn kind(name: &str) -> Key {
    format!("kind:{name}")
}

/// The palette's shapes as kinds, those of no extent left out.
pub fn kinds_of(banks: &Banks) -> BTreeMap<Key, Template> {
    let mut kinds = BTreeMap::new();
    for bank in Bank::ALL {
        for s in banks.bank(bank).iter().filter(|s| s.half_extent != [0; 3]) {
            let cells =
                BTreeMap::from([(function(bank).to_string(), anatomy::capacity(s.half_extent))]);
            let template = Template {
                half_extent: s.half_extent,
                cells,
                shape: String::new(),
            };
            kinds.insert(kind(&s.name), template);
        }
    }
    kinds
}

/// The named palette's kinds.
pub fn kinds(palette: &str) -> Result<BTreeMap<Key, Template>, String> {
    Ok(kinds_of(&sheets().banks(palette)?))
}

/// The shape a tagma names in `bank`, its bank's default standing in for
/// one of no extent.
fn borne<'a>(banks: &'a Banks, bank: Bank, name: Option<&str>) -> Result<&'a Shape, String> {
    let shape = banks.shape(bank, name)?;
    match shape.half_extent == [0; 3] {
        true => banks.shape(bank, None),
        false => Ok(shape),
    }
}

fn facing(word: FacingWord) -> Facing {
    match word {
        FacingWord::Front => Facing::Front,
        FacingWord::Back => Facing::Back,
        FacingWord::Left => Facing::Left,
        FacingWord::Right => Facing::Right,
        FacingWord::Above => Facing::Above,
        FacingWord::Below => Facing::Below,
    }
}

fn anchor(word: AnchorWord) -> Anchor {
    match word {
        AnchorWord::Base => Anchor::Base,
        AnchorWord::Middle => Anchor::Middle,
        AnchorWord::Tip => Anchor::Tip,
    }
}

fn tagma(entry: &TagmaEntry, banks: &Banks) -> Result<Tagma, String> {
    if entry.chain.is_some() {
        return Err("native recipes carry no appendage chain".into());
    }
    let appendage = entry.appendage.unwrap_or(AppendageWord::None);
    let shape = entry.shape.as_deref();
    let bears = match appendage {
        AppendageWord::None => None,
        AppendageWord::Mouth => Some(match entry.mouth.unwrap_or(MouthWord::Bulk) {
            MouthWord::Bulk => (borne(banks, Bank::Mass, shape)?, Facing::Below),
            MouthWord::Jaw => (borne(banks, Bank::Limb, shape)?, Facing::Below),
        }),
        AppendageWord::Plate => {
            let socket = match entry.worn {
                Some(WornWord::Covering) => Facing::Right,
                _ => Facing::Above,
            };
            Some((borne(banks, Bank::Plate, shape)?, socket))
        },
        AppendageWord::Limb | AppendageWord::Vane => {
            Some((borne(banks, Bank::Limb, shape)?, Facing::Right))
        },
        AppendageWord::Feeler => Some((borne(banks, Bank::Sensor, shape)?, Facing::Right)),
    };
    let segment = banks.shape(Bank::Mass, entry.segment.as_deref())?;
    let layout = entry.layout.as_ref();
    Ok(Tagma {
        segments: entry.segments,
        segment: kind(&segment.name),
        bears: bears.map(|(s, _)| kind(&s.name)),
        per_segment: match bears {
            Some(_) => entry.per_segment.unwrap_or(1),
            None => 0,
        },
        parent: layout.and_then(|l| l.parent),
        anchor: layout.map_or(Anchor::Tip, |l| anchor(l.anchor)),
        facing: layout.map_or(Facing::Back, |l| facing(l.facing)),
        socket: bears.map_or(Facing::Right, |(_, socket)| socket),
        variance: layout.and_then(|l| l.variance),
    })
}

/// One body, `<sheet>.<body>`, as a native recipe in the named palette.
pub fn recipe(sheets: &Sheets, body: &str, banks: &Banks) -> Result<Recipe, String> {
    let (_, entry) = sheets.body(body)?;
    let tagmata = entry
        .tagma
        .iter()
        .enumerate()
        .map(|(i, t)| tagma(t, banks).map_err(|why| format!("{body}, tagma {i}: {why}")))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Recipe {
        tagmata,
        variance: entry.variance,
        absence: ABSENCE,
        riff: [0, 1],
        vary: [0, 1],
    })
}

/// A founding's authored bodies, by name, each with whether it is a
/// producer and its native recipe, producers first, then consumers and
/// decomposers, in the sheet's order.
pub fn founding(name: &str) -> Result<Vec<(String, bool, Recipe)>, String> {
    let sheets = sheets();
    let entry = sheets.founding(name)?;
    let banks = sheets.banks(&entry.palette)?;
    let tiers = [
        (&entry.producer, true),
        (&entry.consumer, false),
        (&entry.decomposer, false),
    ];
    let mut out = vec![];
    for (bodies, producer) in tiers {
        for body in bodies {
            out.push((body.clone(), producer, recipe(sheets, body, &banks)?));
        }
    }
    Ok(out)
}

/// The founding the sheets ship by default.
pub fn default_founding() -> &'static str {
    sheets().foundings.default.as_str()
}
