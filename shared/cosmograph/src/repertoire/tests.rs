// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The repertoire as a founding rule and a way for lines to reach the
//! catalog (D37): unset it changes nothing; set it lowers parts with ports
//! consistent with their rows; a review offer and a boundary candidate carry
//! a lowered kind; a mouth grown that way bites flesh, and harms or not by
//! its port; and no line is founded with one.

use super::*;
use crate::{
    Execution, Session,
    bodied::Bodies,
    catalog::catalog,
    development::{develop, soma},
    directing::Played,
    ecology::{Flesh, Spoilage, Web},
    flows::MadeBy,
    history::Command,
    lineage::{boundary, revise},
    process::{NisKind, Tolerance},
};

fn founding(repertoire: Option<Repertoire>, spoilage: bool) -> Founding {
    let grid = crate::map::Grid { width: 3, height: 2, ..crate::map::Grid::drawn(5) };
    Founding {
        seed: 5,
        population: 256,
        sites: grid.width * grid.height,
        map: Some(crate::map::Layout::Grid(grid)),
        ecology: true,
        ecology_rates: Rates::A1,
        bodies: Some(Bodies::default()),
        played: Some(Played { lineage: 1, region_sites: 2 }),
        web: Web::Diets,
        lineages: 5,
        spoilage: spoilage.then(Spoilage::default),
        repertoire,
        ..Default::default()
    }
}

fn all() -> Option<Repertoire> {
    Some(Repertoire::default())
}

const JAW: &str = "kind:catalog-fauna-jaw-take-prey";
const SPECIALIST: &str = "kind:catalog-fauna-jaw-eat-decomposer";
const ORGAN: &str = "kind:catalog-detox-gut";

#[test]
fn unset_founds_and_hashes_as_before() {
    let plain = founding(None, false);
    let json = serde_json::to_string(&plain).unwrap();
    assert!(!json.contains("repertoire"));
    let g = plain.generate().unwrap();
    assert!(g.rules.repertoire.is_none());
    assert!(!serde_json::to_string(&g.rules).unwrap().contains("repertoire"));
    assert!(!g.rules.kinds.keys().any(|k| k.starts_with("kind:catalog-")));
    let bare: Founding = serde_json::from_str(&json).unwrap();
    assert_eq!(crate::digest(&bare.generate().unwrap()), crate::digest(&g));
}

#[test]
fn it_round_trips_and_bad_values_are_named() {
    let f = founding(Some(Repertoire { only: BTreeSet::from([JAW.replace("kind:catalog-", "part:")]), across_kingdoms: true }), false);
    let json = serde_json::to_string(&f).unwrap();
    assert_eq!(serde_json::from_str::<Founding>(&json).unwrap(), f);
    assert!(serde_json::to_string(&founding(all(), false)).unwrap().contains("\"repertoire\":{}"));
    let bad = |only: &str| {
        founding(Some(Repertoire { only: BTreeSet::from([only.to_string()]), ..Default::default() }), false).generate().unwrap_err()
    };
    assert!(bad("part:no-such-part").contains("catalog lacks"));
    let waiting = catalog().parts().iter().find(|p| !matches!(p.status, crate::catalog::Status::Expressible { .. })).unwrap();
    assert!(bad(&waiting.key).contains("not runnable"));
    let leaf = catalog().parts().iter().find(|p| p.process == "mesocosm:fix" && matches!(p.status, crate::catalog::Status::Expressible { .. }) && !needs_categories(p)).unwrap();
    assert!(bad(&leaf.key).contains("intake port can carry"));
    let world_time = Founding { ecology_rates: Rates::WorldTime, ..founding(all(), false) };
    assert!(world_time.generate().unwrap_err().contains("world time"));
    let bare = Founding { bodies: None, ..founding(all(), false) };
    assert!(bare.generate().unwrap_err().contains("bodied"));
    let no_ecology = Founding { ecology: false, ..founding(all(), false) };
    assert!(no_ecology.generate().is_err());
}

