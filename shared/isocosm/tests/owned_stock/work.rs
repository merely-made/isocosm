// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Work spends from the reservation; output retains the input claims (837).
use super::*;

#[test]
fn reserve_work_and_output_count_every_milligram_once() {
    let w = world(21);
    let [a, b, _] = w.bodies;
    let mut r = reserved(&w);
    let home = StockSource::Site(w.home);
    assert_eq!(timber(&r.s, home), SITE_MG - RECIPE_MG);
    assert_eq!(r.s.sim.project_matter(r.id).unwrap(), u128::from(RECIPE_MG));
    assert_eq!(held(&r.s, "lot:bench"), (RECIPE_MG, LotAt::Project(r.id)));
    exact(&r.s, r.mass);
    refused(&mut r.s, act(r.players[0], a, WORK, Some(r.id), &[]));
    grant(&mut r, &w, a);
    for step in 1..=2u64 {
        let out = acted(r.s.command(act(r.players[0], a, WORK, Some(r.id), &[])));
        assert_eq!((out.project, out.output), (Some(r.id), None));
        let project = r.s.sim.state().projects.get(r.id).unwrap();
        assert_eq!((project.progress, project.steps), (step, step));
        assert_eq!(project.reserved, Ledger::from([(TIMBER.into(), RECIPE_MG - step * STEP)]));
        assert_eq!(project.construction, Ledger::from([(TIMBER.into(), step * STEP)]));
        assert_eq!(r.s.sim.project_matter(r.id).unwrap(), u128::from(RECIPE_MG));
        exact(&r.s, r.mass);
    }
    let out = acted(r.s.command(act(r.players[0], a, WORK, Some(r.id), &[])));
    let made = out.output.expect("the final authored step completes the recipe");
    let project = r.s.sim.state().projects.get(r.id).unwrap();
    assert!(project.reserved.is_empty() && project.construction.is_empty());
    assert_eq!(r.s.sim.project_matter(r.id).unwrap(), 0);
    assert_eq!(timber(&r.s, StockSource::Entity(made)), RECIPE_MG);
    assert_eq!(held(&r.s, "lot:bench"), (RECIPE_MG, LotAt::Output(made)));
    let available: Vec<_> = r.s.sim.available_lots().map(|(at, lot)| (at, lot.owners.clone())).collect();
    assert_eq!(available, [(StockSource::Entity(made), BTreeSet::from([Owner::Entity(a), Owner::Entity(b)]))]);
    assert_eq!(r.s.sim.state().population.get(made).unwrap().resident_site(), Some(w.home));
    exact(&r.s, r.mass);
    refused(&mut r.s, act(r.players[0], a, WORK, Some(r.id), &[]));
}

#[test]
fn set_aside_work_keeps_its_reservation_while_the_actor_reserves_again() {
    let w = world(22);
    let [a, b, _] = w.bodies;
    let mut r = reserved(&w);
    grant(&mut r, &w, a);
    acted(r.s.command(act(r.players[0], a, WORK, Some(r.id), &[])));
    r.s.command(Command::PlaceMatter { site: w.home, account: TIMBER.into(), amount: RECIPE_MG }).unwrap();
    let mass = r.s.sim.matter();
    assert_lot(&mut r.s, lot("lot:next", StockSource::Site(w.home), RECIPE_MG, &[b])).unwrap();
    refused(&mut r.s, act(r.players[0], a, RESERVE, Some(r.id), &["lot:next"]));
    let first = r.s.sim.state().projects.get(r.id).unwrap().clone();
    let out = acted(r.s.command(act(r.players[0], a, RESERVE, None, &["lot:next"])));
    let second = out.project.unwrap();
    assert_ne!(second, r.id);
    assert_eq!(r.s.sim.state().projects.get(r.id), Some(&first), "interruption retains the same native project");
    assert_eq!(held(&r.s, "lot:bench").1, LotAt::Project(r.id));
    assert_eq!(held(&r.s, "lot:next").1, LotAt::Project(second));
    acted(r.s.command(act(r.players[0], a, WORK, Some(r.id), &[])));
    assert_eq!(r.s.sim.state().projects.get(r.id).unwrap().progress, 2);
    exact(&r.s, mass);
}
