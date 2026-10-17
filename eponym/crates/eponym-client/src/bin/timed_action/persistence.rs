// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl App {
    pub(super) fn save(&mut self) {
        self.status = match self
            .action
            .save()
            .map_err(|e| format!("{e:?}"))
            .and_then(|bytes| {
                let pending = self.save_path.with_extension("pending");
                std::fs::write(&pending, bytes).map_err(|e| e.to_string())?;
                std::fs::rename(&pending, &self.save_path).map_err(|e| e.to_string())
            }) {
            Ok(()) => vec![format!("Saved {}", self.save_path.display())],
            Err(e) => vec![format!("Save failed: {e}")],
        };
        self.redraw();
    }
    pub(super) fn load(&mut self) {
        self.status = match std::fs::read(&self.save_path)
            .map_err(|e| e.to_string())
            .and_then(|bytes| {
                TimedActionSession::restore_solving(&bytes, eponym_motion::SOLVER)
                    .map_err(|e| format!("{e:?}"))
            }) {
            Ok(action) => {
                if action.session().game().bodies().get(self.target).is_none() {
                    vec!["Load failed: saved session has no selected target".into()]
                } else {
                    self.action = action;
                    self.held = false;
                    self.movement_keys = [false; 4];
                    self.last_motion = Instant::now();
                    self.last_charge = Instant::now();
                    vec![format!("Loaded {}", self.save_path.display())]
                }
            },
            Err(e) => vec![format!("Load failed: {e}")],
        };
        self.redraw();
    }
}
