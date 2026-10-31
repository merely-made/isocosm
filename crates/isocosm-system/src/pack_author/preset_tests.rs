use super::*;
use isocosm_campaign::GeneratorLockPreset;

fn preset() -> GeneratorLockPreset {
    serde_json::from_value(serde_json::json!({
        "key":"quality", "label":"Heirloom \"quality\"\n雪",
        "value":{"type":"object","fields":{"quality":{"type":"list","values":[{"type":"text","value":"heirloom"}]}}}
    })).unwrap()
}
fn propose(s: &Session, target: &str) -> Result<serde_json::Value, String> {
    super::super::preset::propose(s, s.current.revision, target, preset())
}
#[test]
fn proposal_is_read_only_and_apply_uses_existing_edit_and_save() {
    let f = Fixture::new();
    let source = include_str!("../../tests/fixtures/annotated-pack.toml");
    f.write(source);
    let mut s = f.session();
    let before = s.current.clone();
    let p = propose(&s, "forge_item").unwrap();
    assert_eq!(p["applied"], false);
    assert!(Arc::ptr_eq(&before, &s.current));
    assert_eq!(fs::read_to_string(&s.path).unwrap(), source);
    let edits = serde_json::from_value(p["edits"].clone()).unwrap();
    s.edit(p["base"].as_u64().unwrap(), edits).unwrap();
    assert_eq!(&*s.current.text, p["preview"].as_str().unwrap());
    assert_eq!(fs::read_to_string(&s.path).unwrap(), source);
    let result = decode(s.format, &s.current.text).unwrap();
    assert_eq!(result.generators[0].lock_presets[1], preset());
    assert!(
        result.generators[1..]
            .iter()
            .all(|g| g.lock_presets.is_empty())
    );
    assert!(
        s.current
            .text
            .ends_with(&source[source.find("# Foreign extension").unwrap()..])
    );
    assert!(s.current.text.contains("# display name"));
    assert!(
        s.edit(1, serde_json::from_value(p["edits"].clone()).unwrap())
            .is_err()
    );
    assert!(super::super::preset::propose(&s, 1, "forge_item", preset()).is_err());
    assert!(
        propose(&s, "forge_item")
            .unwrap_err()
            .contains("already exists")
    );
    s.save(s.current.revision).unwrap();
    assert_eq!(f.session().current.text, s.current.text);
}
#[test]
fn insertion_covers_middle_last_crlf_and_final_line_comment() {
    let source = include_str!("../../tests/fixtures/annotated-pack.toml");
    for target in ["local_map", "npc"] {
        let f = Fixture::new();
        f.write(&source.replace('\n', "\r\n"));
        let s = f.session();
        let p = propose(&s, target).unwrap();
        let preview = p["preview"].as_str().unwrap();
        assert!(!preview.replace("\r\n", "").contains('\n'));
        let manifest = decode(s.format, preview).unwrap();
        assert_eq!(
            manifest
                .generators
                .iter()
                .find(|g| g.id == target)
                .unwrap()
                .lock_presets,
            vec![preset()]
        );
    }
    let f = Fixture::new();
    let end = source.find("\n# Foreign extension").unwrap();
    f.write(&format!("{} # final comment", source[..end].trim_end()));
    let s = f.session();
    assert!(
        propose(&s, "npc").unwrap()["preview"]
            .as_str()
            .unwrap()
            .contains("fixtures = [] # final comment\n")
    );
}
#[test]
fn uncertain_invalid_or_conflicting_inputs_refuse_without_mutation() {
    let f = Fixture::new();
    let source = fs::read_to_string(f.0.join(GeneratorPack::MANIFEST_FILE)).unwrap();
    for (text, target) in [
        (source.clone(), "missing"),
        (
            source.replace("id = \"local_map\"", "id = \"forge_item\""),
            "forge_item",
        ),
        (
            source.replace("[[generators]]", "[[\"generators\"]]"),
            "forge_item",
        ),
        ("name = \"broken".into(), "forge_item"),
        (
            source.replace(
                "[[generators]]\nid = \"local_map\"",
                "[generators.lock_presets.vendor]\nkeep = 'old preset extension'\n\n[[generators]]\nid = \"local_map\"",
            ),
            "forge_item",
        ),
    ] {
        f.write(&text);
        let s = f.session();
        let before = s.current.clone();
        assert!(propose(&s, target).is_err(), "{text}");
        assert!(Arc::ptr_eq(&before, &s.current));
        assert_eq!(fs::read_to_string(&s.path).unwrap(), text);
    }
    // TOML 1.1 source is fully indexed, so it no longer blocks a proposal.
    let modern = format!("{source}\n[extension]\ntime = 12:34\nescape = \"\\e\"\n");
    f.write(&modern);
    let proposal = propose(&f.session(), "forge_item").unwrap();
    assert!(
        proposal["preview"]
            .as_str()
            .unwrap()
            .ends_with("time = 12:34\nescape = \"\\e\"\n")
    );
    f.write(&source);
    let s = f.session();
    let mut bad = preset();
    bad.key.clear();
    assert!(super::super::preset::propose(&s, 1, "forge_item", bad).is_err());
    fs::remove_file(f.0.join(GeneratorPack::MANIFEST_FILE)).unwrap();
    fs::write(
        f.0.join(GeneratorPack::MANIFEST_FILE_JSON),
        serde_json::to_vec(&decode(Format::Toml, &source).unwrap()).unwrap(),
    )
    .unwrap();
    assert!(
        propose(&f.session(), "forge_item")
            .unwrap_err()
            .contains("complete TOML")
    );
}
