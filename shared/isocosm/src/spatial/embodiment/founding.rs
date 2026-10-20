// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{
    rules::{Binding, Causation, Effect, Expr, Query, Reading},
    simulation::Genesis,
    terrain::View,
};

pub const GENERATED_VERSION: u32 = 3;

/// Explicit new-play founding transformation, before the native Session exists.
/// Loading an existing world never regenerates its declared rules or placements.
pub fn declare(genesis: &mut Genesis) -> Result<()> {
    let Some(mut templates) = genesis.rules.spatial_templates.clone() else {
        return Err("placed play needs generator-declared physical templates".into());
    };
    if templates.version >= 2 {
        return Ok(());
    }
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
    templates.stands.clear();
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
        let mut grounding = Grounding::new(&volume);
        let mut blocked = Vec::new();
        let mut stands = Vec::new();
        for (&id, group) in &genesis.population.groups {
            let e = &group.entity;
            if !e.alive || e.resident_site() != Some(site) || crate::directing::is_participant(e) {
                continue;
            }
            let Some(document) = e.body.as_ref() else {
                continue;
            };
            let yaw = (0..crate::draw(genesis.seed, "body-stand-yaw", &[site, id]) % 4)
                .fold(Yaw::Zero, |yaw, _| yaw.compose(Yaw::Quarter));
            let Ok(pose) = grounding.find(
                document,
                [mid, 0, mid],
                yaw,
                SEARCH_REACH,
                SEARCH_STRIDE,
                SEARCH_CANDIDATES,
                &blocked,
            ) else {
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
            });
            if stands.len() == STANDS_PER_SITE {
                break;
            }
        }
        templates.stands.insert(site, stands);
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
        let Some(feed) = genesis.rules.processes.values().find(|p| p.port_feeding.is_some()
            && p.target.is_some() && p.period.is_some() && p.requires.iter().any(|q|
                matches!(q, Query::Trait { who: Binding::Actor, key } if key == cycle))).cloned() else {
            continue;
        };
        // New placed worlds read appetite from actual fillable body capacity.
        // Keep the world's existing need weight and consumer-cycle applicability.
        if let Some(mind) = &mut genesis.rules.mind {
            for need in &mut mind.needs {
                if need.traits.contains(cycle)
                    && let Query::Below { who: Binding::Actor, key, .. } = &need.query
                    && feed.commitments.iter().chain(&feed.effects).any(|e|
                        matches!(e, Effect::Eat { into, .. } if into == key))
                {
                    need.query = Query::Computed(Expr::AtLeast(
                        Box::new(Expr::Read(Reading::Room { who: Binding::Actor, key: key.clone() })),
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
        templates.foraging.insert(key.clone(), super::super::foraging::Foraging {
            feed: feed.id, reach: 0, search: 32, nodes: 256,
        });
        genesis.rules.processes.insert(key, process);
    }
    genesis.rules.spatial_templates = Some(templates);
    genesis.validate()
}