#[test]
fn the_world_declares_kinds_whose_ports_agree_with_their_rows() {
    let g = founding(all(), false).generate().unwrap();
    let lowered = g.rules.repertoire.as_ref().unwrap();
    assert!(lowered.kinds.len() >= 15, "{} kinds", lowered.kinds.len());
    for (key, e) in lowered.kinds.iter().filter(|(_, e)| e.root_of.is_none()) {
        let part = catalog().part_by_key(&e.part).unwrap();
        let t = &g.rules.kinds[key];
        let port = t.port.expect("every lowered kind has a port");
        assert_eq!(e.function, format!("function:{}", crate::catalog::refs::process_name(&part.process)));
        assert_eq!(crate::mosaic::ports::supporting(port).as_deref(), Some(e.function.as_str()));
        assert_eq!(t.cells.keys().collect::<Vec<_>>(), [&e.function], "{key}: every cell to its process");
        assert!(t.cells.values().sum::<u32>() <= crate::anatomy::capacity(t.half_extent));
        assert_eq!(port.admits_live(NisKind::Consumer), part.admits.iter().any(|a| a == "live:consumer"), "{key}");
        assert_eq!(port.admits_live(NisKind::Producer), part.admits.iter().any(|a| a == "live:producer"), "{key}");
        assert_eq!(port.admits_live(NisKind::Decomposer), part.admits.iter().any(|a| a == "live:decomposer"), "{key}");
        assert_eq!(port.admits_deadstock(), part.admits.iter().any(|a| a.starts_with("dead:")) && e.mouth, "{key}");
        assert_eq!(e.kingdom, part.kingdom);
        if let Some(root) = &e.root {
            assert!(g.rules.kinds[root].cells.contains_key("function:reproduce"));
        }
    }
    let jaw = g.rules.kinds[JAW].port.unwrap();
    assert!(jaw.admits_live(NisKind::Consumer) && !jaw.admits_live(NisKind::Producer));
    let detox = g.rules.kinds[ORGAN].port.unwrap();
    assert!(!detox.has_live() && detox.tolerates(Tolerance::DecomposerFlesh) && detox.tolerates(Tolerance::ToxicFlesh));
    assert!(g.rules.validate().is_ok());
    // No line is founded with one (D37).
    let lowered: BTreeSet<_> = lowered.kinds.keys().collect();
    assert!(g.lineages.values().filter_map(|l| l.development.as_ref()).all(|d| d.lexicon.iter().all(|k| !lowered.contains(k))));
    assert!(g.population.groups.values().flat_map(|g| g.entity.parts.values()).all(|p| p.port.is_none_or(|p| p.tolerance_bits() == 0)));
}

fn session(g: Genesis) -> Session {
    Session::new(g, Execution::Individuals).unwrap()
}

/// The revision of `lineage` named `name` that `g`'s world offers.
fn offered(g: &Genesis, lineage: &str, name: &str) -> crate::rules::Development {
    let s = session(g.clone());
    let d = s.sim.state().lineages[lineage].development.clone().unwrap();
    let all = revise::offered(&s, lineage, &d, usize::MAX);
    all.into_iter().find(|(n, _)| n == name).unwrap_or_else(|| panic!("{name} is not offered")).1
}

/// `g` with `lineage` developing from `d` and its founders grown anew from it.
fn grown(mut g: Genesis, lineage: &str, d: crate::rules::Development) -> Genesis {
    let (rules, seed) = (g.rules.clone(), g.seed);
    g.lineages.get_mut(lineage).unwrap().development = Some(d.clone());
    for (first, group) in g.population.groups.iter_mut().filter(|(_, g)| g.entity.lineage == lineage) {
        let e = group.entity_mut();
        let books = crate::anatomy::books(e);
        let drawn = soma(&rules, &d.recipe, crate::draw(seed, "founder-soma", &[*first]));
        e.embody(develop(&rules, &d, &drawn).unwrap());
        e.absent = vec![];
        e.soma = drawn.segments;
        e.accounts.retain(|k, _| !matches!(rules.accounts.get(k), Some(crate::rules::AccountKind::Matter { lineage: l, .. }) if l == lineage));
        for (k, held) in books.into_iter().filter(|(k, _)| matches!(rules.accounts.get(k), Some(crate::rules::AccountKind::Matter { lineage: l, .. }) if l == lineage)) {
            let give = held.min(crate::anatomy::room(e, &rules, &k));
            if give > 0 {
                crate::anatomy::give(e, &rules, &k, give).unwrap().unwrap();
            }
        }
    }
    g
}

