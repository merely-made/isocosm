// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The wound share and the need that reads it (the 2026-10-18 note to ruling
//! 227). The fixture's cells: root 8, limb 8, eye 1, seventeen in all.

use super::*;

fn wound(s: &mut Session, part: u32, cells: u32) {
    s.command(Command::Wound {
        entity: 1,
        part: Some(PartId(part)),
        cells,
    })
    .unwrap();
}

fn share(s: &Session) -> u64 {
    harm::wound_share(s.sim.state().population.get(1).unwrap())
}

/// The fixture with a mind whose one need is a wound: weight -10 from a
/// quarter of the cells.
fn minded(who: Binding) -> Genesis {
    let mut g = fixture(false);
    g.rules
        .accounts
        .insert("mind:strain".into(), AccountKind::Strain);
    g.rules.mind = Some(Mind {
        strain: "mind:strain".into(),
        needs: vec![Need {
            traits: BTreeSet::new(),
            query: Query::Wounded { who, at_least: 25 },
            weight: -10,
        }],
        bearing: 100,
        bearing_traits: BTreeMap::new(),
        rise: 0,
        rise_traits: BTreeMap::new(),
        stake: 0,
    });
    g
}

#[test]
fn the_share_counts_lost_cells_and_every_severed_lattice_over_all_cells() {
    let mut s = Session::new(fixture(false), Execution::Individuals).unwrap();
    assert_eq!(share(&s), 0);
    wound(&mut s, 0, 4);
    assert_eq!(share(&s), 23, "4 of 17 cells");
    wound(&mut s, 1, 8);
    assert_eq!(
        share(&s),
        76,
        "4 lost from the root, and the severed limb's 8 and eye's 1 whole"
    );
    let ground = s
        .sim
        .state()
        .population
        .groups
        .values()
        .find(|g| g.entity.lineage == "world:ground")
        .unwrap();
    assert_eq!(harm::wound_share(&ground.entity), 0, "no cells, no wound");
}

#[test]
fn a_need_reading_wounds_weighs_on_mood_from_a_quarter_lost() {
    let mut s = Session::new(minded(Binding::Actor), Execution::Individuals).unwrap();
    assert_eq!(s.sim.mood(1), Some(0));
    wound(&mut s, 0, 4);
    assert_eq!((share(&s), s.sim.mood(1)), (23, Some(0)), "under a quarter");
    wound(&mut s, 0, 1);
    assert_eq!((share(&s), s.sim.mood(1)), (29, Some(-10)), "past a quarter");
    let loaded = Session::load(s.save(), Execution::Grouped).unwrap();
    assert_eq!(loaded.sim.mood(1), Some(-10));
    assert_eq!(s.sim.mood(999), None, "an absent body has no mood");
}

#[test]
fn a_need_reads_only_its_own_member() {
    assert!(minded(Binding::Actor).validate().is_ok());
    let refused = minded(Binding::Target).validate().unwrap_err();
    assert!(refused.contains("a need reads only its member"), "{refused}");
}
