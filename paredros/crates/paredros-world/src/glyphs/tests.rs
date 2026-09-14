// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use mesocosm_core::PartId;
use paredros_identity::{BodyRevisionId, Tick};

use super::*;
use crate::fixtures::session::{self, Fixture};
use crate::{GameIntent, ItemId, ItemKind, MotionInput, MotionRules};

/// The bound subject's rules: the shared fixture's demonstration canon, read
/// here as authored evidence rather than a world canon.
fn rules(subject: SubjectId) -> GlyphRules {
    session::demonstration_rules(subject)
}

/// The scripted run every test below reads: a motion step, a charged volley,
/// a dressing picked up and worn, the injury cut, and rest.
fn scripted() -> Fixture {
    let mut world = session::timed_action_world();
    world.step([1, 0, 0]);
    world.volley();
    let item = world.take_dressing();
    world.attach(item, PartId(3));
    world.injure();
    world.rest();
    world
}

/// Host verbs the shared fixture does not own: the glyph tests drive the
/// keeper the way the session host's keys do, through one open action.
impl Fixture {
    fn game(&self) -> &GameState {
        self.action.session().game()
    }

    /// The charged volley that severs the target's authored limb.
    fn volley(&mut self) {
        self.sever_target_limb();
    }

    /// The host's injury debug cut: a fall, then the reconciliation, applied
    /// as one batch so the open action repairs with it.
    fn injure(&mut self) {
        let subject = self.keeper;
        let old = self.game().bodies().get(subject).unwrap().revision;
        let first = self.game().next_tick();
        self.action
            .apply_game_batch(&[
                GameIntent::Fall {
                    tick: first,
                    subject,
                    distance: 5,
                },
                GameIntent::ReconcileAnatomy {
                    tick: Tick(first.0 + 1),
                    subject,
                    from_revision: old,
                    revision: BodyRevisionId(old.0 + 1),
                    severed_parts: vec![PartId(1)],
                },
            ])
            .expect("injury cut");
    }

    fn rest(&mut self) {
        self.action.rest().expect("rest");
    }

    /// The keeper takes the dressing under its feet, the host's `E` verb.
    fn take_dressing(&mut self) -> ItemId {
        let subject = self.keeper;
        let at = self.game().movement().position(subject).expect("position");
        let item = self
            .game()
            .items()
            .at(at)
            .find(|item| item.kind == ItemKind::Dressing)
            .map(|item| item.id)
            .expect("a dressing lies where the keeper stands");
        let tick = self.game().next_tick();
        self.action
            .apply_game_batch(&[GameIntent::Take {
                tick,
                subject,
                item,
            }])
            .expect("pickup");
        item
    }

    /// The keeper wears what it picked up, the equipment panel's verb.
    fn attach(&mut self, item: ItemId, part: PartId) {
        let subject = self.keeper;
        let revision = self.game().bodies().get(subject).unwrap().revision;
        let tick = self.game().next_tick();
        self.action
            .apply_game_batch(&[GameIntent::AttachItem {
                tick,
                subject,
                item,
                part,
                revision,
            }])
            .expect("attach");
    }

    /// One recorded fixed motion step, the host's movement key path.
    fn step(&mut self, toward: [i32; 3]) {
        let subject = self.keeper;
        let pose = self.game().movement().pose(subject).expect("pose");
        let revision = self.game().bodies().get(subject).unwrap().revision;
        let tick = self.game().next_tick();
        self.action
            .apply_game_batch(&[GameIntent::AdvanceMotion {
                tick,
                subject,
                revision,
                step: pose.step + 1,
                input: MotionInput {
                    move_x: (toward[0] * 32767) as i16,
                    move_z: (toward[2] * 32767) as i16,
                },
                rules: MotionRules::default(),
            }])
            .expect("motion step");
    }
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
    let reading = GlyphReading::new(rules(world.keeper), world.game()).unwrap();

