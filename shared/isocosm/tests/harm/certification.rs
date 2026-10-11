// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use isocosm::probe::readings;

#[test]
fn declared_harm_traits_keep_columns_when_drawn_present_or_absent() {
    let mut columns = None;
    let mut present = [false; 2];
    let mut absent = [false; 2];
    for seed in 1..=64 {
        let world = BodyFounding {
            seed,
            harm: Some(HarmFounding::default()),
            ..Default::default()
        }
        .generate()
        .unwrap();
        let derived = readings::derive(&world);
        let similitude = world.similitude().unwrap();
        let keys: Vec<_> = derived
            .iter()
            .map(|r| (r.key.clone(), similitude.bound(&r.key)))
            .collect();
        assert_eq!(
            columns.get_or_insert(keys.clone()).as_slice(),
            keys.as_slice()
        );
        let members: Vec<_> = world
            .genesis
            .population
            .groups
            .values()
            .filter(|c| c.entity.kingdom != "kingdom:world")
            .map(|c| (&c.entity, c.count))
            .collect();
        let values = readings::evaluate(
            &derived,
            &world,
            &members,
            &world.genesis.sites,
            0,
            readings::inspect(&members, 10),
            &Default::default(),
        )
        .unwrap();
        for (j, identity) in [harm::HEAL, harm::FRAGMENT].into_iter().enumerate() {
            let actual: u64 = members
                .iter()
                .filter(|(e, _)| e.alive && e.traits.contains(identity))
                .map(|(_, n)| n)
                .sum();
            let i = keys
                .iter()
                .position(|(key, _)| key == &format!("alive:{identity}"))
                .unwrap();
            assert_eq!(values[i], actual);
            present[j] |= actual > 0;
            absent[j] |= actual == 0;
        }
    }
    assert_eq!(present, [true; 2]);
    assert_eq!(absent, [true; 2]);
}
