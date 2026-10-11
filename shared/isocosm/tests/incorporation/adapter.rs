// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use isocosm::{
    growth, harm,
    mosaic::{self, CellId},
    population::Population,
    probe::{Crowd, ProbeWorld, Variant, readings::crowd_members},
};

const DYNAMICS: u64 = 57;

struct Fixture {
    genesis: Genesis,
    eater: Id,
    prey: Id,
    original: Part,
    known: BTreeSet<CellId>,
    unknown: CellId,
    price: u64,
}

fn injured(domain: u16) -> Fixture {
    let (mut g, eater, prey) = world(domain, 1000, true);
    g.rules.processes.retain(|id, _| id == "test:graze");
    g.rules.competitions.clear();
    g.rules.mind = None;
    g.rules.carriage = None;
    let p = g
        .population
        .lift(prey)
        .unwrap()
        .parts
        .get_mut(&PartId(0))
        .unwrap();
    p.cells.insert("function:fix".into(), 2);
    mosaic::sync(p, [3, 3, 1]);
    let mut wounder = Session::new(g.clone(), Execution::Individuals).unwrap();
    wounder
        .command(Command::Wound {
            entity: prey,
            part: Some(PartId(0)),
            cells: 3,
        })
        .unwrap();
    g.population = wounder.sim.state().population.clone();
    g.sites = wounder.sim.state().sites.clone();
    let mut producer = g.population.get(prey).unwrap().clone();
    let p = producer.parts.get_mut(&PartId(0)).unwrap();
    let unknown = *p.losses.iter().find(|(_, f)| f.is_some()).unwrap().0;
    p.losses.remove(&unknown);
    assert_eq!(p.lost.len(), 3);
    assert!(p.losses.values().any(Option::is_some));
    assert!(p.losses.values().any(Option::is_none));
    let original = p.clone();
    let known = p.losses.keys().copied().collect();
    let price = u64::from(anatomy::cell_mass(
        producer.extent(PartId(0)),
        g.rules.body(),
    ));
    let mut grazer = g.population.get(eater).unwrap().clone();
    grazer.place = producer.place;
    let ground = g
        .population
        .groups
        .values()
        .find(|c| c.entity.lineage == "world:ground" && c.entity.place == producer.place)
        .unwrap()
        .entity
        .clone();
    let mut population = Population::default();
    population.insert(ground, 1).unwrap();
    let prey = population.insert(producer, 1).unwrap();
    let eater = population.insert(grazer, 1).unwrap();
    g.population = population;
    let site = g.population.get(eater).unwrap().place;
    g.sites.retain(|id, _| *id == site);
    g.dynamics = Some(DYNAMICS);
    g.rules.traits.insert(harm::HEAL.into());
    g.lineages
        .get_mut("lineage:1")
        .unwrap()
        .traits
        .insert(harm::HEAL.into());
    let mut graze = g.rules.processes["test:graze"].clone();
    graze.requires.push(Query::Expresses {
        function: "function:intake".into(),
    });
    graze.period = Some(1);
    let target = graze.target.as_mut().unwrap();
    target.same_place = true;
    target.weighted = true;
    let mut heal = graze.clone();
    heal.id = "test:heal".into();
    heal.effects = vec![Effect::Heal {
        into: "world:soil".into(),
    }];
    heal.target = None;
    heal.period = Some(2);
    heal.priority = 1;
    g.rules.processes = BTreeMap::from([(graze.id.clone(), graze), (heal.id.clone(), heal)]);
    Fixture {
        genesis: g,
        eater,
        prey,
        original,
        known,
        unknown,
        price,
    }
}

fn moved(e: &Entity) -> (PartId, &Part) {
    e.parts
        .iter()
        .find(|(_, p)| p.matter.contains_key("tissue:0"))
        .map(|(id, p)| (*id, p))
        .expect("the injured frond transferred whole")
}

fn contents(a: &Entity, b: &Entity) -> Ledger {
    let mut held = anatomy::books(a);
    for (key, n) in anatomy::books(b) {
        *held.entry(key).or_default() += n;
    }
    held
}

