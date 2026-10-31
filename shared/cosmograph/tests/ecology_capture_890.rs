// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Ruling 890: the generator gives producers a real capture path. A default
//! generated ecology, bare or bodied, passes the 875 joint witness's four
//! roles on one trajectory, and the capture role is detected: a world whose
//! producers capture nothing does not.

use cosmograph::{
    Execution, Founding,
    bodied::Bodies,
    ecology::*,
    rules::*,
    schema::*,
    simulation::Genesis,
};

fn selection(process: &str, actor: u32, target: Option<u32>) -> Selection {
    Selection {
        process: process.into(),
        actor_lineage: format!("lineage:{actor}"),
        target_lineage: target.map(|t| format!("lineage:{t}")),
    }
}

/// The default ecology's four roles: producers capture light, return soil at
/// their upkeep, are born; consumers' meals debit their prey.
fn spec(horizon: Tick) -> JointSpec {
    JointSpec {
        version: 1,
        key: "test:joint-890".into(),
        horizon,
        capture: CaptureRequirement {
            selection: selection("ecology:produce-0", 0, None),
            stock: CaptureStock::EnergyAccount("sim:energy".into()),
            at_least: 1,
        },
        returned: ReturnRequirement {
            selection: selection("ecology:upkeep-0", 0, None),
            account: "world:soil".into(),
            at_least: 1,
        },
        reproduction: BirthRequirement { selection: selection("ecology:birth-0", 0, None), at_least: 1 },
        counterplay: CounterRequirement {
            selection: selection("ecology:feed-1", 1, Some(0)),
            proof: CounterProof::TargetDebited { account: "matter:0-0".into(), at_least: 1 },
        },
        independence: None,
    }
}

fn world(seed: u64, bodied: bool, horizon: Tick) -> Genesis {
    world_with(seed, bodied.then(Bodies::default), horizon)
}

fn world_with(seed: u64, bodies: Option<Bodies>, horizon: Tick) -> Genesis {
    let mut g = Founding {
        seed,
        sites: 3,
        population: 60,
        cohort_size: 2,
        lineages: 3,
        ecology: true,
        bodies,
        ..Default::default()
    }
    .generate()
    .unwrap();
    g.rules.ecology_witness = Some(spec(horizon));
    g.validate().unwrap();
    g
}

fn witness(g: &Genesis, horizon: Tick) -> cosmograph::Result<JointWitness> {
    run_joint(g, &spec(horizon), Execution::Individuals, &mut |s| s.advance_tick_with_flows())
}

#[test]
fn default_generated_worlds_pass_the_joint_witness_with_real_capture() {
    // Bare worlds and bodied worlds (which graze by default, 905) pass all
    // four roles.
    for (label, bodies) in [("bare", None), ("bodied", Some(Bodies::default()))] {
        for seed in 0..4 {
            let g = world_with(seed, bodies.clone(), 60);
            let w = witness(&g, 60).unwrap();
            w.require_roles().unwrap_or_else(|why| panic!("{label} seed={seed}: {why}"));
            assert!(w.totals.captured > 0, "captured light is real energy, not tissue");
        }
    }
}

#[test]
fn a_jawed_bodied_world_still_captures_though_its_consumers_eat_themselves() {
    // Capture is 890's claim. With grazing off (a saved world's meaning), a
    // jawed consumer eats its own lineage, so the counterplay role against the
    // producers can be missing; the capture role never is.
    let off = Bodies { grazing: false, ..Bodies::default() };
    for seed in 0..8 {
        let g = world_with(seed, Some(off.clone()), 60);
        let w = witness(&g, 60).unwrap();
        assert!(w.totals.captured > 0, "seed {seed}: producers capture light");
        assert!(w.totals.returned > 0 && w.totals.births > 0, "seed {seed}");
    }
}

#[test]
fn a_world_whose_producers_capture_nothing_fails_the_capture_role() {
    // The planted fault: the producer's light is never taken from the site.
    let mut g = world(1, false, 60);
    let produce = g.rules.processes.get_mut("ecology:produce-0").unwrap();
    produce.effects.retain(|e| match e {
        Effect::Transfer { account, .. } => account != "sim:energy",
        Effect::Transform { take, .. } => !take.contains_key("sim:energy"),
        _ => true,
    });
    g.validate().unwrap();
    let w = witness(&g, 60).unwrap();
    let missing = w.spec.missing(&w.totals);
    assert!(missing.iter().any(|m| m.starts_with("energy capture")), "{missing:?}");
    assert!(w.require_roles().is_err());
}

#[test]
fn the_light_never_binds_and_matter_is_conserved() {
    let g = world(2, false, 60);
    let mut s = cosmograph::Session::new(g, Execution::Individuals).unwrap();
    let before = s.sim.matter();
    for _ in 0..60 {
        let _ = s.advance_tick_with_flows().unwrap();
        assert_eq!(s.sim.matter(), before, "energy is not matter");
    }
    let held: u64 = s.sim.state().sites.values().map(|site| site.accounts.get("sim:energy").copied().unwrap_or(0)).sum();
    assert!(held > 0, "the sun shines");
}
