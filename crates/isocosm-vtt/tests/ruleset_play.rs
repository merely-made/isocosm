#![cfg(feature = "endpoint")]
use chirograph::{IntentInvocation, IntentResult, ProjectionSnapshot};
use graphshell_endpoint::{IntentSink, ProjectionCatalog, ProjectionSource};
use isocosm::Execution;
use isocosm_overlay::Seat;
use isocosm_vtt::{
    SHEETS, TABLE, VttEndpoint,
    ruleset::{SECTION, State},
};
use isometry_system::active_effects::{self as effects, Records, Target};
use isonetry::family::{FamilyHost, FamilyReplica, SessionId};
#[path = "support/table.rs"]
mod table;

fn snapshot(endpoint: &mut VttEndpoint, session: &str) -> ProjectionSnapshot {
    let request = endpoint
        .describe()
        .projections
        .into_iter()
        .find(|offer| offer.request.session.0 == session)
        .unwrap()
        .request;
    endpoint.snapshot(request).unwrap()
}
fn request(
    endpoint: &mut VttEndpoint,
    session: &str,
    card: &str,
    action: &str,
) -> IntentInvocation {
    let snap = snapshot(endpoint, session);
    let target = snap
        .scene
        .active_items_in_order()
        .into_iter()
        .find(|(_, item)| {
            snap.scene.tables.sources[item.source.0 as usize]
                .as_ref()
                .unwrap()
                .id
                == card
        })
        .unwrap()
        .0;
    IntentInvocation {
        session: snap.session,
        target,
        observed_epoch: snap.scene.epoch,
        observed_revision: snap.scene.revision,
        intent: action.into(),
        payload: vec![],
    }
}
fn click(endpoint: &mut VttEndpoint, card: &str, action: &str) {
    let input = request(endpoint, SHEETS, card, action);
    assert_eq!(
        endpoint.invoke(input).unwrap(),
        IntentResult::Accepted,
        "{card} {action}"
    );
}
fn state(host: &FamilyHost) -> State {
    host.read(|_, sections| State::read(sections).unwrap().unwrap())
        .unwrap()
}
fn cast_setup(endpoint: &mut VttEndpoint, body: u64) -> String {
    let sheet = format!("sheet:{body}");
    click(endpoint, &sheet, "vtt.sheet-create");
    click(endpoint, &sheet, "vtt.sheet-learn");
    click(endpoint, &sheet, "vtt.sheet-give-longsword");
    let weapon = format!("weapon:{body}:ruleset:weapon:1");
    click(endpoint, &weapon, "vtt.weapon-equip");
    click(endpoint, &format!("slots:{body}:2"), "vtt.slot-2-1");
    weapon
}

#[test]
fn casting_attack_concentration_save_and_family_replay_use_one_authority() {
    let table::Table {
        world,
        dm,
        player,
        bodies,
    } = table::table_with_sites(1);
    let mut host = FamilyHost::new(SessionId([71; 16]), world).unwrap();
    let mut endpoint = VttEndpoint::open(host.world(), dm, Seat::Dm).unwrap();
    let checkpoint = host.checkpoint().unwrap();
    let mut replica = FamilyReplica::install(checkpoint, &[], Execution::Individuals).unwrap();
    host.drain_records();
    let weapon = cast_setup(&mut endpoint, bodies[0]);
    click(
        &mut endpoint,
        &format!("sheet:{}", bodies[1]),
        "vtt.sheet-create",
    );
    let before = state(&host);
    let cast = request(&mut endpoint, SHEETS, &weapon, "vtt.weapon-cast-2");
    assert_eq!(
        endpoint.invoke(cast.clone()).unwrap(),
        IntentResult::Accepted
    );
    let after = state(&host);
    assert_eq!(
        after.sheets[&isocosm_vtt::ruleset::token(bodies[0]).unwrap()].int("slot:2"),
        Some(0)
    );
    assert_ne!(before, after);
    assert!(matches!(
        endpoint.invoke(cast).unwrap(),
        IntentResult::Stale { .. }
    ));
    assert_eq!(state(&host), after, "retry spends neither slot nor dice");
    let map = after.map();
    let effect = Records::read(&map).unwrap().unwrap().effects[&1].clone();
    let Target::Item(item) = &effect.target else {
        panic!("weapon target")
    };
    assert_eq!(effects::weapon(&map, item).unwrap().attack_bonus, 1);
    click(
        &mut endpoint,
        &weapon,
        &format!("vtt.weapon-rule-attack-{}", bodies[1]),
    );
    let attack = state(&host).log.last().unwrap().clone();
    assert!(attack.note.contains("DM reach ruling"));
    assert_eq!(
        attack.rolls[0].total,
        i32::from(attack.rolls[0].dice[0]) + 3
    );
    for _ in 0..8 {
        let current = state(&host);
        if !effects::weapon(&current.map(), item).unwrap().magical {
            break;
        }
        click(
            &mut endpoint,
            &format!("sheet:{}", bodies[0]),
            "vtt.sheet-damage-1",
        );
    }
    let current = state(&host);
    assert!(!effects::weapon(&current.map(), item).unwrap().magical);
    assert!(
        current
            .log
            .iter()
            .flat_map(|entry| &entry.rolls)
            .any(|roll| roll.by.contains("concentration") && !roll.dice.is_empty())
    );

    let mut other = VttEndpoint::open(host.world(), player, Seat::Player).unwrap();
    let snap = snapshot(&mut other, SHEETS);
    let sources: Vec<_> = snap
        .scene
        .active_items_in_order()
        .iter()
        .map(|(_, item)| {
            snap.scene.tables.sources[item.source.0 as usize]
                .as_ref()
                .unwrap()
                .id
                .as_str()
        })
        .collect();
    assert!(
        !sources.contains(&format!("sheet:{}", bodies[0]).as_str()),
        "another player's private sheet is absent"
    );
    let checkpoint = host.checkpoint().unwrap();
    let records = host.drain_records();
    for record in records.iter().cloned() {
        replica.apply(record).unwrap();
    }
    assert_eq!(
        replica.snapshot().unwrap().save,
        host.world().save().unwrap()
    );
    let restored = FamilyReplica::install(checkpoint, &[], Execution::Individuals).unwrap();
    assert_eq!(
        restored.snapshot().unwrap().save.sections[SECTION],
        host.world().save().unwrap().sections[SECTION]
    );
}

