// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Hamlet, town and city (case 1025), read off living residents (1027)
//! against the thresholds the tiers datasheet holds.

use super::*;
use std::sync::OnceLock;

const SHEET: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/datasheets/ladder/tiers.toml"));

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Tier {
    Hamlet,
    Town,
    City,
}

impl Tier {
    pub fn word(self) -> &'static str {
        match self {
            Self::Hamlet => "hamlet",
            Self::Town => "town",
            Self::City => "city",
        }
    }
}

/// The living residents at or above which a place reads as each tier.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tiers {
    pub hamlet: u64,
    pub town: u64,
    pub city: u64,
}

#[derive(Deserialize)]
struct Sheet {
    schema: u32,
    status: String,
    residents_from: Tiers,
}

impl Tiers {
    pub fn tier(&self, residents: u64) -> Option<Tier> {
        [(self.city, Tier::City), (self.town, Tier::Town), (self.hamlet, Tier::Hamlet)]
            .into_iter().find(|(from, _)| residents >= *from).map(|(_, tier)| tier)
    }
}

fn parse(text: &str) -> Result<(Tiers, String)> {
    let sheet: Sheet = toml::from_str(text).map_err(|e| format!("tiers datasheet: {e}"))?;
    let t = sheet.residents_from;
    if sheet.schema != 1 || t.hamlet == 0 || t.town <= t.hamlet || t.city <= t.town {
        return Err("tiers datasheet: thresholds must rise from at least one resident".into());
    }
    Ok((t, sheet.status))
}

/// The datasheet's thresholds and status, parsed once.
pub fn tiers() -> &'static (Tiers, String) {
    static TIERS: OnceLock<(Tiers, String)> = OnceLock::new();
    TIERS.get_or_init(|| parse(SHEET).expect("the embedded tiers datasheet parses"))
}
