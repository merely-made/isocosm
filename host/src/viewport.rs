// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! One product picture per local window, using native admission and the real producer registry.

use std::collections::{BTreeMap,BTreeSet};
use cambium_rootstock::ProducerRegistry;
use isocosm_overlay::Seat;
#[cfg(any(feature = "mesocosm", feature = "vtt"))]
use mesocosm_genet::borrowed_section::{SceneHandle, producer::SectionProducer};
use crate::{products::Mode, ui::model::{App, Defaults}};
#[cfg(feature = "eponym")]
mod eponym;

pub(crate) fn key(local: u64) -> Result<u64, String> {
    if local >= 1 << 61 { return Err("local viewport identity exhausted".into()); }
    Ok((1 << 61) + local)
}

pub(crate) fn refresh(app: &mut App) {
    #[cfg(feature = "eponym")]
    eponym::refresh(app);
    #[cfg(any(feature = "mesocosm", feature = "vtt"))]
    refresh_section(app);
}

#[cfg(any(feature = "mesocosm", feature = "vtt"))]
fn refresh_section(app: &mut App) {
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
                    // The physical consumer applies marked voxel sight from
                    // this same Session and its validated saved eye/root cells.
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
    let mode = if cut.access.seat == Seat::Player { isocosm::directing::readings::Mode::Survival }
        else { isocosm::directing::readings::Mode::Creative };
    let sight = if mode == isocosm::directing::readings::Mode::Survival {
        let observer = cut.access.subject.ok_or("VTT physical sight has no played native body")?.0;
        if session.sim.plays(cut.access.participant.0) != Some(observer) {
            return Err("VTT physical sight names a stale native played body".into());
        }
        isometry_campaign::native::eye::refine(session,&game,observer,mode,site)?
    } else { None };
    let shows = |id| cut.visible.shows(id) && sight.as_ref().is_none_or(|view| view.shows(id));
    let mut roots = std::collections::BTreeMap::new();
    let mut windows = Vec::new();
    for (key, character) in game.characters() {
        if !shows(character.entity) { continue; }
        let held = session.sim.state().population.get(character.entity).ok_or("VTT picture character is absent")?;
        if held.resident_site() != Some(site) { continue; }
        let binding = game.positions().get(key).ok_or("visible VTT character has no admitted physical position")?;
        let pose = binding.current_pose(session)?;
        if !windows.contains(&binding.placement.window) { windows.push(binding.placement.window.clone()); }
        roots.insert(character.entity, mesocosm_runtime::spatial::Pose {
            at:pose.at, yaw:pose.yaw,
        });
    }
    let mut item_entities = BTreeSet::new();
    for binding in game.item_records().iter().rev().filter_map(|record| record.binding.as_ref()) {
        if !item_entities.insert(binding.entity) || !shows(binding.entity) { continue; }
        let held = session.sim.state().population.get(binding.entity).ok_or("VTT picture item is absent")?;
        if held.resident_site() != Some(site) { continue; }
        let pose = isometry_campaign::native::items::current_item(session, &game, binding.entity)?;
        let at = pose.at;
        if roots.get(&binding.entity).is_some_and(|existing|
            existing.at != at || existing.yaw != pose.yaw) {
            return Err("VTT item and character bindings disagree on the same native root pose".into());
        }
        if !windows.contains(&pose.window) { windows.push(pose.window.clone()); }
        roots.entry(binding.entity).or_insert(mesocosm_runtime::spatial::Pose {
            at, yaw: pose.yaw,
        });
    }
    if roots.is_empty() { return Err("VTT close-up has no current admitted physical body positions".into()); }
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
    handle.update_roots(session, cut.access.participant.0, window.0, mode, site, turn, &frame, &roots)?;
    Ok(())
}

pub(crate) fn available(local:&crate::ui::model::Local)->bool {picture_mode(local).is_some()}

fn picture_mode(local:&crate::ui::model::Local)->Option<Mode> {
    let mode=Mode::parse(&local.native.binding().mode).ok()?;
    #[cfg(feature = "eponym")]
    if mode==Mode::Eponym && local.eponym_scene.available() {return Some(mode);}
    #[cfg(any(feature = "mesocosm", feature = "vtt"))]
    if matches!(mode,Mode::Mesocosm|Mode::Vtt) && local.scene.snapshot().is_some() {return Some(mode);}
    None
}

fn clear(local:&crate::ui::model::Local) {
    #[cfg(feature = "eponym")]
    local.eponym_scene.clear();
    #[cfg(any(feature = "mesocosm", feature = "vtt"))]
    local.scene.clear();
}

fn error(local:&crate::ui::model::Local,mode:Mode)->Option<String> {
    #[cfg(feature = "eponym")]
    if mode==Mode::Eponym {return local.eponym_scene.last_error();}
    #[cfg(any(feature = "mesocosm", feature = "vtt"))]
    if matches!(mode,Mode::Mesocosm|Mode::Vtt) {return local.scene.last_error();}
    None
}

fn register(local:&crate::ui::model::Local,mode:Mode,key:u64,registry:&mut ProducerRegistry)->Result<(),String> {
    #[cfg(feature = "eponym")]
    if mode==Mode::Eponym {
        return registry.register(key,eponym_client::borrowed::Producer::new(local.eponym_scene.clone()),&[])
            .map_err(|e|format!("Eponym viewport registration: {e:?}"));
    }
    #[cfg(any(feature = "mesocosm", feature = "vtt"))]
    if matches!(mode,Mode::Mesocosm|Mode::Vtt) {
        return registry.register(key,SectionProducer::new(local.scene.clone()),&[])
            .map_err(|e|format!("viewport registration: {e:?}"));
    }
    Err("mode has no physical producer in this build".into())
}

#[derive(Default)]
pub(crate) struct Registrations { keys: BTreeMap<u64,Mode> }
impl Registrations {
    /// Remove revoked/closed/mode-switched pictures before the host relayout and paint.
    /// Only registrations owned by this bridge are retired.
    pub fn sync(&mut self, app: &App, registry: &mut ProducerRegistry) -> Vec<(u64, String)> {
        let wanted:BTreeMap<_,_>=app.windows.iter().filter_map(|(local_key,local)|
            Some((key(*local_key).ok()?,picture_mode(local)?))).collect();
        let retired:Vec<_>=self.keys.iter().filter(|(key,mode)|wanted.get(*key)!=Some(*mode)).map(|(key,_)|*key).collect();
        for key in retired { registry.remove(key); self.keys.remove(&key); }
        let mut errors = Vec::new();
        for (&local_key, local) in &app.windows {
            let Some(mode) = picture_mode(local) else { continue; };
            let Ok(key) = key(local_key) else { continue; };
            if !registry.contains(key) {
                match register(local,mode,key,registry) {
                    Ok(()) => { self.keys.insert(key,mode); },
                    Err(why) => { clear(local); errors.push((local_key,why)); },
                }
            } else if !self.keys.contains_key(&key) {
                clear(local); errors.push((local_key, "viewport key belongs to another producer".into()));
            }
            if let Some(why) = error(local,mode) {
                if local.error.as_deref() != Some(&why) { errors.push((local_key, why)); }
            }
        }
        errors
    }
}

#[cfg(any(feature = "mesocosm", feature = "vtt"))]
pub(crate) fn handle() -> SceneHandle { SceneHandle::default() }
