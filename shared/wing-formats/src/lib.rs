// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Primitive, versioned interchange records for the games wing.
//!
//! These records carry no game behavior. Products own their local validation,
//! interpretation, and projection methods; this crate owns only the stable
//! serde layouts, the raw header checked before positional decoding, and the
//! migrations that carry an older chronicle forward.

use serde::{Deserialize, Serialize, de::DeserializeOwned};

/// Magic plus a little-endian `u16` version.
pub const HEADER_LEN: usize = 10;
pub const BODY_SCHEMA: &str = "mesocosm.body/v0";
pub const BODY_MAGIC: [u8; 8] = *b"MESOBODY";
pub const BODY_VERSION: u16 = 0;
pub const CHRONICLE_SCHEMA: &str = "mesocosm.chronicle/v2";
pub const CHRONICLE_MAGIC: [u8; 8] = *b"MESOCHRN";
/// v1 kept v0's layout and renamed the VTT's vessel (case 1040); v2 makes a
/// deed's time signed, so a date before a world's founding is kept (1069).
pub const CHRONICLE_VERSION: u16 = 2;
/// The VTT's vessel name from v1 on.
pub const VTT_VESSEL: &str = "isocosm";
/// The VTT's vessel name in a v0 chronicle.
pub const V0_VTT_VESSEL: &str = "isometry";

/// Shared rejection reasons for framed v0 artifacts.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WireError {
    TooShort { got: usize },
    WrongSchema { found: [u8; 8], expected: [u8; 8] },
    UnknownVersion { found: u16, expected: u16 },
    Malformed,
    Inconsistent,
    Encode,
}

/// Writes a postcard payload after its fixed schema header.
pub fn frame<T: Serialize>(magic: [u8; 8], version: u16, value: &T) -> Result<Vec<u8>, WireError> {
    let payload = postcard::to_allocvec(value).map_err(|_| WireError::Encode)?;
    let mut bytes = Vec::with_capacity(HEADER_LEN + payload.len());
    bytes.extend_from_slice(&magic);
    bytes.extend_from_slice(&version.to_le_bytes());
    bytes.extend_from_slice(&payload);
    Ok(bytes)
}

/// Reads a framed postcard payload, checking magic and version first.
pub fn unframe<T: DeserializeOwned>(
    magic: [u8; 8],
    version: u16,
    bytes: &[u8],
) -> Result<T, WireError> {
    let (found, payload) = split(magic, bytes)?;
    if found != version {
        return Err(WireError::UnknownVersion {
            found,
            expected: version,
        });
    }
    postcard::from_bytes(payload).map_err(|_| WireError::Malformed)
}

/// Reads a matching header without decoding its payload.
pub fn peek(magic: [u8; 8], bytes: &[u8]) -> Result<u16, WireError> {
    split(magic, bytes).map(|(version, _)| version)
}

fn split(magic: [u8; 8], bytes: &[u8]) -> Result<(u16, &[u8]), WireError> {
    if bytes.len() < HEADER_LEN {
        return Err(WireError::TooShort { got: bytes.len() });
    }
    let found: [u8; 8] = bytes[..8].try_into().expect("checked header length");
    if found != magic {
        return Err(WireError::WrongSchema {
            found,
            expected: magic,
        });
    }
    Ok((
        u16::from_le_bytes([bytes[8], bytes[9]]),
        &bytes[HEADER_LEN..],
    ))
}

/// A part's origin, flat so foreign readers need no product enum.
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
pub struct PartOrigin {
    pub from_species: Option<u32>,
    pub from_part: Option<u32>,
    pub epoch: u64,
}

impl PartOrigin {
    pub fn is_incorporated(&self) -> bool {
        self.from_species.is_some()
    }
}

/// One opaque fact recorded by a product. `at` is the world tick it happened
/// at, negative before the world's founding (v2).
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct Deed {
    pub vessel: String,
    pub verb: String,
    pub at: i64,
    pub detail: Vec<u8>,
}

/// v0 and v1's layout: an unsigned time, which postcard encodes differently.
#[derive(Deserialize)]
struct UnsignedDeed {
    vessel: String,
    verb: String,
    at: u64,
    detail: Vec<u8>,
}

#[derive(Deserialize)]
struct UnsignedChronicle {
    species: u32,
    parts: Vec<PartOrigin>,
    deeds: Vec<UnsignedDeed>,
}

