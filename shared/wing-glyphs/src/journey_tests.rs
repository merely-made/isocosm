// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
use super::*;
use crate::{CanonSpec, GlyphDefinition, VariantDefinition};

fn canon() -> Canon {
    Canon::new(CanonSpec {
        version: 1,
        id: "world:letters".into(),
        revision: 3,
        glyphs: ["a", "b", "c"]
            .map(|id| GlyphDefinition {
                id: format!("glyph:{id}"),
                display: id.into(),
                effect: format!("effect:reference-{id}"),
            })
            .into(),
        variants: vec![VariantDefinition {
            id: "glyph:acute-a".into(),
            display: "á".into(),
            base: "glyph:a".into(),
            modifiers: vec!["modifier:acute".into()],
        }],
        limits: Default::default(),
    })
    .unwrap()
}
fn provenance(evidence: &str) -> Provenance {
    Provenance {
        kind: ProvenanceKind::Event,
        evidence: evidence.into(),
        context: Some("explicit test fixture".into()),
    }
}
fn journey() -> Journey {
    Journey::new("individual-7".into(), &canon(), vec![0, 10, 20]).unwrap()
}
fn complete(j: &mut Journey) {
    for (n, id) in ["glyph:c", "glyph:a", "glyph:b"].into_iter().enumerate() {
        j.grant(
            id,
            provenance(&format!("evidence/{n}")),
            n as u64,
            VariantPolicy::RequireOwnedBase,
        )
        .unwrap();
    }
}

#[test]
fn acquisitions_keep_first_order_and_provenance_while_new_evidence_accumulates() {
    let mut j = journey();
    j.grant(
        "glyph:c",
        provenance("first"),
        3,
        VariantPolicy::RequireOwnedBase,
    )
    .unwrap();
    let first = j.acquisitions()[0].clone();
    assert!(matches!(
        j.grant(
            "glyph:c",
            provenance("first"),
            99,
            VariantPolicy::AcquireBase
        )
        .unwrap(),
        GrantOutcome::Duplicate { .. }
    ));
    let mut other = provenance("second");
    other.kind = ProvenanceKind::Quest;
    assert!(matches!(
        j.grant("glyph:c", other, 4, VariantPolicy::RequireOwnedBase)
            .unwrap(),
        GrantOutcome::Recorded { .. }
    ));
    assert_eq!(j.acquisitions(), &[first]);
    assert_eq!(j.grants().len(), 2);
    assert_eq!(
        j.motif().iter().map(|g| g.kind).collect::<Vec<_>>(),
        [&ProvenanceKind::Event, &ProvenanceKind::Quest]
    );
    let before = j.snapshot();
    let mut conflict = provenance("first");
    conflict.kind = ProvenanceKind::Bond;
    assert!(
        j.grant("glyph:c", conflict, 5, VariantPolicy::RequireOwnedBase)
            .is_err()
    );
    assert_eq!(j.snapshot(), before);
}

#[test]
fn variants_have_explicit_base_policy_and_never_add_collection_requirements() {
    let mut j = journey();
    assert!(
        j.grant(
            "glyph:missing",
            provenance("x"),
            0,
            VariantPolicy::AcquireBase
        )
        .is_err()
    );
    assert!(
        j.grant(
            "glyph:acute-a",
            provenance("x"),
            0,
            VariantPolicy::RequireOwnedBase
        )
        .is_err()
    );
    assert!(j.grants().is_empty());
    j.grant(
        "glyph:acute-a",
        provenance("x"),
        0,
        VariantPolicy::AcquireBase,
    )
    .unwrap();
    assert_eq!(j.acquisitions()[0].glyph, "glyph:a");
    j.grant(
        "glyph:b",
        provenance("b"),
        1,
        VariantPolicy::RequireOwnedBase,
    )
    .unwrap();
    j.grant(
        "glyph:c",
        provenance("c"),
        2,
        VariantPolicy::RequireOwnedBase,
    )
    .unwrap();
    assert!(j.eligibility().complete);
    assert_eq!(j.acquisitions().len(), 3);
}

#[test]
fn ascension_reincarnation_and_ordinal_unlocks_preserve_individual_journey() {
    let mut j = journey();
    assert!(j.ascend().is_err());
    assert!(j.reincarnate().is_err());
    assert!(j.inherent_unlocked().is_empty());
    complete(&mut j);
    assert_eq!(
        j.eligibility(),
        Eligibility {
            complete: true,
            can_ascend: true,
            can_wish: true
        }
    );
    j.add_experience(100).unwrap();
    j.ascend().unwrap();
    assert!(j.ascend().is_err());
    assert!(j.inherent_unlocked().is_empty());
    let acquired = j.acquisitions().to_vec();
    j.reincarnate().unwrap();
    assert_eq!(
        (j.individual(), j.life(), j.experience(), j.is_divine()),
        ("individual-7", 2, 0, true)
    );
    assert_eq!(j.acquisitions(), acquired);
    assert_eq!(j.inherent_unlocked()[0].glyph, "glyph:c");
    j.add_experience(10).unwrap();
    assert_eq!(
        j.inherent_unlocked()
            .iter()
            .map(|a| a.glyph.as_str())
            .collect::<Vec<_>>(),
        ["glyph:c", "glyph:a"]
    );
    assert_eq!(
        Journey::from_json(&j.to_json().unwrap())
            .unwrap()
            .snapshot(),
        j.snapshot()
    );
}

