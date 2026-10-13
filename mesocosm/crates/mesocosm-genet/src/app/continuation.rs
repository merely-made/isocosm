// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Explicit choices at the current saved native/product loop cut.

use super::Host;
use cambium::{DetailRow, DetailSection, detail_panel, el, text};
use isocosm_overlay::{ParticipantHandle, Tick};
use mesocosm_runtime::continuation::{self, LoopAction, LoopAnswer, LoopInput,
    LoopPhase, LoopReading};
use mesocosm_views::{Succession, SuccessionChild};
use winit::{event::MouseButton, keyboard::{Key, NamedKey}};

#[derive(Default)]
pub(super) struct Pane {
    reading: Option<LoopReading>, presented: Option<LoopReading>,
    selected: Option<LoopAnswer>, notice: Option<String>,
}

fn active(phase: LoopPhase) -> bool {
    matches!(phase, LoopPhase::LineageEnded | LoopPhase::Watching | LoopPhase::NewWorldRequested)
}

fn choices(reading: &LoopReading) -> Vec<LoopAnswer> {
    if reading.phase == LoopPhase::NewWorldRequested { return vec![LoopAnswer::Watch]; }
    if !matches!(reading.phase, LoopPhase::LineageEnded | LoopPhase::Watching) {
        return Vec::new();
    }
    let mut choices: Vec<_> = reading.lineages.iter().copied().map(LoopAnswer::OtherLineage).collect();
    if reading.phase == LoopPhase::LineageEnded { choices.push(LoopAnswer::Watch); }
    choices.push(LoopAnswer::NewWorld);
    choices
}

pub(super) fn root(state: &Succession) -> SuccessionChild {
    let rows = state.facts.iter().map(|(key, value)| DetailRow::new(key.clone(), value.clone())).collect();
    let mut children: Vec<SuccessionChild> = vec![
        Box::new(el::<_, Succession, ()>("div", text(state.headline.clone())).attr("class", "checkpoint-headline")),
        Box::new(detail_panel::<Succession, ()>(&[DetailSection::new("continue", rows)])),
        Box::new(el::<_, Succession, ()>("div", text(state.stay.clone())).attr("class", "checkpoint-answer")),
    ];
    if let Some(take) = &state.take {
        children.push(Box::new(el::<_, Succession, ()>("div", text(take.clone())).attr("class", "checkpoint-answer")));
    }
    Box::new(el::<_, Succession, ()>("div", children).attr("class", "checkpoint"))
}

impl Host {
    pub(super) fn continuation_open(&self) -> bool {
        self.runtime.loop_reading().is_ok_and(|reading| active(reading.phase))
    }

    pub(super) fn refresh_continuation(&mut self) {
        match self.runtime.loop_reading() {
            Ok(reading) if active(reading.phase) => {
                if self.continuation.selected.as_ref().is_some_and(|selected| !choices(&reading).contains(selected)) {
                    self.continuation.selected = None;
                    self.continuation.notice = Some("That choice is no longer offered. Choose again.".into());
                }
                self.continuation.reading = Some(reading);
            },
            Ok(_) => self.continuation = Pane::default(),
            Err(why) => {
                self.continuation.reading = None;
                self.continuation.presented = None;
                self.continuation.selected = None;
                self.continuation.notice = Some(why);
            },
        }
    }

    pub(super) fn mark_continuation_frame(&mut self) {
        self.continuation.presented = self.gpu.as_ref().and_then(|gpu| gpu.chrome.as_ref())
            .filter(|lanes| lanes.continuation.standing()).and(self.continuation.reading.clone());
    }

    fn continuation_label(&self, answer: &LoopAnswer) -> String {
        match answer {
            LoopAnswer::OtherLineage(id) => self.runtime.sim().state().population.get(id.0)
                .map_or_else(|| "Unavailable body".into(), |body| format!("body {}, {}", id.0, body.lineage)),
            LoopAnswer::Watch => "Watch this world".into(),
            LoopAnswer::NewWorld => "Request a new world".into(),
            LoopAnswer::Checkpoint(_) => "Checkpoint answer".into(),
        }
    }

