// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
use super::*;
use crate::{KindSetLimits, KindSetSpec, SEEDED_KINDS};

fn entry(
    subject: &str,
    object: &str,
    kind: &str,
    tick: u64,
    cause: &str,
    stance: Stance,
) -> Record {
    Record {
        subject: subject.into(),
        object: object.into(),
        kind: kind.into(),
        canon_revision: 3,
        cause: cause.into(),
        tick,
        stance,
    }
}

/// Four records over four ticks: one subject with two objects, one object
/// with two subjects, and a god who is somebody else's object, so both
/// directions and the crossing between them have something to answer with.
fn spec() -> ImpresaSpec {
    ImpresaSpec {
        version: SCHEMA_VERSION,
        id: "world:demo-impresa".into(),
        entries: vec![
            entry(
                "god:aldis",
                "glyph:0",
                "impresa:embody",
                1,
                "event:ascension-1",
                Stance::Associate,
            ),
            entry(
                "faction:wardens",
                "glyph:0",
                "impresa:claim",
                2,
                "event:accord-7",
                Stance::Associate,
            ),
            entry(
                "god:aldis",
                "place:spire",
                "impresa:discover",
                3,
                "event:survey-2",
                Stance::Associate,
            ),
            entry(
                "critter:vole",
                "god:aldis",
                "impresa:defeat",
                4,
                "event:duel-9",
                Stance::Associate,
            ),
        ],
        limits: Default::default(),
    }
}

/// The seeded six plus a pack's own kind, which is how a world grows one.
fn packed() -> KindSet {
    let mut kinds: Vec<String> = SEEDED_KINDS.iter().copied().map(String::from).collect();
    kinds.push("pack:betrothed".into());
    KindSet::new(KindSetSpec {
        version: SCHEMA_VERSION,
        id: "pack:courtly".into(),
        kinds,
        limits: KindSetLimits::default(),
    })
    .unwrap()
}

#[test]
fn the_seeded_set_carries_the_six_ruled_kinds() {
    let kinds = KindSet::seeded();
    assert_eq!(kinds.spec().kinds.len(), 6);
    for kind in SEEDED_KINDS {
        assert!(kinds.contains(kind), "{kind}");
    }
    assert_eq!(kinds.spec().kinds, SEEDED_KINDS, "authored order is kept");
    assert!(!kinds.contains("pack:betrothed"));
    // Closed by name, so a near miss is a miss.
    assert!(!kinds.contains("claim"));
    assert!(!kinds.contains("impresa:claimed"));
}

#[test]
fn a_pack_extends_the_set_the_way_the_canon_is_extended() {
    let kinds = packed();
    assert!(kinds.contains("pack:betrothed"));
    assert!(
        kinds.contains("impresa:defeat"),
        "the seeded six survive the extension"
    );
    assert_eq!(kinds.spec().kinds.len(), 7);
    // A set is a value: extending it produces another one and leaves the
    // seeded set exactly as it was.
    assert!(!KindSet::seeded().contains("pack:betrothed"));
}

#[test]
fn a_duplicate_kind_is_refused_by_name() {
    let mut spec = KindSetSpec::from(packed());
    spec.kinds.push("impresa:invoke".into());
    let why = KindSet::new(spec).unwrap_err();
    assert!(
        why.contains("duplicate association kind: impresa:invoke"),
        "{why}"
    );
}

#[test]
fn an_empty_kind_set_is_refused() {
    let mut spec = KindSetSpec::from(KindSet::seeded());
    spec.kinds.clear();
    let why = KindSet::new(spec).unwrap_err();
    assert!(why.contains("at least one kind"), "{why}");
}

