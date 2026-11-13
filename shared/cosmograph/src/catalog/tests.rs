// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The catalog's validation: the memo's four distinctness tests as far as
//! a datasheet can state them, and the owner's coverage floor (D39).

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use super::refs::{KINGDOMS, MAIN_KINGDOMS, RULES};
use super::*;

#[test]
fn the_catalog_is_consistent() {
    let problems = catalog().problems();
    assert!(problems.is_empty(), "{} problems, first ten: {:#?}", problems.len(), &problems[..problems.len().min(10)]);
}

#[test]
fn keys_are_unique_and_every_reference_exists() {
    let c = catalog();
    let traits: BTreeSet<_> = c.traits().iter().map(|t| &t.key).collect();
    let effects: BTreeSet<_> = c.effects().iter().map(|e| &e.key).collect();
    let parts: BTreeSet<_> = c.parts().iter().map(|p| &p.key).collect();
    assert_eq!(traits.len(), c.traits().len());
    assert_eq!(effects.len(), c.effects().len());
    assert_eq!(parts.len(), c.parts().len());
    for p in c.parts() {
        assert!(p.traits.iter().all(|t| traits.contains(t)), "{}", p.key);
        assert!(p.effect_keys().all(|e| effects.contains(e)), "{}", p.key);
        assert!(KINGDOMS.contains(&p.kingdom.as_str()), "{}", p.key);
    }
}

#[test]
fn every_rule_a_row_cites_exists_in_the_code() {
    // Source files the `rule:` references stand for.
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    for (name, file) in RULES {
        assert!(src.join(file).is_file(), "rule:{name} names missing src/{file}");
    }
    let c = catalog();
    for t in c.traits() {
        if let Status::Expressible { via } = &t.status {
            assert!(via.iter().all(|r| refs::resolves(r)), "{}", t.key);
        }
    }
    for p in c.parts().iter().filter(|p| matches!(p.status, Status::Expressible { .. })) {
        let Status::Expressible { via } = &p.status else { unreachable!() };
        assert!(!via.is_empty() && via.iter().all(|r| refs::resolves(r)), "{}", p.key);
        assert!(p.waits_on.is_empty(), "{}", p.key);
    }
}

#[test]
fn no_two_parts_share_a_fingerprint() {
    let mut seen: BTreeMap<String, &str> = BTreeMap::new();
    for p in catalog().parts() {
        if let Some(first) = seen.insert(p.fingerprint(), &p.key) {
            panic!("{} and {first} are one thing with two names", p.key);
        }
    }
}

#[test]
fn the_floor_the_owner_set_holds() {
    let c = catalog();
    assert!(c.traits().len() >= 40, "traits: {}", c.traits().len());
    assert!(c.effects().len() >= 30, "effects: {}", c.effects().len());
    assert!(c.parts().len() >= 120, "parts: {}", c.parts().len());
    for model in Model::ALL {
        let n = c.parts_serving(model).count();
        assert!(n >= 8, "{model:?} is served by {n} parts");
    }
    for kingdom in MAIN_KINGDOMS {
        let n = c.parts_for(kingdom).count();
        assert!(n >= 30, "{kingdom} has {n} parts");
    }
}

#[test]
fn every_site_and_status_is_used() {
    let c = catalog();
    let sites: BTreeSet<_> = c.effects().iter().map(|e| e.site).collect();
    assert_eq!(sites.len(), Site::ALL.len());
    let words: BTreeSet<_> = c.parts().iter().map(|p| p.status.word()).collect();
    assert_eq!(words.len(), 3, "{words:?}");
}

