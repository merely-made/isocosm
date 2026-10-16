// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The window's side of input (sim-host plan H4, rulings 954 and 955): a
//! winit key lowered once into the core's [`Press`], and the keys that only
//! move host state. What a press asks of the world is Mesocosm's keymap in
//! [`mesocosm_runtime::runtime::keymap`], reached through the core's driver,
//! which caps the backlog (957) so mashing a key cannot build a session-long
//! debt. Auto-repeat is dropped by the window handler; a replay never comes
//! through here.
//!
//! # Dev keys (DT1, DT2, DT3)
//!
//! Live only while `--dev` is set: `P` pause, `.` step, `,` step ten, `[`/`]`
//! speed, `N`/`B` follow the next or previous living critter, `M` follow the
//! played one. Those eight move host state. `X` end the epoch, `F` force a
//! birth, `K` kill and `G` place matter are the keymap's dev intents.

use mesocosm_runtime::{Named, Press};
use winit::keyboard::{Key, NamedKey};

/// The core's press for a winit key; `None` for a key no keymap names.
pub fn lower(key: &Key) -> Option<Press> {
    Some(match key {
        Key::Named(named) => Press::Named(match named {
            NamedKey::Space => Named::Space,
            NamedKey::Enter => Named::Enter,
            NamedKey::Tab => Named::Tab,
            NamedKey::Escape => Named::Escape,
            NamedKey::ArrowUp => Named::ArrowUp,
            NamedKey::ArrowDown => Named::ArrowDown,
            NamedKey::ArrowLeft => Named::ArrowLeft,
            NamedKey::ArrowRight => Named::ArrowRight,
            _ => return None,
        }),
        Key::Character(c) => {
            let mut chars = c.chars();
            match (chars.next(), chars.next()) {
                (Some(c), None) => Press::char(c),
                _ => return None,
            }
        },
        _ => return None,
    })
}

/// Whether a key moves the trait board's cursor, which is host state.
pub fn board_cursor(key: &Key) -> bool { matches!(key, Key::Named(NamedKey::Tab)) }

/// A dev-only action a key maps to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DevKey {
    TogglePause,
    Step,
    StepN,
    SlowDown,
    SpeedUp,
    FollowNext,
    FollowBack,
    FollowSelf,
    EndEpoch,
    ForceBirth,
    Kill,
    PlaceMatter,
}

impl DevKey {
    /// Whether this key queues a dev intent rather than moving host state.
    pub fn changes_the_world(self) -> bool {
        matches!(
            self,
            Self::EndEpoch | Self::ForceBirth | Self::Kill | Self::PlaceMatter
        )
    }
}

pub fn dev_key(key: &Key) -> Option<DevKey> {
    let Key::Character(c) = key else { return None };
    Some(match c.to_lowercase().as_str() {
        "p" => DevKey::TogglePause,
        "." => DevKey::Step,
        "," => DevKey::StepN,
        "[" => DevKey::SlowDown,
        "]" => DevKey::SpeedUp,
        "n" => DevKey::FollowNext,
        "b" => DevKey::FollowBack,
        "m" => DevKey::FollowSelf,
        "x" => DevKey::EndEpoch,
        "f" => DevKey::ForceBirth,
        "k" => DevKey::Kill,
        "g" => DevKey::PlaceMatter,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_lower_once_and_lowercase() {
        assert_eq!(lower(&Key::Character("E".into())), Some(Press::Char('e')));
        assert_eq!(lower(&Key::Named(NamedKey::Space)), Some(Press::Named(Named::Space)));
        assert_eq!(lower(&Key::Named(NamedKey::F1)), None);
    }

    #[test]
    fn dev_keys_never_collide_with_play_keys() {
        for key in ["e", "q", "s", "t", "r"] {
            assert_eq!(dev_key(&Key::Character(key.into())), None, "{key}");
        }
        assert!(!board_cursor(&Key::Named(NamedKey::Enter)));
    }
}
