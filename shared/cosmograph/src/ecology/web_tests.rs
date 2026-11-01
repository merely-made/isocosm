// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A web of diets beside the web of levels.

use super::*;
use crate::{Execution, Session, directing::Played, lineage::boundary};

fn founding(seed: u64, web: Web, lineages: u32) -> Founding {
    Founding {
        seed,
        sites: 8,
        population: 200,
        cohort_size: 4,
        lineages,
        ecology: true,
        ecology_rates: Rates::A1,
        web,
        played: Some(Played {
            lineage: 1,
            region_sites: 4,
        }),
        ..Default::default()
    }
}

#[test]
fn a_web_of_levels_founds_as_before() {
    let named = founding(9, Web::Levels, 3).generate().unwrap();
    let mut plain = founding(9, Web::Levels, 3);
    let json = serde_json::to_string(&plain).unwrap();
    assert!(!json.contains("web"), "levels serialize as before");
    plain = serde_json::from_str(&json).unwrap();
    assert_eq!(crate::digest(&named), crate::digest(&plain.generate().unwrap()));
}

#[test]
fn a_web_of_diets_feeds_each_role_on_its_prey() {
    let g = founding(9, Web::Diets, 5).generate().unwrap();
    let among = |i: u32| -> Vec<String> {
        let t = g.rules.processes[&format!("ecology:feed-{i}")].target.clone().unwrap();
        t.among.into_iter().collect()
    };
    assert_eq!(among(1), vec!["lineage:0"], "the grazer eats producers");
    assert_eq!(among(3), vec!["lineage:1", "lineage:4"], "the predator eats grazers and omnivores");
    assert_eq!(among(4), vec!["lineage:0", "lineage:1"], "the omnivore eats producers and grazers");
    assert_eq!(among(2).len(), 4, "the decomposer eats the dead of all others");
    assert!(!g.rules.processes.contains_key("ecology:feed-0"), "producers do not feed");
    for i in [1, 3, 4] {
        assert!(g.lineages[&format!("lineage:{i}")].traits.contains("life:consumer"));
    }
}

/// The feeds that eat living producers: the grazer's and the omnivore's.
fn producer_feeds(g: &Genesis) -> Vec<String> {
    let producers: Vec<&String> = g
        .lineages
        .iter()
        .filter(|(_, l)| l.traits.contains("life:producer"))
        .map(|(k, _)| k)
        .collect();
    g.rules
        .processes
        .values()
        .filter(|p| {
            p.target
                .as_ref()
                .is_some_and(|t| t.alive == Some(true) && t.among.iter().any(|l| producers.contains(&l)))
        })
        .map(|p| p.id.clone())
        .collect()
}

/// SC1's measured assessment, as an arm only: a feeder can take nothing
/// from a living producer that would leave it below `reserve`, as a plant
/// regrows from what grazing cannot reach.
fn with_reserve(mut g: Genesis, reserve: u64) -> Genesis {
    for id in producer_feeds(&g) {
        for q in &mut g.rules.processes.get_mut(&id).unwrap().requires {
            if let Query::Holds { who: Binding::Target, at_least } = q {
                *at_least += reserve;
            }
        }
    }
    g
}

/// A reserve that grazing cannot push below the producer's birth gate: a
/// feeder takes only what a producer holds above it.
fn above_gate(g: Genesis) -> Genesis {
    let gate = Rates::A1.birth_gate(Founding::default().stock_min, false);
    with_reserve(g, gate + Rates::A1.meal(true) - 1)
}

/// SC1 keyed on place, as an arm only: one site in `every` is a refuge where
/// nothing eats a living producer. The omnivore cannot feed there at all,
/// since its one feed admits grazers too.
fn with_refuges(mut g: Genesis, every: usize) -> Genesis {
    let grazed = "arm:grazed".to_owned();
    g.rules.conditions.insert(grazed.clone());
    for (n, site) in g.sites.values_mut().enumerate() {
        if n % every != 0 {
            site.conditions.insert(grazed.clone(), 1);
        }
    }
    for id in producer_feeds(&g) {
        g.rules.processes.get_mut(&id).unwrap().requires.push(Query::Condition {
            key: grazed.clone(),
            at_least: 1,
        });
    }
    g
}

/// SC2's interference on the bite, as an arm only: a site affords `bites`
/// meals of living producers a tick, up to ten ticks' worth, shared among
/// whoever feeds there. The omnivore's meals of grazers draw on it too.
fn with_forage(mut g: Genesis, bites: u64) -> Genesis {
    let forage = "arm:forage".to_owned();
    g.rules.accounts.insert(forage.clone(), AccountKind::Time);
    let mut regrow = process(
        "arm:forage",
        Causation::Agentless,
        vec![Effect::Transform {
            who: Binding::Place,
            take: BTreeMap::new(),
            give: BTreeMap::from([(forage.clone(), bites)]),
            conversion: None,
        }],
    );
    regrow.requires = vec![Query::Below {
        who: Binding::Place,
        key: forage.clone(),
        amount: bites * 10,
    }];
    regrow.period = Some(1);
    regrow.priority = -2;
    g.rules.processes.insert(regrow.id.clone(), regrow);
    for id in producer_feeds(&g) {
        let feed = g.rules.processes.get_mut(&id).unwrap();
        feed.requires.push(Query::Account {
            who: Binding::Place,
            key: forage.clone(),
            at_least: 1,
        });
        feed.effects.push(Effect::Transform {
            who: Binding::Place,
            take: BTreeMap::from([(forage.clone(), 1)]),
            give: BTreeMap::new(),
            conversion: None,
        });
    }
    g
}

