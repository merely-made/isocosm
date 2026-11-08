// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::collections::BTreeMap;
use std::num::NonZeroU64;

use isometer::hybrid::{HybridPolicy, HybridSettings};
use isometer::{BodyDisplay, SceneHost, SubjectKey};

use crate::appearance::Library;

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
    /// Each token's class style where it is not the baseline (case 1190).
    styles: BTreeMap<SubjectKey, BodyDisplay>,
}

impl BoardHost {
    pub(super) fn new(quality: BoardBodyQuality, examined: Option<SubjectKey>) -> Self {
        Self {
            quality,
            examined,
            styles: BTreeMap::new(),
        }
    }

    /// Resolve each token's class style through the packs' shared style
    /// table. Baseline tokens are left out, so a board of baseline classes
    /// draws and signs exactly as before.
    pub(super) fn styled<'a>(
        mut self,
        library: &Library,
        tokens: impl IntoIterator<Item = (SubjectKey, &'a str)>,
    ) -> Self {
        self.styles = tokens
            .into_iter()
            .map(|(subject, class)| (subject, library.style(class).display))
            .filter(|(_, display)| *display != BodyDisplay::default())
            .collect();
        self
    }

    pub(super) fn append_inputs(&self, inputs: &mut Vec<u64>) {
        if self.quality == BoardBodyQuality::Hybrid {
            // The tagged shape distinguishes live, hybrid without selection,
            // and hybrid with any possible subject ID.
            inputs.push(1);
            inputs.extend(self.examined.map(|subject| subject.0));
        }
        if !self.styles.is_empty() {
            inputs.push(2);
            for (subject, display) in &self.styles {
                inputs.extend([subject.0, *display as u64]);
            }
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

    fn body_display(&self, subject: SubjectKey) -> BodyDisplay {
        self.styles.get(&subject).copied().unwrap_or_default()
    }
}