#[test]
fn forged_serialized_state_is_rejected_by_transition_replay() {
    let mut j = journey();
    complete(&mut j);
    for mutate in [
        |s: &mut JourneySnapshot| s.divine = true,
        |s: &mut JourneySnapshot| s.life = 2,
        |s: &mut JourneySnapshot| s.experience = 100,
        |s: &mut JourneySnapshot| s.acquisitions.swap(0, 1),
        |s: &mut JourneySnapshot| s.transitions.swap(0, 1),
        |s: &mut JourneySnapshot| s.grants[0].life = 2,
    ] {
        let mut snapshot = j.snapshot();
        mutate(&mut snapshot);
        assert!(
            serde_json::from_value::<Journey>(serde_json::to_value(snapshot).unwrap()).is_err()
        );
    }
    let mut value = serde_json::to_value(j.snapshot()).unwrap();
    value["canon"]["glyphs"][1]["effect"] = value["canon"]["glyphs"][0]["effect"].clone();
    assert!(serde_json::from_value::<Journey>(value).is_err());
}

#[test]
fn nonempty_completion_limits_and_checked_failures() {
    let empty = Canon::new(CanonSpec {
        version: 1,
        id: "world:empty".into(),
        revision: 0,
        glyphs: vec![],
        variants: vec![],
        limits: Default::default(),
    })
    .unwrap();
    assert!(
        !Journey::new("someone".into(), &empty, vec![])
            .unwrap()
            .eligibility()
            .complete
    );
    assert!(Journey::new("someone".into(), &canon(), vec![0, 20, 10]).is_err());
    let mut j = Journey::new_with_limits(
        "someone".into(),
        &canon(),
        vec![0; 3],
        JourneyLimits {
            grants: 1,
            transitions: 3,
        },
    )
    .unwrap();
    j.grant("glyph:a", provenance("a"), 0, VariantPolicy::AcquireBase)
        .unwrap();
    let before = j.snapshot();
    assert!(
        j.grant("glyph:b", provenance("b"), 0, VariantPolicy::AcquireBase)
            .is_err()
    );
    assert_eq!(j.snapshot(), before);
    j.add_experience(u64::MAX).unwrap();
    let before = j.snapshot();
    assert_eq!(j.add_experience(1).unwrap_err(), "experience overflow");
    assert_eq!(j.snapshot(), before);
    assert!(j.to_json_with_limit(1).is_err());
}

#[test]
fn ascension_basis_does_not_change_when_later_evidence_accumulates() {
    let mut j = journey();
    complete(&mut j);
    assert!(j.ascension_basis().is_none());
    j.ascend().unwrap();
    let basis = j.ascension_basis().unwrap().to_vec();
    j.grant(
        "glyph:acute-a",
        provenance("later-variant"),
        20,
        VariantPolicy::RequireOwnedBase,
    )
    .unwrap();
    assert_eq!(j.grants().len(), 4);
    assert_eq!(j.ascension_basis().unwrap(), basis);
    j.reincarnate().unwrap();
    let reopened = Journey::from_json(&j.to_json().unwrap()).unwrap();
    assert_eq!(reopened.ascension_basis().unwrap(), basis);
}

#[test]
fn a_grant_keeps_the_revision_it_was_accepted_under_and_refuses_an_older_one() {
    let mut j = journey();
    assert_eq!(j.founding_revision(), 3);
    j.grant("glyph:a", provenance("a"), 0, VariantPolicy::AcquireBase)
        .unwrap();
    j.grant_at_revision("glyph:b", provenance("b"), 1, VariantPolicy::AcquireBase, 9)
        .unwrap();
    assert_eq!(
        j.acquisitions()
            .iter()
            .map(|a| (a.glyph.as_str(), a.canon_revision))
            .collect::<Vec<_>>(),
        [("glyph:a", 3), ("glyph:b", 9)],
        "the plain grant stamps the founding revision"
    );
    assert_eq!(
        j.grants()
            .iter()
            .map(|g| g.canon_revision)
            .collect::<Vec<_>>(),
        [3, 9]
    );

    let before = j.snapshot();
    assert_eq!(
        j.grant_at_revision("glyph:c", provenance("c"), 2, VariantPolicy::AcquireBase, 2)
            .unwrap_err(),
        "grant revision precedes the founding canon"
    );
    assert_eq!(j.snapshot(), before);
    // Eligibility and the ascension basis never read a revision.
    j.grant_at_revision("glyph:c", provenance("c"), 2, VariantPolicy::AcquireBase, 9)
        .unwrap();
    assert!(j.eligibility().can_ascend);
    assert_eq!(
        Journey::from_json(&j.to_json().unwrap())
            .unwrap()
            .snapshot(),
        j.snapshot()
    );
}

#[test]
fn an_archive_written_before_revisions_restores_at_the_founding_revision() {
    let mut j = journey();
    complete(&mut j);
    j.ascend().unwrap();
    let mut value = serde_json::to_value(j.snapshot()).unwrap();
    for list in ["acquisitions", "grants"] {
        for record in value[list].as_array_mut().unwrap() {
            record
                .as_object_mut()
                .unwrap()
                .remove("canon_revision")
                .expect("a current snapshot stamps every record");
        }
    }
    let restored: Journey = serde_json::from_value(value).unwrap();
    assert!(
        restored
            .acquisitions()
            .iter()
            .map(|a| a.canon_revision)
            .chain(restored.grants().iter().map(|g| g.canon_revision))
            .all(|revision| revision == 3)
    );
    assert_eq!(restored.snapshot(), j.snapshot());
    assert!(restored.is_divine());
}
