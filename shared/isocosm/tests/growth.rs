// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Checkpoint 8, step 8c: the provision and growth toward the recipe
//! (rulings 478, 479, 485, 495, 510 and 518). Reproduce cells hold a
//! provision as stores hold the reserve; a body lacks what its recipe
//! develops and it does not hold, nearest the root first, epimorphic bodies
//! keeping the segments they drew and anamorphic ones growing toward the
//! recipe; what was severed waits on healing; a part grows where
//! development puts it or at the first free seat; and growth pays PD2's
//! price from the reserve into the ground and fills the part from hand.

use isocosm::{
    Simulation, anatomy,
    development::{Soma, develop},
    growth::{lacking, price, seat},
    probe::BodyFounding,
    rules::*,
    schema::*,
    simulation::{Genesis, Outcome},
};
use std::collections::{BTreeMap, BTreeSet};

const SOIL: &str = "world:soil";

fn template(half_extent: [i32; 3], cells: &[(&str, u32)]) -> Template {
    Template {
        half_extent,
        cells: cells
            .iter()
            .map(|(f, n)| (format!("function:{f}"), *n))
            .collect(),
        shape: String::new(),
    }
}

/// Two lumps bearing a pair of limbs each; the lump reproduces in a cell.
fn recipe(anamorphic: bool) -> Development {
    let recipe = Recipe {
        tagmata: vec![Tagma {
            segments: 2,
            segment: "kind:lump".into(),
            bears: Some("kind:limb".into()),
            per_segment: 1,
            parent: None,
            anchor: Anchor::Tip,
            facing: Facing::Back,
            socket: Facing::Right,
            variance: None,
        }],
        variance: 0,
        absence: [0, 1],
    };
    Development {
        lexicon: recipe.kinds(),
        recipe,
        policy: Policy::default(),
        domain: 0,
        clutch: 1,
        anamorphic,
    }
}

fn world(anamorphic: bool) -> Genesis {
    let mut g = BodyFounding::default().generate().unwrap().genesis;
    g.rules.kinds = BTreeMap::from([
        (
            "kind:lump".into(),
            template([2, 2, 2], &[("intake", 5), ("store", 1), ("reproduce", 1)]),
        ),
        ("kind:limb".into(), template([3, 1, 1], &[("contract", 2)])),
    ]);
    g.rules.accounts.insert(
        "provision:1".into(),
        AccountKind::Matter {
            lineage: "lineage:1".into(),
            reserve: false,
            provision: true,
        },
    );
    g.lineages.get_mut("lineage:1").unwrap().development = Some(recipe(anamorphic));
    g
}

fn first_grazer(g: &Genesis) -> Id {
    let groups = g.population.groups.iter();
    let mut grazers = groups.filter(|(_, c)| c.entity.lineage == "lineage:1");
    *grazers.next().unwrap().0
}

/// A body developed from `drawn` segments, its tissue full and its reserve
/// `reserve`, lacking what `cut` names.
fn body(g: &Genesis, drawn: &[u8], absent: &[(u8, u8)], cut: &[[u8; 3]], reserve: u64) -> Entity {
    let d = recipe(false);
    let soma = Soma {
        segments: drawn.to_vec(),
        absent: absent.to_vec(),
    };
    let mut e = g.population.get(first_grazer(g)).unwrap().clone();
    e.parts = develop(&g.rules, &d, &soma).unwrap();
    e.parts.retain(|_, p| !cut.contains(&p.situs.unwrap()));
    e.soma = drawn.to_vec();
    e.accounts.clear();
    let room = anatomy::room(&e, &g.rules, "tissue:1");
    anatomy::give(&mut e, &g.rules, "tissue:1", room)
        .unwrap()
        .unwrap();
    anatomy::give(&mut e, &g.rules, "reserve:1", reserve)
        .unwrap()
        .unwrap();
    e
}

fn situs_of(e: &Entity, id: Id) -> [u8; 3] {
    e.parts[&id].situs.unwrap()
}

