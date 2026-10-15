// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Recorded envelope traces. Recording drives a scripted scenario and keeps
//! every action with what it came to; the check replays the actions on a
//! fresh world and requires the same observations and final hash. Rerecord
//! with `MESOCOSM_RECORD_TRACE=1` only when the sim's behaviour is meant to
//! change, and say why in the commit.

use isocosm_overlay::{AttentionChange, EntityHandle, Intent, PlaceHandle,
    mesocosm::{DevIntent, MesocosmIntent, PlayerAct, PlayerActKind}};
use mesocosm_runtime::{Occasion, Refusal, Runtime, runtime::Envelope};
use serde::{Deserialize, Serialize};

use super::worlds::{envelope, keep_parent, nudge};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Action { Queue(Envelope), Step(u64), Advance(u64) }

/// One action and what the runtime showed after it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Observed { pub action: Action, pub taken: Option<u64>, pub tick: u64, pub queued: usize, pub held: bool }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Recorded {
    pub world: String,
    pub actions: Vec<Observed>,
    pub outcomes: Vec<Result<String, Refusal>>,
    pub tick: u64,
    pub entries: u64,
    pub state_hash: u64,
}

fn act(runtime: &mut Runtime, action: Action) -> Observed {
    let taken = match &action {
        Action::Queue(queued) => { runtime.queue(queued.clone()); None },
        Action::Step(steps) => Some(runtime.step(*steps)),
        Action::Advance(elapsed) => Some(runtime.advance(*elapsed)),
    };
    Observed { action, taken, tick: runtime.tick(), queued: runtime.queued_len(), held: runtime.checkpoint().is_some() }
}

fn finish(world: &str, runtime: &Runtime, actions: Vec<Observed>) -> Recorded {
    let receipt = runtime.receipt();
    Recorded { world: world.into(), actions, outcomes: runtime.trace().iter().map(|(_, o)| o.clone()).collect(),
        tick: receipt.tick, entries: receipt.entries, state_hash: receipt.state_hash }
}

fn script(world: &str, mut runtime: Runtime) -> Recorded {
    let mut log = Vec::new();
    let critter = runtime.critter().expect("a played critter");
    let queue = |runtime: &mut Runtime, log: &mut Vec<Observed>, intent| {
        let queued = envelope(runtime, intent);
        log.push(act(runtime, Action::Queue(queued)));
    };
    match world {
        "offered" => {
            queue(&mut runtime, &mut log, nudge(critter));
            log.push(act(&mut runtime, Action::Step(1)));
            log.push(act(&mut runtime, Action::Advance(500_000)));
            log.push(act(&mut runtime, Action::Advance(0)));
            queue(&mut runtime, &mut log, nudge(critter + 1000));
            queue(&mut runtime, &mut log, Intent::Attention(AttentionChange::Examine(PlaceHandle(0))));
            log.push(act(&mut runtime, Action::Step(1)));
            queue(&mut runtime, &mut log, Intent::Game(MesocosmIntent::Act(PlayerAct { entity: EntityHandle(critter),
                kind: PlayerActKind::Speciate { name: "pin".into() } })));
            log.push(act(&mut runtime, Action::Step(1)));
            queue(&mut runtime, &mut log, Intent::Game(MesocosmIntent::Dev(DevIntent::PlaceMatter {
                site: PlaceHandle(0), mass_mg: 1_000 })));
            log.push(act(&mut runtime, Action::Step(3)));
            for elapsed in [1_000_000, 0, 0, 0] { log.push(act(&mut runtime, Action::Advance(elapsed))); }
            log.push(act(&mut runtime, Action::Step(20)));
        },
        "held" => {
            // Answer every birth by keeping the parent, short of the six-tick epoch's review.
            while runtime.tick() < 5 {
                log.push(act(&mut runtime, Action::Step(1)));
                if let Some(Occasion::Birth(birth)) = runtime.checkpoint().map(|c| c.occasion.clone()) {
                    queue(&mut runtime, &mut log, keep_parent(birth.parent));
                    log.push(act(&mut runtime, Action::Advance(0)));
                }
                assert!(log.len() < 60, "the held scenario stalled");
            }
            let played = runtime.critter().unwrap();
            queue(&mut runtime, &mut log, nudge(played));
            log.push(act(&mut runtime, Action::Step(1)));
        },
        _ => unreachable!("unknown pinned world"),
    }
    finish(world, &runtime, log)
}

fn replay(recorded: &Recorded, mut runtime: Runtime) -> Recorded {
    let actions = recorded.actions.iter().map(|observed| act(&mut runtime, observed.action.clone())).collect();
    finish(&recorded.world, &runtime, actions)
}

pub fn check(name: &str, world: fn() -> Runtime) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("tests/fixtures/runtime_trace_{name}.json"));
    if std::env::var_os("MESOCOSM_RECORD_TRACE").is_some() {
        let recorded = script(name, world());
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, serde_json::to_vec_pretty(&recorded).unwrap()).unwrap();
    }
    let recorded: Recorded = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    assert!(recorded.actions.iter().any(|o| matches!(o.action, Action::Queue(_))), "{name} records envelopes");
    assert_eq!(replay(&recorded, world()), recorded, "{name} replays to its recording");
}
