// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use isocosm::{Command,Execution,Session,spatial as native};
use mesocosm_runtime::{Founded,spatial::{Admission,Game}};
#[path="foraging_play/fixtures.rs"]
mod fixtures;
use fixtures::*;

#[test]
fn a_native_food_step_crosses_a_window_boundary_and_restores() {
    let f=control(true,256,true);
    let mut g=f.session.sim.genesis().clone();
    let t=g.rules.spatial_templates.as_mut().unwrap();
    let before=t.stands.values().flatten().find(|s|s.actor==Some(f.actor)).unwrap();
    let delta=127-before.at[0];
    t.version=5; t.window.min=[0;2];
    t.window.max=[g.world.footprint.unwrap().side as i64;2];
    for s in t.stands.values_mut().flatten() {s.at[0]+=delta;}
    t.crossings.clear();
    let mut session=Session::new(g,Execution::Grouped).unwrap();
    let mut game=Game::default(); game.found_generated(&mut session).unwrap();
    let before=game.members[&f.actor].clone();
    let t=session.sim.genesis().rules.spatial_templates.as_ref().unwrap();
    assert_eq!(before.window,native::embodiment::site_window(t,before.pose.at));
    for _ in 0..4 { let _ = game.advance_clock(&mut session).unwrap(); }
    let transitions:Vec<_>=game.driver.as_ref().unwrap().records.iter().flat_map(|r|&r.changes)
        .filter(|c|c.actor==f.actor && matches!(c.admission,Admission::Step {..})).collect();
    assert!(!transitions.is_empty(),"native hunger must choose walking");
    assert!(transitions.iter().any(|c|c.before.as_ref().unwrap().window!=c.after.window),
        "ordinary physical walking must cross the paging boundary");
    for c in transitions {
        let a=c.before.as_ref().unwrap();let b=&c.after;
        assert!(a.pose.at[0].abs_diff(b.pose.at[0])<=1);
        assert!(a.pose.at[2].abs_diff(b.pose.at[2])<=1);
    }
    game.validate_replay(&session).unwrap();
    let restored=Session::load_json(&serde_json::to_vec(&session.save()).unwrap(),Execution::Grouped).unwrap();
    assert_eq!(restored.sim.state_hash(),session.sim.state_hash());
    let mut tampered=game.clone();tampered.members.get_mut(&f.actor).unwrap().window.min[0]+=1;
    assert!(tampered.validate_replay(&restored).is_err());
}

#[test]
fn full_site_founders_are_individual_and_distributed() {
    let mut f=ordinary(15301219037052331983);
    let mut domain=f.genesis.founding.clone().unwrap();
    let bodies=domain.bodies.as_mut().unwrap();
    bodies.roster=true;bodies.founding=Some("SpacedRoster".into());bodies.proportions=true;
    f.genesis=domain.generate().unwrap();
    native::embodiment::declare_site(&mut f.genesis).unwrap();
    let t=f.genesis.rules.spatial_templates.as_ref().unwrap();
    assert_eq!(t.window.min,[0,0]);
    assert_eq!(t.window.max,[f.genesis.world.footprint.unwrap().side as i64;2]);
    assert!(t.stands.values().map(Vec::len).sum::<usize>()>4);
    let mut session=Session::new(f.genesis,Execution::Grouped).unwrap();
    let mut game=Game::default();game.found_generated(&mut session).unwrap();
    let positions:Vec<_>=game.members.values().map(|p|p.pose.at).collect();
    assert!(positions.iter().map(|p|p[0]).max().unwrap()-positions.iter().map(|p|p[0]).min().unwrap()>256);
    for (&id,p) in &game.members {
        p.realized(&session.sim).unwrap();
        assert_eq!(session.sim.state().spatial_poses[&id].pose,p.pose);
        assert!(p.window.max[0]-p.window.min[0]<=256);
    }
    game.validate_replay(&session).unwrap();
}

fn site_control() -> (Session,Game,u64) {
    let f=control(true,256,true);
    let mut g=f.session.sim.genesis().clone();
    let t=g.rules.spatial_templates.as_mut().unwrap();
    t.version=5;t.window.min=[0;2];t.window.max=[g.world.footprint.unwrap().side as i64;2];
    t.crossings.clear();
    let mut session=Session::new(g,Execution::Grouped).unwrap();
    let mut game=Game::default();game.found_generated(&mut session).unwrap();
    (session,game,f.actor)
}

#[test]
fn actual_site_offspring_keep_native_poses_and_canonical_windows() {
    let (mut session,mut game,_)=site_control();
    for _ in 0..18 {let _=game.advance_clock(&mut session).unwrap();}
    let children:Vec<_>=game.driver.as_ref().unwrap().records.iter().flat_map(|r|&r.changes)
        .filter(|c|matches!(c.admission,Admission::Birth {..})).collect();
    assert!(!children.is_empty(),"actual native reproduction must place offspring");
    let t=session.sim.genesis().rules.spatial_templates.as_ref().unwrap();
    for c in children {
        let p=&game.members[&c.actor];
        assert_eq!(p.window,native::embodiment::site_window(t,p.pose.at));
        assert_eq!(session.sim.state().spatial_poses[&c.actor].pose,p.pose);
        p.realized(&session.sim).unwrap();
    }
    game.validate_replay(&session).unwrap();
}

#[test]
fn an_actual_journey_uses_site_arrival_placement_and_restores() {
    let (session,_,actor)=site_control();
    let from=session.sim.state().population.get(actor).unwrap().resident_site().unwrap();
    let mut g=session.sim.genesis().clone();
    let prefix=format!("move:{from}-");
    let mut process=g.rules.processes.values().find(|p|p.id.starts_with(&prefix)).unwrap().clone();
    process.id="test:site-journey".into();
    process.requires.retain(|q|!matches!(q,isocosm::rules::Query::MoodBelow {..}));
    g.rules.processes.insert(process.id.clone(),process);
    let mut session=Session::new(g,Execution::Grouped).unwrap();
    let mut game=Game::default();game.found_generated(&mut session).unwrap();
    session.command(Command::Act {actor,target:None,process:"test:site-journey".into(),cause:None})
        .unwrap();
    let to=session.sim.state().population.get(actor).unwrap().journey.as_ref().unwrap().to;
    for _ in 0..12 {
        let _=game.advance_clock(&mut session).unwrap();
        if session.sim.state().population.get(actor).unwrap().resident_site()==Some(to) {break;}
    }
    let p=&game.members[&actor];
    assert_eq!(p.site.0,to);
    let t=session.sim.genesis().rules.spatial_templates.as_ref().unwrap();
    assert_eq!(p.window,native::embodiment::site_window(t,p.pose.at));
    assert_eq!(session.sim.state().spatial_poses[&actor].pose,p.pose);
    p.realized(&session.sim).unwrap();
    game.validate_replay(&session).unwrap();
}
