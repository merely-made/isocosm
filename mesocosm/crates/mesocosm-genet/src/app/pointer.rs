// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Native suggestions for the current rendered topic. Leaf picking is in Assembly.

use super::Host;
use cosmograph::schema::Id;
use cosmograph_overlay::{ActKey, EntityHandle, PlaceHandle, mesocosm::{MesocosmIntent, Nudge, NudgeMeaning, NudgeTarget}};
use mesocosm_views::Succession;
use winit::keyboard::{Key, NamedKey};

#[derive(Clone, PartialEq, Eq)]
struct Hint { process: String, topic: NudgeTarget }
struct Menu { actor: Id, clicked: NudgeTarget, hints: Vec<Hint>, selected: Option<Hint> }
#[derive(Default)]
pub(super) struct Pointer { menu: Option<Menu>, notice: Option<String> }
fn topic(target: NudgeTarget) -> String {
    match target { NudgeTarget::Thing(id) => format!("body {}", id.0), NudgeTarget::Place(id) => format!("site {}", id.0) }
}
impl Host {
    pub(super) fn directing_open(&self) -> bool { self.pointer.menu.is_some() }
    fn send_hint(&mut self, target: NudgeTarget, meaning: NudgeMeaning) {
        let Some(actor) = self.runtime.critter() else { return; };
        let intent = MesocosmIntent::Nudge(Nudge { entity:EntityHandle(actor),target,meaning });
        if let Err(why) = self.runtime.offer(intent) { eprintln!("input: {why:?}"); }
    }
    pub(super) fn open_directing_in_view(&mut self, clicked: NudgeTarget) {
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
        menu.hints = visible.filter(cosmograph::directing::readings::suggestions(&self.runtime.sim(),menu.actor)).into_iter()
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
            self.open_directing_in_view(NudgeTarget::Place(PlaceHandle(site))); return true;
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
