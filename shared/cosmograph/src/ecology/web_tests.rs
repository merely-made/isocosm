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

/// SC1's measured assessment, as an arm only: a feeder can take nothing
/// from a living producer that would leave it below `reserve`, as a plant
/// regrows from what grazing cannot reach.
fn with_reserve(mut g: crate::simulation::Genesis, reserve: u64) -> crate::simulation::Genesis {
    let producers: Vec<String> = g
        .lineages
        .iter()
        .filter(|(_, l)| l.traits.contains("life:producer"))
        .map(|(k, _)| k.clone())
        .collect();
    for p in g.rules.processes.values_mut() {
        let Some(t) = &p.target else { continue };
        if t.alive != Some(true) || !t.among.iter().any(|l| producers.contains(l)) {
            continue;
        }
        for q in &mut p.requires {
            if let Query::Holds { who: Binding::Target, at_least } = q {
                *at_least += reserve;
            }
        }
    }
    g
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
            let mut s = Session::new(g, Execution::Individuals).unwrap();
            let names: Vec<String> = (0..lineages)
                .map(|i| format!("{:?}", Role::of(web, i)).to_lowercase())
                .collect();
            println!("seed {seed}, {web:?}, reserve {reserve}: members of {}", names.join(", "));
            let mut row = Vec::new();
            for t in 0..=12 {
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
    }
}
