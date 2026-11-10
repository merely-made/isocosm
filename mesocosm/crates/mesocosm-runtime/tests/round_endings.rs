// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! How an epoch ends in the saved loop (lineage-rounds D7, D8; cases 1199,
//! 1212, 1217): capped, waiting for a call, or continuous, with a condition
//! (the forfeit is tested in `continuation::ending_tests`, which can answer a
//! held loss the replay would refuse after a dev kill). Every question reached here is also validated after a save
//! and restore (`Door::send`).

#[path = "round_endings/door.rs"]
mod door;

use cosmograph::directing::interim::Happening;
use cosmograph::directing::readings::Mode;
use door::{Door, EPOCH, door, settings, sheeted};
use mesocosm_runtime::continuation::LoopAction;
use mesocosm_runtime::{Occasion, Refusal, continuation::{LoopAnswer, LoopPhase, Rounds}};

fn boundaries(happenings: &[Happening]) -> usize {
    happenings.iter().filter(|h| matches!(h, Happening::Boundary { .. })).count()
}

fn epoch_hold(door: &Door) -> bool {
    door.saved().checkpoint.as_ref().is_some_and(|c| matches!(c.occasion, Occasion::Epoch(_)))
}

/// Steps until `stop` holds or `ticks` rounds pass, settling births and
/// losses; returns the boundaries seen and whether an epoch held the run.
fn run(door: &mut Door, ticks: u64, mut stop: impl FnMut(&Door) -> bool) -> (usize, bool) {
    let mut seen = 0;
    for _ in 0..ticks {
        if stop(door) { break; }
        seen += boundaries(&door.step());
        if epoch_hold(door) { return (seen, true); }
        door.settle();
    }
    (seen, false)
}

fn unasked<T: std::fmt::Debug>(result: Result<T, Refusal>) {
    assert!(matches!(result, Err(Refusal::Unasked)), "expected an Unasked refusal, got {result:?}");
}

#[test]
fn waiting_holds_past_spans_until_the_call_and_each_epoch_needs_its_own() {
    let mut d = door();
    d.configure(Rounds::Waiting, false);
    let start = d.tick();
    assert_eq!(d.saved().ending.since, start);
    unasked(d.answer(LoopAnswer::Call));
    let mut passed = 0;
    let (seen, held) = run(&mut d, 2 * EPOCH + 2, |d| {
        passed += d.saved().happenings.iter().filter(|h| matches!(h, Happening::SpanPassed { .. })).count();
        false
    });
    assert_eq!((seen, held), (0, false), "two spans pass with no boundary and no review");
    assert_eq!((passed, d.phase()), (1, LoopPhase::Running), "the span is reported once");
    assert!(!d.saved().ending.call);
    d.answer(LoopAnswer::Call).unwrap();
    assert!(d.reading().ending.call, "the reading shows the call");
    unasked(d.answer(LoopAnswer::Call));
    let happened = d.step();
    assert_eq!(boundaries(&happened), 1);
    assert!(epoch_hold(&d) && d.saved().review.is_some(), "the call opens the review");
    assert!(d.reading().checkpoint.is_some() && !d.reading().offers.is_empty());
    d.review().unwrap();
    let (ending, now) = (d.saved().ending, d.tick());
    assert_eq!((ending.since, ending.call, d.phase()), (now, false, LoopPhase::Running), "a fresh epoch");
    unasked(d.answer(LoopAnswer::Call));
    assert_eq!(run(&mut d, EPOCH + 2, |_| false), (0, false), "and it waits for its own call");
    d.answer(LoopAnswer::Call).unwrap();
}

#[test]
fn a_call_before_the_span_has_passed_is_refused() {
    let mut d = door();
    d.configure(Rounds::Waiting, false);
    run(&mut d, EPOCH - 3, |_| false);
    assert!(d.tick() - d.saved().ending.since < EPOCH);
    unasked(d.answer(LoopAnswer::Call));
    unasked(d.answer(LoopAnswer::Extend));
}

#[test]
fn continuous_adapts_every_span_but_opens_the_review_only_after_a_call() {
    let mut d = door();
    d.configure(Rounds::Continuous, false);
    let (seen, held) = run(&mut d, 2 * EPOCH + 2, |_| false);
    assert!(seen >= 2 && !held, "boundaries occur with no epoch hold ({seen})");
    assert!(d.saved().review.is_none() && d.saved().checkpoint.is_none());
    unasked(d.answer(LoopAnswer::Extend));
    d.answer(LoopAnswer::Call).unwrap();
    unasked(d.answer(LoopAnswer::Call));
    let (seen, held) = run(&mut d, EPOCH + 1, |_| false);
    assert_eq!((seen, held), (1, true), "after the call the next boundary holds");
    assert!(d.saved().review.is_some() && !d.reading().offers.is_empty());
    d.review().unwrap();
    assert!(!d.saved().ending.call);
    assert_eq!(run(&mut d, 2 * EPOCH + 2, |_| false).1, false, "and the next one is quiet again");
}

