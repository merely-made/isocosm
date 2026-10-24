// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{
    rules::{Binding, Causation, Effect, Expr, Query, Reading},
    simulation::Genesis,
    terrain::View,
};

pub const GENERATED_VERSION: u32 = 4;

/// Distinct deterministic anchors across the declared physical window.
pub fn anchor(seed: u64, site: u64, id: u64, window: &Window) -> [i64; 3] {
    let p = [0, 1].map(|k| {
        let width = (window.max[k] - window.min[k]) / 2;
        window.min[k]
            + width / 2
            + (crate::draw(seed, "body-placement-anchor", &[site, id, k as u64])
                % width.max(1) as u64) as i64
    });
    [p[0], 0, p[1]]
}

/// Explicit new-play founding transformation, before the native Session exists.
/// Loading an existing world never regenerates its declared rules or placements.
pub fn declare(genesis: &mut Genesis) -> Result<()> {
    declare_in(genesis, false)
}

pub(super) fn declare_in(genesis: &mut Genesis, full_site: bool) -> Result<()> {
    let Some(mut templates) = genesis.rules.spatial_templates.clone() else {
        return Err("placed play needs generator-declared physical templates".into());
    };
    if templates.version >= 2 {
        return Ok(());
    }
    super::super::foraging::declare_senses(genesis);
    templates.walking = Some(Walking::default());
    let guide = Simulation::new(genesis.clone(), crate::Execution::Grouped)?;
    let view = View::of(genesis)?;
    let side =
        i64::try_from(view.footprint.side).map_err(|_| "site exceeds physical coordinates")?;
    let span = genesis
        .population
        .groups
        .values()
        .filter_map(|g| g.entity.body.as_ref())
        .map(|doc| doc.aabb().extent().into_iter().max().unwrap_or(0))
        .max()
        .unwrap_or(16);
    let half = (i64::from(span).max(32) + SEARCH_REACH)
        .min(128)
        .min(side / 2)
        .max(1);
    let mid = side / 2;
    templates.window = Window {
        min: [mid - half; 2],
        max: [mid + half; 2],
        depth: templates.window.depth,
    };
    templates.version = GENERATED_VERSION;
    templates.body_spacing = 2;
    templates.stands.clear();
    if full_site {
        templates.version = habitat::VERSION;
        templates.window.min = [0;2];
        templates.window.max = [side;2];
        templates.stands = habitat::stands(genesis, &templates)?;
    } else {
    for &site in genesis.sites.keys() {
        if !genesis.population.groups.values().any(|group| {
            group.entity.alive
                && group.entity.body.is_some()
                && group.entity.resident_site() == Some(site)
        }) {
            templates.stands.insert(site, Vec::new());
            continue;
        }
        let w = &templates.window;
        let volume = Volume::lift(&view, site, w.min, w.max, w.depth, &[])?;
        let places = Places::derive(&volume, templates.places)?;
        let mut grounding = Grounding::new(&volume)
            .with_walking(templates.walking)
            .with_articulation(true)
            .with_spacing(templates.body_spacing);
        let mut blocked = Vec::new();
        let mut stands: Vec<Stand> = Vec::new();
        let mut groups: Vec<_> = genesis.population.groups.iter().collect();
        // Creation order is not trophic order. Seat producers before the
        // consumers whose actual placed food they become.
        groups.sort_by_key(|(id, g)| (!g.entity.traits.contains("life:producer"), **id));
        for (&id, group) in groups {
            let e = &group.entity;
            if !e.alive || e.resident_site() != Some(site) || crate::directing::is_participant(e) {
                continue;
            }
            let Some(document) = e.body.as_ref() else {
                continue;
            };
            let yaw = (0..crate::draw(genesis.seed, "body-stand-yaw", &[site, id]) % 4)
                .fold(Yaw::Zero, |yaw, _| yaw.compose(Yaw::Quarter));
            let edible = guide.placement_food(id, site);
            let foods: Vec<_> = stands
                .iter()
                .filter_map(|stand| {
                    let food = stand.actor?;
                    let search = *edible.get(&food)?;
                    let doc = genesis.population.get(food)?.body.as_ref()?;
                    FoodStand::of(
                        &volume,
                        doc,
                        &Pose {
                            at: stand.at,
                            yaw: stand.yaw,
                            feet: stand.feet.clone(),
                        },
                        search,
                    )
                    .ok()
                })
                .collect();
            let centre = anchor(genesis.seed, site, id, &templates.window);
            let answer = if edible.is_empty() {
                grounding.find(
                    document,
                    centre,
                    yaw,
                    SEARCH_REACH,
                    SEARCH_STRIDE,
                    SEARCH_CANDIDATES,
                    &blocked,
                )
            } else {
                grounding.near_food(document, centre, yaw, &blocked, &foods)
            };
            let Ok(pose) = answer else {
                continue;
            };
            let eye = [pose.at[0], pose.at[1] + templates.eye[1], pose.at[2]];
            if places.place_at(pose.at).is_none()
                || places.place_at(pose.at) != places.place_at(eye)
            {
                continue;
            }
            blocked.extend(boxes(document, &pose)?);
            stands.push(Stand {
                at: pose.at,
                yaw: pose.yaw,
                actor: Some(id),
                feet: pose.feet,
            });
            if stands.len() == STANDS_PER_SITE {
                break;
            }
        }
        templates.stands.insert(site, stands);
    }
    }
    templates.crossings = genesis
        .sites
        .iter()
        .flat_map(|(&from, site)| {
            site.routes
                .iter()
                .filter_map(|route| {
                    templates.stands[&route.to].first().map(|stand| Crossing {
                        from,
                        to: route.to,
                        stand: stand.clone(),
                    })
                })
                .collect::<Vec<_>>()
        })
        .collect();
    let mut bounds = [[-1; 3], [1; 3]];
    for document in genesis
        .population
        .groups
        .values()
        .filter_map(|g| g.entity.body.as_ref())
    {
        for [lo, hi] in boxes(
            document,
            &Pose {
                at: [0; 3],
                yaw: Yaw::Zero,
                feet: Default::default(),
            },
        )? {
            for k in 0..3 {
                bounds[0][k] = bounds[0][k].min(lo[k] - 8);
                bounds[1][k] = bounds[1][k].max(hi[k] + 8);
            }
        }
    }
    fn biological(effect: &Effect) -> bool {
        matches!(effect, Effect::Birth { .. }) || effect.branches().any(biological)
    }
    for process in genesis.rules.processes.values() {
        if process.effects.iter().any(biological) {
            genesis.rules.birth_profiles.insert(
                format!("body:birth:{}", process.id),
                BirthProfile {
                    process: process.id.clone(),
                    places: templates.places,
                    clearance: bounds,
                    medium: BirthMedium::Air,
                    support: BirthSupport::BodyGround,
                },
            );
        }
    }
    let mut cycles = BTreeSet::new();
    for group in genesis.population.groups.values() {
        let e = &group.entity;
        if !e.traits.contains("life:consumer") {
            continue;
        }
        let Some(cycle) = e
            .traits
            .iter()
            .find(|key| key.starts_with("ability:cycle-"))
        else {
            continue;
        };
        if !cycles.insert(cycle.clone()) {
            continue;
        }
        let Some(feed) =
            genesis
                .rules
                .processes
                .values()
                .find(|p| {
                    p.port_feeding.is_some()
            && p.target.is_some() && p.period.is_some() && p.requires.iter().any(|q|
                matches!(q, Query::Trait { who: Binding::Actor, key } if key == cycle))
                })
                .cloned()
        else {
            continue;
        };
        // New placed worlds read appetite from actual fillable body capacity.
        // Keep the world's existing need weight and consumer-cycle applicability.
        if let Some(mind) = &mut genesis.rules.mind {
            for need in &mut mind.needs {
                if need.traits.contains(cycle)
                    && let Query::Below {
                        who: Binding::Actor,
                        key,
                        ..
                    } = &need.query
                    && feed
                        .commitments
                        .iter()
                        .chain(&feed.effects)
                        .any(|e| matches!(e, Effect::Eat { into, .. } if into == key))
                {
                    need.query = Query::Computed(Expr::AtLeast(
                        Box::new(Expr::Read(Reading::Room {
                            who: Binding::Actor,
                            key: key.clone(),
                        })),
                        Box::new(Expr::Const(1)),
                    ));
                }
            }
        }
        let key = format!("body:walk:{cycle}");
        let mut process = crate::generate::process(
            &key,
            Causation::Choice,
            vec![Effect::Step {
                profile: key.clone(),
            }],
        );
        process.requires.extend([
            Query::Trait {
                who: Binding::Actor,
                key: cycle.clone(),
            },
            Query::Trait {
                who: Binding::Actor,
                key: "life:consumer".into(),
            },
        ]);
        process.period = Some(1);
        process.target = feed.target.clone();
        process.note = true;
        genesis.rules.step_profiles.insert(
            key.clone(),
            StepProfile {
                process: key.clone(),
                cells: 1.into(),
                places: templates.places,
                body: isometer_space::places::Body {
                    width: 1,
                    height: 1,
                    climb: 1,
                    wades: false,
                },
                stance_offset: [0; 3],
            },
        );
        templates.foraging.insert(
            key.clone(),
            super::super::foraging::Foraging {
                feed: feed.id,
                reach: 0,
                search: super::super::foraging::FOOD_SEARCH,
                nodes: 256,
                sight: true,
                reachable: true,
            },
        );
        genesis.rules.processes.insert(key, process);
    }
    genesis.rules.spatial_templates = Some(templates);
    genesis.validate()
}
