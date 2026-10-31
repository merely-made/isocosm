use cosmograph::rules::*;
use isocosm_system::effect_catalogue::*;
use std::collections::BTreeMap;

fn process(effects: Vec<Effect>) -> Process {
    Process {
        id: "test:process".into(),
        causation: Causation::Choice,
        requires: vec![Query::Alive(Binding::Actor)],
        commitments: vec![],
        effects,
        risk: None,
        target: None,
        period: None,
        priority: 0,
        need_account: None,
        need_below: 0,
        glyphs: Default::default(),
        invariants: Default::default(),
        note: false,
        port_feeding: None,
        goal_directives: vec![],
    }
}

#[test]
fn catalogue_covers_its_stated_vocabulary_without_claiming_complete_editions() {
    let catalogue = builtin().unwrap();
    assert_eq!(catalogue.entries.len(), 54);
    assert_eq!(catalogue.family_counts().len(), Family::ALL.len());
    assert_eq!(catalogue.sources.len(), 3);
    let related = catalogue
        .entries
        .iter()
        .flat_map(|e| e.mappings.values())
        .filter(|m| m.relation == Correspondence::Related)
        .count();
    assert_eq!(related, 15);
    let definitions = isocosm_system::active_effects::Definitions::builtin().unwrap();
    catalogue.validate_procedures(&definitions).unwrap();
    let bound: std::collections::BTreeSet<_> = catalogue
        .entries
        .iter()
        .flat_map(|e| e.mappings.values())
        .flat_map(|m| m.procedures.iter())
        .collect();
    assert_eq!(bound.len(), 9);
    assert!(definitions.0.keys().all(|id| bound.contains(id)));
    let mut bad = catalogue.clone();
    bad.entries
        .iter_mut()
        .find(|e| e.id == "wing:detect")
        .unwrap()
        .mappings
        .get_mut("ars5")
        .unwrap()
        .procedures = vec!["ars5:edge-of-the-razor".into()];
    assert!(bad.validate_procedures(&definitions).is_err());
    assert!(catalogue.entry("wing:enchant").unwrap().native.is_empty());
    assert!(catalogue.entry("wing:resize").unwrap().native.is_empty());
    assert!(
        catalogue
            .entry("wing:dimensional-access")
            .unwrap()
            .native
            .is_empty()
    );
    assert!(catalogue.scope.contains("Not a complete inventory"));
    let saved = toml::to_string(&catalogue).unwrap();
    let loaded = Catalogue::from_toml(&saved).unwrap();
    assert_eq!(catalogue.revision(), loaded.revision());
    let mut reordered = catalogue.clone();
    reordered.entries.reverse();
    reordered.sources.reverse();
    reordered.families.reverse();
    assert_eq!(
        Catalogue::from_toml(&toml::to_string(&reordered).unwrap()).unwrap(),
        catalogue
    );
}

#[test]
fn admission_names_gaps_and_refuses_unsupported_claims() {
    let good = builtin().unwrap();
    let refused = |mutate: fn(&mut Catalogue)| {
        let mut changed = good.clone();
        mutate(&mut changed);
        changed.validate().unwrap_err()
    };
    assert!(refused(|c| c.entries.push(c.entries[0].clone())).contains("duplicate effect"));
    assert!(
        refused(|c| c.sources.push(c.sources[0].clone())).contains("duplicate catalogue source")
    );
    assert!(
        refused(|c| {
            c.entries[0].mappings.pop_first();
        })
        .contains("every declared edition")
    );
    assert!(
        refused(|c| {
            c.entries[0].mappings.insert(
                "unknown".into(),
                Mapping {
                    relation: Correspondence::Unmapped,
                    references: vec![],
                    differences: vec![],
                    procedures: vec![],
                },
            );
        })
        .contains("every declared edition")
    );
    assert!(
        refused(|c| {
            let mapping = c
                .entries
                .iter_mut()
                .find(|e| e.id == "wing:enchant")
                .unwrap()
                .mappings
                .values_mut()
                .next()
                .unwrap();
            mapping.references.clear();
        })
        .contains("source reference")
    );
    assert!(refused(|c| c.version += 1).contains("unsupported"));
    let source = include_str!("../data/effects.toml");
    assert!(
        Catalogue::from_toml(&source.replacen(
            "operation = \"allocate\"",
            "operation = \"invented\"",
            1
        ))
        .is_err()
    );
    assert!(
        Catalogue::from_toml(&source.replacen(
            "relation = \"related\"",
            "relation = \"implemented\"",
            1
        ))
        .is_err()
    );
    assert!(
        Catalogue::from_toml(&source.replacen("version = 1", "version = 1\nimplemented = true", 1))
            .is_err()
    );
}

#[test]
fn sensory_binding_checks_both_the_bound_part_and_allocated_function() {
    let catalogue = builtin().unwrap();
    let binding = &catalogue.entry("wing:allocate-sense").unwrap().native[0];
    let mut p = process(vec![Effect::Allocate {
        from: None,
        to: "function:sense".into(),
        cells: 1.into(),
    }]);
    assert!(
        !binding.matches(&p),
        "an unbound allocation is not a sense candidate"
    );
    p.requires.push(Query::Expresses {
        function: "function:sense".into(),
    });
    assert!(binding.matches(&p));
    p.effects = vec![Effect::Allocate {
        from: None,
        to: "function:secrete".into(),
        cells: 1.into(),
    }];
    assert!(
        !binding.matches(&p),
        "an unrelated gland is not a sensory binding"
    );
}

#[test]
fn inspection_includes_commitments_guards_and_risks_and_never_changes_rules() {
    let catalogue = builtin().unwrap();
    let mut p = process(vec![Effect::When {
        guard: Expr::Const(0),
        then: vec![Effect::Death],
        otherwise: vec![Effect::Grow {
            from: "world:soil".into(),
            into: "world:soil".into(),
            conversion: Conversion::Synthesis,
        }],
    }]);
    p.commitments.push(Effect::Practice {
        key: "skill:test".into(),
        amount: 1.into(),
    });
    p.risk = Some(Risk {
        per_million: 1,
        effects: vec![Effect::Wound {
            who: Binding::Actor,
            cells: 1.into(),
            slot: 0,
        }],
    });
    assert_eq!(
        p.effect_kinds(),
        [
            EffectKind::When,
            EffectKind::Death,
            EffectKind::Grow,
            EffectKind::Practice,
            EffectKind::Wound
        ]
        .into()
    );
    let mut g = cosmograph::Founding::default().generate().unwrap();
    g.rules.processes = BTreeMap::from([(p.id.clone(), p)]);
    let before = g.rules.revision();
    assert_eq!(
        catalogue
            .entry("wing:grow-body")
            .unwrap()
            .native_candidates(&g.rules),
        ["test:process"]
    );
    assert_eq!(
        catalogue
            .entry("wing:wound")
            .unwrap()
            .native_candidates(&g.rules),
        ["test:process"]
    );
    assert!(
        catalogue
            .entry("wing:enchant")
            .unwrap()
            .native_candidates(&g.rules)
            .is_empty()
    );
    assert_eq!(
        before,
        g.rules.revision(),
        "inspection cannot admit an act or change rules"
    );
}
