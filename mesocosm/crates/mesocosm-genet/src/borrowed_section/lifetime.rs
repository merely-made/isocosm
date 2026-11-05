// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
use super::SceneHandle;

impl SceneHandle {
    pub(crate) fn lifetime(&self) -> u64 {
        self.lifetime.get()
    }
    pub(crate) fn is_lifetime(&self, lifetime: u64) -> bool {
        self.lifetime() == lifetime
    }
    /// Source replacement revokes pending and accepted queries even if a new
    /// world happens to reuse its numeric revision and subject identifiers.
    pub(crate) fn revoke_source(&self) {
        self.lifetime.set(
            self.lifetime()
                .checked_add(1)
                .expect("scene source lifetime exhausted"),
        );
        self.input.borrow_mut().take();
    }
    pub fn clear(&self) -> bool {
        self.error.borrow_mut().take();
        let had_input = self.input.borrow().is_some();
        if had_input {
            self.revoke_source();
        }
        had_input
    }
}
