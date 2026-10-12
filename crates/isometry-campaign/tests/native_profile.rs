// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

#[path = "native_founding/fixture.rs"]
mod fixture;
use isocosm::{Execution, Founding};
use isometry_campaign::founding::{Campaign, FoundingError, table_genesis};
use isometry_campaign::native::Phase;

#[test]
fn native_profile_is_the_callers_world_and_keeps_pack_entropy_separate() {
    let profile = Founding {
        seed: 59, sites: 4, population: 16, lineages: 2,
        cohort_size: 4, ..Default::default()
    };
    let record = fixture::record();
    let mut saves = Vec::new();
    for mode in [Execution::Individuals, Execution::Grouped] {
        let prepared = Campaign::from_profile(&profile, &record, 16, mode).unwrap();
        let saved = prepared.campaign.save();
        assert_eq!(saved.native.genesis, table_genesis(profile.generate().unwrap()),
            "caller native founding was replaced");
        assert_eq!(saved.native.genesis.founding.as_ref(), Some(&profile));
        assert_eq!(saved.origin.as_ref().unwrap().entropy, u64::MAX);
        assert_eq!(saved.origin.as_ref().unwrap().request, record.request);
        assert_ne!(saved.native.genesis.seed, record.entropy);
        assert_eq!(Campaign::load(saved.clone(), mode).unwrap().save(), saved);
        saves.push(saved);
    }
    assert_eq!(saves[0], saves[1], "native profile execution modes diverged");
}

#[test]
fn native_profile_refusal_keeps_the_generator_reason_and_phase() {
    let profile = Founding { sites: 0, ..Default::default() };
    let expected = profile.generate().unwrap_err();
    let error = Campaign::from_profile(&profile, &fixture::record(), 16, Execution::Grouped)
        .unwrap_err();
    assert_eq!(error, FoundingError::Native(isometry_campaign::native::ReplayError {
        phase: Phase::Founding, reason: expected,
    }));
}