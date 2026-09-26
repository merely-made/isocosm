//! Measures where prey run out part way through a pass of hunters: in the
//! exact runner, a pass at a site where some hunters found prey and some
//! found none; in the crowds, the boundary the real crowd refuses. Worlds and
//! dynamics seeds follow the probe binary's derivation from a master seed.
//! Usage: boundary <master> <draws> [water] [hunters-lo hunters-hi]

use isocosm::probe::{Crowd, PredatorFounding, ProbeFounding, Variant, run_exact};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Default, Debug)]
struct Tally {
    events: u64,
    draws_with: u64,
    mixed: u64,
    first_tick: Vec<u64>,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let master: u64 = args[1].parse().unwrap();
    let draws: u64 = args[2].parse().unwrap();
    let water = args.get(3).is_some_and(|a| a == "water");
    let mut predators = PredatorFounding::default();
    if args.len() >= 6 {
        predators.hunters = [args[4].parse().unwrap(), args[5].parse().unwrap()];
    }
    let mut tallies: BTreeMap<&str, Tally> = BTreeMap::new();
    let mut passes_total = 0u64;
    for k in 0..draws {
        let seed = isocosm::draw(master, "probe-world", &[k]);
        let world = ProbeFounding {
            seed,
            water,
            predators: Some(predators.clone()),
            ..Default::default()
        }
        .generate()
        .unwrap();
        let dynamics = isocosm::draw(master, "probe-dynamics", &[k, 0]);
        isocosm::SEARCHES.lock().unwrap().clear();
        run_exact(&world, dynamics, true).unwrap();
        let log = std::mem::take(&mut *isocosm::SEARCHES.lock().unwrap());
        let mut passes: BTreeMap<(u64, u64, String), (bool, bool, BTreeSet<u64>)> =
            BTreeMap::new();
        for (tick, place, p, found, body) in log {
            let entry = passes.entry((tick, place, p)).or_default();
            if found {
                entry.0 = true;
            } else {
                entry.1 = true;
            }
            entry.2.insert(body);
        }
        passes_total += passes.len() as u64;
        let t = tallies.entry("exact").or_default();
        let events: Vec<_> = passes.iter().filter(|(_, v)| v.0 && v.1).collect();
        t.events += events.len() as u64;
        t.draws_with += u64::from(!events.is_empty());
        t.mixed += events.iter().filter(|(_, v)| v.2.len() > 1).count() as u64;
        if let Some(((tick, _, _), _)) = events.first() {
            t.first_tick.push(*tick);
        }
        for (index, variant, name) in [
            (2u64, Variant::Histogram, "crowd"),
            (3, Variant::Averaged, "averaged"),
            (5, Variant::Unweighted, "unweighted"),
        ] {
            let dynamics = isocosm::draw(master, "probe-dynamics", &[k, index]);
            isocosm::CROWD_BOUNDARIES.lock().unwrap().clear();
            Crowd::new(&world, dynamics, variant).unwrap().run().unwrap();
            let log = std::mem::take(&mut *isocosm::CROWD_BOUNDARIES.lock().unwrap());
            let t = tallies.entry(name).or_default();
            t.events += log.len() as u64;
            t.draws_with += u64::from(!log.is_empty());
            t.mixed += log.iter().filter(|b| b.4 > 1).count() as u64;
            if let Some(b) = log.first() {
                t.first_tick.push(b.0);
            }
        }
    }
    println!(
        "master {master}, draws {draws}, water {water}, hunters {:?}, exact hunting passes {passes_total}",
        predators.hunters
    );
    for (name, t) in &tallies {
        let mut ticks = t.first_tick.clone();
        ticks.sort_unstable();
        println!(
            "{name:10} boundary events {:5}, draws with one {:4} of {draws}, with hunters in more than one state {:5}, first ticks {:?}",
            t.events, t.draws_with, t.mixed, ticks
        );
    }
}