#[test]
fn an_extension_skips_one_capped_boundary_and_is_refused_while_pending() {
    let mut d = door();
    d.configure(Rounds::Capped, false);
    unasked(d.answer(LoopAnswer::Call));
    let first = d.tick() / EPOCH;
    d.answer(LoopAnswer::Extend).unwrap();
    unasked(d.answer(LoopAnswer::Extend));
    let (seen, held) = run(&mut d, EPOCH, |d| d.tick() / EPOCH > first);
    assert_eq!((seen, held, d.saved().ending.extend), (0, false, false), "the first crossing is skipped and spends it");
    let (seen, held) = run(&mut d, 2 * EPOCH, |_| false);
    assert_eq!((seen, held), (1, true), "the second crossing fires");
    let Occasion::Epoch(boundary) = &d.saved().checkpoint.as_ref().unwrap().occasion else { unreachable!() };
    assert_eq!(boundary.tick / EPOCH, first + 2);
}

#[test]
fn a_condition_sets_the_par_on_the_first_step_and_it_survives_a_restore() {
    let mut d = door();
    d.configure(Rounds::Capped, true);
    assert_eq!(d.saved().ending.par, None);
    d.step();
    let par = d.saved().ending.par;
    assert!(par.is_some());
    assert_eq!(d.reading().ending.par, par);
    assert_eq!(d.saved().round.as_ref().unwrap().ending.par, par, "the round source keeps what it ran under");
    assert_eq!(d.reloaded().unwrap().continuation.unwrap().ending.par, par);
    run(&mut d, 3, |_| false);
    assert_eq!(d.saved().ending.par, par, "one par for the epoch");
    // A default loop saves no ending at all.
    let mut plain = door();
    plain.configure(Rounds::Capped, false);
    plain.step();
    assert!(!serde_json::to_string(&plain.game).unwrap().contains("\"ending\""));
}

#[test]
fn with_the_draft_unset_an_epoch_review_is_as_before_in_both_modes() {
    for mode in [Mode::Survival, Mode::Creative] {
        let mut d = door();
        d.send(LoopAction::Configure(mesocosm_runtime::continuation::LoopSettings {
            mode, ..settings(Rounds::Capped, false) })).unwrap();
        let (_, held) = run(&mut d, 2 * EPOCH + 2, |_| false);
        assert!(held, "{mode:?} holds at an epoch");
        let review = d.saved().review.clone().expect("the hold carries its review");
        assert!(review.budget.is_none() && review.offers.iter().all(|o| o.cost.is_none()), "{mode:?}");
        let text = serde_json::to_string(&review).unwrap();
        assert!(!text.contains("\"cost\"") && !text.contains("\"budget\""), "{mode:?} saves as before");
        d.reloaded().expect("the saved review validates after a restore");
    }
}

#[test]
fn the_sheet_rule_is_off_by_default_and_a_sheeted_review_survives_a_restore() {
    // Unset, the reading carries no new keys.
    let mut plain = door();
    plain.configure(Rounds::Capped, false);
    let (_, held) = run(&mut plain, 2 * EPOCH + 2, |_| false);
    assert!(held);
    let reading = plain.reading();
    assert!(reading.sheet.is_none() && reading.offers.iter().all(|o| o.sheet.is_none() && o.cost.is_none()));
    let text = serde_json::to_string(&reading).unwrap();
    assert!(!text.contains("\"sheet\"") && !text.contains("\"cost\""), "{text}");
    // Set, the review and the reading carry it, and it validates after a restore.
    let mut d = sheeted();
    d.configure(Rounds::Capped, false);
    let (_, held) = run(&mut d, 2 * EPOCH + 2, |_| false);
    assert!(held, "the sheeted world holds at an epoch");
    let review = d.saved().review.clone().expect("the hold carries its review");
    // The fixture's lines have no recipe to grow, and a line with none has no sheet.
    assert!(review.sheet.is_none() && review.offers.iter().all(|o| o.sheet.is_none()));
    let reading = d.reading();
    assert_eq!(reading.sheet, review.sheet);
    assert!(reading.offers.iter().zip(&review.offers).all(|(r, o)| r.sheet == o.sheet && r.cost == o.cost));
    d.reloaded().expect("a saved review under the sheet rule validates after a restore");
    // A sheet, where there is one, is carried on the reading and survives the wire.
    use cosmograph::{lineage::sheet::Sheet, process::FeedingMode};
    let sheet = Sheet { mass: 9, speed: None, reach: 1, clutch: 1, trophic: None, feeding: FeedingMode::Grazer,
        functions: Default::default(), systems: Default::default(), upkeep: None, comfort: None, water: None, light: None };
    let mut carried = reading;
    carried.sheet = Some(sheet.clone());
    if let Some(offer) = carried.offers.first_mut() { (offer.sheet, offer.cost) = (Some(sheet), Some(3)); }
    let wire = serde_json::to_string(&carried).unwrap();
    assert!(wire.contains("\"sheet\""));
    assert_eq!(serde_json::from_str::<mesocosm_runtime::continuation::LoopReading>(&wire).unwrap(), carried);
}

