// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::Founding;
use crate::directing::found::Played;

const PACE: Pace = Pace {
    round: 4,
    scoring: 6,
};

fn genesis(seed: u64) -> Genesis {
    let mut g = Founding {
        seed,
        sites: 5,
        population: 60,
        cohort_size: 4,
        lineages: 3,
        ecology: true,
        played: Some(Played {
            lineage: 1,
            region_sites: 2,
        }),
        ..Default::default()
    }
    .generate()
    .unwrap();
    // Three epochs inside a generated consumer's lifespan of 20 to 60
    // ticks, since it cannot feed up to a birth before age takes it.
    g.rules.epoch_ticks = 8;
    g
}

#[test]
fn three_epochs_end_to_end_replay_to_the_same_hash() {
    for seed in [101u64, 202, 303] {
        for mode in [Mode::Survival, Mode::Creative] {
            let start = Start {
                within: 0,
                epochs: 1,
            };
            let mut i =
                Interim::found(genesis(seed), start, mode, PACE, Execution::Individuals).unwrap();
            assert_eq!(
                i.handover.to_tick, 8,
                "seed {seed}: one added epoch of deep time"
            );
            let (mut boundaries, mut ended) = (0, false);
            for _ in 0..6 {
                let happened = i.round(&no_candidates).unwrap();
                boundaries += happened
                    .iter()
                    .filter(|h| matches!(h, Happening::Boundary { .. }))
                    .count();
                if happened
                    .iter()
                    .any(|h| matches!(h, Happening::LineageEnded { .. }))
                {
                    ended = true;
                    break;
                }
                let (v, all) = (i.view(), i.session.sim.state().sites.len());
                assert!(v.sites.len() <= all);
                if mode == Mode::Creative {
                    assert_eq!(v.sites.len(), all, "creative shows every site");
                }
            }
            let json = serde_json::to_vec(&i.session.save()).unwrap();
            let replayed = Session::load_json(&json, Execution::Individuals).unwrap();
            assert_eq!(replayed.sim.state_hash(), i.session.sim.state_hash());
            assert!(
                ended || boundaries == 3,
                "seed {seed}: {boundaries} boundaries"
            );
        }
    }
}

#[test]
fn death_hands_the_next_life_from_the_cohort() {
    let mut i = Interim::found(
        genesis(7),
        Start::default(),
        Mode::Creative,
        PACE,
        Execution::Individuals,
    )
    .unwrap();
    let first = i.critter().unwrap();
    // Planted death: the played critter's body turns to soil it holds,
    // its matter conserved, and it starves at once.
    let e = i.session.sim.state.population.lift(first).unwrap();
    let body = e.accounts.insert("matter:1-0".into(), 0).unwrap();
    *e.accounts.entry("world:soil".into()).or_default() += body;
    let happened = i.round(&no_candidates).unwrap();
    let died = happened.iter().find_map(|h| match h {
        Happening::Died { critter, next } => Some((*critter, *next)),
        _ => None,
    });
    let (dead, next) = died.expect("the starved critter died");
    assert_eq!(dead, first);
    let next = next.expect("the cohort had a next life");
    assert_eq!(i.critter(), Some(next));
    let lineage = |id| {
        i.session
            .sim
            .state()
            .population
            .get(id)
            .unwrap()
            .lineage
            .clone()
    };
    assert_eq!(lineage(next), lineage(first));
    // The bond was seeded from the forebear's (178).
    assert!(i.session.sim.bond(i.participant, next).is_some());
}

