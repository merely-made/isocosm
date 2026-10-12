// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

fn take(world: &mut World, actor: Id) -> Id {
    let first = FirstLife { start: StartTime::Society, begins_as: LifeChoice::Denizen(EntityHandle(actor)) };
    world.submit(None, &EponymIntent::Checkpoint(LifeCheckpoint::First(first))).unwrap().joined.unwrap()
}

fn fatigue(actor: Id, second: Option<&str>) -> EponymIntent {
    let mut acts = vec![TimedAct { act: ActKey(founding::FATIGUE.into()), target: None }];
    if let Some(key) = second { acts.push(TimedAct { act: ActKey(key.into()), target: None }); }
    EponymIntent::Drive(Actuation { body: EntityHandle(actor), motion: None, acts })
}

#[test]
fn borrowed_and_owned_eponym_lower_the_same_real_native_act() {
    let (mut owned, actor, _) = world();
    let participant = take(&mut owned, actor);
    let mut native = owned.session().clone();
    let region = owned.volume().clone();
    let intent = fatigue(actor, None);
    let expected = owned.submit(Some(participant), &intent).unwrap();
    let accepted = Borrowed::new(&mut native, owned.site(), region.min, region.max, 8).unwrap()
        .submit(Some(participant), &intent).unwrap();
    assert_eq!(accepted, expected);
    assert_eq!(native.save(), owned.session().save(), "borrowed Eponym uses the owner's exact native lowering");
}

#[test]
fn borrowed_eponym_refuses_foreign_actor_before_native_publication() {
    let (mut owned, actor, other) = world();
    let participant = take(&mut owned, actor);
    let mut native = owned.session().clone();
    let before = native.save();
    let region = owned.volume().clone();
    let outcome = Borrowed::new(&mut native, owned.site(), region.min, region.max, 8).unwrap()
        .submit(Some(participant), &fatigue(other, None));
    assert_eq!(outcome, Err(WorldError::NotPlayed), "borrowed Eponym refuses another participant's body");
    assert_eq!(native.save(), before);
}

#[test]
fn borrowed_eponym_late_refusal_preserves_the_real_native_history_and_energy() {
    let (mut owned, actor, _) = world();
    let participant = take(&mut owned, actor);
    let mut native = owned.session().clone();
    let before = native.save();
    let region = owned.volume().clone();
    let outcome = Borrowed::new(&mut native, owned.site(), region.min, region.max, 8).unwrap()
        .submit(Some(participant), &fatigue(actor, Some("not-a-native-process")));
    assert!(outcome.is_err());
    assert_eq!(native.save(), before, "borrowed Eponym rolls back an accepted first act when its second act refuses");
}
