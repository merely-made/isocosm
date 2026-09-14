// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Graft provenance and checked compatibility terms.

use mesocosm_core::Graft;

/// The two sentences a transferred branch is owed: where it came from, and
/// what it is doing here. (P3)
///
/// Provenance is the first of them because it is the thing a graft has that
/// growing does not: this tissue was somebody. The second is the verdict made
/// legible — a carried branch that arrived native works, a carried branch over
/// a cross-domain edge is on you and doing nothing, and a regrown one is doing
/// whatever your own rules make of that shape.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraftWords {
    /// Which parts came, off which part of which line.
    pub taken: String,
    /// The crossing, the verdict, and what that leaves the branch doing.
    pub terms: String,
}

/// A branch transfer in plain words.
pub fn graft_words(graft: &Graft, expressing: bool) -> GraftWords {
    let taken = format!(
        "{} part{} from part {} of line {}",
        graft.parts.len(),
        if graft.parts.len() == 1 { "" } else { "s" },
        graft.donor_part.0,
        graft.donor_line.0,
    );
    // What the branch is *doing* is read off the body rather than inferred from
    // the verdict, because they are two different facts and a panel that
    // guessed the second from the first would be describing the table instead
    // of the creature.
    let doing = if expressing {
        "working"
    } else {
        "doing nothing yet"
    };
    let affinity = graft.compatibility.as_ref().map_or_else(
        || graft.verdict.name().to_owned(),
        |receipt| format!("disfavoured carry; {}", compatibility_words(receipt)),
    );
    GraftWords {
        taken,
        terms: format!(
            "{} on part {} — {}, {}",
            graft.crossing.name(),
            graft.root.0,
            affinity,
            doing
        ),
    }
}

/// The core's checked graft terms, shared by preview and landed readings.
pub fn compatibility_words(
    receipt: &mesocosm_core::graft::compatibility::CompatibilityReceipt,
) -> String {
    let mut words = format!(
        "{} / {} mg allowance ({} mg remaining); {} mg extra reserve cost",
        receipt.requested_mg,
        receipt.effective_allowance_mg,
        receipt
            .effective_allowance_mg
            .saturating_sub(receipt.requested_mg),
        receipt.penalty_mg,
    );
    for condition in &receipt.applied {
        if let Some(name) = mesocosm_core::discovery::name_of(*condition) {
            words.push_str(&format!("; raised by {name}"));
        }
    }
    words
}