#[test]
fn the_provision_fills_only_what_reproduces() {
    let g = world(false);
    let mut e = body(&g, &[2], &[], &[], 0);
    let b = g.rules.body();
    for p in e.parts.values() {
        let cells = u64::from(p.cells.get("function:reproduce").copied().unwrap_or(0));
        assert_eq!(
            anatomy::bound(p, &g.rules, "provision:1"),
            cells * anatomy::cell_mass(p, b)
        );
    }
    // Two lumps of one reproducing cell, 12 mg each.
    let room = anatomy::room(&e, &g.rules, "provision:1");
    assert_eq!(room, 24);
    anatomy::give(&mut e, &g.rules, "provision:1", room)
        .unwrap()
        .unwrap();
    let limbs = e
        .parts
        .values()
        .filter(|p| p.functions.contains("function:contract"));
    assert!(
        limbs
            .into_iter()
            .all(|p| !p.matter.contains_key("provision:1"))
    );
    // The control: none beyond the reproduce cells' room.
    assert!(
        anatomy::give(&mut e, &g.rules, "provision:1", 1)
            .unwrap()
            .is_err()
    );
}

#[test]
fn a_body_lacks_what_its_recipe_develops_nearest_the_root_first() {
    let g = world(false);
    let whole = body(&g, &[2], &[], &[], 0);
    assert_eq!(lacking(&g.rules, &recipe(false), &whole).unwrap(), None);
    // Segment 0's left limb and segment 1's right: the nearer first, where
    // development puts it, on the part holding its segment.
    let cut = body(&g, &[2], &[], &[[0, 0, 2], [0, 1, 1]], 0);
    let next = lacking(&g.rules, &recipe(false), &cut).unwrap().unwrap();
    assert_eq!(next.situs, Some([0, 0, 2]));
    assert_eq!(situs_of(&cut, next.parent.unwrap()), [0, 0, 0]);
    assert_eq!(next.offset, [-5, 0, 0]);
    // A limb absent at development grows in later (478).
    let absent = body(&g, &[2], &[(0, 1)], &[], 0);
    let next = lacking(&g.rules, &recipe(false), &absent).unwrap().unwrap();
    assert_eq!(next.situs, Some([0, 1, 1]));
    // An epimorphic body keeps the one segment it drew; an anamorphic one
    // grows the recipe's second, flush behind (495).
    let short = body(&g, &[1], &[], &[], 0);
    assert_eq!(lacking(&g.rules, &recipe(false), &short).unwrap(), None);
    let grows = lacking(&g.rules, &recipe(true), &short).unwrap().unwrap();
    assert_eq!((grows.situs, grows.offset), (Some([0, 1, 0]), [0, 0, 4]));
    // What was severed is not regrown before healing (485).
    let mut severed = whole.clone();
    let limb = severed
        .parts
        .values_mut()
        .find(|p| p.situs == Some([0, 0, 1]));
    limb.unwrap().severed = true;
    assert_eq!(lacking(&g.rules, &recipe(false), &severed).unwrap(), None);
}

#[test]
fn a_part_takes_its_seat_or_the_first_free_facing() {
    let g = world(false);
    let mut e = body(&g, &[1], &[], &[[0, 0, 2]], 0);
    let policy = Policy::default();
    let limb = [3, 1, 1];
    // Development's seat, left of the lump, is free.
    assert_eq!(
        seat(&e, &policy, 0, limb, Some([-5, 0, 0])),
        Some([-5, 0, 0])
    );
    // Taken by a part from elsewhere, the rod tries right (taken), left
    // (taken), then front, flush ahead of the lump.
    e.parts.insert(
        9,
        Part {
            parent: Some(0),
            half_extent: limb,
            offset: [-5, 0, 0],
            ..Default::default()
        },
    );
    assert_eq!(
        seat(&e, &policy, 0, limb, Some([-5, 0, 0])),
        Some([0, 0, -3])
    );
    // The control: with that taken too, no seat within the tolerance.
    e.parts.insert(
        10,
        Part {
            parent: Some(0),
            half_extent: limb,
            offset: [0, 0, -3],
            ..Default::default()
        },
    );
    assert_eq!(seat(&e, &policy, 0, limb, Some([-5, 0, 0])), None);
}