#[test]
fn generation_is_deterministic_and_leaves_authored_parts_first() {
    let c = catalog();
    let authored: Vec<_> = c.parts().iter().filter(|p| p.origin == Origin::Authored).cloned().collect();
    let (a, da) = generate(c.traits(), c.effects(), &authored);
    let (b, db) = generate(c.traits(), c.effects(), &authored);
    assert_eq!(da, db);
    assert_eq!(a.iter().map(|p| &p.key).collect::<Vec<_>>(), b.iter().map(|p| &p.key).collect::<Vec<_>>());
    let generated: Vec<_> = c.parts().iter().filter(|p| p.origin == Origin::Generated).collect();
    assert_eq!(generated.len(), a.len());
    assert!(c.parts().iter().take(authored.len()).all(|p| p.origin == Origin::Authored));
}

#[test]
fn the_borrowed_shapes_are_used() {
    let c = catalog();
    assert!(c.effects().iter().any(|e| e.gates.iter().any(|g| g.inverted)), "an inverted gate");
    assert!(c.effects().iter().filter(|e| e.timing.is_some()).count() >= 5);
    assert!(c.traits().iter().filter(|t| !t.unlocks.is_empty()).count() >= 5);
    assert!(c.traits().iter().filter(|t| !t.strengths.is_empty()).count() >= 10);
    // D37: decomposer flesh harms an eater that lacks tolerance.
    let harm = c.effect_by_key("effect:category-harm").unwrap();
    assert!(harm.gates.iter().any(|g| g.inverted && g.requires == "trait:detox-tolerance"));
}

#[test]
fn accessors_agree_with_the_catalog() {
    assert_eq!(parts().len(), catalog().parts().len());
    assert_eq!(traits().len(), catalog().traits().len());
    assert_eq!(effects().len(), catalog().effects().len());
    assert_eq!(parts_for("kingdom:flora").count(), catalog().parts_for("kingdom:flora").count());
    assert!(parts_serving(Model::Detritus).all(|p| p.models.contains(&Model::Detritus)));
}

/// `cargo test -p cosmograph --release catalog_measurement -- --ignored --nocapture`
#[test]
#[ignore = "measurement, prints the catalog's counts"]
fn catalog_measurement() {
    let c = catalog();
    println!("traits {}, effects {}, parts {}", c.traits().len(), c.effects().len(), c.parts().len());
    println!("generator: {:?}", c.dropped());
    let authored = c.parts().iter().filter(|p| p.origin == Origin::Authored).count();
    println!("authored {authored}, generated {}", c.parts().len() - authored);
    for k in KINGDOMS {
        println!("  {k}: {}", c.parts_for(k).count());
    }
    for m in Model::ALL {
        println!("  {m:?}: {}", c.parts_serving(m).count());
    }
    let mut by_status: BTreeMap<&str, usize> = BTreeMap::new();
    for p in c.parts() {
        *by_status.entry(p.status.word()).or_default() += 1;
    }
    println!("status {by_status:?}");
    for (mechanic, n) in c.waiting_on() {
        println!("  waits on {mechanic}: {n}");
    }
    let mut proposed: BTreeMap<&str, usize> = BTreeMap::new();
    for p in c.parts().iter().filter(|p| p.status.word() == "proposed") {
        *proposed.entry(&p.process).or_default() += 1;
    }
    println!("proposed processes {proposed:?}");
    let thin: Vec<_> = c.traits().iter().filter(|t| t.reads.len() < 3).map(|t| &t.key).collect();
    println!("traits read by fewer than three kernels (memo test 1): {}", thin.len());
    let unused: Vec<_> =
        c.traits().iter().filter(|t| !c.parts().iter().any(|p| p.traits.contains(&t.key))).map(|t| &t.key).collect();
    println!("traits in no part: {unused:?}");
    let unused: Vec<_> =
        c.effects().iter().filter(|e| !c.parts().iter().any(|p| p.effect_keys().any(|k| *k == e.key))).map(|e| &e.key).collect();
    println!("effects in no part: {unused:?}");
    if std::env::var_os("CATALOG_LIST").is_some() {
        for p in c.parts().iter().filter(|p| p.origin == Origin::Generated) {
            println!("{} [{}] {:?}", p.key, p.status.word(), p.models);
        }
    }
}
