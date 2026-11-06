// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[test]
fn actual_founded_predators_digest_other_predators_across_small_parts() {
    for mode in [Execution::Individuals, Execution::Grouped] {
        let (g, eater, prey) = controlled(3, 8);
        let mut direct = Session::new(g.clone(), mode).unwrap();
        let mut scheduled = Session::new(g, mode).unwrap();
        let total = direct.sim.matter();
        let (food, tissue) = ("matter:8-0", "matter:3-0");
        let prey_start = held(&direct, prey, food);
        let tissue_start = held(&direct, eater, tissue);
        let waste_start = site_held(&direct, DETRITUS);
        assert_eq!((prey_start, tissue_start), (15, 15));
        // Keep the ordinary generated mouth, body ledgers, bite and yield.
        // Only the clock and population are controlled. Taking every unit
        // of this natural predator must eventually digest five units, even
        // where the chosen body parts contain less than three each.
        let mut bites = 0;
        while held(&direct, prey, food) > 0 {
            let before = held(&direct, prey, food);
            let receipt = act(&mut direct, eater, Some(prey), "ecology:feed-3");
            assert!(receipt.accepted(), "{mode:?}: {receipt:?}");
            let work = scheduled.advance(1).unwrap();
            assert_eq!((work.accepted, work.blocked), (1, 0), "{work:?}");
            assert!(held(&direct, prey, food) < before);
            for id in [eater, prey] {
                assert_eq!(books(&direct, id), books(&scheduled, id));
            }
            assert_eq!(
                site_held(&direct, DETRITUS),
                site_held(&scheduled, DETRITUS)
            );
            assert_eq!(direct.sim.matter(), total);
            assert_eq!(scheduled.sim.matter(), total);
            replay(&scheduled);
            bites += 1;
            assert!(
                bites <= prey_start,
                "each positive bite removes at least one unit"
            );
        }
        assert_eq!(held(&direct, eater, tissue) - tissue_start, 5);
        assert_eq!(site_held(&direct, DETRITUS) - waste_start, 10);
        assert_eq!(
            held(&direct, prey, "world:soil"),
            4,
            "soil is not animal food"
        );
        replay(&direct);
    }
}
