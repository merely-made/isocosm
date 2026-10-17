// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Runtime outcomes consumed by the shared mesquite scenario lane.

use super::Host;

impl Host {
    pub(super) fn note_outcomes(&mut self) {
        if self.config.scenario.is_none() {
            return;
        }
        let trace = self.runtime.trace();
        if trace.len() == self.noted {
            return;
        }
        for (envelope, outcome) in &trace[self.noted.min(trace.len())..] {
            self.events
                .push(format!("outcome {:?} {outcome:?}", envelope.intent));
        }
        for happening in self.runtime.happenings() {
            self.events.push(format!("happening {happening:?}"));
        }
        self.noted = trace.len();
    }

}
