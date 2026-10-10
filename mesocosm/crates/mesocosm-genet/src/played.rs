// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What a played session leaves behind, and what drives one without a player.
//!
//! A run's trace is its native session's save ([`PlayedTrace`]): the genesis
//! and the log, JSON through serde. `--replay` loads it back through
//! [`Runtime::replayed`], which replays every command and checks every
//! witness, and compares the hash it lands on with the one the save recorded.
//!
//! The trace, the receipt and the capture default to a **scratch** name under
//! the headed-verify home ([`DEFAULT_STEM`]); the golden `ps1_played.*` stem
//! is written only when a flag names it (ruled 2026-09-02).

use std::path::{Path, PathBuf};

use mesocosm_runtime::Runtime;
use serde::Serialize;

pub mod layout;
pub mod scene;
pub use scene::site_scene;

/// A recorded run: the native session's save.
pub type PlayedTrace = isocosm::history::Saved;

/// How often the demo nudges its critter toward its own site.
const NUDGE_EVERY: u64 = 40;

/// The part a selection names, on the way out.
#[derive(Clone, Debug, Serialize)]
pub struct PartSelectionReceipt {
    pub organism: u64,
    pub part: u32,
    pub revision: u64,
}

/// The terrain import's opaque participation in Netrender's graph.
/// Body layering and chrome are separate from these boundary counts.
#[derive(Clone, Debug, Serialize)]
pub struct FrameGraphReceipt {
    pub tenant_name: String,
    pub producer_path: String,
    pub fallback_count: u64,
    pub scene_op_boundary: usize,
    pub caller_reported_physical_submission_count: Option<u64>,
    pub logical_opaque_producer_boundaries: usize,
    pub graph_encoder_batches: usize,
    pub graph_submission_boundaries: usize,
    pub logical_plan_dump: String,
}

/// What a run says about itself on the way out.
#[derive(Clone, Debug, Serialize)]
pub struct PlayedReceipt {
    /// `played` for a session at the keyboard.
    pub mode: &'static str,
    pub seed: u64,
    pub population: u64,
    /// The runtime's own receipt: genesis, tick, log, hash, assistance.
    pub run: mesocosm_runtime::Receipt,
    pub steps: u64,
    pub frames: u32,
    /// Envelopes the runtime refused.
    pub refused: usize,
    /// Why the sim stopped the run, if it did.
    pub fault: Option<String>,
    pub adapter: String,
    pub backend: String,
    /// The site the section drew.
    pub site: Option<u64>,
    pub ground_revision: u64,
    pub body_parts: usize,
    /// Parts of the played body past the lens capsule budget on the last frame.
    pub body_capsules_dropped: u32,
    pub section_roster: u32,
    pub roster_capsules_dropped: u32,
    pub slab_half_height: f32,
    /// Which way the section looked; presentation only.
    pub camera: &'static str,
    pub terrain_style: &'static str,
    pub bodies: &'static str,
    pub body_budget: usize,
    pub body_light: f32,
    pub body_projection: crate::section::BodyFrameStats,
    pub inspecting: bool,
    pub selected_part: Option<PartSelectionReceipt>,
    /// Absent only when the chromeless fallback drew the frame.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frame_graph: Option<FrameGraphReceipt>,
    pub trace: Option<String>,
    pub capture: Option<String>,
    /// Whether `--dev` was set (DT1).
    pub dev: bool,
    /// Dev intents the runtime applied (DT3); nonzero labels the run assisted.
    pub dev_intents: u64,
}

/// `Code/testing/<repo>/`, the workspace's headed-verify home.
pub fn default_out_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(4)
        .map(|code| code.join("testing").join("mesocosm"))
        .unwrap_or_else(|| PathBuf::from("testing/mesocosm"))
}

/// The stem every default output shares; never the golden fixture's.
pub const DEFAULT_STEM: &str = "scratch_played";

pub fn default_trace_path() -> PathBuf {
    default_out_dir().join(format!("{DEFAULT_STEM}.trace.json"))
}

pub fn default_receipt_path() -> PathBuf {
    default_out_dir().join(format!("{DEFAULT_STEM}.json"))
}

pub fn default_capture_path() -> PathBuf {
    default_out_dir().join(format!("{DEFAULT_STEM}.png"))
}

/// Empty for an unaided run, `assisted (N dev intents)` otherwise (DT3).
pub fn assisted_label(dev_intents: u64) -> String {
    if dev_intents > 0 {
        format!("assisted ({dev_intents} dev intents)")
    } else {
        String::new()
    }
}

fn ensure_parent(path: &Path) -> Result<(), String> {
    let Some(parent) = path.parent() else {
        return Ok(());
    };
    std::fs::create_dir_all(parent).map_err(|error| format!("{}: {error}", parent.display()))
}

pub fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    ensure_parent(path)?;
    let json = serde_json::to_string_pretty(value).map_err(|error| error.to_string())?;
    std::fs::write(path, json).map_err(|error| format!("{}: {error}", path.display()))
}

pub fn read_trace(path: &Path) -> Result<PlayedTrace, String> {
    let bytes = std::fs::read(path).map_err(|error| format!("{}: {error}", path.display()))?;
    serde_json::from_slice(&bytes).map_err(|error| format!("{}: {error}", path.display()))
}

/// Takes a recorded trace up where it stands, to play on (786).
pub fn resume(path: &Path, ticks_per_second: u32) -> Result<Runtime, String> {
    let saved = read_trace(path)?;
    let pace = isocosm::directing::interim::Pace {
        round: 1,
        scoring: 6,
    };
    let mode = isocosm::directing::readings::Mode::Creative;
    Runtime::resume(saved, mode, pace, ticks_per_second)
}

/// Replays a recorded trace; returns the recorded hash and the replayed one.
pub fn replay(path: &Path) -> Result<(u64, u64), String> {
    let saved = read_trace(path)?;
    let recorded = saved.state_hash;
    let replayed = Runtime::replayed(saved)?;
    Ok((recorded, replayed.state_hash))
}

pub fn write_png(path: &Path, width: u32, height: u32, pixels: &[u8]) -> Result<(), String> {
    ensure_parent(path)?;
    let file =
        std::fs::File::create(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let mut encoder = png::Encoder::new(file, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder
        .write_header()
        .and_then(|mut writer| writer.write_image_data(pixels))
        .map_err(|error| error.to_string())
}

/// One step of the demo, off the clock: answer a standing question the way
/// a hand would (keep the parent, take the first heir, close the boundary),
/// otherwise nudge now and then, then take one round.
pub fn demo_step(runtime: &mut Runtime, step: u64) {
    if let Some(intent) = demo_intent(runtime, step) {
        let envelope = crate::input::envelope(runtime, intent);
        runtime.queue(envelope);
    }
    runtime.step(1);
}

fn demo_intent(runtime: &Runtime, step: u64) -> Option<isocosm_overlay::mesocosm::MesocosmIntent> {
    use winit::keyboard::{Key, NamedKey};
    if let Some(checkpoint) = runtime.checkpoint() {
        let take = Key::Character("t".into());
        let carry = Key::Named(NamedKey::Enter);
        let key = match checkpoint.occasion {
            mesocosm_runtime::Occasion::Loss(_) => &take,
            _ => &carry,
        };
        return crate::input::answer_for(checkpoint, runtime.critter(), key);
    }
    (step % NUDGE_EVERY == 0)
        .then(|| crate::input::intent_for(runtime, None, &Key::Character("e".into())))
        .flatten()
}
