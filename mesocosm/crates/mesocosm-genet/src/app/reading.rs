// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! One selected native subject captured for the inspector and scene accent.

use super::Host;
use crate::section::BodySelection;
use cosmograph::simulation::Simulation;
use cosmograph_host::world::NativeRevision;

#[derive(Clone, Debug)]
pub(super) struct CommittedSubject {
    pub native: NativeRevision,
    pub revision: u64,
    pub selection: Option<BodySelection>,
    pub inspection: Option<mesocosm_views::PartInspection>,
}

impl CommittedSubject {
    /// The caller holds the shared world's read guard across every reading
    /// and both publications. BodySelection still carries the real renderer's
    /// dependency token; this capture does not manufacture geometry evidence.
    pub fn capture(host: &Host, sim: &Simulation, native: NativeRevision, revision: u64) -> Self {
        let selection = host.inspection.selected;
        let inspection = host.inspection.open.then(|| {
            selection.map_or_else(
                || mesocosm_views::PartInspection {
                    reading: None,
                    notice: Some(host.inspection.notice.into()),
                },
                |selected| {
                    mesocosm_views::part_of_with_terms(
                        sim,
                        selected.organism,
                        selected.part,
                        host.runtime.terms(),
                    )
                },
            )
        });
        Self {
            native,
            revision,
            selection,
            inspection,
        }
    }
}
