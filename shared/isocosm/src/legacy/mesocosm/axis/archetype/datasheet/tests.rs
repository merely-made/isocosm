// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! What the sheets build, and what a sheet may not say.
//!
//! That the sheets build the worlds the code-built rosters did is the founding
//! receipt's to prove (`world/genesis/founding/tests.rs`); P3a's equality
//! test against the code is in history at `18945c84`.

use super::*;
use crate::legacy::mesocosm::axis::archetype;

#[test]
fn every_named_body_and_palette_resolves() {
    // Ruling 632: the names in code are lookups, so each must find its body.
    let bodies: [fn() -> Recipe; 16] = [
        archetype::producer_mat,
        archetype::producer_shrub,
        archetype::producer_stalk,
        archetype::consumer_browser,
        archetype::consumer_pursuit,
        archetype::consumer_armoured,
        archetype::decomposer_crust,
        archetype::decomposer_detritivore,
        archetype::branching::producer_shrub,
        archetype::branching::consumer_browser,
        archetype::branching::consumer_armoured,
        archetype::jointed::producer_shrub,
        archetype::jointed::consumer_browser,
        archetype::jointed::consumer_armoured,
        archetype::spaced::producer_shrub,
        archetype::spaced::consumer_browser,
    ];
    for body in bodies {
        assert!(!body().tagmata.is_empty());
    }
    assert_eq!(
        archetype::spaced::consumer_armoured(),
        archetype::jointed::consumer_armoured()
    );
    for palette in [
        archetype::palette(),
        archetype::jointed::palette(),
        archetype::spaced::palette(),
    ] {
        assert_ne!(palette, PartPalette::primitive());
    }
}

#[test]
fn the_primitive_palette_is_the_codes() {
    assert_eq!(
        foundings().palette("primitive"),
        Ok(PartPalette::primitive())
    );
}

#[test]
fn a_changed_sheet_builds_a_changed_body() {
    // The mat's pads become fronds, which every palette admits: the Roster's
    // first producer must stop matching the shipped one, and only it.
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

#[test]
fn a_founding_resolves_a_body_in_its_own_palette() {
    // The jointed palette has no blade, so the mat with blades cannot be
    // founded there, though the base palette admits it.
    let base = SETS[0]
        .1
        .replacen("shape = \"pad\"", "shape = \"blade\"", 1);
    let mut sets = SETS;
    sets[0].1 = &base;
    let why = load(&sets, FOUNDINGS.1).err().expect("refused");
    assert!(why.contains("admits no Plate shape \"blade\""), "{why}");
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
    let why = try_load(tables).err().expect("refused");
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
    assert_eq!(sheets.body("t.b", "p").as_ref(), Ok(body));
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
