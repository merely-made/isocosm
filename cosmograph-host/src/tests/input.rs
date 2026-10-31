// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Live input through the core's driver (H4): a toy keymap over the fixture
//! world, the backlog cap and stale picks.

use super::*;
use crate::drive::{Button, Context, DEFAULT_MAX_BACKLOG, Driver, InputRefusal, Named, Pacing, Pick, Pointed, Press, Product};

#[derive(Clone, Debug, PartialEq)]
enum Toy { Join, Attend(Option<u64>), Look(u64) }

#[derive(Default)]
struct Keyed;

impl Product for Keyed {
    type Envelope = Toy;
    type Refusal = String;
    fn apply(&mut self, world: &World, envelope: &Toy) -> Result<String, String> {
        match envelope {
            Toy::Join => world.command(Command::Join).map(|_| "joined".into()),
            other => Ok(format!("{other:?}")),
        }
    }
    fn held(&self, _: &World) -> bool { false }
    fn answers(&self, _: &World, _: &Toy) -> bool { false }
    fn round(&mut self, world: &World) -> Result<(), String> { world.advance(1) }
    fn press(&self, _: &World, press: &Press, context: &Context) -> Option<Toy> {
        match press {
            Press::Char('j') => Some(Toy::Join),
            Press::Named(Named::Space) => Some(Toy::Attend(context.focus)),
            _ => None,
        }
    }
    fn point(&self, _: &World, pick: Pick, button: Button, _: &Context) -> Option<Toy> {
        match (pick, button) {
            (Pick::Thing(id), Button::Primary) => Some(Toy::Look(id)),
            _ => None,
        }
    }
}

fn driver() -> Driver<Keyed> { Driver::new(&fixture().world, Keyed, Pacing::Explicit) }

fn queued(driver: &Driver<Keyed>) -> Vec<Toy> { driver.queued().cloned().collect() }

#[test]
fn the_keymap_turns_claimed_presses_into_envelopes_and_leaves_the_rest() {
    let mut driver = driver();
    let context = Context { focus: Some(7), ..Context::default() };
    assert_eq!(driver.press(&Press::char('J'), &context), Ok(true), "lowercased");
    assert_eq!(driver.press(&Press::Named(Named::Space), &context), Ok(true));
    assert_eq!(driver.press(&Press::char('z'), &context), Ok(false), "unclaimed");
    assert_eq!(queued(&driver), [Toy::Join, Toy::Attend(Some(7))], "the context reaches the keymap");
    assert_eq!(driver.step(1), 1);
    assert_eq!(driver.trace()[0].1, Ok("joined".into()), "a press reaches the world");
}

#[test]
fn live_input_stops_at_the_backlog_cap_and_a_replay_does_not() {
    let mut driver = driver();
    assert_eq!(driver.max_backlog(), DEFAULT_MAX_BACKLOG);
    driver.set_max_backlog(2);
    let context = Context::default();
    assert_eq!(driver.press(&Press::char('j'), &context), Ok(true));
    assert_eq!(driver.offer(Toy::Join), Ok(()));
    assert_eq!(driver.press(&Press::char('j'), &context), Err(InputRefusal::Backlog { queued: 2 }));
    driver.queue(Toy::Join);
    assert_eq!(driver.queued_len(), 3, "a replay queues past the cap");
    driver.step(1);
    assert_eq!(driver.press(&Press::char('j'), &context), Ok(true), "drained, live input is taken again");
}

#[test]
fn a_pick_from_a_frame_the_world_has_moved_past_is_refused() {
    let mut driver = driver();
    let context = Context::default();
    let drawn = driver.world().revision().unwrap();
    let click = Pointed { pick: Pick::Thing(3), button: Button::Primary, revision: drawn };
    assert_eq!(driver.point(&click, &context), Ok(true));
    let menu = Pointed { button: Button::Secondary, ..click };
    assert_eq!(driver.point(&menu, &context), Ok(false), "the product makes nothing of it");
    assert_eq!(driver.current(&menu), Ok(Pick::Thing(3)), "but the window may act on the current pick");
    assert_eq!(queued(&driver), [Toy::Look(3)]);
    driver.step(1);
    let current = driver.world().revision().unwrap();
    assert!(current > drawn, "a round moves the revision");
    assert_eq!(driver.point(&click, &context), Err(InputRefusal::Stale { picked: drawn, current }));
    assert_eq!(driver.current(&menu), Err(InputRefusal::Stale { picked: drawn, current }));
    assert_eq!(driver.queued_len(), 0, "nothing stale is queued");
}
