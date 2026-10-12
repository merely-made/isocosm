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
        let mode = match Mode::parse(&local.native.binding().mode) {
            Ok(mode @ (Mode::Mesocosm | Mode::Vtt)) => mode,
            _ => { local.scene.clear(); continue; },
        };
        if !mode.available() {
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
            app.world.read_game(mode, |session, bytes| {
                crate::spatial::region(session, &cut, window, site.0)?;
                let view = if cut.access.seat == Seat::Player {
                    isocosm::directing::readings::Mode::Survival
                } else { isocosm::directing::readings::Mode::Creative };
                #[cfg(feature = "vtt")]
                if mode == Mode::Vtt {
                    let explicit = local.input.as_ref().filter(|input| input.mode == Mode::Vtt)
                        .and_then(|input| input.region.as_ref());
                    return vtt_picture(session, &cut, window, site.0, defaults.quarter_turn,
                        bytes, explicit, &local.scene);
                }
                if mode != Mode::Mesocosm { return Err("mode has no native-body picture in this build".into()); }
                if bytes.is_empty() {
                    if cut.access.seat == Seat::Player {
                        return Err("Mesocosm close-up requires actual saved physical inputs".into());
                    }
                    // The unbound creative picture remains a native
                    // site-grain projection and drives no game geometry.
                    local.scene.update(session, cut.access.participant.0, window.0, view, site.0, defaults.quarter_turn)?;
                } else {
                    let game: mesocosm_runtime::spatial::Game = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
                    game.validate(session)?;
                    local.scene.update_spatial(session, cut.access.participant.0, window.0, view, site.0,
                        defaults.quarter_turn, &game)?;
                }
                Ok::<(), String>(())
            })?;
            Ok::<(), String>(())
        })();
        if let Err(why) = result { local.scene.clear(); local.error = Some(why); }
    }
}

/// Mode-owned admission authenticates coordinates against this exact Session.
/// Table owner/cell and presentation layouts never supply missing positions.
#[cfg(feature = "vtt")]
fn vtt_picture(session: &isocosm::Session, cut: &crate::window::Projection,
    window: isocosm_overlay::WindowHandle, site: u64, turn: u8, bytes: &[u8],
    explicit: Option<&crate::products::SiteRegion>, handle: &SceneHandle) -> Result<(), String> {
    let game = isometry_campaign::native::GameBindings::decode(bytes)?;
    game.validate(session)?;
    let mut roots = std::collections::BTreeMap::new();
    let mut windows = Vec::new();
    for (key, character) in game.characters() {
        if !cut.visible.shows(character.entity) { continue; }
        let held = session.sim.state().population.get(character.entity).ok_or("VTT picture character is absent")?;
        if held.resident_site() != Some(site) { continue; }
        let binding = game.positions().get(key).ok_or("visible VTT character has no admitted physical position")?;
        let at = binding.current(session)?;
        if !windows.contains(&binding.placement.window) { windows.push(binding.placement.window.clone()); }
        roots.insert(character.entity, mesocosm_runtime::spatial::Pose {
            // PositionBinding v2 carries no orientation. Its body document
            // is rendered unrotated; no table facing or inferred yaw is used.
            at, yaw:mesocosm_genet::borrowed_section::RootYaw::Zero,
        });
    }
    if roots.is_empty() { return Err("VTT close-up has no current admitted physical character positions".into()); }
    if cut.access.seat == Seat::Player
        && cut.access.subject.is_none_or(|subject| !roots.contains_key(&subject.0)) {
        return Err("VTT played subject has no current physical position in this close-up".into());
    }
    let frame = match explicit {
        Some(region) if region.site == site => isocosm::operations::Window {
            min:region.min, max:region.max, depth:region.depth },
        Some(_) => return Err("VTT picture region names another native site".into()),
        None if windows.len() == 1 => {
            let region = &windows[0];
            isocosm::operations::Window { min:region.min, max:region.max, depth:region.depth }
        },
        None => return Err("VTT positions have different windows; author a --site-region for this picture".into()),
    };
    let mode = if cut.access.seat == Seat::Player { isocosm::directing::readings::Mode::Survival }
        else { isocosm::directing::readings::Mode::Creative };
    handle.update_roots(session, cut.access.participant.0, window.0, mode, site, turn, &frame, &roots)?;
    Ok(())
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
