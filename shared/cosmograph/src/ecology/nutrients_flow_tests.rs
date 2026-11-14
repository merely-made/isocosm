// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Nutrient axes at work: a producer fixes only as far as its scarcest
//! nutrient allows, a producer short of one declines by upkeep and the
//! existing death, what returns to the ground is sorted in the proportion it
//! was drawn with the budget closed, mutualism and detritus keep working with
//! the rule set, and both executions agree.

use super::mutualism_flow_tests::{BOTH, equip, together};
use super::nutrients_tests::founding;
use super::*;
use crate::{
    Execution, Session,
    flows::{Holder, MadeBy},
    process::Trade,
    schema::Id,
    simulation::Genesis,
};
use std::collections::BTreeMap;

fn world(rule: Nutrients) -> Genesis {
    founding(Some(rule)).generate().unwrap()
}

/// Every site's nutrient `a` set to `amount`.
fn set_all(mut g: Genesis, a: u32, amount: u64) -> Genesis {
    for site in g.sites.values_mut() {
        site.accounts.insert(axis(a), amount);
    }
    g
}

#[derive(Default)]
struct Seen {
    /// What each producer line's acts took of each nutrient (light is not matter, so is not counted).
    drawn: BTreeMap<(String, String), u64>,
    /// What the sorting moved into each other nutrient.
    sorted: BTreeMap<String, u64>,
    closed: bool,
}

fn run(g: Genesis, ticks: u64, mode: Execution) -> (Seen, Session) {
    let mut s = Session::new(g, mode).unwrap();
    let total = s.sim.budget().total();
    let mut seen = Seen { closed: true, ..Seen::default() };
    for _ in 0..ticks {
        let r = s.advance_tick_with_flows().unwrap();
        for f in &r.flows {
            let MadeBy::Process(by) = &f.made_by else { continue };
            if by.starts_with("ecology:produce-") && matches!(f.from.0, Holder::Site(_)) && f.from.1.starts_with("world:soil") {
                *seen.drawn.entry((by.clone(), f.from.1.clone())).or_default() += f.amount * f.count;
            }
            if by.starts_with("nutrients:sort-") && f.to.1.starts_with("world:soil-") {
                *seen.sorted.entry(f.to.1.clone()).or_default() += f.amount * f.count;
            }
        }
        seen.closed &= s.sim.budget().total() == total;
    }
    (seen, s)
}

impl Seen {
    fn took(&self, line: u32, a: u32) -> u64 {
        self.drawn.get(&(format!("ecology:produce-{line}"), axis(a))).copied().unwrap_or(0)
    }
    fn fixed(&self) -> u64 {
        self.drawn.values().sum()
    }
}

fn alive(s: &Session, lines: &[&str]) -> u64 {
    s.sim.state().population.groups.values().filter(|g| g.entity.alive && lines.contains(&g.entity.lineage.as_str())).map(|g| g.count).sum()
}

#[test]
fn a_producer_fixes_only_as_far_as_its_scarcest_nutrient_allows() {
    let rich = Nutrients { spread: [100, 100], ..Nutrients::default() };
    let (plenty, _) = run(world(rich), 60, Execution::Individuals);
    assert!(plenty.fixed() > 0 && plenty.closed, "with both nutrients there is fixing");
    // Each act takes its line's proportion: two to one for line 0, one to two for line 3.
    assert_eq!(plenty.took(0, 0), 2 * plenty.took(0, 1));
    assert_eq!(2 * plenty.took(3, 0), plenty.took(3, 1));
    assert!(plenty.took(0, 1) > 0 && plenty.took(3, 1) > 0, "both lines fix");
    // The same world with the second nutrient gone: no act is possible for either line, whatever the soil holds.
    let (short, _) = run(set_all(world(rich), 1, 0), 60, Execution::Individuals);
    assert_eq!(short.fixed(), 0, "no nutrient, no fixing: {:?}", short.drawn);
    assert!(short.closed);
    // A little of it bounds the acts to what it covers and what the sorting returns of it.
    let (little, _) = run(set_all(world(rich), 1, 6), 60, Execution::Individuals);
    let taken = little.took(0, 1) + little.took(3, 1);
    let returned = little.sorted.get("world:soil-1").copied().unwrap_or(0);
    assert!(taken > 0, "some fixing");
    assert!(taken <= 36 + returned, "{taken} taken against 36 and {returned} returned");
    assert!(little.fixed() < plenty.fixed(), "scarcity cut the fixing");
}