#[test]
fn a_malformed_kind_set_refuses_at_new_and_through_serde() {
    let base = KindSetSpec::from(KindSet::seeded());
    let mut bare = base.clone();
    bare.kinds[0] = "claim".into();
    assert!(KindSet::new(bare).is_err(), "a kind is namespaced");
    let mut unnamed = base.clone();
    unnamed.id = "pack".into();
    assert!(KindSet::new(unnamed).is_err());
    let mut version = base.clone();
    version.version = SCHEMA_VERSION + 1;
    assert!(KindSet::new(version).is_err());
    let mut limited = base.clone();
    limited.limits.kinds = 5;
    assert!(KindSet::new(limited).is_err());
    // Deserialization goes through the same door, so a hand-written file
    // cannot admit what `new` refuses.
    let mut stray = serde_json::to_value(&base).unwrap();
    stray["extra"] = true.into();
    assert!(serde_json::from_value::<KindSet>(stray).is_err());
}

#[test]
fn a_record_naming_an_unconfigured_kind_is_refused_by_name() {
    let mut spec = spec();
    spec.entries[1].kind = "pack:betrothed".into();
    let why = Impresa::new(spec.clone(), &KindSet::seeded()).unwrap_err();
    assert!(
        why.contains("unknown association kind: pack:betrothed"),
        "{why}"
    );
    // The same record is admitted by the world that configured the kind: an
    // unknown kind is a mapping question at the boundary, not a bad record.
    assert!(Impresa::new(spec, &packed()).is_ok());
}

#[test]
fn a_malformed_subject_or_object_identifier_is_refused() {
    let kinds = KindSet::seeded();
    let mut bare = spec();
    bare.entries[0].subject = "aldis".into();
    assert!(
        Impresa::new(bare, &kinds)
            .unwrap_err()
            .contains("namespace:local")
    );
    let mut empty_local = spec();
    empty_local.entries[2].object = "place:".into();
    assert!(
        Impresa::new(empty_local, &kinds)
            .unwrap_err()
            .contains("invalid namespaced ID")
    );
    let mut spaced = spec();
    spaced.entries[3].subject = "critter:field vole".into();
    assert!(Impresa::new(spaced, &kinds).is_err());
    let mut unnamed = spec();
    unnamed.id = "demo".into();
    assert!(Impresa::new(unnamed, &kinds).is_err());
}

#[test]
fn a_subject_is_not_associated_with_itself() {
    let mut spec = spec();
    spec.entries[0].object = "god:aldis".into();
    let why = Impresa::new(spec, &KindSet::seeded()).unwrap_err();
    assert!(
        why.contains("god:aldis cannot be associated with itself"),
        "{why}"
    );
}

#[test]
fn a_cause_receipt_must_be_present_and_bounded() {
    let kinds = KindSet::seeded();
    let mut long = spec();
    long.entries[0].cause = "e".repeat(DEFAULT_MAX_CAUSE_BYTES + 1);
    let why = Impresa::new(long, &kinds).unwrap_err();
    assert!(why.contains("cause receipt must contain 1..512"), "{why}");
    // Nothing is inferred from a name, so a record with no cited event is
    // refused rather than stored as an unsourced association.
    let mut absent = spec();
    absent.entries[0].cause.clear();
    assert!(Impresa::new(absent, &kinds).is_err());
    let mut exact = spec();
    exact.entries[0].cause = "e".repeat(DEFAULT_MAX_CAUSE_BYTES);
    assert!(Impresa::new(exact, &kinds).is_ok());
}

#[test]
fn ticks_must_not_decrease_but_may_repeat() {
    let kinds = KindSet::seeded();
    let mut backwards = spec();
    backwards.entries[2].tick = 1;
    let why = Impresa::new(backwards, &kinds).unwrap_err();
    assert!(why.contains("ticks must not decrease: 1 after 2"), "{why}");
    // Two associations accepted in one settlement share a tick; record order
    // still says which is later.
    let mut together = spec();
    together.entries[1].tick = 1;
    assert!(Impresa::new(together, &kinds).is_ok());
}

