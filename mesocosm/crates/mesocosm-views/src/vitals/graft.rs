// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A graft allowance's receipt in words (`rules::Compatibility`, 755).

use isocosm::rules::GraftReceipt;

pub fn compatibility_words(receipt: &GraftReceipt) -> String {
    let requested = receipt.incoming_mg.saturating_add(receipt.retained_mg);
    let mut words = format!(
        "{} / {} mg allowance ({} mg remaining); {} mg extra reserve cost",
        requested,
        receipt.effective_allowance_mg,
        receipt.effective_allowance_mg.saturating_sub(requested),
        receipt.penalty_mg,
    );
    for condition in &receipt.applied {
        words.push_str(&format!("; raised by {condition}"));
    }
    words
}
