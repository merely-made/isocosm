// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! What a holder reads of what it holds (cases 1216, 1217, 1223).

use super::{Tally, held, holdable, known};
use crate::directing::holding_tests::{STRAIN, WORLD_LINE, genesis, hold, join, keys, start, strain};
use crate::history::Command;

#[test]
fn a_held_surface_reads_its_matter_traits_and_revision() {
    let mut s = start(genesis(&["gene:rot"], None, None));
    let p = join(&mut s);
    hold(&mut s, p, STRAIN).unwrap();
    let read = held(&s.sim, STRAIN).unwrap();
    assert_eq!(read.sites, Tally { count: 1, total: 40 });
    assert_eq!(read.hosts, Tally { count: 1, total: 5 });
    assert_eq!(read.places, Tally::default());
    assert_eq!((read.transit, read.ended), (0, false));
    assert_eq!(read.traits, keys(&["gene:rot"]));
    let revision = read.revision;
    s.command(Command::Steer { participant: p, lineage: STRAIN.into(), on: keys(&["gene:leak"]), off: keys(&[]) }).unwrap();
    let read = held(&s.sim, STRAIN).unwrap();
    assert_eq!((read.revision, read.traits), (revision + 1, keys(&["gene:rot", "gene:leak"])));
    assert!(held(&s.sim, WORLD_LINE).is_none(), "a world-class lineage is no surface");
    assert!(held(&s.sim, "lineage:0").is_none());
}

#[test]
fn the_holdable_are_surfaces_with_matter_and_world_lineages_in_key_order() {
    let mut g = genesis(&[], None, None);
    let mut s = start(g.clone());
    assert_eq!(holdable(&s.sim), vec![STRAIN.to_string(), WORLD_LINE.to_string()]);
    // A registered surface with no matter anywhere is not offered.
    g.sites.get_mut(&0).unwrap().accounts.remove(STRAIN);
    g.population.lift(12).unwrap().accounts.remove(STRAIN);
    s = start(g);
    assert_eq!(holdable(&s.sim), vec![WORLD_LINE.to_string()]);
    assert!(s.sim.state().spreads.identities.contains(&strain()));
}

#[test]
fn a_held_lineage_is_known_to_its_holder_alone() {
    let mut s = start(genesis(&[], None, None));
    let (p, other) = (join(&mut s), join(&mut s));
    assert!(known(&s, p).is_empty());
    hold(&mut s, p, STRAIN).unwrap();
    hold(&mut s, p, WORLD_LINE).unwrap();
    assert_eq!(known(&s, p), keys(&[STRAIN, WORLD_LINE]));
    assert!(known(&s, other).is_empty());
}
