//! `ISOCOSM_REGION_SELFTEST=picker` founds Mill Ford from the quick-start
//! picker. Any other label: once a restored native table offers
//! its projections, select the GM's adventure projection, where the places
//! read off their residents sit beside the polities, for a capture.

use super::*;
use std::time::{Duration, Instant};

const REGION_SELFTEST_TIMEOUT: Duration = Duration::from_secs(90);

pub(crate) struct RegionSelftest {
    party: String,
    step: u8,
    started: Option<Instant>,
    asked: bool,
    done: bool,
}

impl RegionSelftest {
    pub(crate) fn from_env() -> Option<Self> {
        std::env::var("ISOCOSM_REGION_SELFTEST").ok()
            .map(|party| Self { party, step: 0, started: None, asked: false, done: false })
    }

    pub(crate) fn done(&self) -> bool {
        self.done
    }

    fn drive(&mut self, app: &mut App, ctx: &mut Ctx<'_>) {
        if self.done {
            return;
        }
        let started = *self.started.get_or_insert_with(Instant::now);
        assert!(started.elapsed() < REGION_SELFTEST_TIMEOUT,
            "region selftest saw no adventure projection; status={:?}", ctx.runner.state().status);
        if started.elapsed() < Duration::from_secs(2) {
            return;
        }
        if self.party == "picker" {
            self.pick(app, ctx);
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
            eprintln!("[isocosm] region selftest: GM adventure projection selected for {}", self.party);
            self.done = true;
            // The capture waits on a frame; a still table draws one only on change.
            ctx.runner.update(|ui| ui.status = "GM adventure projection: places and polities".into());
        }
    }
}

impl RegionSelftest {
    /// Each step sets the state its button would, then runs the tail a click
    /// runs: pick Mill Ford, open the GM's table through the admission door
    /// with nothing admitted, and save, so a restart reads it at the GM seat.
    fn pick(&mut self, app: &mut App, ctx: &mut Ctx<'_>) {
        let ui = ctx.runner.state();
        match self.step {
            0 => {
                let Some(index) = ui.native.quickstarts.iter().position(|q| q.name == "Mill Ford") else { return };
                ctx.runner.update(|ui| { ui.open_generator(); ui.request_quickstart(index); });
            },
            1 => {
                let Some(world) = ui.native.world.as_ref() else { return };
                eprintln!("[isocosm] region selftest: {}", ui.status);
                let input = isonetry::native_table::NativeTableAdmissionInput {
                    expected_native: world.cursor().expect("a founded world has a cursor"),
                    expected_table: isonetry::native_table::table_digest(None),
                    participant: isocosm_native::native::ParticipantInput::Join,
                    batch: isocosm_native::native::AdmissionBatch { version: 1, assertions: vec![],
                        characters: vec![], placements: vec![], controls: vec![], project_carry: None,
                        spatial: vec![] },
                };
                ctx.runner.update(|ui| ui.native.admission_request = Some(input));
            },
            2 => {
                if ui.native.table.is_none() { return };
                eprintln!("[isocosm] region selftest: {}", ui.status);
                ctx.runner.update(|ui| ui.save_requested = true);
            },
            _ => {
                eprintln!("[isocosm] region selftest: {}", ui.status);
                self.done = true;
                return;
            },
        }
        app.after_dispatch(ctx);
        self.step += 1;
    }
}

impl App {
    pub(crate) fn maybe_region_selftest(&mut self, ctx: &mut Ctx<'_>) {
        let Some(mut test) = self.region_selftest.take() else {
            return;
        };
        test.drive(self, ctx);
        self.region_selftest = Some(test);
    }
}
