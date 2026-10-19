// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The declared foundings and rate profiles A1 draws (after-pass §A1).
//! The candidate is the generator's named preset (907); the original control
//! is its stock rates.

use super::*;
use crate::ecology::Rates;

/// What a draw founds. Bare and bodied are the 2026-10-10 declared three-site
/// worlds; played is Mesocosm's own generated founding, copied from
/// `mesocosm-runtime/src/runtime.rs` (`Founded::generated`), which this crate
/// cannot depend on. Bodied worlds graze by default (905).
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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

    /// Mesocosm's bodies: the default with the probe's hazard, as the
    /// product's founding has them (the 2026-10-18 note to ruling 227).
    pub fn bodies(self) -> Option<Bodies> {
        self.bodied().then(|| Bodies {
            hazard: crate::harm::hazard::RATE,
            wound: crate::harm::hazard::CELLS,
            ..Bodies::default()
        })
    }

    pub(super) fn founding(self, seed: u64, rates: Rates) -> Founding {
        let played = Some(Played { lineage: 1, region_sites: 2 });
        if self == Self::Played {
            let grid = crate::map::Grid { width: 3, height: 2, ..crate::map::Grid::drawn(seed) };
            return Founding {
                seed,
                sites: grid.width * grid.height,
                map: Some(crate::map::Layout::Grid(grid)),
                ecology: true,
                ecology_rates: rates,
                bodies: self.bodies(),
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
            ecology_rates: rates,
            bodies: self.bodies(),
            played,
            ..Default::default()
        }
    }
}

pub(super) fn world(seed: u64, form: Form, original: bool) -> Genesis {
    let rates = if original { Rates::Stock } else { Rates::A1 };
    let founding = form.founding(seed, rates);
    world_from_founding(founding, original, std::env::var("ISOCOSM_REPRO_TWEAK").ok().as_deref())
}

pub(super) fn world_from_founding(
    founding: Founding,
    original: bool,
    adjustment: Option<&str>,
) -> Genesis {
    let mut g = founding.generate().unwrap();
    g.rules.epoch_ticks = EPOCH;
    check(&g, original);
    let lineages = u64::from(g.founding.as_ref().unwrap().lineages);
    tweak(&mut g, lineages, adjustment);
    g
}

/// The control must stay the stock rates and the candidate the preset's, so
/// a later promotion of generator defaults cannot quietly change either arm.
fn check(g: &Genesis, original: bool) {
    let p = &g.rules.processes;
    let (gate, consumer_gate, birth, upkeep, meal, low) =
        if original { (30, 30, 7, 5, 1, 20) } else { (12, 9, 14, 10, 3, 100) };
    for (i, want) in [(0, gate), (1, consumer_gate), (2, gate)] {
        let birth_process = &p[&format!("ecology:birth-{i}")];
        assert!(birth_process.requires.iter().any(|q| matches!(q, Query::Account { at_least, .. } if *at_least == want)));
        assert_eq!(birth_process.period, Some(if i == 1 { birth } else { 7 }));
        assert_eq!(p[&format!("ecology:upkeep-{i}")].period, Some(if i == 1 { upkeep } else { 5 }));
        let ceiling = p[&format!("ecology:age-{i}")].requires.iter().find_map(|q| match q {
            Query::Age { at_least } => Some(*at_least),
            _ => None,
        });
        assert!((low..=low + 40).contains(&ceiling.unwrap()));
        assert_eq!(p.contains_key(&format!("ecology:hazard-{i}")), !original);
        if i != 0 {
            // Producers have no feeding process.
            assert!(matches!(&p[&format!("ecology:feed-{i}")].effects[..], [Effect::Eat {
                amount: Amount::Fixed(m), ..
            }] if *m == if i == 1 { meal } else { 1 }));
        }
    }
}

/// Declared exploratory tweaks, applied after the base checks and only to a
/// run that names them: `ISOCOSM_REPRO_TWEAK` is a comma list of `meal=N`,
/// `upkeep=N`, `feed=N`, `gate=N`, `provision=N`, `bperiod=N` (consumer rates) and
/// `lifebase=N` (every lineage's lifespan ceiling is N plus a draw of 0 to 40). Absent,
/// the world is exactly the generator's stock rates or its A1 preset.
fn tweak(g: &mut Genesis, lineages: u64, list: Option<&str>) {
    let Some(list) = list else { return };
    let seed = g.seed;
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
                // The lifespan ceiling's base (the preset draws base + 0 to 40);
                // the hazard from 60 is the generator's and stays.
                ("lifebase", _) => {
                    let life = n + crate::draw(seed, "lifespan", &[i]) % 41;
                    for q in &mut processes.get_mut(&format!("ecology:age-{i}")).unwrap().requires {
                        if let Query::Age { at_least } = q {
                            *at_least = life;
                        }
                    }
                },
                ("bperiod", 1) => {
                    processes.get_mut(&format!("ecology:birth-{i}")).unwrap().period = Some(n);
                },
                ("gate", 1) => {
                    for q in &mut processes.get_mut(&format!("ecology:birth-{i}")).unwrap().requires {
                        if let Query::Account { at_least, .. } = q {
                            *at_least = n;
                        }
                    }
                },
                ("provision", 1) => {
                    for e in &mut processes.get_mut(&format!("ecology:birth-{i}")).unwrap().effects {
                        if let Effect::Birth { provision } = e {
                            provision.values_mut().for_each(|v| *v = n);
                        }
                    }
                },
                ("meal" | "upkeep" | "feed" | "gate" | "provision" | "bperiod", _) => {},
                _ => panic!("unknown tweak {key}"),
            }
        }
    }
}
