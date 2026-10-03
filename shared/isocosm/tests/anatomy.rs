// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Checkpoint 7, matter in parts: a body's own matter in its parts' ledgers
//! (rulings 459 and 504), takes by what each part holds and gives by the
//! room each has (464), the reserve only in what stores (463), a bite drawn
//! by what each part holds (459), and a part's name read from its box and
//! the tree (494).

use isocosm::{
    anatomy,
    probe::BodyFounding,
    rules::{BodyRules, Rules},
    schema::*,
    simulation::Genesis,
};
use std::collections::BTreeMap;

fn world() -> Genesis {
    BodyFounding::default().generate().unwrap().genesis
}

/// The first grazer: a lump that takes in and stores, a limb or two and an
/// eye, its matter in them.
fn grazer(g: &Genesis) -> Entity {
    let groups = g.population.groups.values();
    groups
        .map(|c| &c.entity)
        .find(|e| e.lineage == "lineage:1")
        .unwrap()
        .clone()
}

fn matter(e: &Entity, key: &str) -> BTreeMap<Id, u64> {
    e.parts
        .iter()
        .map(|(id, p)| (*id, p.matter.get(key).copied().unwrap_or(0)))
        .collect()
}

#[test]
fn a_take_spreads_over_the_parts_by_what_each_holds() {
    let g = world();
    let mut e = grazer(&g);
    let before = matter(&e, "tissue:1");
    let total: u64 = before.values().sum();
    let taken = anatomy::take(&mut e, &g.rules, "tissue:1", total / 3)
        .unwrap()
        .unwrap();
    let after = matter(&e, "tissue:1");
    assert_eq!(taken.iter().map(|t| t.1).sum::<u64>(), total / 3);
    for (id, amount) in &taken {
        // Each part's exact share, floored, or one more for a remainder.
        let exact = u128::from(before[id]) * u128::from(total / 3) / u128::from(total);
        assert!(u128::from(*amount) - exact <= 1, "part {id}");
        assert_eq!(after[id], before[id] - amount);
    }
    // A take beyond what the parts hold is refused whole.
    let held = anatomy::held(&e, &g.rules, "tissue:1");
    let mut copy = e.clone();
    assert!(
        anatomy::take(&mut copy, &g.rules, "tissue:1", held + 1)
            .unwrap()
            .is_err()
    );
    assert_eq!(copy, e);
}

#[test]
fn a_give_fills_the_parts_by_their_room_and_none_overfills() {
    let g = world();
    let mut e = grazer(&g);
    let room = anatomy::room(&e, &g.rules, "tissue:1");
    assert!(room > 2);
    anatomy::give(&mut e, &g.rules, "tissue:1", room - 1)
        .unwrap()
        .unwrap();
    let b = BodyRules::default();
    for p in e.parts.values() {
        assert!(p.matter.get("tissue:1").copied().unwrap_or(0) <= anatomy::ceiling(p, b));
    }
    assert_eq!(anatomy::room(&e, &g.rules, "tissue:1"), 1);
    // The control: more than the room is refused.
    assert!(
        anatomy::give(&mut e, &g.rules, "tissue:1", 2)
            .unwrap()
            .is_err()
    );
}

#[test]
fn the_reserve_lives_only_in_what_stores() {
    let g = world();
    let e = grazer(&g);
    let b = BodyRules::default();
    let cells = |p: &Part| u64::from(p.cells.get("function:store").copied().unwrap_or(0));
    for p in e.parts.values() {
        let bound = cells(p) * anatomy::cell_mass(p, b);
        assert_eq!(anatomy::bound(p, &g.rules, "reserve:1"), bound);
        assert!(p.matter.get("reserve:1").copied().unwrap_or(0) <= bound);
    }
    // A producer stores nothing, so it can be given no reserve at all.
    let mut bodies = g.population.groups.values().map(|c| &c.entity);
    let mut producer = bodies.find(|e| e.lineage == "lineage:0").unwrap().clone();
    assert_eq!(anatomy::room(&producer, &g.rules, "reserve:0"), 0);
    assert!(
        anatomy::give(&mut producer, &g.rules, "reserve:0", 1)
            .unwrap()
            .is_err()
    );
}

