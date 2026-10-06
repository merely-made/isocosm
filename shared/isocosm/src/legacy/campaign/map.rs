//! Generated map data and its validation. Lowering it into the VTT's map
//! documents is the VTT's (ruling 598).

use serde::{Deserialize, Serialize};

pub const MAX_GENERATED_MAP_EDGE: u32 = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MapScale {
    Local,
    Region,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MapPoint {
    pub col: u32,
    pub row: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MapCellProposal {
    pub col: u32,
    pub row: u32,
    #[serde(default)]
    pub ground: Option<String>,
    #[serde(default)]
    pub prop: Option<String>,
    #[serde(default)]
    pub elevation: Option<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpawnZone {
    pub id: String,
    pub cells: Vec<MapPoint>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MapTransition {
    pub id: String,
    pub at: MapPoint,
    pub target_map: String,
    #[serde(default)]
    pub target_entry: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncounterAnchor {
    pub id: String,
    pub at: MapPoint,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Portable pack output. Sparse cells override `default_ground`; the VTT's
/// lowering interns the authored string vocabulary into its map document.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocalMapProposal {
    pub id: String,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub default_ground: String,
    #[serde(default)]
    pub cells: Vec<MapCellProposal>,
    #[serde(default)]
    pub spawn_zones: Vec<SpawnZone>,
    #[serde(default)]
    pub transitions: Vec<MapTransition>,
    #[serde(default)]
    pub encounter_anchors: Vec<EncounterAnchor>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MapProposalError {
    MissingId,
    InvalidDimensions { width: u32, height: u32 },
    MissingDefaultGround,
    OutOfBounds(MapPoint),
    MissingMetadataId(&'static str),
    InvalidInhabitantId,
    DuplicateInhabitantId(u32),
    DuplicateInhabitantPosition(MapPoint),
    MissingInhabitantField(&'static str),
}

impl LocalMapProposal {
    /// Every check lowering makes, in the order it makes them, building
    /// nothing: what the sim can tell of a proposal without the VTT's
    /// document.
    pub fn validate(&self) -> Result<(), MapProposalError> {
        if self.id.trim().is_empty() {
            return Err(MapProposalError::MissingId);
        }
        if self.width == 0
            || self.height == 0
            || self.width > MAX_GENERATED_MAP_EDGE
            || self.height > MAX_GENERATED_MAP_EDGE
        {
            return Err(MapProposalError::InvalidDimensions {
                width: self.width,
                height: self.height,
            });
        }
        if self.default_ground.trim().is_empty() {
            return Err(MapProposalError::MissingDefaultGround);
        }
        for cell in &self.cells {
            let point = MapPoint {
                col: cell.col,
                row: cell.row,
            };
            require_point(self.width, self.height, point)?;
        }
        for zone in &self.spawn_zones {
            if zone.id.trim().is_empty() {
                return Err(MapProposalError::MissingMetadataId("spawn zone"));
            }
            for point in &zone.cells {
                require_point(self.width, self.height, *point)?;
            }
        }
        for transition in &self.transitions {
            if transition.id.trim().is_empty() {
                return Err(MapProposalError::MissingMetadataId("transition"));
            }
            require_point(self.width, self.height, transition.at)?;
        }
        for anchor in &self.encounter_anchors {
            if anchor.id.trim().is_empty() {
                return Err(MapProposalError::MissingMetadataId("encounter anchor"));
            }
            require_point(self.width, self.height, anchor.at)?;
        }
        Ok(())
    }
}

fn require_point(width: u32, height: u32, point: MapPoint) -> Result<(), MapProposalError> {
    if point.col < width && point.row < height {
        Ok(())
    } else {
        Err(MapProposalError::OutOfBounds(point))
    }
}

impl std::fmt::Display for MapProposalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingId => write!(f, "generated map id is required"),
            Self::InvalidDimensions { width, height } => {
                write!(f, "generated map dimensions are invalid: {width}x{height}")
            }
            Self::MissingDefaultGround => write!(f, "generated map default ground is required"),
            Self::OutOfBounds(point) => {
                write!(
                    f,
                    "generated map point is out of bounds: {},{}",
                    point.col, point.row
                )
            }
            Self::MissingMetadataId(kind) => write!(f, "generated {kind} id is required"),
            Self::InvalidInhabitantId => write!(f, "generated inhabitant id must be nonzero"),
            Self::DuplicateInhabitantId(id) => {
                write!(f, "generated inhabitant id is duplicated: {id}")
            },
            Self::DuplicateInhabitantPosition(point) => {
                write!(
                    f,
                    "generated inhabitant position is duplicated: {},{}",
                    point.col, point.row
                )
            },
            Self::MissingInhabitantField(field) => {
                write!(f, "generated inhabitant {field} is required")
            },
        }
    }
}

impl std::error::Error for MapProposalError {}
