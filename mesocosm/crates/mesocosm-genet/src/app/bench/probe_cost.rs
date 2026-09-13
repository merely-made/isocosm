// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Bounded native-host CPU wall-time receipts. No GPU-duration claims.
use cambium_rootstock::FrameProfile;
use serde::Serialize;
use std::collections::BTreeMap;
const MAX_SAMPLES: usize = 4096;
const MAX_PHASES: usize = 32;

#[derive(Clone, Copy, Default)]
pub(super) struct Totals {
    pub redraws: u64,
    pub mesh_bytes: u64,
    pub instance_bytes: u64,
}
#[derive(Clone, Serialize)]
struct Sample {
    frame: u64,
    capture: bool,
    valid: bool,
    times_us: BTreeMap<&'static str, u64>,
    counts: BTreeMap<&'static str, u64>,
}
#[derive(Default)]
pub(super) struct Costs {
    previous: Totals,
    first_document: Option<Sample>,
    first_populated: Option<Sample>,
    active: Option<Phase>,
    phases: Vec<Phase>,
    retained: usize,
    failed: bool,
}
struct Phase {
    name: String,
    samples: Vec<Sample>,
}
#[derive(Debug, Serialize)]
struct Distribution {
    count: usize,
    median: f64,
    p95: u64,
    max: u64,
}
#[derive(Serialize)]
struct PhaseReport<'a> {
    name: &'a str,
    completed: bool,
    samples: &'a [Sample],
    eligible_frames: usize,
    capture_frames: usize,
    invalid_frames: usize,
    first_frame: Option<&'a Sample>,
    all_frames_us: BTreeMap<&'static str, Distribution>,
    steady_after_first_us: BTreeMap<&'static str, Distribution>,
    eligible_counter_totals: BTreeMap<&'static str, u64>,
}
fn distribution(values: impl IntoIterator<Item = u64>) -> Option<Distribution> {
    let mut values: Vec<_> = values.into_iter().collect();
    if values.is_empty() {
        return None;
    }
    values.sort_unstable();
    let count = values.len();
    let median = if count % 2 == 0 {
        values[count / 2 - 1] as f64 / 2. + values[count / 2] as f64 / 2.
    } else {
        values[count / 2] as f64
    };
    Some(Distribution {
        count,
        median,
        p95: values[(count * 95).div_ceil(100) - 1],
        max: values[count - 1],
    })
}
impl Phase {
    fn report(&self, completed: bool) -> PhaseReport<'_> {
        let eligible: Vec<_> = self
            .samples
            .iter()
            .filter(|s| s.valid && !s.capture)
            .collect();
        let mut all = BTreeMap::new();
        let mut steady = BTreeMap::new();
        let mut counters = BTreeMap::<_, u64>::new();
        if let Some(first) = eligible.first() {
            for &key in first.times_us.keys() {
                all.insert(
                    key,
                    distribution(eligible.iter().map(|s| s.times_us[key])).unwrap(),
                );
                if let Some(d) = distribution(eligible.iter().skip(1).map(|s| s.times_us[key])) {
                    steady.insert(key, d);
                }
            }
        }
        for sample in &eligible {
            for (&key, &value) in &sample.counts {
                let n = counters.entry(key).or_default();
                *n = n.saturating_add(value);
            }
        }
        PhaseReport {
            name: &self.name,
            completed,
            samples: &self.samples,
            eligible_frames: eligible.len(),
            capture_frames: self.samples.iter().filter(|s| s.capture).count(),
            invalid_frames: self.samples.iter().filter(|s| !s.valid).count(),
            first_frame: eligible.first().copied(),
            all_frames_us: all,
            steady_after_first_us: steady,
            eligible_counter_totals: counters,
        }
    }
}
impl Costs {
    pub fn observe_context(
        &mut self,
        frame: u64,
        ctx: &super::Context<'_>,
        capture: bool,
    ) -> Result<(), String> {
        let state = ctx.runner.state();
        let mut totals = Totals::default();
        let mut populated = false;
        for scene in std::iter::once(&state.scene).chain(state.cards.iter()) {
            let s = scene.borrow();
            totals.redraws += s.renders;
            totals.mesh_bytes += s.mesh_upload_bytes;
            totals.instance_bytes += s.instance_upload_bytes;
            populated |= s.stats.voxel_bodies > 0;
        }
        // Retired producers keep diagnostic history. Only a producer whose
        // DOM is active may invalidate the current document's sample.
        let mut valid = true;
        if state.visible && !state.effects.open {
            valid = state.scene.borrow().error.is_none();
            if let Some(comparison) = &state.model.borrow().comparison {
                for (index, card) in comparison.cards.iter().enumerate() {
                    if card.world.is_some() {
                        valid &= state.cards[index].borrow().error.is_none();
                    }
                }
            }
        }
        self.observe(frame, ctx.frame_profile, totals, capture, valid, populated)
    }
    pub fn begin(&mut self, name: &str) -> Result<(), String> {
        if self.failed {
            return Err("cost sample budget was exceeded".into());
        }
        if self.active.is_some() {
            return Err("end the active cost phase first".into());
        }
        if self.phases.len() >= MAX_PHASES {
            return Err("at most 32 cost phases are admitted".into());
        }
        if name.is_empty()
            || name.len() > 64
            || !name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            return Err("cost phase names require 1..64 ASCII letters, digits, - or _".into());
        }
        if self.phases.iter().any(|p| p.name == name) {
            return Err(format!("duplicate cost phase {name}"));
        }
        self.active = Some(Phase {
            name: name.into(),
            samples: Vec::new(),
        });
        Ok(())
    }
    pub fn end(&mut self) -> Result<(), String> {
        let phase = self.active.take().ok_or("no cost phase is active")?;
        let valid = phase.samples.iter().any(|s| s.valid && !s.capture);
        self.phases.push(phase);
        if !valid {
            return Err("cost phase has no eligible frame profiles; use settle between cost-begin and cost-end".into());
        }
        Ok(())
    }
    pub fn finish(&self) -> Result<(), String> {
        if self.active.is_some() {
            Err("cost phase was not ended".into())
        } else {
            Ok(())
        }
    }
    pub fn observe(
        &mut self,
        frame: u64,
        profile: Option<FrameProfile>,
        totals: Totals,
        capture: bool,
        valid: bool,
        populated: bool,
    ) -> Result<(), String> {
        let delta = Totals {
            redraws: totals.redraws.saturating_sub(self.previous.redraws),
            mesh_bytes: totals.mesh_bytes.saturating_sub(self.previous.mesh_bytes),
            instance_bytes: totals
                .instance_bytes
                .saturating_sub(self.previous.instance_bytes),
        };
        self.previous = totals;
        if self.failed {
            return Ok(());
        }
        let sample = sample(frame, profile, delta, capture, valid);
        if self.first_document.is_none() {
            self.first_document = Some(sample.clone());
        }
        if self.first_populated.is_none() && populated && delta.redraws > 0 && sample.valid {
            self.first_populated = Some(sample.clone());
        }
        if let Some(phase) = &mut self.active {
            if self.retained >= MAX_SAMPLES {
                self.failed = true;
                return Err(
                    "cost receipt exceeds 4096 samples; split the run into bounded phases".into(),
                );
            }
            phase.samples.push(sample);
            self.retained += 1;
        }
        Ok(())
    }
    pub fn report(&self) -> serde_json::Value {
        serde_json::json!({
            "schema":1,"timing":"CPU wall-time microseconds on the native host thread, not GPU durations",
            "scope":"all bench scenes; actual redraw and upload deltas distinguish retained producer calls",
            "eligibility":"profile available, no active producer error, no capture pending; the host callback does not prove successful presentation or GPU completion per sample",
            "summary_policy":"capture and invalid frames excluded; steady excludes the first eligible frame per phase; p95 is nearest rank; stage timings overlap and must not be summed",
            "first_frame_policy":"first_document is the first observed host frame; first_populated is the first successful scene redraw with body geometry; neither measures process startup or asynchronous generation latency",
            "limits":{"samples":MAX_SAMPLES,"phases":MAX_PHASES},"budget_exceeded":self.failed,
            "first_document":self.first_document,"first_populated":self.first_populated,
            "phases":self.phases.iter().map(|p|p.report(true)).collect::<Vec<_>>(),
            "unfinished":self.active.as_ref().map(|p|p.report(false)),
        })
    }
}
fn sample(
    frame: u64,
    profile: Option<FrameProfile>,
    delta: Totals,
    capture: bool,
    valid: bool,
) -> Sample {
    let available = profile.is_some();
    let p = profile.unwrap_or_default();
    let mut times = BTreeMap::new();
    macro_rules! timings {($($field:ident),*)=>{$(times.insert(stringify!($field),p.$field);)*};}
    timings!(
        total_us,
        frame_hook_us,
        relayout_us,
        layout_update_us,
        layout_tick_us,
        layout_apply_us,
        layout_rebuild_us,
        style_resolve_us,
        layout_with_text_us,
        content_extent_us,
        leaf_boxes_us,
        leaf_render_us,
        leaf_fragments_us,
        ime_us,
        emit_scene_us,
        shadows_us,
        raster_us,
        acquire_us,
        clear_us,
        compose_us,
        present_us,
        capture_us,
        pointer_us,
        a11y_us,
        raster_total_us,
        tile_invalidate_us,
        dirty_tile_rebuild_us,
        master_compose_us,
        vello_render_us
    );
    times.insert("producer_render_us", p.producers.render_us);
    times.insert("producer_stage_us", p.producers.stage_us);
    times.insert(
        "excluding_acquire_present_capture_us",
        p.total_us
            .saturating_sub(p.acquire_us)
            .saturating_sub(p.present_us)
            .saturating_sub(p.capture_us),
    );
    let counts = BTreeMap::from([
        ("producer_calls", p.producers.render_calls),
        ("producer_stages", p.producers.stages),
        ("producer_suspensions", p.producers.suspensions),
        ("producer_retirements", p.producers.retirements),
        ("producer_invalid_frames", p.producers.invalid_frames),
        ("actual_scene_redraws", delta.redraws),
        ("mesh_upload_bytes", delta.mesh_bytes),
        ("instance_upload_bytes", delta.instance_bytes),
        ("layout_mutations", p.layout_mutations),
        ("layout_rebuilt", u64::from(p.layout_rebuilt)),
        ("leaf_repaints", p.leaf_repaints),
        ("dirty_tiles", p.dirty_tiles),
    ]);
    Sample {
        frame,
        capture,
        valid: valid && available && p.producers.invalid_frames == 0,
        times_us: times,
        counts,
    }
}
#[cfg(test)]
#[path = "probe_cost/tests.rs"]
mod tests;
