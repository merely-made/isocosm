// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The systems sheet (case 1215; lineage-rounds LR4).

use super::draft::grown_body;
use super::sheet::{self, Sheet};
use super::tests::learned;
use super::boundary;
use crate::process::BodyProcesses;
use crate::{Execution, Session, history::Command, lineage::Review, rules::*, schema::*};
use std::collections::BTreeMap;
use std::sync::Arc;

fn rule(s: &mut Session, sheet: bool, draft: Option<Draft>) {
    let g = Arc::make_mut(&mut s.sim.genesis);
    let d = g.rules.directing.get_or_insert_with(Directing::default);
    d.sheet = sheet;
    d.draft = draft;
}

fn development(s: &Session, line: &str) -> Development {
    s.sim.state().lineages[line].development.clone().unwrap()
}

fn ate(s: &mut Session, line: &str, kind: &str) {
    let l = s.sim.state.lineages.get_mut(line).unwrap();
    l.meals.insert(kind.into(), Eaten { count: 1, needed: 1, last: 10 });
}

fn review(s: &Session, line: &str, since: Option<Tick>) -> Review {
    Review::of_draft_using(s, line, 2, since, &mut boundary::score).unwrap()
}

fn sheet_of(s: &Session, line: &str, d: &Development) -> Sheet {
    let lines = &s.sim.state().lineages;
    sheet::of(&s.sim.genesis().rules, lines, line, d).unwrap()
}

fn revised(c: &[Command]) -> &Development {
    match &c[0] {
        Command::Revise { development, .. } => development,
        other => panic!("not a revision: {other:?}"),
    }
}

/// The kind `learned()` taught its line, in the lexicon and not the recipe.
fn taught(s: &Session, line: &str) -> Key {
    let d = development(s, line);
    let named = d.recipe.kinds();
    d.lexicon.iter().find(|k| !named.contains(*k)).cloned().unwrap()
}

/// A session whose line has eaten its taught kind, with both rules set and
/// room for every variant.
fn drafted() -> (Session, Key) {
    let (mut s, line, _) = learned();
    let kind = taught(&s, &line);
    ate(&mut s, &line, &kind);
    rule(&mut s, true, Some(Draft { share: [1_000_000, 1] }));
    Arc::make_mut(&mut s.sim.genesis).rules.directing.as_mut().unwrap().variants = 64;
    (s, line)
}

fn bearing(r: &Review) -> Vec<&super::Offer> {
    r.offers.iter().filter(|o| o.name.contains("-bears-")).collect()
}

#[test]
fn unset_nothing_is_attached_and_the_review_is_as_before() {
    let (mut s, line, _) = learned();
    let kind = taught(&s, &line);
    ate(&mut s, &line, &kind);
    let plain = Review::of(&s, &line, 2).unwrap();
    assert!(plain.sheet.is_none() && plain.offers.iter().all(|o| o.sheet.is_none()));
    assert_eq!(review(&s, &line, None), plain);
    assert_eq!(review(&s, &line, Some(0)), plain);
    rule(&mut s, false, Some(Draft::default()));
    let drafted = review(&s, &line, Some(0));
    assert!(drafted.sheet.is_none() && drafted.offers.iter().all(|o| o.sheet.is_none()));
    // Nothing of it reaches the wire.
    let json = serde_json::to_string(&plain).unwrap();
    assert!(!json.contains("\"sheet\""), "{json}");
}

#[test]
fn a_sheet_is_a_pure_reading() {
    let (mut a, line, _) = learned();
    let (b, _, _) = learned();
    let d = development(&a, &line);
    let (first, twin) = (sheet_of(&a, &line, &d), sheet_of(&b, &line, &d));
    assert_eq!(first, twin, "two sessions from one seed");
    a.advance(12).unwrap();
    assert_eq!(sheet_of(&a, &line, &d), first, "unchanged by advancing");
    assert_eq!(sheet_of(&a, &line, &d), sheet_of(&a, &line, &d));
}

#[test]
fn its_mass_is_the_grown_mass_and_an_offer_differs_by_its_cost() {
    let (s, line) = drafted();
    let r = review(&s, &line, Some(10));
    let own = r.sheet.clone().expect("the line's own sheet");
    let rules = &s.sim.genesis().rules;
    let by_hand: u128 = grown_body(rules, &development(&s, &line)).unwrap().doc.parts.iter()
        .map(|p| u128::from(crate::anatomy::ceiling(p.half_extent, rules.body()))).sum();
    assert_eq!(own.mass, by_hand);
    let offers = bearing(&r);
    assert!(!offers.is_empty());
    for o in offers {
        let had = o.sheet.as_ref().expect("a revision carries its sheet");
        assert_eq!(had.mass.saturating_sub(own.mass), o.cost.unwrap(), "{}", o.name);
    }
    assert!(r.offers[0].sheet.is_none(), "the status quo has nothing to compare");
}

#[test]
fn a_preview_is_the_result() {
    let (mut s, line) = drafted();
    let r = review(&s, &line, Some(10));
    let offer = bearing(&r)[0].clone();
    assert!(offer.takeable());
    for command in offer.commands.clone() {
        s.command(command).unwrap();
    }
    let now = development(&s, &line);
    assert_eq!(&now, revised(&offer.commands), "the revision took");
    assert_eq!(Some(sheet_of(&s, &line, &now)), offer.sheet);
    // The next review's own sheet is that sheet.
    assert_eq!(review(&s, &line, Some(10)).sheet, offer.sheet);
}

