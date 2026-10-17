// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Body-part inspection (dev ground truth): `I` opens it, `J`/`L` walk the
//! followed body's drawn parts, `U` clears.

use winit::keyboard::Key;

use crate::section::BodySelection;

#[derive(Default)]
pub(super) struct Inspection {
    pub open: bool,
    pub selected: Option<BodySelection>,
    pub notice: &'static str,
}

impl Inspection {
    /// Shared key semantics for the legacy host and the Assembly head.
    pub(super) fn key(
        &mut self,
        key: &Key,
        dev: bool,
        followed: Option<u64>,
        mut select: impl FnMut(u64, Option<BodySelection>, bool) -> Option<BodySelection>,
        mut validate: impl FnMut(BodySelection) -> bool,
    ) -> bool {
        if !dev {
            return false;
        }
        let letter = match key {
            Key::Character(value) => value.to_lowercase(),
            _ => String::new(),
        };
        if letter == "i" {
            self.open = !self.open;
            self.selected = None;
            if self.open {
                self.select_part(false, followed, &mut select, &mut validate);
            }
            return true;
        }
        if !self.open {
            return false;
        }
        match letter.as_str() {
            "j" => self.select_part(true, followed, &mut select, &mut validate),
            "l" => self.select_part(false, followed, &mut select, &mut validate),
            "u" => {
                self.selected = None;
                self.notice = "Selection cleared. J/L selects a drawn part.";
            },
            // Time, follow and camera stay live while inspecting.
            "p" | "." | "," | "[" | "]" | "n" | "b" | "m" => return false,
            _ => {},
        }
        true
    }

    fn select_part(
        &mut self,
        backwards: bool,
        followed: Option<u64>,
        select: &mut impl FnMut(u64, Option<BodySelection>, bool) -> Option<BodySelection>,
        validate: &mut impl FnMut(BodySelection) -> bool,
    ) {
        self.update(followed, &mut *validate);
        self.selected = followed
            .and_then(|id| select(id, self.selected, backwards))
            .filter(|selection| validate(*selection));
        self.notice = if self.selected.is_some() {
            ""
        } else {
            "No exact drawn part available. J/L retries after a frame."
        };
    }

    pub(super) fn update(
        &mut self,
        followed: Option<u64>,
        mut validate: impl FnMut(BodySelection) -> bool,
    ) {
        let Some(selected) = self.selected else {
            return;
        };
        if followed != Some(selected.organism) || !validate(selected) {
            self.selected = None;
            self.notice = "Selection expired: body or view changed. J/L selects again.";
        }
    }
}
