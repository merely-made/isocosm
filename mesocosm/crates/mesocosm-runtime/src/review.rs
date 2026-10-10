// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The played line's turn at the boundary, `lineage::Review` over the
//! native session (ruling 762): the status quo first, every variant scored
//! by grow-a-copy, and those the world would refuse kept with the reason.
//! Built when the question opens, never per frame; nothing in it is
//! written back.

use isocosm::directing::interim::Interim;
pub use isocosm::lineage::{Offer, Reading, Review};

/// The review for the played critter's line, when it has one.
pub(crate) fn of(interim: &Interim) -> Option<Review> {
    let critter = interim.critter()?;
    let pop = &interim.session.sim.state().population;
    let lineage = pop.get(critter)?.lineage.clone();
    Review::of(&interim.session, &lineage, interim.pace.scoring).ok()
}
