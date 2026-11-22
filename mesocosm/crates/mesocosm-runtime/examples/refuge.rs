// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Print the same paired observations that the native specimen bench displays.
use mesocosm_runtime::refuge::{MeadowChange, compare_at};

fn main() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let nudge = match args.next().as_deref() {
        Some("nudge") => true,
        Some("alone") => false,
        _ => return Err("choose nudge or alone".into()),
    };
    let when = match args.next().as_deref() {
        None | Some("open") => MeadowChange::Reopens,
        Some("lean") => MeadowChange::TurnsLean,
        _ => return Err("choose open or lean".into()),
    };
    if args.next().is_some() {
        return Err("expected nudge|alone [open|lean]".into());
    }
    let mut observations = Vec::new();
    compare_at(when, nudge, |pair| {
        observations.push(pair);
        true
    })?;
    serde_json::to_writer(std::io::stdout().lock(), &observations).map_err(|why| why.to_string())
}
