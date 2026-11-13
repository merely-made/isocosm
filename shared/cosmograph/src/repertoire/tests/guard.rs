// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The repertoire with the guard rule (`ecology::guard`): the armour, spine
//! and gland parts that need the rule lower only where it is set, with the
//! strengths their traits give; mouths carry their attack and a detox organ
//! its tolerance of toxin; a line takes one by editing, and the body regrown
//! from it carries a part that works.

use super::*;
use crate::{
    ecology::{Guard, Ward},
    mosaic::ports::active,
    process::{IntakePort, Strength},
};

const PLATED: &str = "kind:catalog-plated-shell";
const QUILL: &str = "kind:catalog-quill";
const CYANIDE: &str = "kind:catalog-cyanide-leaf";
const JAW_STRONG: &str = "kind:catalog-fauna-jaw-take-prey";

fn guarded() -> Founding {
    Founding { guard: Some(Guard::default()), ..founding(all(), false) }
}

#[test]
fn parts_that_need_the_rule_lower_only_where_it_is_set() {
    let kinds = |g: &Genesis| g.rules.repertoire.as_ref().unwrap().kinds.keys().cloned().collect::<BTreeSet<_>>();
    let (a, b) = (kinds(&founding(all(), false).generate().unwrap()), kinds(&guarded().generate().unwrap()));
    assert!(a.is_subset(&b) && b.len() > a.len(), "{} without, {} with", a.len(), b.len());
    for kind in [PLATED, QUILL, CYANIDE, "kind:catalog-hide", "kind:catalog-flora-spine-spine-prick"] {
        assert!(b.contains(kind) && !a.contains(kind), "{kind}");
    }
    for part in catalog().parts().iter().filter(|p| needs_guard(p)) {
        assert!(!a.contains(&kind_key(&part.key)), "{}", part.key);
        assert_eq!(lowering_for(part, true, false).unwrap_err(), "it needs the guard rule");
    }
    let named = Founding { repertoire: Some(Repertoire { only: BTreeSet::from(["part:plated-shell".into()]), ..Default::default() }), ..founding(None, false) };
    assert!(named.generate().unwrap_err().contains("needs the guard rule"));
    // The rule adds no part that waits on another mechanic: the catalog's status is the rows'.
    assert!(catalog().parts().iter().filter(|p| needs_guard(p)).all(|p| matches!(p.status, crate::catalog::Status::Expressible { .. })));
}

#[test]
fn the_lowered_ports_carry_the_strengths_their_traits_give() {
    let g = guarded().generate().unwrap();
    let port = |k: &str| g.rules.kinds[k].port.unwrap();
    assert_eq!(port(PLATED).strength(), Strength { crush: 6, pierce: 5, spine: 3, toxin: 0 }, "armour from its traits, a spine's default from its effect");
    assert_eq!(port(QUILL).strength(), Strength { spine: 4, ..Strength::NONE }, "a spine's strength from its trait");
    assert_eq!(port(CYANIDE).strength(), Strength { toxin: 5, ..Strength::NONE });
    for k in [PLATED, QUILL, CYANIDE] {
        assert!(!port(k).is_mouth(), "{k} is an organ, not a mouth");
    }
    assert_eq!(port(JAW_STRONG).strength(), Strength { crush: 6, ..Strength::NONE }, "a mouth carries the catalog's attack");
    assert_eq!(port("kind:catalog-fauna-broad-mouth-graze-producer").strength(), Strength { crush: 2, ..Strength::NONE });
    // The piercing mouth, which the guard rule's by-kind attack made runnable, bites by pierce.
    let stylet = port("kind:catalog-fauna-piercing-graze-producer");
    assert!(stylet.is_mouth() && stylet.admits_live(NisKind::Producer));
    assert_eq!(stylet.strength(), Strength { pierce: 5, ..Strength::NONE });
    let part = catalog().part_by_key("part:fauna-piercing-graze-producer").unwrap();
    assert_eq!(lowering_for(part, true, false).unwrap_err(), "it needs the guard rule");
    assert!(port(ORGAN).tolerates(Tolerance::Toxin) && port(ORGAN).tolerates(Tolerance::DecomposerFlesh));
    // Without the rule nothing carries a strength or a toxin tolerance.
    let plain = founding(all(), false).generate().unwrap();
    assert!(plain.rules.kinds.values().filter_map(|t| t.port).all(|p| p.strength().is_none() && !p.tolerates(Tolerance::Toxin)));
    assert!(g.rules.validate().is_ok());
}

/// `lineage` grown with the kind `named` as its tail.
fn taking(g: Genesis, lineage: &str, kind: &str) -> Genesis {
    let name = format!("revision:repertoire-tail-{kind}");
    let d = offered(&g, lineage, &name);
    grown(g, lineage, d)
}

