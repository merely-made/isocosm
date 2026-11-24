// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Found an observed patch from the planet's actual resident bodies and stands.
use super::{Result, boxes};
use cosmograph::{
    Simulation, anatomy, development,
    ecology::soil::{self, Bed, Contact},
    geometry::Body,
    rules::{AccountKind, Binding, Effect, Process, Query, Target},
    schema::Id,
    simulation::Genesis,
    spatial::Pose,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;

const P: &str = "soil:phosphate";
const BOUND: &str = "soil:litter-phosphorus";
const USED: &str = "soil:incorporated-phosphorus";
const BACKGROUND: &str = "soil:background";
fn has(who: Binding, key: &str) -> Query {
    Query::Account {
        who,
        key: key.into(),
        at_least: 1,
    }
}
fn transfer(from: Binding, to: Binding, key: &str) -> Effect {
    Effect::Transfer {
        from,
        to,
        account: key.into(),
        amount: 1.into(),
    }
}
fn convert(who: Binding, from: &str, to: &str) -> Effect {
    Effect::Transform {
        who,
        take: [(from.into(), 1)].into(),
        give: [(to.into(), 1)].into(),
        conversion: None,
    }
}

#[derive(Clone, Copy)]
pub(super) enum Survey {
    Litter(bool),
    Nurse(bool),
}

impl Survey {
    pub fn label(self) -> &'static str {
        match self {
            Self::Litter(true) => "near",
            Self::Litter(false) => "far",
            Self::Nurse(true) => "nurse-near",
            Self::Nurse(false) => "nurse-far",
        }
    }
}

