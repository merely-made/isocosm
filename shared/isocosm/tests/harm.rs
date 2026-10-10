// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use isocosm::{
    Execution, Session, anatomy,
    flows::{Flow, Holder},
    geometry::{Body, Sketch},
    harm,
    history::Command,
    probe::{BodyFounding, Crowd, HarmFounding, ProbeWorld, Variant},
    rules::*,
    schema::*,
    simulation::Genesis,
};
use std::collections::{BTreeMap, BTreeSet};

#[path = "harm/crowd.rs"]
mod crowd_tests;

fn fixture(fragment: bool) -> Genesis {
    let mut g = BodyFounding {
        sites: [1, 1],
        producers: [0, 0],
        grazers: [1, 1],
        cohort: 1,
        ..Default::default()
    }
    .generate()
    .unwrap()
    .genesis;
    g.rules.processes.clear();
    g.rules.competitions.clear();
    g.rules
        .traits
        .extend([harm::HEAL.into(), harm::FRAGMENT.into()]);
    if fragment {
        g.lineages
            .get_mut("lineage:1")
            .unwrap()
            .traits
            .insert(harm::FRAGMENT.into());
    }
    let root = Part {
        functions: BTreeSet::from([
            "function:intake".into(),
            "function:store".into(),
            "function:reproduce".into(),
        ]),
        cells: BTreeMap::from([
            ("function:intake".into(), 3),
            ("function:store".into(), 4),
            ("function:reproduce".into(), 1),
        ]),
        matter: Ledger::from([("tissue:1".into(), 100)]),
        ..Default::default()
    };
    let limb = Part {
        functions: BTreeSet::from(["function:contract".into()]),
        cells: BTreeMap::from([("function:contract".into(), 8)]),
        matter: Ledger::from([("tissue:1".into(), 100)]),
        ..Default::default()
    };
    let eye = Part {
        functions: BTreeSet::from(["function:sense".into()]),
        cells: BTreeMap::from([("function:sense".into(), 1)]),
        matter: Ledger::from([("tissue:1".into(), 80)]),
        ..Default::default()
    };
    let body = Body::sketch([
        (
            0,
            Sketch {
                half_extent: [2; 3],
                situs: Some([0, 0, 0]),
                part: root,
                ..Default::default()
            },
        ),
        (
            1,
            Sketch {
                parent: Some(0),
                half_extent: [2; 3],
                offset: [6, 0, 0],
                situs: Some([0, 1, 0]),
                part: limb,
                ..Default::default()
            },
        ),
        (
            2,
            Sketch {
                parent: Some(1),
                half_extent: [1; 3],
                offset: [6, 0, 0],
                situs: Some([0, 2, 0]),
                part: eye,
                ..Default::default()
            },
        ),
    ]);
    g.population.lift(1).unwrap().embody(body);
    g.validate().unwrap();
    g
}

type Holdings = BTreeMap<(Holder, Key), i128>;

fn holdings(s: &Session) -> Holdings {
    let mut out = Holdings::new();
    for (&id, site) in &s.sim.state().sites {
        for (key, n) in &site.accounts {
            out.insert((Holder::Site(id), key.clone()), i128::from(*n));
        }
    }
    for (&first, group) in &s.sim.state().population.groups {
        for id in first..first + group.count {
            for (key, n) in &group.entity.accounts {
                out.insert((Holder::Entity(id), key.clone()), i128::from(*n));
            }
            for (part, p) in &group.entity.parts {
                for (key, n) in &p.matter {
                    out.insert((Holder::Part(id, *part), key.clone()), i128::from(*n));
                }
            }
        }
    }
    out.retain(|_, n| *n != 0);
    out
}

