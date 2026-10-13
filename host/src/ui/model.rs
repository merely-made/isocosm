// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use std::{collections::BTreeMap, path::PathBuf};
use isocosm_overlay::{AttentionChange, AttentionSet, EntityHandle, EventHandle, ParticipantHandle,
    PlaceHandle, Pointable, Seat, WindowHandle, key_for_handle, key_handle};
use serde::{Deserialize, Serialize};
use crate::{binding::Binding, products::{self, Input, Mode, Rows}, window::Window, world::World};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct Defaults {
    pub quarter_turn: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub party: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inspection: Option<super::inspection::Selection>,
    #[cfg(feature = "vtt")]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub overmap: Option<isometry_campaign::native::overmap::ViewSettings>,
}
impl Default for Defaults { fn default() -> Self { Self { quarter_turn: 0, party: None, inspection: None,
    #[cfg(feature = "vtt")]
    overmap: None,
} } }

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub(crate) struct Preferences { pub modes: BTreeMap<String, Defaults> }

pub(crate) struct PinChoice {
    pub id: String, pub label: String, pub point: Pointable, pub pinned: bool,
}

pub(crate) struct Local {
    #[cfg(feature = "vtt")]
    pub hook_note:cambium::TextInput,
    #[cfg(feature = "vtt")]
    pub authored_input:cambium::TextInput,
    #[cfg(feature = "vtt")]
    pub author_quote:Option<String>,
    pub native: Window, pub rows: Rows, pub error: Option<String>,
    pub preferences: Preferences, pub attention: BTreeMap<String, AttentionSet>,
    pub input: Option<Input>, pub close_requested: bool,
    #[cfg(any(feature = "mesocosm", feature = "vtt"))]
    pub scene: mesocosm_genet::borrowed_section::SceneHandle,
    #[cfg(feature = "eponym")]
    pub eponym_scene: eponym_client::borrowed::SceneHandle,
}

pub(crate) struct App {
    pub world: World, pub windows: BTreeMap<u64, Local>,
    pub next: u64, pub pending_open: Vec<u64>, pub save_path: Option<PathBuf>,
}