#[test]
fn a_producer_short_of_an_essential_nutrient_declines_by_upkeep_and_the_existing_death() {
    let rich = Nutrients { spread: [100, 100], ..Nutrients::default() };
    let plants = ["lineage:0", "lineage:3"];
    // Old age and the hazard are taken away, so the only way out of life is the founded death below one unit.
    let mortal = |g: Genesis| {
        let mut g = g;
        g.rules.processes.retain(|k, _| !k.starts_with("ecology:age-") && !k.starts_with("ecology:hazard-"));
        g
    };
    let (_, fed) = run(mortal(world(rich)), 150, Execution::Individuals);
    let (starved, s) = run(mortal(set_all(world(rich), 1, 0)), 150, Execution::Individuals);
    assert!(alive(&fed, &plants) > 0, "fed plants live");
    assert_eq!(alive(&s, &plants), 0, "plants with no second nutrient are gone, by upkeep and the death below one unit");
    assert!(starved.closed && starved.fixed() == 0);
    // No process of dying was added.
    let g = world(rich);
    assert!(!g.rules.processes.keys().any(|k| k.starts_with("nutrients:") && k.contains("death")));
}

#[test]
fn what_returns_is_sorted_in_the_proportion_it_was_drawn_and_the_budget_stays_closed() {
    let (seen, s) = run(world(Nutrients::default()), 200, Execution::Individuals);
    assert!(seen.closed, "the budget stayed closed");
    let (drawn, sorted) = (seen.took(0, 1) + seen.took(3, 1), seen.sorted.get("world:soil-1").copied().unwrap_or(0));
    assert!(drawn > 0 && sorted > 0, "drawn {drawn}, sorted {sorted}");
    // What the sites still owe is what was drawn and not yet sorted.
    let owed: i64 = s.sim.state().sites.values().map(|site| site.conditions.get("nutrients:owed-1").copied().unwrap_or(0)).sum();
    assert!(owed >= 0);
    assert_eq!(drawn, sorted + u64::try_from(owed).unwrap(), "sorted plus owed is drawn");
    // And the total of all the nutrients is only what the ground and the bodies held.
    assert!(s.sim.state().sites.values().all(|site| site.accounts.keys().filter(|k| k.starts_with("world:soil")).count() == 2));
}

#[test]
fn a_nutrient_a_line_needs_nought_of_does_not_limit_it() {
    let sole = Nutrients { base: 0, tilt: 2, spread: [100, 100], ..Nutrients::default() };
    let g = set_all(world(sole), 1, 0);
    // Line 0 leads on the first nutrient and needs none of the second; line 3 needs the second only.
    let (seen, _) = run(g, 60, Execution::Individuals);
    assert!(seen.took(0, 0) > 0 && seen.took(0, 1) == 0, "line 0 fixes with no second nutrient");
    assert_eq!(seen.took(3, 0) + seen.took(3, 1), 0, "line 3 cannot");
}

#[test]
fn the_rule_is_the_same_in_both_executions() {
    let hash = |mode| {
        let mut s = Session::new(world(Nutrients::default()), mode).unwrap();
        s.advance(80).unwrap();
        s.sim.state_hash()
    };
    assert_eq!(hash(Execution::Individuals), hash(Execution::Grouped));
    let scarce = |mode| {
        let mut s = Session::new(set_all(world(Nutrients::default()), 1, 12), mode).unwrap();
        s.advance(80).unwrap();
        s.sim.state_hash()
    };
    assert_eq!(scarce(Execution::Individuals), scarce(Execution::Grouped));
}

#[test]
fn the_budget_is_closed_in_both_executions_and_they_agree_on_the_sorting() {
    let tally = |mode| {
        let (seen, s) = run(world(Nutrients::default()), 120, mode);
        let owed: i64 = s.sim.state().sites.values().map(|site| site.conditions.get("nutrients:owed-1").copied().unwrap_or(0)).sum();
        (seen.closed, seen.sorted, owed)
    };
    let (a, b) = (tally(Execution::Individuals), tally(Execution::Grouped));
    assert!(a.0 && b.0);
    assert_eq!(a, b);
}

fn plants_and_fungi(rule: Option<Nutrients>, mutual: bool) -> Genesis {
    let mut f = founding(rule);
    f.mutualism = mutual.then(Mutualism::default);
    together(equip(equip(f.generate().unwrap(), "lineage:0", &BOTH), "lineage:2", &BOTH), )
}

fn sites(g: &Genesis) -> Vec<Id> {
    g.sites.keys().copied().collect()
}

