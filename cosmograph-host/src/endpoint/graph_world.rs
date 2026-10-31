// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The world-graph projection's session (sim host plan H9): the graph of
//! the viewed state, a world card that carries the timeline's actions, and
//! the intents. Opening, closing, filtering, scrubbing and switching lines
//! are curation; stepping, branching and a change inside a branch are
//! domain truth for the line they touch.

use chirograph::{
    AdvertisedAction, IntentEffect, IntentInvocation, IntentResult, ProjectionRequest,
    ProjectionSnapshot, ResourceRequest, ResourceResponse,
};
use cosmograph::Command;
use serde_json::Value;

use super::disclosure::{Disclosure, Item, NULL, action, card, rejected, value};
use super::graph::{CLOSE, Domain, OPEN, View, graph};
use super::timeline::Timeline;
use super::{STEP, ticks};
use crate::world::World;

pub const GRAPH: &str = "cosmograph:graph";
pub const SCRUB: &str = "cosmograph.scrub";
pub const BRANCH: &str = "cosmograph.branch";
pub const LINE: &str = "cosmograph.line";
pub const FILTER: &str = "cosmograph.filter";
pub const DEATH: &str = "cosmograph.death";
pub const BIRTH: &str = "cosmograph.birth";
const ADAPTER: &str = "cosmograph.graph";

pub struct GraphSession {
    disclosure: Disclosure,
    view: View,
    timeline: Timeline,
}

/// A payload's one named number.
fn number(payload: &[u8], key: &str) -> Result<u64, String> {
    let value: Value = serde_json::from_slice(payload).map_err(|e| e.to_string())?;
    value.get(key).and_then(Value::as_u64).ok_or_else(|| format!("this intent names a {key}"))
}

fn domains(payload: &[u8]) -> Result<Vec<Domain>, String> {
    let value: Value = serde_json::from_slice(payload).map_err(|e| e.to_string())?;
    let words = value.get("domains").and_then(Value::as_array).ok_or("a filter names its domains")?;
    words.iter().map(|w| w.as_str().and_then(Domain::parse)
        .ok_or_else(|| format!("{w} is not geography, ecology, society or polities"))).collect()
}

impl GraphSession {
    pub fn new(world: &World) -> Self {
        let timeline = Timeline::new(world).expect("a fresh world is readable");
        Self { disclosure: Disclosure::new(GRAPH, world), view: View::default(), timeline }
    }

    pub fn view(&self) -> &View { &self.view }
    pub fn timeline(&self) -> &Timeline { &self.timeline }

    fn world_actions(&self) -> Vec<AdvertisedAction> {
        let mut actions = vec![
            action(STEP, "Advance", "Advance the played line by the requested ticks.",
                r#"{"type":"object","properties":{"ticks":{"type":"integer","minimum":1}}}"#,
                IntentEffect::DomainTruth),
            action(SCRUB, "Scrub", "View a past tick of this line, replayed; the line is unchanged.",
                r#"{"type":"object","properties":{"tick":{"type":"integer","minimum":0}},"required":["tick"]}"#,
                IntentEffect::Curation),
            action(BRANCH, "Branch", "Fork a branch at the viewed tick and play it.", NULL,
                IntentEffect::DomainTruth),
            action(FILTER, "Filter", "Keep only these domains: geography, ecology, society, polities.",
                r#"{"type":"object","properties":{"domains":{"type":"array","items":{"type":"string"}}},"required":["domains"]}"#,
                IntentEffect::Curation),
        ];
        if self.timeline.lines() > 1 {
            actions.push(action(LINE, "Switch line", "Play another line at its head (0 is the main line).",
                r#"{"type":"object","properties":{"line":{"type":"integer","minimum":0}},"required":["line"]}"#,
                IntentEffect::Curation));
        }
        actions
    }

    /// The acts offered on a group: inside a branch, at its head (case 1048).
    fn acts(&self) -> Vec<AdvertisedAction> {
        if self.timeline.line() == 0 || !self.timeline.at_head() { return Vec::new(); }
        vec![action(DEATH, "Death", "A death inside this branch.", NULL, IntentEffect::DomainTruth),
            action(BIRTH, "Birth", "A birth to this group inside this branch.", NULL, IntentEffect::DomainTruth)]
    }

    pub fn snapshot(&mut self, request: ProjectionRequest) -> Result<ProjectionSnapshot, String> {
        let head = self.timeline.head()?;
        let (line, from) = (self.timeline.line(), self.timeline.forked_from());
        let (view, acts, actions) = (self.view.clone(), self.acts(), self.world_actions());
        let mut built = self.timeline.read(|s| {
            let mut world = vec![value("Tick", s.sim.state().tick), value("Head", head), value("Line", line),
                value("Entries", s.entries.len()), value("State hash", format!("{:016x}", s.sim.state_hash())),
                value("Filters", view.domains.iter().map(|d| d.word()).collect::<Vec<_>>().join(", "))];
            if let Some((parent, tick)) = from { world.push(value("Forked from", format!("line {parent} at tick {tick}"))); }
            let mut g = graph(s, &view, &acts);
            g.items.insert(0, Item { id: "world".into(), card: card("World", world, &["world"]), actions });
            g
        })?;
        built.links.retain(|(a, b, _)| a != b);
        self.disclosure.snapshot_folded(request, ADAPTER, &built.items, &built.links, &built.folds)
    }

    pub fn resource(&mut self, request: ResourceRequest) -> Result<ResourceResponse, String> {
        self.disclosure.resource(request)
    }

    pub fn invoke(&mut self, intent: IntentInvocation) -> Result<IntentResult, String> {
        let source = match self.disclosure.admit(&intent)? {
            Ok(source) => source,
            Err(refused) => return Ok(refused),
        };
        let first = || source.strip_prefix("group:").and_then(|n| n.parse::<u64>().ok());
        let done = match intent.intent.as_str() {
            OPEN => { self.view.open(&source); Ok(()) },
            CLOSE => { self.view.close(&source); Ok(()) },
            FILTER => domains(&intent.payload).map(|d| self.view.domains = d.into_iter().collect()),
            STEP => ticks(&intent.payload).and_then(|n| self.timeline.advance(n)),
            SCRUB => number(&intent.payload, "tick").and_then(|t| self.timeline.scrub(t)),
            BRANCH => self.timeline.branch(),
            LINE => number(&intent.payload, "line").and_then(|l| self.timeline.switch(l as usize)),
            DEATH => first().ok_or("only a group dies".into())
                .and_then(|entity| self.timeline.change(Command::Kill { entity })).map(|_| ()),
            BIRTH => first().ok_or("only a group gives birth".into())
                .and_then(|parent| self.timeline.change(Command::ForceBirth { parent })).map(|_| ()),
            _ => Err("unknown world-graph intent".into()),
        };
        Ok(match done {
            Ok(()) => self.disclosure.accepted(),
            Err(why) => rejected(why),
        })
    }
}
