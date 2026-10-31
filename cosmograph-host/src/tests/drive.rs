// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The core's driver over a toy product on the fixture world: envelopes
//! that join, refuse, ask and answer, and a round that advances one tick.

use super::*;
use crate::drive::{DEFAULT_MAX_STEPS, Driver, Pacing, Product};

#[derive(Clone, Debug, PartialEq)]
enum Toy { Join, Refuse, Ask, Answer }

#[derive(Default)]
struct Questions { asked: bool, rounds: u64, refuse_round: Option<u64> }

impl Product for Questions {
    type Envelope = Toy;
    type Refusal = String;
    fn apply(&mut self, world: &World, envelope: &Toy) -> Result<String, String> {
        match envelope {
            Toy::Join => world.command(Command::Join).map(|_| "joined".into()),
            Toy::Refuse => Err("refused".into()),
            Toy::Ask => { self.asked = true; Ok("asked".into()) },
            Toy::Answer if self.asked => { self.asked = false; Ok("answered".into()) },
            Toy::Answer => Err("unasked".into()),
        }
    }
    fn held(&self, _: &World) -> bool { self.asked }
    fn answers(&self, _: &World, envelope: &Toy) -> bool { *envelope == Toy::Answer }
    fn round(&mut self, world: &World) -> Result<(), String> {
        self.rounds += 1;
        if self.refuse_round == Some(self.rounds) { return Err("round refused".into()); }
        world.advance(1)
    }
}

fn driver(pacing: Pacing) -> Driver<Questions> {
    Driver::new(&fixture().world, Questions::default(), pacing)
}

fn tick(driver: &Driver<Questions>) -> u64 { driver.world().cut().unwrap().tick }

fn outcomes(driver: &Driver<Questions>) -> Vec<Result<String, String>> {
    driver.trace().iter().map(|(_, outcome)| outcome.clone()).collect()
}

#[test]
fn queued_envelopes_apply_in_order_before_one_round() {
    let mut driver = driver(Pacing::Explicit);
    let (start, entries) = (tick(&driver), driver.world().cut().unwrap().entries);
    for envelope in [Toy::Join, Toy::Refuse, Toy::Answer, Toy::Join] { driver.queue(envelope); }
    assert_eq!(driver.step(1), 1);
    assert_eq!(outcomes(&driver), [Ok("joined".into()), Err("refused".into()), Err("unasked".into()), Ok("joined".into())]);
    assert_eq!(driver.trace().iter().map(|(e, _)| e.clone()).collect::<Vec<_>>(), [Toy::Join, Toy::Refuse, Toy::Answer, Toy::Join]);
    assert_eq!((tick(&driver), driver.product().rounds, driver.queued_len()), (start + 1, 1, 0));
    assert_eq!(driver.world().cut().unwrap().entries, entries + 2, "the two accepted joins");
}

#[test]
fn the_clock_paces_rounds_under_a_ceiling_that_defers() {
    let mut driver = driver(Pacing::Clock { ticks_per_second: 30 }).with_max_steps(3);
    let start = tick(&driver);
    let mut calls = vec![driver.advance(1_000_000)];
    calls.extend((0..10).map(|_| driver.advance(0)));
    assert_eq!(calls, [3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 0]);
    assert_eq!(tick(&driver), start + 30);
    let mut default = self::driver(Pacing::Clock { ticks_per_second: 30 });
    assert_eq!(default.advance(1_000_000), DEFAULT_MAX_STEPS);
}

#[test]
fn an_explicit_driver_runs_rounds_only_when_stepped() {
    let mut driver = driver(Pacing::Explicit);
    let start = tick(&driver);
    assert!(driver.clock().is_none());
    assert_eq!(driver.advance(10_000_000), 0);
    assert_eq!(driver.step(2), 2);
    assert_eq!(tick(&driver), start + 2);
}

#[test]
fn a_question_holds_the_run_banks_no_time_and_drains_only_answers_in_front() {
    let mut driver = driver(Pacing::Clock { ticks_per_second: 30 });
    driver.queue(Toy::Ask);
    assert_eq!(driver.step(1), 1, "the asking step ends before its round");
    let start = tick(&driver);
    assert_eq!(driver.product().rounds, 0);
    assert_eq!((driver.step(3), driver.advance(5_000_000), tick(&driver)), (0, 0, start));
    driver.queue(Toy::Join);
    driver.queue(Toy::Answer);
    assert_eq!(driver.step(1), 0, "an answer behind another envelope waits with it");
    assert_eq!(driver.queued_len(), 2);
    driver.restore_queue([Toy::Answer, Toy::Join]);
    assert_eq!(driver.queued().cloned().collect::<Vec<_>>(), [Toy::Answer, Toy::Join]);
    assert_eq!(driver.advance(0), 0, "an answer drains in a held call, which runs no round");
    assert_eq!((driver.queued_len(), driver.product().asked), (1, false));
    assert_eq!(driver.advance(0), 0, "the held seconds were never banked");
    assert_eq!(driver.step(1), 1);
    assert_eq!((tick(&driver), driver.queued_len()), (start + 1, 0));
    assert_eq!(outcomes(&driver).last(), Some(&Ok("joined".into())));
}

#[test]
fn a_refused_round_stops_the_run_until_its_fault_is_cleared() {
    let mut driver = driver(Pacing::Explicit);
    driver.product_mut().refuse_round = Some(2);
    assert_eq!(driver.step(5), 1);
    assert_eq!(driver.fault(), Some("round refused"));
    assert_eq!(driver.step(1), 0);
    driver.restore_fault(None);
    assert_eq!(driver.step(1), 1);
}
