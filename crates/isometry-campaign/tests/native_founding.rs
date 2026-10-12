// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use isocosm::{Command, Execution, asserted::{Applied, Assertion, Fact, Place}};
use isometry_campaign::{LowerDraftMap, founding::{Campaign, CampaignSaved, FoundingError}};
#[path = "native_founding/fixture.rs"]
mod fixture;

#[test]
fn native_founding_preserves_authored_content_and_keeps_table_authority_separate() {
    let draft = fixture::draft();
    let before = draft.clone();
    let proposed = Campaign::from_draft(fixture::genesis(), &draft, Execution::Grouped);
    assert!(proposed.is_ok(), "VTT founding must declare the authored history kind");
    let prepared = proposed.unwrap();
    let campaign = &prepared.campaign;
    let content = campaign.content();
    assert_eq!(draft, before);
    assert_eq!((&content.id, &content.name), (&draft.id, &draft.name));
    assert_eq!(content.maps, draft.maps);
    assert_eq!(content.storylets, draft.world.storylets);
    assert_eq!(content.rewards, draft.rewards);
    assert_eq!((&content.starting_map, &content.final_storylet), (&draft.starting_map, &draft.final_storylet));
    assert_eq!(content.table.faction_sheets, draft.world.faction_sheets);
    assert_eq!(content.table.faction_control, draft.world.faction_control);
    assert_eq!(content.table.party_node, draft.world.party_node);
    assert_eq!(content.table.party_pace, draft.world.party_pace);
    assert_eq!(content.table.party_known, draft.world.party_known);
    assert_eq!(content.table.party_resources, draft.world.party_resources);
    assert_eq!(content.lowered_maps().unwrap(), draft.maps.iter().map(|m| m.lower().unwrap()).collect::<Vec<_>>());
    let read = campaign.reading();
    for (entry, input) in read.entries().iter().zip(draft.world.assertions()) {
        assert_eq!(entry.command, Command::Assert(input.clone()));
    }
    assert_eq!(read.entries().len(), draft.world.assertions().len());
    let (_, ford) = read.authored_site("ford").unwrap();
    assert_eq!(ford.authored.as_ref(), draft.world.places().get("ford"));
    assert_eq!(ford.routes[0].travel, 19);
    assert_eq!(ford.routes[0].transmission, 0);
    assert!(ford.conditions.is_empty());
    let (_, mira) = read.authored_entity("mira").unwrap();
    assert_eq!(mira.authored.as_ref(), draft.world.characters().get("mira"));
    assert_eq!(mira.place, isocosm::directing::PLACELESS);
    assert!(mira.body.is_none());
    assert!(mira.parts.is_empty());
    assert_eq!(read.state().laws["oath"], draft.world.laws()["oath"]);
    let event = &read.state().events["line:arrival"];
    assert_eq!(event.authored.as_ref(), draft.world.history().first());
    assert_eq!(event.strength, 0);
    assert_eq!(read.state().sites[&0].conditions["weather:storm"], -3);
    assert_eq!(prepared.private.secret("secret-oath"), Some(&draft.secrets[0]));
    let public = serde_json::to_string(&campaign.save()).unwrap();
    assert!(!public.contains("HOST PRIVATE OATH"));
    assert!(!public.contains("\"assertions\":"));
}

#[test]
fn native_founding_modes_save_replay_and_generation_provenance_agree() {
    let record = fixture::record();
    let before = record.clone();
    let a = Campaign::from_generation(fixture::genesis(), &record, 16, Execution::Individuals).unwrap();
    let b = Campaign::from_generation(fixture::genesis(), &record, 16, Execution::Grouped).unwrap();
    assert_eq!(record, before);
    assert_eq!(a.campaign.save(), b.campaign.save());
    let origin = a.campaign.origin().unwrap();
    assert_eq!(origin.id, record.id);
    assert_eq!(origin.request, record.request);
    assert_eq!(origin.entropy, record.entropy);
    let json = serde_json::to_vec(&a.campaign.save()).unwrap();
    let saved: CampaignSaved = serde_json::from_slice(&json).unwrap();
    let loaded = Campaign::load(saved, Execution::Grouped).unwrap();
    assert_eq!(loaded.save(), a.campaign.save());
    assert_eq!(loaded.reading().state_hash(), a.campaign.reading().state_hash());
    assert_eq!(loaded.content().maps, fixture::draft().maps);
    let note = isocosm_overlay::vtt::Assertion::Fact(isocosm_overlay::vtt::Fact {
        key: "revealed".into(), kind: "reveal".into(), about: None,
        text: "The keeper speaks.".into(), tags: vec!["river".into()],
    });
    let with_note = loaded.with_contract_assertions(&[note]).unwrap();
    assert_eq!(loaded.save(), a.campaign.save());
    assert!(with_note.reading().state().notes.iter().any(|n| n.core.object == "revealed" && n.core.kind == "reveal"));
    for kind in ["reveal", "narration", "history"] {
        assert!(loaded.reading().genesis().rules.note_kinds.contains(kind));
    }
    assert!(!std::str::from_utf8(&json).unwrap().contains("HOST PRIVATE OATH"));
}

