// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Ordinary saved-world continuation through the same presses as the window.
use mesocosm_runtime::{Runtime, Context, Press, Named, Occasion};

pub fn run() {
    let bytes = std::fs::read(std::env::var("MESOCOSM_ACTIVE_SAVE").unwrap()).unwrap();
    let saved = Runtime::decode_world_save(&bytes).unwrap();
    let start_entries = saved.native.entries.len();
    let mut r = Runtime::resume_world(saved, 2, None).unwrap();
    let initial = r.critter();
    let count: u64 = std::env::var("MESOCOSM_PLAY_STEPS").unwrap().parse().unwrap();
    let mut reviews = 0;
    for _ in 0..count {
        let reading = r.loop_reading().unwrap();
        use mesocosm_runtime::continuation::{self, LoopPhase, LoopInput, LoopAction, LoopAnswer};
        if matches!(reading.phase, LoopPhase::LineageEnded | LoopPhase::WorldEnded) {
            // This is the same ordinary Watch choice as the host's lifecycle pane.
            let input = LoopInput { kind: continuation::KIND.into(), version: continuation::VERSION,
                participant: cosmograph_overlay::ParticipantHandle(r.interim().participant),
                tick: cosmograph_overlay::Tick(r.tick()), expected: reading.cut,
                action: LoopAction::Answer(LoopAnswer::Watch) };
            r.submit_loop_input(&input).unwrap();
        }
        if let Some(c) = r.checkpoint().cloned() {
            let press = match c.occasion {
                Occasion::Birth(_) if r.loop_reading().unwrap().checkpoint.is_some() => Press::Char('t'),
                _ => Press::Named(Named::Enter),
            };
            let context = Context { focus:r.critter(), row:0, dev:false };
            eprintln!("[play] tick={} checkpoint={c:?} intent={:?}", r.tick(), r.keyed(&press, &context));
            let claimed = r.press(&press, &context).unwrap();
            eprintln!("[play] claimed={claimed} queued={}", r.queued_len());
            r.advance(0);
            eprintln!("[play] answered played={:?} trace={:?}", r.critter(), r.trace().last());
            assert!(r.checkpoint().is_none(), "ordinary answer must release the hold");
            if matches!(c.occasion, Occasion::Epoch(_)) { reviews += 1; }
        }
        let before = r.tick();
        let steps = r.step(1);
        let sim = r.sim();
        let placement = r.critter().and_then(|id| r.spatial().members.get(&id))
            .map(|p| p.realized(&sim));
        eprintln!("[play] tick={} played={:?} placement={placement:?} fault={:?}", r.tick(), r.critter(),r.fault());
        if r.tick() % 5 == 0 { save(&r); }
        if steps == 0 || r.tick() == before || r.fault().is_some() {
            save(&r);
            panic!("ordinary play stopped");
        }
    }
    let mut depths = std::collections::BTreeMap::new();
    let game = r.spatial();
    if let Some(driver) = &game.driver {
        for (&id, child) in &driver.children {
            let depth = depths.get(&child.birth.parent).copied().unwrap_or(0u64) + 1;
            depths.insert(id, depth);
        }
    }
    // Native control can change automatically before the loss confirmation.
    // Count actual recorded takes, rather than just answer-time changes.
    let (mut handovers, mut offspring_handovers, mut cohort_handovers) = (0, 0, 0);
    let mut previous = initial;
    let participant = r.interim().participant;
    for entry in r.session().entries.iter().skip(start_entries) {
        if let cosmograph::Command::Take { participant: owner, critter } = &entry.command
            && *owner == participant && entry.outcome == "taken" && previous != Some(*critter) {
            handovers += 1;
            if previous.is_some_and(|parent| game.driver.as_ref().and_then(|d| d.children.get(critter))
                .is_some_and(|child| child.birth.parent == parent)) {
                offspring_handovers += 1;
            } else { cohort_handovers += 1; }
            previous = Some(*critter);
        }
    }
    let generations = depths.values().copied().max().unwrap_or(0);
    let required: u64 = std::env::var("MESOCOSM_PLAY_MIN_GENERATIONS")
        .ok().map(|s| s.parse().unwrap()).unwrap_or(0);
    save(&r);
    let saved = r.save_world().unwrap();
    let restored = Runtime::resume_world(saved.clone(), 2, None).unwrap();
    assert_eq!(restored.state_hash(), r.state_hash());
    assert_eq!(restored.save_world().unwrap().sections, saved.sections);
    let summary = serde_json::json!({"tick":r.tick(), "hash":format!("{:016x}",r.state_hash()),
        "control_changes":handovers, "biological_generations":generations,
        "offspring_handovers":offspring_handovers, "cohort_handovers":cohort_handovers,
        "epoch_reviews":reviews, "phase":r.current_loop_phase(),
        "born_identities":depths.len(), "dev_intents":r.dev_intents(), "save_reload_equal":true});
    eprintln!("[play] completed {summary}");
    if let Ok(path) = std::env::var("MESOCOSM_PLAY_OUT") {
        std::fs::write(std::path::Path::new(&path).with_extension("summary.json"),
            serde_json::to_vec_pretty(&summary).unwrap()).unwrap();
    }
    assert!(generations >= required, "ordinary world did not reach the required biological generations");
}

fn save(r: &Runtime) {
    if let Ok(path) = std::env::var("MESOCOSM_PLAY_OUT") {
        std::fs::write(path, serde_json::to_vec(&r.save_world().unwrap()).unwrap()).unwrap();
    }
}
