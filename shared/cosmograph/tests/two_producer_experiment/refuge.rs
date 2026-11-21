// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! One native attention nudge, forked from the same reopening meadow as its control.
use super::*;
use cosmograph::{
    directing::{Aim, Answer, Played, Toward, readings::unable},
    rules::Binding,
};

const REFUGE: Id = 1;
const OPEN: u64 = 2 * DAY;
const BEGIN_LEAN: &str = "experiment:begin-lean";

fn weather(s: &mut Session, ground: Id, open: bool) {
    s.command(Command::Act {
        actor: ground,
        process: if open { END_LEAN } else { BEGIN_LEAN }.into(),
        target: None,
        cause: None,
    })
    .unwrap();
    assert_eq!(
        s.sim.state().sites[&MEADOW].conditions[GROWTH],
        i64::from(open)
    );
}

#[derive(Serialize)]
struct Sample {
    tick: u64,
    alive: bool,
    grazer_site: Option<Id>,
    // Minutes resident and food transferred to the focal animal since the fork.
    minutes: [u64; 2],
    eaten: [u64; 2],
    patches: [Reading; 2],
}

#[derive(Serialize)]
struct Outcome {
    nudged: bool,
    first_refuge_tick: Option<u64>,
    answer: Option<Answer>,
    blocked_route_minutes: BTreeMap<String, u64>,
    samples: Vec<Sample>,
    final_hash: String,
}

fn follow(mut s: Session, ground: Id, player: Id, grazer: Id, nudged: bool) -> Outcome {
    let initial_matter = matter(&s);
    let span = s.sim.genesis().rules.directing.as_ref().unwrap().span;
    if nudged {
        s.command(Command::Nudge {
            participant: player,
            critter: grazer,
            aim: Aim::Attend,
            toward: Toward::Site(REFUGE),
            act: None,
        })
        .unwrap();
    }
    let mut blocked_route_minutes = BTreeMap::new();
    for process in ["ecology:wake-0", "ecology:wake-3"] {
        s.sim.watch(process);
    }
    let mut samples = Vec::new();
    let mut minutes = [0; 2];
    let mut eaten = [0; 2];
    let mut grazed = Counts::new();
    let mut recruited = Counts::new();
    let mut first_refuge_tick = None;
    for tick in OPEN..=2 * SEASON {
        if tick < OPEN + span
            && let Some(route) = unable(&s.sim, grazer)
                .into_iter()
                .find(|a| a.process == "move:0-1")
        {
            *blocked_route_minutes.entry(route.why).or_default() += 1;
        }
        if tick == SEASON {
            weather(&mut s, ground, false);
        } else if tick == SEASON + OPEN {
            weather(&mut s, ground, true);
        }
        let expected_open = !(SEASON..SEASON + OPEN).contains(&tick);
        assert_eq!(
            s.sim.state().sites[&MEADOW].conditions[GROWTH],
            i64::from(expected_open)
        );
        assert_eq!(s.sim.state().sites[&REFUGE].conditions[GROWTH], 1);
        let focal = s.sim.state().population.get(grazer).unwrap();
        let resident = focal.resident_site().filter(|_| focal.alive);
        if resident == Some(REFUGE) {
            first_refuge_tick.get_or_insert(tick);
        }
        if tick % DAY == 0 || tick == OPEN + span {
            assert_eq!(matter(&s), initial_matter);
            samples.push(Sample {
                tick,
                alive: focal.alive,
                grazer_site: resident,
                minutes,
                eaten,
                patches: [MEADOW, REFUGE]
                    .map(|site| read(&s, tick / DAY, site, &grazed, &recruited)),
            });
        }
        if tick == 2 * SEASON {
            break;
        }
        if let Some(site) = resident {
            minutes[site as usize] += 1;
        }
        let advanced = s.advance_tick_with_flows().unwrap();
        for flow in advanced.flows.iter().filter(|f| f.by_process(FEED)) {
            let Some(plant) = flow
                .from
                .0
                .body()
                .and_then(|id| s.sim.state().population.get(id))
                .filter(|e| e.kingdom == "kingdom:flora")
            else {
                continue;
            };
            let site = plant
                .resident_site()
                .expect("a grazed plant resides at the feeding site");
            let amount = flow.amount * flow.count;
            *grazed.entry((site, plant.lineage.clone())).or_default() += amount;
            if flow.to.0.body() == Some(grazer) {
                eaten[site as usize] += amount;
            }
        }
        for act in s.sim.take_watched() {
            let e = s.sim.state().population.get(act.actor).unwrap();
            if e.alive && !s.sim.genesis().rules.dormant(e) {
                *recruited.entry((e.place, e.lineage.clone())).or_default() += act.count;
            }
        }
    }
    assert_eq!(s.sim.state().nudges.len(), usize::from(nudged));
    let answer = s.sim.state().nudges.first().and_then(|n| n.answer.clone());
    let hash = s.sim.state_hash();
    let replay = Session::load(s.save(), Execution::Individuals).unwrap();
    assert_eq!(replay.sim.state_hash(), hash);
    Outcome {
        nudged,
        first_refuge_tick,
        answer,
        blocked_route_minutes,
        samples,
        final_hash: format!("{hash:016x}"),
    }
}

