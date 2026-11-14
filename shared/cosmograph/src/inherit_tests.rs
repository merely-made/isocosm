// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Young grow from the line's recipe (D40): unset it changes nothing; a line
//! whose recipe was revised bears young with the revised body while its parent
//! keeps the old one, and without the rule the young copy their parent; the
//! budget stays closed; and both executions hatch the same bodies.

use crate::{
    Execution, Founding, Session,
    bodied::Bodies,
    directing::Played,
    ecology::{Rates, Web},
    history::Command,
};

fn founding(inherit: bool) -> Founding {
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
        inherit,
        ..Default::default()
    }
}

#[test]
fn unset_founds_and_hashes_as_before() {
    let plain = founding(false);
    let json = serde_json::to_string(&plain).unwrap();
    assert!(!json.contains("\"inherit\":"));
    let g = plain.generate().unwrap();
    assert!(!g.rules.inherit && !serde_json::to_string(&g.rules).unwrap().contains("\"inherit\":"));
    let bare: Founding = serde_json::from_str(&json).unwrap();
    assert_eq!(crate::digest(&bare.generate().unwrap()), crate::digest(&g));
    // Set, it is written, reads back, and changes the world's rules.
    let set = founding(true);
    assert!(serde_json::to_string(&set).unwrap().contains("\"inherit\":true"));
    assert_eq!(serde_json::from_str::<Founding>(&serde_json::to_string(&set).unwrap()).unwrap(), set);
    let g2 = set.generate().unwrap();
    assert!(g2.rules.inherit);
    assert_ne!(crate::digest(&g2), crate::digest(&g));
    assert!(Founding { ecology: false, ..set }.generate().is_err());
}

#[test]
fn young_keep_the_parents_variation_where_it_fits() {
    use crate::{development::soma, rules::Development};
    let g = founding(true).generate().unwrap();
    let (line, d): (&String, &Development) = g.lineages.iter()
        .find_map(|(k, l)| l.development.as_ref().filter(|d| d.recipe.tagmata.iter().any(|t| t.variance != Some(0) && t.segments > 2)).map(|d| (k, d))).unwrap();
    let rules = &g.rules;
    let drawn = (0..200).map(|seed| soma(rules, &d.recipe, seed)).find(|x| x.segments != d.recipe.tagmata.iter().map(|t| t.segments).collect::<Vec<_>>()).unwrap();
    let mut parent = g.population.groups.values().map(|g| (*g.entity).clone()).find(|e| &e.lineage == line).unwrap();
    parent.soma = drawn.segments.clone();
    parent.absent = vec![];
    let mut child = parent.clone();
    let mut l = g.lineages[line].clone();
    crate::inherit::rebody(rules, Some(&l), &mut child).unwrap();
    assert_eq!(child.soma, drawn.segments, "a drawn soma that fits is kept");
    // A count past the recipe's range is replaced, a tagma beyond the parent's is the recipe's own.
    parent.soma = vec![250];
    let mut far = parent.clone();
    crate::inherit::rebody(rules, Some(&l), &mut far).unwrap();
    assert_eq!(far.soma, d.recipe.tagmata.iter().map(|t| t.segments).collect::<Vec<_>>());
    l.development = None;
    let mut same = parent.clone();
    crate::inherit::rebody(rules, Some(&l), &mut same).unwrap();
    assert_eq!(same.soma, parent.soma, "a line with no development keeps its parent's form");
}

/// The parts a body holds.
fn size(e: &crate::schema::Entity) -> usize {
    e.parts.len()
}

/// `line`'s recipe given four more segments in its first tagma that has any, committed as a revision.
fn lengthened(s: &mut Session, line: &str) {
    let mut d = s.sim.state().lineages[line].development.clone().unwrap();
    let t = d.recipe.tagmata.iter_mut().find(|t| t.segments > 1 && t.variance != Some(0)).expect("a tagma to lengthen");
    t.segments += 4; // past any variance, so the parent's count no longer fits
    s.command(Command::Revise { lineage: line.into(), development: d }).unwrap();
}

