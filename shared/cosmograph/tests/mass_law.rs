// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The body-size rule (case 1148): rates scale with adult mass to the
//! three-quarter and one-quarter powers. Unset it changes nothing; a lineage
//! at the reference keeps the flat rates; heavier lineages pay more upkeep a
//! body and less a unit of mass, eat bigger meals, mature later, breed less
//! often and live longer; factors stay in their bounds; matter is conserved;
//! a run replays.

use cosmograph::{
    Execution, Founding, Session, Simulation,
    bodied::Bodies,
    digest,
    directing::{Played, readings},
    ecology::{MassLaw, Rates, mass_law},
    rules::*,
    simulation::Genesis,
};

fn founding(law: Option<MassLaw>) -> Founding {
    let grid = cosmograph::map::Grid { width: 3, height: 2, ..cosmograph::map::Grid::drawn(9) };
    Founding {
        seed: 9,
        sites: 6,
        map: Some(cosmograph::map::Layout::Grid(grid)),
        ecology: true,
        ecology_rates: Rates::A1,
        bodies: Some(Bodies { hazard: [2, 8], wound: [1, 8], ..Bodies::default() }),
        played: Some(Played { lineage: 1, region_sites: 2 }),
        mass_law: law,
        ..Founding::default()
    }
}

fn world(law: Option<MassLaw>) -> Genesis {
    founding(law).generate().unwrap()
}

fn process<'a>(g: &'a Genesis, kind: &str, i: u32) -> &'a Process {
    &g.rules.processes[&format!("ecology:{kind}-{i}")]
}

fn json(p: &Process) -> String {
    serde_json::to_string(p).unwrap()
}

fn age(p: &Process) -> u64 {
    p.requires.iter().find_map(|q| if let Query::Age { at_least } = q { Some(*at_least) } else { None }).unwrap()
}

/// Matter an upkeep moves to the soil each tick.
fn upkeep_rate(p: &Process) -> f64 {
    let paid: u64 = p.effects.iter().filter_map(|e| match e {
        Effect::Transfer { amount: Amount::Fixed(a), .. } => Some(*a),
        _ => None,
    }).sum();
    paid as f64 / p.period.unwrap() as f64
}

fn meal(p: &Process) -> u64 {
    match &p.effects[..] {
        [Effect::Eat { amount: Amount::Fixed(n), .. }] => *n,
        other => panic!("a single meal, found {other:?}"),
    }
}

/// Close to `want` within `tol`, as a ratio.
fn near(got: f64, want: f64, tol: f64, what: &str) {
    assert!((got / want - 1.0).abs() <= tol, "{what}: got {got}, wanted {want}");
}

fn masses(g: &Genesis) -> std::collections::BTreeMap<u32, u64> {
    mass_law::masses(&g.rules, &g.lineages).unwrap()
}

#[test]
fn unset_the_genesis_never_mentions_the_rule() {
    let g = world(None);
    assert!(g.founding.as_ref().unwrap().mass_law.is_none());
    let text = serde_json::to_string(&g).unwrap();
    assert!(!text.contains("mass_law"), "no mention of the rule");
    assert_eq!(digest(&world(None)), digest(&g), "founding is deterministic");
}

#[test]
fn the_reference_is_stated_and_a_lineage_at_it_keeps_the_flat_rates() {
    let (flat, law) = (world(None), world(Some(MassLaw::default())));
    let stated = law.founding.as_ref().unwrap().mass_law.clone().unwrap();
    let m = masses(&law);
    let mut all: Vec<u64> = m.values().copied().collect();
    all.sort_unstable();
    assert_eq!(stated.reference, Some(all[(all.len() - 1) / 2]), "the median adult mass, recorded");
    assert_eq!(masses(&flat), m, "the rule changes no body");
    let at = *m.iter().find(|(_, mass)| Some(**mass) == stated.reference).unwrap().0;
    for kind in ["upkeep", "birth", "age", "hazard", "feed", "produce", "death"] {
        match (flat.rules.processes.get(&format!("ecology:{kind}-{at}")), law.rules.processes.get(&format!("ecology:{kind}-{at}"))) {
            (Some(a), Some(b)) => assert_eq!(json(a), json(b), "{kind} at the reference"),
            (None, None) => {},
            _ => panic!("{kind} declared in one world only"),
        }
    }
    // A lineage with no body keeps the flat rates whatever the rule says.
    let bare = (0..4).find(|i| !m.contains_key(i)).expect("a decomposer has no body");
    for kind in ["upkeep", "birth", "age", "feed"] {
        assert_eq!(json(process(&flat, kind, bare)), json(process(&law, kind, bare)));
    }
}

