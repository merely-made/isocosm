// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::app::reading::CommittedSubject;
use cambium_rootstock::{PreparedPresentationObserver, PresentedFrame};
use mesocosm_runtime::continuation::LoopReading;

/// Product facts belong to the candidate's prepare cut. They are never read
/// back out of mutable State when GPU validation completes later.
pub(super) struct Facts {
    pub subject: Option<CommittedSubject>,
    pub picture: Option<Rc<crate::borrowed_section::Snapshot>>,
    continuation: Option<LoopReading>,
    lifetime: u64,
}
pub(super) struct Accepted {
    pub stamp: PresentedFrame,
    pub facts: Facts,
}
#[derive(Default)]
pub(super) struct Publication {
    pending: Rc<RefCell<Option<Accepted>>>,
    pub accepted: Option<Accepted>,
}

impl State {
    fn presentation_facts(&self) -> Facts {
        Facts {
            subject: self.committed.clone(),
            picture: self.picture.snapshot(),
            continuation: self.host.prepared_continuation(),
            lifetime: self.picture.lifetime(),
        }
    }
    pub(super) fn prepare_publication(
        &self,
        proposed: PresentedFrame,
    ) -> PreparedPresentationObserver {
        let facts = self.presentation_facts();
        let lifetime = facts.lifetime;
        let handle = self.picture.clone();
        let sink = self.presentation.pending.clone();
        PreparedPresentationObserver::new(move |actual| {
            if actual == proposed {
                *sink.borrow_mut() = Some(Accepted {
                    stamp: actual,
                    facts,
                });
            }
        })
        .while_valid(move || handle.is_lifetime(lifetime))
    }
    pub(super) fn adopt_presentation(&mut self) {
        let Some(accepted) = self.presentation.pending.borrow_mut().take() else {
            return;
        };
        if !self.picture.is_lifetime(accepted.facts.lifetime) {
            return;
        }
        self.host.frames += 1;
        self.host
            .mark_continuation_presented(accepted.facts.continuation.clone());
        // A newer desired selection stays pending; validation of A must not
        // expire B merely because only A has reached the presentation boundary.
        let same_cut = self
            .committed
            .as_ref()
            .zip(accepted.facts.subject.as_ref())
            .is_some_and(|(desired, shown)| {
                desired.native == shown.native
                    && desired.revision == shown.revision
                    && desired.selection == shown.selection
            });
        if same_cut {
            let followed = self.host.followed();
            let selected = self.host.inspection.selected;
            self.host.inspection.update(followed, |selection| {
                self.producer.borrow_mut().validate_selection(selection)
            });
            self.refresh_required |= selected != self.host.inspection.selected;
        }
        self.presentation.accepted = Some(accepted);
    }
}

#[cfg(test)]
pub(super) mod tests;
