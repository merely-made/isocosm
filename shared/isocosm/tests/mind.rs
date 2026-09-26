use isocosm::{
    Execution, Founding, Simulation,
    rules::*,
    simulation::{Genesis, Outcome},
};
use std::collections::{BTreeMap, BTreeSet};

const STRAIN: &str = "mind:strain";

/// A reservoir world whose members are strained while they hold less than
/// `low` soil, and relieved otherwise.
fn world(low: u64) -> Genesis {
    let mut g = Founding {
        seed: 5,
        sites: 2,
        population: 12,
        cohort_size: 4,
        lineages: 1,
        ..Default::default()
    }
    .generate()
    .unwrap();
    let rules = &mut g.rules;
    rules.accounts.insert(STRAIN.into(), AccountKind::Strain);
    rules.mind = Some(Mind {
        strain: STRAIN.into(),
        needs: vec![Need {
            traits: BTreeSet::new(),
            query: Query::Below {
                who: Binding::Actor,
                key: "world:soil".into(),
                amount: low,
            },
            weight: -2,
        }],
        bearing: 5,
        bearing_traits: BTreeMap::new(),
        rise: 500,
        rise_traits: BTreeMap::new(),
        stake: 0,
    });
    let mut build = rules.processes["sim:remember"].clone();
    build.id = "mind:strain".into();
    build.shape = Shape::Transition;
    build.note = false;
    build.requires = vec![
        Query::Alive(Binding::Actor),
        Query::MoodBelow { amount: -1 },
    ];
    build.effects = vec![Effect::Transform {
        who: Binding::Actor,
        take: BTreeMap::new(),
        give: BTreeMap::from([(STRAIN.into(), 3)]),
    }];
    build.period = Some(1);
    let mut bleed = build.clone();
    bleed.id = "mind:relief".into();
    bleed.requires = vec![Query::Alive(Binding::Actor), Query::Mood { at_least: -1 }];
    bleed.effects = vec![Effect::Ease {
        who: Binding::Actor,
        key: STRAIN.into(),
        amount: 2,
    }];
    for p in [build, bleed] {
        rules.processes.insert(p.id.clone(), p);
    }
    g
}

fn strain(sim: &Simulation, id: u64) -> u64 {
    let e = sim.state().population.get(id).unwrap();
    e.accounts.get(STRAIN).copied().unwrap_or(0)
}

#[test]
fn strain_builds_while_mood_is_low_and_bleeds_off_with_relief() {
    // Every member starts with 4 soil: below 5 they are low, below 4 not.
    for mode in [Execution::Individuals, Execution::Grouped] {
        let mut low = Simulation::new(world(5), mode).unwrap();
        low.advance(3).unwrap();
        assert_eq!(strain(&low, 2), 9, "{mode:?}");
        let mut content = Simulation::new(world(4), mode).unwrap();
        content.advance(3).unwrap();
        assert_eq!(strain(&content, 2), 0, "{mode:?}");
    }
    // Relief bleeds strain off, never below nothing.
    let mut sim = Simulation::new(world(5), Execution::Individuals).unwrap();
    sim.advance(2).unwrap();
    assert_eq!(strain(&sim, 2), 6);
    for _ in 0..2 {
        let r = sim.execute(2, Some(3), "sim:give", None);
        assert_eq!(r.outcome, Outcome::Accepted);
    }
    // Member 3 now holds 6 soil and is content; member 2 holds 2.
    sim.advance(3).unwrap();
    assert_eq!(strain(&sim, 3), 0);
    assert_eq!(strain(&sim, 2), 15);
    let r = sim.execute(2, None, "mind:strain", None);
    assert!(
        r.facts_read.iter().any(|f| f.starts_with("MoodBelow")),
        "{r:?}"
    );
}

#[test]
fn a_mind_is_admitted_only_as_the_rules_declare_it() {
    let refused = |edit: &dyn Fn(&mut Genesis)| {
        let mut g = world(5);
        edit(&mut g);
        g.validate().unwrap_err()
    };
    assert!(
        refused(&|g| {
            g.rules.accounts.insert(STRAIN.into(), AccountKind::Energy);
        })
        .contains("not a strain account")
    );
    assert!(
        refused(&|g| {
            let need = &mut g.rules.mind.as_mut().unwrap().needs[0];
            need.query = Query::Alive(Binding::Target);
        })
        .contains("a need reads only")
    );
    assert!(
        refused(&|g| {
            let need = &mut g.rules.mind.as_mut().unwrap().needs[0];
            need.query = Query::Mood { at_least: 0 };
        })
        .contains("a need reads only")
    );
    assert!(
        refused(&|g| {
            g.rules.mind = None;
        })
        .contains("mood is read only in a world with a mind")
    );
    assert!(
        refused(&|g| {
            let p = g.rules.processes.get_mut("mind:relief").unwrap();
            p.effects = vec![Effect::Ease {
                who: Binding::Actor,
                key: "world:soil".into(),
                amount: 1,
            }];
        })
        .contains("eases what cannot be eased")
    );
}

#[test]
fn worlds_without_a_mind_serialize_and_hash_as_before() {
    let g = Founding::default().generate().unwrap();
    let json = serde_json::to_string(&g.rules).unwrap();
    assert!(!json.contains("\"mind\""));
    let with = world(5);
    let json = serde_json::to_string(&with.rules).unwrap();
    let back: Rules = serde_json::from_str(&json).unwrap();
    assert_eq!(back, with.rules);
}