#[test]
fn native_time_expires_the_spell_and_refusals_publish_no_prefix() {
    let table::Table {
        world, dm, bodies, ..
    } = table::table_with_sites(1);
    let host = FamilyHost::new(SessionId([72; 16]), world).unwrap();
    let mut endpoint = VttEndpoint::open(host.world(), dm, Seat::Dm).unwrap();
    let weapon = cast_setup(&mut endpoint, bodies[0]);
    click(&mut endpoint, &weapon, "vtt.weapon-cast-2");
    let before = (host.sequence(), host.world().save().unwrap(), state(&host));
    let bad = request(
        &mut endpoint,
        SHEETS,
        &format!("sheet:{}", bodies[0]),
        "vtt.sheet-damage-0",
    );
    assert!(matches!(
        endpoint.invoke(bad).unwrap(),
        IntentResult::Rejected { .. }
    ));
    assert_eq!(
        (host.sequence(), host.world().save().unwrap(), state(&host)),
        before
    );
    // Advance through the existing table consent/advance path, not a spell clock.
    let empty = request(&mut endpoint, TABLE, "roster", "vtt.roster-empty");
    assert_eq!(endpoint.invoke(empty).unwrap(), IntentResult::Accepted);
    for _ in 0..6 {
        let propose = request(&mut endpoint, TABLE, "time", "vtt.propose-10");
        assert_eq!(endpoint.invoke(propose).unwrap(), IntentResult::Accepted);
        let snap = snapshot(&mut endpoint, TABLE);
        let (target, action) = snap
            .scene
            .active_items_in_order()
            .into_iter()
            .find_map(|(id, _)| {
                snap.presentation
                    .offers_for(id)
                    .into_iter()
                    .flatten()
                    .flat_map(|offer| &offer.semantics.actions)
                    .find(|action| action.intent.0 == "vtt.advance")
                    .map(|a| (id, a.intent.0.clone()))
            })
            .expect("empty roster can advance");
        assert_eq!(
            endpoint
                .invoke(IntentInvocation {
                    session: snap.session,
                    target,
                    observed_epoch: snap.scene.epoch,
                    observed_revision: snap.scene.revision,
                    intent: action,
                    payload: vec![]
                })
                .unwrap(),
            IntentResult::Accepted
        );
    }
    let current = state(&host);
    let map = current.map();
    let records = Records::read(&map).unwrap().unwrap();
    assert_eq!(records.tick, 60);
    assert!(!records.effects[&1].applies(60));
    assert_eq!(records.effects[&1].ended, Some((60, effects::End::Expired)));
}

