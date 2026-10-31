use super::*;

fn no_files(path: &str) -> Result<Vec<u8>, String> {
    Err(format!("no file {path}"))
}

fn item(template: &str, hand: &str) -> ItemLook {
    ItemLook {
        template: template.to_owned(),
        hand: hand.to_owned(),
        seed: 7,
        ..ItemLook::default()
    }
}

fn look(slot: &str, layer: &str, index: u8, colour: Rgb) -> TokenAppearance {
    let mut look = TokenAppearance::default();
    look.layers.insert(slot.to_owned(), layer.to_owned());
    look.palette.insert(index, colour);
    look
}

#[test]
fn every_sprite_a_save_can_hold_is_a_core_class() {
    // The names tokens were saved under before looks existed (ruling 1022: no
    // save migration), and every one resolves.
    let library = Library::core();
    for sprite in ["knight", "hero", "scavenger", "traveler", "goblin", "orc", "skeleton", "wolf"] {
        let resolved = library.resolve(sprite, None, &[]).unwrap();
        assert!(resolved.voxels.filled() > 0, "{sprite} draws something");
    }
    assert!(library.resolve("no-such-class", None, &[]).is_err());
}

#[test]
fn a_reskin_reaches_what_the_token_did_not_override() {
    let mut library = Library::core();
    let own = look("hair", "hair:long", 0, [1, 2, 3]);
    let before = library.resolve("hero", Some(&own), &[]).unwrap();

    // A campaign pack redeclares the body layer and the default shirt.
    let reskin = r##"
        [palettes.default]
        colours = ["#000000", "#00ff00"]
        [layers."body:base"]
        fills = [[0, 10, 0, 24, 0, 8, 1]]
    "##;
    library.overlay(reskin, &no_files).unwrap();
    let after = library.resolve("hero", Some(&own), &[]).unwrap();

    assert_ne!(before.key, after.key);
    // Not overridden: the shirt and the body come from the reskin.
    assert_eq!(after.palette.color(1), [0, 255, 0]);
    assert_eq!(after.voxels.filled(), 10 * 24 * 8, "the reskin's body fills the frame");
    // Overridden: the token's own skin colour and its long hair stay.
    assert_eq!(after.palette.color(0), [1, 2, 3]);
    assert_eq!(library.slot_layer("hero", Some(&own), "hair"), Some("hair:long"));
}

#[test]
fn an_equipped_item_draws_its_bound_layer_over_the_class() {
    let library = Library::core();
    let bare = library.resolve("hero", None, &[]).unwrap();
    let armed = library.resolve("hero", None, &[item("srd5e:longsword", "main")]).unwrap();
    assert!(armed.voxels.filled() > bare.voxels.filled());
    assert_ne!(bare.key, armed.key);
    // An item no pack binds a sort for changes nothing.
    let plain = library.resolve("hero", None, &[item("srd5e:torch", "main")]).unwrap();
    assert_eq!(plain.key, bare.key);
}

#[test]
fn a_missing_layer_is_an_error_not_a_hatless_token() {
    let library = Library::core();
    let own = look("hair", "hair:none-declared", 0, [0, 0, 0]);
    assert!(library.resolve("hero", Some(&own), &[]).is_err());
    // An empty choice is nothing, and resolves.
    let bald = look("hair", "", 0, [0, 0, 0]);
    assert!(library.resolve("hero", Some(&bald), &[]).is_ok());
}

#[test]
fn a_layer_can_come_from_a_toml_file_or_a_vox() {
    let mut library = Library::core();
    let pack = r#"
        [layers."hair:short"]
        file = "hair.toml"
    "#;
    let read = |path: &str| match path {
        "hair.toml" => Ok(b"fills = [[0, 1, 0, 1, 0, 1, 4]]".to_vec()),
        other => Err(format!("no file {other}")),
    };
    library.overlay(pack, &read).unwrap();
    let bare = library.resolve("hero", Some(&look("hair", "", 0, [9, 9, 9])), &[]).unwrap();
    let filed = library.resolve("hero", Some(&look("hair", "hair:short", 0, [9, 9, 9])), &[]).unwrap();
    assert_eq!(filed.voxels.filled(), bare.voxels.filled() + 1, "one voxel the head does not cover");

    let not_vox = r#"
        [layers."hair:short"]
        file = "hair.vox"
    "#;
    let garbage = |_: &str| Ok(b"not a vox".to_vec());
    assert!(library.overlay(not_vox, &garbage).is_err());
}

#[test]
fn the_core_packs_vox_layer_loads_and_draws() {
    let library = Library::core();
    let hooded = library.resolve("hero", Some(&look("hair", "hair:hood", 0, [9, 9, 9])), &[]).unwrap();
    let bare = library.resolve("hero", Some(&look("hair", "", 0, [9, 9, 9])), &[]).unwrap();
    assert!(hooded.voxels.filled() > bare.voxels.filled() + 40, "the hood's cloth is drawn");
    // Colour slot 10 in the file is palette index 9, the hood's cloth.
    assert!(hooded.voxels.iter().any(|(_, _, _, index)| index == 9));
}

#[test]
fn material_make_and_condition_each_change_an_items_look() {
    let library = Library::core();
    let base = item("pf2e:longsword", "main");
    let plain = library.resolve_item(&base).unwrap();
    for (what, varied) in [
        ("material", ItemLook { material: Some("bronze".into()), ..base.clone() }),
        ("make", ItemLook { grade: Some("fine".into()), ..base.clone() }),
        ("condition", ItemLook { condition: Some("worn".into()), ..base.clone() }),
    ] {
        let look = library.resolve_item(&varied).unwrap();
        assert_ne!(look.key, plain.key, "{what} changes the look");
        assert_eq!(look.voxels.filled() > 0, true);
    }
    // An open key no pack maps falls back to the sort's own material.
    let unknown = ItemLook { material: Some("no-such-metal".into()), ..base.clone() };
    assert_eq!(library.resolve_item(&unknown).unwrap().key, plain.key);
}

#[test]
fn a_weapon_in_the_off_hand_is_mirrored_and_armour_wears_over_the_body() {
    let library = Library::core();
    let main = library.resolve_item(&item("pf2e:longsword", "main")).unwrap();
    let off = library.resolve_item(&item("pf2e:longsword", "off")).unwrap();
    let max_x = |r: &Resolved| r.voxels.iter().map(|(x, ..)| x).max().unwrap();
    assert!(max_x(&main) > max_x(&off), "the off hand is the other side");
    let bare = library.resolve("hero", None, &[]).unwrap();
    let armoured = library.resolve("hero", None, &[item("pf2e:chain-mail", "body")]).unwrap();
    assert!(armoured.voxels.filled() > bare.voxels.filled());
}