#[test]
fn a_review_offer_grows_the_part_and_its_port_into_the_body() {
    let g = founding(all(), false).generate().unwrap();
    let name = format!("revision:repertoire-mouth-{JAW}");
    let d = offered(&g, "lineage:3", &name);
    assert!(d.lexicon.contains(JAW) && d.recipe.kinds().contains(JAW));
    // The edit commits as any revision, and the body regrown from it has the part and its port.
    let mut s = session(g.clone());
    s.command(Command::Revise { lineage: "lineage:3".into(), development: d.clone() }).unwrap();
    let rules = &g.rules;
    let body = develop(rules, &d, &soma(rules, &d.recipe, 7)).unwrap();
    let jaws: Vec<_> = body.parts.values().filter(|p| p.port.is_some_and(|p| p.admits_live(NisKind::Consumer) && !p.admits_live(NisKind::Producer))).collect();
    assert_eq!(jaws.len(), 1, "one jaw, in place of the plant mouth");
    assert!(jaws[0].cells.contains_key("function:intake"));
    // A tolerance organ is added behind the body, and a kind outside the world's repertoire is refused.
    let tail = offered(&g, "lineage:3", &format!("revision:repertoire-tail-{ORGAN}"));
    let body = develop(rules, &tail, &soma(rules, &tail.recipe, 7)).unwrap();
    assert!(body.parts.values().any(|p| p.port.is_some_and(|p| p.tolerates(Tolerance::DecomposerFlesh) && !p.has_live())));
    let mut stranger = d.clone();
    let other = rules.kinds.keys().find(|k| !d.lexicon.contains(*k)).unwrap().clone();
    stranger.lexicon.insert(other);
    assert!(s.sim.revisable("lineage:3", &stranger).is_err());
    // Another kingdom's parts are not offered, unless the rule lets lines cross.
    let plant = revise::offered(&s, "lineage:0", &s.sim.state().lineages["lineage:0"].development.clone().unwrap(), usize::MAX);
    assert!(plant.iter().all(|(n, _)| !n.contains("repertoire-")), "no flora part is lowered, and no animal part reaches a plant");
    let across = founding(Some(Repertoire { across_kingdoms: true, ..Default::default() }), false).generate().unwrap();
    let s = session(across);
    let plant = revise::offered(&s, "lineage:0", &s.sim.state().lineages["lineage:0"].development.clone().unwrap(), usize::MAX);
    assert!(plant.iter().any(|(n, _)| n.contains("repertoire-")));
}

#[test]
fn without_the_rule_the_offers_are_as_before() {
    let g = founding(None, false).generate().unwrap();
    let s = session(g);
    for lineage in s.sim.state().lineages.keys() {
        let Some(d) = s.sim.state().lineages[lineage].development.clone() else { continue };
        assert!(revise::offered(&s, lineage, &d, usize::MAX).iter().all(|(n, _)| !n.contains("repertoire-")));
        assert!(revise::revisions(&s, lineage).iter().all(|c| !c.name.contains("repertoire-")));
    }
    let d = s.sim.state().lineages["lineage:3"].development.clone().unwrap();
    let mut widened = d.clone();
    widened.lexicon.insert(JAW.into());
    assert!(s.sim.revisable("lineage:3", &widened).is_err(), "a lexicon moves only where the rule is set");
}

#[test]
fn a_boundary_candidate_can_carry_a_lowered_kind_and_is_scored_like_the_others() {
    let g = founding(all(), false).generate().unwrap();
    let mut s = session(g);
    let seen = revise::revisions(&s, "lineage:3");
    assert!(seen.iter().any(|c| c.name.contains("repertoire-")), "{:?}", seen.iter().map(|c| &c.name).collect::<Vec<_>>());
    let turns = boundary::adapt(&mut s, Some("lineage:1"), &revise::revisions, 3).unwrap();
    let turn = turns.iter().find(|t| t.lineage == "lineage:3").expect("lineage:3 took a turn");
    assert!(turn.considered.iter().any(|(n, _)| n.contains("repertoire-")), "scored beside the others");
    assert!(turn.considered.iter().any(|(n, _)| n == "candidate:stay"));
}

/// Lineage 3 grown with `names` taken in turn, gathered with the decomposer at one site.
fn world(spoil: bool, names: &[&str]) -> Genesis {
    let mut g = founding(all(), spoil).generate().unwrap();
    for name in names {
        let d = offered(&g, "lineage:3", name);
        g = grown(g, "lineage:3", d);
    }
    let site = *g.sites.keys().next().unwrap();
    for group in g.population.groups.values_mut() {
        if ["lineage:2", "lineage:3"].contains(&group.entity.lineage.as_str()) {
            group.entity_mut().place = site;
        }
    }
    g
}

fn bites(g: Genesis) -> (u64, Vec<crate::flows::Harm>) {
    let mut s = session(g);
    let (mut meals, mut harms) = (0, vec![]);
    for _ in 0..100 {
        let r = s.advance_tick_with_flows().unwrap();
        for f in &r.flows {
            if f.made_by == MadeBy::Process("ecology:feed-3".into()) && f.from_kind.as_ref().is_some_and(|k| k.lineage == "lineage:2") {
                meals += f.amount * f.count;
            }
        }
        harms.extend(s.sim.harms().iter().filter(|h| h.made_by == "ecology:feed-3" && h.lineage == "lineage:2").cloned());
    }
    (meals, harms)
}

