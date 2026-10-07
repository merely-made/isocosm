// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The founding datasheets, read into palettes and recipes.
//!
//! Wing design record rulings 625 to 631. The sheets live in
//! `datasheets/foundings/` and compile in: one per roster set (`base`,
//! `branching`, `jointed`, `spaced`) holding palettes, chains and bodies, and
//! `foundings.toml` naming each founding's palette and per-tier bodies. The
//! reasons for every shape and body are comments in the sheets.
//!
//! Sheets name shapes; this module computes the selectors. A tagma's
//! `segment` and `shape` name slots in the palette of the founding that
//! admits the body, `mouth = "jaw"` adds [`JAW_SHAPE`], and
//! `worn = "covering"` adds [`ARMOUR_SHAPE`]. A name that palette does not
//! admit is refused.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use serde::Deserialize;

use crate::legacy::mesocosm::axis::{
    ARMOUR_SHAPE, Anchor, Appendage, AppendageStep, ChainFacing, JAW_SHAPE, Recipe, Stretch, Tagma,
};
use crate::legacy::mesocosm::body::VolumeRef;
use crate::legacy::mesocosm::development::{PALETTE_SHAPES, PartPalette, PartTemplate, RoleShapes};
use crate::legacy::mesocosm::organism::Kingdom;
use crate::legacy::mesocosm::plan::{Facing, Role};

macro_rules! sheet {
    ($name:literal) => {
        (
            $name,
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/datasheets/foundings/",
                $name,
                ".toml"
            )),
        )
    };
}

/// The roster-set sheets, by the name a founding cites them with.
const SETS: [(&str, &str); 4] = [
    sheet!("base"),
    sheet!("branching"),
    sheet!("jointed"),
    sheet!("spaced"),
];
const FOUNDINGS: (&str, &str) = sheet!("foundings");

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

/// Every founding the sheets declare, and every palette they define.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Foundings {
    /// The founding a world takes when it names none.
    pub default: String,
    foundings: BTreeMap<String, FoundingSheet>,
    palettes: BTreeMap<String, PartPalette>,
}

impl Foundings {
    pub fn get(&self, name: &str) -> Option<&FoundingSheet> {
        self.foundings.get(name)
    }

    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.foundings.keys().map(String::as_str)
    }

    pub fn palette(&self, name: &str) -> Option<PartPalette> {
        self.palettes.get(name).copied()
    }
}

/// The embedded sheets, parsed on first use. Tests prove they load, so a
/// failure here is a broken build rather than a runtime condition.
pub fn foundings() -> &'static Foundings {
    static SHEETS: OnceLock<Foundings> = OnceLock::new();
    SHEETS.get_or_init(|| load(&SETS, FOUNDINGS.1).unwrap_or_else(|why| panic!("{why}")))
}

