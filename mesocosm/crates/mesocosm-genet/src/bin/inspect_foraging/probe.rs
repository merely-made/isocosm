// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Inspect refusals without changing the recorded world or its budgets.
use super::*;
use isocosm::spatial::{embodiment as body, foraging};

pub(super) fn census(session: &Session) -> Result<(), String> {
    for entry in &session.entries {
        let Command::AdvanceBirthSpatial {
            ticks,
            terrain,
            steps,
            arrivals,
            births,
        } = &entry.command
        else {
            continue;
        };
        if steps.is_empty() {
            continue;
        }
        let before = session.before_entry(&entry.id)?;
        let Preparation::Step(pending) =
            before.prepare_birth_spatial(*ticks, terrain, &[], arrivals, births)?
        else {
            return Err("first food choice cut absent".into());
        };
        let sim = &pending.sim;
        let mut actors = Vec::new();
        for (&actor, at) in &sim.state().spatial_poses {
            let e = sim.state().population.get(actor).unwrap();
            if at.site != pending.request.site || !e.alive || !e.traits.contains("life:consumer") {
                continue;
            }
            for walk in sim
                .genesis()
                .rules
                .spatial_templates
                .as_ref()
                .unwrap()
                .foraging
                .keys()
            {
                let foods = sim.placement_food(actor, at.site);
                let detected: Vec<_> = foods
                    .keys()
                    .filter_map(|id| sim.food_detection(actor, *id, walk).map(|d| (*id, d)))
                    .collect();
                let started = Instant::now();
                let report = sim.food_route(actor, walk);
                let read_us = started.elapsed().as_micros();
                let repeated = Instant::now();
                if sim.food_route(actor, walk) != report {
                    return Err("identical route reads disagree".into());
                }
                actors.push(
                    serde_json::json!({"actor":actor,"walk":walk,"detected":detected,
                    "report":report,"read_us":read_us,"repeat_us":repeated.elapsed().as_micros()}),
                );
            }
        }
        println!(
            "{}",
            serde_json::json!({"tick":pending.request.tick,"site":pending.request.site,
            "cut":"first pending step, before its publication","actors":actors})
        );
        return Ok(());
    }
    Err("run has no food choice cut to inspect".into())
}

pub(super) fn verify(
    sim: &isocosm::Simulation,
    r: &isocosm::spatial::Request,
    route: &foraging::FoodRoute,
) -> Result<(), String> {
    let document = sim
        .state()
        .population
        .get(r.actor)
        .unwrap()
        .body
        .as_ref()
        .unwrap();
    let templates = sim.genesis().rules.spatial_templates.as_ref().unwrap();
    let window = body::site_window(templates, route.path[0].at);
    let profile = &sim.genesis().rules.step_profiles[&r.profile];
    let geometry = sim.site_geometry(r.site, &window, profile.places)?;
    let blocked: Vec<_> = sim
        .state()
        .spatial_poses
        .iter()
        .filter(|(id, at)| **id != r.actor && at.site == r.site)
        .flat_map(|(id, at)| {
            body::boxes(
                sim.state()
                    .population
                    .get(*id)
                    .unwrap()
                    .body
                    .as_ref()
                    .unwrap(),
                &at.pose,
            )
            .unwrap()
        })
        .collect();
    let walking = body::walking_rules(sim);
    for (index, pair) in route.path.windows(2).enumerate() {
        if !body::neighbours_with_walking(
            &geometry.volume,
            &geometry.places,
            profile,
            document,
            &pair[0],
            walking,
        )
        .contains(&pair[1])
        {
            return Err("complete route crosses an unavailable native neighbour".into());
        }
        body::check_walk(
            &geometry.volume,
            document,
            &pair[0],
            &pair[1],
            walking,
            route.strides.get(index).and_then(Option::as_ref),
            &blocked,
        )?;
    }
    let food = &sim.state().spatial_poses[&route.target];
    let boxes = body::boxes(
        sim.state()
            .population
            .get(route.target)
            .unwrap()
            .body
            .as_ref()
            .unwrap(),
        &food.pose,
    )?;
    if foraging::gap(&body::boxes(document, route.path.last().unwrap())?, &boxes)
        > sim.foraging_policy(&r.profile).unwrap().reach
    {
        return Err("complete route does not reach food contact".into());
    }
    Ok(())
}

