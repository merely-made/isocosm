// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
use super::*;
use crate::{CanonSpec, GlyphDefinition};

fn unit() -> CostUnit {
    CostUnit {
        id: "demo:voxel".into(),
        label: "removed voxels".into(),
    }
}

fn declaration(n: u32) -> EffectDeclaration {
    EffectDeclaration {
        effect: format!("effect:reference-{n}"),
        display: format!("{n}"),
        behaviour: BehaviourKind::Inscribe,
        receiver: ReceiverClass::Terrain,
        cost: CostShape::PerUse { unit: unit() },
        explanation: "{actor} inscribes {amount} {unit}".into(),
    }
}

fn spec() -> EffectPackSpec {
    EffectPackSpec {
        version: 1,
        id: "world:demo-pack".into(),
        revision: 0,
        declarations: (0..5).map(declaration).collect(),
        limits: Default::default(),
    }
}

fn canon(count: u32) -> Canon {
    Canon::new(CanonSpec {
        version: 1,
        id: "world:demo".into(),
        revision: 0,
        glyphs: (0..count)
            .map(|n| GlyphDefinition {
                id: format!("glyph:{n}"),
                display: format!("{n}"),
                effect: format!("effect:reference-{n}"),
            })
            .collect(),
        variants: Vec::new(),
        limits: Default::default(),
    })
    .unwrap()
}

#[test]
fn a_declaration_round_trips_json_and_keeps_every_authored_field() {
    let pack = EffectPack::new(spec()).unwrap();
    let json = pack.to_json().unwrap();
    let reopened = EffectPack::from_json(&json).unwrap();
    assert_eq!(reopened, pack);
    assert_eq!(reopened.to_json().unwrap(), json);
    let found = reopened.declaration("effect:reference-3").unwrap();
    assert_eq!(found, &declaration(3));
    assert_eq!(found.cost.unit(), Some(&unit()));
    assert_eq!(CostShape::Free.unit(), None);
    assert!(reopened.declaration("effect:absent").is_none());
    // Every behaviour and receiver pole survives the round trip by name.
    let mut spec = spec();
    for (n, behaviour) in [
        BehaviourKind::Inscribe,
        BehaviourKind::Emit,
        BehaviourKind::Trail,
        BehaviourKind::Enclose,
    ]
    .into_iter()
    .enumerate()
    {
        spec.declarations[n].behaviour = behaviour;
        spec.declarations[n].receiver = [
            ReceiverClass::Terrain,
            ReceiverClass::Body,
            ReceiverClass::Bearer,
            ReceiverClass::None,
        ][n];
    }
    spec.declarations[4].cost = CostShape::WhileSustained { unit: unit() };
    let pack = EffectPack::new(spec).unwrap();
    assert_eq!(
        EffectPack::from_json(&pack.to_json().unwrap()).unwrap(),
        pack
    );
}

#[test]
fn covers_accepts_a_fully_declared_canon_and_names_the_first_undeclared_base() {
    let pack = EffectPack::new(spec()).unwrap();
    assert!(pack.covers(&canon(5)).is_ok());
    assert!(pack.covers(&canon(3)).is_ok());
    let missing = pack.covers(&canon(6)).unwrap_err();
    assert!(
        missing.contains("glyph:5") && missing.contains("effect:reference-5"),
        "{missing}"
    );
    // Variants carry no effect of their own, so they add no coverage duty.
    let mut with_variant = canon(5).spec().clone();
    with_variant.variants.push(crate::VariantDefinition {
        id: "glyph:variant".into(),
        display: "é".into(),
        base: "glyph:0".into(),
        modifiers: vec!["modifier:acute".into()],
    });
    assert!(pack.covers(&Canon::new(with_variant).unwrap()).is_ok());
}

#[test]
fn an_empty_pack_covers_only_an_empty_canon() {
    let mut empty = spec();
    empty.declarations.clear();
    let empty = EffectPack::new(empty).unwrap();
    assert!(empty.covers(&canon(1)).is_err());
    let mut bare = canon(1).spec().clone();
    bare.glyphs.clear();
    assert!(empty.covers(&Canon::new(bare).unwrap()).is_ok());
}

#[test]
fn identifier_and_bounded_text_rules_match_the_canon_and_admit_no_duplicates() {
    for bad in ["no-namespace", "world:", ":demo", "world:bad space"] {
        let mut spec = spec();
        spec.id = bad.into();
        assert!(EffectPack::new(spec.clone()).is_err(), "{bad}");
        spec = self::spec();
        spec.declarations[0].effect = bad.into();
        assert!(EffectPack::new(spec.clone()).is_err(), "{bad}");
        spec = self::spec();
        spec.declarations[0].cost = CostShape::WhileSustained {
            unit: CostUnit {
                id: bad.into(),
                label: "unit".into(),
            },
        };
        assert!(EffectPack::new(spec).is_err(), "{bad}");
    }
    let mut spec = spec();
    spec.declarations[0].display = String::new();
    assert!(EffectPack::new(spec).is_err());
    let mut spec = self::spec();
    spec.declarations[0].explanation = "x".repeat(4097);
    assert!(EffectPack::new(spec).is_err());
    let mut spec = self::spec();
    spec.declarations[0].cost = CostShape::PerUse {
        unit: CostUnit {
            id: "demo:voxel".into(),
            label: "x".repeat(257),
        },
    };
    assert!(EffectPack::new(spec).is_err());
    let mut duplicate = self::spec();
    duplicate.declarations[1].effect = duplicate.declarations[0].effect.clone();
    assert!(EffectPack::new(duplicate.clone()).is_err());
    assert!(
        serde_json::from_value::<EffectPack>(serde_json::to_value(duplicate).unwrap()).is_err(),
        "deserialization admits nothing new() refuses"
    );
    let mut version = self::spec();
    version.version = SCHEMA_VERSION + 1;
    assert!(EffectPack::new(version).is_err());
    let mut unknown = serde_json::to_value(self::spec()).unwrap();
    unknown["extra"] = true.into();
    assert!(serde_json::from_value::<EffectPack>(unknown).is_err());
}

#[test]
fn admission_and_json_bounds_are_explicit_and_configurable() {
    let mut limited = spec();
    limited.limits.declarations = 4;
    assert!(EffectPack::new(limited.clone()).is_err());
    limited.limits.declarations = 5;
    let pack = EffectPack::new(limited).unwrap();
    assert!(pack.to_json_with_limit(1).is_err());
    let json = pack.to_json().unwrap();
    assert!(EffectPack::from_json_with_limit(&json, json.len() - 1).is_err());
    assert_eq!(
        EffectPack::from_json_with_limit(&json, json.len()).unwrap(),
        pack
    );
    let clone = pack.clone();
    assert!(Arc::ptr_eq(&pack.spec, &clone.spec));
    assert!(Arc::ptr_eq(&pack.indices, &clone.indices));
    // Omitted limits restore the default, as CanonLimits does.
    let mut without = serde_json::to_value(spec()).unwrap();
    without.as_object_mut().unwrap().remove("limits");
    assert_eq!(
        serde_json::from_value::<EffectPack>(without)
            .unwrap()
            .spec()
            .limits,
        PackLimits::default()
    );
}
