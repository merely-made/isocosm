// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What `remember` took, and the four verbs that read it back.
//!
//! A checkpoint is one snapshot's fields under a name. The lane owns them; a
//! [`Product`](crate::Product) is handed this read-only view alongside the
//! scenario line, so a product verb can compare against the same checkpoints
//! the shared verbs use instead of keeping a second set of its own.

use std::collections::BTreeMap;

use genet_probe::ProbeSnapshot;

/// One `remember`ed snapshot's fields.
pub type Checkpoint = BTreeMap<String, String>;

/// A read-only view of every checkpoint `remember` has taken so far.
#[derive(Clone, Copy)]
pub struct Checkpoints<'a>(pub(crate) &'a BTreeMap<String, Checkpoint>);

impl<'a> Checkpoints<'a> {
    /// The fields one checkpoint holds, if it was taken.
    pub fn get(&self, name: &str) -> Option<&'a Checkpoint> {
        self.0.get(name)
    }

    /// One field of one checkpoint.
    pub fn field(&self, name: &str, field: &str) -> Option<&'a str> {
        self.0.get(name)?.get(field).map(String::as_str)
    }

    /// Every checkpoint name, in the order the receipt reports them.
    pub fn names(&self) -> impl Iterator<Item = &'a str> {
        self.0.keys().map(String::as_str)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The `same`/`differs`/`more`/`dropped` comparison, against `now`.
    ///
    /// `same` asks every named field to equal the checkpoint's, `differs` asks
    /// every one to have moved, `more` reads both as counters and asks the
    /// current one to be larger, and `dropped` reads both as numbers and asks
    /// the current one to be strictly smaller. The error names the verb, the
    /// checkpoint, the field and both values, so a miss is attributable
    /// without rerunning.
    pub fn compare(
        &self,
        verb: &str,
        name: &str,
        fields: &[&str],
        now: &ProbeSnapshot,
    ) -> Result<(), String> {
        let before = self
            .get(name)
            .ok_or_else(|| format!("unknown checkpoint {name}"))?;
        for field in fields {
            let first = before
                .get(*field)
                .ok_or_else(|| format!("checkpoint {name} has no {field}"))?;
            let current = now
                .field(field)
                .ok_or_else(|| format!("no snapshot field {field}"))?;
            let held = match verb {
                "same" => first == current,
                "differs" => first != current,
                "more" => {
                    let counter = |value: &str, tense: &str| {
                        value
                            .parse::<u64>()
                            .map_err(|_| format!("{field} {tense} a counter"))
                    };
                    counter(current, "is not")? > counter(first, "was not")?
                },
                "dropped" => {
                    let number = |value: &str, tense: &str| {
                        value
                            .parse::<f64>()
                            .map_err(|_| format!("{field} {tense} a number"))
                    };
                    number(current, "is not")? < number(first, "was not")?
                },
                _ => return Err(format!("unknown checkpoint verb {verb}")),
            };
            if !held {
                return Err(format!("{verb} {name} {field}: {first} -> {current}"));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "checkpoints/tests.rs"]
mod tests;
