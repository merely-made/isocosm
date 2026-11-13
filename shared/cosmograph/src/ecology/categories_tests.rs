// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Categories as a founding rule: unset it changes nothing, set it round
//! trips, bad values are named, and which tissue is tough is read from the
//! part (`ecology::categories`).

use super::spoilage_tests::founding as spoiled;
use super::*;
use crate::process::{DeadClass, IntakePort, NisKind, Tolerance};

pub(super) fn founding(categories: Option<Categories>) -> Founding {
    Founding { categories, ..spoiled(None) }
}

#[test]
fn unset_categories_found_and_hash_as_before() {
    let plain = founding(None);
    let json = serde_json::to_string(&plain).unwrap();
    assert!(!json.contains("categories"), "no mention of the rule");
    let bare: Founding = serde_json::from_str(&json).unwrap();
    let (a, b) = (plain.generate().unwrap(), bare.generate().unwrap());
    assert_eq!(crate::digest(&a), crate::digest(&b));
    assert!(a.rules.categories.is_none());
    assert!(!serde_json::to_string(&a.rules).unwrap().contains("categories"));
    // The port's new fields are absent from the wire until set.
    let json = serde_json::to_string(&IntakePort::deadstock()).unwrap();
    assert!(!json.contains("classes") && !json.contains("tolerance"), "{json}");
    assert!(a.population.groups.values().flat_map(|g| g.entity.parts.values()).all(|p| p.port.is_none_or(|p| p.class_bits() == 0)));
}

#[test]
fn categories_round_trip_and_the_defaults_are_not_written() {
    let f = founding(Some(Categories::default()));
    let json = serde_json::to_string(&f).unwrap();
    assert!(json.contains("\"categories\"") && !json.contains("wood_rot_every"));
    assert_eq!(serde_json::from_str::<Founding>(&json).unwrap(), f);
    let slow = Categories { wood_rot_every: 7, ..Categories::default() };
    let json = serde_json::to_string(&slow).unwrap();
    assert!(json.contains("wood_rot_every"));
    assert_eq!(serde_json::from_str::<Categories>(&json).unwrap(), slow);
    let g = f.generate().unwrap();
    assert_eq!(g.rules.categories, Some(Categories::default()));
    // The rule alone changes no founded mouth, as it only reads them at the bite.
    let plain = founding(None).generate().unwrap();
    assert_eq!(g.rules.kinds, plain.rules.kinds);
    assert!(g.population.groups.values().flat_map(|g| g.entity.parts.values()).all(|p| p.port.is_none_or(|p| p.tolerance_bits() == 0)));
}

#[test]
fn categories_name_what_they_cannot_found() {
    let bad = |c: Categories| founding(Some(c)).generate().unwrap_err();
    assert!(bad(Categories { harm: 0, ..Categories::default() }).contains("harm"));
    assert!(bad(Categories { wood_rot_every: 0, ..Categories::default() }).contains("wood_rot_every"));
    let world_time = Founding { ecology_rates: Rates::WorldTime, ..founding(Some(Categories::default())) };
    assert!(world_time.generate().unwrap_err().contains("world time"));
    let no_ecology = Founding { ecology: false, ..founding(Some(Categories::default())) };
    assert!(no_ecology.generate().unwrap_err().contains("ecology"));
}

#[test]
fn a_plants_tough_tissue_is_what_holds_no_fixing_cell_or_holds_a_supporting_one() {
    let g = founding(Some(Categories::default())).generate().unwrap();
    let plant = g.population.groups.values().map(|g| g.entity.as_ref()).find(|e| e.lineage == "lineage:0").unwrap();
    let (mut soft, mut hard) = (0, 0);
    for (id, part) in &plant.parts {
        let fixes = part.cells.get("function:fix").copied().unwrap_or(0) > 0;
        assert_eq!(tough(plant, *id), !fixes, "{id:?}: {:?}", part.cells);
        if fixes { soft += 1 } else { hard += 1 }
    }
    assert!(soft > 0 && hard > 0, "a plant has leaf plates and stalk: {soft} soft, {hard} tough");
    // A supporting cell makes even a fixing part tough.
    let mut braced = plant.clone();
    let id = *braced.parts.iter().find(|(_, p)| p.cells.contains_key("function:fix")).unwrap().0;
    braced.part_mut(&id).unwrap().cells.insert("function:support".into(), 1);
    assert!(tough(&braced, id));
    // No animal's tissue is tough, and a part that is not there is not.
    let animal = g.population.groups.values().map(|g| g.entity.as_ref()).find(|e| e.lineage == "lineage:3").unwrap();
    assert!(animal.parts.keys().all(|id| !tough(animal, *id)));
    assert!(!tough(plant, crate::schema::PartId(9999)));
}

#[test]
fn a_dead_stock_mouth_is_limited_to_the_classes_it_takes() {
    let any = IntakePort::deadstock();
    assert!(DeadClass::ALL.iter().all(|c| any.takes(*c)), "no limit takes every class");
    let litter = any.taking(DeadClass::Litter);
    assert!(litter.takes(DeadClass::Litter) && !litter.takes(DeadClass::Wood) && !litter.takes(DeadClass::Carrion));
    let both = litter.taking(DeadClass::Wood);
    assert!(both.takes(DeadClass::Litter) && both.takes(DeadClass::Wood) && !both.takes(DeadClass::Carrion));
    let json = serde_json::to_string(&litter).unwrap();
    assert_eq!(serde_json::from_str::<IntakePort>(&json).unwrap(), litter);
    // The rule gives a mouth for the dead the grinding to work wood; a plant mouth none.
    let rule = Categories::default();
    assert!(rule.founded(IntakePort::deadstock()).tolerates(Tolerance::ToughPlant));
    assert!(!rule.founded(IntakePort::live(NisKind::Producer)).tolerates(Tolerance::ToughPlant));
    assert!(rule.rests(1) == false && Categories { wood_rot_every: 5, ..rule }.rests(1));
}
