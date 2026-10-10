// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The founding datasheets, read into palettes and recipes.
//!
//! Wing design record rulings 625 to 632. The sheets live in
//! `datasheets/foundings/` and compile in: one per roster set (`base`,
//! `branching`, `jointed`, `spaced`) holding palettes, chains and bodies, and
//! `foundings.toml` naming each founding's palette and per-tier bodies. The
//! reasons for every shape and body are comments in the sheets.
//!
//! Sheets name shapes; this module computes the selectors. A tagma's
//! `segment` and `shape` name slots in the palette a body is resolved in
//! (for a founding, the palette it admits), `mouth = "jaw"` adds
//! [`JAW_SHAPE`], and `worn = "covering"` adds [`ARMOUR_SHAPE`]. A name that
//! palette does not admit is refused.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use crate::datasheet::schema::*;
use crate::datasheet::{Bank, Banks, Sheets};
pub(crate) use crate::datasheet::{FOUNDINGS, SETS};
use crate::legacy::mesocosm::axis::{
    ARMOUR_SHAPE, Anchor, Appendage, AppendageStep, ChainFacing, JAW_SHAPE, Recipe, Stretch, Tagma,
};
use crate::legacy::mesocosm::body::VolumeRef;
use crate::legacy::mesocosm::development::{PartPalette, PartTemplate, RoleShapes};
use crate::legacy::mesocosm::organism::Kingdom;
use crate::legacy::mesocosm::plan::{Facing, Role};

/// One founding, resolved: what a world founded this way admits, and the
/// authored bodies each tier installs, one lineage each, in founding order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FoundingSheet {
    pub palette: PartPalette,
    pub producer: Vec<Recipe>,
    pub consumer: Vec<Recipe>,
    pub decomposer: Vec<Recipe>,
}

impl FoundingSheet {
    /// A tier's authored bodies. Empty means the tier still draws.
    pub fn tier(&self, kingdom: Kingdom) -> &[Recipe] {
        match kingdom {
            Kingdom::Producer => &self.producer,
            Kingdom::Consumer => &self.consumer,
            Kingdom::Decomposer => &self.decomposer,
        }
    }
}

/// Every founding the sheets declare, every palette they define, and the
/// bodies, for resolving one by name.
pub struct Foundings {
    /// The founding a world takes when it names none.
    pub default: String,
    foundings: BTreeMap<String, FoundingSheet>,
    palettes: BTreeMap<String, NamedPalette>,
    sheets: Sheets,
}

impl Foundings {
    pub fn get(&self, name: &str) -> Option<&FoundingSheet> {
        self.foundings.get(name)
    }

    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.foundings.keys().map(String::as_str)
    }

    pub fn palette(&self, name: &str) -> Result<PartPalette, String> {
        self.palettes
            .get(name)
            .ok_or_else(|| format!("no palette {name:?}"))?
            .admitted()
    }

    /// One body, `<sheet>.<body>`, resolved in the named palette.
    pub fn body(&self, body: &str, palette: &str) -> Result<Recipe, String> {
        let named = self
            .palettes
            .get(palette)
            .ok_or_else(|| format!("no palette {palette:?}"))?;
        recipe(&self.sheets, body, named)
    }
}

/// The embedded sheets, parsed on first use. Tests prove they load, so a
/// failure here is a broken build rather than a runtime condition.
pub fn foundings() -> &'static Foundings {
    static SHEETS: OnceLock<Foundings> = OnceLock::new();
    SHEETS.get_or_init(|| load(&SETS, FOUNDINGS.1).unwrap_or_else(|why| panic!("{why}")))
}

/// An embedded palette, by name.
pub fn palette(name: &str) -> PartPalette {
    foundings()
        .palette(name)
        .unwrap_or_else(|why| panic!("{why}"))
}

/// An embedded body, `<sheet>.<body>`, resolved in the named palette.
pub fn body(body: &str, palette: &str) -> Recipe {
    foundings()
        .body(body, palette)
        .unwrap_or_else(|why| panic!("{why}"))
}

