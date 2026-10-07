// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The sheets against the code they replace, and what a sheet may not say.

use super::*;
use crate::legacy::mesocosm::axis::archetype;
use crate::legacy::mesocosm::world::genesis::Founding;

const ALL: [Founding; 8] = [
    Founding::Drawn,
    Founding::BrowsingConsumer,
    Founding::RosterStand,
    Founding::RosterFauna,
    Founding::Roster,
    Founding::BranchingRoster,
    Founding::JointedRoster,
    Founding::SpacedRoster,
];
const KINGDOMS: [Kingdom; 3] = [Kingdom::Producer, Kingdom::Consumer, Kingdom::Decomposer];

fn code_tier(founding: Founding, kingdom: Kingdom) -> Vec<Recipe> {
    founding.tier(kingdom).iter().map(|body| body()).collect()
}

#[test]
fn every_founding_sheet_builds_the_codes_palette_and_bodies() {
    let sheets = foundings();
    for founding in ALL {
        let name = format!("{founding:?}");
        let sheet = sheets
            .get(&name)
            .unwrap_or_else(|| panic!("no sheet for {name}"));
        assert_eq!(sheet.palette, founding.palette(), "{name}: palette");
        for kingdom in KINGDOMS {
            assert_eq!(
                sheet.tier(kingdom),
                code_tier(founding, kingdom),
                "{name}, {kingdom:?}"
            );
        }
    }
}

#[test]
fn the_sheets_name_the_eight_foundings_and_the_default() {
    let mut expected: Vec<String> = ALL.iter().map(|founding| format!("{founding:?}")).collect();
    expected.sort();
    assert_eq!(foundings().names().collect::<Vec<_>>(), expected);
    assert_eq!(foundings().default, format!("{:?}", Founding::default()));
}

#[test]
fn the_primitive_palette_is_the_codes() {
    assert_eq!(
        foundings().palette("primitive"),
        Some(PartPalette::primitive())
    );
}

#[test]
fn a_changed_sheet_is_caught() {
    // The control for the equality test above: the mat's pads become fronds,
    // which every palette admits, and the Roster's first producer must stop
    // matching the code.
    let base = SETS[0]
        .1
        .replacen("shape = \"pad\"", "shape = \"frond\"", 1);
    assert_ne!(base, SETS[0].1, "the mat names its pads");
    let mut sets = SETS;
    sets[0].1 = &base;
    let changed = load(&sets, FOUNDINGS.1).expect("a frond is admitted");
    let roster = changed.get("Roster").expect("the roster");
    assert_ne!(roster.producer[0], archetype::producer_mat());
    assert_eq!(roster.producer[1], archetype::producer_shrub());
}

const KEYS: &str = "schema = 1\nowner = \"test\"\nconsumer = \"test\"\nstatus = \"test\"\n";
const SOURCES: &str = "[sources.test]\ndoc = \"none\"\nnote = \"a test sheet\"\n";
const PALETTE: &str = r#"
[palettes.p]
[[palettes.p.mass]]
name = "block"
tag = 1
half_extent = [2, 2, 2]
[[palettes.p.limb]]
name = "rod"
tag = 2
half_extent = [4, 1, 1]
[[palettes.p.plate]]
name = "frond"
tag = 3
half_extent = [4, 4, 1]
[[palettes.p.sensor]]
name = "eye"
tag = 4
half_extent = [1, 1, 1]
"#;

/// Loads one test sheet whose palette is `p` and whose body `b` founds `F`.
fn try_load(tables: &str) -> Result<Foundings, String> {
    let set = format!("{KEYS}{SOURCES}{tables}");
    let founding = format!(
        "{KEYS}default = \"F\"\n{SOURCES}[founding.F]\npalette = \"p\"\nconsumer = [\"t.b\"]\n"
    );
    load(&[("t", &set)], &founding)
}

fn refused(tables: &str, words: &str) {
    let why = try_load(tables).expect_err("refused");
    assert!(why.contains(words), "{why:?} does not say {words:?}");
}

#[test]
fn the_test_sheet_loads_with_its_encodings() {
    let sheets = try_load(&format!(
        "{PALETTE}[body.b]\nvariance = 0\n\
         [[body.b.tagma]]\nsegments = 1\nappendage = \"mouth\"\nmouth = \"jaw\"\n\
         [[body.b.tagma]]\nsegments = 2\nappendage = \"plate\"\nshape = \"frond\"\nworn = \"covering\"\n"
    ))
    .expect("loads");
    let body = &sheets.get("F").expect("F").consumer[0];
    assert_eq!(body.tagmata[0].appendage_shape, JAW_SHAPE);
    assert_eq!(body.tagmata[1].appendage_shape, ARMOUR_SHAPE);
    assert!(body.layout.is_empty() && body.appendage_chains.is_empty());
}

#[test]
fn a_shape_the_palette_does_not_admit_is_refused() {
    let body = "[body.b]\nvariance = 0\n[[body.b.tagma]]\nsegments = 1\nsegment = \"slim\"\n";
    refused(&format!("{PALETTE}{body}"), "admits no Mass shape \"slim\"");
}

#[test]
fn a_misspelt_key_is_refused() {
    let body = "[body.b]\nvariance = 0\n[[body.b.tagma]]\nsegments = 1\nsegmnet = \"block\"\n";
    refused(&format!("{PALETTE}{body}"), "unknown field");
}

#[test]
fn a_layout_places_every_tagma_or_none() {
    let body = "[body.b]\nvariance = 0\n\
                [[body.b.tagma]]\nsegments = 1\nlayout = { anchor = \"tip\", facing = \"back\" }\n\
                [[body.b.tagma]]\nsegments = 1\n";
    refused(&format!("{PALETTE}{body}"), "every tagma or none");
}

#[test]
fn only_a_plate_is_worn() {
    let body = "[body.b]\nvariance = 0\n\
                [[body.b.tagma]]\nsegments = 1\nappendage = \"limb\"\nworn = \"covering\"\n";
    refused(&format!("{PALETTE}{body}"), "only a plate is worn");
}

#[test]
fn a_full_bank_refuses_another_shape() {
    let more: String = (5..9)
        .map(|tag| {
            format!(
                "[[palettes.p.mass]]\nname = \"m{tag}\"\ntag = {tag}\nhalf_extent = [1, 1, 1]\n"
            )
        })
        .collect();
    let body = "[body.b]\nvariance = 0\n[[body.b.tagma]]\nsegments = 1\n";
    refused(&format!("{PALETTE}{more}{body}"), "holds 4 shapes");
}

#[test]
fn a_replacement_names_a_slot_that_exists() {
    let extended = "[palettes.q]\nextends = \"p\"\n\
                    [[palettes.q.plate]]\nreplaces = \"blade\"\nname = \"leaf\"\ntag = 15\nhalf_extent = [3, 0, 2]\n";
    let body = "[body.b]\nvariance = 0\n[[body.b.tagma]]\nsegments = 1\n";
    refused(
        &format!("{PALETTE}{extended}{body}"),
        "no Plate shape \"blade\" to replace",
    );
}
