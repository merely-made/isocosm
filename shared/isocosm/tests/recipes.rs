// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Checkpoint 8, step 8a: what bodies develop from as world data (rulings
//! 478, 510, 511, 516, 518, 530 and 531). Kinds are part templates in the
//! rules, a lineage keeps its recipe, policy, domain, lexicon and clutch,
//! a part keeps its offset, and all of it round-trips; a world without any
//! of it serializes as before.

use isocosm::{
    digest,
    probe::BodyFounding,
    rules::{AccountKind, Affinity, Development, Policy, Recipe, Tagma, Template, Verdict},
    simulation::Genesis,
};
use std::collections::BTreeSet;

fn world() -> Genesis {
    BodyFounding::default().generate().unwrap().genesis
}

fn template(half_extent: [i32; 3], cells: &[(&str, u32)]) -> Template {
    Template {
        half_extent,
        cells: cells
            .iter()
            .map(|(f, n)| (format!("function:{f}"), *n))
            .collect(),
        shape: String::new(),
    }
}

/// The grazer's body as a recipe: a lump bearing a pair of limbs.
fn developed() -> Genesis {
    let mut g = world();
    g.rules.kinds.extend([
        (
            "kind:lump".into(),
            template([2, 2, 2], &[("intake", 5), ("store", 2), ("reproduce", 1)]),
        ),
        ("kind:limb".into(), template([3, 1, 1], &[("contract", 2)])),
    ]);
    g.rules.affinity = Some(Affinity::default());
    let recipe = Recipe {
        tagmata: vec![Tagma {
            segments: 1,
            segment: "kind:lump".into(),
            bears: Some("kind:limb".into()),
            per_segment: 1,
            parent: None,
            anchor: Default::default(),
            facing: isocosm::rules::Facing::Back,
            socket: isocosm::rules::Facing::Right,
            variance: None,
        }],
        variance: 1,
        absence: [1, 12],
    };
    let lexicon = recipe.kinds();
    g.lineages.get_mut("lineage:1").unwrap().development = Some(Development {
        recipe,
        policy: Policy::default(),
        domain: 1,
        lexicon,
        clutch: 3,
        anamorphic: false,
    });
    g
}

#[test]
fn development_round_trips_and_a_world_without_it_serializes_as_before() {
    let g = developed();
    g.validate().unwrap();
    let text = serde_json::to_string(&g).unwrap();
    let back: Genesis = serde_json::from_str(&text).unwrap();
    assert_eq!(back, g);
    assert_eq!(digest(&back), digest(&g));
    // The control: a world that asks for no bodies, the ecology family's
    // (525), names none of the new fields.
    let plain = serde_json::to_string(&isocosm::Founding::default().generate().unwrap()).unwrap();
    let named = |text: &str, field: &str| text.contains(&format!("\"{field}\""));
    for field in ["kinds", "affinity", "development"] {
        assert!(named(&text, field), "{field}");
    }
    for field in ["kinds", "affinity", "development", "offset"] {
        assert!(!named(&plain, field), "{field}");
    }
    // Its rates name a provision of their own; no account is one, as the
    // probe's are.
    let account = "\"provision\":true";
    assert!(text.contains(account) && !plain.contains(account));
}

#[test]
fn affinity_is_mesocosms_three_domains_each_favouring_the_next() {
    let a = Affinity::default();
    assert_eq!(a.verdict(1, 1), Verdict::Native);
    assert_eq!(a.verdict(0, 1), Verdict::Adapter);
    assert_eq!(a.verdict(2, 0), Verdict::Adapter);
    assert_eq!(a.verdict(1, 0), Verdict::Refused);
    assert_eq!(
        a.verdict(3, 3),
        Verdict::Refused,
        "a domain the world lacks"
    );
}

#[test]
fn the_policy_tries_the_preferred_facing_its_mirror_and_then_its_tolerance() {
    use isocosm::rules::Facing::*;
    let p = Policy::default();
    assert_eq!(p.candidates("part-shape:rod"), [Right, Left, Front]);
    assert_eq!(p.candidates("part-shape:lump"), [Below, Front, Back]);
    let none = Policy {
        tolerance: 0,
        ..Policy::default()
    };
    assert_eq!(none.candidates("part-shape:rod"), [Right, Left]);
    assert_eq!(none.candidates("part-shape:sheet"), [Above]);
}

#[test]
fn what_a_development_cannot_be_is_refused() {
    let refused = |change: &dyn Fn(&mut Genesis)| {
        let mut g = developed();
        change(&mut g);
        g.validate().err().unwrap_or_default()
    };
    assert_eq!(refused(&|_| {}), "", "the control validates");
    let dev = |g: &mut Genesis| {
        g.lineages
            .get_mut("lineage:1")
            .unwrap()
            .development
            .as_mut()
            .unwrap()
            .clone()
    };
    let set = |g: &mut Genesis, d: Development| {
        g.lineages.get_mut("lineage:1").unwrap().development = Some(d);
    };
    // A kind with more cells than its box holds (a [2,2,2] lump holds 8).
    let over = refused(&|g| {
        let lump = g.rules.kinds.get_mut("kind:lump").unwrap();
        lump.cells.insert("function:store".into(), 3);
    });
    assert!(over.contains("more cells"), "{over}");
    // A recipe naming a kind its lexicon lacks.
    let unlearned = refused(&|g| {
        let mut d = dev(g);
        d.lexicon = BTreeSet::from(["kind:lump".to_string()]);
        set(g, d);
    });
    assert!(unlearned.contains("lexicon lacks"), "{unlearned}");
    // A domain the world's three lack.
    let foreign = refused(&|g| {
        let mut d = dev(g);
        d.domain = 3;
        set(g, d);
    });
    assert!(foreign.contains("domain"), "{foreign}");
    // Absence odds over one.
    let odds = refused(&|g| {
        let mut d = dev(g);
        d.recipe.absence = [13, 12];
        set(g, d);
    });
    assert!(odds.contains("absence"), "{odds}");
    // A clutch of none.
    let clutch = refused(&|g| {
        let mut d = dev(g);
        d.clutch = 0;
        set(g, d);
    });
    assert!(clutch.contains("clutches"), "{clutch}");
    // An account both a reserve and a provision.
    let both = refused(&|g| {
        g.rules.accounts.insert(
            "reserve:1".into(),
            AccountKind::Matter {
                lineage: "lineage:1".into(),
                reserve: true,
                provision: true,
            },
        );
    });
    assert!(both.contains("both a reserve and a provision"), "{both}");
}