#[test]
fn an_adapter_clears_only_known_loss_allocations() {
    for domain in [0, 2] {
        let f = injured(domain);
        let mut eater = f.genesis.population.get(f.eater).unwrap().clone();
        let mut prey = f.genesis.population.get(f.prey).unwrap().clone();
        let d = |l: &Key| f.genesis.lineages[l].development.as_ref().unwrap();
        let affinity = f.genesis.rules.affinity.clone().unwrap_or_default();
        let w = growth::whole(
            (&eater, &prey),
            PartId(0),
            (d(&eater.lineage), d(&prey.lineage)),
            &affinity,
        )
        .unwrap();
        assert_eq!(
            w.verdict,
            if domain == 0 {
                Verdict::Native
            } else {
                Verdict::Adapter
            }
        );
        let before = contents(&eater, &prey);
        let id = growth::take_whole(&mut eater, &mut prey, PartId(0), &w).unwrap();
        let p = &eater.parts[&id];
        assert_eq!(p.lost, f.original.lost);
        assert_eq!(p.losses.keys().copied().collect::<BTreeSet<_>>(), f.known);
        assert!(
            !p.losses.contains_key(&f.unknown),
            "811: absent loss stays unknown"
        );
        if domain == 0 {
            assert_eq!(
                p, &f.original,
                "native transfer retains recorded donor allocations"
            );
        } else {
            assert!(
                p.losses.values().all(Option::is_none),
                "812: donor allocation cleared"
            );
            assert!(p.cells.is_empty() && p.functions.is_empty() && p.tracts.is_empty());
            assert_eq!(p.matter, f.original.matter);
        }
        assert!(!prey.parts.contains_key(&PartId(0)) && !prey.alive);
        assert!(!prey.lives(PartId(0)));
        assert_eq!(contents(&eater, &prey), before);
    }
}

fn states(members: Vec<(&Entity, u64)>) -> BTreeMap<Entity, u64> {
    let mut states = BTreeMap::new();
    for (e, n) in members {
        if e.kingdom == "kingdom:world" {
            continue;
        }
        let mut e = e.clone();
        e.accounts.retain(|_, n| *n != 0);
        *states.entry(e).or_default() += n;
    }
    states
}

fn native_states(s: &Session) -> BTreeMap<Entity, u64> {
    states(
        s.sim
            .state()
            .population
            .groups
            .values()
            .map(|c| (&c.entity, c.count))
            .collect(),
    )
}

fn crowd_matter(crowd: &Crowd<'_>, rules: &Rules) -> u128 {
    let mass = |ledger: &Ledger| -> u128 {
        ledger
            .iter()
            .filter(|(k, _)| matches!(rules.accounts.get(*k), Some(AccountKind::Matter { .. })))
            .map(|(_, n)| u128::from(*n))
            .sum()
    };
    crowd
        .bins
        .iter()
        .map(|(e, n)| mass(&anatomy::books(e)) * u128::from(*n))
        .sum::<u128>()
        + crowd
            .kin
            .values()
            .map(|e| mass(&anatomy::books(e)))
            .sum::<u128>()
        + crowd
            .sites
            .values()
            .map(|s| mass(&s.accounts))
            .sum::<u128>()
}

fn tick(s: &mut Session) {
    let before = books(s);
    let total = s.sim.matter();
    let out = s.advance_tick_with_flows().unwrap();
    let mut claimed = Books::new();
    for f in &out.flows {
        for (h, k, sign) in [(f.from.0, &f.from.1, -1), (f.to.0, &f.to.1, 1)] {
            *claimed.entry((h, k.clone())).or_default() +=
                sign * i128::from(f.amount) * i128::from(f.count);
        }
    }
    let after = books(s);
    let keys: BTreeSet<_> = before
        .keys()
        .chain(after.keys())
        .chain(claimed.keys())
        .collect();
    for key in keys {
        assert_eq!(
            after.get(key).copied().unwrap_or(0) - before.get(key).copied().unwrap_or(0),
            claimed.get(key).copied().unwrap_or(0),
            "{key:?}"
        );
    }
    assert_eq!(s.sim.matter(), total);
}

