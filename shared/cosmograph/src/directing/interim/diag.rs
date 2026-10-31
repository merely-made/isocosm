// Scratch diagnostic (repro lane): a consumer's income, upkeep and age.
use super::*;
use crate::Founding;
use crate::directing::found::Played;
use crate::flows::{Holder, MadeBy};
use std::collections::BTreeMap;

fn bare(seed: u64) -> Genesis {
    Founding {
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
    .unwrap()
}

fn bodied(seed: u64) -> Genesis {
    let grid = crate::map::Grid {
        width: 3,
        height: 2,
        ..crate::map::Grid::drawn(seed)
    };
    Founding {
        seed,
        sites: 6,
        map: Some(crate::map::Layout::Grid(grid)),
        ecology: true,
        bodies: Some(crate::bodied::Bodies::default()),
        played: Some(Played {
            lineage: 1,
            region_sites: 2,
        }),
        ..Default::default()
    }
    .generate()
    .unwrap()
}

#[derive(Default, Debug)]
struct Life {
    born: Tick,
    income: BTreeMap<Key, u64>,
    outgo: BTreeMap<Key, u64>,
    births: u64,
    died: Option<Tick>,
    last_matter: u128,
}

fn run(label: &str, mut g: Genesis, ticks: u64) {
    g.rules.epoch_ticks = 1_000_000;
    let lifespan = g.rules.processes.get("ecology:age-1").map(|p| {
        p.requires.iter().find_map(|q| match q {
            crate::rules::Query::Age { at_least } => Some(*at_least),
            _ => None,
        })
    });
    let pace = Pace {
        round: 1,
        scoring: 1,
    };
    let start = Start { within: 0, epochs: 0 };
    {
        let sim = crate::Simulation::new(g.clone(), Execution::Individuals).unwrap();
        let want = crate::directing::readings::habitable::conditions(&sim, "lineage:1");
        let sites: Vec<_> = sim.state().sites.values().map(|s| s.conditions.clone()).collect();
        eprintln!("## {label}: wants {want:?}
   sites {sites:?}");
    }
    let found = Interim::found(g, start, Mode::Creative, pace, Execution::Individuals);
    let Ok(mut i) = found else { eprintln!("## {label}: {:?}", found.err()); return; };
    let mut lives: BTreeMap<Id, Life> = BTreeMap::new();
    let consumer = |e: &Entity| e.lineage == "lineage:1";
    let mut series = Vec::new();
    for _ in 0..ticks {
        let r = i.session.advance_tick_with_flows();
        let Ok(r) = r else {
            eprintln!("{label}: advance failed {:?}", r.err());
            break;
        };
        let now = i.session.sim.state().tick;
        let pop = &i.session.sim.state().population;
        for f in &r.flows {
            let MadeBy::Process(k) = &f.made_by else { continue };
            if let Some(id) = f.to.0.body() {
                if !f.internal() && pop.get(id).is_some_and(consumer) {
                    *lives.entry(id).or_default().income.entry(k.clone()).or_default() += f.amount;
                }
            }
            if let Some(id) = f.from.0.body() {
                if !f.internal() && pop.get(id).is_some_and(consumer) {
                    let l = lives.entry(id).or_default();
                    *l.outgo.entry(k.clone()).or_default() += f.amount;
                    if k.contains("birth") || matches!(f.to.0, Holder::Entity(t) if t != id) {
                        if k.contains("birth") {
                            l.births += 1;
                        }
                    }
                }
            }
        }
        let rules = &i.session.sim.genesis().rules;
        let mut living = 0;
        for (id, c) in &pop.groups {
            let e = &c.entity;
            if !consumer(e) {
                continue;
            }
            let l = lives.entry(*id).or_default();
            l.born = e.born;
            if e.alive {
                living += c.count;
                l.last_matter = crate::meaning::mass(&crate::anatomy::books(e), rules);
            } else if l.died.is_none() {
                l.died = Some(now);
            }
        }
        series.push(living);
        if living == 0 {
            break;
        }
    }
    eprintln!(
        "## {label}: lifespan {lifespan:?}, living consumers by 10 ticks {:?}",
        series.iter().step_by(10).collect::<Vec<_>>()
    );
    for (id, l) in lives.iter().take(12) {
        eprintln!(
            "  #{id} born {} died {:?} age {:?} in {:?} out {:?} births {} matter {}",
            l.born,
            l.died,
            l.died.map(|d| d - l.born),
            l.income,
            l.outgo,
            l.births,
            l.last_matter
        );
    }
}

#[test]
#[ignore]
fn consumer_income_upkeep_age() {
    for seed in [101u64, 202, 303, 7] {
        run(&format!("bare {seed}"), bare(seed), 200);
        run(&format!("bodied {seed}"), bodied(seed), 200);
    }
}

#[test]
#[ignore]
fn bodied_feed_probe() {
    for seed in [101u64, 303, 7] {
        let g = bodied(seed);
        let mut sim = crate::Simulation::new(g, Execution::Individuals).unwrap();
        let rules = sim.genesis().rules.clone();
        let pop = sim.state().population.clone();
        let c = pop.groups.iter().find(|(_, c)| c.entity.lineage == "lineage:1" && c.entity.alive);
        let Some((cid, c)) = c else { continue };
        let e = &c.entity;
        eprintln!("## seed {seed} consumer #{cid} at {} books {:?} room {} mass {}", e.place,
            crate::anatomy::books(e), crate::anatomy::room(e, &rules, "matter:1-0"),
            crate::meaning::mass(&crate::anatomy::books(e), &rules));
        let lineage1: Vec<_> = rules.processes.keys().filter(|k| k.ends_with("-1") || k.contains("1-")).collect();
        eprintln!("   processes naming 1: {lineage1:?}");
        let prey = pop.groups.iter().find(|(_, p)| p.entity.lineage == "lineage:0" && p.entity.place == e.place);
        if let Some((pid, p)) = prey {
            eprintln!("   prey #{pid} books {:?}", crate::anatomy::books(&p.entity));
            let r = sim.execute(*cid, Some(*pid), "ecology:feed-1", None);
            eprintln!("   feed outcome {:?} read {:?}", r.outcome, r.facts_read);
            let after = sim.state().population.get(*cid).unwrap();
            let prey_after = sim.state().population.get(*pid).unwrap();
            eprintln!("   consumer after {:?} parts {:?}", crate::anatomy::books(after), after.parts.values().map(|p| p.matter.clone()).collect::<Vec<_>>());
            eprintln!("   prey after {:?}", crate::anatomy::books(prey_after));
        } else {
            eprintln!("   no producer at its site");
        }
    }
}

#[test]
#[ignore]
fn bodied_choice_probe() {
    for (label, g) in [("bare", bare(303)), ("bodied", bodied(303))] {
        let mut s = crate::Session::new(g, Execution::Individuals).unwrap();
        let cid = *s.sim.state().population.groups.iter()
            .find(|(_, c)| c.entity.lineage == "lineage:1").unwrap().0;
        for _ in 0..16 {
            let e = s.sim.state().population.get(cid).unwrap().clone();
            let c = s.sim.consider(cid, false);
            let rules = &s.sim.genesis().rules;
            eprintln!("## {label} t{} #{cid} at {} mass {} mood {:?} chose {:?} {:?}",
                s.sim.state().tick, e.place, crate::meaning::mass(&crate::anatomy::books(&e), rules),
                s.sim.minded(cid).0, c.process, c.options);
            s.advance(1).unwrap();
        }
    }
}
