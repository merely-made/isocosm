// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{rules::Process, simulation::Receipt};

#[derive(Clone, Debug, Default)]
pub(crate) struct Report { pub requests: Vec<DebitRequest>, pub claims: Vec<Key>, pub failed: bool }

impl Simulation {
    pub(crate) fn begin_process_debits(&mut self, actor:Id, target:Option<Id>, process:&str, act:u64) -> Result<()> {
        self.owned_report = Report::default();
        if self.state.projects.debit_capture.active() { return Ok(()); }
        if self.state.projects.ownership.is_empty() { return Ok(()); }
        let action = self.owned_action(actor,process,act);
        let origin = DebitOrigin::Process { process:process.into(),actor,target };
        let mut context = Context::new(&self.state,&self.genesis.rules,
            crate::digest(&(&self.revision,&self.state)),origin,action,Mode::Authored);
        context.produced = self.produced_at_start();
        self.state.projects.debit_capture.set(Some(context));
        Ok(())
    }
    pub(crate) fn finish_process_debits(&mut self, receipt:&Receipt) {
        if let Some(context) = self.state.projects.debit_capture.take() {
            self.owned_report.requests = context.requests;
            self.owned_report.failed = context.failed;
            if receipt.accepted() { self.owned_report.claims = context.claims; }
        }
    }
    pub(crate) fn complete_process_debits(&self) -> Result<()> {
        self.state.projects.debit_capture.with(|context| context.complete()).map(|_| ())
    }
    pub(crate) fn owned_action(&self, actor:Id, process:&str, act:u64) -> Key {
        format!("event:{}",crate::digest(&(self.genesis.seed,self.state.tick,act,actor,process)))
    }
    pub(crate) fn clear_owned_capture(&mut self) {
        self.state.projects.debit_capture.set(None);
        self.owned_report = Report::default();
        self.owned_planning = false;
    }
    /// Reads the same pass frame as the native choice, without publishing any writes.
    pub(crate) fn owned_process_ready(&self, actor:Id, target:Option<Id>, process:&Process) -> bool {
        if self.owned_planning || self.state.projects.ownership.is_empty() { return true; }
        let mut candidate = self.clone();
        candidate.state.projects.debit_capture.set(None);
        candidate.owned_planning = true;
        let act = candidate.state.next_action;
        let _ = candidate.demands(actor,target,&process.id,1,act);
        !candidate.owned_report.failed
    }
}