fn reconciles(mut before: Holdings, flows: &[Flow], after: Holdings) {
    for f in flows {
        assert_eq!(f.count, 1, "these fixtures have individual flow legs");
        *before.entry(f.from.clone()).or_default() -= i128::from(f.amount);
        *before.entry(f.to.clone()).or_default() += i128::from(f.amount);
    }
    before.retain(|_, n| *n != 0);
    assert_eq!(
        before, after,
        "every holder and account reconciles independently"
    );
}

#[test]
fn severing_keeps_matter_in_site_fragment_and_empty_tombstones() {
    for fragment in [false, true] {
        for mode in [Execution::Individuals, Execution::Grouped] {
            let mut s = Session::new(fixture(fragment), mode).unwrap();
            s.advance(3).unwrap();
            let before = s.sim.matter();
            let books = holdings(&s);
            let original = s.sim.state().population.get(1).unwrap().clone();
            let revision = original.body_revision;
            let ground = s.sim.state().sites[&0].accounts.clone();
            let cut = s
                .command_with_flows(Command::Wound {
                    entity: 1,
                    part: Some(PartId(1)),
                    cells: 8,
                })
                .unwrap();
            let w: harm::Wounded = serde_json::from_str(&cut.result).unwrap();
            assert_eq!(w.severed, vec![PartId(1), PartId(2)]);
            let id = w.fragment.unwrap();
            let parent = s.sim.state().population.get(1).unwrap();
            let child = s.sim.state().population.get(id).unwrap();
            assert_eq!(parent.body_revision, revision + 1);
            assert!(
                parent.parts[&PartId(1)].matter.is_empty()
                    && parent.parts[&PartId(2)].matter.is_empty()
            );
            assert_eq!(
                anatomy::books(parent)["tissue:1"],
                anatomy::held(parent, &s.sim.genesis().rules, "tissue:1")
            );
            assert_eq!(child.alive, fragment);
            assert_eq!(child.born, if fragment { 3 } else { original.born });
            assert_eq!(
                child.provenance,
                if fragment {
                    Provenance::Born(original.lineage.clone())
                } else {
                    original.provenance
                }
            );
            assert!(
                child.accounts.is_empty(),
                "a fragment receives no provision or income"
            );
            assert_eq!(child.situs(PartId(0)), Some([0; 3]));
            assert_eq!(
                child.parts[&PartId(0)].lost.len(),
                8,
                "re-rooting never restores cells for free"
            );
            assert_eq!(child.parts[&PartId(1)].matter["tissue:1"], 80);
            assert_eq!(
                s.sim.state().sites[&0]
                    .accounts
                    .get("tissue:1")
                    .copied()
                    .unwrap_or(0),
                ground.get("tissue:1").copied().unwrap_or(0) + 100
            );
            assert_eq!(s.sim.matter(), before);
            reconciles(books, &cut.flows, holdings(&s));
            let relation = Relation {
                subject: id,
                object: 1,
                kind: "sim:parent".into(),
                value: 0,
            };
            assert_eq!(s.sim.state().relations.contains(&relation), fragment);
            assert!(
                cut.flows
                    .iter()
                    .any(|f| f.from.0 == isocosm::flows::Holder::Part(1, PartId(2))
                        && f.to.0 == isocosm::flows::Holder::Part(id, PartId(1))
                        && f.amount == 80)
            );
            let loaded = Session::load(s.save(), mode).unwrap();
            assert_eq!(s.sim.state_hash(), loaded.sim.state_hash());
        }
    }
}

#[test]
fn losing_every_root_cell_kills_without_severing_or_losing_descendant_matter() {
    let mut s = Session::new(fixture(true), Execution::Individuals).unwrap();
    let before = s.sim.matter();
    let cut = s
        .command(Command::Wound {
            entity: 1,
            part: Some(PartId(0)),
            cells: 8,
        })
        .unwrap();
    let w: harm::Wounded = serde_json::from_str(&cut).unwrap();
    assert!(w.died && w.severed.is_empty() && w.fragment.is_none());
    let e = s.sim.state().population.get(1).unwrap();
    assert!(!e.alive && e.lives(PartId(2)));
    assert_eq!(anatomy::books(e)["tissue:1"], 180);
    assert_eq!(s.sim.matter(), before);
}

