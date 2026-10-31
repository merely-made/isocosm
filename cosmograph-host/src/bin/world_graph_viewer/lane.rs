// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The viewer's half of the scenario lane: what a scenario can assert, and
//! the commands whose target depends on the world drawn (a seed nobody chose
//! has no site a script could name in advance).

use cambium_genet_winit_host::AppCtx;
use cosmograph_host::endpoint::{BIRTH, DEATH};
use cosmograph_host::endpoint::graph::OPEN;
use taproot::ProbeSnapshot;

use crate::view::{Child, Logic, SHEET, Viewer};

type Ctx<'a> = AppCtx<'a, Viewer, Logic, Child>;

pub struct Lane;

impl mesquite::Product for Lane {
    type State = Viewer;
    type Logic = Logic;
    type View = Child;
    const KIND: &'static str = "world-graph-viewer";
    const SURFACE: &'static str = "viewer";
    const LOG_PREFIX: &'static str = "world-graph-viewer";

    fn sheet(&self) -> &str { SHEET }

    fn snapshot(&self, ctx: &Ctx<'_>, _captures: usize, _: f32) -> ProbeSnapshot {
        let v = ctx.runner.state();
        let board = v.board.borrow();
        let shown = |prefix: &str| v.client.shown.iter().filter(|s| !s.hidden && s.id.starts_with(prefix)).count();
        ProbeSnapshot::default()
            .with_field("tick", v.number("Tick").to_string())
            .with_field("head", v.number("Head").to_string())
            .with_field("line", v.number("Line").to_string())
            .with_field("lines", v.number("Lines").to_string())
            .with_field("entries", v.number("Entries").to_string())
            .with_field("hash", v.client.world().and_then(|w| w.value("State hash")).unwrap_or("").to_string())
            .with_field("regions", shown("region:").to_string())
            .with_field("sites", shown("site:").to_string())
            .with_field("groups", shown("group:").to_string())
            .with_field("cards", board.cards().to_string())
            .with_field("edges", board.edges().to_string())
            .with_field("notice", v.client.notice.clone())
    }

    fn drain_events(&mut self, ctx: &mut Ctx<'_>) -> Vec<String> {
        let mut drained = Vec::new();
        ctx.runner.update(|s| drained = std::mem::take(&mut s.events));
        drained
    }

    /// `act open-busiest-site`, `act death`, `act birth`: the world-dependent picks.
    fn act(&mut self, ctx: &mut Ctx<'_>, label: &str) -> bool {
        let mut done = false;
        ctx.runner.update(|v| {
            let (target, intent) = match label {
                "open-busiest-site" => (v.busiest_site(), OPEN),
                "death" => (v.group_offering(DEATH), DEATH),
                "birth" => (v.group_offering(BIRTH), BIRTH),
                _ => return,
            };
            if let Some(id) = target {
                v.invoke(&id, intent, b"null");
                done = v.client.notice.is_empty();
            }
        });
        done
    }
}