impl App {
    pub fn new(world: World, save_path: Option<PathBuf>) -> Self {
        Self { world, windows: BTreeMap::new(), next: 0, pending_open: Vec::new(), save_path }
    }
    pub fn open(&mut self, participant: ParticipantHandle, seat: Seat, mode: Mode,
        preferences: Preferences, restore: Option<WindowHandle>) -> Result<u64, String> {
        self.open_with_saved_preferences(participant, seat, mode, Some(preferences), restore)
    }
    pub fn open_with_saved_preferences(&mut self, participant: ParticipantHandle, seat: Seat, mode: Mode,
        preferences: Option<Preferences>, restore: Option<WindowHandle>) -> Result<u64, String> {
        let preferences = crate::mode_sections::preferences(&self.world, mode, participant, restore, preferences)?;
        let key = self.next;
        #[cfg(any(feature = "mesocosm", feature = "vtt", feature = "eponym"))]
        crate::viewport::key(key)?;
        let next = key.checked_add(1).ok_or("local window identity exhausted")?;
        let mut attention = AttentionSet::default();
        if let Some(handle) = restore {
            attention = self.world.read(|session| crate::native_attention::restored(session, participant, handle, mode.key()))??;
        }
        let binding = binding(mode, &preferences, attention)?;
        let native = match restore {
            Some(handle) => Window::restore(&self.world, participant, seat, handle, binding)?,
            None => Window::attach(&self.world, participant, seat, binding)?,
        };
        self.next = next;
        self.windows.insert(key, Local { native, rows: Rows::default(), error: None, preferences,
            #[cfg(feature = "vtt")]
            hook_note:Default::default(),
            #[cfg(feature = "vtt")]
            authored_input:Default::default(),
            #[cfg(feature = "vtt")]
            author_quote:None,
            attention: BTreeMap::new(), input: None, close_requested: false,
            #[cfg(any(feature = "mesocosm", feature = "vtt"))]
            scene: crate::viewport::handle(),
            #[cfg(feature = "eponym")]
            eponym_scene: eponym_client::borrowed::SceneHandle::default(), });
        self.refresh();
        Ok(key)
    }
    pub fn switch(&mut self, key: u64, mode: Mode) -> Result<(), String> {
        let local = self.windows.get_mut(&key).ok_or("window is closed")?;
        let previous = local.native.binding().clone();
        if previous.mode == mode.key() { return Ok(()); }
        let access = local.native.projection()?.access;
        let remembered = crate::mode_sections::recalled(&self.world, mode, access.participant, local.native.handle()?)?;
        let mut preferences = local.preferences.clone();
        if let Some((defaults, _)) = &remembered {
            preferences.modes.entry(mode.key().into()).or_insert_with(|| defaults.clone());
        }
        let attention = local.attention.get(mode.key()).cloned()
            .or_else(|| remembered.map(|(_, attention)| attention)).unwrap_or_default();
        let requested = binding(mode, &preferences, attention)?;
        let prepared = local.native.prepare_commit(requested)?;
        local.native.commit_binding(prepared)?;
        local.attention.insert(previous.mode, previous.attention);
        local.preferences = preferences;
        self.refresh();
        Ok(())
    }
    pub fn turn(&mut self, key: u64) -> Result<(), String> {
        let local = self.windows.get_mut(&key).ok_or("window is closed")?;
        let mut defaults: Defaults = serde_json::from_slice(&local.native.binding().view_defaults)
            .map_err(|e| e.to_string())?;
        defaults.quarter_turn = (defaults.quarter_turn + 1) % 4;
        let mode = local.native.binding().mode.clone();
        local.native.set_view_defaults(serde_json::to_vec(&defaults).map_err(|e| e.to_string())?)?;
        local.preferences.modes.insert(mode, defaults);
        self.refresh();
        Ok(())
    }
    pub fn examine(&mut self, key: u64, site: Option<u64>) -> Result<(), String> {
        let local = self.windows.get_mut(&key).ok_or("window is closed")?;
        let cut = local.native.projection()?;
        if site.is_some_and(|s| !cut.visible.sites.contains(&s)) { return Err("region is outside the seat cut".into()); }
        let change = site.map_or(AttentionChange::StopExamining, |s| AttentionChange::Examine(PlaceHandle(s)));
        let request = local.native.window_attention_request(change)?;
        local.native.apply_attention(request)?;
        self.refresh();
        Ok(())
    }
    pub fn pin_choices(&self, key: u64) -> Result<Vec<PinChoice>, String> {
        let local = self.windows.get(&key).ok_or("window is closed")?;
        let cut = local.native.projection()?;
        self.world.read(|session| -> Result<Vec<PinChoice>, String> {
            let mut choices = Vec::new();
            let mut push = |id: String, label: String, point: Pointable| {
                choices.push(PinChoice { id, label, point,
                    pinned: local.native.binding().attention.pinned.contains(&point) });
            };
            for id in cut.visible.things.keys() {
                push(format!("pin-entity-{id}"), format!("Entity {id}"), Pointable::Entity(EntityHandle(*id)));
            }
            for id in &cut.visible.sites {
                push(format!("pin-site-{id}"), format!("Site {id}"), Pointable::Place(PlaceHandle(*id)));
            }
            for record in crate::stream::records(session, &cut)? {
                let event: isocosm::schema::Event = serde_json::from_slice(&record.payload)
                    .map_err(|e| e.to_string())?;
                let handle = key_handle(&event.id);
                if key_for_handle(session.sim.state().events.keys().map(String::as_str), handle)
                    != Some(event.id.as_str()) { continue; }
                push(format!("pin-event-{handle}"), format!("Event {}", event.id),
                    Pointable::Event(EventHandle(handle)));
            }
            Ok(choices)
        })?
    }
    pub fn pin(&mut self, key: u64, point: Pointable) -> Result<(), String> {
        let local = self.windows.get_mut(&key).ok_or("window is closed")?;
        let mut cut = local.native.projection()?;
        let pinned = local.native.binding().attention.pinned.contains(&point);
        if !pinned {
            cut.attention.pinned.insert(point);
            self.world.read(|session| crate::access::filter_attention(session, cut.access,
                &cut.visible, &mut cut.attention))?;
            if !cut.attention.pinned.contains(&point) { return Err("pin is outside the seat cut".into()); }
        }
        let change = if pinned { AttentionChange::Unpin(point) } else { AttentionChange::Pin(point) };
        let request = local.native.window_attention_request(change)?;
        local.native.apply_attention(request)?;
        self.refresh();
        Ok(())
    }
    pub fn send(&mut self, key: u64) -> Result<String, String> {
        let local = self.windows.get_mut(&key).ok_or("window is closed")?;
        let input = local.input.as_ref().ok_or("choose a product intent file first")?;
        let accepted = local.native.submit_product(input)?;
        self.refresh();
        Ok(accepted)
    }
    pub fn step(&mut self, key: u64) -> Result<(), String> {
        let local = self.windows.get(&key).ok_or("window is closed")?;
        if local.native.binding().mode == "vtt" {
            return Err("choose a ready downtime proposal in the table's time controls".into());
        }
        self.world.advance(1)?; self.refresh(); Ok(())
    }
    pub fn close(&mut self, key: u64) -> Result<(), String> {
        let local = self.windows.get_mut(&key).ok_or("window is closed")?;
        local.native.close()?;
        self.windows.remove(&key);
        self.refresh();
        Ok(())
    }
    pub fn duplicate(&mut self, key: u64) -> Result<u64, String> {
        let local = self.windows.get(&key).ok_or("window is closed")?;
        let access = local.native.projection()?.access;
        let mode = Mode::parse(&local.native.binding().mode)?;
        let preferences = local.preferences.clone();
        let input = local.input.clone();
        let new = self.open(access.participant, access.seat, mode, preferences, None)?;
        self.windows.get_mut(&new).ok_or("opened window is missing")?.input = input;
        self.pending_open.push(new);
        Ok(new)
    }
    pub fn snapshot(&self) -> Result<crate::save::WorldSave, String> {
        crate::mode_sections::capture(self.world.save()?, &self.windows)
    }
    pub fn save(&self) -> Result<(), String> {
        let path = self.save_path.as_ref().ok_or("choose a world save path at launch")?;
        let bytes = self.snapshot()?.encode()?;
        crate::file_save::replace(path, &bytes).map_err(|e| e.to_string())
    }
    pub fn refresh(&mut self) {
        for local in self.windows.values_mut() {
            let result = local.native.projection().and_then(|cut| {
                let mode = Mode::parse(&local.native.binding().mode)?;
                self.world.read_game(mode, |session, game| products::read_game(session, &cut, mode, game))
            });
            match result {
                Ok(rows) => { local.rows = rows; local.error = None; },
                Err(why) => { local.rows = Rows::default(); local.error = Some(why); },
            }
        }
        self.refresh_viewport();
    }
    pub fn refresh_viewport(&mut self) {
        #[cfg(any(feature = "mesocosm", feature = "vtt", feature = "eponym"))]
        crate::viewport::refresh(self);
    }
    pub fn record(&mut self, key: u64, result: Result<(), String>) {
        if let Some(local) = self.windows.get_mut(&key) { local.error = result.err(); }
    }
}

fn binding(mode: Mode, preferences: &Preferences, attention: AttentionSet) -> Result<Binding, String> {
    let defaults = preferences.modes.get(mode.key()).cloned().unwrap_or_default();
    if defaults.quarter_turn > 3 { return Err("quarter turn must be zero through three".into()); }
    mode.binding(serde_json::to_vec(&defaults).map_err(|e| e.to_string())?, attention)
}
