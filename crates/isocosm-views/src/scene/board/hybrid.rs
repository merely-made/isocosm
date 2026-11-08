// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::num::NonZeroU64;

use isometer::hybrid::{HybridPolicy, HybridSettings};
use isometer::{SceneHost, SubjectKey};

#[cfg(test)]
mod tests;

/// Local scene quality. Campaign input, visibility and picking use the same
/// native data at either quality.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BoardBodyQuality {
    #[default]
    Live,
    Hybrid,
}

pub(super) struct BoardHost {
    quality: BoardBodyQuality,
    examined: Option<SubjectKey>,
}

impl BoardHost {
    pub(super) fn new(quality: BoardBodyQuality, examined: Option<SubjectKey>) -> Self {
        Self { quality, examined }
    }

    pub(super) fn append_inputs(&self, inputs: &mut Vec<u64>) {
        if self.quality == BoardBodyQuality::Hybrid {
            // The tagged shape distinguishes live, hybrid without selection,
            // and hybrid with any possible subject ID.
            inputs.push(1);
            inputs.extend(self.examined.map(|subject| subject.0));
        }
    }
}

impl SceneHost for BoardHost {
    fn hybrid_settings(&self) -> Option<HybridSettings> {
        if self.quality != BoardBodyQuality::Hybrid {
            return None;
        }
        let policy = HybridPolicy::new(
            [[12.0, 16.0], [26.0, 30.0], [44.0, 56.0]],
            [12, 4, 2].map(|period| NonZeroU64::new(period).unwrap()),
        )
        .expect("board hybrid trial bands and periods are ordered");
        Some(HybridSettings::new(policy, 32, 4 * 1024 * 1024, 8))
    }

    fn hybrid_examined(&self, subject: SubjectKey) -> bool {
        self.examined == Some(subject)
    }
}
