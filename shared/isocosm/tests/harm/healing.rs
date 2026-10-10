// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use isocosm::mosaic::CellId;

fn healer(fragment: bool, heals: bool) -> Genesis {
    let mut g = fixture(fragment);
    if heals {
        g.lineages
            .get_mut("lineage:1")
            .unwrap()
            .traits
            .insert(harm::HEAL.into());
    }
    let source = BodyFounding::default().generate().unwrap();
    let mut p = source.genesis.rules.processes["body:upkeep-1"].clone();
    p.id = "test:heal".into();
    p.effects = vec![Effect::Heal {
        into: "world:soil".into(),
    }];
    g.rules.processes.insert(p.id.clone(), p);
    g
}

fn wound(s: &mut Session, part: u32, cells: u32) {
    s.command(Command::Wound {
        entity: 1,
        part: Some(PartId(part)),
        cells,
    })
    .unwrap();
}

fn actor(s: &Session) -> &Entity {
    s.sim.state().population.get(1).unwrap()
}

#[test]
fn healing_restores_recorded_cells_and_pays_reserve_before_tissue() {
    for mode in [Execution::Individuals, Execution::Grouped] {
        let mut g = healer(false, true);
        g.population
            .lift(1)
            .unwrap()
            .parts
            .get_mut(&PartId(0))
            .unwrap()
            .matter
            .insert("reserve:1".into(), 48);
        let original = g.population.get(1).unwrap().parts[&PartId(0)].clone();
        let mut s = Session::new(g, mode).unwrap();
        wound(&mut s, 0, 1);
        let lost = actor(&s).parts[&PartId(0)].lost.clone();
        assert_eq!(lost.len(), 1);
        assert_eq!(
            actor(&s).parts[&PartId(0)].losses[&lost[0]],
            Some("function:store".into())
        );
        let before = s.sim.matter();
        let books = holdings(&s);
        let revision = actor(&s).body_revision;
        let tissue = anatomy::held(actor(&s), &s.sim.genesis().rules, "tissue:1");
        let reserve = anatomy::held(actor(&s), &s.sim.genesis().rules, "reserve:1");
        let out = s.advance_tick_with_flows().unwrap();
        let p = &actor(&s).parts[&PartId(0)];
        assert!(p.lost.is_empty() && p.losses.is_empty());
        assert_eq!(p.cells, original.cells);
        assert_eq!(p.tracts, original.tracts);
        assert_eq!(actor(&s).body_revision, revision);
        assert_eq!(
            anatomy::held(actor(&s), &s.sim.genesis().rules, "tissue:1"),
            tissue
        );
        assert_eq!(
            anatomy::held(actor(&s), &s.sim.genesis().rules, "reserve:1"),
            reserve - 12
        );
        assert_eq!(
            out.flows
                .iter()
                .filter(|f| f.by_process("test:heal"))
                .map(|f| f.amount)
                .sum::<u64>(),
            12
        );
        assert_eq!(s.sim.matter(), before);
        reconciles(books, &out.flows, holdings(&s));
        let saved = Session::load(s.save(), mode).unwrap();
        assert_eq!(saved.sim.witness(), s.sim.witness());
    }
}

#[test]
fn a_restored_free_cell_stays_free_and_varied_allocation_survives() {
    let mut g = healer(false, true);
    let limb = g
        .population
        .lift(1)
        .unwrap()
        .parts
        .get_mut(&PartId(1))
        .unwrap();
    limb.cells.insert("function:contract".into(), 4);
    isocosm::mosaic::sync(limb, [2; 3]);
    let original = limb.clone();
    let mut s = Session::new(g, Execution::Individuals).unwrap();
    wound(&mut s, 1, 4);
    let p = &actor(&s).parts[&PartId(1)];
    assert_eq!(p.losses.values().filter(|f| f.is_none()).count(), 2);
    assert_eq!(p.losses.values().filter(|f| f.is_some()).count(), 2);
    let before = s.sim.matter();
    s.advance(1).unwrap();
    let p = &actor(&s).parts[&PartId(1)];
    assert_eq!(p.cells, original.cells);
    assert_eq!(p.tracts, original.tracts);
    assert_eq!(anatomy::living_cells([2; 3], p), 8);
    assert!(p.losses.is_empty() && p.lost.is_empty());
    assert_eq!(s.sim.matter(), before);
}

