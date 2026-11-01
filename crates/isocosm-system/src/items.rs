//! SRD equipment as content, loaded from the bundled `5e-srd` edition pack.
//!
//! The pack's `items` records are the SRD 5.1 equipment list, in TOML, transformed from
//! 5e-database (P2b), used under CC-BY-4.0 with attribution (bootstrap
//! decision #7); the pack declares the licence and credit (case 1064).

use serde::{Deserialize, Serialize};

/// An equipment entry.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Item {
    pub key: String,
    pub name: String,
    /// "Weapon", "Armor", "Adventuring Gear", ...
    pub category: String,
    pub cost: String,
    pub weight: String,
    /// The mechanical line (damage for weapons, AC for armor), if any.
    pub detail: String,
    pub desc: String,
}

/// The SRD equipment list from the bundled `5e-srd` pack (SRD 5.1, CC-BY-4.0).
pub fn srd_items() -> Vec<Item> {
    srd_records("items")
}

/// The bundled 5e pack's records of one kind, each file a TOML array under
/// the kind's name; empty when the pack is absent or does not parse, as the
/// vendored lists always were.
pub(crate) fn srd_records<T: serde::de::DeserializeOwned>(kind: &str) -> Vec<T> {
    let texts = isocosm_campaign::bundled_records("5e-srd", kind).unwrap_or_default();
    let parsed = texts.first().and_then(|text| {
        let mut table: toml::Table = toml::from_str(text).ok()?;
        table.remove(kind)?.try_into().ok()
    });
    parsed.unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn items_load_and_read() {
        let items = srd_items();
        assert!(
            items.len() > 50,
            "vendored equipment should be the full SRD set"
        );
        let sword = items.iter().find(|i| i.key == "longsword").unwrap();
        assert_eq!(sword.category, "Weapon");
        assert!(sword.detail.contains("1d8"));
    }
}
