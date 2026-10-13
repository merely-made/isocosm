// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Current presented-frame topics become native nudges, never physical driving.

use super::Host;
use isocosm::{schema::Id, directing::readings::Mode};
use isocosm_overlay::{ActKey, EntityHandle, PlaceHandle, mesocosm::{MesocosmIntent, Nudge, NudgeMeaning, NudgeTarget}};
use mesocosm_views::Succession;
use winit::{event::MouseButton, keyboard::{Key, NamedKey}};

#[derive(Clone, PartialEq, Eq)]
struct Source { state: u64, spatial: String, mode: Mode }
#[derive(Clone, PartialEq, Eq)]
struct Hint { process: String, topic: NudgeTarget }
struct Menu { actor: Id, clicked: NudgeTarget, hints: Vec<Hint>, selected: Option<Hint> }

#[derive(Default)]
pub(super) struct Pointer {
    at: Option<[f32; 2]>, presented: Option<Source>, menu: Option<Menu>, notice: Option<String>,
}

fn topic(target: NudgeTarget) -> String {
    match target { NudgeTarget::Thing(id) => format!("body {}", id.0), NudgeTarget::Place(id) => format!("site {}", id.0) }
}
fn contains([x,y,w,h]: [f32;4], at: [f32;2]) -> bool {
    at[0] >= x && at[0] < x+w && at[1] >= y && at[1] < y+h
}