/// A world with every line's recipe lengthened by four segments in its first tagma, played `ticks`.
fn played(mode: Execution, inherit: bool, ticks: u64) -> Session {
    let g = founding(inherit).generate().unwrap();
    let mut s = Session::new(g, mode).unwrap();
    let lines: Vec<String> = s.sim.state().lineages.iter()
        .filter(|(_, l)| l.development.as_ref().is_some_and(|d| d.recipe.tagmata.iter().any(|t| t.segments > 1 && t.variance != Some(0))))
        .map(|(k, _)| k.clone()).collect();
    assert!(!lines.is_empty());
    for line in &lines { lengthened(&mut s, line); }
    s.advance(ticks).unwrap();
    s
}

/// For each young (born after the start): the parts it has, the parts its own soma grows from its line's
/// current recipe, and whether every tagma's segment count lies in the range that recipe could draw.
fn young(s: &Session) -> Vec<(usize, usize, bool)> {
    let rules = &s.sim.genesis().rules;
    s.sim.state().population.groups.values().map(|g| &g.entity)
        .filter(|e| e.alive && e.born > 0)
        .filter_map(|e| {
            let d = s.sim.state().lineages.get(&e.lineage)?.development.as_ref()?;
            let own = crate::development::Soma { segments: e.soma.clone(), absent: e.absent.clone(), seed: 0 };
            let fits = d.recipe.tagmata.iter().zip(&e.soma).all(|(t, n)| {
                let v = i32::from(t.variance.unwrap_or(d.recipe.variance));
                (i32::from(t.segments) - v..=i32::from(t.segments) + v).contains(&i32::from(*n))
            });
            Some((size(e), crate::development::develop(rules, d, &own).ok()?.parts.len(), fits))
        }).collect()
}

#[test]
fn young_grow_the_revised_recipe_and_the_parent_keeps_its_old_body() {
    let s = played(Execution::Individuals, true, 80);
    let kids = young(&s);
    assert!(kids.len() > 10, "{} young", kids.len());
    assert!(kids.iter().all(|(have, grown, fits)| have == grown && *fits), "each young body is its line's current recipe, within the range it could draw");
    // The founders, never regrown, keep the body they had.
    let before = founding(true).generate().unwrap();
    let founder_sizes: Vec<usize> = before.population.groups.values().map(|g| size(&g.entity)).collect();
    let longer = s.sim.state().population.groups.values().map(|g| &g.entity)
        .filter(|e| e.alive && e.born == 0)
        .all(|e| founder_sizes.contains(&size(e)));
    assert!(longer, "a founding parent keeps the body it had");
}

#[test]
fn without_the_rule_the_young_copy_their_parent() {
    let s = played(Execution::Individuals, false, 80);
    let kids = young(&s);
    assert!(kids.len() > 10, "{} young", kids.len());
    assert!(kids.iter().any(|(_, _, fits)| !fits), "a copy keeps the body its parent had, the revision unreached");
}

#[test]
fn the_budget_stays_closed_and_both_executions_agree() {
    let hash = |mode| {
        let mut s = played(mode, true, 0);
        let total = s.sim.budget().total();
        for _ in 0..80 {
            s.advance(1).unwrap();
            assert_eq!(s.sim.budget().total(), total, "growing from the recipe spawned or lost matter");
        }
        s.sim.state_hash()
    };
    assert_eq!(hash(Execution::Individuals), hash(Execution::Grouped));
}


