// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Bounded opt-in diagnostic capture of the actual presented viewport.
//! Uses Section's existing GPU readback and the ordinary native pose driver.
use super::*;
use std::{path::PathBuf, time::Instant};

pub(super) struct Recording {
    root: Option<PathBuf>,
    ready: bool,
    start: Instant,
    frames: Vec<serde_json::Value>,
}
impl Default for Recording {
    fn default() -> Self {
        Self { root: std::env::var_os("MESOCOSM_CAPTURE_MOTION").map(PathBuf::from),
            ready: false, start: Instant::now(), frames: Vec::new() }
    }
}
impl Recording {
    pub(super) fn frame(&mut self, section: &Section, texture: &wgpu::Texture,
        input: &super::super::Snapshot, was_animating: bool) -> Result<(), String> {
        let Some(root) = &self.root else { return Ok(()) };
        if self.frames.len() >= 24 || (!self.frames.is_empty() && !was_animating && !section.animating()) {
            return Ok(());
        }
        if !self.ready {
            // An existing directory may contain a cited receipt. Never reuse it.
            std::fs::create_dir(root).map_err(|e| format!("motion capture directory: {e}"))?;
            self.ready = true;
        }
        let (width, height, rgba) = section.capture_from(texture, |_,_,_| {})
            .ok_or("motion frame GPU readback failed")?;
        let stats = section.body_stats();
        let name = format!("frame-{:02}.png", self.frames.len());
        crate::played::write_png(&root.join(&name), width, height, &rgba)?;
        self.frames.push(serde_json::json!({ "image": name,
            "elapsed_ms": self.start.elapsed().as_millis(), "revision": input.revision,
            "focus": input.presentation.as_ref().and_then(|p| p.focus),
            "planted": stats.planted_contacts, "swinging": stats.swinging_contacts,
            "reachable_feet": stats.foot_contacts, "joints": stats.posed_joints,
            "projection_failures": stats.projection_failures,
            "roots": input.scene.bodies.iter().filter_map(|b| b.physical.as_ref()
                .map(|pose| serde_json::json!({"id":b.id,"at":pose.at}))).collect::<Vec<_>>() }));
        crate::played::write_json(&root.join("frames.json"), &self.frames)?;
        Ok(())
    }
}
