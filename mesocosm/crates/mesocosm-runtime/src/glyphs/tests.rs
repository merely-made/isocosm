// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::Trial;
use wing_glyphs::GlyphDefinition;

fn rules(organism: OrganismId, event: AcceptedKind) -> GlyphRules {
    GlyphRules {
        canon: CanonSpec {
            version: 1,
            limits: Default::default(),
            id: "mesocosm:test-canon".into(),
            revision: 1,
            glyphs: vec![
                GlyphDefinition {
                    id: "test:earth".into(),
                    display: "#".into(),
                    effect: "test:reshape".into(),
                },
                GlyphDefinition {
                    id: "test:passage".into(),
                    display: "/".into(),
                    effect: "test:traverse".into(),
                },
            ],
            variants: Vec::new(),
        },
        individual: "test-individual".into(),
        organism,
        unlock_thresholds: vec![10, 20],
        grants: vec![EventGrant {
            event,
            glyph: "test:earth".into(),
        }],
    }
}

fn carve_fixture() -> (World, [i32; 3]) {
    // Select an actual seeded founder standing above removable terrain. Do
    // not relocate it or synthesize history to make the grant happen.
    (0..32)
        .find_map(|seed| {
            let world = World::new(seed, 0);
            let position = world.controlled().unwrap().position;
            let at = [position[0], position[1] - 1, position[2]];
            (at[1] >= 1 && world.ground().solid(at) && world.in_reach(at)).then_some((world, at))
        })
        .expect("a seeded founder stands on removable terrain")
}

#[test]
fn opted_in_carve_grant_has_exact_core_provenance_and_replays() {
    let (world, at) = carve_fixture();
    let baseline_hash = state_hash(&world);
    let mut enabled = Trial::new(&world).unwrap();
    let mut ordinary = Trial::new(&world).unwrap();
    enabled
        .enable_glyphs(rules(world.controlled_id().unwrap(), AcceptedKind::Carved))
        .unwrap();
    assert!(ordinary.glyphs().is_none());
    assert_eq!(enabled.state_hash(), baseline_hash);
    for operation in 0..4 {
        let (a, b) = match operation {
            0 => (enabled.carve(at, 1), ordinary.carve(at, 1)),
            1 => (enabled.carve(at, 1), ordinary.carve(at, 1)),
            2 => (enabled.carve(at, 0), ordinary.carve(at, 0)),
            _ => (enabled.step(), ordinary.step()),
        };
        assert!(a && b);
        assert_eq!(enabled.state_hash(), ordinary.state_hash());
        assert_eq!(enabled.history(), ordinary.history());
        assert_eq!(enabled.trace(), ordinary.trace());
    }
    let reading = enabled.glyphs().unwrap();
    assert_eq!(
        reading.records().len(),
        1,
        "zero removal and rejection never grant"
    );
    let record = &reading.records()[0];
    assert_eq!(record.outcome, GlyphGrantOutcome::Acquired);
    let event = &enabled.history().log().entries()[record.sequence as usize];
    assert_eq!(record.tick, event.tick);
    assert_eq!(record.event, event.record);
    let acquisition = &reading.journey().acquisitions()[0];
    assert_eq!(acquisition.glyph, "test:earth");
    assert_eq!(acquisition.provenance.evidence, record.evidence);
    assert_eq!(acquisition.tick, record.tick);
    assert!(!reading.journey().eligibility().complete);
    let records = reading.records().to_vec();
    let snapshot = reading.journey().to_json().unwrap();
    let final_hash = enabled.state_hash();
    assert_eq!(reading.records(), records, "repeated reads do not grant");
    enabled.reset();
    assert_eq!(enabled.state_hash(), baseline_hash);
    assert!(enabled.glyphs().unwrap().records().is_empty());
    assert!(
        enabled
            .glyphs()
            .unwrap()
            .journey()
            .acquisitions()
            .is_empty()
    );
    assert!(enabled.carve(at, 1));
    assert!(enabled.carve(at, 1));
    assert!(enabled.carve(at, 0));
    assert!(enabled.step());
    assert_eq!(enabled.state_hash(), final_hash);
    assert_eq!(enabled.glyphs().unwrap().records(), records);
    assert_eq!(
        enabled.glyphs().unwrap().journey().to_json().unwrap(),
        snapshot
    );
    assert_eq!(state_hash(&world), baseline_hash);
}

#[test]
fn invalid_rules_and_late_enable_preserve_existing_configuration() {
    let (world, at) = carve_fixture();
    let valid = rules(world.controlled_id().unwrap(), AcceptedKind::Carved);
    let mut trial = Trial::new(&world).unwrap();
    trial.enable_glyphs(valid.clone()).unwrap();
    let mut invalid = Vec::new();
    let mut r = valid.clone();
    r.grants[0].glyph = "unknown".into();
    invalid.push(r);
    let mut r = valid.clone();
    r.grants.push(r.grants[0].clone());
    invalid.push(r);
    let mut r = valid.clone();
    r.grants.clear();
    invalid.push(r);
    let mut r = valid.clone();
    r.organism = OrganismId(u32::MAX);
    invalid.push(r);
    let mut r = valid.clone();
    r.canon.glyphs.clear();
    invalid.push(r);
    let mut r = valid.clone();
    r.individual.clear();
    invalid.push(r);
    for config in invalid {
        assert!(trial.enable_glyphs(config).is_err());
        assert_eq!(trial.glyphs().unwrap().rules().individual, valid.individual);
        assert_eq!(trial.state_hash(), state_hash(&world));
        assert!(trial.trace().is_empty());
    }
    assert!(trial.carve(at, 1));
    let records = trial.glyphs().unwrap().records().to_vec();
    assert!(trial.enable_glyphs(valid).is_err());
    assert_eq!(trial.glyphs().unwrap().records(), records);
}