pub(super) fn run(session: &Session) -> Result<(), String> {
    let actors = std::env::args()
        .find_map(|arg| {
            arg.strip_prefix("--actors=").map(|list| {
                list.split(',')
                    .map(|id| {
                        id.parse::<u64>()
                            .map_err(|_| "invalid probe actor".to_string())
                    })
                    .collect::<Result<Vec<_>, _>>()
            })
        })
        .transpose()?;
    let mut probes = Vec::new();
    for entry in &session.entries {
        let Command::AdvanceBirthSpatial {
            ticks,
            terrain,
            steps,
            arrivals,
            births,
        } = &entry.command
        else {
            continue;
        };
        let before = session.before_entry(&entry.id)?;
        for (index, answer) in steps.iter().enumerate() {
            if before
                .sim
                .foraging_policy(&answer.request.profile)
                .is_none()
                || actors
                    .as_ref()
                    .is_some_and(|ids| !ids.contains(&answer.request.actor))
            {
                continue;
            }
            if actors.is_none() && probes.len() >= 4 {
                break;
            }
            let Preparation::Step(pending) =
                before.prepare_birth_spatial(*ticks, terrain, &steps[..index], arrivals, births)?
            else {
                return Err("probe request absent".into());
            };
            if pending.request != answer.request {
                return Err("probe request changed".into());
            }
            let sim = &pending.sim;
            let r = &pending.request;
            let document = sim
                .state()
                .population
                .get(r.actor)
                .unwrap()
                .body
                .as_ref()
                .unwrap();
            let pose = &sim.state().spatial_poses[&r.actor].pose;
            let templates = sim.genesis().rules.spatial_templates.as_ref().unwrap();
            let window = body::site_window(templates, pose.at);
            let profile = &sim.genesis().rules.step_profiles[&r.profile];
            let geometry = sim.site_geometry(r.site, &window, profile.places)?;
            let blocked: Vec<_> = sim
                .state()
                .spatial_poses
                .iter()
                .filter(|(id, at)| **id != r.actor && at.site == r.site)
                .flat_map(|(id, at)| {
                    body::boxes(
                        sim.state()
                            .population
                            .get(*id)
                            .unwrap()
                            .body
                            .as_ref()
                            .unwrap(),
                        &at.pose,
                    )
                    .unwrap()
                })
                .collect();
            let walking = body::walking_rules(sim).or_else(|| {
                std::env::args()
                    .any(|arg| arg == "--try-walking")
                    .then(body::Walking::default)
            });
            let neighbours: Vec<_> = body::neighbours_with_walking(
                &geometry.volume,
                &geometry.places,
                profile,
                document,
                pose,
                walking,
            )
            .into_iter()
            .map(|next| {
                let occupied = body::boxes(document, &next).unwrap();
                let wider: Vec<_> = walking.filter(|_| std::env::args().any(|a| a == "--try-walking"))
                    .into_iter().flat_map(|rules| (1..=4).map(move |shift| body::Walking {shift, ..rules}))
                    .map(|rules| serde_json::json!({"shift":rules.shift,
                        "plan":body::step_stride(&geometry.volume,document,pose,&next,Some(rules),&blocked)}))
                    .collect();
                serde_json::json!({"pose":next,"intersects":body::intersects(&occupied,&blocked),
                    "stride":body::step_stride(&geometry.volume,document,pose,&next,walking,&blocked),
                    "shift_trials":wider,
                    "legacy_swing":body::sweep_clear(&geometry.volume,document,pose,&next,&blocked)})
            })
            .collect();
            let started = Instant::now();
            let result = foraging::geometry(sim, r, &window);
            probes.push(
                serde_json::json!({"request":r,"before":pose,"probe_walking":walking,
                "food":sim.state().spatial_poses.get(&r.target.unwrap()),
                "neighbours":neighbours,"route":result,"route_us":started.elapsed().as_micros()}),
            );
        }
        break;
    }
    println!("{}", serde_json::json!({"probes":probes}));
    Ok(())
}
