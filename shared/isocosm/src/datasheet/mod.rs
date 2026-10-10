// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The founding datasheets (wing rulings 625 to 632, 626 and 758): read once
//! here, embedded in isocosm, and lowered onto native kinds and recipes
//! ([`native`]); the legacy loader lowers the same parse onto its own
//! recipes until the world families move. One founding names a palette and
//! per-tier bodies, `<sheet>.<body>`; a palette's banks hold named shapes,
//! one bank per box class, a palette extending another.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use serde::Deserialize;

pub mod native;
pub mod schema;

use schema::*;

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
pub const SETS: [(&str, &str); 4] = [
    sheet!("base"),
    sheet!("branching"),
    sheet!("jointed"),
    sheet!("spaced"),
];
pub const FOUNDINGS: (&str, &str) = sheet!("foundings");

/// The shapes a bank may hold: a default and three spares.
pub const BANK_SLOTS: usize = 4;

/// The four banks, one per box class, in this order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Bank {
    Mass,
    Limb,
    Plate,
    Sensor,
}

impl Bank {
    pub const ALL: [Bank; 4] = [Bank::Mass, Bank::Limb, Bank::Plate, Bank::Sensor];

    fn index(self) -> usize {
        self as usize
    }
}

/// One shape a bank holds: its name, volume tag and half-extents.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Shape {
    pub name: String,
    pub tag: u8,
    pub half_extent: [i32; 3],
}

/// A palette resolved through what it extends, each bank in selector order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Banks {
    banks: [Vec<Shape>; 4],
}

impl Banks {
    pub fn bank(&self, bank: Bank) -> &[Shape] {
        &self.banks[bank.index()]
    }

    /// The named shape's slot in `bank`, the default where none is named.
    pub fn slot(&self, bank: Bank, name: Option<&str>) -> Result<u8, String> {
        let Some(name) = name else { return Ok(0) };
        self.bank(bank)
            .iter()
            .position(|s| s.name == name)
            .map(|i| i as u8)
            .ok_or_else(|| format!("the palette admits no {bank:?} shape {name:?}"))
    }

    /// The shape at that slot.
    pub fn shape(&self, bank: Bank, name: Option<&str>) -> Result<&Shape, String> {
        let slot = self.slot(bank, name)?;
        self.bank(bank)
            .get(usize::from(slot))
            .ok_or_else(|| format!("the {bank:?} bank has no default"))
    }
}

/// Every sheet, parsed and checked.
pub struct Sheets {
    pub sets: BTreeMap<String, SetSheet>,
    pub foundings: FoundingsSheet,
}

/// The embedded sheets, parsed on first use; a failure is a broken build.
pub fn sheets() -> &'static Sheets {
    static SHEETS: OnceLock<Sheets> = OnceLock::new();
    SHEETS.get_or_init(|| Sheets::parse(&SETS, FOUNDINGS.1).unwrap_or_else(|why| panic!("{why}")))
}

fn parse<T: for<'de> Deserialize<'de>>(name: &str, text: &str) -> Result<T, String> {
    toml::from_str(text).map_err(|why| format!("{name}.toml: {why}"))
}

impl Sheets {
    /// Reads roster-set sheets, given as `(name, text)`, and a foundings
    /// sheet, refusing a palette defined twice or a default that names none.
    pub fn parse(sets: &[(&str, &str)], foundings: &str) -> Result<Sheets, String> {
        let mut parsed = BTreeMap::new();
        for &(name, text) in sets {
            let sheet: SetSheet = parse(name, text)?;
            sheet.check(name)?;
            if parsed.insert(name.to_string(), sheet).is_some() {
                return Err(format!("{name}: a sheet is named twice"));
            }
        }
        let founding: FoundingsSheet = parse(FOUNDINGS.0, foundings)?;
        founding.check(FOUNDINGS.0)?;
        let sheets = Sheets {
            sets: parsed,
            foundings: founding,
        };
        let mut seen = BTreeMap::new();
        for (sheet, set) in &sheets.sets {
            for name in set.palettes.keys() {
                if seen.insert(name.clone(), sheet.clone()).is_some() {
                    return Err(format!("{sheet}: palette {name:?} is defined twice"));
                }
            }
        }
        if !sheets
            .foundings
            .founding
            .contains_key(&sheets.foundings.default)
        {
            let default = &sheets.foundings.default;
            return Err(format!("no founding {default:?} to default to"));
        }
        Ok(sheets)
    }

    fn palette_entry(&self, name: &str) -> Option<&PaletteEntry> {
        self.sets.values().find_map(|s| s.palettes.get(name))
    }

    /// Every palette any sheet defines.
    pub fn palette_names(&self) -> impl Iterator<Item = &str> {
        self.sets
            .values()
            .flat_map(|s| s.palettes.keys().map(String::as_str))
    }

    /// A palette resolved over what it extends: a shape that replaces one
    /// takes its slot, any other joins its bank, each bank at most
    /// [`BANK_SLOTS`] shapes and no name twice.
    pub fn banks(&self, name: &str) -> Result<Banks, String> {
        self.banks_at(name, 0)
    }

    fn banks_at(&self, name: &str, depth: usize) -> Result<Banks, String> {
        let entry = self
            .palette_entry(name)
            .ok_or_else(|| format!("no palette {name:?}"))?;
        if depth > self.palette_names().count() {
            return Err(format!("palette {name:?} extends itself"));
        }
        let mut banks = match &entry.extends {
            Some(parent) => self.banks_at(parent, depth + 1)?,
            None => Banks::default(),
        };
        let own = [&entry.mass, &entry.limb, &entry.plate, &entry.sensor];
        for (bank, shapes) in Bank::ALL.into_iter().zip(own) {
            let held = &mut banks.banks[bank.index()];
            for s in shapes {
                let shape = Shape {
                    name: s.name.clone(),
                    tag: s.tag,
                    half_extent: s.half_extent,
                };
                let taken = held.iter().any(|h| h.name == s.name);
                let twice = || format!("palette {name:?}: {bank:?} shape {:?} twice", s.name);
                match &s.replaces {
                    Some(old) => {
                        let slot = held.iter().position(|h| &h.name == old).ok_or_else(|| {
                            format!("palette {name:?}: no {bank:?} shape {old:?} to replace")
                        })?;
                        if taken && held[slot].name != s.name {
                            return Err(twice());
                        }
                        held[slot] = shape;
                    },
                    None if taken => return Err(twice()),
                    None if held.len() == BANK_SLOTS => {
                        return Err(format!(
                            "palette {name:?}: the {bank:?} bank holds {BANK_SLOTS} shapes"
                        ));
                    },
                    None => held.push(shape),
                }
            }
        }
        Ok(banks)
    }

    /// A body by `<sheet>.<body>`, with the sheet it lies in.
    pub fn body(&self, body: &str) -> Result<(&str, &BodyEntry), String> {
        let (sheet, name) = body
            .split_once('.')
            .ok_or_else(|| format!("{body:?} is not <sheet>.<body>"))?;
        let set = self
            .sets
            .get_key_value(sheet)
            .ok_or_else(|| format!("{body}: no sheet {sheet:?}"))?;
        let entry = set
            .1
            .body
            .get(name)
            .ok_or_else(|| format!("{body}: no such body"))?;
        Ok((set.0.as_str(), entry))
    }

    /// A founding by name.
    pub fn founding(&self, name: &str) -> Result<&FoundingEntry, String> {
        self.foundings
            .founding
            .get(name)
            .ok_or_else(|| format!("the founding datasheet declares no {name:?}"))
    }
}

#[cfg(test)]
mod tests;