#[test]
fn the_entry_limit_is_enforced() {
    let kinds = KindSet::seeded();
    let mut limited = spec();
    limited.limits.entries = 3;
    let why = Impresa::new(limited.clone(), &kinds).unwrap_err();
    assert!(why.contains("exceeds entry limit"), "{why}");
    limited.limits.entries = 4;
    assert_eq!(Impresa::new(limited, &kinds).unwrap().len(), 4);
}

#[test]
fn stance_is_the_latest_record_for_that_triple_and_only_that_triple() {
    let mut spec = spec();
    spec.entries.push(entry(
        "god:aldis",
        "glyph:0",
        "impresa:embody",
        6,
        "event:doubt-1",
        Stance::Lapse,
    ));
    spec.entries.push(entry(
        "god:aldis",
        "glyph:0",
        "impresa:invoke",
        7,
        "event:rite-4",
        Stance::Associate,
    ));
    let impresa = Impresa::new(spec, &KindSet::seeded()).unwrap();
    assert_eq!(
        impresa.stance("god:aldis", "glyph:0", "impresa:embody"),
        Some(Stance::Lapse)
    );
    // The kind carries the relation, so lapsing one says nothing about
    // another on the same pair.
    assert_eq!(
        impresa.stance("god:aldis", "glyph:0", "impresa:invoke"),
        Some(Stance::Associate)
    );
    // Direction is carried too: the vole defeated the god, not the reverse.
    assert_eq!(
        impresa.stance("critter:vole", "god:aldis", "impresa:defeat"),
        Some(Stance::Associate)
    );
    assert_eq!(
        impresa.stance("god:aldis", "critter:vole", "impresa:defeat"),
        None
    );
    assert_eq!(impresa.len(), 6);
    assert!(!impresa.is_empty());
}

#[test]
fn a_lapse_and_a_re_association_are_both_pointable_with_their_causes() {
    let mut spec = spec();
    spec.entries.push(entry(
        "faction:wardens",
        "glyph:0",
        "impresa:claim",
        5,
        "event:schism-3",
        Stance::Lapse,
    ));
    spec.entries.push(entry(
        "faction:wardens",
        "glyph:0",
        "impresa:claim",
        9,
        "event:accord-12",
        Stance::Associate,
    ));
    let impresa = Impresa::new(spec, &KindSet::seeded()).unwrap();
    let history: Vec<_> = impresa
        .history("faction:wardens", "glyph:0", "impresa:claim")
        .map(|r| (r.tick, r.stance, r.cause.as_str()))
        .collect();
    assert_eq!(
        history,
        [
            (2, Stance::Associate, "event:accord-7"),
            (5, Stance::Lapse, "event:schism-3"),
            (9, Stance::Associate, "event:accord-12"),
        ],
        "the lapse is a record, so all three causes survive it"
    );
    assert_eq!(
        impresa.stance("faction:wardens", "glyph:0", "impresa:claim"),
        Some(Stance::Associate)
    );
    // Never recorded is not the same fact as lapsed, and reads as neither.
    assert_eq!(
        impresa.stance("god:aldis", "glyph:0", "impresa:claim"),
        None
    );
    assert_eq!(
        impresa.stance("nobody:here", "glyph:0", "impresa:claim"),
        None
    );
    assert_eq!(
        impresa
            .history("nobody:here", "glyph:0", "impresa:claim")
            .count(),
        0
    );
}

