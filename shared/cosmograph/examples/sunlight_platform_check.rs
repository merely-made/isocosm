// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use cosmograph::{Execution, Founding, Session, light::SunRules, rules::year_ticks};

fn witness() -> u64 {
    let mut genesis = Founding {
        seed: 9,
        sites: 3,
        lineages: 3,
        population: 24,
        cohort_size: 1,
        ecology: true,
        ..Default::default()
    }
    .generate()
    .unwrap();
    genesis
        .rules
        .processes
        .retain(|id, _| id == "ecology:sunlight");
    for site in genesis.sites.values_mut() {
        site.accounts.insert("sim:energy".into(), 0);
    }
    let year = year_ticks(genesis.rules.tick_microseconds());
    let day = 1_440;
    let mut hashes = Vec::new();
    for season in [0, year / 4, year / 2, 3 * year / 4] {
        let start = season - season % day;
        for hour in [4, 5, 6, 17, 18, 19] {
            genesis.rules.sun = Some(SunRules {
                start_hour: hour,
                ..SunRules::default()
            });
            genesis.started = start;
            for group in genesis.population.groups.values_mut() {
                group.entity_mut().arrived = start;
            }
            let mut world = Session::new(genesis.clone(), Execution::Individuals).unwrap();
            world.advance(60).unwrap();
            let mut restored = Session::load(world.save(), Execution::Grouped).unwrap();
            restored.advance(60).unwrap();
            world.advance(60).unwrap();
            assert_eq!(restored.sim.witness(), world.sim.witness());
            hashes.push(world.sim.state_hash());
        }
    }
    cosmograph::draw(0, "sunlight-platform-check", &hashes)
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn sunlight_witness() -> u64 {
    witness()
}

fn main() {
    println!("{:016x}", witness());
}
