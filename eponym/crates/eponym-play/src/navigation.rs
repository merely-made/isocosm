// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Derived routes in the played site, over isometer's places (wing ruling
//! 702): waypoints between the crossings a body fits. Advice only; movement
//! saves keep accepted inputs, never a planner's route.

use isometer_space::places::{Body, Places, Rules};

use crate::World;
use crate::walking::{WALKER_HEIGHT, to_ground, to_site};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Navigation {
    /// The body routes are found for.
    pub body: Body,
}

impl Default for Navigation {
    fn default() -> Self {
        let height = WALKER_HEIGHT as u32;
        Self { body: Body { width: 1, height, climb: 1, wades: false } }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavigationError {
    /// The site's places could not be read.
    NoPlaces,
    NoRoute { from: [i32; 3], toward: [i32; 3] },
}

impl Navigation {
    /// Waypoints from `from` to `target` in the played site, `from` first.
    pub fn route_to_position(
        &self,
        world: &World,
        from: [i32; 3],
        target: [i32; 3],
    ) -> Result<Vec<[i32; 3]>, NavigationError> {
        let v = world.volume();
        let places = Places::derive(v, Rules::default()).map_err(|_| NavigationError::NoPlaces)?;
        let walk = places.walk(to_site(v, from), to_site(v, target), &self.body);
        let walk = walk.ok_or(NavigationError::NoRoute { from, toward: target })?;
        let mut route = vec![from];
        route.extend(walk.into_iter().map(|at| to_ground(v, at)));
        route.dedup();
        Ok(route)
    }
}
