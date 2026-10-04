// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Checkpoint 8, steps 8e and 8i: care (rulings 526 to 528 and 554). A
//! parent finds its own young by the child relation; a fed parent nurses
//! a hungry, unweaned young with milk from its provision; and a young is
//! weaned the first time it is fed, nursed no more.

use super::*;

/// Care finds its own young (526): an act requiring the child relation
/// is refused another grazer and accepted on the child, and a scheduled
/// pass of it finds the child for itself.
#[test]
fn a_parent_finds_its_own_young() {
    let (g, id) = world(1, 12);
    let mut s = session(g);
    run(&mut s, id, "test:brood");
    let (kid, _) = children(&s, id)[0];
    assert!(s.sim.state().relations.contains(&Relation {
        subject: id,
        kind: "sim:child".into(),
        object: kid,
    }));
    run(&mut s, id, "test:feed");
    let outcome = |s: &mut Session, process: &str, target: Id| -> String {
        let command = Command::Act {
            actor: id,
            target: Some(target),
            process: process.into(),
            cause: None,
        };
        let r = s.command_with_flows(command).unwrap();
        let v: serde_json::Value = serde_json::from_str(&r.result).unwrap();
        v["outcome"].to_string()
    };
    let state = s.sim.state();
    let other = state
        .population
        .groups
        .iter()
        .find(|(f, c)| c.entity.lineage == "lineage:1" && **f != id && **f != kid)
        .map(|(f, _)| *f)
        .unwrap();
    assert!(outcome(&mut s, "test:nurse", other).contains("Blocked"));
    assert_eq!(outcome(&mut s, "test:nurse", kid), "\"Accepted\"");
    let child = s.sim.state().population.get(kid).unwrap().clone();
    assert_eq!(held(&s, &child, "provision:1"), 6);
}

/// A scheduled pass of the act chooses its target by the selector and the
/// requirements together, so it finds the child; without the relation it
/// takes the first grazer at the site.
#[test]
fn a_scheduled_pass_finds_the_young_by_the_relation() {
    for young in [true, false] {
        let (mut g, id) = world(1, 12);
        let process = if young {
            "test:nurse"
        } else {
            "test:nurse-any"
        };
        g.rules.processes.get_mut(process).unwrap().period = Some(1);
        let mut s = session(g);
        run(&mut s, id, "test:brood");
        run(&mut s, id, "test:feed");
        let (kid, _) = children(&s, id)[0];
        let given = |s: &Session, who: Id| {
            let e = s.sim.state().population.get(who).unwrap().clone();
            held(s, &e, "provision:1")
        };
        let before = given(&s, kid);
        let _ = s.advance_tick_with_flows().unwrap();
        let fed = given(&s, kid) > before;
        assert_eq!(fed, young, "the child is fed only by the relation");
    }
}

/// Milk (527, 528, 551): a parent whose stores are full nurses its hungry
/// young from its provision, digested into the young's tissue as far as it
/// has room; a parent still filling its stores does not.
#[test]
fn a_fed_parent_nurses_its_hungry_young_and_a_hungry_one_does_not() {
    for fed in [true, false] {
        let (mut g, id) = world(1, 12);
        if fed {
            let rules = g.rules.clone();
            let e = g.population.lift(id).unwrap();
            let room = anatomy::room(e, &rules, "reserve:1");
            anatomy::give(e, &rules, "reserve:1", room)
                .unwrap()
                .unwrap();
        }
        let mut s = session(g);
        run(&mut s, id, "test:brood");
        let (kid, _) = children(&s, id)[0];
        run(&mut s, id, "test:feed");
        let of = |s: &Session, who: Id, key: &str| {
            let e = s.sim.state().population.get(who).unwrap().clone();
            held(s, &e, key)
        };
        let (young, kept) = (of(&s, kid, "tissue:1"), of(&s, id, "provision:1"));
        assert_eq!(kept, 12);
        let (outcome, _) = run_on(&mut s, id, Some(kid), "body:nurse-1");
        let given = kept - of(&s, id, "provision:1");
        assert_eq!(of(&s, kid, "tissue:1") - young, given, "fed {fed}");
        match fed {
            true => {
                assert_eq!(outcome, "\"Accepted\"");
                assert!(given > 0);
            },
            false => {
                assert!(outcome.contains("Blocked"), "{outcome}");
                assert_eq!(given, 0);
            },
        }
    }
}

/// Weaning (554): a brood's young are born unweaned; one is not weaned
/// while hungry, is weaned once its stores fill, and is nursed no more,
/// hungry again or not.
#[test]
fn a_young_is_weaned_the_first_time_it_is_fed() {
    let (mut g, id) = world(1, 12);
    let rules = g.rules.clone();
    let e = g.population.lift(id).unwrap();
    let room = anatomy::room(e, &rules, "reserve:1");
    anatomy::give(e, &rules, "reserve:1", room)
        .unwrap()
        .unwrap();
    let mut s = session(g);
    run(&mut s, id, "test:brood");
    let (kid, child) = children(&s, id)[0].clone();
    assert!(child.traits.contains(UNWEANED), "born unweaned");
    let unweaned = |s: &Session| {
        s.sim
            .state()
            .population
            .get(kid)
            .unwrap()
            .traits
            .contains(UNWEANED)
    };
    let (outcome, _) = run_on(&mut s, kid, None, "body:wean-1");
    assert!(
        outcome.contains("Blocked") && unweaned(&s),
        "hungry, so not weaned"
    );
    run(&mut s, kid, "test:fill");
    let (outcome, _) = run_on(&mut s, kid, None, "body:wean-1");
    assert_eq!(outcome, "\"Accepted\"");
    assert!(!unweaned(&s));
    // Hungry again after its rent, and the parent fed and provisioned.
    run(&mut s, kid, "body:upkeep-1");
    run(&mut s, id, "test:feed");
    let (outcome, _) = run_on(&mut s, id, Some(kid), "body:nurse-1");
    assert!(
        outcome.contains("Blocked"),
        "a weaned young is not nursed: {outcome}"
    );
}