/// Reads roster-set sheets, given as `(name, text)`, and a foundings sheet.
pub fn load(sets: &[(&str, &str)], foundings: &str) -> Result<Foundings, String> {
    let mut sheets = BTreeMap::new();
    for &(name, text) in sets {
        let sheet: SetSheet = parse(name, text)?;
        sheet.check(name)?;
        if sheets.insert(name, sheet).is_some() {
            return Err(format!("{name}: a sheet is named twice"));
        }
    }
    let founding_sheet: FoundingsSheet = parse(FOUNDINGS.0, foundings)?;
    founding_sheet.check(FOUNDINGS.0)?;

    let mut entries = BTreeMap::new();
    for (sheet, set) in &sheets {
        for (name, palette) in &set.palettes {
            if entries.insert(name.as_str(), palette).is_some() {
                return Err(format!("{sheet}: palette {name:?} is defined twice"));
            }
        }
    }
    let mut named = BTreeMap::new();
    for name in entries.keys() {
        named.insert(name.to_string(), named_palette(name, &entries, 0)?);
    }

    let mut resolved = BTreeMap::new();
    for (name, founding) in &founding_sheet.founding {
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
    if !resolved.contains_key(&founding_sheet.default) {
        return Err(format!(
            "no founding {:?} to default to",
            founding_sheet.default
        ));
    }
    let palettes = named
        .iter()
        .map(|(name, palette)| Ok((name.clone(), palette.admitted()?)))
        .collect::<Result<_, String>>()?;
    Ok(Foundings {
        default: founding_sheet.default,
        foundings: resolved,
        palettes,
    })
}

fn parse<T: for<'de> Deserialize<'de>>(name: &str, text: &str) -> Result<T, String> {
    toml::from_str(text).map_err(|why| format!("{name}.toml: {why}"))
}

/// One role's admitted shapes, by name, in selector order.
type Bank = Vec<(String, PartTemplate)>;

/// A palette whose slots keep their names, so bodies can be resolved in it.
#[derive(Clone)]
struct NamedPalette {
    banks: [Bank; 4],
}

impl NamedPalette {
    fn bank(&self, role: Role) -> &Bank {
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

/// Builds a palette from its own entries over the one it extends.
fn named_palette(
    name: &str,
    entries: &BTreeMap<&str, &PaletteEntry>,
    depth: usize,
) -> Result<NamedPalette, String> {
    let entry = entries
        .get(name)
        .ok_or_else(|| format!("no palette {name:?}"))?;
    if depth > entries.len() {
        return Err(format!("palette {name:?} extends itself"));
    }
    let mut palette = match &entry.extends {
        Some(parent) => named_palette(parent, entries, depth + 1)?,
        None => NamedPalette {
            banks: Default::default(),
        },
    };
    for (role, shapes) in [
        (Role::Mass, &entry.mass),
        (Role::Limb, &entry.limb),
        (Role::Plate, &entry.plate),
        (Role::Sensor, &entry.sensor),
    ] {
        let bank = &mut palette.banks[role_index(role)];
        for shape in shapes {
            let template = PartTemplate {
                volume: VolumeRef::from_tag(shape.tag),
                half_extent: shape.half_extent,
            };
            let taken = bank.iter().any(|(slot, _)| *slot == shape.name);
            match &shape.replaces {
                Some(old) => {
                    let slot = bank
                        .iter()
                        .position(|(slot, _)| slot == old)
                        .ok_or_else(|| {
                            format!("palette {name:?}: no {role:?} shape {old:?} to replace")
                        })?;
                    if taken && bank[slot].0 != shape.name {
                        return Err(format!(
                            "palette {name:?}: {role:?} shape {:?} twice",
                            shape.name
                        ));
                    }
                    bank[slot] = (shape.name.clone(), template);
                },
                None if taken => {
                    return Err(format!(
                        "palette {name:?}: {role:?} shape {:?} twice",
                        shape.name
                    ));
                },
                None if bank.len() == PALETTE_SHAPES => {
                    return Err(format!(
                        "palette {name:?}: the {role:?} bank holds {PALETTE_SHAPES} shapes"
                    ));
                },
                None => bank.push((shape.name.clone(), template)),
            }
        }
    }
    Ok(palette)
}

/// Resolves `<sheet>.<body>` in the palette of the founding that admits it.
fn recipe(
    sheets: &BTreeMap<&str, SetSheet>,
    body: &str,
    palette: &NamedPalette,
) -> Result<Recipe, String> {
    let (sheet_name, body_name) = body
        .split_once('.')
        .ok_or_else(|| format!("{body:?} is not <sheet>.<body>"))?;
    let sheet = sheets
        .get(sheet_name)
        .ok_or_else(|| format!("{body}: no sheet {sheet_name:?}"))?;
    let entry = sheet
        .body
        .get(body_name)
        .ok_or_else(|| format!("{body}: no such body"))?;
    let fail = |index: usize, why: String| format!("{body}, tagma {index}: {why}");

    let mut tagmata = Vec::with_capacity(entry.tagma.len());
    let mut layout = Vec::new();
    let mut chains = Vec::new();
    for (index, tagma) in entry.tagma.iter().enumerate() {
        tagmata.push(self::tagma(tagma, palette).map_err(|why| fail(index, why))?);
        if let Some(stretch) = &tagma.layout {
            layout.push(Stretch {
                parent: stretch.parent,
                anchor: stretch.anchor.into(),
                facing: stretch.facing.into(),
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
    let appendage = entry.appendage.map_or(Appendage::None, Appendage::from);
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
    sheets: &BTreeMap<&str, SetSheet>,
    home: &str,
    name: &str,
    palette: &NamedPalette,
) -> Result<Vec<AppendageStep>, String> {
    let (sheet_name, chain_name) = name.split_once('.').unwrap_or((home, name));
    let steps = sheets
        .get(sheet_name)
        .and_then(|sheet| sheet.chain.get(chain_name))
        .ok_or_else(|| format!("no chain {name:?}"))?;
    steps
        .iter()
        .map(|step| {
            let role: Role = step.role.into();
            Ok(AppendageStep {
                role,
                shape: palette.slot(role, Some(&step.shape))?,
                facing: step.facing.into(),
                distal: step.distal,
            })
        })
        .collect()
}

// The sheets' own shapes. Everything is `deny_unknown_fields`, so a misspelt
// key is refused rather than read as its default.

/// Livery's header, which every sheet carries.
macro_rules! header_fields {
    ($sheet:ident { $($(#[$attr:meta])* $field:ident: $ty:ty),* $(,)? }) => {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct $sheet {
            schema: u32,
            owner: String,
            consumer: String,
            status: String,
            sources: BTreeMap<String, Source>,
            $($(#[$attr])* $field: $ty),*
        }

        impl $sheet {
            fn check(&self, sheet: &str) -> Result<(), String> {
                if self.schema != 1 {
                    return Err(format!("{sheet}: schema {} is not 1", self.schema));
                }
                let said = |text: &String| !text.trim().is_empty();
                let sourced = !self.sources.is_empty()
                    && self.sources.values().all(|source| said(&source.doc) && said(&source.note));
                if !(said(&self.owner) && said(&self.consumer) && said(&self.status) && sourced) {
                    return Err(format!("{sheet}: owner, consumer, status and sources are required"));
                }
                Ok(())
            }
        }
    };
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Source {
    doc: String,
    note: String,
}

header_fields!(SetSheet {
    #[serde(default)]
    palettes: BTreeMap<String, PaletteEntry>,
    #[serde(default)]
    chain: BTreeMap<String, Vec<StepEntry>>,
    #[serde(default)]
    body: BTreeMap<String, BodyEntry>,
});

header_fields!(FoundingsSheet {
    default: String,
    founding: BTreeMap<String, FoundingEntry>,
});

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FoundingEntry {
    palette: String,
    #[serde(default)]
    producer: Vec<String>,
    #[serde(default)]
    consumer: Vec<String>,
    #[serde(default)]
    decomposer: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PaletteEntry {
    extends: Option<String>,
    #[serde(default)]
    mass: Vec<ShapeEntry>,
    #[serde(default)]
    limb: Vec<ShapeEntry>,
    #[serde(default)]
    plate: Vec<ShapeEntry>,
    #[serde(default)]
    sensor: Vec<ShapeEntry>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ShapeEntry {
    name: String,
    tag: u8,
    half_extent: [i32; 3],
    replaces: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StepEntry {
    role: RoleWord,
    shape: String,
    facing: ChainFacingWord,
    #[serde(default)]
    distal: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BodyEntry {
    variance: u8,
    tagma: Vec<TagmaEntry>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TagmaEntry {
    segments: u8,
    segment: Option<String>,
    appendage: Option<AppendageWord>,
    shape: Option<String>,
    mouth: Option<MouthWord>,
    worn: Option<WornWord>,
    per_segment: Option<u8>,
    layout: Option<LayoutEntry>,
    chain: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LayoutEntry {
    parent: Option<u8>,
    anchor: AnchorWord,
    facing: FacingWord,
    variance: Option<u8>,
}

/// A lower-case word in a sheet for each variant of a code enum.
macro_rules! words {
    ($word:ident => $code:ident { $($variant:ident),* $(,)? }) => {
        #[derive(Clone, Copy, Deserialize)]
        #[serde(rename_all = "snake_case")]
        enum $word { $($variant),* }

        impl From<$word> for $code {
            fn from(word: $word) -> Self {
                match word { $($word::$variant => $code::$variant),* }
            }
        }
    };
}

words!(AppendageWord => Appendage { None, Limb, Feeler, Plate, Mouth, Vane });
words!(RoleWord => Role { Mass, Limb, Plate, Sensor });
words!(AnchorWord => Anchor { Base, Middle, Tip });
words!(FacingWord => Facing { Front, Back, Left, Right, Above, Below });
words!(ChainFacingWord => ChainFacing { Outward, Inward, Above, Below, Front, Back });

/// Which bank a mouth is drawn from: the mass bank as bulk, the limb bank as
/// an actuator.
#[derive(Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum MouthWord {
    Bulk,
    Jaw,
}

/// Whether a plate is held out from the body or worn against it.
#[derive(Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum WornWord {
    Held,
    Covering,
}

#[cfg(test)]
mod tests;