/// Reads roster-set sheets, given as `(name, text)`, and a foundings sheet,
/// through the native parse (758), lowering them onto legacy recipes.
pub fn load(sets: &[(&str, &str)], foundings: &str) -> Result<Foundings, String> {
    let sheets = Sheets::parse(sets, foundings)?;
    let mut named = BTreeMap::new();
    for name in sheets.palette_names() {
        named.insert(name.to_string(), NamedPalette::of(&sheets.banks(name)?));
    }
    let mut resolved = BTreeMap::new();
    for (name, founding) in &sheets.foundings.founding {
        let palette = named
            .get(&founding.palette)
            .ok_or_else(|| format!("{name}: no palette {:?}", founding.palette))?;
        let tier = |bodies: &[String]| -> Result<Vec<Recipe>, String> {
            bodies
                .iter()
                .map(|body| recipe(&sheets, body, palette).map_err(|why| format!("{name}: {why}")))
                .collect()
        };
        resolved.insert(
            name.clone(),
            FoundingSheet {
                palette: palette.admitted()?,
                producer: tier(&founding.producer)?,
                consumer: tier(&founding.consumer)?,
                decomposer: tier(&founding.decomposer)?,
            },
        );
    }
    Ok(Foundings {
        default: sheets.foundings.default.clone(),
        foundings: resolved,
        palettes: named,
        sheets,
    })
}

/// One role's admitted shapes, by name, in selector order.
type Shapes = Vec<(String, PartTemplate)>;

/// A palette whose slots keep their names, so bodies can be resolved in it.
#[derive(Clone)]
struct NamedPalette {
    banks: [Shapes; 4],
}

impl NamedPalette {
    /// The native resolution's banks, as legacy templates.
    fn of(banks: &Banks) -> Self {
        let mut named = NamedPalette {
            banks: Default::default(),
        };
        for (role, bank) in [
            (Role::Mass, Bank::Mass),
            (Role::Limb, Bank::Limb),
            (Role::Plate, Bank::Plate),
            (Role::Sensor, Bank::Sensor),
        ] {
            named.banks[role_index(role)] = banks
                .bank(bank)
                .iter()
                .map(|s| {
                    let template = PartTemplate {
                        volume: VolumeRef::from_tag(s.tag),
                        half_extent: s.half_extent,
                    };
                    (s.name.clone(), template)
                })
                .collect();
        }
        named
    }

    fn bank(&self, role: Role) -> &Shapes {
        &self.banks[role_index(role)]
    }

    fn slot(&self, role: Role, name: Option<&str>) -> Result<u8, String> {
        let Some(name) = name else { return Ok(0) };
        self.bank(role)
            .iter()
            .position(|(slot, _)| slot == name)
            .map(|index| index as u8)
            .ok_or_else(|| format!("the palette admits no {role:?} shape {name:?}"))
    }

    fn admitted(&self) -> Result<PartPalette, String> {
        let bank = |role: Role| -> Result<RoleShapes, String> {
            let shapes = self.bank(role);
            let (_, default) = shapes
                .first()
                .ok_or_else(|| format!("the {role:?} bank has no default"))?;
            let mut admitted = RoleShapes::only(*default);
            for (index, (_, template)) in shapes.iter().enumerate().skip(1) {
                admitted.extra[index - 1] = Some(*template);
            }
            Ok(admitted)
        };
        Ok(PartPalette {
            mass: bank(Role::Mass)?,
            limb: bank(Role::Limb)?,
            plate: bank(Role::Plate)?,
            sensor: bank(Role::Sensor)?,
        })
    }
}

fn role_index(role: Role) -> usize {
    match role {
        Role::Mass => 0,
        Role::Limb => 1,
        Role::Plate => 2,
        Role::Sensor => 3,
    }
}

/// Resolves `<sheet>.<body>` in the given palette.
fn recipe(sheets: &Sheets, body: &str, palette: &NamedPalette) -> Result<Recipe, String> {
    let (sheet_name, entry) = sheets.body(body)?;
    let fail = |index: usize, why: String| format!("{body}, tagma {index}: {why}");

    let mut tagmata = Vec::with_capacity(entry.tagma.len());
    let mut layout = Vec::new();
    let mut chains = Vec::new();
    for (index, tagma) in entry.tagma.iter().enumerate() {
        tagmata.push(self::tagma(tagma, palette).map_err(|why| fail(index, why))?);
        if let Some(stretch) = &tagma.layout {
            layout.push(Stretch {
                parent: stretch.parent,
                anchor: anchor(stretch.anchor),
                facing: facing(stretch.facing),
                variance: stretch.variance,
            });
        }
        let steps = match &tagma.chain {
            Some(chain) => {
                self::chain(sheets, sheet_name, chain, palette).map_err(|why| fail(index, why))?
            },
            None => Vec::new(),
        };
        chains.push(steps);
    }
    if !layout.is_empty() && layout.len() != tagmata.len() {
        return Err(format!("{body}: a layout places every tagma or none"));
    }
    if chains.iter().all(Vec::is_empty) {
        chains.clear();
    }
    let mut recipe = Recipe::of(tagmata)
        .with_layout(layout)
        .with_appendage_chains(chains);
    recipe.variance = entry.variance;
    Ok(recipe)
}

