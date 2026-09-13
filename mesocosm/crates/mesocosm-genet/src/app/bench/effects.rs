// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A decorative plane experiment. Never advances or adjudicates the world.
use super::{state::Bench, view::Child};
use cambium::{clickable, custom_leaf, el, focusable, text};
use mesocosm_core::effect_experiment::{Behavior, Experiment, Glyph, Profile, Receiver, Request};
use serde::{Deserialize, Serialize};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};
mod paint;
pub(super) const KEY: u64 = 0x4546_4645;

pub(super) struct Effects {
    pub open: bool,
    pub playing: bool,
    pub tick: u16,
    pub experiment: Experiment,
    pub notice: String,
    pub saved: Option<PathBuf>,
    last: Instant,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Saved {
    request: Request,
    tick: u16,
}
impl Default for Effects {
    fn default() -> Self {
        Self {
            open: false,
            playing: false,
            tick: 0,
            experiment: Request::default().prepare().expect("default effect"),
            notice: String::new(),
            saved: None,
            last: Instant::now(),
        }
    }
}
impl Effects {
    pub fn load(path: &std::path::Path) -> Result<Self, String> {
        let saved: Saved = serde_json::from_slice(&std::fs::read(path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
        if saved.tick > saved.request.duration_ticks {
            return Err("Saved effect tick exceeds duration.".into());
        }
        Ok(Self {
            open: true,
            tick: saved.tick,
            experiment: saved.request.prepare()?,
            saved: Some(path.to_owned()),
            notice: "Saved experiment restored.".into(),
            ..Self::default()
        })
    }

    fn change(&mut self, edit: impl FnOnce(&mut Request)) {
        let mut request = self.experiment.request.clone();
        edit(&mut request);
        match request.prepare() {
            Ok(experiment) => {
                self.experiment = experiment;
                self.tick = 0;
                self.playing = false;
                self.notice.clear();
            },
            Err(why) => self.notice = why,
        }
    }
    pub fn advance(&mut self) {
        if self.open && self.playing && self.last.elapsed() >= Duration::from_millis(33) {
            let elapsed = (self.last.elapsed().as_millis() / 33).min(120) as u16;
            self.tick = self
                .tick
                .saturating_add(elapsed)
                .min(self.experiment.request.duration_ticks);
            self.last += Duration::from_millis(u64::from(elapsed) * 33);
            if self.tick == self.experiment.request.duration_ticks {
                self.playing = false;
            }
        }
    }
    fn step(&mut self) {
        self.tick = self
            .tick
            .saturating_add(10)
            .min(self.experiment.request.duration_ticks);
        if self.tick == self.experiment.request.duration_ticks {
            self.playing = false;
        }
    }
    pub fn sync(&self, leaves: &mut sprigging::LeafRegistry<u64>) {
        if !self.open {
            leaves.remove(&KEY);
            return;
        }
        if !leaves.contains(&KEY) {
            leaves.insert(KEY, Box::new(paint::EffectLeaf::default()));
        }
        leaves
            .get_mut_as::<paint::EffectLeaf>(&KEY)
            .unwrap()
            .set(&self.experiment, self.tick);
    }
    pub fn probe_fields(&self) -> Vec<(&'static str, String)> {
        let r = &self.experiment.request;
        let report = &self.experiment.report;
        vec![
            ("effects-open", self.open.to_string()),
            ("effects-playing", self.playing.to_string()),
            ("effects-tick", self.tick.to_string()),
            ("effects-world-seed", r.world_seed.to_string()),
            ("effects-appearance-seed", r.appearance_seed.to_string()),
            ("effects-glyph", r.glyph.label().into()),
            ("effects-behavior", r.behavior.label().into()),
            ("effects-receiver", r.receiver.label().into()),
            ("effects-profile", r.profile.label().into()),
            ("effects-response", report.response.label().into()),
            ("effects-connection", report.connection.label().into()),
            ("effects-origin", report.origin.label().into()),
            (
                "effects-marks",
                self.experiment.sample(self.tick).len().to_string(),
            ),
            ("effects-request", serde_json::to_string(r).unwrap()),
            (
                "effects-sample",
                serde_json::to_string(&self.experiment.sample(self.tick)).unwrap(),
            ),
            (
                "effects-saved",
                self.saved
                    .as_ref()
                    .map(|p| p.display().to_string())
                    .unwrap_or_default(),
            ),
            ("effects-notice", self.notice.clone()),
        ]
    }
}
impl Bench {
    fn save_effect(&mut self) {
        let result = (|| -> Result<PathBuf, String> {
            let bytes = serde_json::to_vec_pretty(&Saved {
                request: self.effects.experiment.request.clone(),
                tick: self.effects.tick,
            })
            .map_err(|e| e.to_string())?;
            let mut file = tempfile::Builder::new()
                .prefix("effect-experiment-")
                .suffix(".json")
                .tempfile_in(&self.export_directory)
                .map_err(|e| e.to_string())?;
            std::io::Write::write_all(&mut file, &bytes).map_err(|e| e.to_string())?;
            file.keep().map(|(_, path)| path).map_err(|e| e.to_string())
        })();
        match result {
            Ok(path) => {
                self.effects.notice = format!("Saved {}", path.display());
                self.effects.saved = Some(path);
            },
            Err(why) => self.effects.notice = why,
        }
    }
    fn reopen_effect(&mut self) {
        let result = self
            .effects
            .saved
            .as_ref()
            .ok_or_else(|| "Save an experiment first.".to_owned())
            .and_then(|path| Effects::load(path));
        match result {
            Ok(effects) => self.effects = effects,
            Err(why) => self.effects.notice = why,
        }
    }
}
fn button(label: impl Into<String>, action: impl Fn(&mut Bench) + 'static) -> Child {
    let label = label.into();
    Box::new(focusable(clickable(
        el("button", text(label.clone())).attr("aria-label", label),
        move |s: &mut Bench, _| action(s),
    )))
}
fn choices(state: &Bench) -> Vec<Child> {
    let mut items = Vec::new();
    for &glyph in Glyph::ALL {
        items.push(button(format!("Glyph {}", glyph.label()), move |s| {
            s.effects.change(|r| r.glyph = glyph)
        }));
    }
    for &behavior in Behavior::ALL {
        items.push(button(format!("Behavior {}", behavior.label()), move |s| {
            s.effects.change(|r| r.behavior = behavior)
        }));
    }
    for &receiver in Receiver::ALL {
        items.push(button(format!("Receiver {}", receiver.label()), move |s| {
            s.effects.change(|r| r.receiver = receiver)
        }));
    }
    for &profile in Profile::ALL {
        items.push(button(format!("Rules {}", profile.label()), move |s| {
            s.effects.change(|r| r.profile = profile)
        }));
    }
    let _ = state;
    items
}
pub(super) fn view(state: &Bench) -> Child {
    if !state.effects.open {
        return Box::new(el("div", ()));
    }
    let e = &state.effects;
    let r = &e.experiment.request;
    let report = &e.experiment.report;
    let controls = vec![
        button("Close effects experiment", |s| {
            s.effects.open = false;
            s.effects.playing = false;
        }),
        button("Reseed interactions", |s| {
            s.effects
                .change(|r| r.world_seed = r.world_seed.wrapping_add(1))
        }),
        button("Reseed appearance", |s| {
            s.effects
                .change(|r| r.appearance_seed = r.appearance_seed.wrapping_add(1))
        }),
        button("Play effects", |s| {
            if s.effects.tick == s.effects.experiment.request.duration_ticks {
                s.effects.tick = 0;
            }
            s.effects.playing = true;
            s.effects.last = Instant::now();
        }),
        button("Pause effects", |s| s.effects.playing = false),
        button("Step effects", |s| {
            s.effects.playing = false;
            s.effects.step();
        }),
        button("Reset effects", |s| {
            s.effects.playing = false;
            s.effects.tick = 0;
        }),
        button("Save experiment", Bench::save_effect),
        button("Reopen experiment", Bench::reopen_effect),
    ];
    Box::new(el("section",(
        el("h2",text("Glyph effects / 2D experiment")),
        el("p",text("A decorative plane for comparing rules and movement. The specimen world remains unchanged.")),
        el("div",choices(state)).attr("class","toolbar"),
        el("p",text(format!("{} / {} / {} / {} | World seed {} / Appearance seed {} | Tick {} / {}",r.glyph.label(),r.behavior.label(),r.receiver.label(),r.profile.label(),r.world_seed,r.appearance_seed,e.tick,r.duration_ticks))),
        custom_leaf::<Bench,()>(KEY,800,280).attr("id","effect-viewport").attr("class","effect-viewport").attr("role","img").attr("aria-label","Glyph effect experiment"),
        el("div",controls).attr("class","toolbar"),
        el("p",text(format!("{} / {} / {}. {}",report.response.label(),report.connection.label(),report.origin.label(),report.explanation))).attr("id","effect-report"),
        el("p",text(e.notice.clone())).attr("role","status"),
    )).attr("class","effect-panel"))
}
