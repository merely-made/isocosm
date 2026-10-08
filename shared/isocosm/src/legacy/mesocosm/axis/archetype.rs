// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Authored creatures: the bodies a world is meant to open holding.
//!
//! The bodies, palettes and the reasons for them live in the founding
//! datasheets (`datasheets/foundings/`, wing design record rulings 625 to
//! 632), read by [`datasheet`]. These functions are their names in code: each
//! looks its body or palette up in the embedded sheets, so a body is
//! developed by the same `Recipe -> Soma -> develop_body` path every other
//! body takes and nothing here builds one.
//!
//! The functions here are the base roster, `base.toml`; [`branching`],
//! [`jointed`] and [`spaced`] are the later roster sets. Which bodies a
//! founding installs, per tier, is `foundings.toml`'s to say.

use super::Recipe;
use crate::legacy::mesocosm::development::PartPalette;

/// Names a sheet's bodies in code, each resolved in one palette.
macro_rules! bodies {
    ($palette:literal: $($(#[$doc:meta])* $name:ident => $body:literal),* $(,)?) => {
        $(
            $(#[$doc])*
            pub fn $name() -> Recipe {
                crate::legacy::mesocosm::axis::archetype::datasheet::body($body, $palette)
            }
        )*
    };
}

/// The primitive palette plus the shapes the base roster is carved from.
pub fn palette() -> PartPalette {
    datasheet::palette("base")
}

bodies!("base":
    /// A low ground mat of flat pads.
    producer_mat => "base.producer_mat",
    /// Two leaf sizes on one branching runner.
    producer_shrub => "base.producer_shrub",
    /// A long bare stem carrying one crown of big fronds.
    producer_stalk => "base.producer_stalk",
    /// A long-necked hexapod that crops the stand.
    consumer_browser => "base.consumer_browser",
    /// A short-bodied sprinter with a swinging jaw.
    consumer_pursuit => "base.consumer_pursuit",
    /// A small cropper wearing its plates as covering.
    consumer_armoured => "base.consumer_armoured",
    /// A flat creeper whose pads lie in the litter.
    decomposer_crust => "base.decomposer_crust",
    /// An eight-legged walker that finds the dead.
    decomposer_detritivore => "base.decomposer_detritivore",
);

#[cfg(test)]
mod tests;

pub mod branching;
pub mod datasheet;
pub mod jointed;
pub mod spaced;
