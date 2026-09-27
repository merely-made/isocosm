// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Part shapes and the function catalogue (rulings 276, 338 to 341), and the
//! process's causal kind that gave up the name `Shape` for them (ruling 340).

use isocosm::{Execution, Session, history::Saved, rules::*};

/// A session saved by the core before ruling 340, its processes' causal
/// kinds written under the old name.
const PRE_CAUSATION: &str = include_str!("data/pre-causation-world.json");

#[test]
fn a_world_saved_before_the_rename_still_loads() {
    for mode in [Execution::Individuals, Execution::Grouped] {
        let saved: Saved = serde_json::from_str(PRE_CAUSATION).unwrap();
        let hash = saved.state_hash.clone();
        // Loading checks the genesis digest, replays every command and
        // advance, and compares the state hash and every checkpoint.
        let session = Session::load(saved, mode).unwrap();
        assert_eq!(session.sim.state_hash(), hash, "{mode:?}");
        let rules = &session.sim.genesis().rules;
        let causation = |id: &str| rules.processes[id].causation;
        assert_eq!(causation("sim:remember"), Causation::Choice);
        assert_eq!(causation("ecology:death-0"), Causation::Transition);
        // Saved again, it writes the field under its old name.
        let json = serde_json::to_string(&session.save()).unwrap();
        assert!(json.contains(r#""shape":"Transition""#));
        assert!(!json.contains("causation"));
    }
}