#[test]
fn generated_descendants_vary_and_replay() {
    let mut f = founding(true);
    f.inherited_variation = 65;
    f.map = None;
    let g = f.generate().unwrap();
    assert!(g.lineages.values().any(|line| line.development.as_ref().is_some_and(|d| {
        d.grammar.as_ref().is_some_and(|grammar| {
            !grammar.rules.is_empty() && crate::rules::unfold(grammar).unwrap() == d.recipe.tagmata
        })
    })));
    let mut s = Session::new(g, Execution::Individuals).unwrap();
    let total = s.sim.budget().total();
    s.advance(75).unwrap();
    let state = s.sim.state();
    let changed = state.relations.iter().filter(|r| r.kind == "sim:parent").any(|r| {
        state.population.get(r.object).zip(state.population.get(r.subject))
            .is_some_and(|(parent, child)| !parent.soma.is_empty() && parent.soma != child.soma)
    });
    assert!(changed, "the ordinary birth path introduces variation");
    assert_eq!(s.sim.budget().total(), total);
    let replayed = Session::load(s.save(), Execution::Grouped).unwrap();
    assert_eq!(replayed.sim.state_hash(), s.sim.state_hash());
}


#[test]
fn a_subclade_carries_its_own_edited_recipe_into_birth() {
    use crate::{rules::{Rule, regrown}, schema::Id};

    let mut f = founding(true);
    f.inherited_variation = 30;
    let g = f.generate().unwrap();
    let mut s = Session::new(g, Execution::Individuals).unwrap();
    let (&id, group) = s.sim.state().population.groups.iter()
        .find(|(_, group)| group.entity.soma.len() > 1)
        .expect("a founder with a developed body");
    let root = group.entity.lineage.clone();
    let original = group.entity.soma[1];
    let branch = s.command(Command::Speciate { founder: id, name: "ridge".into() }).unwrap();
    let mut development = s.sim.state().lineages[&branch].development.clone().unwrap();
    let grammar = development.grammar.as_mut().unwrap();
    grammar.seed[1].variance = Some(0);
    grammar.rules.push(Rule::Repeat { tagma: 1, by: 1 });
    let development = regrown(&development).unwrap();
    let next = development.recipe.tagmata[1].segments;
    s.command(Command::Revise { lineage: branch.clone(), development }).unwrap();
    assert_eq!(s.sim.state().lineages[&branch].parent.as_deref(), Some(root.as_str()));
    assert_eq!(s.sim.state().population.get(id).unwrap().soma[1], original);
    let born: Id = s.sim.state().population.next_id;
    let mut child = s.sim.state().population.get(id).unwrap().clone();
    crate::inherit::rebody_child(&s.sim, born, &mut child).unwrap();
    assert_eq!(child.soma[1], next);
    assert_eq!(Session::load(s.save(), Execution::Grouped).unwrap().sim.state_hash(), s.sim.state_hash());
}

#[test]
#[ignore = "release measurement of inherited anatomy"]
fn measure_shared_birth_anatomy() {
    use std::{hint::black_box, time::Instant};

    for population in [256, 2048, 16384] {
        let mut f = founding(true);
        f.played = None;
        f.map = None;
        f.population = population;
        f.inherited_variation = 60;
        let sim = crate::Simulation::new(f.generate().unwrap(), Execution::Grouped).unwrap();
        let births = sim.state().population.groups.iter().flat_map(|(&first, group)| {
            (first..first + group.count).map(move |id| (id, &group.entity))
        }).filter(|(_, parent)| !parent.soma.is_empty()).collect::<Vec<_>>();
        let take = |cached| {
            let mut witness = 0u64;
            let mut forms = std::collections::BTreeSet::new();
            let start = Instant::now();
            for &(id, parent) in &births {
                let mut child = (**parent).clone();
                let grow = if cached { crate::inherit::rebody_child } else { crate::inherit::rebody_uncached };
                grow(&sim, id, &mut child).unwrap();
                witness = witness.wrapping_add(child.parts.len() as u64);
                forms.insert((child.lineage.clone(), child.soma.clone(), child.absent.clone()));
                black_box(child);
            }
            (forms.len(), witness, start.elapsed().as_secs_f64() * 1000.0)
        };
        let (forms, uncached, baseline) = take(false);
        let (cached_forms, cached, shared) = take(true);
        assert_eq!((forms, uncached), (cached_forms, cached));
        eprintln!("founders={population} children={} forms={forms} uncached_ms={baseline:.2} cached_ms={shared:.2}", births.len());
    }
}