#[test]
fn a_wound_spills_all_accounts_over_the_lowered_bound_without_changing_geometry() {
    let mut g = fixture(false);
    let e = g.population.lift(1).unwrap();
    let half = e.extent(PartId(0));
    let p = e.parts.get_mut(&PartId(0)).unwrap();
    for key in ["tissue:1", "reserve:1", "provision:1"] {
        p.matter
            .insert(key.into(), anatomy::bound(half, p, &g.rules, key));
    }
    let doc = e.body.clone();
    let revision = e.body_revision;
    let mut s = Session::new(g, Execution::Individuals).unwrap();
    let before = s.sim.matter();
    s.command(Command::Wound {
        entity: 1,
        part: Some(PartId(0)),
        cells: 4,
    })
    .unwrap();
    let e = s.sim.state().population.get(1).unwrap();
    assert_eq!(e.body, doc);
    assert_eq!(e.body_revision, revision);
    for key in ["tissue:1", "reserve:1", "provision:1"] {
        assert!(
            e.parts[&PartId(0)].matter[key]
                <= anatomy::bound(half, &e.parts[&PartId(0)], &s.sim.genesis().rules, key)
        );
    }
    assert_eq!(s.sim.matter(), before);
}

#[test]
fn hazard_zero_wounds_nothing_in_native_and_crowd_runs() {
    let world = BodyFounding {
        ticks: 12,
        harm: Some(HarmFounding {
            hazard: [0, 0],
            ..Default::default()
        }),
        ..Default::default()
    }
    .generate()
    .unwrap();
    let mut s = Session::new(world.genesis.clone(), Execution::Individuals).unwrap();
    s.advance(world.ticks).unwrap();
    assert!(
        s.sim.state().population.groups.values().all(|g| g
            .entity
            .parts
            .values()
            .all(|p| p.lost.is_empty()))
    );
    let crowd = Crowd::new(&world, world.genesis.dynamics_seed(), Variant::Histogram)
        .unwrap()
        .run()
        .unwrap();
    assert!(
        crowd
            .bins
            .keys()
            .chain(crowd.kin.values())
            .all(|e| e.parts.values().all(|p| p.lost.is_empty()))
    );
}

#[test]
fn rot_returns_every_dead_part_account_to_the_site() {
    let mut g = fixture(false);
    g.population.lift(1).unwrap().alive = false;
    let world = BodyFounding {
        harm: Some(HarmFounding {
            hazard: [0, 0],
            rot: [1000, 1000],
            ..Default::default()
        }),
        ..Default::default()
    }
    .generate()
    .unwrap();
    let rot = world.genesis.rules.processes["body:rot"].clone();
    g.rules.processes.insert(rot.id.clone(), rot);
    let mut s = Session::new(g, Execution::Individuals).unwrap();
    let before = s.sim.matter();
    let books = holdings(&s);
    let out = s.advance_tick_with_flows().unwrap();
    assert_eq!(
        anatomy::books(s.sim.state().population.get(1).unwrap())
            .values()
            .sum::<u64>(),
        0
    );
    assert_eq!(s.sim.state().sites[&0].accounts["tissue:1"], 280);
    assert_eq!(
        out.flows
            .iter()
            .filter(|f| f.by_process("body:rot"))
            .map(|f| f.amount)
            .sum::<u64>(),
        280
    );
    assert_eq!(s.sim.matter(), before);
    reconciles(books, &out.flows, holdings(&s));
}