#[test]
fn it_changes_when_the_development_does_and_not_otherwise() {
    let (s, line) = drafted();
    let r = review(&s, &line, Some(10));
    let had = development(&s, &line);
    let offer = bearing(&r)[0];
    let (a, b) = (sheet_of(&s, &line, &had), offer.sheet.clone().unwrap());
    assert!(b.mass > a.mass);
    assert_ne!(a.functions, b.functions);
    assert_eq!(a, sheet_of(&s, &line, &had.clone()), "an identical development");
    // A change the body does not show is no change in the sheet.
    let mut lexed = had.clone();
    lexed.lexicon.clear();
    assert_eq!(sheet_of(&s, &line, &lexed), a);
    // Eggs a birth lays are the development's.
    let mut more = had.clone();
    more.clutch += 3;
    let c = sheet_of(&s, &line, &more);
    assert_eq!((c.clutch, Sheet { clutch: a.clutch, ..c }), (a.clutch + 3, a));
}

#[test]
fn a_small_authored_body_reads_as_it_is() {
    let (s, line, member) = learned();
    let mut rules = s.sim.genesis().rules.clone();
    let functions: Vec<Key> = rules.functions.keys().take(2).cloned().collect();
    rules.kinds.insert("kind:authored".into(), Template {
        half_extent: [2, 2, 2],
        cells: BTreeMap::from([(functions[0].clone(), 3), (functions[1].clone(), 2)]),
        shape: String::new(), port: None,
    });
    let mut d = development(&s, &line);
    let mut tagma = d.recipe.tagmata[0].clone();
    (tagma.segments, tagma.segment) = (2, "kind:authored".into());
    (tagma.bears, tagma.per_segment, tagma.chain) = (None, 0, vec![]);
    d.recipe.tagmata = vec![tagma];
    d.clutch = 3;
    d.systems.clear();
    rules.speed = None;
    let lines = s.sim.state().lineages.clone();
    let bare = sheet::of(&rules, &lines, &line, &d).unwrap();
    assert_eq!(bare.speed, None, "no speed rules, no speed");
    assert_eq!(bare.clutch, 3);
    assert_eq!(bare.upkeep, None, "no body-priced upkeep in this world");
    let body = grown_body(&rules, &d).unwrap();
    let mut sums = BTreeMap::<Key, u32>::new();
    for p in body.parts.values() {
        for (f, n) in &p.cells {
            *sums.entry(f.clone()).or_default() += n;
        }
    }
    assert_eq!(bare.functions, sums);
    assert!(bare.functions.contains_key(&functions[0]));
    // A real entity of the body reads the same.
    let mut e = s.sim.state().population.get(member).unwrap().clone();
    e.embody(body);
    e.lay_out();
    let doc = e.body.as_ref().unwrap();
    assert_eq!(bare.reach, doc.reach());
    assert_eq!(bare.trophic, crate::trophic::of(&e));
    assert_eq!(bare.feeding, crate::mosaic::ports::feeding(&e));
    let cells: u32 = e.parts.values().flat_map(|p| p.cells.values()).sum();
    assert_eq!(bare.functions.values().sum::<u32>(), cells);
    // With speed rules the world's speed reads off the body.
    rules.speed = Some(crate::speed::Speed::froude(1000));
    let fast = sheet::of(&rules, &lines, &line, &d).unwrap();
    assert_eq!(fast.speed, rules.speed.map(|v| v.of(doc)));
    assert!(fast.speed.is_some());
    assert_eq!(Sheet { speed: None, ..fast }, bare);
}

#[test]
fn upkeep_is_what_the_process_charges_a_grown_member() {
    let g = crate::probe::BodyFounding { seed: 3, ..Default::default() }.generate().unwrap().genesis;
    let rules = g.rules.clone();
    let lines = g.lineages.clone();
    let mut sim = crate::Simulation::new(g, Execution::Individuals).unwrap();
    let mut checked = 0;
    for line in lines.keys().filter(|k| k.starts_with("lineage:")) {
        let Some(d) = lines[line].development.as_ref() else { continue };
        let Some((first, held)) = sim.state().population.groups.iter()
            .find(|(_, c)| c.entity.alive && c.entity.lineage == *line)
            .map(|(id, c)| (*id, c.entity.as_ref().clone())) else { continue };
        let sheet = sheet::of(&rules, &lines, line, d).unwrap();
        let mut adult = held;
        adult.embody(grown_body(&rules, d).unwrap());
        adult.accounts.clear();
        let tissue = crate::anatomy::own(&rules, line).tissue.unwrap();
        let room = crate::anatomy::room(&adult, &rules, &tissue);
        crate::anatomy::give(&mut adult, &rules, &tissue, room).unwrap().unwrap();
        let before = crate::anatomy::held(&adult, &rules, &tissue);
        *sim.state.population.lift(first).unwrap() = adult;
        sim.conserved = sim.matter();
        let process = format!("body:upkeep-{}", &line[8..]);
        let done = sim.execute(first, None, &process, None);
        assert_eq!(done.outcome, crate::simulation::Outcome::Accepted, "{process}");
        let after = sim.state().population.get(first).unwrap();
        let charged = before - crate::anatomy::held(after, &rules, &tissue);
        assert_eq!(sheet.upkeep, Some(charged), "{line}");
        checked += 1;
    }
    assert!(checked > 0, "no body-priced line to check");
    // The flat upkeep of an ecology world is not the body's.
    let (s, line, _) = learned();
    assert_eq!(sheet_of(&s, &line, &development(&s, &line)).upkeep, None);
}
