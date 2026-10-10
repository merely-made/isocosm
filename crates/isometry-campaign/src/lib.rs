//! The VTT's half of the campaign layer (ruling 598): generated maps lowered
//! into the substrate document peers replicate, and the host-private store and
//! proposal lifecycle the sim's laws exclude, handed back (ruling 669). The
//! campaign data, and every check lowering makes, live in the sim at
//! `isocosm::legacy::campaign` (rulings 591, 599).

mod collaboration;
mod store;

pub use collaboration::{CampaignProposal, CampaignProposalError, CampaignProposalMode};
pub use store::CampaignStore;

use isocosm::legacy::campaign::{
    DraftMap, EncounterAnchor, LocalMapProposal, MapProposalError, MapScale, MapTransition,
    SpawnZone,
};
use isometry_core::{Facing, MapDocument, SheetData, Token, TokenId};
use serde::{Deserialize, Serialize};

/// A generated or authored map retained in the campaign registry. The active
/// board remains `GameSnapshot::map`; this record carries scale and traversal
/// metadata that the substrate itself does not interpret.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CampaignMap {
    pub id: String,
    pub scale: MapScale,
    pub document: MapDocument,
    #[serde(default)]
    pub spawn_zones: Vec<SpawnZone>,
    #[serde(default)]
    pub transitions: Vec<MapTransition>,
    #[serde(default)]
    pub encounter_anchors: Vec<EncounterAnchor>,
}

/// Lower a portable map proposal, interning its authored string vocabulary
/// into a `MapDocument`.
pub trait LowerMap {
    fn lower(&self, scale: MapScale) -> Result<CampaignMap, MapProposalError>;
}

impl LowerMap for LocalMapProposal {
    fn lower(&self, scale: MapScale) -> Result<CampaignMap, MapProposalError> {
        self.validate()?;
        let mut document = MapDocument::new(&self.name, self.width, self.height);
        let default_ground = document.intern_tile_kind(&self.default_ground);
        for row in 0..self.height {
            for col in 0..self.width {
                document.ground.set(col, row, default_ground);
            }
        }
        for cell in &self.cells {
            if let Some(ground) = &cell.ground {
                let kind = document.intern_tile_kind(ground);
                document.ground.set(cell.col, cell.row, kind);
            }
            if let Some(prop) = &cell.prop {
                let kind = document.intern_tile_kind(prop);
                document.props.set(cell.col, cell.row, kind);
            }
            if let Some(elevation) = cell.elevation {
                document.elevation.set(cell.col, cell.row, elevation);
            }
        }
        Ok(CampaignMap {
            id: self.id.clone(),
            scale,
            document,
            spawn_zones: self.spawn_zones.clone(),
            transitions: self.transitions.clone(),
            encounter_anchors: self.encounter_anchors.clone(),
        })
    }
}

/// Lower a campaign draft's map with its inhabitants as tokens and sheets.
pub trait LowerDraftMap {
    fn lower(&self) -> Result<CampaignMap, MapProposalError>;
}

impl LowerDraftMap for DraftMap {
    /// Inhabitants are validated before any is installed, so a bad generated
    /// person cannot escape as a partial map.
    fn lower(&self) -> Result<CampaignMap, MapProposalError> {
        self.validate()?;
        let mut lowered = self.map.lower(self.scale)?;
        for inhabitant in &self.inhabitants {
            let id = TokenId(inhabitant.id);
            let mut sheet = SheetData::new(&inhabitant.system);
            for (key, value) in &inhabitant.stats {
                sheet.set_int(key, *value);
            }
            // A sheet's display name is ordinary text data. Keep it separate
            // from the generated numeric vocabulary even if a generator chose
            // the same key in its number map.
            sheet.set_text("name", &inhabitant.name);
            lowered.document.tokens.push(Token {
                id,
                at: (inhabitant.at.col as i32, inhabitant.at.row as i32),
                facing: Facing::South,
                sprite: inhabitant.sprite.clone(),
                owner: inhabitant.owner.clone(),
            });
            lowered.document.set_sheet(id, sheet);
        }
        Ok(lowered)
    }
}

#[cfg(test)]
mod tests;
