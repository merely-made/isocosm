// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Print the same paired observations that the native specimen bench displays.
use cosmograph_overlay::mesocosm::NudgeMeaning;
use mesocosm_runtime::refuge::{MeadowChange, compare_request_at};

fn main() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let request = match args.next().as_deref() {
        Some("point" | "nudge") => Some(NudgeMeaning::Attend),
        Some("toward") => Some(NudgeMeaning::MoveToward),
        Some("away") => Some(NudgeMeaning::MoveAway),
        Some("near") => Some(NudgeMeaning::StayNear),
        Some("alone") => None,
        _ => return Err("choose toward, away, near, point, or alone".into()),
    };
    let when = match args.next().as_deref() {
        None | Some("open") => MeadowChange::Reopens,
        Some("lean") => MeadowChange::TurnsLean,
        _ => return Err("choose open or lean".into()),
    };
    if args.next().is_some() {
        return Err("expected toward|away|near|point|alone [open|lean]".into());
    }
    let mut observations = Vec::new();
    compare_request_at(when, request, |pair| {
        observations.push(pair);
        true
    })?;
    serde_json::to_writer(std::io::stdout().lock(), &observations).map_err(|why| why.to_string())
}