#[test]
fn a_player_can_cast_but_cannot_grant_slots_edit_sheets_or_override_reach() {
    let table::Table {
        world,
        dm,
        player,
        bodies,
    } = table::table_with_sites(1);
    let host = FamilyHost::new(SessionId([74; 16]), world).unwrap();
    let mut dm_endpoint = VttEndpoint::open(host.world(), dm, Seat::Dm).unwrap();
    let weapon = cast_setup(&mut dm_endpoint, bodies[1]);
    let mut endpoint = VttEndpoint::open(host.world(), player, Seat::Player).unwrap();
    let before = host.world().save().unwrap();
    let edit = request(
        &mut endpoint,
        SHEETS,
        &format!("sheet:{}", bodies[1]),
        "vtt.sheet-learn",
    );
    assert!(matches!(
        endpoint.invoke(edit).unwrap(),
        IntentResult::Rejected { .. }
    ));
    assert_eq!(host.world().save().unwrap(), before);
    click(&mut endpoint, &weapon, "vtt.weapon-cast-2");
    let current = state(&host);
    assert_eq!(
        current.sheets[&isocosm_vtt::ruleset::token(bodies[1]).unwrap()].int("slot:2"),
        Some(0)
    );
    assert_eq!(current.log.last().unwrap().by, player.0);
    let before = host.world().save().unwrap();
    let forged = request(
        &mut endpoint,
        SHEETS,
        &weapon,
        &format!("vtt.weapon-rule-attack-{}", bodies[0]),
    );
    assert!(matches!(
        endpoint.invoke(forged).unwrap(),
        IntentResult::Rejected { .. }
    ));
    assert_eq!(host.world().save().unwrap(), before);
    host.drain_records();
}

#[test]
fn measured_attacks_read_paid_native_poses_and_share_the_terrain_cache() {
    let table::Table {
        world, dm, bodies, ..
    } = table::table_with_native_geometry();
    let host = FamilyHost::new(SessionId([75; 16]), world).unwrap();
    let mut endpoint = VttEndpoint::open(host.world(), dm, Seat::Dm).unwrap();
    let weapon = cast_setup(&mut endpoint, bodies[0]);
    click(
        &mut endpoint,
        &format!("sheet:{}", bodies[1]),
        "vtt.sheet-create",
    );
    let span = host
        .read(|session, _| {
            let a = &session.sim.state().spatial_poses[&bodies[0]];
            let b = &session.sim.state().spatial_poses[&bodies[1]];
            for id in bodies {
                let body = session.sim.state().population.get(id).unwrap();
                let placed = &session.sim.state().spatial_poses[&id];
                assert!(
                    body.resident_site() == Some(placed.site)
                        && body.arrived == placed.arrived
                        && body.patch.is_some(),
                    "body {id}: site {:?}/{}, arrival {}/{}, patch {:?}, pose {:?}",
                    body.resident_site(),
                    placed.site,
                    body.arrived,
                    placed.arrived,
                    body.patch,
                    placed.pose
                );
            }
            assert_eq!(a.site, b.site);
            (0..3)
                .map(|axis| a.pose.at[axis].abs_diff(b.pose.at[axis]))
                .max()
                .unwrap()
        })
        .unwrap();
    let mut calibrate = request(
        &mut endpoint,
        isocosm_vtt::AUTHORING,
        "ruleset-scale",
        "vtt.ruleset-scale",
    );
    calibrate.payload =
        serde_json::to_vec(&serde_json::json!({"cells_per_square":span.max(1)})).unwrap();
    assert_eq!(endpoint.invoke(calibrate).unwrap(), IntentResult::Accepted);
    let before = state(&host);
    let reach = host
        .read(|session, sections| {
            isocosm_vtt::ruleset::measured_reach(
                &before,
                session,
                sections,
                isocosm_overlay::OrdinaryAccess {
                    participant: dm,
                    seat: Seat::Dm,
                    subject: session.sim.plays(dm.0).map(isocosm_overlay::EntityHandle),
                },
                bodies[0],
                bodies[1],
            )
        })
        .unwrap();
    assert!(reach.is_ok(), "paid native pair: {reach:?}");
    let first = snapshot(&mut endpoint, SHEETS);
    let measured = first.scene.active_items_in_order().iter().any(|(id, _)| {
        first
            .presentation
            .offers_for(*id)
            .into_iter()
            .flatten()
            .flat_map(|offer| &offer.semantics.actions)
            .any(|a| a.intent.0 == format!("vtt.weapon-attack-{}", bodies[1]))
    });
    assert!(
        measured,
        "the paid native pair is within measured clear reach"
    );
    assert_eq!(
        state(&host),
        before,
        "reading measured reach changes no edition state or dice"
    );
    click(
        &mut endpoint,
        &weapon,
        &format!("vtt.weapon-attack-{}", bodies[1]),
    );
    assert!(
        state(&host)
            .log
            .last()
            .unwrap()
            .note
            .contains("native root reach")
    );
    if span > 1 {
        let mut calibrate = request(
            &mut endpoint,
            isocosm_vtt::AUTHORING,
            "ruleset-scale",
            "vtt.ruleset-scale",
        );
        calibrate.payload = br#"{"cells_per_square":1}"#.to_vec();
        assert_eq!(endpoint.invoke(calibrate).unwrap(), IntentResult::Accepted);
        let before = host.world().save().unwrap();
        let refused = request(
            &mut endpoint,
            SHEETS,
            &weapon,
            &format!("vtt.weapon-attack-{}", bodies[1]),
        );
        assert!(matches!(
            endpoint.invoke(refused).unwrap(),
            IntentResult::Rejected { .. }
        ));
        assert_eq!(host.world().save().unwrap(), before);
    }
    host.drain_records();
}
