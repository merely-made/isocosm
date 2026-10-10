// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A line's program, read from the log (legacy `program`): every committed
//! `Command::Revise` in order, append-only because the log is. A refused
//! command is never logged, so each entry here was committed. A speciated
//! line's program begins with its parent's up to the split, as legacy
//! forks inherited it.

use crate::{
    Session,
    history::Command,
    rules::Development,
    schema::{Key, Tick},
};
use serde::{Deserialize, Serialize};

/// One committed revision.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Revision {
    /// The line that committed it, itself or a forebear.
    pub lineage: Key,
    pub tick: Tick,
    /// What the line develops from after it.
    pub development: Development,
    /// Over the line and the development: two worlds holding the same
    /// revision agree on it.
    pub digest: String,
}

/// The revisions `lineage` is born under, oldest first: each forebear's
/// until the next line in its ancestry split off, then its own.
pub fn program(session: &Session, lineage: &str) -> Vec<Revision> {
    let mut line = super::tree::ancestry(&session.sim.state().lineages, lineage);
    line.reverse();
    let mut at = 0;
    let mut out = vec![];
    for entry in &session.entries {
        match &entry.command {
            Command::Speciate { name, .. } => {
                let made = super::speciate::key(name);
                if let Some(j) = line.iter().position(|k| *k == made) {
                    at = at.max(j);
                }
            },
            Command::Revise {
                lineage: by,
                development,
            } if line.get(at) == Some(by) => out.push(Revision {
                lineage: by.clone(),
                tick: entry.tick,
                development: development.clone(),
                digest: crate::digest(&(by, development)),
            }),
            _ => {},
        }
    }
    out
}

/// The program's identity; `None` for a line born under its founding.
pub fn digest(revisions: &[Revision]) -> Option<String> {
    let all: Vec<&str> = revisions.iter().map(|r| r.digest.as_str()).collect();
    (!all.is_empty()).then(|| crate::digest(&all))
}
