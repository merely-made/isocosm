// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Literal byte pins, captured from this tree before `isometer-core` exists.
//!
//! The isometer family plan moves `BodyDocument`, `PartId`, `VolumeRef`, `Yaw`,
//! `Provenance`, `Origin`, `SpeciesId`, `Ground` and the `snapshot`/`wire` seam
//! out of this crate into `isometer-core`, with `mesocosm-core` re-exporting
//! every moved item at its current path. Postcard is positional and serde's
//! derive is not blind to attribute changes, so that move is byte-neutral only
//! if field order, field names, variant order and derive order survive it
//! untouched. **These pins exist so the isometer-core move can prove byte
//! identity** against a value captured *before* it rather than against itself:
//! a stray `#[serde(rename)]`, a reordered variant or a re-derived enum shows
//! up here instead of silently rewriting every saved world.
//!
//! Every literal below was captured by running this file once against the tree
//! at plan step 5 (commit e088d7e) and pasting the observed value. **A
//! legitimate format change must update them deliberately**, in the same commit
//! that makes the change and with the reason written down. A failing pin is not
//! noise to re-capture; it is the receipt doing its job.
//!
//! Pins that already exist elsewhere, and are not duplicated here:
//! `mesocosm-mesh`'s `content_tests.rs` pins the blake3 content address of the
//! v1 sensor fixture volume as a literal hex digest (`content_ref` is private
//! to that crate, and blake3 is not a dependency of this one); its `profile.rs`
//! suite pins `PROFILE_SCHEMA`/`PROFILE_VERSION` round trips and the framed
//! header bytes; and `mesocosm/testing/bench/` carries the scenario receipts.

use mesocosm_core::{
    Attachment, BodyDocument, Origin, PartId, Provenance, SpeciesId, VolumeRef, World, Yaw,
    snapshot::{encode, hash_bytes},
    state_hash, wire,
};

/// The fixture `mesocosm-runtime`'s `glyph_journey` example hashes: the first
/// seed in `0..32` whose founder stands on solid, in-reach ground. That search
/// selects seed 0, which this fixture therefore names directly.
///
/// Note the committed receipt `mesocosm/testing/glyphs/journey.json` records
/// `765c055b377dfb62` for the same world. That receipt is stale — running the
/// example at e088d7e prints `df397e7c183eec55`, the value pinned below — and
/// is the same class of fixture drift the plan's §5 risk 1 records for
/// `structure-cli.scenario`. The pin records what the tree does today, which is
/// what the move has to preserve.
fn journey_world() -> World {
    let world = World::new(0, 0);
    let p = world.controlled().expect("seed 0 has a founder").position;
    let at = [p[0], p[1] - 1, p[2]];
    assert!(
        at[1] >= 1 && world.ground().solid(at) && world.in_reach(at),
        "seed 0 must still be the grounded founder glyph_journey selects"
    );
    world
}

/// A deliberately small body: a root plus one yawed, incorporated child, so the
/// pinned bytes exercise `Option<Attachment>`, both `Origin` variants and a
/// non-default `Yaw` rather than only the default path.
fn fixture_body() -> BodyDocument {
    let mut body = BodyDocument::new(SpeciesId(7), VolumeRef::from_tag(3), 1_234, [2, 1, 2]);
    body.attach(
        VolumeRef::from_tag(9),
        56,
        [1, 1, 1],
        Attachment {
            parent: PartId(0),
            offset: [3, 0, 0],
            yaw: Yaw::Quarter,
        },
        Provenance {
            origin: Origin::Incorporated {
                from_species: SpeciesId(11),
                from_part: PartId(2),
            },
            epoch: 4,
        },
    )
    .expect("the root is a valid parent");
    body
}

#[test]
fn deterministic_world_state_hash_is_pinned() {
    assert_eq!(
        format!("{:016x}", state_hash(&journey_world())),
        "df397e7c183eec55",
        "seed 0 at tick 0 is glyph_journey's baseline world"
    );
}

#[test]
fn body_profile_schema_and_version_are_pinned() {
    // `mesocosm-mesh` re-exports these as `PROFILE_SCHEMA` / `PROFILE_VERSION`.
    assert_eq!(wing_formats::BODY_SCHEMA, "mesocosm.body/v0");
    assert_eq!(wing_formats::BODY_VERSION, 0u16);
    assert_eq!(wing_formats::BODY_MAGIC, *b"MESOBODY");
    assert_eq!(wing_formats::HEADER_LEN, 10);
}

#[test]
fn fixture_body_document_postcard_bytes_are_pinned() {
    assert_eq!(encode(&fixture_body()).unwrap(), FIXTURE_BODY_POSTCARD);
}

#[test]
fn fixture_body_document_wire_frame_bytes_are_pinned() {
    let framed = wire::frame(
        wing_formats::BODY_MAGIC,
        wing_formats::BODY_VERSION,
        &fixture_body(),
    )
    .unwrap();
    let mut expected = Vec::from(*b"MESOBODY");
    expected.extend_from_slice(&0u16.to_le_bytes());
    expected.extend_from_slice(FIXTURE_BODY_POSTCARD);
    assert_eq!(framed, expected);
    let restored: BodyDocument = wire::unframe(
        wing_formats::BODY_MAGIC,
        wing_formats::BODY_VERSION,
        &framed,
    )
    .unwrap();
    assert_eq!(restored, fixture_body());
}

#[test]
fn fixture_body_revision_hash_is_pinned() {
    // `mesocosm-lens`'s `BodyRevision`, which the tracer keys its body cache
    // on, is exactly `hash_bytes(encode(body))` (`lens/src/body.rs:113-114`).
    // Both halves live in this crate today, so the pin does too.
    assert_eq!(
        format!("{:016x}", hash_bytes(&encode(&fixture_body()).unwrap())),
        "f4abe81ee0b320a4"
    );
}

#[test]
fn fixture_volume_reference_bytes_are_pinned() {
    // `VolumeRef` is a 32-byte content address moving to `isometer-core` with
    // the document. Its postcard form is the raw array, with no length prefix.
    let mut expected = vec![0u8; 32];
    expected[0] = 3;
    assert_eq!(encode(&VolumeRef::from_tag(3)).unwrap(), expected);
}

/// Captured 2026-09-14 at e088d7e. Read the module comment before changing it.
#[rustfmt::skip]
const FIXTURE_BODY_POSTCARD: &[u8] = &[
    0x07, 0x00, 0x00, 0x05, 0x03, 0x04, 0x00, 0x02, 0x02, 0x00,
    0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0xd2, 0x09, 0x04, 0x02, 0x04, 0x04, 0x02, 0x04,
    0x00, 0x00, 0x00, 0x00, 0x01, 0x09, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x38, 0x02, 0x02,
    0x02, 0x02, 0x02, 0x02, 0x01, 0x00, 0x06, 0x00, 0x00, 0x01,
    0x01, 0x0b, 0x02, 0x04, 0x00,
];
