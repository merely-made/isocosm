//! Portable, seat-disclosed native encounter geometry for a table card.
//! Positions are native base cells in x, y (height), z order.
use serde::{Deserialize, Serialize};

pub const BADGE: &str = "pf2e-board-v1";
pub const VALUE: &str = "Board";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EncounterBoard {
    pub site: u64,
    /// Display spacing in base cells. An unmeasured table uses one base cell.
    pub cells_per_square: u32,
    pub cells: Vec<BoardCell>,
    pub tokens: Vec<BoardToken>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoardCell {
    pub at: [i64; 3],
    /// The advertised intent on this very card for moving to this exact cell.
    pub stride: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoardToken {
    pub body: u64,
    pub name: String,
    pub at: [i64; 3],
    pub active: bool,
}
