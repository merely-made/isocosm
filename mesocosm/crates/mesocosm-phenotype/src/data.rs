// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Reading a pack data file in either syntax (wing design record, ruling 624).
//!
//! The extension chooses the parser, into the same serde types, so a pack's
//! meaning never depends on which syntax wrote it: TOML for what people author,
//! JSON for what a program records (mere's data formats brief, F1).

use std::path::Path;

use serde::de::DeserializeOwned;

/// Whether `path` names a pack data file: `.toml` or `.json`.
pub(crate) fn is_data(path: &Path) -> bool {
    path.extension()
        .is_some_and(|ext| ext == "toml" || ext == "json")
}

/// Reads and decodes one data file. The error is the reason alone; callers
/// attach the path and the refusal that fits them.
pub(crate) fn read<T: DeserializeOwned>(path: &Path) -> Result<T, ReadError> {
    let bytes = std::fs::read(path).map_err(|error| ReadError::Unreadable(error.to_string()))?;
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("toml") => {
            let text = std::str::from_utf8(&bytes)
                .map_err(|error| ReadError::Malformed(error.to_string()))?;
            toml::from_str(text).map_err(|error| ReadError::Malformed(error.to_string()))
        },
        Some("json") => {
            serde_json::from_slice(&bytes).map_err(|error| ReadError::Malformed(error.to_string()))
        },
        _ => Err(ReadError::Malformed(
            "not a pack data file: expected .toml or .json".to_string(),
        )),
    }
}

/// Why a data file did not read.
pub(crate) enum ReadError {
    /// The file could not be opened or read.
    Unreadable(String),
    /// It read, and is not the shape the schema declares in its syntax.
    Malformed(String),
}