#[test]
fn a_lineage_without_healing_keeps_its_recorded_losses() {
    let mut s = Session::new(healer(false, false), Execution::Individuals).unwrap();
    wound(&mut s, 1, 1);
    let original = actor(&s).clone();
    let ground = s.sim.state().sites.clone();
    let out = s.advance_tick_with_flows().unwrap();
    assert_eq!(actor(&s), &original);
    assert_eq!(s.sim.state().sites, ground);
    assert!(out.flows.is_empty());
}

#[test]
fn an_unaffordable_cell_remains_lost_without_partial_payment() {
    let mut g = healer(false, true);
    for p in g.population.lift(1).unwrap().parts.values_mut() {
        p.matter.clear();
    }
    g.population
        .lift(1)
        .unwrap()
        .parts
        .get_mut(&PartId(0))
        .unwrap()
        .matter
        .insert("tissue:1".into(), 11);
    let mut s = Session::new(g, Execution::Grouped).unwrap();
    wound(&mut s, 1, 1);
    let books = holdings(&s);
    let lost = actor(&s).parts[&PartId(1)].lost.clone();
    let out = s.advance_tick_with_flows().unwrap();
    assert_eq!(holdings(&s), books);
    assert_eq!(actor(&s).parts[&PartId(1)].lost, lost);
    assert!(out.flows.is_empty());
}

#[test]
fn tombstones_revive_in_place_without_reclaiming_fragment_matter() {
    for mode in [Execution::Individuals, Execution::Grouped] {
        let mut g = healer(true, true);
        g.population
            .lift(1)
            .unwrap()
            .parts
            .get_mut(&PartId(0))
            .unwrap()
            .matter
            .insert("reserve:1".into(), 48);
        let original = g.population.get(1).unwrap().clone();
        let mut s = Session::new(g, mode).unwrap();
        wound(&mut s, 1, 8);
        let fragment = s.sim.state().population.get(2).unwrap().clone();
        assert_eq!(actor(&s).parts[&PartId(2)].lost, vec![CellId(0)]);
        assert_eq!(fragment.parts[&PartId(1)].cells["function:sense"], 1);
        let before = s.sim.matter();
        let books = holdings(&s);
        let revision = actor(&s).body_revision;
        let out = s.advance_tick_with_flows().unwrap();
        let healed = actor(&s);
        assert!(healed.lives(PartId(1)) && healed.lives(PartId(2)));
        assert_eq!(healed.body, original.body);
        assert_eq!(healed.body_revision, revision + 2);
        for id in [PartId(1), PartId(2)] {
            assert_eq!(healed.parts[&id].cells, original.parts[&id].cells);
            assert_eq!(healed.parts[&id].tracts, original.parts[&id].tracts);
            assert!(healed.parts[&id].matter.is_empty());
        }
        // The fragment's root also heals, paid by its own retained tissue.
        let child = s.sim.state().population.get(2).unwrap();
        assert!(child.alive);
        assert_eq!(
            child.parts[&PartId(1)].cells,
            fragment.parts[&PartId(1)].cells
        );
        assert!(anatomy::books(child).values().sum::<u64>() < 80);
        assert_eq!(s.sim.matter(), before);
        reconciles(books, &out.flows, holdings(&s));
        assert_eq!(
            s.sim.witness(),
            Session::load(s.save(), mode).unwrap().sim.witness()
        );
    }
}

#[test]
fn one_paid_cell_revives_only_the_top_tombstone() {
    let mut g = healer(false, true);
    g.population
        .lift(1)
        .unwrap()
        .parts
        .get_mut(&PartId(0))
        .unwrap()
        .matter
        .insert("tissue:1".into(), 12);
    let mut s = Session::new(g, Execution::Individuals).unwrap();
    wound(&mut s, 1, 8);
    s.advance(1).unwrap();
    assert!(actor(&s).lives(PartId(1)) && !actor(&s).lives(PartId(2)));
    assert_eq!(actor(&s).parts[&PartId(1)].cells["function:contract"], 1);
    assert_eq!(actor(&s).parts[&PartId(1)].lost.len(), 7);
    assert_eq!(actor(&s).parts[&PartId(2)].lost.len(), 1);
    assert_eq!(
        anatomy::held(actor(&s), &s.sim.genesis().rules, "tissue:1"),
        0
    );
}

fn damaged(known: bool) -> Genesis {
    let g = healer(false, true);
    let mut s = Session::new(g.clone(), Execution::Individuals).unwrap();
    wound(&mut s, 1, 2);
    let mut g = g;
    g.population = s.sim.state().population.clone();
    g.sites = s.sim.state().sites.clone();
    let p = &mut g
        .population
        .lift(1)
        .unwrap()
        .parts
        .get_mut(&PartId(1))
        .unwrap();
    if known {
        p.losses.remove(&p.lost[0]);
    } else {
        p.losses.clear();
    }
    g
}

