//! `ISOMETRY_REGION_SELFTEST=<label>`: once a restored native table offers
//! its projections, select the GM's adventure projection, where the places
//! read off their residents sit beside the polities, for a capture.

use super::*;
use std::time::{Duration, Instant};

const REGION_SELFTEST_TIMEOUT: Duration = Duration::from_secs(90);

pub(crate) struct RegionSelftest {
    party: String,
    started: Option<Instant>,
    asked: bool,
    done: bool,
}

impl RegionSelftest {
    pub(crate) fn from_env() -> Option<Self> {
        std::env::var("ISOMETRY_REGION_SELFTEST").ok()
            .map(|party| Self { party, started: None, asked: false, done: false })
    }

    pub(crate) fn done(&self) -> bool {
        self.done
    }

    fn drive(&mut self, ctx: &mut Ctx<'_>) {
        if self.done {
            return;
        }
        let started = *self.started.get_or_insert_with(Instant::now);
        assert!(started.elapsed() < REGION_SELFTEST_TIMEOUT,
            "region selftest saw no adventure projection; status={:?}", ctx.runner.state().status);
        if started.elapsed() < Duration::from_secs(2) {
            return;
        }
        // A restored native table opens as the endpoint's projections: select
        // the GM's adventure projection, as its tab does, and let it draw.
        if !self.asked {
            let id = isocosm_native::ADVENTURE;
            if !ctx.runner.state().vtt_projections.iter().any(|view| view.id == id) {
                return;
            }
            ctx.runner.update(|ui| ui.select_vtt_projection(id));
            self.asked = true;
            return;
        }
        if ctx.runner.state().vtt_selected_projection == isocosm_native::ADVENTURE {
            eprintln!("[isometry] region selftest: GM adventure projection selected for {}", self.party);
            self.done = true;
            // The capture waits on a frame; a still table draws one only on change.
            ctx.runner.update(|ui| ui.status = "GM adventure projection: places and polities".into());
        }
    }
}

impl App {
    pub(crate) fn maybe_region_selftest(&mut self, ctx: &mut Ctx<'_>) {
        let Some(mut test) = self.region_selftest.take() else {
            return;
        };
        test.drive(ctx);
        self.region_selftest = Some(test);
    }
}
