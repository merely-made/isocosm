// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Lineages and the boundary, natively (the families plan's fifth family;
//! rulings 57, 182, 684 and 752). A line is a `schema::Lineage` keyed by
//! name: its descent is `tree`, splitting it is `speciate`, its program is
//! the log's committed revisions (`program`), what it amounts to at a
//! boundary is `reckon`, and the played line's turn is `review`. The
//! unplayed lines' turn and the revision itself stay where directing built
//! them (`directing::interim::boundary`, `directing::revise`).
//!
//! What a line learns is the stage's: eating a part whole teaches its kind
//! to the eater's lexicon (468), which is what a revision draws on.

pub mod body;
pub mod program;
pub mod reckon;
pub mod review;
pub mod speciate;
pub mod tree;

pub use body::*;
pub use program::Revision;
pub use reckon::Reading;
pub use review::{Offer, Review};
pub use speciate::speciated_at;

#[cfg(test)]
mod tests;
