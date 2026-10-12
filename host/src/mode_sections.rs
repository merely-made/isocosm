// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The mode adapter's local view memory, beside opaque product-owned bytes.
//! Parsing is confined to an available, requested mode. Unknown versions or
//! fields refuse; absent modes remain byte-exact. No section grants a seat,
//! participant, played body, native attention, or replay authority.

use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use isocosm_overlay::{AttentionSet, ParticipantHandle, WindowHandle};
use crate::{products::Mode, save::WorldSave, ui::model::{Defaults, Local, Preferences}, world::World};

const VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ViewMemory {
    participant: u64,
    defaults: Defaults,
    pinned: std::collections::BTreeSet<isocosm_overlay::Pointable>,
    examined: Option<isocosm_overlay::PlaceHandle>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Section {
    version: u32,
    windows: BTreeMap<u64, ViewMemory>,
    // Each actual game adapter owns this payload and its migration. The
    // host view-memory update carries it unread and unchanged.
    game: Vec<u8>,
}

impl Section {
    fn empty() -> Self { Self { version: VERSION, windows: BTreeMap::new(), game: Vec::new() } }
    fn decode(bytes: Option<&Vec<u8>>) -> Result<Self, String> {
        let value: Self = match bytes {
            Some(bytes) => serde_json::from_slice(bytes).map_err(|e| format!("mode section: {e}"))?,
            None => Self::empty(),
        };
        if value.version != VERSION { return Err("unsupported available-mode section version".into()); }
        if value.windows.values().any(|row| row.defaults.quarter_turn > 3) {
            return Err("invalid saved mode camera defaults".into());
        }
        Ok(value)
    }
}

pub(crate) fn recalled(world: &World, mode: Mode, participant: ParticipantHandle, window: WindowHandle)
    -> Result<Option<(Defaults, AttentionSet)>, String> {
    if !mode.available() { return Err("mode is absent from this build".into()); }
    // The exact native token must already belong to this participant. Its
    // current native profile may differ from this inactive mode's memory.
    world.read(|s| s.sim.attention_window(participant.0, window.0))??;
    let save = world.save()?;
    let section = Section::decode(save.sections.get(mode.key()))?;
    let Some(row) = section.windows.get(&window.0) else { return Ok(None); };
    if row.participant != participant.0 { return Err("mode memory belongs to another participant".into()); }
    Ok(Some((row.defaults.clone(), AttentionSet { pinned: row.pinned.clone(), examined: row.examined,
        ..Default::default() })))
}

pub(crate) fn preferences(world: &World, mode: Mode, participant: ParticipantHandle,
    window: Option<WindowHandle>, explicit: Option<Preferences>) -> Result<Preferences, String> {
    if let Some(value) = explicit { return Ok(value); }
    let mut value = Preferences::default();
    if let Some(window) = window {
        if let Some((defaults, _)) = recalled(world, mode, participant, window)? {
            value.modes.insert(mode.key().into(), defaults);
        }
    }
    Ok(value)
}

/// Build a save snapshot without mutating the Session, sections, or local
/// windows. A late section refusal cannot publish any earlier replacement.
pub(crate) fn capture(mut save: WorldSave, locals: &BTreeMap<u64, Local>) -> Result<WorldSave, String> {
    let mut sections = save.sections.clone();
    for mode in Mode::enabled() {
        let rows: Vec<_> = locals.values().filter(|local| local.native.binding().mode == mode.key()
            || local.preferences.modes.contains_key(mode.key()) || local.attention.contains_key(mode.key())).collect();
        if rows.is_empty() { continue; }
        let mut section = Section::decode(sections.get(mode.key()))?;
        for local in rows {
            let handle = local.native.handle()?;
            let access = local.native.projection()?.access;
            let current = local.native.binding().mode == mode.key();
            let defaults = if current {
                serde_json::from_slice(&local.native.binding().view_defaults).map_err(|e| e.to_string())?
            } else { local.preferences.modes.get(mode.key()).cloned().unwrap_or_default() };
            let attention = if current { local.native.binding().attention.clone() }
                else { local.attention.get(mode.key()).cloned().unwrap_or_default() };
            section.windows.insert(handle.0, ViewMemory { participant: access.participant.0, defaults,
                pinned: attention.pinned.clone(), examined: attention.examined });
        }
        sections.insert(mode.key().into(), serde_json::to_vec(&section).map_err(|e| e.to_string())?);
    }
    save.sections = sections;
    Ok(save)
}


/// Available-mode product data; other mode bytes stay unread.
pub(crate) fn game(sections: &BTreeMap<String, Vec<u8>>, mode: Mode) -> Result<Vec<u8>, String> {
    if !mode.available() { return Err("mode is absent from this build".into()); }
    Ok(Section::decode(sections.get(mode.key()))?.game)
}

pub(crate) fn with_game(
    sections: &BTreeMap<String, Vec<u8>>, mode: Mode, game: Vec<u8>,
) -> Result<BTreeMap<String, Vec<u8>>, String> {
    if !mode.available() { return Err("mode is absent from this build".into()); }
    let mut section = Section::decode(sections.get(mode.key()))?;
    section.game = game;
    let mut next = sections.clone();
    next.insert(mode.key().into(), serde_json::to_vec(&section).map_err(|e| e.to_string())?);
    Ok(next)
}