/// The strongest part of `lineage`'s founders' bodies by `read`, with the body.
fn carrying(g: &Genesis, lineage: &str) -> crate::schema::Entity {
    g.population.groups.values().find(|g| g.entity.lineage == lineage).unwrap().entity.as_ref().clone()
}

#[test]
fn a_line_that_takes_armour_or_a_spine_has_a_part_that_works() {
    let rule = Guard::default();
    let g = guarded().generate().unwrap();
    let plated = taking(g.clone(), "lineage:3", PLATED);
    let body = carrying(&plated, "lineage:3");
    let (id, _) = body.living().find(|(_, p)| active(p).is_some_and(|p| p.strength().crush == 6)).expect("the armour part is in the body");
    assert_eq!(rule.resisted(&body, id, rule.attack(IntakePort::none())), 4, "against a founded mouth");
    assert!(rule.spined(&body, id, rule.attack(IntakePort::none()), IntakePort::none()) > 0, "and the plates are spined");
    let quill = taking(g.clone(), "lineage:0", "kind:catalog-flora-spine-spine-prick");
    let body = carrying(&quill, "lineage:0");
    let (id, _) = body.living().find(|(_, p)| active(p).is_some_and(|p| p.strength().spine > 0)).expect("the spine is in the body");
    assert!(rule.spined(&body, id, rule.attack(IntakePort::none()), IntakePort::none()) > 0);
    // It is held for the guard's rent, and the founded body is not.
    let cells = |e: &crate::schema::Entity| e.living().map(|(_, p)| p.cells.get("function:support").copied().unwrap_or(0)).sum::<u32>();
    assert!(cells(&body) > 0 && cells(&carrying(&g, "lineage:0")) == 0);
}

fn gather(mut g: Genesis) -> Genesis {
    let site = *g.sites.keys().next().unwrap();
    for group in g.population.groups.values_mut().filter(|g| ["lineage:0", "lineage:3"].contains(&g.entity.lineage.as_str())) {
        group.entity_mut().place = site;
    }
    g
}

/// Lineage 3's meals of lineage 0 over 60 ticks: matter taken, the wards that acted, the rent paid, and whether the budget stayed closed.
fn grazed(g: Genesis) -> (u64, Vec<crate::flows::Harm>, u64, bool) {
    let mut s = session(g);
    let total = s.sim.budget().total();
    let (mut taken, mut harms, mut rent, mut closed) = (0, vec![], 0, true);
    for _ in 0..60 {
        let r = s.advance_tick_with_flows().unwrap();
        for f in &r.flows {
            let by = |n: &str| f.made_by == MadeBy::Process(n.into());
            if by("ecology:feed-3") && f.from_kind.as_ref().is_some_and(|k| k.lineage == "lineage:0") {
                taken += f.amount * f.count;
            }
            if by("guard:rent-0") && !f.internal() {
                rent += f.amount * f.count;
            }
        }
        harms.extend(s.sim.harms().iter().filter(|h| h.made_by == "ecology:feed-3" && h.ward == Some(Ward::Toxin)).cloned());
        closed &= s.sim.budget().total() == total;
    }
    (taken, harms, rent, closed)
}

#[test]
fn a_gland_a_plant_takes_harms_its_grazer_and_a_detox_organ_a_grazer_takes_spares_it() {
    let base = guarded().generate().unwrap();
    let (taken, harms, rent, closed) = grazed(gather(base.clone()));
    assert!(taken > 0 && harms.is_empty() && rent == 0 && closed, "the founded plant is not toxic and pays no rent");
    let poisoned = taking(base.clone(), "lineage:0", CYANIDE);
    let (taken, harms, rent, closed) = grazed(gather(poisoned.clone()));
    assert!(taken > 0, "the meal still happens");
    assert!(!harms.is_empty(), "and the grazer pays for the plant's gland");
    assert!(rent > 0, "the plant pays rent for its tissue");
    assert!(closed);
    let cured = taking(poisoned, "lineage:3", ORGAN);
    let (taken, harms, _, closed) = grazed(gather(cured));
    assert!(taken > 0 && harms.is_empty() && closed, "a grazer with the detox organ is spared");
}

#[test]
fn the_rule_is_the_same_in_both_executions_for_a_line_that_took_a_gland() {
    let hash = |mode| {
        let g = gather(taking(guarded().generate().unwrap(), "lineage:0", CYANIDE));
        let mut s = Session::new(g, mode).unwrap();
        s.advance(40).unwrap();
        s.sim.state_hash()
    };
    assert_eq!(hash(Execution::Individuals), hash(Execution::Grouped));
}
