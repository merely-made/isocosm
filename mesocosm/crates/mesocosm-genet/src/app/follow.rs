// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Where the camera looks (DT2): the followed body, which is the played one
//! unless `--dev` moved it. Following moves the camera and nothing else, so
//! none of it reaches the queue.

use isocosm::schema::Id;
use isocosm::simulation::Simulation;
use mesocosm_views::Lost;

use super::Host;
use crate::input;

impl Host {
    /// The body the camera is on: the followed one, else the played one.
    pub(super) fn followed(&self) -> Option<Id> {
        self.follow.or_else(|| self.runtime.critter())
    }

    /// Where the camera centres: the followed body's presentation spot, else
    /// the played site's centre column.
    pub(super) fn follow_at(&self) -> [i32; 3] {
        let Some(scene) = &self.scene else {
            return [0, 0, 0];
        };
        self.followed()
            .and_then(|id| scene.at(id))
            .unwrap_or_else(|| crate::played::layout::centre(&scene.ground))
    }

    pub(super) fn follow_key(&mut self, action: input::DevKey) {
        let sim = self.runtime.sim();
        let from = self.followed();
        self.follow = match action {
            input::DevKey::FollowSelf => None,
            input::DevKey::FollowNext => next_living(sim, from, false),
            input::DevKey::FollowBack => next_living(sim, from, true),
            _ => self.follow,
        };
        self.follow_lost = None;
        if self.follow == self.runtime.critter() {
            self.follow = None;
        }
    }

    /// Drops a followed body that is no longer alive, and says so.
    pub(super) fn update_follow(&mut self) {
        let Some(target) = self.follow else { return };
        let pop = &self.runtime.sim().state().population;
        if pop.get(target).is_some_and(|e| e.alive) {
            return;
        }
        self.follow_lost = Some(mesocosm_views::lost_of(target, self.runtime.tick()));
        self.follow = None;
    }

    pub(super) fn follow_reading(&self) -> (Option<mesocosm_views::Follow>, Option<Lost>) {
        let follow = self
            .followed()
            .and_then(|id| mesocosm_views::follow_of(self.runtime.sim(), id));
        (follow, self.follow_lost)
    }
}

/// The next living cohort in id order from `from`, wrapping.
fn next_living(sim: &Simulation, from: Option<Id>, back: bool) -> Option<Id> {
    let groups = &sim.state().population.groups;
    let ids: Vec<Id> = groups
        .iter()
        .filter(|(_, group)| group.entity.alive)
        .map(|(id, _)| *id)
        .collect();
    let from = from?;
    if back {
        ids.iter()
            .rev()
            .find(|id| **id < from)
            .or(ids.last())
            .copied()
    } else {
        ids.iter().find(|id| **id > from).or(ids.first()).copied()
    }
}
