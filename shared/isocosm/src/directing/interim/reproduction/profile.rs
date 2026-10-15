// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The declared foundings and rate profiles A1 draws (after-pass §A1).

use super::*;

/// What a draw founds. Bare and bodied are the 2026-10-10 declared three-site
/// worlds; played is Mesocosm's own generated founding, copied from
/// `mesocosm-runtime/src/runtime.rs` (`Founded::generated`), which this crate
/// cannot depend on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub(super) enum Form {
    Bare,
    Bodied,
    Played,
}

impl Form {
    pub fn name(self) -> &'static str {
        match self {
            Self::Bare => "bare",
            Self::Bodied => "bodied",
            Self::Played => "played",
        }
    }

    pub fn bodied(self) -> bool {
        self != Self::Bare
    }

    fn founding(self, seed: u64) -> Founding {
        let played = Some(Played { lineage: 1, region_sites: 2 });
        if self == Self::Played {
            let grid = crate::map::Grid { width: 3, height: 2, ..crate::map::Grid::drawn(seed) };
            return Founding {
                seed,
                sites: grid.width * grid.height,
                map: Some(crate::map::Layout::Grid(grid)),
                ecology: true,
                bodies: Some(Bodies::default()),
                played,
                ..Default::default()
            };
        }
        Founding {
            seed,
            sites: 3,
            population: 60,
            cohort_size: 2,
            lineages: 3,
            ecology: true,
            bodies: self.bodied().then(Bodies::default),
            played,
            ..Default::default()
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Cadence {
    Original,
    Candidate,
}

impl Cadence {
    fn birth(self) -> u64 {
        match self {
            Self::Original => 30,
            Self::Candidate => 12,
        }
    }

    fn birth_period(self, lineage: u64) -> Tick {
        match (self, lineage) {
            (Self::Candidate, 1) => 14,
            _ => 7,
        }
    }

    fn meal(self, lineage: u64) -> u64 {
        match (self, lineage) {
            (Self::Candidate, 1) => 3,
            _ => 1,
        }
    }

    fn lifespan(self, seed: u64, lineage: u64) -> Tick {
        let base = match self {
            Self::Original => 20,
            Self::Candidate => 80,
        };
        base + crate::draw(seed, "lifespan", &[lineage]) % 41
    }
}

pub(super) fn world(seed: u64, form: Form, original: bool) -> Genesis {
    let mut g = form.founding(seed).generate().unwrap();
    g.rules.epoch_ticks = EPOCH;
    // Absolute profiles preserve the unchanged old control after any later
    // promotion of generator defaults. These are the declared stock_min=8
    // rates from ecology.rs. The candidate consumer has meal 3 and birth
    // period 14; other birth periods, provision and upkeep stay fixed.
    let cadence = if original { Cadence::Original } else { Cadence::Candidate };
    let lineages = u64::from(g.founding.as_ref().unwrap().lineages);
    let processes = &mut g.rules.processes;
    for i in 0..lineages {
        let birth = processes.get_mut(&format!("ecology:birth-{i}")).unwrap();
        birth.period = Some(cadence.birth_period(i));
        for q in &mut birth.requires {
            if let Query::Account { at_least, .. } = q {
                *at_least = cadence.birth();
            }
        }
        let age = processes.get_mut(&format!("ecology:age-{i}")).unwrap();
        for q in &mut age.requires {
            if let Query::Age { at_least } = q {
                *at_least = cadence.lifespan(seed, i);
            }
        }
        if i % 3 != 0 {
            let feed = processes.get_mut(&format!("ecology:feed-{i}")).unwrap();
            for effect in &mut feed.effects {
                if let Effect::Eat { amount, .. } = effect {
                    *amount = Amount::Fixed(cadence.meal(i));
                }
            }
        }
    }
    check(&g, cadence, lineages);
    tweak(&mut g, lineages);
    g
}

/// Declared exploratory tweaks (declaration 2), after the base checks:
/// `ISOCOSM_REPRO_TWEAK` is a comma list of `meal=N`, `upkeep=N`, `feed=N`,
/// `yield=N`. Absent, the profile is exactly declaration 1's.
fn tweak(g: &mut Genesis, lineages: u64) {
    let Ok(list) = std::env::var("ISOCOSM_REPRO_TWEAK") else { return };
    let processes = &mut g.rules.processes;
    for pair in list.split(',').filter(|p| !p.is_empty()) {
        let (key, value) = pair.split_once('=').expect("key=value");
        let n: u64 = value.parse().expect("integer");
        for i in 0..lineages {
            match (key, i % 3) {
                ("meal", 1) => {
                    for e in &mut processes.get_mut(&format!("ecology:feed-{i}")).unwrap().effects {
                        if let Effect::Eat { amount, .. } = e {
                            *amount = Amount::Fixed(n);
                        }
                    }
                },
                ("upkeep", 1) => {
                    processes.get_mut(&format!("ecology:upkeep-{i}")).unwrap().period = Some(n);
                },
                ("feed", 1) => {
                    processes.get_mut(&format!("ecology:feed-{i}")).unwrap().period = Some(n);
                },
                ("yield", 0) => {
                    for e in &mut processes.get_mut(&format!("ecology:produce-{i}")).unwrap().effects {
                        match e {
                            Effect::Transfer { amount, .. } => *amount = Amount::Fixed(n),
                            Effect::Transform { take, give, .. } => {
                                take.values_mut().for_each(|v| *v = n);
                                give.values_mut().for_each(|v| *v = n);
                            },
                            _ => {},
                        }
                    }
                },
                ("meal" | "upkeep" | "feed" | "yield", _) => {},
                _ => panic!("unknown tweak {key}"),
            }
        }
    }
}

fn check(g: &Genesis, cadence: Cadence, lineages: u64) {
    use std::collections::BTreeMap;
    for i in 0..lineages {
        let birth = &g.rules.processes[&format!("ecology:birth-{i}")];
        assert_eq!(birth.period, Some(cadence.birth_period(i)));
        assert!(matches!(&birth.effects[..], [Effect::Birth { provision }]
            if provision == &BTreeMap::from([(format!("matter:{i}-0"), 8)])));
        let upkeep = &g.rules.processes[&format!("ecology:upkeep-{i}")];
        assert_eq!(upkeep.period, Some(5));
        assert!(matches!(&upkeep.effects[0], Effect::Transform { take, .. }
            if take == &BTreeMap::from([(format!("matter:{i}-0"), 1)])));
        if i % 3 != 0 {
            let feed = &g.rules.processes[&format!("ecology:feed-{i}")];
            assert_eq!(feed.period, Some(3));
            assert!(matches!(&feed.effects[..], [Effect::Eat {
                amount: Amount::Fixed(amount), ..
            }] if *amount == cadence.meal(i)));
        }
    }
}