fn assert_transfer(s: &Session, f: &Fixture) {
    let (_, p) = moved(s.sim.state().population.get(f.eater).unwrap());
    assert_eq!(p.lost, f.original.lost);
    assert_eq!(p.losses.keys().copied().collect::<BTreeSet<_>>(), f.known);
    assert!(p.losses.values().all(Option::is_none));
    assert!(!p.losses.contains_key(&f.unknown));
    assert!(p.functions.is_empty() && p.cells.is_empty() && p.tracts.is_empty());
    assert_eq!(p.matter, f.original.matter);
}

fn assert_healed(s: &Session, f: &Fixture) {
    let (id, p) = moved(s.sim.state().population.get(f.eater).unwrap());
    assert_eq!(
        p.lost,
        vec![f.unknown],
        "811: no invented allocation or restoration"
    );
    assert!(p.losses.is_empty());
    assert!(
        p.functions.is_empty() && p.cells.is_empty() && p.tracts.is_empty(),
        "812: paid healing cannot recreate donor functions"
    );
    let e = s.sim.state().population.get(f.eater).unwrap();
    assert_eq!(
        anatomy::living_cells(e.extent(id), p),
        anatomy::capacity(e.extent(id)) - 1
    );
}

#[test]
fn adapter_transfer_and_paid_healing_agree_in_native_and_crowd() {
    let f = injured(2);
    for ticks in [1, 2] {
        let world = ProbeWorld {
            genesis: f.genesis.clone(),
            ticks,
        };
        let crowd = Crowd::new(&world, DYNAMICS, Variant::Histogram)
            .unwrap()
            .run()
            .unwrap();
        for mode in [Execution::Individuals, Execution::Grouped] {
            let mut s = Session::new(f.genesis.clone(), mode).unwrap();
            let before = s.sim.matter();
            tick(&mut s);
            assert_transfer(&s, &f);
            if ticks == 2 {
                let e = s.sim.state().population.get(f.eater).unwrap();
                let own_before = anatomy::held(e, &f.genesis.rules, "tissue:1");
                let site = e.place;
                let soil_before = s.sim.state().sites[&site].accounts["world:soil"];
                tick(&mut s);
                assert_healed(&s, &f);
                let paid = f.price * f.known.len() as u64;
                let e = s.sim.state().population.get(f.eater).unwrap();
                assert_eq!(
                    anatomy::held(e, &f.genesis.rules, "tissue:1"),
                    own_before - paid
                );
                assert_eq!(
                    s.sim.state().sites[&site].accounts["world:soil"],
                    soil_before + paid
                );
            }
            assert_eq!(s.sim.matter(), before);
            assert_eq!(crowd_matter(&crowd, &f.genesis.rules), before);
            assert_eq!(
                native_states(&s),
                states(crowd_members(&crowd)),
                "{ticks} {mode:?}"
            );
            assert_eq!(s.sim.state().sites, crowd.sites);
        }
    }
}

#[test]
fn adapter_loss_reset_survives_new_save_and_replay() {
    let f = injured(2);
    for mode in [Execution::Individuals, Execution::Grouped] {
        let mut s = Session::new(f.genesis.clone(), mode).unwrap();
        tick(&mut s);
        assert_transfer(&s, &f);
        let saved = serde_json::to_vec(&s.save()).unwrap();
        let mut loaded = Session::load_json(&saved, mode).unwrap();
        assert_eq!(loaded.sim.witness(), s.sim.witness());
        assert_transfer(&loaded, &f);
        tick(&mut s);
        tick(&mut loaded);
        assert_healed(&loaded, &f);
        assert_eq!(loaded.sim.witness(), s.sim.witness());
        assert_eq!(native_states(&loaded), native_states(&s));
        let json = serde_json::to_vec(&loaded.save()).unwrap();
        let roundtrip = Session::load_json(&json, mode).unwrap();
        assert_eq!(roundtrip.sim.witness(), loaded.sim.witness());
        assert_healed(&roundtrip, &f);
    }
}
