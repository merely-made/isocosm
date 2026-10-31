// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use serde::{Deserialize, Serialize};

/// An opaque reference to one entity in the sim, minted by the sim and
/// carried by value (wing design record §5.2 point 6). Never a pointer, and
/// never constructed by a game.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EntityHandle(pub u64);

/// An opaque reference to a place: a node of the sim's place graph (wing
/// design record rulings 72, 147), such as a range, a boundary or a home.
///
/// A PLACES directive names a location this way, never by site or by
/// coordinates (ruling 205).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PlaceHandle(pub u64);

/// An opaque reference to a priced candidate on the review's table (the
/// sim-side counterpart of a discovered condition), named in a review
/// answer at the epoch boundary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CandidateHandle(pub u64);

/// An opaque reference to a participant: an identity holding a grant with
/// the right to petition (the terminology supersession of 2026-09-20), such
/// as a player, a servitor or a scenario runner. Each has one attention set.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ParticipantHandle(pub u64);

/// An opaque reference to a lineage.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct LineageHandle(pub u64);

/// An opaque reference to a faction, a polity among them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FactionHandle(pub u64);

/// An opaque reference to an event in the record.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EventHandle(pub u64);

/// The shared adapter mapping for authored keys and native event keys (807).
pub fn key_handle(key: &str) -> u64 {
    key.bytes().fold(0xcbf2_9ce4_8422_2325, |h, b| {
        (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
    })
}

/// Reverse lookup succeeds only when a handle names one distinct key.
pub fn key_for_handle<'a>(keys: impl IntoIterator<Item = &'a str>, handle: u64) -> Option<&'a str> {
    unique(keys.into_iter().filter(|key| key_handle(key) == handle))
}

fn unique<'a>(mut keys: impl Iterator<Item = &'a str>) -> Option<&'a str> {
    let key = keys.next()?;
    keys.all(|other| other == key).then_some(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adapter_keys_are_stable_and_reverse_only_when_unique() {
        assert_eq!(key_handle("hello"), 0xa430_d846_80aa_bd0b);
        let keys = ["one", "two"];
        assert_eq!(key_for_handle(keys, key_handle("two")), Some("two"));
        assert_eq!(key_for_handle(keys, key_handle("three")), None);
        assert_eq!(unique(["one", "one"].into_iter()), Some("one"));
        assert_eq!(
            unique(["one", "two"].into_iter()),
            None,
            "a collision cannot pick a key"
        );
    }
}
