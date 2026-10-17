// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use std::collections::BTreeMap;

use isocosm::Session;

/// An observer that can refuse a candidate before a World publication.
///
/// Implementations must not re-enter the World. This boundary supplies both
/// immutable states while the core holds its publication borrow.
pub trait CommitGate {
    fn before_publish(
        &self,
        before: &Session,
        before_sections: &BTreeMap<String, Vec<u8>>,
        before_revision: u64,
        candidate: &Session,
        candidate_sections: &BTreeMap<String, Vec<u8>>,
        candidate_revision: u64,
    ) -> Result<(), String>;
}
