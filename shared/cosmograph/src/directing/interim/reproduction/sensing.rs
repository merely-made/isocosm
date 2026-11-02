// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Case 1071 under A1's three-epoch measurement and qualification predicate:
//! the played and bodied candidate forms with every form sensing and without.
//! Only `Bodies::sensing` changes; site-grain ecology, no receipts written.
//! On 2026-11-01 each draw's save failed its replay on main before 1071
//! (the take-up refusal in the sim execution plan's Findings); measured with replay
//! set aside, all 40 pairs ran identical histories.
use super::*;

#[test]
#[ignore = "explicit 80-draw sensing comparison over the A1 three-epoch gate"]
fn sensing_keeps_the_a1_gate() {
    let workers = std::env::var("COSMOGRAPH_REPRO_WORKERS")
        .map_or(2, |value| value.parse::<u64>().expect("worker count")).clamp(1, 20);
    let mut totals = Vec::new();
    for form in [Form::Played, Form::Bodied] {
        for sensing in [false, true] {
            let (send, receive) = mpsc::channel();
            std::thread::scope(|scope| {
                for worker in 0..workers {
                    let send = send.clone();
                    scope.spawn(move || {
                        for seed in (worker..20).step_by(workers as usize) {
                            let mut f = form.founding(seed, crate::ecology::Rates::A1);
                            f.bodies.as_mut().unwrap().sensing = sensing;
                            let out = played(profile::world_from_founding(f, false, None), false);
                            eprintln!("SENSING {}", serde_json::json!({ "form": form.name(), "sensing": sensing,
                                "seed": seed, "qualified": out.qualifies(), "outcome": &out }));
                            send.send(out.qualifies()).unwrap();
                        }
                    });
                }
            });
            drop(send);
            let qualified = receive.into_iter().filter(|q| *q).count();
            eprintln!("SENSING_TOTAL form={} sensing={sensing}: {qualified}/20", form.name());
            totals.push((form, sensing, qualified));
        }
    }
    // Sensing may not cost a form a draw that qualifies without it.
    for pair in totals.chunks(2) {
        let [(form, _, off), (_, _, on)] = [pair[0], pair[1]];
        assert!(on >= off, "A1 gate, form={}: {on}/20 sensing against {off}/20 without", form.name());
    }
}
