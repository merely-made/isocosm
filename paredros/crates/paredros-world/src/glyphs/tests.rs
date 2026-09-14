// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::fixture::{self, TARGET};
use super::*;
use crate::GameIntent;
use mesocosm_core::PartId;

/// The scripted run every test below reads: a motion step, a charged volley,
/// a dressing picked up and worn, the injury cut, and rest.
fn scripted() -> fixture::Fixture {
    let mut world = fixture::world();
    world.step([1, 0, 0]);
    world.volley();
    let item = world.take_dressing();
    world.attach(item, PartId(3));
    world.injure();
    world.rest();
    world
}

fn acquired(reading: &GlyphReading) -> Vec<String> {
    reading
        .journey()
        .acquisitions()
        .iter()
        .map(|acquisition| acquisition.glyph.clone())
        .collect()
}

#[test]
fn scripted_history_grants_in_acceptance_order_with_event_provenance() {
    let world = scripted();
    let reading = GlyphReading::new(fixture::rules(world.keeper), world.game()).unwrap();

    assert_eq!(
        acquired(&reading),
        [
            "paredros-test:step",
            "paredros-test:strike",
            "paredros-test:carry",
            "paredros-test:wear",
            "paredros-test:endure",
            "paredros-test:mend",
        ],
        "first-acquisition order is accepted-event order"
    );

    // Every acquisition names the event kind it came from, the exact accepted
    // event index, and that event's own tick.
    let events = world.game().events();
    for acquisition in reading.journey().acquisitions() {
        let record = reading
            .records()
            .iter()
            .find(|record| {
                record.glyph == acquisition.glyph
                    && record.outcome == GlyphGrantOutcome::Acquired
            })
            .expect("every acquisition has an evidence record");
        assert_eq!(
            acquisition.provenance.kind,
            ProvenanceKind::Custom(record.kind.label().to_owned())
        );
        assert_eq!(
            acquisition.provenance.evidence,
            format!(
                "paredros.session/subject/{}/events/{}",
                world.keeper.0, record.index
            )
        );
        assert_eq!(acquisition.tick, record.tick);
        assert_eq!(
            accepted(&events[record.index]).map(|(kind, subject)| (kind, subject)),
            Some((record.kind, world.keeper))
        );
    }

    assert!(!reading.ended());
    assert!(!reading.eligibility().complete, "the death glyph is unearned");
    assert_eq!(reading.rules().subject, world.keeper);
}

#[test]
fn a_rebuilt_reading_from_a_reloaded_game_has_an_identical_journal() {
    let world = scripted();
    let first = GlyphReading::new(fixture::rules(world.keeper), world.game()).unwrap();
    let restored = GameState::restore(&world.game().save().unwrap()).unwrap();
    let second = GlyphReading::new(fixture::rules(world.keeper), &restored).unwrap();

    assert_eq!(first.journey().snapshot(), second.journey().snapshot());
    assert_eq!(first.records(), second.records());
}

#[test]
fn another_subject_over_the_same_history_starts_an_empty_journal() {
    let world = scripted();
    let other = GlyphReading::new(fixture::rules(TARGET), world.game()).unwrap();

    assert!(other.journey().acquisitions().is_empty());
    assert!(other.records().is_empty());
    assert!(!other.ended(), "the target survived the volley");
    assert_eq!(other.journey().grants().len(), 0);
}

#[test]
fn a_reading_moves_no_world_fact() {
    let world = scripted();
    let before = world.game().state_hash().unwrap();
    let mut reading = GlyphReading::new(fixture::rules(world.keeper), world.game()).unwrap();
    reading.advance(world.game());
    assert_eq!(world.game().state_hash().unwrap(), before);
    assert_eq!(
        world.game().save().unwrap(),
        GameState::restore(&world.game().save().unwrap())
            .unwrap()
            .save()
            .unwrap(),
        "the reading is not in the save at all"
    );
}