#[test]
fn a_collapse_happens_only_where_a_level_is_gone() {
    // Control: the founded world stands.
    let mut i = Interim::found(
        genesis(5),
        Start::default(),
        Mode::Survival,
        PACE,
        Execution::Individuals,
    )
    .unwrap();
    let standing = i.round(&no_candidates).unwrap();
    assert!(
        !standing
            .iter()
            .any(|h| matches!(h, Happening::Collapsed { .. }))
    );
    // Planted: every founded producer lies dead, so every region that
    // held producers has collapsed.
    let mut g = genesis(5);
    for c in g.population.groups.values_mut() {
        if c.entity.traits.contains("life:producer") {
            c.entity_mut().alive = false;
        }
    }
    let mut i = Interim::found(
        g,
        Start::default(),
        Mode::Survival,
        PACE,
        Execution::Individuals,
    )
    .unwrap();
    i.on_collapse = OnCollapse::Elsewhere;
    let held = |r: &Region| r.held.contains("life:producer");
    let regions = readings::regions(&i.session.sim);
    assert!(regions.iter().all(|r| r.collapsed == held(r)));
    let c = i.critter().unwrap();
    let place = i.session.sim.state().population.get(c).unwrap().place;
    let fell = held(readings::region_of(&regions, place).unwrap());
    let fallen = i.round(&no_candidates).unwrap();
    let collapsed = fallen.iter().find_map(|h| match h {
        Happening::Collapsed { moved_to, .. } => Some(*moved_to),
        _ => None,
    });
    assert_eq!(
        collapsed.is_some(),
        fell,
        "the played region fell, or stood"
    );
    // Play went on only in a region still standing (753).
    if let Some(Some(to)) = collapsed {
        let now = readings::regions(&i.session.sim);
        let at = i.session.sim.state().population.get(to).unwrap().place;
        assert!(!readings::region_of(&now, at).unwrap().collapsed);
    }
}

#[test]
fn the_boundary_scores_by_growing_a_copy_and_leaves_the_world_alone() {
    let i = Interim::found(
        genesis(9),
        Start::default(),
        Mode::Creative,
        PACE,
        Execution::Individuals,
    )
    .unwrap();
    let before = i.session.sim.state_hash();
    let site = *i.session.sim.state().sites.keys().next().unwrap();
    let feast = Candidate {
        name: "candidate:feast".into(),
        commands: vec![Command::PlaceMatter {
            site,
            account: "world:soil".into(),
            amount: 10_000,
        }],
    };
    let stay = Candidate {
        name: "candidate:stay".into(),
        commands: vec![],
    };
    let fed = boundary::score(&i.session, "lineage:0", &feast, 12).unwrap();
    let kept = boundary::score(&i.session, "lineage:0", &stay, 12).unwrap();
    assert_eq!(
        i.session.sim.state_hash(),
        before,
        "scoring touched the world"
    );
    assert!(!kept.beats(&kept));
    // Producers fed soil grow at least as much as without it.
    assert!(!kept.beats(&fed));
    let mut s = i.session.clone();
    let offer = |_: &Session, l: &str| {
        if l == "lineage:0" {
            vec![feast.clone()]
        } else {
            vec![]
        }
    };
    let turns = boundary::adapt(&mut s, Some("lineage:1"), &offer, 12).unwrap();
    assert_eq!(turns.len(), 1);
    assert_eq!(turns[0].considered.len(), 2);
}

/// The founded world (seed 7) with the sites in a line, 0-1-2-3-4, the
/// played critter (9) at site 0, and only the lineage members named in
/// `kin` (id, site) left alive beside it.
fn succession(kin: &[(Id, Id)]) -> Interim {
    let mut i = Interim::found(
        genesis(7),
        Start::default(),
        Mode::Creative,
        PACE,
        Execution::Individuals,
    )
    .unwrap();
    for (&id, site) in i.session.sim.state.sites.iter_mut() {
        site.routes.retain(|r| r.to.abs_diff(id) == 1);
    }
    let lineage = i.session.sim.state().population.get(9).unwrap().lineage.clone();
    let members: Vec<Id> = i.session.sim.state().population.groups.iter()
        .filter(|(_, g)| g.entity.lineage == lineage)
        .flat_map(|(&first, g)| first..first + g.count).filter(|&id| id != 9).collect();
    for id in members {
        let e = i.session.sim.state.population.lift(id).unwrap();
        match kin.iter().find(|(kept, _)| *kept == id) {
            Some((_, site)) => e.place = *site,
            None => e.alive = false,
        }
    }
    i
}

