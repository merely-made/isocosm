// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The shared foreground stepper (case 1084, D50). In view, a body walks a
//! route sixty times a world minute and hands back where it stands at the
//! minute's end. The route, its strides and its feet are the simulator's own
//! (`Simulation::food_route`: the choice of meal, the search and the stride
//! checks native walking uses); the body's speed is the world's
//! (`Rules::speed`). Nothing here reads a window, a renderer or a clock: a
//! minute is a pure function of the simulation and the body, so a windowed
//! round and a headless one step the same poses (D51).
//!
//! A route edge is one cell of travel. A body is `Step`ped along its edges
//! and rests on a stance at each edge's end; the handback is the last stance
//! it reached, so a minute's remainder under one cell is not carried.

use cosmograph::{
    Simulation,
    schema::Id,
    spatial::{ArrivalGeometry, Handback, Pose, embodiment, foraging::FoodRoute},
};

/// Foreground steps in one world minute (case 1080).
pub const STEPS_PER_MINUTE: u32 = 60;

/// Milli-cells in a cell, the unit of `cosmograph::speed::Speed`.
const MILLI: u128 = 1000;

/// Where the body is after a step: on `edge` of the route, `fraction`
/// thousandths of the way along it. `edge` is the route's edge count once the
/// body has arrived.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Step {
    pub edge: usize,
    pub fraction: u32,
}

/// One foreground body's walk over a world minute.
#[derive(Clone, Debug)]
pub struct Walk {
    actor: Id,
    site: Id,
    body_revision: u64,
    terrain_revision: cosmograph::schema::Key,
    route: FoodRoute,
    /// The body's speed in milli-cells a world second.
    speed: u64,
    /// World microseconds one step is a sixtieth of.
    minute_us: u64,
    taken: u32,
}

impl Walk {
    /// The walk `actor` takes this minute along the route its walking profile
    /// `walk` finds, or `None` when it has nowhere to go (no need, food in
    /// reach, no route within the node budget). The world must state a
    /// walking speed.
    pub fn begin(sim: &Simulation, actor: Id, walk: &str) -> Result<Option<Self>, String> {
        let rules = &sim.genesis().rules;
        let speed = rules.speed.ok_or("a foreground walk needs the world's speed")?;
        let e = sim.state().population.get(actor).ok_or("walker absent")?;
        let doc = e.body.as_ref().ok_or("walker has no body")?;
        let site = sim.state().spatial_poses.get(&actor).ok_or("walker is not posed")?.site;
        let Some(route) = sim.food_route(actor, walk)?.route.filter(|r| r.path.len() > 1) else {
            return Ok(None);
        };
        Ok(Some(Self {
            actor,
            site,
            body_revision: e.body_revision,
            terrain_revision: sim.terrain_revision(),
            route,
            speed: speed.of(doc),
            minute_us: rules.tick_microseconds(),
            taken: 0,
        }))
    }

    /// The simulator's route: its poses, with feet, and a stride for each
    /// edge the walking rules witness.
    pub fn route(&self) -> &FoodRoute {
        &self.route
    }

    /// Steps taken so far, at most [`STEPS_PER_MINUTE`].
    pub fn taken(&self) -> u32 {
        self.taken
    }

    fn edges(&self) -> usize {
        self.route.path.len() - 1
    }

    /// Milli-cells travelled after `n` steps, exact in integers.
    fn travelled(&self, n: u32) -> u128 {
        u128::from(self.speed) * u128::from(self.minute_us) * u128::from(n)
            / (u128::from(STEPS_PER_MINUTE) * 1_000_000)
    }

    /// Where the body is after `n` steps, where it has gone no farther than
    /// the route's end.
    pub fn at(&self, n: u32) -> Step {
        let milli = self.travelled(n.min(STEPS_PER_MINUTE));
        let end = self.edges() as u128 * MILLI;
        if milli >= end {
            return Step { edge: self.edges(), fraction: 0 };
        }
        Step { edge: (milli / MILLI) as usize, fraction: (milli % MILLI) as u32 }
    }

    /// Takes one step, the next sixtieth of the minute, or `None` once the
    /// minute is done.
    pub fn step(&mut self) -> Option<Step> {
        if self.taken >= STEPS_PER_MINUTE {
            return None;
        }
        self.taken += 1;
        Some(self.at(self.taken))
    }

    /// The pose of the last stance the body reached after `n` steps.
    pub fn stance(&self, n: u32) -> &Pose {
        &self.route.path[self.at(n).edge]
    }

    /// Distance walked after `n` steps in whole cells of route.
    pub fn cells(&self, n: u32) -> usize {
        self.at(n).edge
    }

    /// The minute's handback for the stance reached after `n` steps (the
    /// whole minute at [`STEPS_PER_MINUTE`]), or `None` if the body has not
    /// left its first stance, in which case the sim keeps its own pose. A
    /// body that stands within reach of its meal names it as a contact.
    pub fn handback(&self, sim: &Simulation, n: u32) -> Result<Option<Handback>, String> {
        let at = self.at(n).edge;
        if at == 0 {
            return Ok(None);
        }
        let pose = self.route.path[at].clone();
        let t = sim.genesis().rules.spatial_templates.as_ref().ok_or("walking needs templates")?;
        let contacts = if at == self.edges() { vec![self.route.target] } else { Vec::new() };
        Ok(Some(Handback {
            actor: self.actor,
            site: self.site,
            body_revision: self.body_revision,
            terrain_revision: self.terrain_revision.clone(),
            geometry: ArrivalGeometry {
                window: embodiment::site_window(t, pose.at),
                eye: std::array::from_fn(|k| pose.at[k] + t.eye[k]),
                pose,
                places: t.places,
            },
            contacts,
        }))
    }
}

/// The handbacks of a participant's foreground for one minute: each
/// foreground body that has a walk, in the set's order, its sixty steps
/// taken. The foreground is the sim's (`Simulation::foreground`), so the
/// played body leads and the cap is the world's.
pub fn foreground_minute(sim: &Simulation, participant: Id, walk: &str) -> Result<Vec<Handback>, String> {
    let mut out = Vec::new();
    for actor in sim.foreground(participant) {
        // A body the player steers has no route of the sim's choosing.
        let Some(w) = Walk::begin(sim, actor, walk).ok().flatten() else { continue };
        if let Some(h) = w.handback(sim, STEPS_PER_MINUTE)? {
            out.push(h);
        }
    }
    Ok(out)
}
