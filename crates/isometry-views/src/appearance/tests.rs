use super::*;

fn no_files(path: &str) -> Result<Vec<u8>, String> {
    Err(format!("no file {path}"))
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
        let resolved = library.resolve(sprite, None, []).unwrap();
        assert!(resolved.voxels.filled() > 0, "{sprite} draws something");
    }
    assert!(library.resolve("no-such-class", None, []).is_err());
}

#[test]
fn a_reskin_reaches_what_the_token_did_not_override() {
    let mut library = Library::core();
    let own = look("hair", "hair:long", 0, [1, 2, 3]);
    let before = library.resolve("hero", Some(&own), []).unwrap();

    // A campaign pack redeclares the body layer and the default shirt.
    let reskin = r##"
        [palettes.default]
        colours = ["#000000", "#00ff00"]
        [layers."body:base"]
        fills = [[0, 10, 0, 24, 0, 8, 1]]
    "##;
    library.overlay(reskin, &no_files).unwrap();
    let after = library.resolve("hero", Some(&own), []).unwrap();

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
    let bare = library.resolve("hero", None, []).unwrap();
    let armed = library.resolve("hero", None, ["srd5e:longsword"]).unwrap();
    assert!(armed.voxels.filled() > bare.voxels.filled());
    assert_ne!(bare.key, armed.key);
    // An item no pack binds changes nothing.
    let plain = library.resolve("hero", None, ["srd5e:torch"]).unwrap();
    assert_eq!(plain.key, bare.key);
}

#[test]
fn a_missing_layer_is_an_error_not_a_hatless_token() {
    let library = Library::core();
    let own = look("hair", "hair:none-declared", 0, [0, 0, 0]);
    assert!(library.resolve("hero", Some(&own), []).is_err());
    // An empty choice is nothing, and resolves.
    let bald = look("hair", "", 0, [0, 0, 0]);
    assert!(library.resolve("hero", Some(&bald), []).is_ok());
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
    let bare = library.resolve("hero", Some(&look("hair", "", 0, [9, 9, 9])), []).unwrap();
    let filed = library.resolve("hero", Some(&look("hair", "hair:short", 0, [9, 9, 9])), []).unwrap();
    assert_eq!(filed.voxels.filled(), bare.voxels.filled() + 1, "one voxel the head does not cover");

    let not_vox = r#"
        [layers."hair:short"]
        file = "hair.vox"
    "#;
    let garbage = |_: &str| Ok(b"not a vox".to_vec());
    assert!(library.overlay(not_vox, &garbage).is_err());
}
