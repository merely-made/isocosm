// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Full-site territory, with overlapping local windows for exact admission.
use super::*;
use crate::simulation::Genesis;

pub const VERSION: u32 = 5;
pub const MAX_STANDS: usize = 256;
const STRIDE: i64 = 128;
const MARGIN: i64 = 64;

pub fn whole_site(sim: &Simulation) -> bool {
    sim.genesis().rules.spatial_templates.as_ref().is_some_and(|t| t.version >= VERSION)
}

/// A canonical window in site coordinates, shared by native and product code.
/// Its overlap allows a step across a tile edge to be checked in both windows.
pub fn site_window(t: &Templates, at: [i64; 3]) -> Window {
    if t.version < VERSION { return t.window.clone(); }
    let width = STRIDE + 2 * MARGIN;
    let min = [0, 1].map(|k| {
        let lo = at[2 * k].div_euclid(STRIDE) * STRIDE - MARGIN;
        lo.clamp(t.window.min[k], (t.window.max[k] - width).max(t.window.min[k]))
    });
    Window { min, max: [0, 1].map(|k| (min[k] + width).min(t.window.max[k])), depth:t.window.depth }
}

/// New play only. Older declared templates are never regenerated on load.
pub fn declare_site(g: &mut Genesis) -> Result<()> {
    if g.rules.spatial_templates.as_ref().is_some_and(|t| t.version >= 2) { return Ok(()); }
    // The same declaration owns processes, diets, birth profiles and defaults.
    // Install full-site stands in that declaration rather than a second engine.
    founding::declare_in(g, true)
}

/// One placement policy for founding and actual arrivals, using native diets.
pub fn place_in_site(sim: &Simulation, t: &Templates, id: Id, site: Id,
    placed: &BTreeMap<Id, Pose>) -> Result<Pose> {
    let entity = sim.state().population.get(id).ok_or("habitat actor absent")?;
    let doc = entity.body.as_ref().ok_or("habitat body absent")?;
    let margin = i64::from(doc.aabb().extent().into_iter().max().unwrap_or(0)) + SEARCH_REACH;
    let centre = [0,1].map(|k| {
        let span = (t.window.max[k]-t.window.min[k]-2*margin).max(1);
        t.window.min[k] + margin.min((t.window.max[k]-t.window.min[k])/2)
            + (crate::draw(sim.genesis().seed,"habitat-anchor",&[site,id,k as u64]) % span as u64) as i64
    });
    let centre = [centre[0],0,centre[1]];
    let yaw = (0..crate::draw(sim.genesis().seed,"body-stand-yaw",&[site,id])%4)
        .fold(Yaw::Zero, |y,_| y.compose(Yaw::Quarter));
    let edible = sim.placement_food(id,site);
    let mut anchors = vec![centre];
    if !edible.is_empty() {
        anchors = placed.iter().filter(|(id,_)| edible.contains_key(id)).map(|(_,p)| p.at).collect();
        anchors.sort_by_key(|a| a[0].abs_diff(centre[0])+a[2].abs_diff(centre[2]));
        anchors.truncate(4);
        if anchors.is_empty() { anchors.push(centre); }
    }
    let blocked: Vec<_> = placed.iter().filter_map(|(id,p)|
        sim.state().population.get(*id)?.body.as_ref().and_then(|d| boxes(d,p).ok()))
        .flatten().collect();
    for anchor in anchors {
        let w = site_window(t,anchor);
        let volume = sim.site_volume(site,&w)?;
        let mut grounding = Grounding::new(&volume).with_articulation(true).with_spacing(t.body_spacing);
        let foods: Vec<_> = placed.iter().filter_map(|(id,p)| {
            let search = *edible.get(id)?;
            FoodStand::of(&volume,sim.state().population.get(*id)?.body.as_ref()?,p,search).ok()
        }).collect();
        let answer = if foods.is_empty() {
            grounding.find(doc,anchor,yaw,SEARCH_REACH,SEARCH_STRIDE,SEARCH_CANDIDATES,&blocked)
        } else { grounding.near_food(doc,centre,yaw,&blocked,&foods) };
        if let Ok(pose) = answer { return Ok(pose); }
    }
    Err("no supported unoccupied habitat stand within placement budget".into())
}

pub(super) fn stands(g: &Genesis, t: &Templates) -> Result<BTreeMap<Id, Vec<Stand>>> {
    let mut source = g.clone();
    let mut declaration = t.clone();
    declaration.crossings.clear();
    declaration.stands.clear();
    source.rules.spatial_templates = Some(declaration);
    let guide = Simulation::new(source, crate::Execution::Grouped)?;
    let mut out = BTreeMap::new();
    for &site in g.sites.keys() {
        let mut placed = BTreeMap::new();
        let mut ids: Vec<_> = g.population.groups.iter().flat_map(|(&first, group)|
            (first..first+group.count).filter_map(move |id| {
                let e = &group.entity;
                (e.alive && e.body.is_some() && e.resident_site() == Some(site)
                    && !crate::directing::is_participant(e)).then_some((id,e))
            })).collect();
        ids.sort_by_key(|(id,e)| (!e.traits.contains("life:producer"), *id));
        for (id, _) in ids.into_iter().take(MAX_STANDS) {
            let _span = crate::profile::Span::new("habitat.founder");
            if let Ok(pose) = place_in_site(&guide,t,id,site,&placed) { placed.insert(id,pose); }
        }
        out.insert(site,placed.into_iter().map(|(id,p)|
            Stand { at:p.at,yaw:p.yaw,actor:Some(id),feet:p.feet }).collect());
    }
    Ok(out)
}