impl Host {
    fn pointer_source(&self) -> Source {
        Source { state:self.runtime.state_hash(), spatial:isocosm::digest(self.runtime.spatial()), mode:self.runtime.interim().mode }
    }
    pub(super) fn mark_pointer_frame(&mut self) {
        self.pointer.presented = self.gpu.as_ref().and_then(|gpu| gpu.section.query_generation())
            .filter(|_| self.scene.is_some()).map(|_| self.pointer_source());
    }
    pub(super) fn move_pointer(&mut self, at: [f32;2]) {
        self.pointer.at = at.iter().all(|v| v.is_finite()).then_some(at);
    }
    pub(super) fn clear_pointer(&mut self) {
        self.pointer.at = None; self.pointer.presented = None; self.pointer.menu = None;
    }
    pub(super) fn directing_open(&self) -> bool { self.pointer.menu.is_some() }
    fn pointer_topic(&mut self) -> Result<NudgeTarget,String> {
        if self.pointer.presented.as_ref() != Some(&self.pointer_source()) {
            return Err("the pointed frame is no longer current".into());
        }
        let at = self.pointer.at.ok_or("pointer has no current window position")?;
        let visible = self.runtime.physical_view()?;
        let scene = self.scene.as_ref().ok_or("physical scene is unavailable")?;
        let gpu = self.gpu.as_mut().ok_or("scene has no presented device")?;
        if at[0] < 0.0 || at[1] < 0.0 || at[0] >= gpu.config.width as f32 || at[1] >= gpu.config.height as f32 {
            return Err("pointer is outside the presented frame".into());
        }
        match gpu.section.pick_at_pixel([at[0].floor() as u32, at[1].floor() as u32]).map_err(|e| format!("pick: {e:?}"))? {
            Some(isometer::Pick::Body(pick)) => {
                let id = pick.address.subject.0;
                let body = scene.body(id).ok_or("pointed body is not in the current picture")?;
                let native = self.runtime.sim().state().population.get(id).ok_or("pointed native body disappeared")?;
                if !visible.shows(id) || native.resident_site() != Some(scene.site) || native.body.as_ref() != Some(&body.document)
                    || native.alive != body.alive || !gpu.section.validate_body_pick(pick, scene) {
                    return Err("pointed body identity, geometry or awareness is stale".into());
                }
                if body.physical.is_some() {
                    self.runtime.spatial().members.get(&id).ok_or("pointed body has no physical source")?.realized(self.runtime.sim())?;
                }
                Ok(NudgeTarget::Thing(EntityHandle(id)))
            },
            Some(isometer::Pick::Terrain(_)) if visible.sites.contains(&scene.site) =>
                Ok(NudgeTarget::Place(PlaceHandle(scene.site))),
            _ => Err("the current view supplies no pointed body or site".into()),
        }
    }
    fn over_chrome(&self, at: [f32;2]) -> bool {
        let Some(gpu) = &self.gpu else { return true; };
        let Some(lanes) = &gpu.chrome else { return false; };
        let frame = (gpu.config.width,gpu.config.height);
        let hud = crate::hud::placement(frame);
        contains([hud.0,hud.1,hud.2,hud.3],at) || contains(lanes.vitals.probe(frame).1,at)
            || (self.config.dev && contains(lanes.dev.probe(frame).1,at))
            || (lanes.board.standing() && contains(lanes.board.probe(frame).1,at))
            || (lanes.checkpoint.standing() && contains(lanes.checkpoint.probe(frame).1,at))
            || (lanes.directing.standing() && contains(lanes.directing.probe(frame).1,at))
    }
    pub(super) fn press_pointer(&mut self, button: MouseButton) {
        let Some(at) = self.pointer.at else { return; };
        if self.press_continuation_pointer(button,at) || self.runtime.checkpoint().is_some() { return; }
        let on_menu = self.gpu.as_ref().and_then(|gpu| gpu.chrome.as_ref().map(|lanes|
            lanes.directing.standing() && contains(lanes.directing.probe((gpu.config.width,gpu.config.height)).1,at))).unwrap_or(false);
        if button == MouseButton::Left && on_menu { self.send_selected_hint(); return; }
        if self.over_chrome(at) { return; }
        let result = self.pointer_topic();
        match (button,result) {
            (MouseButton::Left,Ok(target)) => { self.pointer.menu = None; self.send_hint(target,NudgeMeaning::Attend); },
            (MouseButton::Right,Ok(target)) => self.open_directing(target),
            (MouseButton::Left | MouseButton::Right,Err(why)) => { self.pointer.notice = Some(why.clone()); eprintln!("attention: {why}"); },
            _ => {},
        }
    }
    fn send_hint(&mut self, target: NudgeTarget, meaning: NudgeMeaning) {
        let Some(actor) = self.runtime.critter() else { return; };
        if !crate::input::admits(self.runtime.queued_len()) { return; }
        let intent = MesocosmIntent::Nudge(Nudge { entity:EntityHandle(actor),target,meaning });
        self.runtime.queue(crate::input::envelope(&self.runtime,intent));
    }
    fn open_directing(&mut self, clicked: NudgeTarget) {
        if self.gpu.as_ref().and_then(|gpu| gpu.chrome.as_ref()).is_none() { return; }
        let Some(actor) = self.runtime.critter() else { return; };
        self.pointer.notice = None;
        self.pointer.menu = Some(Menu {actor,clicked,hints:Vec::new(),selected:None});
        self.refresh_directing();
    }
    pub(super) fn refresh_directing(&mut self) {
        let Some(mut menu) = self.pointer.menu.take() else { return; };
        if self.continuation_open() || self.runtime.checkpoint().is_some() || self.runtime.critter() != Some(menu.actor) { return; }
        let Ok(visible) = self.runtime.physical_view() else { return; };
        let Some(site) = self.runtime.played().and_then(|body| body.resident_site()) else { return; };
        let clicked_visible = match menu.clicked { NudgeTarget::Thing(id) => visible.shows(id.0), NudgeTarget::Place(id) => visible.sites.contains(&id.0) };
        if !clicked_visible { return; }
        menu.hints = visible.filter(isocosm::directing::readings::suggestions(self.runtime.sim(),menu.actor)).into_iter()
            .filter_map(|hint| {
                let topic = match hint.target { Some(id) => NudgeTarget::Thing(EntityHandle(id)),
                    None if visible.sites.contains(&site) => NudgeTarget::Place(PlaceHandle(site)), None => return None };
                Some(Hint {process:hint.process,topic})
            }).collect();
        if menu.selected.as_ref().is_some_and(|selected| !menu.hints.contains(selected)) {
            menu.selected = None; self.pointer.notice = Some("That suggestion is no longer current. Choose again.".into());
        }
        self.pointer.menu = Some(menu);
    }
    fn send_selected_hint(&mut self) {
        self.refresh_directing();
        let Some(menu) = &self.pointer.menu else { return; };
        let Some(selected) = menu.selected.clone() else { self.pointer.notice = Some("Choose a current suggestion first.".into()); return; };
        self.send_hint(selected.topic,NudgeMeaning::Act(ActKey(selected.process)));
        self.pointer.menu = None;
    }
    pub(super) fn try_directing_key(&mut self, key: &Key) -> bool {
        if self.continuation_open() { return false; }
        if matches!(key,Key::Character(c) if c.eq_ignore_ascii_case("q")) && self.runtime.checkpoint().is_none() {
            let Some(site) = self.runtime.played().and_then(|body| body.resident_site()) else { return true; };
            self.open_directing(NudgeTarget::Place(PlaceHandle(site))); return true;
        }
        if self.pointer.menu.is_none() { return false; }
        match key {
            Key::Named(NamedKey::Escape) => { self.pointer.menu = None; },
            Key::Named(NamedKey::Enter) => self.send_selected_hint(),
            Key::Named(NamedKey::ArrowUp | NamedKey::ArrowDown) => {
                self.refresh_directing();
                if let Some(menu) = &mut self.pointer.menu {
                    let n = menu.hints.len();
                    if n != 0 {
                        let current = menu.selected.as_ref().and_then(|selected| menu.hints.iter().position(|hint| hint == selected));
                        let next = if matches!(key,Key::Named(NamedKey::ArrowUp)) { current.map_or(n-1,|i| (i+n-1)%n) }
                            else { current.map_or(0,|i| (i+1)%n) };
                        menu.selected = Some(menu.hints[next].clone()); self.pointer.notice = None;
                    }
                }
            },
            _ => return false,
        }
        true
    }
    pub(super) fn directing_reading(&self) -> Option<Succession> {
        let menu = self.pointer.menu.as_ref()?;
        let selected = menu.selected.as_ref();
        let name = selected.map_or("No act selected".into(),|hint| hint.process.clone());
        let actual_topic = selected.map_or("Choose with Up/Down".into(),|hint| topic(hint.topic));
        Some(Succession { headline:"Choose a hint".into(), facts:vec![
            ("clicked".into(),topic(menu.clicked)),("act".into(),name),("hint topic".into(),actual_topic),
            ("choices".into(),self.pointer.notice.clone().unwrap_or_else(|| format!("{} current suggestions; Up/Down, Escape closes",menu.hints.len())))],
            stay:"send the selected hint; the critter chooses its own act".into(),take:None })
    }
}
