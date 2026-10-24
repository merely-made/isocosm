// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! One physical search shared by native food choice and the step provider.
use super::*;
use isometer_core::BodyDocument;
use std::sync::Arc;
mod cache;
mod search;
pub(crate) use cache::Cache;

/// A complete contact route. Only its first adjacent step is published;
/// subsequent rounds re-read needs, detection, terrain and current residents.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct FoodRoute {
    pub target: Id,
    pub path: Vec<Pose>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub strides: Vec<Option<isometer_core::legs::Stride>>,
}

/// Bounded derivative evidence, never saved authority or a claim that the
/// entire site is unreachable when the declared node budget is exhausted.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct RouteSearch {
    pub route: Option<FoodRoute>,
    pub nodes: usize,
    pub checked_edges: usize,
    pub refused_edges: usize,
    pub first_refusal: Option<String>,
    pub exhausted: bool,
}

#[derive(Clone, Debug)]
struct Input {
    actor: Id,
    site: Id,
    terrain: Arc<super::super::SiteGeometry>,
    document: BodyDocument,
    before: Pose,
    profile: StepProfile,
    foods: Vec<(Id, Vec<embodiment::Box3>)>,
    blocked: Vec<embodiment::Box3>,
    reach: u64,
    nodes: usize,
    walking: Option<embodiment::Walking>,
}

impl Simulation {
    /// Read native appetite, meal eligibility and detection, then find a route
    /// to any of those meals using the same physical machinery as admission.
    pub fn food_route(&self, actor: Id, walk: &str) -> Result<RouteSearch> {
        let f = self
            .foraging_policy(walk)
            .ok_or("foraging profile absent")?;
        let feed = &self.genesis.rules.processes[&f.feed];
        if self.food_need_score(actor, feed) <= 0 {
            return Ok(RouteSearch::default());
        }
        let foods = self.food_candidates(actor, feed, f);
        if foods.iter().any(|(_, d)| *d <= f.reach) {
            return Ok(RouteSearch::default());
        }
        let at = self
            .spatial_pose_at_start(actor)
            .ok_or("foraging pose absent")?;
        let t = self
            .genesis
            .rules
            .spatial_templates
            .as_ref()
            .ok_or("foraging templates absent")?;
        self.route_to_foods(
            actor,
            walk,
            &embodiment::site_window(t, at.pose.at),
            foods
                .into_iter()
                .filter(|(_, d)| *d <= f.search)
                .map(|(id, _)| id),
        )
    }

    fn route_to_foods(
        &self,
        actor: Id,
        walk: &str,
        window: &Window,
        foods: impl IntoIterator<Item = Id>,
    ) -> Result<RouteSearch> {
        let _span = crate::profile::Span::new("foraging.route.read");
        let f = self
            .foraging_policy(walk)
            .ok_or("foraging profile absent")?;
        let at = self
            .spatial_pose_at_start(actor)
            .ok_or("foraging pose absent")?;
        let foods: Vec<_> = foods
            .into_iter()
            .filter_map(|id| {
                self.current_placed_boxes(id, at.site)
                    .map(|boxes| (id, boxes))
            })
            .collect();
        if foods.is_empty() {
            return Ok(RouteSearch::default());
        }
        let profile = self
            .genesis
            .rules
            .step_profiles
            .get(walk)
            .ok_or("step profile absent")?;
        let terrain = self.site_geometry(at.site, window, profile.places)?;
        let document = self
            .body_at_start(actor)
            .and_then(|e| e.body.as_ref())
            .ok_or("walking body absent")?;
        embodiment::check_for(self, &terrain.volume, document, &at.pose)?;
        let input = Input {
            actor,
            site: at.site,
            terrain,
            document: document.clone(),
            before: at.pose.clone(),
            profile: profile.clone(),
            walking: embodiment::walking_rules(self),
            foods,
            reach: f.reach,
            nodes: f.nodes,
            blocked: self
                .state
                .spatial_poses
                .keys()
                .copied()
                .filter(|id| *id != actor)
                .filter_map(|id| self.current_placed_boxes(id, at.site))
                .flatten()
                .collect(),
        };
        self.foraging_routes.find(input)
    }
}

/// The native choice and provider consume the same bounded full-route witness.
pub fn geometry(sim: &Simulation, request: &Request, window: &Window) -> Result<Geometry> {
    let f = sim
        .foraging_policy(&request.profile)
        .ok_or("foraging profile absent")?;
    let p = &sim.genesis.rules.processes[&request.process];
    if !sim.foraging_walk_matches(request.actor, request.target, p) {
        return Err("chosen food no longer serves a current need within search range".into());
    }
    let target = request.target.ok_or("foraging target absent")?;
    let found = if f.reachable {
        sim.food_route(request.actor, &request.profile)?
    } else {
        sim.route_to_foods(request.actor, &request.profile, window, [target])?
    };
    let route = found
        .route
        .ok_or("no supported route to chosen food within the saved node budget")?;
    if route.target != target {
        return Err("chosen food route changed with current occupancy".into());
    }
    let path: Vec<_> = route.path.into_iter().take(2).collect();
    let after = path.get(1).ok_or("food is already within feeding reach")?;
    let templates = sim
        .genesis
        .rules
        .spatial_templates
        .as_ref()
        .ok_or("foraging templates absent")?;
    let mut eye = after.at;
    for k in 0..3 {
        eye[k] = eye[k]
            .checked_add(templates.eye[k])
            .ok_or("foraging eye overflow")?;
    }
    Ok(Geometry {
        strides: route.strides.into_iter().take(1).collect(),
        window: if templates.version >= 5 {
            embodiment::site_window(templates, after.at)
        } else {
            window.clone()
        },
        path,
        eye,
    })
}