pub(super) fn found(g: &mut Genesis, site: Id, survey: Survey) -> Result<()> {
    use Binding::{Actor, Place, Target as Partner};
    let stands = &g
        .rules
        .spatial_templates
        .as_ref()
        .ok_or("missing surface stands")?
        .stands[&site];
    let plants: Vec<_> = stands
        .iter()
        .filter_map(|s| {
            let id = s.actor?;
            let e = g.population.get(id)?;
            (e.kingdom == "kingdom:flora").then_some((
                id,
                Pose {
                    at: s.at,
                    yaw: s.yaw,
                    feet: s.feet.clone(),
                },
            ))
        })
        .collect();
    let fungi: Vec<_> = g
        .population
        .groups
        .iter()
        .filter(|(_, r)| {
            r.entity.kingdom == "kingdom:myco" && r.entity.resident_site() == Some(site)
        })
        .flat_map(|(&first, r)| first..first + r.count)
        .collect();
    if plants.len() < fungi.len() || fungi.is_empty() {
        return Err("soil survey needs a living root for each fungal founder".into());
    }
    let lineage = g.population.get(fungi[0]).unwrap().lineage.clone();
    let plant_line = g.population.get(plants[0].0).unwrap().lineage.clone();
    if fungi
        .iter()
        .any(|id| g.population.get(*id).unwrap().lineage != lineage)
        || plants
            .iter()
            .take(fungi.len())
            .any(|(id, _)| g.population.get(*id).unwrap().lineage != plant_line)
    {
        return Err("this patch needs a single fungal lineage and a single root lineage".into());
    }
    let pindex = plant_line
        .strip_prefix("lineage:")
        .ok_or("unnumbered producer")?;
    let findex = lineage
        .strip_prefix("lineage:")
        .ok_or("unnumbered fungus")?;
    let carbon = format!("matter:{pindex}-0");
    let tissue = format!("matter:{findex}-0");
    // Resolve the authored grammar with the very same body-map compiler as ordinary founding.
    let f = g.founding.as_mut().ok_or("missing founding map")?;
    f.body_maps.insert(
        lineage.clone(),
        include_str!("../../../../../tools/planet/soil-body.toml").parse()?,
    );
    let mapped = f.generate()?;
    let d = mapped.lineages[&lineage]
        .development
        .clone()
        .ok_or("no fungal development")?;
    for key in d.recipe.kinds() {
        g.rules
            .kinds
            .insert(key.clone(), mapped.rules.kinds[&key].clone());
    }
    g.lineages.get_mut(&lineage).unwrap().development = Some(d.clone());
    let mut anchors = BTreeMap::new();
    for (&id, (plant, stand)) in fungi.iter().zip(&plants) {
        let collar =
            soil::collar(g.population.get(*plant).unwrap(), stand).ok_or("plant collar missing")?;
        let fungus = g.population.lift(id)?;
        let adult = development::lineage_soma(
            &g.rules,
            &d,
            cosmograph::draw(g.seed, "founder-soma", &[id]),
        );
        let full = development::develop(&g.rules, &d, &adult)?;
        let root = full.doc.root;
        let mut doc = full.doc.clone();
        doc.parts.truncate(1);
        let body = Body {
            doc,
            parts: [(root, full.parts[&root].clone())].into(),
        };
        fungus.soma = adult.segments;
        fungus.embody(body);
        let n = fungus
            .accounts
            .remove(&tissue)
            .ok_or("fungal stock missing")?;
        fungus
            .part_mut(&root)
            .unwrap()
            .matter
            .insert(tissue.clone(), n);
        let key = format!("soil:founder-{id}");
        g.rules.traits.insert(key.clone());
        fungus.traits.insert(key);
        let half = fungus.extent(root);
        let at = [stand.at[0], collar[0][1] - i64::from(half[1]), stand.at[2]];
        anchors.insert(
            id,
            Pose {
                at,
                yaw: isometer_core::Yaw::Zero,
                feet: Default::default(),
            },
        );
    }
    let origin = anchors[&fungi[0]].at;
    // Only the lateral displacement changes between the two treatments.
    let x = origin[0] - 20;
    let z = origin[2]
        + if matches!(survey, Survey::Litter(false)) {
            40
        } else {
            0
        };
    let litter = [[x, origin[1] - 4, z - 2], [x + 4, origin[1] - 2, z + 2]];
    g.rules.soil_beds.insert(
        site,
        Bed {
            litter,
            colonies: anchors,
            seed_rain: None,
        },
    );
    g.rules.conditions.insert(BACKGROUND.into());
    for (&id, s) in &mut g.sites {
        if id != site {
            s.conditions.insert(BACKGROUND.into(), 1);
        }
    }
    for key in [P, BOUND, USED] {
        g.rules.accounts.insert(
            key.into(),
            AccountKind::Matter {
                lineage: "world:ground".into(),
                reserve: false,
                provision: false,
            },
        );
    }
    g.sites
        .get_mut(&site)
        .unwrap()
        .accounts
        .insert(BOUND.into(), 12);
    let base = g
        .rules
        .processes
        .get_mut(&format!("ecology:produce-{pindex}"))
        .ok_or("plant production missing")?;
    let mut growth = base.clone();
    base.requires.push(Query::Condition {
        key: BACKGROUND.into(),
        at_least: 1,
    });
    growth.id = "ecology:root-production".into();
    growth.requires.extend([
        Query::Condition {
            key: format!("site:{site}"),
            at_least: 1,
        },
        has(Actor, P),
    ]);
    growth.effects.push(convert(Actor, P, USED));
    let make = |id: &str,
                actor_line: &str,
                priority: i32,
                mut requires: Vec<Query>,
                effects: Vec<Effect>,
                target: Option<Target>| {
        requires.insert(
            0,
            Query::Trait {
                who: Actor,
                key: format!("ability:cycle-{actor_line}"),
            },
        );
        requires.insert(0, Query::Alive(Actor));
        Process {
            id: id.into(),
            causation: cosmograph::rules::Causation::Transition,
            priority,
            period: Some(1),
            requires,
            effects,
            target,
            turn: None,
            commitments: vec![],
            risk: None,
            need_account: None,
            need_below: 0,
            glyphs: Default::default(),
            invariants: ["sim:matter".into(), "sim:nonnegative".into()].into(),
            note: false,
            port_feeding: None,
            goal_directives: vec![],
        }
    };
    let extend = make(
        "ecology:extend-hypha",
        findex,
        -5,
        vec![Query::SoilContact(Contact::InSoil)],
        vec![Effect::Extend {
            tissue: tissue.clone(),
        }],
        None,
    );
    let recover = make(
        "ecology:recover-phosphate",
        findex,
        -4,
        vec![Query::SoilContact(Contact::Litter), has(Place, BOUND)],
        vec![transfer(Place, Actor, BOUND), convert(Actor, BOUND, P)],
        None,
    );
    let trade = make(
        "ecology:root-exchange",
        pindex,
        -3,
        vec![
            Query::SoilContact(Contact::Root),
            has(Partner, P),
            has(Actor, &carbon),
        ],
        vec![
            transfer(Partner, Actor, P),
            transfer(Actor, Partner, &carbon),
        ],
        Some(Target {
            same_place: true,
            alive: Some(true),
            lineage: Some(lineage.clone()),
            among: Default::default(),
            weighted: false,
        }),
    );
    let assimilate = make(
        "ecology:use-root-carbon",
        findex,
        -2,
        vec![Query::SoilContact(Contact::InSoil), has(Actor, &carbon)],
        vec![convert(Actor, &carbon, &tissue)],
        None,
    );
    for p in [extend, recover, trade, assimilate, growth] {
        g.rules.processes.insert(p.id.clone(), p);
    }
    if let Survey::Nurse(near) = survey {
        let donor = *fungi
            .get(1)
            .ok_or("seed nursery needs a second resident fungus")?;
        seed_bed(g, site, near, &plants[0], donor, &plant_line, pindex)?;
    }
    Ok(())
}