#[test]
fn a_jaw_grown_by_edit_bites_a_decomposer_and_is_harmed_unless_it_tolerates_it() {
    let jaw = format!("revision:repertoire-mouth-{JAW}");
    let (meals, harms) = bites(world(true, &[&jaw]));
    assert!(meals > 0, "the grown jaw bit a decomposer");
    assert!(!harms.is_empty() && harms.iter().all(|h| h.cells == 2), "and paid for it (D36)");
    // The mouth that admits decomposers, and the jaw with an organ that tolerates them, do not.
    let specialist = format!("revision:repertoire-mouth-{SPECIALIST}");
    let (meals, harms) = bites(world(true, &[&specialist]));
    assert!(meals > 0 && harms.is_empty(), "the specialist mouth is not harmed");
    let organ = format!("revision:repertoire-tail-{ORGAN}");
    let (meals, harms) = bites(world(true, &[&jaw, &organ]));
    assert!(meals > 0 && harms.iter().all(|h| h.flesh != Flesh::Fresh), "the tolerance organ spares the jaw the living flesh");
    // Without spoilage the same jaw is refused decomposers.
    let (meals, harms) = bites(world(false, &[&jaw]));
    assert_eq!((meals, harms.len()), (0, 0));
}

#[test]
fn the_rule_is_the_same_in_both_executions() {
    let jaw = format!("revision:repertoire-mouth-{JAW}");
    let hash = |mode| {
        let mut s = Session::new(world(true, &[&jaw]), mode).unwrap();
        s.advance(30).unwrap();
        s.sim.state_hash()
    };
    assert_eq!(hash(Execution::Individuals), hash(Execution::Grouped));
}

#[test]
fn a_line_with_a_grammar_is_offered_the_part_as_a_differentiation_of_its_last_run() {
    let mut g = founding(all(), false).generate().unwrap();
    let d = g.lineages.get_mut("lineage:3").unwrap().development.as_mut().unwrap();
    let mut seed = d.recipe.tagmata.clone();
    seed.last_mut().unwrap().segments = 4;
    d.grammar = Some(crate::rules::Grammar { seed, rules: vec![] });
    d.recipe.tagmata = crate::rules::unfold(d.grammar.as_ref().unwrap()).unwrap();
    let s = session(g);
    let d = s.sim.state().lineages["lineage:3"].development.clone().unwrap();
    let all = revise::offered(&s, "lineage:3", &d, usize::MAX);
    let (_, v) = all.iter().find(|(n, _)| n == &format!("revision:rule-differentiate-{ORGAN}")).expect("offered as a rule");
    assert!(v.lexicon.contains(ORGAN) && v.recipe.kinds().contains(ORGAN));
    assert!(v.grammar.as_ref().unwrap().rules.iter().any(|r| matches!(r, crate::rules::Rule::Differentiate { kind, .. } if kind == ORGAN)));
    assert!(s.sim.revisable("lineage:3", v).is_ok());
}


#[test]
fn the_wider_feeds_change_nothing_for_founded_bodies() {
    let counts = |repertoire| {
        let mut s = session(founding(repertoire, true).generate().unwrap());
        let mut out = vec![];
        for _ in 0..4 {
            s.advance(40).unwrap();
            let mut n = [0u64; 5];
            for g in s.sim.state().population.groups.values().filter(|g| g.entity.alive) {
                if let Some(i) = g.entity.lineage.strip_prefix("lineage:").and_then(|i| i.parse::<usize>().ok()) { n[i] += g.count; }
            }
            out.push(n);
        }
        out
    };
    assert_eq!(counts(None), counts(all()));
}

/// Lineage 4 (the omnivore) revised to the mouth named `kind`, then played `ticks`.
fn revised(mode: Execution, ticks: u64) -> Session {
    let g = founding(all(), true).generate().unwrap();
    let name = format!("revision:repertoire-mouth-{}", "kind:catalog-fauna-broad-mouth-eat-decomposer");
    let d = offered(&g, "lineage:4", &name);
    let mut s = Session::new(g, mode).unwrap();
    s.command(Command::Revise { lineage: "lineage:4".into(), development: d }).unwrap();
    s.advance(ticks).unwrap();
    s
}

#[test]
fn the_young_of_a_line_that_took_a_part_carry_it_and_both_executions_agree() {
    let s = revised(Execution::Individuals, 60);
    let port = s.sim.genesis().rules.kinds["kind:catalog-fauna-broad-mouth-eat-decomposer"].port.unwrap();
    let young: Vec<_> = s.sim.state().population.groups.values()
        .filter(|g| g.entity.lineage == "lineage:4" && g.entity.alive && g.entity.born > 0).collect();
    assert!(!young.is_empty(), "the omnivore line bore young");
    assert!(young.iter().all(|g| g.entity.parts.values().any(|p| p.port == Some(port))), "each carries the part's port");
    assert_eq!(revised(Execution::Individuals, 60).sim.state_hash(), revised(Execution::Grouped, 60).sim.state_hash());
}

mod categories;
mod detritus;
mod guard;
mod mutualism;