#[test]
fn a_bite_lands_on_a_part_drawn_by_what_each_holds() {
    let mut e = grazer(&world());
    let held: Vec<u64> = (0..e.parts.len() as u64).map(|i| 10 * (i + 1)).collect();
    for (p, h) in e.parts.values_mut().zip(&held) {
        p.matter = BTreeMap::from([("tissue:1".into(), *h)]);
    }
    let of = ["tissue:1".to_string()];
    let total: u64 = held.iter().sum();
    let mut landed: BTreeMap<Id, u64> = BTreeMap::new();
    for draw in 0..total {
        *landed
            .entry(anatomy::bitten(&e, &of, draw).unwrap())
            .or_default() += 1;
    }
    // Every draw over one round of the total lands on each part as often as
    // it holds.
    let want: BTreeMap<Id, u64> = (0..).zip(held).collect();
    assert_eq!(landed, want);
    // A body holding none of what the meal names is not bitten.
    assert_eq!(anatomy::bitten(&e, &["tissue:9".into()], 3), None);
}

#[test]
fn a_part_is_named_by_its_box_and_the_tree() {
    let part = |half_extent, parent: Option<Id>| Part {
        half_extent,
        parent,
        ..Default::default()
    };
    let mut e = grazer(&world());
    e.parts = BTreeMap::from([
        (0, part([2, 2, 2], None)),
        (1, part([4, 1, 1], Some(0))),
        (2, part([1, 1, 1], Some(1))),
        (3, part([3, 1, 1], Some(2))),
        (4, part([3, 1, 1], Some(2))),
        (5, part([4, 4, 1], Some(0))),
    ]);
    let names: Vec<&str> = (0..6).map(|id| anatomy::name(&e, id).unwrap()).collect();
    assert_eq!(
        names,
        [
            "part-shape:lump",
            "part-shape:rod",
            // A point between a parent and children is a joint.
            "part-shape:joint",
            "part-shape:rod",
            "part-shape:rod",
            "part-shape:sheet",
        ]
    );
    // A rod carrying two is a branch; a declared hollow stands.
    e.parts.get_mut(&4).unwrap().parent = Some(3);
    e.parts.insert(6, part([1, 1, 1], Some(3)));
    assert_eq!(anatomy::name(&e, 3), Some("part-shape:branch"));
    e.parts.get_mut(&1).unwrap().shape = "part-shape:tube".into();
    assert_eq!(anatomy::name(&e, 1), Some("part-shape:tube"));
}

#[test]
fn a_declared_name_that_lies_and_matter_outside_the_parts_are_refused() {
    assert!(world().validate().is_ok(), "the control passes");
    let refused = |change: &dyn Fn(&mut Entity, &Rules)| {
        let mut g = world();
        let rules = g.rules.clone();
        let groups = g.population.groups.iter();
        let mut grazers = groups.filter(|(_, c)| c.entity.lineage == "lineage:1");
        let first = *grazers.next().unwrap().0;
        change(g.population.lift(first).unwrap(), &rules);
        g.validate().err().unwrap_or_default()
    };
    // A lump's box declared a sheet.
    let lie = refused(&|e, _| e.parts.get_mut(&0).unwrap().shape = "part-shape:sheet".into());
    assert!(lie.contains("reads as"), "{lie}");
    // Its own tissue moved out of its parts onto its ledger.
    let outside = refused(&|e, _| {
        let i = e.lineage.trim_start_matches("lineage:").to_string();
        let key = format!("tissue:{i}");
        let held: u64 = e
            .parts
            .values_mut()
            .filter_map(|p| p.matter.remove(&key))
            .sum();
        e.accounts.insert(key, held.max(1));
    });
    assert!(outside.contains("outside them"), "{outside}");
}
