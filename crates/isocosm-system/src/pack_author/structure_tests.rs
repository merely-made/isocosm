//! Structure-aware commands: tree-computed edits, typed checks, shared receipts.
use super::*;
use isocosm_campaign::GeneratorLockPreset;
use structure::{insert, publish, remove, replace as replace_preset, set_field};

const ANNOTATED: &str = include_str!("../../tests/fixtures/annotated-pack.toml");
fn preset(key: &str, label: &str) -> GeneratorLockPreset {
    serde_json::from_value(serde_json::json!({"key":key,"label":label,
        "value":{"type":"list","values":[{"type":"text","value":"heirloom"}]}}))
    .unwrap()
}
fn annotated(f: &Fixture, text: &str) -> Session {
    f.write(text);
    f.session()
}
fn publish_plan(
    s: &mut Session,
    base: u64,
    plan: impl FnOnce(&Session) -> Result<(Vec<Edit>, isocosm_campaign::ContentPackManifest), String>,
    operation: &'static str,
) -> Result<serde_json::Value, String> {
    let plan = plan(s)?;
    publish(s, base, plan, operation)
}
fn manifest(s: &Session) -> isocosm_campaign::ContentPackManifest {
    decode(s.format, &s.current.text).unwrap()
}

#[test]
fn insert_replace_and_remove_round_trip_through_shared_receipts() {
    let f = Fixture::new();
    let mut s = annotated(&f, ANNOTATED);
    let quality = preset("quality", "Heirloom \"quality\"\n雪");
    let proposal = preset::propose(&s, 1, "forge_item", quality.clone()).unwrap();
    let r = publish_plan(
        &mut s,
        1,
        |s| insert(s, "forge_item", &quality),
        "insert_lock_preset",
    )
    .unwrap();
    assert_eq!(r["receipt"]["operation"], "insert_lock_preset");
    assert_eq!(
        (
            r["receipt"]["base"].as_u64(),
            r["receipt"]["result"].as_u64()
        ),
        (Some(1), Some(2))
    );
    assert_eq!(r["edits"], proposal["edits"]);
    assert_eq!(&*s.current.text, proposal["preview"].as_str().unwrap());
    assert_eq!(
        fs::read_to_string(&s.path).unwrap(),
        ANNOTATED,
        "no implicit save"
    );
    // A comment inside the block survives an in-place replacement.
    replace(&mut s, "key = \"quality\"", "key = \"quality\" # kept");
    let heirloom = preset("heirloom", "Heirloom");
    let plan = replace_preset(&s, "forge_item", "quality", &heirloom).unwrap();
    publish(&mut s, 3, plan, "replace_lock_preset").unwrap();
    assert!(
        s.current.text.contains("key = \"heirloom\" # kept"),
        "{}",
        s.current.text
    );
    assert_eq!(manifest(&s).generators[0].lock_presets[1], heirloom);
    replace(&mut s, " # kept", "");
    let plan = remove(&s, "forge_item", "heirloom").unwrap();
    publish(&mut s, 5, plan, "remove_lock_preset").unwrap();
    assert_eq!(
        &*s.current.text, ANNOTATED,
        "insert then remove restores the bytes"
    );
    let plan = remove(&s, "forge_item", "culture").unwrap();
    publish(&mut s, 6, plan, "remove_lock_preset").unwrap();
    assert!(
        manifest(&s)
            .generators
            .iter()
            .all(|g| g.lock_presets.is_empty())
    );
    assert!(
        s.current
            .text
            .contains("fixtures = [\"fixtures/forge_item.json\"]\n\n[[generators]]")
    );
    // Undo still restores through ordinary text publication.
    s.undo(7, 1).unwrap();
    assert_eq!(&*s.current.text, ANNOTATED);
}
#[test]
fn setting_fields_replaces_values_or_adds_entries() {
    let f = Fixture::new();
    let mut s = annotated(&f, &ANNOTATED.replace('\n', "\r\n"));
    replace(
        &mut s,
        "name = \"Forge item\"",
        "name = \"Forge item\"  # shown in pickers",
    );
    let value = serde_json::json!("Forge a river heirloom");
    publish_plan(
        &mut s,
        2,
        |s| set_field(s, "forge_item", "name", &value),
        "set",
    )
    .unwrap();
    assert!(
        s.current
            .text
            .contains("name = \"Forge a river heirloom\"  # shown in pickers\r\ndefault_args")
    );
    let value = serde_json::json!(["fixtures/npc.json"]);
    publish_plan(
        &mut s,
        3,
        |s| set_field(s, "npc", "fixtures", &value),
        "set",
    )
    .unwrap();
    // Removing then setting default_args adds the entry after the generator's own entries.
    replace(
        &mut s,
        "default_args = { type = \"text\", value = \"river\" }\r\nscript = \"generators/forge_item.lua\"",
        "script = \"generators/forge_item.lua\"",
    );
    let value = serde_json::json!({"type":"text","value":"delta"});
    let revision = s.current.revision;
    publish_plan(
        &mut s,
        revision,
        |s| set_field(s, "forge_item", "default_args", &value),
        "set",
    )
    .unwrap();
    assert!(s.current.text.contains("fixtures = [\"fixtures/forge_item.json\"]\r\ndefault_args = { \"type\" = \"text\", \"value\" = \"delta\" }\r\n\r\n[[generators.lock_presets]]"));
    assert!(!s.current.text.replace("\r\n", "").contains('\n'));
    let m = manifest(&s);
    assert_eq!(m.generators[0].name, "Forge a river heirloom");
    assert_eq!(m.generators[3].fixtures, ["fixtures/npc.json"]);
    assert_eq!(
        serde_json::to_value(&m.generators[0].default_args).unwrap(),
        value
    );
    assert_eq!(m.generators[0].lock_presets.len(), 1);
    assert!(s.current.text.contains("# display name") && s.current.text.contains("literal string"));
}
#[test]
fn unprovable_or_invalid_structural_edits_refuse_without_publication() {
    let f = Fixture::new();
    let s = annotated(&f, ANNOTATED);
    let quality = preset("quality", "Quality");
    let refusals: Vec<Result<(Vec<Edit>, _), String>> = vec![
        insert(&s, "missing", &quality),
        insert(&s, "forge_item", &preset("culture", "Duplicate")),
        remove(&s, "forge_item", "missing"),
        remove(&s, "npc", "culture"),
        replace_preset(&s, "forge_item", "culture", &preset("", "Empty key")),
        set_field(&s, "forge_item", "lock_presets", &serde_json::json!([])),
        set_field(&s, "forge_item", "bogus", &serde_json::json!(1)),
        set_field(
            &s,
            "forge_item",
            "script",
            &serde_json::json!("../escape.lua"),
        ),
        set_field(&s, "forge_item", "id", &serde_json::json!("npc")),
        set_field(&s, "forge_item", "name", &serde_json::Value::Null),
        set_field(
            &s,
            "forge_item",
            "fixtures",
            &serde_json::json!("not a list"),
        ),
    ];
    for (i, refusal) in refusals.into_iter().enumerate() {
        assert!(refusal.is_err(), "case {i}");
    }
    let mut s = s;
    let before = s.current.clone();
    let plan = set_field(&s, "forge_item", "name", &serde_json::json!("x")).unwrap();
    replace(&mut s, "Local map", "Moved on");
    assert!(
        publish(&mut s, 1, plan, "set")
            .unwrap_err()
            .contains("stale")
    );
    assert!(!Arc::ptr_eq(&before, &s.current) && s.current.revision == 2);
    // An inline preset array cannot be edited by block, but the generator's own
    // fields still can; quoted headers and generator sub-tables refuse both.
    let inline = annotated(
        &f,
        &ANNOTATED.replace("[[generators.lock_presets]]\nkey = \"culture\"\nlabel = \"River-clan culture\"\nvalue = { type = \"text\", value = \"river-clans\" }\n", "")
            .replace("fixtures = [\"fixtures/forge_item.json\"]", "fixtures = [\"fixtures/forge_item.json\"]\nlock_presets = [{ key = \"culture\", label = \"C\", value = { type = \"text\", value = \"c\" } }]"),
    );
    assert!(
        remove(&inline, "forge_item", "culture")
            .unwrap_err()
            .contains("preset layout")
    );
    assert!(set_field(&inline, "forge_item", "name", &serde_json::json!("x")).is_ok());
    for layout in [
        ANNOTATED.replace("[[generators]]", "[[\"generators\"]]"),
        ANNOTATED.replace(
            "[[generators]]\nid = \"local_map\"",
            "[generators.lock_presets.vendor]\nkeep = 1\n\n[[generators]]\nid = \"local_map\"",
        ),
    ] {
        let s = annotated(&f, &layout);
        assert!(remove(&s, "forge_item", "culture").is_err());
        assert!(set_field(&s, "forge_item", "name", &serde_json::json!("x")).is_err());
    }
    fs::remove_file(f.0.join(GeneratorPack::MANIFEST_FILE)).unwrap();
    fs::write(
        f.0.join(GeneratorPack::MANIFEST_FILE_JSON),
        serde_json::to_vec(&decode(Format::Toml, ANNOTATED).unwrap()).unwrap(),
    )
    .unwrap();
    let json = f.session();
    assert!(
        set_field(&json, "forge_item", "name", &serde_json::json!("x"))
            .unwrap_err()
            .contains("complete TOML")
    );
}

#[test]
fn an_old_named_manifest_is_saved_under_the_current_name() {
    let f = Fixture::new();
    let [old_toml, _] = GeneratorPack::OLD_MANIFEST_FILES;
    fs::rename(f.0.join(GeneratorPack::MANIFEST_FILE), f.0.join(old_toml)).unwrap();
    let mut s = f.session();
    replace(&mut s, "Demo Generator Pack", "Moved pack");
    s.save(2).unwrap();
    assert!(!f.0.join(old_toml).exists());
    let text = fs::read_to_string(f.0.join(GeneratorPack::MANIFEST_FILE)).unwrap();
    assert!(text.contains("Moved pack"));
    fs::copy(f.0.join(GeneratorPack::MANIFEST_FILE), f.0.join(old_toml)).unwrap();
    assert!(Session::open(&f.0, 32).is_err(), "an old and a current manifest together are refused");
}