    assert_eq!(
        acquired(&reading),
        [
            "paredros-fixture:step",
            "paredros-fixture:strike",
            "paredros-fixture:carry",
            "paredros-fixture:wear",
            "paredros-fixture:endure",
            "paredros-fixture:mend",
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
                record.glyph == acquisition.glyph && record.outcome == GlyphGrantOutcome::Acquired
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
    assert!(
        !reading.eligibility().complete,
        "the death glyph is unearned"
    );
    assert_eq!(reading.rules().subject, world.keeper);
}

#[test]
fn a_rebuilt_reading_from_a_reloaded_game_has_an_identical_journal() {
    let world = scripted();
    let first = GlyphReading::new(rules(world.keeper), world.game()).unwrap();
    let restored = GameState::restore(&world.game().save().unwrap()).unwrap();
    let second = GlyphReading::new(rules(world.keeper), &restored).unwrap();

    assert_eq!(first.journey().snapshot(), second.journey().snapshot());
    assert_eq!(first.records(), second.records());
}

#[test]
fn another_subject_over_the_same_history_reads_only_its_own_evidence() {
    let world = scripted();
    let keeper = GlyphReading::new(rules(world.keeper), world.game()).unwrap();
    let other = GlyphReading::new(rules(world.target), world.game()).unwrap();

    // The shared fixture gives the target its own accepted acts: it takes the
    // dressing and wears it. Nothing the keeper did reaches this reading.
    assert_eq!(
        acquired(&other),
        ["paredros-fixture:carry", "paredros-fixture:wear"]
    );
    for record in other.records() {
        assert_eq!(
            accepted(&world.game().events()[record.index]).map(|(_, subject)| subject),
            Some(world.target),
            "every record is the target's own evidence"
        );
        assert!(
            record
                .evidence
                .contains(&format!("subject/{}/", world.target.0))
        );
    }
    assert!(
        !other.journey().owns_base("paredros-fixture:strike"),
        "the keeper's volley is not the target's evidence"
    );
    assert!(keeper.journey().owns_base("paredros-fixture:strike"));
    assert!(!other.ended(), "the target survived the volley");
}

#[test]
fn a_reading_moves_no_world_fact() {
    let world = scripted();
    let before = world.game().state_hash().unwrap();
    let mut reading = GlyphReading::new(rules(world.keeper), world.game()).unwrap();
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
    let mut world = session::timed_action_world();
    let mut reading = GlyphReading::new(rules(world.keeper), world.game()).unwrap();
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
    assert!(reading.journey().owns_base("paredros-fixture:strike"));
}

#[test]
fn death_ends_the_reading_and_the_world_goes_on_without_it() {
    let world = session::timed_action_world();
    // A plain `GameState`, so the world can keep accepting events for another
    // subject after the bound one dies; a `Session` gates them behind control.
    let mut game = GameState::restore(&world.game().save().unwrap()).unwrap();
    let mut reading = GlyphReading::new(rules(world.keeper), &game).unwrap();

    let tick = game.next_tick();
    game.apply(GameIntent::Fall {
        tick,
        subject: world.keeper,
        distance: 40,
    })
    .unwrap();
    reading.advance(&game);

    assert!(reading.ended());
    assert!(reading.journey().owns_base("paredros-fixture:end"));
    let sealed = reading.records().len();
    let cursor = reading.cursor();
    let acquisitions = acquired(&reading);

    let at = game.movement().position(world.target).unwrap();
    for _ in 0..4 {
        let tick = game.next_tick();
        game.apply(GameIntent::Move {
            tick,
            subject: world.target,
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
    let world = session::timed_action_world();
    let mut unknown = rules(world.keeper);
    unknown.grants[0].glyph = "paredros-fixture:absent".into();
    assert!(GlyphReading::new(unknown, world.game()).is_err());

    let mut duplicated = rules(world.keeper);
    duplicated.grants[1].event = AcceptedKind::MotionAdvanced;
    assert!(GlyphReading::new(duplicated, world.game()).is_err());

    let mut none = rules(world.keeper);
    none.grants.clear();
    assert!(GlyphReading::new(none, world.game()).is_err());

    assert!(
        GlyphReading::new(rules(SubjectId(9_999)), world.game()).is_err(),
        "an unknown subject has no body to bind"
    );
}

#[test]
fn a_repeated_kind_records_without_a_second_acquisition() {
    let mut world = session::timed_action_world();
    for _ in 0..3 {
        world.step([1, 0, 0]);
    }
    let reading = GlyphReading::new(rules(world.keeper), world.game()).unwrap();
    assert_eq!(acquired(&reading), ["paredros-fixture:step"]);
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
        Some("paredros-fixture:step")
    ));
}