impl Deed {
    pub fn new(vessel: impl Into<String>, verb: impl Into<String>, at: i64) -> Self {
        Self {
            vessel: vessel.into(),
            verb: verb.into(),
            at,
            detail: Vec::new(),
        }
    }

    pub fn detailed(
        vessel: impl Into<String>,
        verb: impl Into<String>,
        at: i64,
        detail: Vec<u8>,
    ) -> Self {
        Self {
            vessel: vessel.into(),
            verb: verb.into(),
            at,
            detail,
        }
    }
}

/// Primitive history record shared across products.
///
/// This is a raw DTO. A product applies [`validate_chronicle`] where its v0
/// contract requires a body-bearing record.
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Eq, Serialize)]
pub struct Chronicle {
    pub species: u32,
    pub parts: Vec<PartOrigin>,
    pub deeds: Vec<Deed>,
}

/// A flattened voxel body and its per-voxel provenance.
///
/// This is a raw DTO. It can represent incoherent dimensions and grids until
/// a boundary calls [`validate_body_profile`].
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct BodyProfile {
    pub species: u32,
    pub size: [u32; 3],
    pub origin: [i32; 3],
    pub cells: Vec<u8>,
    pub attribution: Vec<u16>,
    pub parts: Vec<PartOrigin>,
}

/// Ensures the parallel voxel and attribution grids describe the same body.
pub fn validate_body_profile(profile: &BodyProfile) -> Result<(), WireError> {
    let cells = profile
        .size
        .iter()
        .try_fold(1usize, |total, dimension| {
            total.checked_mul(*dimension as usize)
        })
        .ok_or(WireError::Inconsistent)?;
    let highest = profile.attribution.iter().copied().max().unwrap_or(0) as usize;
    if cells != profile.cells.len()
        || cells != profile.attribution.len()
        || highest > profile.parts.len()
    {
        return Err(WireError::Inconsistent);
    }
    Ok(())
}

/// Reads a chronicle of any version this build knows, carried forward to the
/// current one: v0 and v1 times become signed, and v0's VTT deeds are
/// renamed. Writers always [`frame`] at [`CHRONICLE_VERSION`], so a migrated
/// chronicle is written new (case 1040).
pub fn read_chronicle(bytes: &[u8]) -> Result<Chronicle, WireError> {
    let version = peek(CHRONICLE_MAGIC, bytes)?;
    if version != 0 && version != 1 {
        return unframe(CHRONICLE_MAGIC, CHRONICLE_VERSION, bytes);
    }
    let old: UnsignedChronicle = unframe(CHRONICLE_MAGIC, version, bytes)?;
    let deeds = old.deeds.into_iter().map(|deed| {
        let vessel = if version == 0 && deed.vessel == V0_VTT_VESSEL { VTT_VESSEL.to_owned() } else { deed.vessel };
        let at = i64::try_from(deed.at).map_err(|_| WireError::Inconsistent)?;
        Ok(Deed { vessel, verb: deed.verb, at, detail: deed.detail })
    }).collect::<Result<_, WireError>>()?;
    Ok(Chronicle { species: old.species, parts: old.parts, deeds })
}