#[test]
fn advance_consumes_only_what_arrived_since_the_cursor() {
    let mut world = fixture::world();
    let mut reading = GlyphReading::new(fixture::rules(world.keeper), world.game()).unwrap();
    let opened = reading.records().len();
    assert!(!reading.pending(world.game()));

    world.step([1, 0, 0]);
    assert!(reading.pending(world.game()));
    reading.advance(world.game());
    let after_step = reading.records().len();
    assert_eq!(after_step, opened + 1);
    assert_eq!(reading.cursor(), world.game().events().len());

    // A second advance with nothing new accepted changes nothing.
    reading.advance(world.game());
    assert_eq!(reading.records().len(), after_step);

    world.volley();
    reading.advance(world.game());
    assert!(
        reading
            .journey()
            .owns_base("paredros-test:strike")
    );
}

#[test]
fn death_ends_the_reading_and_the_world_goes_on_without_it() {
    let world = fixture::world();
    // A plain `GameState`, so the world can keep accepting events for another
    // subject after the bound one dies; a `Session` gates them behind control.
    let mut game = GameState::restore(&world.game().save().unwrap()).unwrap();
    let mut reading = GlyphReading::new(fixture::rules(world.keeper), &game).unwrap();

    let tick = game.next_tick();
    game.apply(GameIntent::Fall {
        tick,
        subject: world.keeper,
        distance: 40,
    })
    .unwrap();
    reading.advance(&game);

    assert!(reading.ended());
    assert!(reading.journey().owns_base("paredros-test:end"));
    let sealed = reading.records().len();
    let cursor = reading.cursor();
    let acquisitions = acquired(&reading);

    let at = game.movement().position(TARGET).unwrap();
    for _ in 0..4 {
        let tick = game.next_tick();
        game.apply(GameIntent::Move {
            tick,
            subject: TARGET,
            toward: [at[0] + 8, at[1], at[2]],
        })
        .unwrap();
    }
    reading.advance(&game);

    assert_eq!(reading.records().len(), sealed, "later events are ignored");
    assert_eq!(acquired(&reading), acquisitions);
    assert_eq!(reading.cursor(), cursor);
    assert!(cursor < game.events().len(), "the reading stopped short");
    assert!(!reading.pending(&game));
}

#[test]
fn rules_are_validated_before_any_history_is_read() {
    let world = fixture::world();
    let mut unknown = fixture::rules(world.keeper);
    unknown.grants[0].glyph = "paredros-test:absent".into();
    assert!(GlyphReading::new(unknown, world.game()).is_err());

    let mut duplicated = fixture::rules(world.keeper);
    duplicated.grants[1].event = AcceptedKind::MotionAdvanced;
    assert!(GlyphReading::new(duplicated, world.game()).is_err());

    let mut none = fixture::rules(world.keeper);
    none.grants.clear();
    assert!(GlyphReading::new(none, world.game()).is_err());

    assert!(
        GlyphReading::new(fixture::rules(SubjectId(9_999)), world.game()).is_err(),
        "an unknown subject has no body to bind"
    );
}

#[test]
fn a_repeated_kind_records_without_a_second_acquisition() {
    let mut world = fixture::world();
    for _ in 0..3 {
        world.step([1, 0, 0]);
    }
    let reading = GlyphReading::new(fixture::rules(world.keeper), world.game()).unwrap();
    assert_eq!(acquired(&reading), ["paredros-test:step"]);
    assert_eq!(reading.records().len(), 3);
    assert_eq!(
        reading
            .records()
            .iter()
            .filter(|record| record.outcome == GlyphGrantOutcome::Recorded)
            .count(),
        2,
        "later steps are evidence, not new acquisitions"
    );
    assert!(matches!(
        reading
            .journey()
            .grants()
            .first()
            .map(|grant| grant.glyph.as_str()),
        Some("paredros-test:step")
    ));
}
