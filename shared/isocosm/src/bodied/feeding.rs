// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A new founding migrates a recipe's port and its meal together. Saved
//! unmarked definitions and bare ecology never pass through this migration.

use crate::{Result, process::{IntakePort, NisKind, Process as Native, Registry}, rules::*, schema::Key};
use std::collections::BTreeMap;

pub(super) fn declare(
    kinds: &mut BTreeMap<Key, Template>, recipe: &mut Recipe, lineage: &str,
) -> Result<Key> {
    let head = recipe.tagmata.first_mut().ok_or("a feeding recipe has no root")?;
    let (original, mouth) = match &head.bears {
        Some(kind) if head.socket == Facing::Below => (kind.clone(), true),
        _ => (head.segment.clone(), false),
    };
    let mut kind = kinds.get(&original).ok_or("feeding kind is absent")?.clone();
    let native = if !mouth || kind.cells.get("function:intake").is_some_and(|n| *n > 0) {
        Native::Intake
    } else if kind.cells.get("function:contract").is_some_and(|n| *n > 0) {
        Native::Contract
    } else { return Err("mouth has no supported native feeding function".into()); };
    let port = match (mouth, native) {
        (false, _) => IntakePort::deadstock(),
        (true, Native::Intake) => IntakePort::live(NisKind::Producer),
        (true, _) => IntakePort::live(NisKind::Consumer),
    }.supported_by(Registry::native().of_native(native).reference());
    let function = crate::mosaic::ports::supporting(port).ok_or("unknown port support")?;
    // Distinct mouth kinds do not turn every limb or mass segment into an
    // intake organ. Geometry and the exact cell allocation are unchanged.
    kind.port = Some(port);
    let key = format!("{original}-port-{}", lineage.trim_start_matches("lineage:"));
    kinds.insert(key.clone(), kind);
    if mouth { head.bears = Some(key); } else { head.segment = key; }
    Ok(function)
}
