// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A partner (lineage-rounds D44, D51): a grammar reserves a slot on a run,
//! a whole part taken in while it is free lands there keeping all its
//! functions and its own kind whatever its crossing, and the lineage's
//! recipe records it so young grow it. Matter is conserved throughout.

use super::*;

const FROND: &str = "kind:frond";

/// The grazer's grammar keeps a slot on its one run.
fn partnered(domain: u16) -> (Genesis, Id, Id, Development) {
    let (mut g, eater, prey) = world(domain, 1000, true);
    let mut d = g.lineages["lineage:1"].development.clone().unwrap();
    // The lump's run bears limbs; the slot sits on the same run.
    d.grammar = Some(Grammar {
        seed: d.recipe.tagmata.clone(),
        rules: vec![Rule::Partner { tagma: 0, kept: None }],
    });
    let d = regrown(&d).unwrap();
    g.lineages.get_mut("lineage:1").unwrap().development = Some(d.clone());
    dress(&mut g, eater, &d, "tissue:1");
    (g, eater, prey, d)
}

fn total(s: &Session) -> i128 {
    books(s).values().sum()
}

#[test]
fn a_whole_part_lands_in_a_free_slot_keeping_its_functions_and_the_recipe_records_it() {
    // Domain 2 is an adapter's: elsewhere it lands expressing nothing.
    let (g, eater, prey, d) = partnered(2);
    assert_eq!(d.recipe.tagmata[0].slot, Some(Slot { kept: None }));
    let mut s = Session::new(g, Execution::Individuals).unwrap();
    let before = total(&s);
    let act = Command::Act { actor: eater, target: Some(prey), process: "test:graze".into(), cause: None };
    s.command(act).unwrap();
    assert_eq!(total(&s), before, "no matter made or lost");
    let grazer = body(&s, eater);
    let landed = grazer.living().find(|(id, _)| grazer.situs(*id) == Some([0, 0, PARTNER_SLOT]));
    let (_, part) = landed.expect("the frond landed in the slot");
    assert_eq!(part.cells["function:fix"], 3, "an adapter's crossing, and it keeps working");
    assert!(part.functions.contains("function:fix"));
    // The recipe, its grammar and the lexicon hold the kind.
    let line = s.sim.state().lineages["lineage:1"].development.clone().unwrap();
    assert_eq!(line.recipe.tagmata[0].slot, Some(Slot { kept: Some(FROND.into()) }));
    assert_eq!(line.grammar.as_ref().unwrap().rules, [Rule::Partner { tagma: 0, kept: Some(FROND.into()) }]);
    assert!(line.lexicon.contains(FROND));
    assert_eq!(regrown(&line).unwrap(), line, "the grammar says the same");
    // Young grow it: a body developed from the recipe holds the part, and
    // the slot is no longer free for a second.
    let soma = Soma { segments: vec![1], absent: vec![], seed: 0 };
    let young = develop(&s.sim.genesis().rules, &line, &soma).unwrap();
    let held = young.doc.parts.iter().filter(|p| p.situs == Some([0, 0, PARTNER_SLOT])).count();
    assert_eq!(held, 1);
}

#[test]
fn without_a_slot_an_adapter_still_lands_expressing_nothing() {
    let (s, eater, _, _) = graze(2, 1000, true);
    let grazer = body(&s, eater);
    assert!(grazer.parts.values().all(|p| !p.functions.contains("function:fix")));
    assert!(grazer.living().all(|(id, _)| grazer.situs(id).is_none_or(|s| s[2] != PARTNER_SLOT)));
}
