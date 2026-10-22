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
        let some = |id: Option<u64>| id.map_or_else(|| "none".into(), |id| id.to_string());
        let yes_no = |value| if value { "yes" } else { "no" }.to_owned();
        let dev_intents = r.dev_intents();
        let body_stats = s.producer.borrow_mut().body_stats().unwrap_or_default();
        let terrain_stats = s.producer.borrow_mut().terrain_stats().unwrap_or_default();
        let physical = r.spatial();
        let (walks, children) = physical.driver.as_ref().map_or((0, 0), |driver| (
            driver.records.iter().map(|record| record.steps.len()).sum::<usize>(),
            driver.records.iter().flat_map(|record| &record.changes)
                .filter(|change| matches!(change.admission, mesocosm_runtime::spatial::Admission::Birth { .. })).count()));
        let living = r
            .sim()
            .state()
            .population
            .groups
            .values()
            .filter(|g| g.entity.alive)
            .map(|g| g.count)
            .sum::<u64>();
        ProbeSnapshot {
            focused: s.host.followed().map(|id| format!("critter {id}")),
            fields: [
                ("mode", "played".into()),
                ("camera", s.host.config.camera.name().into()),
                ("render-scale", s.host.config.render_scale.to_string()),
                ("frames", s.host.frames.to_string()),
                ("dev", s.host.config.dev.to_string()),
                ("dev-intents", dev_intents.to_string()),
                (
                    "assisted",
                    if dev_intents == 0 {
                        "unassisted".into()
                    } else {
                        crate::played::assisted_label(dev_intents)
                    },
                ),
                ("controlled", some(r.critter())),
                ("follow", some(s.host.followed())),
                ("site", some(s.host.scene.as_ref().map(|s| s.site))),
                ("living", living.to_string()),
                ("physical-members", physical.members.len().to_string()),
                ("whole-body-placement", yes_no(isocosm::spatial::embodiment::enabled(&r.sim()))),
                ("accepted-walks", walks.to_string()),
                ("food-directed-walking", yes_no(r.sim().foraging_enabled())),
                ("native-poses", r.sim().state().spatial_poses.len().to_string()),
                ("placed-offspring", children.to_string()),
                ("controlled-placement", yes_no(r.critter().is_some_and(|id| physical.members.get(&id).is_some_and(|at| at.realized(&r.sim()).is_ok()
                    && physical.placement_unoccupied(&r.sim(), id).is_ok())))),
                ("body-founding", r.sim().genesis().founding.as_ref()
                    .and_then(|f| f.bodies.as_ref()).and_then(|b| b.founding.clone())
                    .unwrap_or_else(|| "historical".into())),
                ("functional-parts", s.host.scene.as_ref().map_or(0, |scene| scene.bodies.iter()
                    .map(|b| b.materials.len()).sum::<usize>()).to_string()),
                ("bodies", s.host.config.body_mode.name().into()),
                ("voxel-bodies", body_stats.voxel_bodies.to_string()),
                ("terrain-bricks", terrain_stats.resident.to_string()),
                ("terrain-capacity", terrain_stats.capacity.to_string()),
                ("terrain-overflow", terrain_stats.overflow.to_string()),
                ("voxel-parts", body_stats.voxel_parts.to_string()),
                ("smooth-primitives", body_stats.smooth_primitives.to_string()),
                ("posed-joints", body_stats.posed_joints.to_string()),
                ("foot-contacts", body_stats.foot_contacts.to_string()),
                ("planted-contacts", body_stats.planted_contacts.to_string()),
                ("swinging-contacts", body_stats.swinging_contacts.to_string()),
                ("support-contacts", body_stats.support_contacts.to_string()),
                ("fallback-bodies", body_stats.fallback_bodies.to_string()),
                ("projection-failures", body_stats.projection_failures.to_string()),
                ("body-projection-error", body_stats.last_error.clone().unwrap_or_default()),
                ("scene-physical-bodies", s.host.scene.as_ref().map_or(0, |scene|
                    scene.bodies.iter().filter(|body| body.physical.is_some()).count()).to_string()),
                ("scene-parts", s.host.scene.as_ref().map_or(0, |scene|
                    scene.bodies.iter().map(|body| body.document.living().count()).sum::<usize>()).to_string()),
                ("boundary", yes_no(r.review().is_some())),
                ("fault", r.fault().unwrap_or("").to_owned()),
                ("tick", r.tick().to_string()),
                ("hash", format!("{:016x}", r.state_hash())),
                ("revision", r.revision().unwrap_or(0).to_string()),
                ("captures", captures.to_string()),
                ("board", r.review().is_some().to_string()),
                ("checkpoint", yes_no(r.checkpoint().is_some())),
                ("queued", r.queued_len().to_string()),
                ("steps", s.host.steps.to_string()),
                ("renders", s.producer.borrow().renders().to_string()),
                ("paused", yes_no(s.host.dev_paused)),
                ("inspecting", yes_no(s.host.inspection.open)),
                (
                    "selected-part",
                    s.host
                        .inspection
                        .selected
                        .map_or_else(|| "none".into(), |p| p.part.0.to_string()),
                ),
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
        if s.host.scene.is_some() && s.producer.borrow().renders() == 0 {
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
                "steps":s.host.steps,"frames":s.host.frames,"bench":s.host.config.bench_world}),
            )?;
        }
        Ok(())
    }
}
