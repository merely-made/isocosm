// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Telling (case 1066): one act passes everything a teller knows about a
//! place, taken or refused by a seeded draw, and replays alike.

use cosmograph::{Command, Execution, Founding, Session, directing::Played, knowing::{Knowing, NOTE_KINDS,
    telling::{Telling, Topic}}, schema::Id};

fn world() -> Session {
    let mut genesis = Founding { seed: 11, sites: 3, population: 48, cohort_size: 1, lineages: 3, ecology: true,
        bodies: Some(Default::default()), played: Some(Played { lineage: 1, region_sites: 1 }), ..Default::default() }
        .generate().unwrap();
    for kind in NOTE_KINDS { genesis.rules.note_kinds.insert(kind.into()); }
    let mut session = Session::new(genesis, Execution::Individuals).unwrap();
    session.advance(25).unwrap();
    // Run until bodies at one site know the world differently, as a split
    // party does when it regroups: some heard of events the others missed.
    while candidates(&session).len() < 8 {
        assert!(session.sim.state().tick < 80, "no regrouped pair knows the world differently");
        session.advance(1).unwrap();
    }
    session
}

/// Every (teller, hearer, place) where the two share a site, and the teller
/// knows an event at the place that the hearer does not.
fn candidates(s: &Session) -> Vec<(Id, Id, Id, String)> {
    let state = s.sim.state();
    let living: Vec<(Id, Id)> = state.population.groups.iter()
        .filter(|(_, g)| g.count == 1 && g.entity.alive && !cosmograph::directing::is_participant(&g.entity))
        .filter_map(|(id, g)| g.entity.resident_site().map(|site| (*id, site))).collect();
    let mut out = Vec::new();
    for &(teller, site) in &living {
        for &(hearer, other) in &living {
            if teller == hearer || site != other { continue; }
            for (key, event) in &state.events {
                let Some(place) = event.place.site() else { continue };
                if s.sim.knows(teller, key).unwrap() && !s.sim.knows(hearer, key).unwrap() {
                    out.push((teller, hearer, place, key.clone()));
                    break;
                }
            }
        }
    }
    out
}

#[test]
fn a_teller_passes_everything_about_a_place_or_the_draw_refuses_and_both_replay() {
    let mut s = world();
    let found = candidates(&s);
    assert!(!found.is_empty(), "a regrouped pair knows the world differently");
    let (mut taken, mut refused) = (false, false);
    for (teller, hearer, place, event) in found {
        if taken && refused { break; }
        let outcome = s.command(Command::Know(Knowing::Tell { teller, hearer, topic: Topic::Place(place) })).unwrap();
        let telling: Telling = serde_json::from_str(&outcome).unwrap();
        assert!((50..=950).contains(&telling.chance));
        let knows = s.sim.knows(hearer, &event).unwrap();
        if telling.taken {
            assert!(knows && !telling.copied.is_empty(), "a taken telling copies what the teller knew");
            let view = cosmograph::directing::readings::view(&s.sim, hearer,
                cosmograph::directing::readings::Mode::Survival);
            assert!(view.events.contains(&event) && view.sites.contains(&place), "the hearer's view shows the place");
            taken = true;
        } else {
            assert!(!knows && telling.copied.is_empty(), "a refused telling copies nothing");
            refused = true;
        }
    }
    assert!(taken && refused, "the draw both takes and refuses over the candidates");
    // Telling someone at another site is refused before any draw.
    let state = s.sim.state();
    let apart = state.population.groups.iter().filter(|(_, g)| g.count == 1 && g.entity.alive)
        .filter_map(|(id, g)| g.entity.resident_site().map(|site| (*id, site))).collect::<Vec<_>>();
    let pair = apart.iter().flat_map(|a| apart.iter().map(move |b| (a, b))).find(|(a, b)| a.1 != b.1);
    if let Some((a, b)) = pair {
        assert!(s.command(Command::Know(Knowing::Tell { teller: a.0, hearer: b.0, topic: Topic::Place(a.1) })).is_err());
    }
    let replayed = Session::load(s.save(), Execution::Individuals).unwrap();
    assert_eq!(cosmograph::digest(&replayed.save()), cosmograph::digest(&s.save()), "tellings replay alike");
    assert_eq!(cosmograph::digest(replayed.sim.state()), cosmograph::digest(s.sim.state()));
}