#[test]
#[ignore = "matched native meadow and refuge experiment"]
fn grazer_nudge() {
    let seed = 9;
    let (mut g, ground) = found_with_player(
        seed,
        [EXPOSURE, RESERVES],
        Some(Played {
            lineage: 1,
            region_sites: 2,
        }),
    );
    // Only the meadow's inert ground can change the lean season. Creatures
    // introduced by the played founding must never select a weather command.
    let keeper = "experiment:weather-keeper";
    g.rules.traits.insert(keeper.into());
    g.population
        .groups
        .get_mut(&ground)
        .unwrap()
        .entity_mut()
        .traits
        .insert(keeper.into());
    let opening = g.rules.processes.get_mut(END_LEAN).unwrap();
    opening.requires.push(Query::Trait {
        who: Binding::Actor,
        key: keeper.into(),
    });
    let mut closing = opening.clone();
    closing.id = BEGIN_LEAN.into();
    closing.effects = vec![Effect::Condition {
        key: GROWTH.into(),
        delta: -1,
    }];
    g.rules.processes.insert(closing.id.clone(), closing);
    g.validate().unwrap();
    let mut s = Session::new(g, Execution::Individuals).unwrap();
    s.advance(OPEN).unwrap();
    assert_eq!(s.sim.state().sites[&MEADOW].conditions[GROWTH], 0);
    weather(&mut s, ground, true);
    // Choose by stable identity among residents before observing either outcome.
    let (&grazer, _) = s
        .sim
        .state()
        .population
        .groups
        .iter()
        .find(|(_, group)| {
            let e = &group.entity;
            e.alive && e.lineage == GRAZER && e.resident_site() == Some(MEADOW)
        })
        .expect("a living grazer in the reopening meadow");
    let joined = s.command(Command::Join).unwrap();
    let player = joined
        .strip_prefix("participant:")
        .unwrap()
        .parse()
        .unwrap();
    s.command(Command::Take {
        participant: player,
        critter: grazer,
    })
    .unwrap();
    let fork = format!("{:016x}", s.sim.state_hash());
    let control = follow(s.clone(), ground, player, grazer, false);
    let nudged = follow(s, ground, player, grazer, true);
    assert_eq!(
        serde_json::to_value(&control.samples[0]).unwrap(),
        serde_json::to_value(&nudged.samples[0]).unwrap()
    );
    let result = serde_json::json!({ "seed": seed, "grazer": grazer, "fork_hash": fork, "control": control, "nudged": nudged });
    if let Some(path) = std::env::var_os("COSMOGRAPH_REFUGE_REPORT") {
        let file = std::fs::File::create_new(path).unwrap();
        serde_json::to_writer(file, &result).unwrap();
    } else {
        println!("{result}");
    }
}
