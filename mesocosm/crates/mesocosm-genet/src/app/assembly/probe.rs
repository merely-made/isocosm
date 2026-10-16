// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::{State, view};
use cambium_genet_winit_host::AppCtx;
use taproot::ProbeSnapshot;

type Context<'a> = AppCtx<'a, State, view::Logic, view::Child>;
pub(super) struct Probe;
impl mesquite::Product for Probe {
    type State = State;
    type Logic = view::Logic;
    type View = view::Child;
    const KIND: &'static str = "mesocosm-core";
    const SURFACE: &'static str = "mesocosm";
    const LOG_PREFIX: &'static str = "mesocosm";
    fn sheet(&self) -> &str {
        ""
    }
    fn snapshot(&self, ctx: &Context<'_>, captures: usize, _: f32) -> ProbeSnapshot {
        let s = ctx.runner.state();
        let r = &s.host.runtime;
        ProbeSnapshot {
            focused: None,
            fields: [
                ("tick", r.tick().to_string()),
                ("hash", format!("{:016x}", r.state_hash())),
                ("revision", r.revision().unwrap_or(0).to_string()),
                ("captures", captures.to_string()),
                ("board", r.review().is_some().to_string()),
                ("checkpoint", r.checkpoint().is_some().to_string()),
                ("queued", r.queued_len().to_string()),
                ("steps", s.host.steps.to_string()),
                (
                    "pointed",
                    s.last_point
                        .map_or_else(|| "none".into(), |p| format!("{p:?}")),
                ),
            ]
            .into_iter()
            .map(|(k, v)| (k.to_owned(), v))
            .collect(),
        }
    }
    fn default_capture_path(&self) -> std::path::PathBuf {
        crate::played::default_out_dir().join("scratch_core.png")
    }
    fn act(&mut self, ctx: &mut Context<'_>, label: &str) -> bool {
        let mut done = false;
        ctx.runner.update(|s| {
            if let Some(key) = super::super::actions::key_named(label.trim()) {
                s.press(&key);
                s.host.note_outcomes();
                done = true;
            } else {
                done = s.host.run_action(label);
            }
            s.refresh();
        });
        done
    }
    fn drain_events(&mut self, ctx: &mut Context<'_>) -> Vec<String> {
        let mut events = Vec::new();
        ctx.runner
            .update(|s| events = std::mem::take(&mut s.host.events));
        events
    }
    fn busy(&self, ctx: &Context<'_>, capture_pending: bool) -> Option<bool> {
        let s = ctx.runner.state();
        Some(capture_pending || s.host.pump.is_some() || s.host.runtime.queued_len() > 0)
    }
    fn complete(&mut self, ctx: &mut Context<'_>, _: &taproot::Outcome) -> Result<(), String> {
        let s = ctx.runner.state();
        if s.producer.borrow().renders() == 0 || s.picture.snapshot().is_none() {
            return Err("the core viewport never presented its native picture".into());
        }
        if let Some(why) = s.picture.last_error() {
            return Err(why);
        }
        if let Some(path) = &s.host.config.trace {
            crate::played::write_world_trace(path, &s.host.runtime)?;
        }
        if let Some(path) = &s.host.config.receipt {
            let timing = path.with_extension("timing.json");
            crate::played::write_json(
                &timing,
                &serde_json::json!({"frame_work_us":s.frame_us,
                "frame_intervals_us":s.host.frame_intervals_us,
                "state_hash":s.host.runtime.state_hash(),"world_revision":s.host.runtime.revision()?,
                "steps":s.host.steps,"frames":s.host.frames,"bench":true}),
            )?;
        }
        Ok(())
    }
}