#[test]
fn mutualism_carries_the_partners_need_and_works_with_the_rule_set() {
    let g = plants_and_fungi(Some(Nutrients::default()), true);
    // The carry moves each nutrient the partner's line needs, a pair of transfers each.
    let carry = &g.rules.processes["mutualism:nutrient-2-0"];
    let moved: Vec<(String, u64)> = carry.effects.iter().filter_map(|e| match e {
        Effect::Transfer { account, amount, from: Binding::Place, .. } => Some((account.clone(), amount.resolved().unwrap())),
        _ => None,
    }).collect();
    assert_eq!(moved, [("world:soil".to_string(), 4), ("world:soil-1".to_string(), 2)], "two acts of line 0's two to one");
    let grow = &g.rules.processes["mutualism:grow-0"];
    assert!(!grow.effects.iter().any(|e| matches!(e, Effect::Transfer { account, .. } if account != "sim:energy")), "it fixes with what it was given");
    assert!(sites(&g).len() == 6);
    let mut carried = 0;
    let mut paid = 0;
    let mut s = Session::new(g, Execution::Individuals).unwrap();
    let total = s.sim.budget().total();
    for _ in 0..120 {
        let r = s.advance_tick_with_flows().unwrap();
        for f in &r.flows {
            match &f.made_by {
                MadeBy::Process(p) if p == "mutualism:nutrient-2-0" && !f.internal() && f.from.1 == "world:soil-1" => carried += f.amount * f.count,
                MadeBy::Process(p) if p == "mutualism:sugar-0-2" && !f.internal() => paid += f.amount * f.count,
                _ => {},
            }
        }
        assert_eq!(s.sim.budget().total(), total, "closed");
    }
    assert!(carried > 0 && paid > 0, "the second nutrient was carried ({carried}) and the sugar paid ({paid})");
}

#[test]
fn mutualism_unset_or_set_the_unrelated_rule_changes_nothing_about_it() {
    // Mutualism without the nutrient rule is the founded trade: the same turns, no second nutrient.
    let g = plants_and_fungi(None, true);
    assert!(g.rules.processes["mutualism:nutrient-2-0"].effects.iter().all(|e| !matches!(e, Effect::Transfer { account, .. } if account.starts_with("world:soil-"))));
    let hash = |mode| {
        let mut s = Session::new(plants_and_fungi(Some(Nutrients::default()), true), mode).unwrap();
        s.advance(60).unwrap();
        s.sim.state_hash()
    };
    assert_eq!(hash(Execution::Individuals), hash(Execution::Grouped));
}

#[test]
fn detritus_keeps_working_with_the_rule_set_and_the_budget_stays_closed() {
    let with = |nutrients: Option<Nutrients>| {
        let mut f = founding(nutrients);
        f.web = Web::Diets;
        f.lineages = 5;
        f.categories = Some(Categories::default());
        f.detritus = Some(Detritus::default());
        let g = together_all(f.generate().unwrap());
        let mut s = Session::new(g, Execution::Individuals).unwrap();
        let total = s.sim.budget().total();
        let (mut dung, mut closed) = (0, true);
        for _ in 0..80 {
            let r = s.advance_tick_with_flows().unwrap();
            dung += r.flows.iter().filter(|f| matches!(&f.made_by, MadeBy::Process(p) if p.starts_with("detritus:")) && matches!(f.to.0, Holder::Site(_))).map(|f| f.amount * f.count).sum::<u64>();
            closed &= s.sim.budget().total() == total;
        }
        (dung, closed)
    };
    let (off, on) = (with(None), with(Some(Nutrients::default())));
    assert!(off.0 > 0 && off.1, "dung moves without the rule");
    assert!(on.0 > 0 && on.1, "dung moves with it, the budget closed");
}

fn together_all(mut g: Genesis) -> Genesis {
    let site = *g.sites.keys().next().unwrap();
    for group in g.population.groups.values_mut() {
        group.entity_mut().place = site;
    }
    g
}

#[test]
fn a_tuned_organ_draws_the_nutrient_the_site_holds_least_of_and_a_body_without_one_draws_none() {
    let rich = Nutrients { spread: [100, 100], ..Nutrients::default() };
    // The first nutrient plentiful and the second short at every site, so the second is the scarcest.
    let tally = |organ: bool| {
        let g = set_all(set_all(world(rich), 0, 800), 1, 40);
        let g = if organ { equip(g, "lineage:0", &[Trade::Draws]) } else { g };
        let mut s = Session::new(g, Execution::Individuals).unwrap();
        let total = s.sim.budget().total();
        let mut by_account: BTreeMap<String, u64> = BTreeMap::new();
        for _ in 0..60 {
            let r = s.advance_tick_with_flows().unwrap();
            for f in &r.flows {
                if matches!(&f.made_by, MadeBy::Process(p) if p.starts_with("nutrients:tuned-0-")) && matches!(f.from.0, Holder::Site(_)) {
                    *by_account.entry(f.from.1.clone()).or_default() += f.amount * f.count;
                }
            }
            assert_eq!(s.sim.budget().total(), total, "closed");
        }
        by_account
    };
    let drew = tally(true);
    assert!(drew.get("world:soil-1").copied().unwrap_or(0) > 0, "{drew:?}");
    assert!(drew.get("world:soil").is_none(), "never the plentiful one: {drew:?}");
    assert!(tally(false).is_empty(), "no organ, no draw");
}
