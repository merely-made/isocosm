// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The bench's scenario lane: mesquite's ticking, capture and receipt
//! machinery, with the bench's own snapshot.

use std::{cell::Cell, path::PathBuf, rc::Rc};

use cambium_genet_winit_host::AppCtx;
use taproot::{ProbeSnapshot, Scenario};

use super::{Bench, Child, Logic, SHEET};

pub(super) type Context<'a> = AppCtx<'a, Bench, Logic, Child>;

struct BenchProduct;

impl mesquite::Product for BenchProduct {
    type State = Bench;
    type Logic = Logic;
    type View = Child;

    const KIND: &'static str = "native-bench";
    const SURFACE: &'static str = "bench";
    const LOG_PREFIX: &'static str = "bench";

    fn sheet(&self) -> &str {
        SHEET.as_str()
    }

    fn snapshot(&self, ctx: &Context<'_>, captures: usize, _opacity: f32) -> ProbeSnapshot {
        let sim = &ctx.runner.state().sim;
        let session = sim.session.as_ref();
        let state = session.map(|s| s.sim.state());
        let fields = [
            ("captures", captures.to_string()),
            ("founded", (session.is_some()).to_string()),
            ("tick", state.map_or(0, |s| s.tick).to_string()),
            (
                "population",
                state.map_or(0, |s| s.population.count()).to_string(),
            ),
            ("sites", state.map_or(0, |s| s.sites.len()).to_string()),
            ("selected", sim.selected.to_string()),
            ("playing", sim.playing.to_string()),
            ("branched", sim.other.is_some().to_string()),
            (
                "hash",
                session.map_or(String::new(), |s| format!("{:016x}", s.sim.state_hash())),
            ),
        ];
        ProbeSnapshot {
            focused: None,
            fields: fields
                .into_iter()
                .map(|(name, value)| (name.to_string(), value))
                .collect(),
        }
    }

    fn drain_events(&mut self, ctx: &mut Context<'_>) -> Vec<String> {
        let mut events = Vec::new();
        ctx.runner
            .update(|state| events = std::mem::take(&mut state.events));
        events
    }

    fn default_capture_path(&self) -> PathBuf {
        crate::played::default_out_dir().join("bench.png")
    }
}

pub(super) struct Lane(mesquite::Lane<BenchProduct>);

impl Lane {
    pub fn new(
        scenario: Option<Scenario>,
        receipt: Option<PathBuf>,
        final_capture: Option<PathBuf>,
        exit_code: Rc<Cell<i32>>,
    ) -> Self {
        Self(mesquite::Lane::new(
            BenchProduct,
            scenario,
            receipt,
            final_capture,
            exit_code,
        ))
    }

    pub fn with_frame_limit(self, limit: Option<u32>) -> Self {
        Self(self.0.with_frame_limit(limit))
    }

    pub fn request_close(&mut self) {
        self.0.request_close();
    }

    pub fn after_frame(&mut self, ctx: &mut Context<'_>) {
        self.0.after_frame(ctx);
    }
}
