// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Same-line bodies differ only through the parts a native meal retains.
use super::*;

fn scruples(s: &Session, id: Id) -> BTreeMap<PartId, Ledger> {
    body(s, id).parts.iter().map(|(&id, part)| {
        (id, cosmograph::matter::scruple::of(part, &s.sim.genesis().rules).unwrap())
    }).collect()
}

#[test]
fn different_meals_change_same_line_parts_and_replay_keeps_each_source() {
    for mode in [Execution::Individuals, Execution::Grouped] {
        let (mut g, grazers, prey) = pass_world(0, [1000, 1000]);
        let of = ["tissue:0".to_string()];
        g.seed = (0..128).find(|seed| {
            anatomy::bitten(g.population.get(prey).unwrap(), &of,
                cosmograph::draw(*seed, "bite", &[0])) == Some(PartId(1))
        }).expect("a bounded fixture draws the leaf part");
        let mut s = Session::new(g, mode).unwrap();
        let baseline = scruples(&s, grazers[0]);
        assert_eq!(baseline, scruples(&s, grazers[1]), "same founding tissue");
        let total = s.sim.matter();
        let command = |actor| Command::Act {
            actor, target: Some(prey), process: "test:graze".into(), cause: None,
        };
        let meal = s.command(command(grazers[0])).unwrap();
        let result: serde_json::Value = serde_json::from_str(&meal).unwrap();
        assert_eq!(result["outcome"], "Accepted", "the native meal must land");
        assert_eq!(body(&s, grazers[0]).lineage, body(&s, grazers[1]).lineage);
        assert_ne!(scruples(&s, grazers[0]), scruples(&s, grazers[1]));
        assert_eq!(scruples(&s, grazers[1]), baseline, "unfed control is unchanged");
        assert!(scruples(&s, grazers[0]).values().any(|mix| mix.contains_key("lineage:0")),
            "the incorporated tissue still names the producer line");
        assert_eq!(s.sim.matter(), total);
        let saved = s.save();
        for replay in [Execution::Individuals, Execution::Grouped] {
            let loaded = Session::load(saved.clone(), replay).unwrap();
            for id in grazers {
                assert_eq!(scruples(&loaded, id), scruples(&s, id));
            }
            assert_eq!(loaded.save().state_hash, saved.state_hash);
        }
        let hash = s.sim.state_hash();
        let _ = scruples(&s, grazers[0]);
        assert_eq!(s.sim.state_hash(), hash, "the reading cannot affect identity");
    }
}

#[test]
fn equal_native_meals_have_equal_scruples() {
    let (left, eater, _, accepted) = graze(0, 1000, true);
    let (right, control, _, also_accepted) = graze(0, 1000, true);
    assert_eq!(accepted, "\"Accepted\"");
    assert_eq!(also_accepted, accepted);
    assert_eq!(body(&left, eater).lineage, body(&right, control).lineage);
    assert_eq!(scruples(&left, eater), scruples(&right, control));
}