/// Producers stand still, as the forms-of-life brief has a fed producer do:
/// every route's move asks for a trait only the other roles carry. Without
/// it a producer below its hunger line, as each parent is after a birth,
/// takes the routes like any member.
fn sessile(mut g: Genesis) -> Genesis {
    let moves = "arm:moves".to_owned();
    g.rules.traits.insert(moves.clone());
    for group in g.population.groups.values_mut() {
        let e = group.entity_mut();
        if e.kingdom != "kingdom:world" && !e.traits.contains("life:producer") {
            e.traits.insert(moves.clone());
        }
    }
    for p in g.rules.processes.values_mut().filter(|p| p.id.starts_with("move:")) {
        p.requires.push(Query::Trait {
            who: Binding::Actor,
            key: moves.clone(),
        });
    }
    g
}

/// Members of each role every 30 ticks of `g`, to `ticks` or extinction.
fn census(g: Genesis, web: Web, lineages: u32, label: &str, ticks: u64) {
    let mut s = Session::new(g, Execution::Individuals).unwrap();
    let names: Vec<String> = (0..lineages)
        .map(|i| format!("{:?}", Role::of(web, i)).to_lowercase())
        .collect();
    println!("{label}: members of {}", names.join(", "));
    let mut row = Vec::new();
    for t in 0..=ticks / 30 {
        let counts: Vec<String> = (0..lineages)
            .map(|i| boundary::standing(&s, &format!("lineage:{i}")).1.to_string())
            .collect();
        row.push(format!("{:>3}: {}", t * 30, counts.join("/")));
        if counts.iter().all(|c| c == "0") {
            break;
        }
        s.advance(30).unwrap();
    }
    println!("  {}", row.join("  "));
}

/// Members of each role every 30 ticks: a web of levels, a web of diets,
/// and a web of diets with SC1's reserve, on the same seeds.
/// `cargo test web_readout -- --ignored --nocapture`.
#[test]
#[ignore = "a readout, not a control"]
fn web_readout() {
    for seed in [9, 3, 7] {
        for (web, lineages, reserve) in [(Web::Levels, 3, 0), (Web::Diets, 5, 0), (Web::Diets, 5, 6)] {
            let g = with_reserve(founding(seed, web, lineages).generate().unwrap(), reserve);
            census(g, web, lineages, &format!("seed {seed}, {web:?}, reserve {reserve}"), 360);
        }
    }
}

/// The refuges SC1 and SC2 might take on a web of diets: producers walking
/// and standing still, a refuge in one site of four on each, and on sessile
/// producers a reserve above the birth gate and a bite a site per tick.
/// `cargo test refuge_readout -- --ignored --nocapture`.
#[test]
#[ignore = "a readout, not a control"]
fn refuge_readout() {
    type Arm = fn(Genesis) -> Genesis;
    let arms: [(&str, Arm); 6] = [
        ("walking", |g| g),
        ("walking, refuge 1 in 4", |g| with_refuges(g, 4)),
        ("sessile", sessile),
        ("sessile, above the gate", |g| above_gate(sessile(g))),
        ("sessile, refuge 1 in 4", |g| with_refuges(sessile(g), 4)),
        ("sessile, a bite a tick", |g| with_forage(sessile(g), 1)),
    ];
    for seed in [9, 3, 7] {
        for (label, arm) in arms {
            let g = arm(founding(seed, Web::Diets, 5).generate().unwrap());
            census(g, Web::Diets, 5, &format!("seed {seed}, {label}"), 600);
        }
    }
}

/// What the consumers lack once producers hold, on sessile producers: a
/// lower reserve, and a refuge in a world whose predators cannot feed.
/// `cargo test consumer_readout -- --ignored --nocapture`.
#[test]
#[ignore = "a readout, not a control"]
fn consumer_readout() {
    type Arm = fn(Genesis) -> Genesis;
    let arms: [(&str, Arm); 3] = [
        ("sessile, reserve 6", |g| with_reserve(sessile(g), 6)),
        ("sessile, reserve 9", |g| with_reserve(sessile(g), 9)),
        ("sessile, refuge 1 in 4, predators unfed", |g| unfed(with_refuges(sessile(g), 4), Role::Predator)),
    ];
    for seed in [9, 3, 7] {
        for (label, arm) in arms {
            let g = arm(founding(seed, Web::Diets, 5).generate().unwrap());
            census(g, Web::Diets, 5, &format!("seed {seed}, {label}"), 600);
        }
    }
}

/// A role's feed taken away, so its members live on what they hold.
fn unfed(mut g: Genesis, role: Role) -> Genesis {
    for i in (0..5).filter(|&i| Role::of(Web::Diets, i) == role) {
        g.rules.processes.remove(&format!("ecology:feed-{i}"));
    }
    g
}
