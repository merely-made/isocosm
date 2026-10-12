// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! One product picture per local window, using native admission and the real producer registry.

use std::collections::BTreeSet;
use cambium_rootstock::ProducerRegistry;
use isocosm_overlay::Seat;
use mesocosm_genet::borrowed_section::{SceneHandle, producer::SectionProducer};
use crate::{products::Mode, ui::model::{App, Defaults}};

pub(crate) fn key(local: u64) -> Result<u64, String> {
    if local >= 1 << 61 { return Err("local viewport identity exhausted".into()); }
    Ok((1 << 61) + local)
}

pub(crate) fn refresh(app: &mut App) {
    for local in app.windows.values_mut() {
        if local.native.binding().mode != Mode::Mesocosm.key() {
            local.scene.clear(); continue;
        }
        let result = (|| {
            let Some(site) = local.native.binding().attention.examined else {
                local.scene.clear(); return Ok(());
            };
            let cut = local.native.projection()?;
            let window = local.native.handle()?;
            let defaults: Defaults = serde_json::from_slice(&local.native.binding().view_defaults)
                .map_err(|e| e.to_string())?;
            app.world.read_game(Mode::Mesocosm, |session, bytes| {
                crate::spatial::region(session, &cut, window, site.0)?;
                let mode = if cut.access.seat == Seat::Player {
                    isocosm::directing::readings::Mode::Survival
                } else { isocosm::directing::readings::Mode::Creative };
                if bytes.is_empty() {
                    if cut.access.seat == Seat::Player {
                        return Err("Mesocosm close-up requires actual saved physical inputs".into());
                    }
                    // The unbound creative picture remains a native
                    // site-grain projection and drives no game geometry.
                    local.scene.update(session, cut.access.participant.0, window.0, mode, site.0, defaults.quarter_turn)?;
                } else {
                    let game: mesocosm_runtime::spatial::Game = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
                    local.scene.update_spatial(session, cut.access.participant.0, window.0, mode, site.0,
                        defaults.quarter_turn, &game)?;
                }
                Ok::<(), String>(())
            })?;
            Ok::<(), String>(())
        })();
        if let Err(why) = result { local.scene.clear(); local.error = Some(why); }
    }
}

#[derive(Default)]
pub(crate) struct Registrations { keys: BTreeSet<u64> }
impl Registrations {
    /// Remove revoked/closed/mode-switched pictures before the host relayout and paint.
    /// Only registrations owned by this bridge are retired.
    pub fn sync(&mut self, app: &App, registry: &mut ProducerRegistry) -> Vec<(u64, String)> {
        let wanted: BTreeSet<_> = app.windows.iter().filter(|(_, local)| local.scene.snapshot().is_some())
            .filter_map(|(local,_)| key(*local).ok()).collect();
        let retired: Vec<_> = self.keys.difference(&wanted).copied().collect();
        for key in retired { registry.remove(key); self.keys.remove(&key); }
        let mut errors = Vec::new();
        for (&local_key, local) in &app.windows {
            let Some(_) = local.scene.snapshot() else { continue; };
            let Ok(key) = key(local_key) else { continue; };
            if !registry.contains(key) {
                match registry.register(key, SectionProducer::new(local.scene.clone()), &[]) {
                    Ok(()) => { self.keys.insert(key); },
                    Err(why) => { local.scene.clear(); errors.push((local_key, format!("viewport registration: {why:?}"))); },
                }
            } else if !self.keys.contains(&key) {
                local.scene.clear(); errors.push((local_key, "viewport key belongs to another producer".into()));
            }
            if let Some(why) = local.scene.last_error() {
                if local.error.as_deref() != Some(&why) { errors.push((local_key, why)); }
            }
        }
        errors
    }
}

pub(crate) fn handle() -> SceneHandle { SceneHandle::default() }
