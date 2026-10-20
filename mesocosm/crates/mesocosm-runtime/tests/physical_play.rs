// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use isocosm::{
    Execution, Session,
    rules::{Causation, Query},
    spatial::{self as native, embodiment as body},
};
use isometer_core::{Attachment, BodyDocument, VolumeRef, Yaw};
use mesocosm_runtime::{
    Founded,
    spatial::{Admission, Game},
};

fn founded() -> Founded {
    let mut founded = Founded::generated(16_003_851_315_348_448_882, 60).unwrap();
    let mut domain = founded.genesis.founding.clone().unwrap();
    domain.ecology_rates = isocosm::ecology::Rates::A1;
    let bodies = domain.bodies.as_mut().unwrap();
    bodies.roster = true;
    bodies.hazard = [0; 2];
    bodies.wound = [0; 2];
    founded.genesis = domain.generate().unwrap();
    // A small, provisioned native population keeps this acceptance control
    // about physical transitions rather than a large ecological boom.
    for group in founded.genesis.population.groups.values_mut() {
        if group.entity.body.is_none() {
            continue;
        }
        group.count = 1;
        let entity = std::sync::Arc::make_mut(&mut group.entity);
        if let Some((_, reserve)) = entity
            .accounts
            .iter_mut()
            .find(|(key, _)| key.starts_with("matter:"))
        {
            *reserve = 64;
        }
    }
    for site in founded.genesis.sites.values_mut() {
        for (key, value) in [
            ("terrain:elevation", 64),
            ("terrain:relief", 0),
            ("terrain:water", 0),
        ] {
            site.conditions.insert(key.into(), value);
        }
    }
    // Native choice still chooses: this control makes other choices unavailable.
    // Metabolism, natural reproduction and accepted spatial work remain native.
    for process in founded
        .genesis
        .rules
        .processes
        .values_mut()
        .filter(|p| p.causation == Causation::Choice)
    {
        process.requires.push(Query::Age { at_least: u64::MAX });
    }
    body::declare(&mut founded.genesis).unwrap();
    founded
}

fn source() -> (Session, Game) {
    let founded = founded();
    let mut session = Session::new(founded.genesis, Execution::Grouped).unwrap();
    let mut game = Game::default();
    game.found_generated(&mut session).unwrap();
    assert!(!game.members.is_empty());
    (session, game)
}

fn assert_placements(session: &Session, game: &Game) {
    let mut all: Vec<(u64, Vec<body::Box3>)> = Vec::new();
    for (&id, at) in &game.members {
        let member = session.sim.state().population.get(id).unwrap();
        if member.resident_site() != Some(at.site.0) {
            continue;
        }
        at.realized(&session.sim).unwrap();
        let document = member.body.as_ref().unwrap();
        let volume = session
            .sim
            .volume(at.site.0, at.window.min, at.window.max, at.window.depth)
            .unwrap();
        let pose = native::Pose {
            at: at.pose.at,
            yaw: at.pose.yaw,
        };
        body::check(&volume, document, &pose).unwrap();
        let boxes = body::boxes(document, &pose).unwrap();
        for (site, other) in &all {
            if *site == at.site.0 {
                assert!(!body::intersects(&boxes, other), "placed bodies overlap");
            }
        }
        all.push((at.site.0, boxes));
    }
}

#[test]
fn rotated_limbs_and_the_whole_window_are_checked_beyond_root_clearance() {
    let (session, game) = source();
    let at = game.members.values().next().unwrap();
    let volume = session
        .sim
        .volume(at.site.0, at.window.min, at.window.max, at.window.depth)
        .unwrap();
    let mut document = BodyDocument::new(VolumeRef::from_tag(0), [2; 3]);
    document
        .attach(
            VolumeRef::from_tag(0),
            [3, 1, 1],
            Attachment {
                parent: document.root,
                offset: [10, 0, 0],
                yaw: Yaw::Quarter,
            },
            None,
        )
        .unwrap();
    let good = body::Grounding::new(&volume)
        .at(
            &document,
            volume.max[0] - 3,
            (volume.min[1] + volume.max[1]) / 2,
            Yaw::Quarter,
        )
        .unwrap();
    body::check(&volume, &document, &good).unwrap();
    let mut buried = good.clone();
    buried.at[1] -= 1;
    assert!(
        body::air(&volume, buried.at),
        "root remains clear while a lower part penetrates"
    );
    assert!(body::check(&volume, &document, &buried).is_err());
    let turned = native::Pose {
        yaw: Yaw::Zero,
        ..good
    };
    assert!(
        body::check(&volume, &document, &turned).is_err(),
        "a rotated limb leaves the window"
    );
}