#[test]
fn heavier_lineages_pay_more_a_body_and_less_a_gram_eat_more_mature_later_and_live_longer() {
    let (flat, law) = (world(None), world(Some(MassLaw::default())));
    let m = masses(&law);
    let reference = law.founding.as_ref().unwrap().mass_law.as_ref().unwrap().reference.unwrap() as f64;
    let (light, heavy) = (*m.iter().min_by_key(|(_, v)| **v).unwrap().0, *m.iter().max_by_key(|(_, v)| **v).unwrap().0);
    assert!(m[&light] < m[&heavy]);
    let mut seen = Vec::new();
    for i in [light, heavy] {
        let ratio = m[&i] as f64 / reference;
        // Per body per unit time, against the same lineage's flat rate.
        let up = upkeep_rate(process(&law, "upkeep", i)) / upkeep_rate(process(&flat, "upkeep", i));
        near(up, ratio.powf(0.75), 0.03, "upkeep per body is the 3/4 power");
        near(up / ratio, ratio.powf(-0.25), 0.03, "upkeep per unit of mass is the -1/4 power");
        let time = |k: &str| age(process(&law, k, i)) as f64 / age(process(&flat, k, i)) as f64;
        for k in ["age", "hazard"] {
            if law.rules.processes.contains_key(&format!("ecology:{k}-{i}")) {
                near(time(k), ratio.powf(0.25), 0.02, "ages are the 1/4 power");
            }
        }
        let spacing = process(&law, "birth", i).period.unwrap() as f64 / process(&flat, "birth", i).period.unwrap() as f64;
        near(spacing, ratio.powf(0.25), 0.04, "birth spacing is the 1/4 power");
        if law.rules.processes.contains_key(&format!("ecology:feed-{i}")) {
            let (a, b) = (meal(process(&law, "feed", i)), meal(process(&flat, "feed", i)));
            // Meals are whole units; the ratio holds to within half a unit.
            assert!((a as f64 - b as f64 * ratio.powf(0.75)).abs() <= 0.5 + 1e-9, "meal {a} from {b}");
        }
        let flat_of = |k: &str| age(process(&flat, k, i)) as f64;
        seen.push((up, age(process(&law, "age", i)) as f64 / flat_of("age"), spacing));
    }
    // Against each lineage's own flat rate: the heavy pay more upkeep a body
    // and live longer and breed less often than the light.
    let ((up_l, life_l, birth_l), (up_h, life_h, birth_h)) = (seen[0], seen[1]);
    assert!(up_h > up_l, "a heavier body pays more upkeep a unit of time");
    assert!(life_h > life_l && birth_h > birth_l, "it lives longer and breeds less often");
    let per_gram = |i: u32| upkeep_rate(process(&law, "upkeep", i)) / upkeep_rate(process(&flat, "upkeep", i)) / m[&i] as f64;
    assert!(per_gram(heavy) < per_gram(light), "and less for each unit of its mass");
    // It eats bigger meals, where both feed: its meal grows against its flat one.
    let grown = |i: u32| {
        let id = format!("ecology:feed-{i}");
        let (a, b) = (law.rules.processes.get(&id)?, flat.rules.processes.get(&id)?);
        Some(meal(a) as f64 / meal(b) as f64)
    };
    if let (Some(l), Some(h)) = (grown(light), grown(heavy)) {
        assert!(h > l, "a heavier lineage's meal is the bigger against its flat one");
    }
}

#[test]
fn a_consumer_eats_the_bigger_meal_the_heavier_it_is_against_the_reference() {
    let at = |reference| {
        let g = world(Some(MassLaw { reference: Some(reference), ..MassLaw::default() }));
        meal(process(&g, "feed", 1))
    };
    let flat = meal(process(&world(None), "feed", 1));
    // The consumer weighs 900 mg: lighter than 3246, as heavy as 900, heavier than 300.
    let (light, same, heavy) = (at(3246), at(900), at(300));
    assert_eq!(same, flat, "at the reference the flat meal");
    assert!(light < flat && flat < heavy, "{light} < {flat} < {heavy}");
}

