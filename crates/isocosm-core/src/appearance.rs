//! A token's look as the table keeps it (ruling 1022): its class is the
//! token's `sprite`, and this holds only what the token overrides.
//!
//! Whatever is not overridden follows the pack that binds the class, so a
//! reskin reaches it. The substrate stores the overrides and never reads them:
//! the views resolve them against the pack's bindings.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// The overrides on a token's class: which layer a slot shows, and which
/// palette entries are replaced. Empty means "exactly the class".
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TokenAppearance {
    /// Slot name to layer name, for the slots this token chose itself.
    #[serde(default)]
    pub layers: BTreeMap<String, String>,
    /// Palette index to colour, for the entries this token recoloured.
    #[serde(default)]
    pub palette: BTreeMap<u8, [u8; 3]>,
}

impl TokenAppearance {
    pub fn is_empty(&self) -> bool {
        self.layers.is_empty() && self.palette.is_empty()
    }
}