/// An outsider of the played line arrives holding matter: a dev grant that
/// makes the live world hold more than its unsteered copy, which the loop's
/// own input door has no way to do in this static fixture. The saved native
/// cut is resealed to it, as a dev edit between rounds must be.
fn immigrate(d: &mut Door, amount: u64) {
    use cosmograph::{Command, arrival::Arrival, schema::Method};
    let played = d.session.sim.plays(d.participant).unwrap();
    let e = d.session.sim.state().population.get(played).unwrap();
    let (lineage, site) = (e.lineage.clone(), e.place);
    let account = d.session.sim.genesis().rules.accounts.iter().find(|(_, k)| matches!(k,
        cosmograph::rules::AccountKind::Matter { lineage: l, reserve: false, provision: false } if *l == lineage))
        .map(|(k, _)| k.clone()).unwrap();
    d.session.command(Command::Arrive(Arrival { lineage, site, method: Method::Reactive,
        accounts: [(account, amount)].into(), traits: Default::default(), skills: Default::default(),
        disposition: [0; 5], character: None })).unwrap();
    let saved = d.game.continuation.as_mut().unwrap();
    saved.native = cosmograph::digest(&(&d.session.branch, d.session.sim.genesis(), &d.session.entries,
        d.session.sim.state().tick));
    saved.native_entries = d.session.entries.len() as u64;
    saved.native_tick = d.session.sim.state().tick;
}

/// A round ended by its condition, end to end (case 1212; lineage-rounds D8).
/// The par is the real one, grown by the forward copy; it is beaten, not
/// planted, by a dev grant to the live lineage after the par is set (lever
/// (b): a dev `Arrive` of matter). The round waits with no call, so only the
/// condition can end it.
#[test]
fn a_met_condition_ends_the_round_and_holds_the_epoch_for_its_review() {
    let mut d = door();
    d.configure(Rounds::Waiting, true);
    d.step();
    let par = d.saved().ending.par.expect("the first step sets the par");
    let (_, held) = run(&mut d, 2, |_| false);
    assert!(!held && d.saved().ending.par == Some(par), "left to itself the lineage never beats its copy");
    let line = d.session.sim.state().population.get(d.session.sim.plays(d.participant).unwrap()).unwrap().lineage.clone();
    immigrate(&mut d, 40);
    assert!(cosmograph::lineage::boundary::standing(&d.session, &line).0 > par);
    assert!(!d.saved().ending.call && d.saved().settings.rounds == Rounds::Waiting && d.tick() - d.saved().ending.since < EPOCH,
        "no call and no span: only the condition is left to end it");
    let happened = d.step();
    let at = happened.iter().position(|h| matches!(h, Happening::ConditionMet { .. })).expect("the condition is met");
    assert!(matches!(&happened[at], Happening::ConditionMet { held, par: p, .. } if *p == par && *held > par));
    assert!(happened[at..].iter().any(|h| matches!(h, Happening::Boundary { .. })), "then the boundary");
    assert!(epoch_hold(&d) && d.saved().review.is_some() && d.phase() == LoopPhase::Held, "and the epoch is held");
    assert!(d.reading().checkpoint.is_some() && !d.reading().offers.is_empty());
    d.reloaded().expect("the held question validates after a save and a restore");
    assert_eq!(d.saved().ending.par, None, "the par is cleared at the boundary");
    d.review().unwrap();
    assert_eq!((d.phase(), d.saved().ending.par), (LoopPhase::Running, None));
    d.step();
    let fresh = d.saved().ending.par.expect("the next epoch's first step sets a fresh par");
    assert!(fresh > par, "grown from the world as it now stands, with the newcomer ({fresh} over {par})");
}
