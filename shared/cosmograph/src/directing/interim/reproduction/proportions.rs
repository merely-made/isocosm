// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! B2 uses A1's existing three-epoch measurement and qualification predicate.
//! Only the native founding's shape/proportion opt-in changes. This is a
//! site-grain ecology gate, separate from captured physical walking.
use super::*;

#[test]
#[ignore = "explicit 20-seed, three-epoch native ecology/replay receipt"]
fn varied_body_proportions_keep_the_a1_gate() {
    // The same bounded worker setting as the existing A1 matrix.
    let workers = std::env::var("ISOCOSM_REPRO_WORKERS")
        .map_or(2, |value| value.parse::<u64>().expect("worker count")).clamp(1, 20);
    let (send, receive) = mpsc::channel();
    let mut rows = std::thread::scope(|scope| {
        for worker in 0..workers {
            let send = send.clone();
            scope.spawn(move || {
                for seed in (worker..20).step_by(workers as usize) {
                    let mut f = Form::Played.founding(seed, crate::ecology::Rates::A1);
                    let b = f.bodies.as_mut().unwrap();
                    b.founding = Some("SpacedRoster".into());
                    b.proportions = true;
                    let g = profile::world_from_founding(f, false, None);
                    // Existing runner checks conservation every round and
                    // exact final-save replay in both execution modes.
                    let out = played(g, false);
                    eprintln!("BODY_PROPORTIONS {}", serde_json::json!({
                        "seed": seed, "qualified": out.qualifies(), "outcome": &out }));
                    send.send((seed, out)).unwrap();
                }
            });
        }
        drop(send);
        receive.into_iter().collect::<Vec<_>>()
    });
    rows.sort_by_key(|row|row.0);
    assert_eq!(rows.len(), 20);
    let qualified = rows.iter().filter(|(_, out)|out.qualifies()).count();
    eprintln!("BODY_PROPORTIONS_TOTAL {qualified}/20");
    assert_eq!(qualified, 20, "unchanged A1 three-epoch gate");
}
