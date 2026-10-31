// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The bench world with processes whose accepted acts adopt, serve and end goals.
use super::*;
use cosmograph::{goals::{self, Directive, Ending}, simulation::Receipt};

pub const HEARTH: &str = "goal:hearth";
pub const ROOF: &str = "goal:roof";
pub const ADOPT_HEARTH: &str = "test:adopt-hearth";
pub const ADOPT_ROOF: &str = "test:adopt-roof";
pub const SERVE_BOTH: &str = "test:serve-both";
pub const SERVE_LATE: &str = "test:serve-late";
pub const END_HEARTH: &str = "test:end-hearth";
pub const IDLE: &str = "test:idle";

fn directed(id: &str, directives: Vec<Directive>) -> Process {
    let mut p = process(id, vec![], vec![]);
    p.goal_directives = directives;
    p
}

pub fn goal_world(seed: u64) -> World {
    let mut w = world(seed);
    let g = &mut w.genesis;
    g.rules.note_kinds.extend(goals::NOTE_KINDS.iter().map(|kind| kind.to_string()));
    let both = BTreeSet::from([HEARTH.to_string(), ROOF.to_string()]);
    for p in [
        directed(ADOPT_HEARTH, vec![Directive::Adopt { goal: HEARTH.into(), text: "keep a hearth".into() }]),
        directed(ADOPT_ROOF, vec![Directive::Adopt { goal: ROOF.into(), text: "raise a roof".into() }]),
        directed(SERVE_BOTH, vec![Directive::Participate { goals: both }]),
        directed(SERVE_LATE, vec![Directive::Participate { goals: BTreeSet::from([HEARTH.to_string()]) }]),
        directed(END_HEARTH, vec![Directive::End { goal: HEARTH.into(), ending: Ending::Met }]),
        directed(IDLE, vec![]),
    ] { g.rules.processes.insert(p.id.clone(), p); }
    g.validate().unwrap();
    w
}

pub fn receipt(result: cosmograph::Result<String>) -> Receipt { serde_json::from_str(&result.unwrap()).unwrap() }

/// One native act by `actor`; its receipt id is the event a goal record names.
pub fn perform(s: &mut Session, actor: Id, process: &str) -> Receipt {
    receipt(s.command(Command::Act { actor, target: None, process: process.into(), cause: None }))
}

pub fn goal(s: &Session, key: &str) -> goals::Thread { s.sim.goal_thread(key).unwrap() }

pub fn record(change: goals::Change) -> Command { Command::Goal(change) }
