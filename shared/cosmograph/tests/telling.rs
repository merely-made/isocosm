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
