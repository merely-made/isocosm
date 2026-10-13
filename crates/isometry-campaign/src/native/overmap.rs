// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Presentation over retained disclosures; display cells confer no native reach.

use isocosm::{Session, schema::{Id, Key}};
use isocosm_overlay::{OrdinaryAccess, Seat};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

mod read;
mod types;
pub use read::{OvermapReading, CellReading, BiomeReading, ClaimReading, RouteReading, read_overmap};
pub use types::{Presentation, ViewSettings, Cell, Biome, Thumbnail, OvermapInput, Color};

pub fn validate_presentation(session: &Session, presentation: &Presentation) -> Result<(), String> {
    presentation.validate()?;
    if let Some(disclosure) = &presentation.settings.disclosure {
        types::disclosure(session,disclosure)?;
    }
    for cell in &presentation.cells {
        let disclosure = types::disclosure(session, &cell.disclosure)?;
        if !disclosure.places.iter().any(|place| place.site == cell.site) {
            return Err("overmap cell site is absent from its exact recorded disclosure".into());
        }
    }
    Ok(())
}

/// The actual product candidate publishes these bytes; native facts stay unchanged.
pub fn submit_presentation(session: &mut Session, game: Vec<u8>, access: OrdinaryAccess,
    input: &OvermapInput) -> Result<(String, Vec<u8>), String> {
    super::movement::current_access(session, access)?;
    if access.seat != Seat::Dm || input.version != 1 {
        return Err("overmap presentation requires an explicit current DM input".into());
    }
    validate_presentation(session, &input.presentation)?;
    let mut bindings = super::GameBindings::decode(&game)?;
    bindings.record_overmap(input.presentation.clone());
    Ok(("overmap presentation recorded".into(), bindings.encode()?))
}