fn relate(i: &mut Interim, parent: Id, child: Id) {
    let relations = &mut i.session.sim.state.relations;
    relations.insert(Relation::new(child, "sim:parent", parent));
    relations.insert(Relation::new(parent, "sim:child", child));
}

/// The planted death of `death_hands_the_next_life_from_the_cohort`, one
/// round on: who died and who was handed on to.
fn die(i: &mut Interim) -> (Vec<Happening>, Option<Id>) {
    let first = i.critter().unwrap();
    let e = i.session.sim.state.population.lift(first).unwrap();
    let body = e.accounts.insert("matter:1-0".into(), 0).unwrap();
    *e.accounts.entry("world:soil".into()).or_default() += body;
    let happened = i.round(&no_candidates).unwrap();
    let next = happened.iter().find_map(|h| match h {
        Happening::Died { next, .. } => Some(*next),
        _ => None,
    });
    (happened, next.expect("the played critter died"))
}

#[test]
fn a_death_hands_on_to_a_living_child_before_a_same_site_stranger() {
    let mut i = succession(&[(33, 0), (21, 3)]);
    relate(&mut i, 9, 21);
    assert_eq!(readings::heirs(&i.session.sim, 9), [21, 33]);
    let (_, next) = die(&mut i);
    assert_eq!(next, Some(21));
    assert_eq!(i.critter(), Some(21));
}

#[test]
fn without_descendants_the_nearer_site_comes_first() {
    // Ids run against distance: 10 is two steps off, 33 one, 45 here.
    let mut i = succession(&[(10, 2), (33, 1), (45, 0)]);
    assert_eq!(readings::heirs(&i.session.sim, 9), [45, 33, 10]);
    // The round's ticks may move them; the life taken is the reading's first.
    let next = die(&mut i).1;
    assert_eq!(next, readings::heirs(&i.session.sim, 9).first().copied());
}

#[test]
fn a_far_member_is_an_heir_and_the_lineage_is_not_ended() {
    let mut i = succession(&[(53, 4)]);
    assert_eq!(readings::heirs(&i.session.sim, 9), [53]);
    let (happened, next) = die(&mut i);
    assert_eq!(next, Some(53));
    assert!(!happened.iter().any(|h| matches!(h, Happening::LineageEnded { .. })));
    // Where nobody lives, it is ended.
    let mut i = succession(&[]);
    let (happened, next) = die(&mut i);
    assert_eq!(next, None);
    assert!(happened.iter().any(|h| matches!(h, Happening::LineageEnded { .. })));
}

#[test]
fn unreachable_sites_and_unknown_kin_come_last() {
    let mut i = succession(&[(21, 3), (33, 1), (45, 1)]);
    i.session.sim.state.sites.get_mut(&3).unwrap().routes.clear();
    i.session.sim.state.sites.get_mut(&2).unwrap().routes.retain(|r| r.to != 3);
    relate(&mut i, 1000, 45);
    relate(&mut i, 1000, 9);
    // 33 and 45 tie at a step; 45 is a sibling, and the cut-off 21 is last.
    assert_eq!(readings::heirs(&i.session.sim, 9), [45, 33, 21]);
}

#[test]
fn heirs_read_the_same_from_a_replayed_run() {
    let mut i = Interim::found(genesis(7), Start::default(), Mode::Creative, PACE,
        Execution::Individuals).unwrap();
    for _ in 0..3 { i.round(&no_candidates).unwrap(); }
    let json = serde_json::to_vec(&i.session.save()).unwrap();
    let replayed = Session::load_json(&json, Execution::Individuals).unwrap();
    assert_eq!(replayed.sim.state_hash(), i.session.sim.state_hash());
    let lost = i.critter().unwrap();
    let live = readings::heirs(&i.session.sim, lost);
    assert!(!live.is_empty());
    assert_eq!(live, readings::heirs(&replayed.sim, lost));
}
