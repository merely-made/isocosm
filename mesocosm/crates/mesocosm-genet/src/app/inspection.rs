// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Body-part inspection (dev ground truth): `I` opens it, `J`/`L` walk the
//! followed body's drawn parts, `U` clears. Host state only.

use winit::keyboard::Key;

use super::Host;
use crate::section::BodySelection;

#[derive(Default)]
pub(super) struct Inspection {
    pub open: bool,
    pub selected: Option<BodySelection>,
    pub notice: &'static str,
}

impl Host {
    pub(super) fn try_inspection_key(&mut self, key: &Key) -> bool {
        if !self.config.dev {
            return false;
        }
        let letter = match key {
            Key::Character(value) => value.to_lowercase(),
            _ => String::new(),
        };
        if letter == "i" {
            self.inspection.open = !self.inspection.open;
            self.inspection.selected = None;
            if self.inspection.open {
                self.select_part(false);
            }
            return true;
        }
        if !self.inspection.open {
            return false;
        }
        match letter.as_str() {
            "j" => self.select_part(true),
            "l" => self.select_part(false),
            "u" => {
                self.inspection.selected = None;
                self.inspection.notice = "Selection cleared. J/L selects a drawn part.";
            },
            // Time, follow and camera stay live while inspecting.
            "p" | "." | "," | "[" | "]" | "n" | "b" | "m" => return false,
            _ => {},
        }
        true
    }

    fn select_part(&mut self, backwards: bool) {
        self.update_inspection();
        let followed = self.followed();
        let selected = match (followed, self.gpu.as_mut(), &self.scene) {
            (Some(id), Some(gpu), Some(scene)) => gpu
                .section
                .select_part(id, self.inspection.selected, backwards)
                .filter(|selection| gpu.section.validate_selection(*selection, scene)),
            _ => None,
        };
        self.inspection.selected = selected;
        self.inspection.notice = if selected.is_some() {
            ""
        } else {
            "No exact drawn part available. J/L retries after a frame."
        };
    }

    pub(super) fn update_inspection(&mut self) {
        let Some(selected) = self.inspection.selected else {
            return;
        };
        let valid = self.followed() == Some(selected.organism)
            && match (self.gpu.as_mut(), &self.scene) {
                (Some(gpu), Some(scene)) => gpu.section.validate_selection(selected, scene),
                _ => false,
            };
        if !valid {
            self.inspection.selected = None;
            self.inspection.notice = "Selection expired: body or view changed. J/L selects again.";
        }
    }
}
