// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! What a campaign is founded on (cases 1029, 1064, 1067): an edition, the
//! packs it plays pinned by id, version and content hash, the primary first,
//! and optionally a named hybrid profile. A save from before packs carries
//! only the edition key and reads as that edition's bundled pack.

use super::subsystems;
use isocosm_campaign::{EditionPack, PackPin, bundled};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "Repr", into = "Repr")]
pub struct PackSet {
    pub edition: String,
    /// Pinned packs, the edition's own first. Empty reads as its bundled pack.
    pub packs: Vec<PackPin>,
    pub hybrid: Option<String>,
}

/// An unpinned set keeps the bare key it was always saved as.
#[derive(Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum Repr {
    Key(String),
    Set {
        edition: String,
        packs: Vec<PackPin>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        hybrid: Option<String>,
    },
}

impl From<Repr> for PackSet {
    fn from(repr: Repr) -> Self {
        match repr {
            Repr::Key(edition) => edition.into(),
            Repr::Set { edition, packs, hybrid } => Self { edition, packs, hybrid },
        }
    }
}

impl From<PackSet> for Repr {
    fn from(set: PackSet) -> Self {
        if set.packs.is_empty() && set.hybrid.is_none() {
            Repr::Key(set.edition)
        } else {
            Repr::Set { edition: set.edition, packs: set.packs, hybrid: set.hybrid }
        }
    }
}

impl From<String> for PackSet {
    fn from(edition: String) -> Self {
        Self { edition, packs: Vec::new(), hybrid: None }
    }
}

impl From<&str> for PackSet {
    fn from(edition: &str) -> Self {
        edition.to_owned().into()
    }
}

/// The edition a table that never chose is founded under.
pub fn default_edition<T: From<&'static str>>() -> T {
    "5e-srd".into()
}

pub fn is_default_edition(set: &PackSet) -> bool {
    *set == default_edition::<PackSet>()
}

/// The bundled pack a campaign founded under `edition` plays.
pub fn edition_pack(edition: &str) -> Result<&'static EditionPack, String> {
    bundled()?
        .iter()
        .find(|pack| pack.edition().system.as_deref() == Some(edition))
        .ok_or_else(|| format!("unknown edition {edition}"))
}

/// The editions founding offers: bundled packs naming a system whose every
/// declared subsystem is registered.
pub fn editions() -> Vec<&'static str> {
    let Ok(packs) = bundled() else { return Vec::new() };
    packs
        .iter()
        .filter(|pack| {
            let edition = pack.edition();
            !edition.subsystems.is_empty()
                && edition.subsystems.iter().all(|key| subsystems::subsystem(key).is_some())
        })
        .filter_map(|pack| pack.edition().system.as_deref())
        .collect()
}

/// What the table calls an edition: its pack's name.
pub fn edition_label(edition: &str) -> String {
    edition_pack(edition).map_or_else(|_| edition.to_owned(), |pack| pack.manifest().name.clone())
}

impl PackSet {
    /// Pin `edition`'s bundled pack as the whole set.
    pub fn bundled(edition: &str) -> Result<Self, String> {
        Ok(Self { edition: edition.to_owned(), packs: vec![edition_pack(edition)?.pin()], hybrid: None })
    }

    /// The packs this set plays, each checked against its pin. An unpinned
    /// set is its edition's bundled pack. Pack transfer at join is not built:
    /// a pin this host's bundled packs do not match is refused.
    pub fn resolve(&self) -> Result<Vec<&'static EditionPack>, String> {
        let primary = edition_pack(&self.edition)?;
        if self.packs.is_empty() {
            return Ok(vec![primary]);
        }
        if self.packs[0].id != primary.id() {
            return Err(format!("the first pinned pack must be edition {}'s own", self.edition));
        }
        let packs = bundled()?;
        self.packs
            .iter()
            .map(|pin| {
                let pack = packs
                    .iter()
                    .find(|pack| pack.id() == pin.id)
                    .ok_or_else(|| format!("pack {} is pinned but not held here", pin.id))?;
                if pack.hash() != pin.hash {
                    return Err(format!(
                        "pack {} {} is pinned at {}, but this host holds {} at {}",
                        pin.id, pin.version, pin.hash, pack.manifest().version, pack.hash()
                    ));
                }
                Ok(pack)
            })
            .collect()
    }

    /// The founding checks: the edition is offered, every pin holds, and the
    /// declared subsystems do not overlap unless the hybrid profile settles it.
    pub fn validate(&self) -> Result<(), String> {
        if !editions().contains(&self.edition.as_str()) {
            bundled()?;
            return Err(format!("unknown edition {}", self.edition));
        }
        let packs = self.resolve()?;
        let mut ids: Vec<&str> = packs.iter().map(|pack| pack.id()).collect();
        ids.sort();
        if ids.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err("a pack is pinned twice".into());
        }
        let keys: Vec<&str> = packs
            .iter()
            .flat_map(|pack| pack.edition().subsystems.iter().map(String::as_str))
            .collect();
        subsystems::check(&keys, self.hybrid.as_deref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_unpinned_set_saves_as_the_bare_key_and_a_pinned_one_resolves() {
        let old: PackSet = serde_json::from_str("\"pf2e-srd\"").unwrap();
        assert_eq!(serde_json::to_string(&old).unwrap(), "\"pf2e-srd\"");
        assert!(old.validate().is_ok());
        let pinned = PackSet::bundled("pf2e-srd").unwrap();
        let text = serde_json::to_string(&pinned).unwrap();
        assert_eq!(serde_json::from_str::<PackSet>(&text).unwrap(), pinned);
        assert!(pinned.validate().is_ok());
        let mut stale = pinned.clone();
        stale.packs[0].hash = "0".repeat(64);
        assert!(stale.validate().unwrap_err().contains("pinned at"));
        let mut mixed = pinned;
        mixed.packs.push(edition_pack("5e-srd").unwrap().pin());
        assert!(mixed.validate().unwrap_err().contains("both govern"));
        let mut ars = PackSet::bundled("5e-srd").unwrap();
        ars.packs.push(edition_pack("ars5").unwrap().pin());
        assert!(ars.validate().unwrap_err().contains("both govern"), "two editions' creation overlap");
    }

    #[test]
    fn founding_offers_the_playable_bundled_editions() {
        assert_eq!(editions(), ["5e-srd", "ars5", "pf2e-srd"]);
        assert_eq!(edition_label("pf2e-srd"), "Pathfinder 2e Player Core (Remaster)");
        assert!(PackSet::from("ars5").validate().is_ok());
        assert!(PackSet::from("unknown").validate().is_err());
    }
}