#[test]
fn an_extreme_mass_is_held_at_the_bounds() {
    let b = mass_law::BOUNDS;
    let f = mass_law::factors(u64::MAX, 1, b);
    assert_eq!((f.upkeep, f.time), (b[1], b[1]));
    let f = mass_law::factors(1, u64::MAX, b);
    assert_eq!((f.upkeep, f.time), (b[0], b[0]));
    assert_eq!(mass_law::factors(500, 500, b).upkeep, 100);
    // A world whose reference is a gram a body: every lineage runs at the top.
    let law = |reference| Some(MassLaw { reference: Some(reference), bounds: [30, 250] });
    let (flat, top) = (world(None), world(law(1)));
    let sim = Simulation::new(top.clone(), Execution::Individuals).unwrap();
    for (key, row) in readings::mass_law(&sim) {
        assert_eq!((row.upkeep, row.time), (250, 250), "{key}");
        let i: u32 = key.trim_start_matches("lineage:").parse().unwrap();
        near(upkeep_rate(process(&top, "upkeep", i)), upkeep_rate(process(&flat, "upkeep", i)) * 2.5, 0.02, "upkeep at the top bound");
        near(age(process(&top, "age", i)) as f64, age(process(&flat, "age", i)) as f64 * 2.5, 0.01, "lifespan at the top bound");
    }
    let low = world(law(1_000_000_000));
    let sim = Simulation::new(low, Execution::Individuals).unwrap();
    assert!(readings::mass_law(&sim).values().all(|r| (r.upkeep, r.time) == (30, 30)));
    // Bounds that exclude the reference factor are refused.
    assert!(founding(Some(MassLaw { reference: None, bounds: [110, 400] })).generate().is_err());
}

#[test]
fn world_matter_is_unchanged_over_a_run_with_the_rule_on() {
    let mut sim = Simulation::new(world(Some(MassLaw::default())), Execution::Individuals).unwrap();
    let before = sim.matter();
    sim.advance(200).unwrap();
    assert_eq!(sim.matter(), before);
}

#[test]
fn a_saved_run_reloads_and_replays_under_both_modes() {
    for mode in [Execution::Individuals, Execution::Grouped] {
        let mut session = Session::new(world(Some(MassLaw::default())), mode).unwrap();
        session.advance(40).unwrap();
        let saved = session.save();
        let mut loaded = Session::load(saved, mode).unwrap();
        assert_eq!(loaded.sim.state_hash(), session.sim.state_hash());
        session.advance(40).unwrap();
        loaded.advance(40).unwrap();
        assert_eq!(loaded.sim.state_hash(), session.sim.state_hash(), "{mode:?}");
    }
}

fn living(sim: &Simulation) -> std::collections::BTreeMap<String, u64> {
    let mut out = std::collections::BTreeMap::new();
    for g in sim.state().population.groups.values().filter(|g| g.entity.alive) {
        *out.entry(g.entity.lineage.clone()).or_insert(0) += g.count;
    }
    out
}

#[test]
fn survival_note_the_same_seeded_world_with_the_rule_off_and_on() {
    let run = |law| {
        let mut sim = Simulation::new(world(law), Execution::Individuals).unwrap();
        let mut seen = vec![living(&sim)];
        for _ in 0..3 {
            sim.advance(100).unwrap();
            seen.push(living(&sim));
        }
        (seen, sim)
    };
    let (off, _) = run(None);
    let (on, sim) = run(Some(MassLaw::default()));
    let rows = readings::mass_law(&sim);
    println!("living members by lineage at tick 0/100/200/300, rule off then on");
    println!("lineage     off: 0   100   200   300 | on: 0   100   200   300   mass(mg) upkeep% time%");
    for k in off[0].keys().filter(|k| k.as_str() != "world:ground") {
        let at = |w: &Vec<std::collections::BTreeMap<String, u64>>| w.iter().map(|m| format!("{:>5}", m.get(k).unwrap_or(&0))).collect::<String>();
        let r = rows.get(k);
        println!("{k:<10} {} | {} {:>9} {:>7} {:>5}", at(&off), at(&on),
            r.map_or("-".into(), |r| r.mass.to_string()), r.map_or("-".into(), |r| r.upkeep.to_string()), r.map_or("-".into(), |r| r.time.to_string()));
    }
}