/// A sheltered substrate-scale experiment with one nutrient-bearing nurse colony.
/// The donor carries an equal share of the finite litter phosphorus in both arms.
fn seed_bed(
    g: &mut Genesis,
    site: Id,
    near: bool,
    source: &(Id, Pose),
    donor: Id,
    lineage: &str,
    cycle: &str,
) -> Result<()> {
    let rain = soil::SeedRain {
        lineage: lineage.into(),
        drift: [32, 0],
        jitter: 1,
    };
    let guide = Simulation::new(g.clone(), cosmograph::Execution::Grouped)?;
    let t = guide
        .genesis()
        .rules
        .spatial_templates
        .as_ref()
        .ok_or("seed nursery needs terrain")?;
    let (plant, root) = source;
    let mut downwind = root.clone();
    downwind.at[0] += i64::from(rain.drift[0]);
    let stand = cosmograph::spatial::embodiment::settle(
        &guide,
        t,
        *plant,
        site,
        &downwind,
        &BTreeMap::new(),
    )?;
    let collar = soil::collar(g.population.get(*plant).unwrap(), &stand)
        .ok_or("missing recipient collar")?;
    let fungus = g.population.get(donor).ok_or("missing recipient colony")?;
    let half = fungus.extent(fungus.body.as_ref().ok_or("missing colony map")?.root);
    let bed = g.rules.soil_beds.get_mut(&site).unwrap();
    let nurse = bed.colonies.get_mut(&donor).unwrap();
    nurse.at = [
        stand.at[0],
        collar[0][1] - i64::from(half[1]),
        stand.at[2] + if near { 0 } else { 40 },
    ];
    bed.seed_rain = Some(rain);
    *g.sites
        .get_mut(&site)
        .unwrap()
        .accounts
        .get_mut(BOUND)
        .ok_or("litter phosphorus missing")? -= 4;
    g.population.lift(donor)?.accounts.insert(P.into(), 4);
    let location = format!("site:{site}");
    for p in g.rules.processes.values_mut() {
        if p.effects
            .iter()
            .any(|e| matches!(e, Effect::Move { .. } | Effect::MovePaced { .. }))
            && p.requires
                .iter()
                .any(|q| matches!(q, Query::Condition { key, .. } if key == &location))
        {
            p.requires.push(Query::SoilContact(Contact::Unanchored));
        }
    }
    g.rules
        .processes
        .get_mut(&format!("ecology:wake-{cycle}"))
        .ok_or("seed nursery needs seasonal germination")?
        .requires
        .push(Query::SoilContact(Contact::SeedLanded));
    Ok(())
}

pub(super) fn observe(
    sim: &Simulation,
    site: Id,
    bed: &Bed,
    poses: &mut Vec<Value>,
) -> Result<Value> {
    let state = sim.state();
    let mut roots = vec![];
    let mut collars = Vec::new();
    for (&id, r) in &state.spatial_poses {
        let Some(e) = state.population.get(id) else {
            continue;
        };
        if e.kingdom != "kingdom:flora"
            || !e.alive
            || e.resident_site() != Some(site)
            || r.site != site
            || r.arrived != e.arrived
        {
            continue;
        }
        if let Some(b) = soil::collar(e, &r.pose) {
            collars.push((id, b));
            roots.push(json!({"id":id,"bounds":b,"p":anatomy::held(e,&sim.genesis().rules,P),"used":anatomy::held(e,&sim.genesis().rules,USED)}));
        }
    }
    let mut colonies = vec![];
    for &id in bed.colonies.keys() {
        let Some(e) = state.population.get(id) else {
            continue;
        };
        let Some(pose) = soil::pose(bed, site, id, e) else {
            continue;
        };
        let parts = boxes(e, pose)?;
        let host = collars
            .iter()
            .find(|(_, b)| soil::reaches(e, pose, *b))
            .map(|(id, _)| id);
        colonies.push(
            json!({"id":id,"parts":parts.len(),"contact":soil::reaches(e, pose, bed.litter),
            "p":anatomy::held(e,&sim.genesis().rules,P),"host":host}),
        );
        poses.push(
            json!({"id":id,"site":site,"pose":pose,"kingdom":e.kingdom,"boxes":parts,"soil":true}),
        );
    }
    Ok(
        json!({"litter":bed.litter,"remaining":state.sites[&site].accounts.get(BOUND).copied().unwrap_or(0),"colonies":colonies,"roots":roots}),
    )
}