fn grow_act() -> Process {
    Process {
        id: "test:grow".into(),
        causation: Causation::Choice,
        requires: vec![Query::Alive(Binding::Actor)],
        commitments: vec![],
        effects: vec![Effect::Grow {
            from: SOIL.into(),
            into: SOIL.into(),
            conversion: Conversion::Synthesis,
        }],
        risk: None,
        target: None,
        period: None,
        priority: 0,
        need_account: None,
        need_below: 0,
        glyphs: BTreeSet::new(),
        invariants: BTreeSet::new(),
        note: false,
    }
}

/// Grows the cut grazer from `hand` of soil with `reserve` in store and
/// `taken` of its tissue gone; returns the simulation, its id and the
/// world's matter before.
fn grown(hand: u64, reserve: u64, taken: u64) -> (Simulation, Id, u128) {
    let mut g = world(false);
    g.rules.processes.insert("test:grow".into(), grow_act());
    let id = first_grazer(&g);
    let mut e = body(&g, &[1], &[], &[[0, 0, 2]], reserve);
    anatomy::take(&mut e, &g.rules, "tissue:1", taken)
        .unwrap()
        .unwrap();
    e.accounts.insert(SOIL.into(), hand);
    *g.population.lift(id).unwrap() = e;
    let mut sim = Simulation::new(g, isocosm::Execution::Individuals).unwrap();
    let before = sim.matter();
    let r = sim.execute(id, None, "test:grow", None);
    assert_eq!(r.outcome, Outcome::Accepted, "{r:?}");
    (sim, id, before)
}

fn grew(sim: &Simulation, id: Id) -> bool {
    let e = sim.state().population.get(id).unwrap();
    e.parts.values().any(|p| p.situs == Some([0, 0, 2]))
}

#[test]
fn growth_pays_pd2s_price_from_the_reserve_and_fills_the_part_from_hand() {
    let (sim, id, before) = grown(30, 12, 0);
    let rules = &sim.genesis().rules;
    let e = sim.state().population.get(id).unwrap();
    let limb = e
        .parts
        .values()
        .find(|p| p.situs == Some([0, 0, 2]))
        .unwrap();
    // A limb of 63 voxels weighs 50 mg in 2 cells: its price is 50 mg,
    // the reserve's 12 and then 38 of tissue, into the ground as soil.
    assert_eq!(price(rules, limb), 50);
    assert_eq!(anatomy::held(e, rules, "reserve:1"), 0);
    // The hand of 30 fills the room the price and the new limb opened.
    assert_eq!(e.accounts.get(SOIL).copied().unwrap_or(0), 0);
    assert!(limb.matter.get("tissue:1").copied().unwrap_or(0) > 0);
    assert_eq!(sim.matter(), before, "no matter made or lost");
    // The controls: nothing grows from an empty hand, in a body with room
    // left in its parts, or one whose reserve and tissue cannot pay.
    assert!(!grew(&grown(0, 12, 0).0, id));
    assert!(!grew(&grown(30, 12, 1).0, id));
    let (sim, id, _) = grown(30, 0, 0);
    assert!(grew(&sim, id), "tissue alone pays, as a lean body does");
}

/// The control the other cannot reach: a part whose price is more than
/// the body's reserve and tissue together is not grown.
#[test]
fn a_part_the_body_cannot_pay_for_is_not_grown() {
    let mut g = world(false);
    // A limb of 729 voxels weighs 583 mg in 27 cells: 567 mg to express.
    let big = template([4, 4, 4], &[("contract", 27)]);
    g.rules.kinds.insert("kind:limb".into(), big);
    g.rules.processes.insert("test:grow".into(), grow_act());
    let id = first_grazer(&g);
    let mut e = body(&g, &[1], &[], &[[0, 0, 1], [0, 0, 2]], 12);
    e.accounts.insert(SOIL.into(), 30);
    *g.population.lift(id).unwrap() = e;
    let mut sim = Simulation::new(g, isocosm::Execution::Individuals).unwrap();
    let r = sim.execute(id, None, "test:grow", None);
    assert_eq!(r.outcome, Outcome::Accepted, "{r:?}");
    let e = sim.state().population.get(id).unwrap();
    assert_eq!(e.parts.len(), 1, "only the lump");
    assert_eq!(e.accounts[SOIL], 30, "the hand untouched");
}