#[test]
fn a_part_draw_follows_living_cells_instead_of_part_count() {
    let original = fixture(false);
    let mut hits = [0u32; 3];
    for seed in 0..512 {
        let mut g = original.clone();
        g.dynamics = Some(seed);
        let mut s = Session::new(g, Execution::Individuals).unwrap();
        let result = s
            .command(Command::Wound {
                entity: 1,
                part: None,
                cells: 1,
            })
            .unwrap();
        let wound: harm::Wounded = serde_json::from_str(&result).unwrap();
        hits[wound.part.0 as usize] += 1;
    }
    assert!(
        hits[0] > 190 && hits[1] > 190 && hits[2] < 65,
        "8:8:1 cells: {hits:?}"
    );
}

#[test]
fn a_refused_fragment_birth_changes_nothing() {
    let mut g = fixture(true);
    g.rules.limits.entities = g.population.count();
    let mut s = Session::new(g, Execution::Grouped).unwrap();
    let before = s.sim.state_hash();
    assert!(
        s.command_with_flows(Command::Wound {
            entity: 1,
            part: Some(PartId(1)),
            cells: 8,
        })
        .is_err()
    );
    assert_eq!(s.sim.state_hash(), before);
}

#[test]
fn admission_refuses_matter_hidden_in_a_tombstone() {
    let mut g = fixture(false);
    g.population
        .lift(1)
        .unwrap()
        .body
        .as_mut()
        .unwrap()
        .sever(PartId(2));
    assert!(
        g.validate()
            .unwrap_err()
            .contains("tombstone still holds matter")
    );
    g.population
        .lift(1)
        .unwrap()
        .parts
        .get_mut(&PartId(2))
        .unwrap()
        .matter
        .clear();
    g.validate().unwrap();
}

#[test]
fn admission_refuses_a_part_draw_sharing_an_expression_slot() {
    let mut g = BodyFounding {
        harm: Some(HarmFounding::default()),
        ..Default::default()
    }
    .generate()
    .unwrap()
    .genesis;
    let Effect::When { then, .. } =
        &mut g.rules.processes.get_mut("body:hazard").unwrap().effects[0]
    else {
        panic!("the hazard is guarded")
    };
    let Effect::Wound { slot, .. } = &mut then[0] else {
        panic!("the hazard wounds")
    };
    *slot = 0;
    assert!(
        g.validate()
            .unwrap_err()
            .contains("reuses a wound's part draw slot")
    );
}

#[test]
fn a_certain_hazard_wounds_once_and_crowd_preserves_all_matter() {
    let founding = BodyFounding {
        harm: Some(HarmFounding {
            hazard: [64, 64],
            cells: [1, 1],
            rot: [0, 0],
            ..Default::default()
        }),
        ..Default::default()
    }
    .generate()
    .unwrap();
    let mut g = fixture(false);
    let hazard = founding.genesis.rules.processes["body:hazard"].clone();
    g.rules.processes.insert(hazard.id.clone(), hazard);
    let world = ProbeWorld {
        genesis: g.clone(),
        ticks: 1,
    };
    let mut s = Session::new(g, Execution::Individuals).unwrap();
    let books = holdings(&s);
    let out = s.advance_tick_with_flows().unwrap();
    reconciles(books, &out.flows, holdings(&s));
    let lost = |e: &Entity| e.living().map(|(_, p)| p.lost.len()).sum::<usize>();
    assert_eq!(lost(s.sim.state().population.get(1).unwrap()), 1);
    let crowd = Crowd::new(&world, 57, Variant::Histogram)
        .unwrap()
        .run()
        .unwrap();
    assert_eq!(
        crowd
            .bins
            .iter()
            .map(|(e, n)| lost(e) as u64 * n)
            .sum::<u64>(),
        1
    );
    let native_mass = s.sim.matter();
    let crowd_mass = crowd
        .sites
        .values()
        .flat_map(|s| s.accounts.values())
        .sum::<u64>()
        + crowd
            .bins
            .iter()
            .map(|(e, n)| anatomy::books(e).values().sum::<u64>() * n)
            .sum::<u64>();
    assert_eq!(u128::from(crowd_mass), native_mass);
}