#[test]
fn movement_and_feeding_rules_consume_actual_actor_events_only() {
    let world = World::new(42, 60);
    for kind in [AcceptedKind::Moved, AcceptedKind::Fed] {
        let mut scout = Trial::new(&world).unwrap();
        let organism = loop {
            assert!(
                scout.step(),
                "fixture must produce the requested accepted act"
            );
            if let Some((_, actor)) = scout
                .history()
                .log()
                .entries()
                .iter()
                .filter_map(|e| accepted(e.record))
                .find(|(event, actor)| {
                    *event == kind && world.organisms.iter().any(|o| o.id == *actor)
                })
            {
                break actor;
            }
        };
        let mut trial = Trial::new(&world).unwrap();
        trial.enable_glyphs(rules(organism, kind)).unwrap();
        while trial.step() {}
        let expected: Vec<_> = trial
            .history()
            .log()
            .entries()
            .iter()
            .enumerate()
            .filter(|(_, e)| accepted(e.record) == Some((kind, organism)))
            .collect();
        let records = trial.glyphs().unwrap().records();
        assert!(!records.is_empty());
        assert_eq!(records.len(), expected.len());
        for (record, (sequence, envelope)) in records.iter().zip(expected) {
            assert_eq!(record.sequence, sequence as u64);
            assert_eq!(record.tick, envelope.tick);
            assert_eq!(record.event, envelope.record);
            assert!(!matches!(record.outcome, GlyphGrantOutcome::Rejected(_)));
        }
        let records = records.to_vec();
        assert!(!trial.step());
        assert_eq!(trial.glyphs().unwrap().records(), records);
        assert_eq!(trial.glyphs().unwrap().journey().acquisitions().len(), 1);
    }
}

#[test]
fn changing_control_does_not_transfer_the_bound_collection() {
    use mesocosm_core::{Intent, Outcome};
    let (mut runtime, target, at) = (0..32)
        .find_map(|seed| {
            let runtime = crate::Runtime::new(seed, 60, 1);
            let original = runtime.world().controlled_id().unwrap();
            let target = runtime.world().organisms.iter().find(|body| {
                let at = [body.position[0], body.position[1] - 1, body.position[2]];
                body.id != original
                    && runtime.world().eligibility(body.id).is_ok()
                    && at[1] >= 1
                    && runtime.world().ground().solid(at)
            })?;
            let id = target.id;
            let at = [
                target.position[0],
                target.position[1] - 1,
                target.position[2],
            ];
            Some((runtime, id, at))
        })
        .expect("fixture has another eligible grounded body");
    let original = runtime.world().controlled_id().unwrap();
    let mut reading =
        GlyphReading::new(rules(original, AcceptedKind::Carved), runtime.world()).unwrap();
    runtime.queue(Intent::TakeControl { organism: target });
    assert_eq!(runtime.step(1), 1);
    assert!(
        matches!(runtime.last_outcomes(), [Outcome::Inhabited { organism }] if *organism == target)
    );
    reading.absorb(runtime.history(), 0, runtime.state_hash());
    let start = runtime.history().len();
    runtime.queue(Intent::Carve { at, radius: 1 });
    assert_eq!(runtime.step(1), 1);
    assert!(matches!(runtime.last_outcomes(), [Outcome::Carved { removed, .. }] if *removed > 0));
    reading.absorb(runtime.history(), start, runtime.state_hash());
    assert!(reading.records().is_empty());
    assert!(reading.journey().acquisitions().is_empty());
    assert_eq!(reading.journey().life(), 1);
    assert_eq!(reading.journey().experience(), 0);
    assert_eq!(reading.rules().organism, original);
}

#[test]
fn real_trial_acquisition_can_feed_a_standalone_shared_progression_demo() {
    let (world, at) = carve_fixture();
    let mut config = rules(world.controlled_id().unwrap(), AcceptedKind::Carved);
    config.canon.glyphs.truncate(1);
    config.unlock_thresholds = vec![10];
    let mut trial = Trial::new(&world).unwrap();
    trial.enable_glyphs(config).unwrap();
    assert!(trial.carve(at, 1));
    let hash = trial.state_hash();
    let reading = trial.glyphs().unwrap();
    // This clone demonstrates shared progression only. It is not a world
    // command, a reincarnated organism or an awarded simulation experience.
    let mut journey = Journey::from_json(&reading.journey().to_json().unwrap()).unwrap();
    assert!(journey.eligibility().can_wish && journey.eligibility().can_ascend);
    journey.ascend().unwrap();
    assert!(journey.is_divine());
    journey.add_experience(10).unwrap();
    assert_eq!(journey.unlocked().len(), 1);
    journey.reincarnate().unwrap();
    assert!(journey.unlocked().is_empty());
    assert_eq!(journey.acquisitions().len(), 1);
    journey.add_experience(9).unwrap();
    assert!(journey.unlocked().is_empty());
    journey.add_experience(1).unwrap();
    assert_eq!(journey.unlocked().len(), 1);
    let json = journey.to_json().unwrap();
    assert_eq!(Journey::from_json(&json).unwrap().to_json().unwrap(), json);
    println!("standalone-glyph-progression-receipt={json}");
    assert_eq!(trial.state_hash(), hash);
    assert!(!trial.glyphs().unwrap().journey().is_divine());
    assert_eq!(trial.glyphs().unwrap().journey().life(), 1);
}
