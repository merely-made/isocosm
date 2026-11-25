// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Requests about a picked body or site. Cosmograph supplies the creature's acts.

use super::Host;
use cosmograph::schema::Id;
use cosmograph_overlay::{ActKey, EntityHandle, PlaceHandle, mesocosm::{MesocosmIntent, Nudge, NudgeMeaning, NudgeTarget}};
use mesocosm_views::Succession;
use winit::keyboard::{Key, NamedKey};

#[derive(Clone, PartialEq, Eq)]
struct Hint { process: String, topic: NudgeTarget }

const REQUESTS: [(&str, Option<NudgeMeaning>); 5] = [
    ("Move toward", Some(NudgeMeaning::MoveToward)),
    ("Move away", Some(NudgeMeaning::MoveAway)),
    ("Stay near", Some(NudgeMeaning::StayNear)),
    ("Point out", Some(NudgeMeaning::Attend)),
    ("Suggest an act", None),
];

enum Page { Requests(usize), Acts { hints: Vec<Hint>, selected: Option<Hint> } }
struct Menu { actor: Id, clicked: NudgeTarget, revision: Option<u64>, page: Page }
#[derive(Default)]
pub(super) struct Pointer { menu: Option<Menu>, notice: Option<String> }

fn topic(target: NudgeTarget) -> String {
    match target { NudgeTarget::Thing(id) => format!("body {}", id.0), NudgeTarget::Place(id) => format!("site {}", id.0) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HostConfig;
    use cosmograph::directing::{Aim, Toward};

    fn host() -> Host {
        Host::new(HostConfig { seed: 18_181_373_014_846_284_518, play: true, ..Default::default() })
    }

    #[test]
    fn the_menu_sends_each_request_about_the_picked_topic() {
        for (index, aim) in [Aim::MoveToward, Aim::MoveAway, Aim::StayNear, Aim::Attend].into_iter().enumerate() {
            let mut host = host();
            let actor = host.runtime.critter().unwrap();
            host.open_directing_in_view(NudgeTarget::Thing(EntityHandle(actor)));
            assert_eq!(host.directing_reading().unwrap().headline, "Direct creature");
            for _ in 0..index { host.try_directing_key(&Key::Named(NamedKey::ArrowDown)); }
            host.try_directing_key(&Key::Named(NamedKey::Enter));
            assert!(!host.directing_open());
            host.runtime.step(1);
            let sim = host.runtime.sim();
            let request = sim.state().nudges.last().expect("the host submitted a native request");
            assert_eq!((request.aim, request.toward, request.critter), (aim, Toward::Thing(actor), actor));
        }
    }

    #[test]
    fn opening_acts_reads_the_native_alternatives_without_issuing_a_command() {
        let mut host = host();
        host.try_directing_key(&Key::Character("q".into()));
        host.try_directing_key(&Key::Named(NamedKey::ArrowUp));
        host.try_directing_key(&Key::Named(NamedKey::Enter));
        let menu = host.pointer.menu.as_ref().unwrap();
        let Page::Acts { hints, .. } = &menu.page else { panic!("expected act suggestions") };
        let native = host.runtime.physical_view().unwrap().filter(
            cosmograph::directing::readings::suggestions(&host.runtime.sim(), menu.actor));
        assert_eq!(hints.iter().map(|h| &h.process).collect::<Vec<_>>(),
            native.iter().map(|h| &h.process).collect::<Vec<_>>());
        assert!(host.runtime.sim().state().nudges.is_empty());
        assert_eq!(host.directing_reading().unwrap().headline, "Suggest an act");
    }
}
fn cycle(current: Option<usize>, count: usize, up: bool) -> usize {
    if up { current.map_or(count - 1, |i| (i + count - 1) % count) }
    else { current.map_or(0, |i| (i + 1) % count) }
}

impl Host {
    pub(super) fn directing_open(&self) -> bool { self.pointer.menu.is_some() }
    pub(super) fn open_directing_in_view(&mut self, clicked: NudgeTarget) {
        let Some(actor) = self.runtime.critter() else { return; };
        self.pointer.notice = None;
        self.pointer.menu = Some(Menu { actor, clicked, revision: None, page: Page::Requests(0) });
        self.refresh_directing();
    }
    pub(super) fn refresh_directing(&mut self) {
        let Some(mut menu) = self.pointer.menu.take() else { return; };
        if self.continuation_open() || self.runtime.checkpoint().is_some() || self.runtime.critter() != Some(menu.actor) { return; }
        let Ok(revision) = self.runtime.revision() else { return; };
        if menu.revision == Some(revision) {
            self.pointer.menu = Some(menu);
            return;
        }
        let Ok(visible) = self.runtime.physical_view() else { return; };
        let Some(site) = self.runtime.played().and_then(|body| body.resident_site()) else { return; };
        let visible_topic = match menu.clicked {
            NudgeTarget::Thing(id) => visible.shows(id.0),
            NudgeTarget::Place(id) => visible.sites.contains(&id.0),
        };
        if !visible_topic { return; }
        if let Page::Acts { hints, selected } = &mut menu.page {
            *hints = visible.filter(cosmograph::directing::readings::suggestions(&self.runtime.sim(), menu.actor))
                .into_iter().filter_map(|hint| {
                    let topic = match hint.target {
                        Some(id) => NudgeTarget::Thing(EntityHandle(id)),
                        None if visible.sites.contains(&site) => NudgeTarget::Place(PlaceHandle(site)),
                        None => return None,
                    };
                    Some(Hint { process: hint.process, topic })
                }).collect();
            if selected.as_ref().is_some_and(|hint| !hints.contains(hint)) {
                *selected = None;
                self.pointer.notice = Some("That act or its target has changed. Choose again.".into());
            }
        }
        menu.revision = Some(revision);
        self.pointer.menu = Some(menu);
    }

    fn confirm_request(&mut self) {
        self.refresh_directing();
        let Some(menu) = &mut self.pointer.menu else { return; };
        let (target, meaning) = match &menu.page {
            Page::Requests(index) => match REQUESTS[*index].1.clone() {
                Some(meaning) => (menu.clicked, meaning),
                None => {
                    menu.page = Page::Acts { hints: Vec::new(), selected: None };
                    menu.revision = None;
                    self.refresh_directing();
                    return;
                }
            },
            Page::Acts { selected: Some(hint), .. } => {
                let act = ActKey(hint.process.clone());
                (hint.topic, match hint.topic {
                    NudgeTarget::Thing(_) => NudgeMeaning::Act(act),
                    NudgeTarget::Place(_) => NudgeMeaning::ActHere(act),
                })
            }
            Page::Acts { selected: None, .. } => return,
        };
        let intent = MesocosmIntent::Nudge(Nudge { entity: EntityHandle(menu.actor), target, meaning });
        match self.runtime.offer(intent) {
            Ok(_) => self.pointer.menu = None,
            Err(why) => self.pointer.notice = Some(format!("Request refused: {why:?}")),
        }
    }

    pub(super) fn try_directing_key(&mut self, key: &Key) -> bool {
        if self.continuation_open() { return false; }
        if matches!(key,Key::Character(c) if c.eq_ignore_ascii_case("q")) && self.runtime.checkpoint().is_none() {
            let Some(site) = self.runtime.played().and_then(|body| body.resident_site()) else { return true; };
            self.open_directing_in_view(NudgeTarget::Place(PlaceHandle(site)));
            return true;
        }
        if self.pointer.menu.is_none() { return false; }
        match key {
            Key::Named(NamedKey::Escape) => self.pointer.menu = None,
            Key::Named(NamedKey::Enter) => self.confirm_request(),
            Key::Named(NamedKey::ArrowUp | NamedKey::ArrowDown) => {
                self.refresh_directing();
                let up = matches!(key, Key::Named(NamedKey::ArrowUp));
                match self.pointer.menu.as_mut().map(|m| &mut m.page) {
                    Some(Page::Requests(index)) => *index = cycle(Some(*index), REQUESTS.len(), up),
                    Some(Page::Acts { hints, selected }) if !hints.is_empty() => {
                        let current = selected.as_ref().and_then(|hint| hints.iter().position(|h| h == hint));
                        *selected = Some(hints[cycle(current, hints.len(), up)].clone());
                    }
                    _ => {}
                }
                self.pointer.notice = None;
            }
            _ => return false,
        }
        true
    }

    pub(super) fn directing_reading(&self) -> Option<Succession> {
        let menu = self.pointer.menu.as_ref()?;
        let (headline, mut facts, stay) = match &menu.page {
            Page::Requests(index) => {
                let participant = self.runtime.interim().participant;
                let sim = self.runtime.sim();
                let seed = sim.player_seed(participant).ok()?;
                let bond = sim.bond(participant, menu.actor)?;
                let facts = vec![
                    ("Creature".into(), format!("body {}", menu.actor)),
                    ("Target".into(), topic(menu.clicked)),
                    ("Player".into(), format!("{seed:016x} · bond {bond}/{}", sim.genesis().rules.directing_ref().most)),
                    ("Request".into(), REQUESTS[*index].0.into()),
                ];
                ("Direct creature", facts, if REQUESTS[*index].1.is_none() { "see available acts" } else { "send request" })
            }
            Page::Acts { hints, selected } => {
                let label = selected.as_ref().map_or_else(|| if hints.is_empty() {
                    "No alternative act available".into()
                } else { format!("{} available", hints.len()) }, |hint| hint.process.clone());
                let mut facts = vec![("Act".into(), label)];
                if let Some(hint) = selected {
                    let label = match hint.topic { NudgeTarget::Thing(_) => "Target", NudgeTarget::Place(_) => "Act at" };
                    facts.push((label.into(), topic(hint.topic)));
                }
                ("Suggest an act", facts, "suggest selected act")
            }
        };
        facts.push(("Controls".into(), self.pointer.notice.clone().unwrap_or_else(|| "Up/Down to choose · Esc to close".into())));
        Some(Succession { headline: headline.into(), facts, stay: stay.into(), take: None })
    }
}