    pub(super) fn continuation_reading(&self) -> Option<Succession> {
        let reading = self.continuation.reading.as_ref()?;
        if reading.phase == LoopPhase::NewWorldRequested {
            return Some(Succession { headline: "New world requested".into(), facts: vec![
                ("world".into(), "This world remains owned and can be resumed.".into()),
                ("save on quit".into(), self.config.trace.as_ref().map_or("No trace path configured.".into(), |path| path.display().to_string())),
                ("choice".into(), self.continuation.notice.clone().unwrap_or_else(||
                    self.continuation.selected.as_ref().map_or("Nothing selected".into(), |answer| self.continuation_label(answer)))),
            ], stay: "[W] keep watching; Up/Down then Enter / click also selects Watch".into(),
                take: Some("[Esc] quit; --watch PATH resumes this save. Start a separate run for a new world.".into()) });
        }
        let selected = self.continuation.selected.as_ref().map_or("Nothing selected".into(), |answer| self.continuation_label(answer));
        Some(Succession {
            headline: if reading.phase == LoopPhase::Watching { "Watching this world" } else { "Your lineage has ended" }.into(),
            facts: vec![("choose".into(), selected),
                ("other bodies".into(), format!("{} offered in the current native view", reading.lineages.len())),
                ("status".into(), self.continuation.notice.clone().unwrap_or_else(||
                    if reading.phase == LoopPhase::Watching { "The native world keeps running.".into() }
                    else { "The world waits for your choice.".into() }))],
            stay: "[Up/Down / Tab] select; [Enter / click panel] apply".into(),
            take: Some(if reading.phase == LoopPhase::LineageEnded { "[W] watch; [N] request new world" }
                else { "[N] request new world" }.into()),
        })
    }

    fn submit_continuation(&mut self, answer: LoopAnswer) {
        let result = (|| {
            let shown = self.continuation.presented.as_ref().ok_or("The continuation pane has not been presented.")?;
            let current = self.runtime.loop_reading()?;
            if shown.cut != current.cut || shown.phase != current.phase {
                return Err("The displayed loop cut changed. Read the refreshed choices.".into());
            }
            if !choices(&current).contains(&answer) { return Err("That choice is not currently offered.".into()); }
            let input = LoopInput { kind: continuation::KIND.into(), version: continuation::VERSION,
                participant: ParticipantHandle(self.runtime.interim().participant), tick: Tick(self.runtime.tick()),
                expected: shown.cut.clone(), action: LoopAction::Answer(answer) };
            self.runtime.submit_loop_input(&input).map_err(|why| format!("{why:?}"))
        })();
        match result {
            Ok(note) => {
                self.events.push(note);
                self.continuation.selected = None;
                self.continuation.notice = None;
                self.continuation.presented = None;
                self.last = None;
            },
            Err(why) => { eprintln!("continuation: {why}"); self.continuation.notice = Some(why); },
        }
        self.refresh_continuation();
    }

    fn submit_selected_continuation(&mut self) {
        match self.continuation.selected.clone() {
            Some(answer) => self.submit_continuation(answer),
            None => self.continuation.notice = Some("Choose an offered option first.".into()),
        }
    }

    pub(super) fn try_continuation_key(&mut self, key: &Key) -> bool {
        if !self.continuation_open() { return false; }
        match key {
            Key::Named(NamedKey::ArrowUp | NamedKey::ArrowDown | NamedKey::Tab) => {
                let Some(reading) = &self.continuation.presented else { return true; };
                let choices = choices(reading);
                if !choices.is_empty() {
                    let at = self.continuation.selected.as_ref().and_then(|answer| choices.iter().position(|choice| choice == answer));
                    let next = if matches!(key, Key::Named(NamedKey::ArrowUp)) { at.map_or(choices.len()-1, |i| (i+choices.len()-1)%choices.len()) }
                        else { at.map_or(0, |i| (i+1)%choices.len()) };
                    self.continuation.selected = Some(choices[next].clone()); self.continuation.notice = None;
                }
            },
            Key::Named(NamedKey::Enter) => self.submit_selected_continuation(),
            Key::Character(c) if c.eq_ignore_ascii_case("w") => self.submit_continuation(LoopAnswer::Watch),
            Key::Character(c) if c.eq_ignore_ascii_case("n") => self.submit_continuation(LoopAnswer::NewWorld),
            _ => return false,
        }
        true
    }

    pub(super) fn press_continuation_pointer(&mut self, button: MouseButton, at: [f32; 2]) -> bool {
        if !self.continuation_open() { return false; }
        let on_panel = self.gpu.as_ref().and_then(|gpu| gpu.chrome.as_ref().map(|lanes| {
            let [x,y,w,h] = lanes.continuation.probe((gpu.config.width,gpu.config.height)).1;
            lanes.continuation.standing() && at[0] >= x && at[0] < x+w && at[1] >= y && at[1] < y+h
        })).unwrap_or(false);
        if button == MouseButton::Left && on_panel { self.submit_selected_continuation(); }
        true
    }
}