fn tagma(entry: &TagmaEntry, palette: &NamedPalette) -> Result<Tagma, String> {
    let appendage = entry.appendage.map_or(Appendage::None, appendage);
    let shape = entry.shape.as_deref();
    if entry.mouth.is_some() && appendage != Appendage::Mouth {
        return Err("only a mouth is a jaw or bulk".into());
    }
    if entry.worn.is_some() && appendage != Appendage::Plate {
        return Err("only a plate is worn".into());
    }
    if appendage == Appendage::None
        && (shape.is_some() || entry.per_segment.is_some() || entry.chain.is_some())
    {
        return Err("a bare tagma grows no appendage shape, count or chain".into());
    }
    let appendage_shape = match appendage {
        Appendage::None => 0,
        Appendage::Mouth => match entry.mouth.unwrap_or(MouthWord::Bulk) {
            MouthWord::Bulk => palette.slot(Role::Mass, shape)?,
            MouthWord::Jaw => JAW_SHAPE + palette.slot(Role::Limb, shape)?,
        },
        Appendage::Plate => {
            let covering = entry.worn == Some(WornWord::Covering);
            palette.slot(Role::Plate, shape)? + if covering { ARMOUR_SHAPE } else { 0 }
        },
        Appendage::Limb | Appendage::Vane => palette.slot(Role::Limb, shape)?,
        Appendage::Feeler => palette.slot(Role::Sensor, shape)?,
    };
    let per_segment = match appendage {
        Appendage::None => 0,
        _ => entry.per_segment.unwrap_or(1),
    };
    Ok(Tagma {
        segments: entry.segments,
        appendage,
        per_segment,
        segment_shape: palette.slot(Role::Mass, entry.segment.as_deref())?,
        appendage_shape,
    })
}

/// A chain is named within its body's sheet, or as `<sheet>.<chain>`.
fn chain(
    sheets: &Sheets,
    home: &str,
    name: &str,
    palette: &NamedPalette,
) -> Result<Vec<AppendageStep>, String> {
    let (sheet_name, chain_name) = name.split_once('.').unwrap_or((home, name));
    let steps = sheets
        .sets
        .get(sheet_name)
        .and_then(|sheet| sheet.chain.get(chain_name))
        .ok_or_else(|| format!("no chain {name:?}"))?;
    steps
        .iter()
        .map(|step| {
            let role = role(step.role);
            Ok(AppendageStep {
                role,
                shape: palette.slot(role, Some(&step.shape))?,
                facing: chain_facing(step.facing),
                distal: step.distal,
            })
        })
        .collect()
}

/// The sheets' words as legacy types.
fn appendage(word: AppendageWord) -> Appendage {
    match word {
        AppendageWord::None => Appendage::None,
        AppendageWord::Limb => Appendage::Limb,
        AppendageWord::Feeler => Appendage::Feeler,
        AppendageWord::Plate => Appendage::Plate,
        AppendageWord::Mouth => Appendage::Mouth,
        AppendageWord::Vane => Appendage::Vane,
    }
}

fn role(word: RoleWord) -> Role {
    match word {
        RoleWord::Mass => Role::Mass,
        RoleWord::Limb => Role::Limb,
        RoleWord::Plate => Role::Plate,
        RoleWord::Sensor => Role::Sensor,
    }
}

fn anchor(word: AnchorWord) -> Anchor {
    match word {
        AnchorWord::Base => Anchor::Base,
        AnchorWord::Middle => Anchor::Middle,
        AnchorWord::Tip => Anchor::Tip,
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

fn chain_facing(word: ChainFacingWord) -> ChainFacing {
    match word {
        ChainFacingWord::Outward => ChainFacing::Outward,
        ChainFacingWord::Inward => ChainFacing::Inward,
        ChainFacingWord::Above => ChainFacing::Above,
        ChainFacingWord::Below => ChainFacing::Below,
        ChainFacingWord::Front => ChainFacing::Front,
        ChainFacingWord::Back => ChainFacing::Back,
    }
}

#[cfg(test)]
mod tests;
