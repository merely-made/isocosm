// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A deposit (D47, D48) end to end: a body pays a kind of matter from its
//! reserve into the account its place holds, the place lets it decay to the
//! soil, and the budget stays closed with the totals adding up.

use super::{deposit::{Payer, install}, detritus_tests::founding};
use crate::{Execution, Session, flows::MadeBy, rules::Grammar, simulation::Genesis};

const SLIME: &str = "deposit:slime";
const RATE: u64 = 2;
const DECAY: u64 = 1;

/// The world with lineage 3's members at the first site, laying slime.
fn world() -> (Genesis, String) {
    let mut g = founding(None).generate().unwrap();
    let site = *g.sites.keys().next().unwrap();
    for group in g.population.groups.values_mut().filter(|g| g.entity.lineage == "lineage:3") {
        group.entity_mut().place = site;
    }
    // The A1 world's accounts are matter:i-0 and matter:i-1; the first stands for a reserve.
    let reserve = "matter:3-0".to_string();
    let payer = Payer { marked: "ability:cycle-3", reserve: &reserve };
    install(&mut g, &payer, SLIME, RATE, DECAY).unwrap();
    (g, reserve)
}

fn slime(s: &Session) -> u64 {
    s.sim.state().sites.values().map(|site| site.accounts.get(SLIME).copied().unwrap_or(0)).sum()
}

#[test]
fn a_body_pays_a_deposit_from_its_reserve_and_the_place_lets_it_decay() {
    let (g, reserve) = world();
    assert!(g.population.groups.values().any(|g| g.entity.lineage == "lineage:3" && g.entity.traits.contains("ability:cycle-3")));
    let mut s = Session::new(g, Execution::Individuals).unwrap();
    let total = s.sim.budget().total();
    let (mut paid, mut decayed) = (0, 0);
    for _ in 0..30 {
        let r = s.advance_tick_with_flows().unwrap();
        for f in &r.flows {
            let MadeBy::Process(p) = &f.made_by else { continue };
            let n = f.amount * f.count;
            match (p.starts_with("deposit:ability"), p.starts_with("deposit:decay")) {
                (true, _) if f.to.1 == SLIME => paid += n,
                (_, true) if f.from.1 == SLIME => decayed += n,
                _ => {},
            }
        }
        assert_eq!(s.sim.budget().total(), total, "matter moved, never made");
    }
    let _ = reserve;
    assert!(paid > 0, "the body paid slime");
    assert!(decayed > 0, "the place let it decay");
    assert_eq!(slime(&s) + decayed, paid, "what is paid is held or decayed");
}

#[test]
fn a_deposit_of_nought_or_into_a_kind_that_is_taken_is_refused() {
    let (mut g, reserve) = world();
    let payer = Payer { marked: "ability:cycle-3", reserve: &reserve };
    install(&mut g, &payer, "deposit:wall", 0, 1).unwrap_err();
    install(&mut g, &payer, "deposit:wall", 1, 0).unwrap_err();
    install(&mut g, &payer, "matter:3-1", 1, 1).unwrap_err();
    let _: Option<Grammar> = None;
}
