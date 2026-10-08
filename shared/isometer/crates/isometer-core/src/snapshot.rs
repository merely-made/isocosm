// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The codec seam: postcard bytes in and out, and an equality witness over
//! them.
//!
//! Split out of Mesocosm's `snapshot` module, which keeps the world-facing
//! half — capturing a `World`, restoring one under a ruleset, and
//! `state_hash`. What lives here is the part that has no world in it, so the
//! family and both products encode through one implementation and agree byte
//! for byte.

use serde::{Serialize, de::DeserializeOwned};

/// Why a value would not cross the seam.
///
/// Deliberately two variants and no payload: a product that wants to say more
/// about a refusal wraps this in its own error, as Mesocosm's `SnapshotError`
/// does for a stale ruleset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CodecError {
    Encode,
    Decode,
}

/// Round-trips any value. Used by body-document tests and by hosts that carry
/// parts of a world without carrying all of it.
pub fn encode<T: Serialize>(value: &T) -> Result<Vec<u8>, CodecError> {
    postcard::to_allocvec(value).map_err(|_| CodecError::Encode)
}

pub fn decode<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, CodecError> {
    postcard::from_bytes(bytes).map_err(|_| CodecError::Decode)
}

/// FNV-1a over encoded bytes: an equality witness for replay, not a
/// cryptographic digest. Mere's `state-witness` owns it (wing ruling 609,
/// mere's F119); this re-export keeps every caller's path.
pub use state_witness::hash_bytes;

#[cfg(test)]
mod tests {
    use super::*;

    /// Fixed values, computed apart from this crate, that every saved witness
    /// depends on: a change of implementation must keep them.
    #[test]
    fn the_witness_keeps_its_values() {
        let all: Vec<u8> = (0..=255).collect();
        assert_eq!(hash_bytes(b""), 0xcbf2_9ce4_8422_2325);
        assert_eq!(hash_bytes(b"a"), 0xaf63_dc4c_8601_ec8c);
        assert_eq!(hash_bytes(b"isocosm"), 0xa005_dbe9_febc_5e04);
        assert_eq!(hash_bytes(&all), 0x4242_dc52_49c3_3625);
    }
    use crate::body::{BodyDocument, SpeciesId, VolumeRef};

    #[test]
    fn a_body_round_trips_through_the_seam() {
        let body = BodyDocument::new(SpeciesId(1), VolumeRef::from_tag(1), 1_000, [2, 2, 2]);
        let bytes = encode(&body).unwrap();
        assert_eq!(decode::<BodyDocument>(&bytes).unwrap(), body);
    }

    #[test]
    fn the_witness_separates_different_bytes() {
        assert_eq!(hash_bytes(b"abc"), hash_bytes(b"abc"));
        assert_ne!(hash_bytes(b"abc"), hash_bytes(b"abd"));
    }

    #[test]
    fn a_short_buffer_is_refused_rather_than_guessed() {
        assert_eq!(decode::<BodyDocument>(&[]), Err(CodecError::Decode));
    }
}
