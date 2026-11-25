// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use cosmograph::flows::MadeBy;

#[test]
fn ordinary_world_finds_exchange_partners() {
    let mut session =
        Session::new(Panel::starter(0).generate().unwrap(), Execution::Grouped).unwrap();
    for _ in 0..40 {
        let tick = session.advance_tick_with_flows().unwrap();
        if let Some(flow) = tick.flows.iter().find(|flow| {
            matches!(&flow.made_by, MadeBy::Process(p) if p.starts_with("mutualism:exchange-"))
                && flow.from.1 == "world:exchange-sugar"
                && flow.to.1 == flow.from.1
                && !flow.internal()
        }) {
            let buyer = flow.from.0.body().unwrap();
            let supplier = flow.to.0.body().unwrap();
            assert_ne!(buyer, supplier);
            assert!(flow.amount > 0);
            return;
        }
    }
    panic!(
        "ordinary New World seed 0 produced no nutrient-for-sugar exchange in its first 40 ticks"
    );
}