#[test]
fn native_walking_and_actual_children_share_checked_poses_and_replay() {
    let (mut session, mut game) = source();
    assert_placements(&session, &game);
    for _ in 0..18 {
        let _ = game.advance_clock(&mut session).unwrap();
        assert_placements(&session, &game);
    }
    let changes: Vec<_> = game
        .driver
        .as_ref()
        .unwrap()
        .records
        .iter()
        .flat_map(|r| &r.changes)
        .collect();
    assert!(
        changes
            .iter()
            .any(|c| matches!(c.admission, Admission::Step { .. })
                && c.before.as_ref().unwrap().pose.at != c.after.pose.at),
        "native walking must move an actual pose"
    );
    let children: Vec<_> = changes
        .iter()
        .filter_map(|c| matches!(c.admission, Admission::Birth { .. }).then_some(c.actor))
        .collect();
    assert!(
        !children.is_empty(),
        "natural biological children must acquire admitted physical poses"
    );
    assert!(
        changes
            .iter()
            .any(|c| children.contains(&c.actor) && matches!(c.admission, Admission::Step { .. })),
        "an actual newborn must receive future walking input"
    );
    game.validate_replay(&session).unwrap();
    let restored = Session::load_json(
        &serde_json::to_vec(&session.save()).unwrap(),
        Execution::Grouped,
    )
    .unwrap();
    assert_eq!(restored.sim.state_hash(), session.sim.state_hash());
    let mut saved: Game = serde_json::from_slice(&serde_json::to_vec(&game).unwrap()).unwrap();
    saved.validate_replay(&restored).unwrap();
    let record = saved
        .driver
        .as_mut()
        .unwrap()
        .records
        .iter_mut()
        .find(|r| !r.changes.is_empty())
        .unwrap();
    record.changes[0].after.pose.at[1] += 1;
    assert!(
        saved.validate_replay(&restored).is_err(),
        "invented saved poses must refuse"
    );
}

#[test]
fn generated_route_arrivals_keep_full_body_support_and_replay() {
    let mut founded = founded();
    let templates = founded.genesis.rules.spatial_templates.as_ref().unwrap();
    let (&from, stands) = templates
        .stands
        .iter()
        .find(|(_, stands)| !stands.is_empty())
        .unwrap();
    let traveller = stands[0].actor.unwrap();
    let route = founded.genesis.sites[&from].routes[0].clone();
    let key = format!("move:{from}-{}", route.to);
    founded
        .genesis
        .rules
        .processes
        .get_mut(&key)
        .unwrap()
        .requires
        .retain(|q| !matches!(q, Query::MoodBelow { .. } | Query::Age { .. }));
    let mut session = Session::new(founded.genesis, Execution::Grouped).unwrap();
    let mut game = Game::default();
    game.found_generated(&mut session).unwrap();
    session
        .command(isocosm::Command::Act {
            actor: traveller,
            target: None,
            process: key,
            cause: None,
        })
        .unwrap();
    for _ in 0..route.travel + 2 {
        let _ = game.advance_clock(&mut session).unwrap();
        if game.members[&traveller].site.0 == route.to {
            break;
        }
    }
    assert_eq!(game.members[&traveller].site.0, route.to);
    assert_placements(&session, &game);
    assert!(
        game.driver
            .as_ref()
            .unwrap()
            .records
            .iter()
            .flat_map(|r| &r.changes)
            .any(|c| c.actor == traveller && matches!(c.admission, Admission::Arrival))
    );
    game.validate_replay(&session).unwrap();
}

#[test]
fn a_refused_birth_keeps_its_biology_and_can_gain_a_pose_on_native_arrival() {
    let mut founded = founded();
    let (&from, _) = founded
        .genesis
        .rules
        .spatial_templates
        .as_ref()
        .unwrap()
        .stands
        .iter()
        .find(|(_, stands)| !stands.is_empty())
        .unwrap();
    let route = founded.genesis.sites[&from].routes[0].clone();
    let mut movement =
        founded.genesis.rules.processes[&format!("move:{from}-{}", route.to)].clone();
    movement.id = "test:move-refused-child".into();
    movement.period = None;
    movement
        .requires
        .retain(|q| !matches!(q, Query::MoodBelow { .. } | Query::Age { .. }));
    founded
        .genesis
        .rules
        .processes
        .insert(movement.id.clone(), movement);
    // Deliberately insufficient declared birth clearance: native biology must
    // still happen, with no physical pose manufactured for the refused child.
    for profile in founded.genesis.rules.birth_profiles.values_mut() {
        profile.clearance = [[-1; 3], [1; 3]];
    }
    let mut session = Session::new(founded.genesis, Execution::Grouped).unwrap();
    let mut game = Game::default();
    game.found_generated(&mut session).unwrap();
    let mut child = None;
    for _ in 0..18 {
        let _ = game.advance_clock(&mut session).unwrap();
        child = game
            .driver
            .as_ref()
            .unwrap()
            .records
            .iter()
            .flat_map(|r| &r.births)
            .find(|report| report.patch.is_err() && report.request.child_journey.is_none())
            .map(|report| report.request.child);
        if child.is_some() {
            break;
        }
    }
    let child = child.expect("an actual native birth was physically refused");
    assert!(session.sim.state().population.get(child).is_some());
    assert!(!game.members.contains_key(&child));
    assert!(game.driver.as_ref().unwrap().children.contains_key(&child));
    session
        .command(isocosm::Command::Act {
            actor: child,
            target: None,
            process: "test:move-refused-child".into(),
            cause: None,
        })
        .unwrap();
    for _ in 0..route.travel + 2 {
        let _ = game.advance_clock(&mut session).unwrap();
        if game
            .members
            .get(&child)
            .is_some_and(|at| at.site.0 == route.to)
        {
            break;
        }
    }
    let at = game
        .members
        .get(&child)
        .expect("native arrival admits a checked child pose");
    assert_eq!(at.site.0, route.to);
    at.realized(&session.sim).unwrap();
    game.placement_unoccupied(&session.sim, child).unwrap();
    game.validate_replay(&session).unwrap();
}