/// The draw's numbers are the world's (1066): absent rules keep the numbers
/// telling was built with, and a world's own move the chance.
#[test]
fn the_draw_reads_the_worlds_telling_rules_with_todays_defaults() {
    assert_eq!(cosmograph::rules::Telling::default(), cosmograph::rules::Telling { base: 700, related: 150,
        per_caution: 2, caution_clamp: 100, floor: 50, ceiling: 950, place_notes: false, event_hops: 2 });
    let mut s = world();
    let (a, b, _, _) = candidates(&s)[0].clone();
    let default = s.sim.telling_chance(a, b).unwrap();
    let mut genesis = s.sim.genesis().clone();
    genesis.rules.telling = Some(cosmograph::rules::Telling { floor: 0, ceiling: 1000, base: 300, ..Default::default() });
    // Telling rules move no dynamics: the same world under them runs alike.
    let tick = s.sim.state().tick;
    s = Session::new(genesis, Execution::Individuals).unwrap();
    s.advance(tick).unwrap();
    assert_eq!(s.sim.telling_chance(a, b).unwrap(), default - 400, "the world's base moves the chance by its difference");
}

/// Where a world keeps place notes (1066), a place the teller has been to
/// passes as a place note, and the hearer's view shows it with no event of
/// its own telling it so.
#[test]
fn a_place_note_passes_a_place_known_only_by_having_been_there() {
    let mut genesis = cosmograph::Founding { seed: 5, sites: 8, population: 240, cohort_size: 1, lineages: 4,
        ecology: true, played: Some(Played { lineage: 1, region_sites: 2 }), ..Default::default() }.generate().unwrap();
    genesis.rules.note_kinds.insert(cosmograph::rules::PLACE_NOTE.into());
    genesis.rules.telling = Some(cosmograph::rules::Telling { place_notes: true, base: 1000, floor: 1000,
        ceiling: 1000, ..Default::default() });
    let mut s = Session::new(genesis, Execution::Individuals).unwrap();
    let pick = |s: &Session| {
        let st = s.sim.state();
        let living: Vec<(Id, Id)> = st.population.groups.iter()
            .filter(|(_, g)| g.count == 1 && g.entity.alive && !cosmograph::directing::is_participant(&g.entity))
            .filter_map(|(id, g)| g.entity.resident_site().map(|site| (*id, site))).collect();
        for &(teller, site) in &living { for &(hearer, other) in &living {
            if teller == hearer || site != other { continue; }
            let seen = cosmograph::directing::readings::view(&s.sim, hearer, cosmograph::directing::readings::Mode::Survival).sites;
            if let Some(v) = st.population.get(teller).unwrap().visits.iter().find(|v| !seen.contains(&v.place)) {
                return Some((teller, hearer, v.place));
            }
        } }
        None
    };
    while pick(&s).is_none() {
        assert!(s.sim.state().tick < 80, "no traveller meets one who has not been where it went");
        s.advance(1).unwrap();
    }
    let (teller, hearer, place) = pick(&s).unwrap();
    let outcome = s.command(Command::Know(Knowing::Tell { teller, hearer, topic: Topic::Place(place) })).unwrap();
    let telling: Telling = serde_json::from_str(&outcome).unwrap();
    assert!(telling.taken && telling.copied.contains(&format!("site:{place}")), "{telling:?}");
    let seen = cosmograph::directing::readings::view(&s.sim, hearer, cosmograph::directing::readings::Mode::Survival).sites;
    assert!(seen.contains(&place), "the hearer's view shows the told place");
    let replayed = Session::load(s.save(), Execution::Individuals).unwrap();
    assert_eq!(cosmograph::digest(replayed.sim.state()), cosmograph::digest(s.sim.state()));
}

/// Offered events are relevant to the hearer (1066): within the world's
/// hops of what it knows, nearest first, never one it already knows.
#[test]
fn tellable_events_are_unknown_to_the_hearer_and_near_what_it_knows() {
    let s = world();
    for (teller, hearer, _, _) in candidates(&s) {
        let offered = s.sim.tellable_events(teller, hearer, 2);
        assert!(offered.windows(2).all(|w| w[0].1 <= w[1].1), "nearest first");
        for (event, hop) in &offered {
            assert!((1..=2).contains(hop));
            assert!(s.sim.knows(teller, event).unwrap() && !s.sim.knows(hearer, event).unwrap());
        }
        let one: Vec<_> = s.sim.tellable_events(teller, hearer, 1).into_iter().map(|(e, _)| e).collect();
        assert!(one.iter().all(|e| offered.iter().any(|(o, _)| o == e)), "a deeper reach keeps the nearer offers");
        assert!(s.sim.tellable_events(teller, hearer, 0).is_empty());
    }
}
