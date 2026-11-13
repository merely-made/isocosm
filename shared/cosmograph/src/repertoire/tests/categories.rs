// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The repertoire with the categories rule (`ecology::categories`): parts
//! that need the rule lower only where it is set, grinding mouths lower with
//! a tolerance of tough plant matter and class mouths with their classes, and
//! a grinding mouth a line takes eats tough tissue unharmed.

use super::*;
use crate::{ecology::Categories, process::DeadClass};

const GRINDER: &str = "kind:catalog-fauna-broad-mouth-grind-tough";
const LITTER: &str = "kind:catalog-litter-fungus";
const WOOD: &str = "kind:catalog-cellulose-microbe";

fn both() -> Founding {
    Founding { categories: Some(Categories::default()), ..founding(all(), false) }
}

#[test]
fn parts_that_need_the_rule_lower_only_where_it_is_set() {
    let without = founding(all(), false).generate().unwrap();
    let kinds = |g: &Genesis| g.rules.repertoire.as_ref().unwrap().kinds.keys().cloned().collect::<BTreeSet<_>>();
    let with = both().generate().unwrap();
    let (a, b) = (kinds(&without), kinds(&with));
    assert!(a.is_subset(&b) && b.len() > a.len(), "{} without, {} with", a.len(), b.len());
    for kind in [GRINDER, LITTER, WOOD] {
        assert!(b.contains(kind) && !a.contains(kind), "{kind}");
    }
    // Every part that needs the rule is absent where it is unset, and the world has no dung mouth.
    for part in catalog().parts().iter().filter(|p| needs_categories(p)) {
        assert!(!a.contains(&kind_key(&part.key)), "{}", part.key);
    }
    assert!(b.iter().all(|k| !k.contains("dung")));
    let named = Founding { repertoire: Some(Repertoire { only: BTreeSet::from(["part:fauna-broad-mouth-grind-tough".into()]), ..Default::default() }), ..founding(None, false) };
    assert!(named.generate().unwrap_err().contains("needs the categories rule"));
}

#[test]
fn the_lowered_ports_carry_the_rules_words() {
    let g = both().generate().unwrap();
    let port = |k: &str| g.rules.kinds[k].port.unwrap();
    let grinder = port(GRINDER);
    assert!(grinder.admits_live(NisKind::Producer) && grinder.tolerates(Tolerance::ToughPlant));
    assert!(grinder.takes(DeadClass::Wood) && !grinder.takes(DeadClass::Litter), "dead wood, limited to it");
    let litter = port(LITTER);
    assert!(litter.admits_deadstock() && litter.takes(DeadClass::Litter) && !litter.takes(DeadClass::Wood) && !litter.takes(DeadClass::Carrion));
    let wood = port(WOOD);
    assert!(wood.takes(DeadClass::Wood) && !wood.takes(DeadClass::Litter) && wood.tolerates(Tolerance::ToughPlant));
    // Without the rule's words a dead mouth takes every class and nothing grinds.
    let plain = founding(all(), false).generate().unwrap();
    assert!(plain.rules.kinds.values().filter_map(|t| t.port).all(|p| p.class_bits() == 0 && !p.tolerates(Tolerance::ToughPlant)));
    assert!(g.rules.validate().is_ok());
}

/// Lineage 3, the grazer of lineage 0, grown with `kind` as its mouth, lineage 0's matter all in tough parts.
fn grazing(kind: Option<&str>) -> (u64, Vec<crate::flows::Harm>, bool) {
    let mut g = both().generate().unwrap();
    if let Some(kind) = kind {
        let d = offered(&g, "lineage:3", &format!("revision:repertoire-mouth-{kind}"));
        g = grown(g, "lineage:3", d);
    }
    let site = *g.sites.keys().next().unwrap();
    for group in g.population.groups.values_mut() {
        let e = group.entity_mut();
        if ["lineage:0", "lineage:3"].contains(&e.lineage.as_str()) {
            e.place = site;
        }
        if e.lineage == "lineage:0" {
            let id = *e.parts.keys().find(|id| crate::ecology::tough(e, **id)).unwrap();
            let mut all = crate::schema::Ledger::new();
            for part in e.parts.values_mut() {
                for (k, n) in std::mem::take(&mut std::sync::Arc::make_mut(part).matter) {
                    *all.entry(k).or_default() += n;
                }
            }
            e.part_mut(&id).unwrap().matter = all;
        }
    }
    let mut s = session(g);
    let total = s.sim.budget().total();
    let (mut meals, mut harms, mut closed) = (0, vec![], true);
    for _ in 0..60 {
        let r = s.advance_tick_with_flows().unwrap();
        for f in &r.flows {
            if f.made_by == MadeBy::Process("ecology:feed-3".into()) && f.from_kind.as_ref().is_some_and(|k| k.lineage == "lineage:0") {
                meals += f.amount * f.count;
            }
        }
        harms.extend(s.sim.harms().iter().filter(|h| h.made_by == "ecology:feed-3").cloned());
        closed &= s.sim.budget().total() == total;
    }
    (meals, harms, closed)
}

#[test]
fn a_grinding_mouth_a_line_takes_eats_tough_matter_unharmed() {
    let name = format!("revision:repertoire-mouth-{GRINDER}");
    let g = both().generate().unwrap();
    let d = offered(&g, "lineage:3", &name);
    assert!(d.recipe.kinds().contains(GRINDER), "the mouth is offered, and it is the line's after the edit");
    let (meals, harms, closed) = grazing(Some(GRINDER));
    assert!(meals > 0, "the grinding mouth ate the plant");
    assert!(harms.is_empty(), "and was not harmed: {} harms", harms.len());
    assert!(closed);
    // The founded plant mouth, on the same tissue, is.
    let (meals, harms, closed) = grazing(None);
    assert!(meals > 0 && !harms.is_empty() && harms.iter().all(|h| h.tough));
    assert!(closed);
}
