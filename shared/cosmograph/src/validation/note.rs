// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use crate::Result;

/// Declared note names share native key spelling, with an optional namespace.
pub(super) fn kind(value: &str) -> Result<()> {
    if value.contains(':') {
        return super::key(value);
    }
    let valid = !value.is_empty()
        && value.len() <= 256
        && value
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"_-/ .".contains(&c) && c != b' ');
    if valid {
        Ok(())
    } else {
        Err(format!("invalid note kind {value:?}"))
    }
}