#[test]
fn unknown_legacy_losses_do_not_block_known_healing_in_the_same_body() {
    let g = damaged(true);
    let p = &g.population.get(1).unwrap().parts[&PartId(1)];
    let unknown = p.lost[0];
    assert!(!p.losses.contains_key(&unknown) && p.losses.len() == 1);
    let mut s = Session::new(g, Execution::Grouped).unwrap();
    s.advance(1).unwrap();
    let p = &actor(&s).parts[&PartId(1)];
    assert_eq!(p.lost, vec![unknown]);
    assert!(p.losses.is_empty());
    assert_eq!(p.cells["function:contract"], 7);
    assert_eq!(
        Session::load(s.save(), Execution::Grouped)
            .unwrap()
            .sim
            .witness(),
        s.sim.witness()
    );
}

#[test]
fn a_legacy_genesis_loss_loads_only_with_its_normal_digests_intact() {
    let mut s = Session::new(damaged(false), Execution::Individuals).unwrap();
    s.advance(3).unwrap();
    let json = serde_json::to_vec(&s.save()).unwrap();
    let loaded = Session::load_json(&json, Execution::Grouped).unwrap();
    assert_eq!(loaded.sim.witness(), s.sim.witness());
    assert_eq!(actor(&loaded).parts[&PartId(1)].lost.len(), 2);
    let mut changed = s.save();
    let p = changed
        .genesis
        .population
        .lift(1)
        .unwrap()
        .parts
        .get_mut(&PartId(1))
        .unwrap();
    p.losses.insert(p.lost[0], None);
    let failure = Session::load(changed, Execution::Individuals)
        .err()
        .unwrap();
    assert!(failure.contains("genesis digest mismatch"), "{failure}");
    let mut changed = s.save();
    let p = changed
        .genesis
        .population
        .lift(1)
        .unwrap()
        .parts
        .get_mut(&PartId(1))
        .unwrap();
    p.losses.insert(p.lost[0], None);
    changed.genesis_digest = isocosm::digest(&changed.genesis);
    let failure = Session::load(changed, Execution::Individuals)
        .err()
        .unwrap();
    assert!(
        failure.contains("diverge") || failure.contains("mismatch"),
        "{failure}"
    );
}

#[test]
fn known_healing_agrees_exactly_with_the_crowd_on_a_deterministic_fixture() {
    let world = ProbeWorld {
        genesis: damaged(true),
        ticks: 1,
    };
    let mut s = Session::new(world.genesis.clone(), Execution::Individuals).unwrap();
    s.advance(1).unwrap();
    let crowd = Crowd::new(&world, 57, Variant::Histogram)
        .unwrap()
        .run()
        .unwrap();
    let e = crowd
        .bins
        .iter()
        .find(|(e, _)| e.lineage == "lineage:1")
        .unwrap();
    assert_eq!(*e.1, 1);
    assert_eq!(e.0.parts, actor(&s).parts);
    assert_eq!(crowd.sites, s.sim.state().sites);
}

#[test]
fn admission_rejects_loss_metadata_for_present_cells_or_unknown_functions() {
    let mut g = damaged(true);
    let p = g
        .population
        .lift(1)
        .unwrap()
        .parts
        .get_mut(&PartId(1))
        .unwrap();
    p.losses.insert(CellId(0), None);
    assert!(g.validate().unwrap_err().contains("present cell"));
    let mut g = damaged(true);
    let p = g
        .population
        .lift(1)
        .unwrap()
        .parts
        .get_mut(&PartId(1))
        .unwrap();
    p.losses.insert(p.lost[0], Some("function:absent".into()));
    assert!(g.validate().unwrap_err().contains("unknown function"));
}