#[test]
fn native_founding_refuses_bad_content_genesis_history_and_unknown_note_kind() {
    let mut draft = fixture::draft();
    draft.maps.push(draft.maps[0].clone());
    assert!(matches!(Campaign::from_draft(fixture::genesis(), &draft, Execution::Grouped), Err(FoundingError::Content(_))));
    let mut secrets = fixture::draft();
    let mut conflicting = secrets.secrets[0].clone();
    conflicting.text = "Different private oath".into();
    secrets.secrets.push(conflicting);
    assert!(matches!(Campaign::from_draft(fixture::genesis(), &secrets, Execution::Grouped), Err(FoundingError::Content(_))));
    let mut record = fixture::record();
    record.id.clear();
    assert_eq!(Campaign::from_generation(fixture::genesis(), &record, 16, Execution::Grouped).unwrap_err(),
        FoundingError::Content("generation record id is required".into()));
    record.id = "wrong-value".into();
    record.proposal = isometry_campaign::GenValue::Text { value: "text".into() };
    assert_eq!(Campaign::from_generation(fixture::genesis(), &record, 16, Execution::Grouped).unwrap_err(),
        FoundingError::Content("generation is not a campaign draft".into()));
    let mut g = fixture::genesis();
    g.rules.conditions.remove("weather:storm");
    let err = Campaign::from_draft(g, &fixture::draft(), Execution::Grouped).unwrap_err();
    assert_eq!(err, FoundingError::Native(isometry_campaign::native::ReplayError {
        phase: isometry_campaign::native::Phase::Founding, reason: "unknown site condition".into() }));
    let campaign = Campaign::from_draft(fixture::genesis(), &fixture::draft(), Execution::Grouped).unwrap().campaign;
    let before = campaign.save();
    let mut bad_content = before.clone();
    bad_content.content.starting_map = "absent-map".into();
    assert_eq!(Campaign::load(bad_content, Execution::Grouped).unwrap_err(),
        FoundingError::Content("missing starting map absent-map".into()));
    let bad = Assertion::Fact(Fact { key: "bad".into(), kind: "undeclared".into(), ..Default::default() });
    assert_eq!(campaign.with_assertions(&[bad]).unwrap_err().reason, "unknown note kind");
    assert_eq!(campaign.save(), before);
    let mut saved = before;
    saved.native.genesis.seed ^= 1;
    assert_eq!(Campaign::load(saved, Execution::Individuals).unwrap_err(), FoundingError::Native(
        isometry_campaign::native::ReplayError { phase: isometry_campaign::native::Phase::Load,
            reason: "save version or genesis digest mismatch".into() }));
}

#[test]
fn native_founding_forced_pack_is_atomic_and_keeps_its_native_record() {
    let campaign = Campaign::from_draft(fixture::genesis(), &fixture::draft(), Execution::Grouped).unwrap().campaign;
    let before = campaign.save();
    let applied = Applied { key: "river-extension".into(), source: "river-pack".into(),
        asserts: vec![Assertion::Place(Place { key: "dock".into(), name: "Dock".into(), ..Default::default() }),
            Assertion::Fact(Fact { key: "dock-note".into(), kind: "narration".into(), ..Default::default() })] };
    let accepted = campaign.force_pack(applied.clone()).unwrap();
    assert_eq!(campaign.save(), before);
    assert_eq!(accepted.reading().state().asserted.packs["river-extension"], applied);
    assert!(accepted.reading().authored_site("dock").is_some());
    let duplicate = accepted.force_pack(applied.clone()).unwrap();
    assert_eq!(duplicate.reading().state_hash(), accepted.reading().state_hash());
    let mut invalid = applied;
    invalid.key = "bad-extension".into();
    if let Assertion::Place(place) = &mut invalid.asserts[0] { place.key = "bad-dock".into(); }
    if let Assertion::Fact(fact) = &mut invalid.asserts[1] { fact.kind = "undeclared".into(); }
    assert_eq!(accepted.force_pack(invalid).unwrap_err().reason, "unknown note kind");
    assert!(accepted.reading().authored_site("bad-dock").is_none());
    let saved = serde_json::from_slice(&serde_json::to_vec(&accepted.save()).unwrap()).unwrap();
    assert_eq!(Campaign::load(saved, Execution::Individuals).unwrap().save(), accepted.save());
}
