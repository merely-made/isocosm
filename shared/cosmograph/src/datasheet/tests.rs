// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::native;
use super::*;

#[test]
fn every_founding_palette_resolves_and_the_default_names_one() {
    let sheets = sheets();
    for (name, founding) in &sheets.foundings.founding {
        let banks = sheets.banks(&founding.palette).expect(name);
        assert!(
            Bank::ALL.iter().all(|b| !banks.bank(*b).is_empty()),
            "{name}"
        );
    }
    assert!(sheets.founding(native::default_founding()).is_ok());
}

#[test]
fn the_default_roster_lowers_onto_its_own_kinds() {
    let founding = native::default_founding();
    let palette = &sheets().founding(founding).unwrap().palette;
    let kinds = native::kinds(palette).unwrap();
    let roster = native::founding(founding).unwrap();
    assert_eq!(roster.len(), 8, "Mesocosm's eight authored bodies (512)");
    assert_eq!(roster.iter().filter(|r| r.1).count(), 3, "three producers");
    for (name, _, recipe) in &roster {
        assert!(
            recipe.kinds().iter().all(|k| kinds.contains_key(k)),
            "{name}"
        );
    }
}

#[test]
fn richer_rosters_develop_paid_serial_chains_and_keep_old_recipe_bytes() {
    use crate::rules::*;
    let old = native::founding("Roster").unwrap();
    let bytes = serde_json::to_string(&old[0].2).unwrap();
    assert!(!bytes.contains("chain"));
    assert_eq!(serde_json::from_str::<Recipe>(&bytes).unwrap(), old[0].2);
    for name in ["BranchingRoster", "JointedRoster", "SpacedRoster"] {
        let sheets = sheets();
        let kinds = native::kinds(&sheets.founding(name).unwrap().palette).unwrap();
        let mut rules = crate::probe::BodyFounding::default()
            .generate()
            .unwrap()
            .genesis
            .rules;
        rules.kinds = kinds;
        for (body, _, recipe) in native::founding(name).unwrap() {
            let d = Development {
                lexicon: recipe.kinds(),
                recipe,
                policy: Policy::default(),
                domain: 0,
                clutch: 1,
                anamorphic: false,
                tracts: vec![],
                systems: Default::default(),
            };
            let soma = crate::development::soma(&rules, &d.recipe, 123);
            let grown = crate::development::develop(&rules, &d, &soma).expect(&body);
            let mut situs = std::collections::BTreeSet::new();
            for p in grown.doc.living() {
                assert!(situs.insert(p.situs.unwrap()), "duplicate situs in {body}");
                assert!(grown.parts.contains_key(&p.id));
            }
            if body.ends_with("consumer_browser") && name != "BranchingRoster" {
                let feet: Vec<_> = grown
                    .doc
                    .living()
                    .filter(|p| p.half_extent == [1, 1, 3])
                    .collect();
                assert_eq!(feet.len(), 6, "{body}");
                for foot in feet {
                    let lower = grown.doc.part(foot.attachment.unwrap().parent).unwrap();
                    assert_eq!(lower.half_extent, [1, 3, 1]);
                    let upper = grown.doc.part(lower.attachment.unwrap().parent).unwrap();
                    assert_eq!(upper.half_extent, [3, 1, 1]);
                }
            }
        }
    }
}

#[test]
fn explicit_rich_founding_is_native_valid_and_seed_reproducible() {
    let mut founding = crate::Founding::default();
    founding.seed = 734;
    founding.population = 12;
    founding.bodies = Some(crate::bodied::Bodies::default());
    let bodies = founding.bodies.as_mut().unwrap();
    bodies.founding = Some("SpacedRoster".into());
    let first = founding.generate().unwrap();
    assert_eq!(
        crate::digest(&first),
        crate::digest(&founding.generate().unwrap())
    );
    crate::Simulation::new(first, crate::Execution::Grouped).unwrap();
}
