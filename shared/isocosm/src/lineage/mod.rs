// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Lineages and the boundary, natively (the families plan's fifth family;
//! rulings 57, 182, 684 and 752). A line is a `schema::Lineage` keyed by
//! name: its descent is `tree`, splitting it is `speciate`, its program is
//! the log's committed revisions (`program`), what it amounts to at a
//! boundary is `reckon`, the played line's turn is `review`, the unplayed
//! lines' turn is `boundary`, and the revision itself is `revise`, both
//! moved here from directing (763), which re-exports them.
//!
//! What a line learns is the stage's: eating a part whole teaches its kind
//! to the eater's lexicon (468), which is what a revision draws on.

pub mod body;
pub mod boundary;
pub mod chronicle;
pub mod program;
pub mod reckon;
pub mod review;
pub mod revise;
pub mod speciate;
pub mod tree;

pub use body::*;
pub use chronicle::{Chronicle, Consequence};
pub use program::Revision;
pub use reckon::Reading;
pub use review::{Offer, Review};
pub use speciate::speciated_at;

#[cfg(test)]
mod tests;