#[test]
fn associations_and_about_read_the_same_records_from_either_end() {
    let impresa = Impresa::new(spec(), &KindSet::seeded()).unwrap();
    assert_eq!(
        impresa
            .associations("god:aldis")
            .map(|r| r.object.as_str())
            .collect::<Vec<_>>(),
        ["glyph:0", "place:spire"]
    );
    assert_eq!(
        impresa
            .about("glyph:0")
            .map(|r| r.subject.as_str())
            .collect::<Vec<_>>(),
        ["god:aldis", "faction:wardens"]
    );
    // Every record reachable from the subject end is the same record reached
    // from the object end; the inverse is a projection, never a second copy.
    for record in impresa.associations("god:aldis") {
        assert!(
            impresa
                .about(&record.object)
                .any(|r| std::ptr::eq(r, record))
        );
    }
    // A subject in one record is an object in another, which is what makes
    // "what is associated with me" answerable for anything pointable.
    assert_eq!(
        impresa
            .about("god:aldis")
            .map(|r| r.subject.as_str())
            .collect::<Vec<_>>(),
        ["critter:vole"]
    );
    // An unrecorded end is an empty answer, never a panic and never a guess.
    assert_eq!(impresa.associations("place:spire").count(), 0);
    assert_eq!(impresa.about("nobody:here").count(), 0);
}

#[test]
fn a_full_impresa_round_trips_json_and_re_serializes_byte_identically() {
    let kinds = KindSet::seeded();
    let impresa = Impresa::new(spec(), &kinds).unwrap();
    let json = impresa.to_json().unwrap();
    let read = Impresa::from_json(&json, &kinds).unwrap();
    assert_eq!(read, impresa);
    assert_eq!(
        read.to_json().unwrap(),
        json,
        "the file a product wrote is the file it reads back"
    );
    assert_eq!(read.spec(), &spec());
    // The wire spelling is the ruling's, not Rust's.
    assert!(json.contains("\"stance\": \"associate\""), "{json}");
    assert!(impresa.to_json_with_limit(1).is_err());
    assert!(Impresa::from_json_with_limit(&json, &kinds, json.len() - 1).is_err());
    let clone = impresa.clone();
    assert!(Arc::ptr_eq(&impresa.spec, &clone.spec));
    assert!(Arc::ptr_eq(&impresa.by_object, &clone.by_object));
}

#[test]
fn a_stray_field_or_a_future_version_refuses_at_the_serde_door() {
    let mut stray = serde_json::to_value(spec()).unwrap();
    stray["extra"] = true.into();
    assert!(serde_json::from_value::<Impresa>(stray).is_err());
    let mut version = spec();
    version.version = SCHEMA_VERSION + 1;
    assert!(Impresa::new(version, &KindSet::seeded()).is_err());
    // Shape is checked by serde alone; the kind set is a world's
    // configuration and arrives only at `new`, so `from_json` is the door a
    // product reads through.
    let mut unconfigured = spec();
    unconfigured.entries[0].kind = "pack:betrothed".into();
    let json = serde_json::to_string(&unconfigured).unwrap();
    assert!(serde_json::from_str::<Impresa>(&json).is_ok());
    assert!(Impresa::from_json(&json, &KindSet::seeded()).is_err());
    assert!(Impresa::from_json(&json, &packed()).is_ok());
    let mut malformed = spec();
    malformed.entries[0].subject = "aldis".into();
    let json = serde_json::to_string(&malformed).unwrap();
    assert!(serde_json::from_str::<Impresa>(&json).is_err());
}

/// The counterpart of the sibling's `ExpressionTable::covers(&Canon)`: what a
/// caller who took the serde door rather than `from_json` uses to finish the
/// check the kind set could not reach.
#[test]
fn covers_finishes_the_check_the_serde_door_cannot() {
    let mut unconfigured = spec();
    unconfigured.entries[0].kind = "pack:betrothed".into();
    let json = serde_json::to_string(&unconfigured).unwrap();
    let admitted: Impresa = serde_json::from_str(&json).expect("shape alone admits");

    let refused = admitted
        .covers(&KindSet::seeded())
        .expect_err("the seeded set does not configure a pack kind");
    assert!(refused.contains("pack:betrothed"), "{refused}");
    admitted
        .covers(&packed())
        .expect("the pack's own set configures it");

    // A wholly seeded impresa is covered by the seeded set.
    Impresa::new(spec(), &KindSet::seeded())
        .expect("a seeded impresa")
        .covers(&KindSet::seeded())
        .expect("covered");
}
