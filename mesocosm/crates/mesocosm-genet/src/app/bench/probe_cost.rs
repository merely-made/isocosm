// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Which bench scenes the shared cost receipt is counting. The sampling,
//! bounding and summary policy are `mesquite::Costs`; the scene walk is
//! the bench's, because only the bench knows it owns one main scene plus five
//! comparison cards.

use mesquite::{CostObservation, Totals};

use super::Context;

pub(super) fn observation(ctx: &Context<'_>) -> CostObservation {
    let state = ctx.runner.state();
    let mut totals = Totals::default();
    let mut populated = false;
    for scene in std::iter::once(&state.scene).chain(state.cards.iter()) {
        let s = scene.borrow();
        totals.redraws += s.renders();
        totals.mesh_bytes += s.mesh_upload_bytes;
        totals.instance_bytes += s.instance_upload_bytes;
        populated |= s.stats.voxel_bodies > 0;
    }
    // Retired producers keep diagnostic history. Only a producer whose DOM is
    // active may invalidate the current document's sample.
    let mut valid = true;
    if state.visible && !state.effects.open {
        valid = state.scene.borrow().error.is_none();
        if let Some(comparison) = &state.model.borrow().comparison {
            for (index, card) in comparison.cards.iter().enumerate() {
                if card.world.is_some() {
                    valid &= state.cards[index].borrow().error.is_none();
                }
            }
        }
    }
    CostObservation {
        totals,
        valid,
        populated,
    }
}
