// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Actual operator file action. The file supplies every identity, current
//! revision, attachment and selected stock portion; this chooses none.

use super::Host;
use std::path::Path;
use mesocosm_runtime::runtime::graft::GraftInput;

impl Host {
    pub(super) fn graft_file(&mut self, path: &Path, apply: bool) -> bool {
        let result = std::fs::read(path).map_err(|why| why.to_string())
            .and_then(|bytes| serde_json::from_slice::<GraftInput>(&bytes).map_err(|why| why.to_string()))
            .and_then(|input| {
                if apply { self.runtime.apply_graft(&input) } else {
                    self.runtime.prepare_graft(&input).and_then(|quote|
                        serde_json::to_string(&quote).map_err(|why| why.to_string()))
                }
            });
        let action = if apply { "graft" } else { "graft-preview" };
        match result {
            Ok(reading) => {
                eprintln!("{action} {}: {reading}", path.display());
                self.events.push(format!("{action} {reading}"));
                if apply {
                    self.scene_key = None;
                    self.refresh_scene();
                }
                true
            },
            Err(why) => {
                eprintln!("{action} {}: {why}; native source {} at tick {}",
                    path.display(), self.runtime.graft_source(), self.runtime.tick());
                self.events.push(format!("{action}-refused {why}"));
                false
            },
        }
    }
}
