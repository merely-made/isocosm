// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Bounded native-frame capture, promoted from Isometry's `capture.rs`.
//!
//! A headed receipt wants one final image. Continuous readback and PNG
//! encoding stall the render thread, so they stay opt-in. The policy is three
//! facts — a directory, whether every frame counts, and whether one has
//! already landed — and none of the three is about Isometry; what was about
//! Isometry is the two environment variable names, the file name and the log
//! prefix, so those are [`CaptureNames`] and the rest is here.
//!
//! **Not mesquite's capture.** `mesquite::Lane` captures on a scenario's say-so,
//! at names the scenario chooses, with pixel comparison and a receipt behind
//! it. This is the other lane: an unscripted headed run that wants one PNG
//! when the host has stopped moving. A product with a scenario wants mesquite;
//! a product with an environment variable wants this; Isometry has both.
//!
//! The write is to a `.tmp` beside the target and then a rename, so a reader
//! watching the directory never sees half a PNG. `mesquite::write_png` does
//! the encoding — there is no second PNG encoder in the wing.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use cambium_genet_winit_host::read_frame;

use super::{Ctx, Product};

/// What a product calls its capture policy: the two environment variables it
/// reads, the file it writes, and the prefix on a failure line.
#[derive(Clone, Copy, Debug)]
pub struct CaptureNames {
    /// The variable naming the output directory. Nothing is captured without
    /// it.
    pub directory_var: &'static str,
    /// The variable that turns every frame into a capture. `=1` arms it.
    pub every_frame_var: &'static str,
    /// The file written inside the directory.
    pub file: &'static str,
    /// The prefix on the failure line.
    pub log_prefix: &'static str,
}

/// Bounded capture policy.
pub struct Capture {
    names: CaptureNames,
    directory: Option<PathBuf>,
    every_frame: bool,
    /// Shared with the armed closure, which runs on the presentation path.
    saved: Arc<AtomicBool>,
}

impl Capture {
    /// The policy this process's environment declares.
    pub fn from_env(names: CaptureNames) -> Self {
        Self {
            names,
            directory: std::env::var_os(names.directory_var).map(Into::into),
            every_frame: std::env::var(names.every_frame_var).is_ok_and(|value| value == "1"),
            saved: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Whether this frame may be read back. A capture that has already landed
    /// is not taken again unless every frame was asked for.
    fn should_arm(&self, ready: bool) -> bool {
        self.directory.is_some()
            && (self.every_frame || (ready && !self.saved.load(Ordering::Acquire)))
    }

    /// Request a readback for this presentation when the policy allows it.
    ///
    /// A failed readback or write leaves `saved` clear, so a later natural
    /// frame retries. Capture never creates an idle redraw loop.
    pub fn arm<P: Product>(&self, ctx: &mut Ctx<'_, P>, ready: bool) {
        if !self.should_arm(ready) {
            return;
        }
        let directory = self.directory.clone().expect("checked by should_arm");
        let every_frame = self.every_frame;
        let saved = self.saved.clone();
        let (file, prefix) = (self.names.file, self.names.log_prefix);
        *ctx.capture = Some(Box::new(move |surface, view, width, height| {
            if !every_frame && saved.load(Ordering::Acquire) {
                return;
            }
            let Some(frame) = read_frame(surface, view, width, height) else {
                return;
            };
            let path = directory.join(file);
            let pending = directory.join(format!("{file}.tmp"));
            let result = mesquite::write_png(&pending, &frame)
                .and_then(|()| std::fs::rename(&pending, &path).map_err(|why| why.to_string()));
            match result {
                Ok(()) => saved.store(true, Ordering::Release),
                Err(error) => eprintln!("[{prefix}] capture failed: {error}"),
            }
        }));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NAMES: CaptureNames = CaptureNames {
        directory_var: "ISOMERE_TEST_CAPTURE_DIR",
        every_frame_var: "ISOMERE_TEST_CAPTURE_EVERY_FRAME",
        file: "capture.png",
        log_prefix: "isomere",
    };

    fn capture(every_frame: bool, saved: bool) -> Capture {
        Capture {
            names: NAMES,
            directory: Some(PathBuf::from("receipt")),
            every_frame,
            saved: Arc::new(AtomicBool::new(saved)),
        }
    }

    #[test]
    fn ordinary_capture_waits_for_ready_and_stops_after_success() {
        let capture = capture(false, false);
        assert!(!capture.should_arm(false));
        assert!(capture.should_arm(true));
        capture.saved.store(true, Ordering::Release);
        assert!(!capture.should_arm(true));
    }

    #[test]
    fn every_frame_capture_remains_armed_while_the_product_settles() {
        let capture = capture(true, true);
        assert!(capture.should_arm(false));
        assert!(capture.should_arm(true));
    }

    #[test]
    fn no_directory_captures_nothing() {
        let mut capture = capture(true, false);
        capture.directory = None;
        assert!(!capture.should_arm(true));
    }
}