/// Ensures a chronicle still names at least one body part.
pub fn validate_chronicle(chronicle: &Chronicle) -> Result<(), WireError> {
    if chronicle.parts.is_empty() {
        Err(WireError::Inconsistent)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MAGIC: [u8; 8] = *b"TESTWIRE";
    const OTHER: [u8; 8] = *b"OTHERSCH";

    /// What a v0 or v1 writer framed: the same fields, an unsigned time.
    #[derive(Serialize)]
    struct OldDeed<'a>(&'a str, &'a str, u64, Vec<u8>);
    #[derive(Serialize)]
    struct OldChronicle<'a>(u32, Vec<PartOrigin>, Vec<OldDeed<'a>>);

    fn old(vessel: &'static str, at: u64) -> OldChronicle<'static> {
        OldChronicle(3, vec![PartOrigin::default()],
            vec![OldDeed(vessel, "held", at, vec![1]), OldDeed("mesocosm", "ate", 300, vec![])])
    }

    #[test]
    fn older_chronicles_read_forward_and_write_v2() {
        let v0 = frame(CHRONICLE_MAGIC, 0, &old(V0_VTT_VESSEL, 1)).unwrap();
        let read = read_chronicle(&v0).unwrap();
        assert_eq!(read.deeds[0], Deed::detailed(VTT_VESSEL, "held", 1, vec![1]));
        assert_eq!(read.deeds[1], Deed::new("mesocosm", "ate", 300));
        let v1 = frame(CHRONICLE_MAGIC, 1, &old(VTT_VESSEL, 1)).unwrap();
        assert_eq!(read_chronicle(&v1), Ok(read.clone()), "v1 reads as v0 did, less the rename");
        let too_late = frame(CHRONICLE_MAGIC, 1, &old(VTT_VESSEL, u64::MAX)).unwrap();
        assert_eq!(read_chronicle(&too_late), Err(WireError::Inconsistent));
        let v2 = frame(CHRONICLE_MAGIC, CHRONICLE_VERSION, &read).unwrap();
        assert_eq!(peek(CHRONICLE_MAGIC, &v2), Ok(2));
        assert_eq!(read_chronicle(&v2), Ok(read));
        let mut v9 = v2.clone();
        v9[8] = 9;
        assert_eq!(
            read_chronicle(&v9),
            Err(WireError::UnknownVersion { found: 9, expected: CHRONICLE_VERSION })
        );
    }

    #[test]
    fn a_v2_deed_keeps_a_date_before_the_founding() {
        let born = Chronicle { species: 1, parts: vec![PartOrigin::default()],
            deeds: vec![Deed::new(VTT_VESSEL, "birth", -13_149_000)] };
        let bytes = frame(CHRONICLE_MAGIC, CHRONICLE_VERSION, &born).unwrap();
        assert_eq!(read_chronicle(&bytes), Ok(born));
    }

    #[test]
    fn framed_values_round_trip() {
        let bytes = frame(MAGIC, 0, &(7u32, "hello".to_string())).unwrap();
        let back: (u32, String) = unframe(MAGIC, 0, &bytes).unwrap();
        assert_eq!(back, (7, "hello".to_string()));
    }

    #[test]
    fn a_short_read_is_refused_first() {
        assert_eq!(
            unframe::<u32>(MAGIC, 0, b"TEST"),
            Err(WireError::TooShort { got: 4 })
        );
    }

    #[test]
    fn another_schema_is_named_rather_than_mis_decoded() {
        let bytes = frame(OTHER, 0, &1u32).unwrap();
        assert_eq!(
            unframe::<u32>(MAGIC, 0, &bytes),
            Err(WireError::WrongSchema {
                found: OTHER,
                expected: MAGIC,
            })
        );
    }

    #[test]
    fn a_newer_writer_is_diagnosed_without_decoding() {
        let mut bytes = MAGIC.to_vec();
        bytes.extend_from_slice(&9u16.to_le_bytes());
        bytes.extend_from_slice(&[0xff; 24]);
        assert_eq!(
            unframe::<u32>(MAGIC, 0, &bytes),
            Err(WireError::UnknownVersion {
                found: 9,
                expected: 0,
            })
        );
    }

    #[test]
    fn a_truncated_payload_is_malformed_not_a_version_problem() {
        let bytes = frame(MAGIC, 0, &"a reasonably long string".to_string()).unwrap();
        assert_eq!(
            unframe::<String>(MAGIC, 0, &bytes[..HEADER_LEN + 2]),
            Err(WireError::Malformed)
        );
    }

    #[test]
    fn v0_preserves_existing_postcard_trailing_byte_tolerance() {
        let mut bytes = frame(MAGIC, 0, &7u32).unwrap();
        bytes.push(0);
        // Existing v0 readers use postcard::from_bytes, which decodes one
        // value without requiring exhaustion. Tightening this is a separate
        // format decision, not part of a byte-compatible extraction.
        assert_eq!(unframe::<u32>(MAGIC, 0, &bytes), Ok(7));
    }

    #[test]
    fn peeking_reads_the_version_of_a_payload_we_cannot_decode() {
        let bytes = frame(MAGIC, 3, &"anything".to_string()).unwrap();
        assert_eq!(peek(MAGIC, &bytes), Ok(3));
        assert_eq!(
            peek(OTHER, &bytes),
            Err(WireError::WrongSchema {
                found: MAGIC,
                expected: OTHER,
            })
        );
    }

    #[test]
    fn body_validation_refuses_overflowing_dimensions() {
        let profile = BodyProfile {
            species: 1,
            size: [u32::MAX, u32::MAX, u32::MAX],
            origin: [0; 3],
            cells: Vec::new(),
            attribution: Vec::new(),
            parts: Vec::new(),
        };
        assert_eq!(
            validate_body_profile(&profile),
            Err(WireError::Inconsistent)
        );
    }
}
