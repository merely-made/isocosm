// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The window's foreground (E10 step 7, D50, D51). In a world founded with
//! the foreground rule the frame loop drives the pacer: elapsed time becomes
//! world minutes for the runtime and, between them, a step of the minute's
//! stepper walks (`cosmograph_host::drive::Walk`) that the scene draws. The
//! walks are the ones the clock's handbacks come from
//! (`world_foreground_walks`), so what is drawn at the minute's last step is
//! what is handed back. Presentation only: nothing here reaches the log.

use std::rc::Rc;
use std::sync::{Arc, OnceLock};
use std::time::Instant;

use cosmograph_host::drive::{Pace, Pacer, STEPS_PER_MINUTE, Speed, Walk};

use super::Host;
use crate::section::{SiteScene, Walk as Trail};

/// The pacer and the minute's walks.
pub(super) struct Foreground {
    pacer: Pacer,
    /// The plan that supplied the last drawing.
    walks: Option<Arc<[Walk]>>,
    /// The sim tick the pacer began at, and the minutes it has authorised.
    base: u64,
    authorised: u64,
    /// The scene as last drawn, and what it was drawn from.
    drawn: Option<(Rc<SiteScene>, u64, u32, Rc<SiteScene>)>,
}

impl Foreground {
    pub(super) fn new(speed: Speed, base: u64) -> Self {
        Self { pacer: Pacer::new(Pace::Round { speed }), walks: None, base, authorised: 0, drawn: None }
    }

    /// The step to draw at sim tick `tick`: the pacer's, or the minute's last
    /// (what is handed back) while an authorised minute has not landed, so a
    /// slow tick holds bodies at their end stance rather than restarting.
    fn step(&self, tick: u64) -> u32 {
        if tick < self.base + self.authorised {
            STEPS_PER_MINUTE
        } else {
            self.pacer.minute_step(STEPS_PER_MINUTE)
        }
    }
}

/// `scene` with each walking body at the stance it has reached after `step`
/// steps, and the edges walked to it as its trail. A body whose walk has not
/// left its first stance, or has none, stands where the sim has it (D66).
pub(super) fn at_step(scene: &SiteScene, walks: &[Walk], step: u32) -> Option<SiteScene> {
    let mut out = scene.clone();
    let mut moved = false;
    for walk in walks {
        let edge = walk.at(step).edge;
        let route = walk.route();
        let Some(body) = out.bodies.iter_mut().find(|b| b.id == walk.actor()) else { continue };
        let pose = &route.path[edge];
        if edge == 0 || !out.window.holds(pose.at[0], pose.at[2]) {
            continue;
        }
        body.at = out.window.to_ground(pose.at);
        body.physical = Some(pose.clone());
        body.walks = (0..edge)
            .map(|i| Trail {
                poses: [route.path[i].clone(), route.path[i + 1].clone()],
                stride: route.strides.get(i).cloned().flatten(),
            })
            .collect();
        moved = true;
    }
    moved.then_some(out)
}

impl Host {
    /// The foreground rule's pacer, for a world founded with it.
    pub(super) fn found_foreground(&mut self) {
        let (walks, tick) = {
            let sim = self.runtime.sim();
            let rules = &sim.genesis().rules;
            (rules.foreground_walks && rules.speed.is_some(), sim.state().tick)
        };
        if walks {
            // The round's first step would attach the frame; do it before minute 0 is drawn (D70).
            self.runtime.attach_foreground();
            self.foreground = Some(Foreground::new(self.config.round_speed, tick));
        }
    }

    /// Authorises the minutes the pacer reaches in `elapsed_us` of real time;
    /// the runtime runs each on its worker, one clock step apiece, so a slow
    /// minute never stalls the frame. A held run answers its queue and banks
    /// no time, as the clock's does.
    pub(super) fn pace_foreground(&mut self, elapsed_us: u64) -> u64 {
        let held = self.runtime.checkpoint().is_some() || self.runtime.review().is_some();
        let Some(fg) = self.foreground.as_mut().filter(|_| !held) else {
            return self.runtime.advance(0);
        };
        let ticks = fg.pacer.advance(elapsed_us).ticks;
        fg.authorised += ticks;
        let clock_step_us = 1_000_000 / u64::from(self.config.ticks_per_second.max(1));
        self.runtime.advance(ticks * clock_step_us)
    }

    /// Draw ready walks for this revision. While the preview is pending, or
    /// when no body has stepped, the view uses its ordinary native scene.
    pub(super) fn foreground_scene(&mut self) -> Option<Rc<SiteScene>> {
        let source = self.scene.clone()?;
        let fg = self.foreground.as_mut()?;
        let tick = self.runtime.tick();
        static PROFILE: OnceLock<bool> = OnceLock::new();
        let profile = *PROFILE.get_or_init(|| std::env::var_os("MESOCOSM_PROFILE_ACTIVE").is_some_and(|v| v == "1"));
        let begun = profile.then(Instant::now);
        let Some(walks) = self.runtime.poll_foreground_walks().ok().flatten() else {
            fg.walks = None;
            fg.drawn = None;
            return None;
        };
        if !fg.walks.as_ref().is_some_and(|prior| Arc::ptr_eq(prior, &walks)) {
            fg.drawn = None;
            fg.walks = Some(Arc::clone(&walks));
            if let Some(begun) = begun {
                eprintln!("[mesocosm-view] foreground walks={} in {}us", walks.len(), begun.elapsed().as_micros());
            }
        }
        let step = fg.step(tick);
        if let Some((from, at, was, scene)) = &fg.drawn
            && Rc::ptr_eq(from, &source) && *at == tick && *was == step
        {
            return Some(scene.clone());
        }
        let scene = Rc::new(at_step(&source, &walks, step)?);
        if profile {
            let ms = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_millis());
            eprintln!("[mesocosm-view] foreground drawn tick={tick} step={step} at {ms}ms");
        }
        fg.drawn = Some((source, tick, step, scene.clone()));
        Some(scene)
    }

    /// A held frame has a frozen pacer step, but its pending walks can
    /// still arrive. Refresh once to adopt them at that step.
    pub(super) fn foreground_became_ready(&mut self) -> bool {
        self.scene.is_some() && self.foreground.as_ref().is_some_and(|fg| fg.walks.is_none())
            && self.runtime.poll_foreground_walks().is_ok_and(|walks| walks.is_some())
    }

    /// The scene as drawn this frame: the foreground's when it has one
    /// (so the camera follows a body as it is drawn), else the sim's.
    pub(super) fn drawn_scene(&self) -> Option<&Rc<SiteScene>> {
        let source = self.scene.as_ref()?;
        match self.foreground.as_ref().and_then(|fg| fg.drawn.as_ref()) {
            Some((from, _, _, scene)) if Rc::ptr_eq(from, source) => Some(scene),
            _ => Some(source),
        }
    }

    /// The handbacks of the walks being drawn, at the minute's last step.
    #[cfg(test)]
    pub(super) fn drawn_handbacks(&self) -> Option<Vec<cosmograph::spatial::Handback>> {
        let sim = self.runtime.sim();
        let walks = self.foreground.as_ref()?.walks.as_deref()?;
        Some(walks.iter().filter_map(|w| w.handback(&sim, STEPS_PER_MINUTE).unwrap()).collect())
    }

    /// The step of the minute now drawn, for a capture's receipt.
    pub fn foreground_step(&self) -> Option<u32> {
        let tick = self.runtime.sim().state().tick;
        self.foreground.as_ref().map(|fg| fg.step(tick))
    }
}