#[test]
fn generated_landing_heals_before_tissue_and_after_a_hungry_reserve() {
    for hungry in [false, true] {
        let mut g = healer(false, true);
        let source = BodyFounding::default().generate().unwrap();
        g.population
            .lift(1)
            .unwrap()
            .parts
            .get_mut(&PartId(2))
            .unwrap()
            .matter
            .insert("tissue:1".into(), 21);
        let landing = source.genesis.rules.processes["body:fix-1"].effects[2..].to_vec();
        let p = g.rules.processes.get_mut("test:heal").unwrap();
        p.effects = [
            vec![Effect::Transfer {
                from: Binding::Place,
                to: Binding::Actor,
                account: "world:soil".into(),
                amount: 13.into(),
            }],
            landing,
        ]
        .concat();
        if !hungry {
            g.population
                .lift(1)
                .unwrap()
                .parts
                .get_mut(&PartId(0))
                .unwrap()
                .matter
                .insert("reserve:1".into(), 48);
        }
        let mut s = Session::new(g, Execution::Individuals).unwrap();
        wound(&mut s, 1, 1);
        let tissue = anatomy::held(actor(&s), &s.sim.genesis().rules, "tissue:1");
        let before = s.sim.matter();
        let books = holdings(&s);
        let out = s.advance_tick_with_flows().unwrap();
        assert!(actor(&s).parts[&PartId(1)].lost.is_empty());
        let held = |k| anatomy::held(actor(&s), &s.sim.genesis().rules, k);
        assert_eq!(held("reserve:1"), if hungry { 1 } else { 36 });
        assert_eq!(held("tissue:1"), if hungry { tissue } else { tissue + 13 });
        assert_eq!(s.sim.matter(), before);
        reconciles(books, &out.flows, holdings(&s));
    }
}

#[test]
fn full_wounded_parts_request_healing_income() {
    let mut g = healer(false, true);
    let p = g.rules.processes.get_mut("test:heal").unwrap();
    p.effects = vec![Effect::When {
        guard: Expr::Read(Reading::Healing {
            who: Binding::Actor,
        }),
        then: vec![Effect::Trait {
            who: Binding::Actor,
            key: harm::HEAL.into(),
            present: true,
        }],
        otherwise: vec![],
    }];
    let mut s = Session::new(g, Execution::Grouped).unwrap();
    wound(&mut s, 1, 1);
    assert_eq!(
        anatomy::room(actor(&s), &s.sim.genesis().rules, "tissue:1"),
        0
    );
    s.advance(1).unwrap();
    assert!(actor(&s).traits.contains(harm::HEAL));
    assert_eq!(actor(&s).parts[&PartId(1)].lost.len(), 1);
}

#[test]
fn a_full_wounded_producer_draws_healing_income_through_its_working_route() {
    let mut g = BodyFounding {
        sites: [1, 1],
        producers: [1, 1],
        grazers: [0, 0],
        cohort: 1,
        reproduce: [0, 0],
        variance: [0, 0],
        absence: [0, 0],
        tissue: [1000, 1000],
        reserve: [0, 0],
        candidates: 0,
        fixes: [100_000, 100_000],
        carriage: [10_000, 10_000],
        harm: Some(HarmFounding {
            hazard: [0, 0],
            heals: 1000,
            ..Default::default()
        }),
        ..Default::default()
    }
    .generate()
    .unwrap()
    .genesis;
    g.rules.processes.retain(|id, _| id == "body:fix-0");
    let full = g.population.get(1).unwrap().parts.clone();
    for mode in [Execution::Individuals, Execution::Grouped] {
        let mut s = Session::new(g.clone(), mode).unwrap();
        wound(&mut s, 0, 1);
        assert_eq!(anatomy::room(actor(&s), &g.rules, "tissue:0"), 0);
        let mut damaged = g.clone();
        damaged.population = s.sim.state().population.clone();
        damaged.sites = s.sim.state().sites.clone();
        let before = s.sim.matter();
        let books = holdings(&s);
        let out = s.advance_tick_with_flows().unwrap();
        assert!(actor(&s).parts[&PartId(0)].lost.is_empty());
        for (id, p) in &full {
            assert_eq!(actor(&s).parts[id].cells, p.cells);
            assert_eq!(actor(&s).parts[id].matter["tissue:0"], p.matter["tissue:0"]);
        }
        assert_eq!(s.sim.matter(), before);
        reconciles(books, &out.flows, holdings(&s));
        let world = ProbeWorld {
            genesis: damaged,
            ticks: 1,
        };
        let crowd = Crowd::new(&world, 57, Variant::Histogram)
            .unwrap()
            .run()
            .unwrap();
        let found = crowd
            .bins
            .keys()
            .find(|e| e.lineage == "lineage:0")
            .unwrap();
        let clean = |e: &Entity| {
            let mut parts = e.parts.clone();
            for p in parts.values_mut() {
                p.matter.retain(|_, n| *n > 0);
            }
            parts
        };
        assert_eq!(clean(found), clean(actor(&s)));
        assert_eq!(
            crowd.sites[&0].accounts["world:soil"],
            s.sim.state().sites[&0].accounts["world:soil"]
        );
    }
}
