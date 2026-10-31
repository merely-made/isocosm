use super::*;
use std::{fs, path::Path};
#[path = "history_tests.rs"]
mod history_tests;
#[cfg(unix)]
#[path = "lease_tests.rs"]
mod lease_tests;
#[path = "preset_tests.rs"]
mod preset_tests;
#[path = "shared_tests.rs"]
mod shared_tests;
#[path = "structure_tests.rs"]
mod structure_tests;
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "isocosm-pack-author-{}-{}",
            std::process::id(),
            INSTANCES.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        let source =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/packs/demo/isocosm-pack.toml");
        fs::copy(source, path.join(GeneratorPack::MANIFEST_FILE)).unwrap();
        Self(path)
    }
    fn session(&self) -> Session {
        Session::open(&self.0, 32).unwrap()
    }
    fn write(&self, source: &str) {
        fs::write(self.0.join(GeneratorPack::MANIFEST_FILE), source).unwrap();
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn replace(session: &mut Session, old: &str, text: &str) {
    let start = session.current.text.find(old).unwrap();
    session
        .edit(
            session.current.revision,
            vec![Edit {
                start,
                end: start + old.len(),
                text: text.into(),
            }],
        )
        .unwrap();
}
#[test]
fn revisions_comments_handles_drafts_and_undo() {
    let f = Fixture::new();
    let source = include_str!("../../tests/fixtures/annotated-pack.toml").to_string();
    f.write(&source);
    let mut s = f.session();
    let original = s.current.clone();
    let root = original.syntax.as_ref().unwrap().clone().into_syntax();
    let handle = s.handle(&root);
    assert!(s.resolve(&handle).is_ok());
    replace(&mut s, "Demo Generator Pack", "Edited pack");
    assert!(s.resolve(&handle).is_err());
    let other = f.session();
    let wrong = other.handle(&other.current.syntax.as_ref().unwrap().clone().into_syntax());
    assert!(s.resolve(&wrong).is_err());
    assert!(
        s.edit(
            1,
            vec![Edit {
                start: 0,
                end: 0,
                text: "stale".into()
            }]
        )
        .is_err()
    );
    assert_eq!(&*original.text, source);
    assert_eq!(
        original
            .syntax
            .as_ref()
            .unwrap()
            .clone()
            .into_syntax()
            .to_string(),
        source
    );
    let indexed = syntax::inspect(&s);
    assert_eq!(indexed["indexed"], true);
    assert!(
        indexed["occurrences"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v["path"].as_str().unwrap().contains("lock_presets"))
    );
    replace(&mut s, "name = \"Edited pack\"", "name = \"half");
    assert!(s.current.validation.is_some());
    assert!(decode(s.format, &s.current.text).is_err());
    s.save(s.current.revision).unwrap();
    assert!(
        fs::read_to_string(&s.path)
            .unwrap()
            .contains("name = \"half")
    );
    let draft = f.session();
    assert!(draft.current.validation.is_some());
    let revision = s.current.revision;
    s.undo(revision, 1).unwrap();
    assert!(s.current.revision > revision);
    assert_eq!(&*s.current.text, source);
    assert!(s.resolve(&handle).is_err());
    s.save(s.current.revision).unwrap();
    assert_eq!(fs::read_to_string(&s.path).unwrap(), source);
}
#[test]
fn full_schema_and_toml11_are_indexed_without_probe_restrictions() {
    let f = Fixture::new();
    let mut manifest = GeneratorPack::load(&f.0).unwrap().manifest().clone();
    manifest.generators[0].default_args = Some(isocosm_campaign::GenValue::Object {
        fields: BTreeMap::from([(
            "nested".into(),
            isocosm_campaign::GenValue::List {
                values: vec![isocosm_campaign::GenValue::Text {
                    value: "full ABI".into(),
                }],
            },
        )]),
    });
    let mut source = toml::to_string(&manifest).unwrap();
    source.push_str("\n[vendor.modern]\nescape = \"\\e\\x41\"\nvalue = 1.5\ndate = 1979-05-27T07:32:00Z\ntime = 07:32\ninline = {\n a = 1, # TOML 1.1\n b = 2,\n}\n");
    f.write(&source);
    let mut s = f.session();
    assert_eq!(decode(s.format, &s.current.text).unwrap(), manifest);
    assert!(s.current.index_note.is_none());
    assert!(s.current.validation.is_none());
    assert_eq!(GeneratorPack::load(&f.0).unwrap().manifest(), &manifest);
    assert!(s.current.diagnostics().is_empty());
    let paths = syntax::inspect(&s)["occurrences"]
        .as_array()
        .unwrap()
        .iter()
        .map(|o| o["path"].as_str().unwrap().to_string())
        .collect::<Vec<_>>();
    for path in [
        "vendor.modern.escape",
        "vendor.modern.time",
        "vendor.modern.inline.b",
    ] {
        assert!(paths.iter().any(|p| p == path), "{path}: {paths:?}");
    }
    s.edit(
        1,
        vec![Edit {
            start: 0,
            end: 0,
            text: "# retained modern source\n".into(),
        }],
    )
    .unwrap();
    s.save(2).unwrap();
    assert_eq!(
        fs::read_to_string(&s.path).unwrap(),
        format!("# retained modern source\n{source}")
    );
    assert_eq!(GeneratorPack::load(&f.0).unwrap().manifest(), &manifest);
}

#[test]
fn json_compatibility_and_one_manifest_rule() {
    let f = Fixture::new();
    let manifest = GeneratorPack::load(&f.0).unwrap().manifest().clone();
    fs::remove_file(f.0.join(GeneratorPack::MANIFEST_FILE)).unwrap();
    let path = f.0.join(GeneratorPack::MANIFEST_FILE_JSON);
    fs::write(&path, serde_json::to_string_pretty(&manifest).unwrap()).unwrap();
    let mut s = f.session();
    assert!(s.current.validation.is_none());
    replace(&mut s, "Demo Generator Pack", "External JSON edited");
    s.save(2).unwrap();
    assert!(
        GeneratorPack::load(&f.0)
            .unwrap()
            .manifest()
            .name
            .contains("edited")
    );
    assert!(!f.0.join(GeneratorPack::MANIFEST_FILE).exists());
    assert_eq!(syntax::inspect(&s)["indexed"], false);
    f.write("format = 1\n");
    assert!(Session::open(&f.0, 8).is_err());
    assert!(GeneratorPack::load(&f.0).is_err());
}
#[test]
fn ranges_batches_limits_and_eviction_are_transactional() {
    let f = Fixture::new();
    let mut s = f.session();
    replace(&mut s, "Demo Generator Pack", "Démø 🦀 pack");
    let old = s.current.clone();
    let inside = s.current.text.find('🦀').unwrap() + 1;
    assert!(
        s.edit(
            2,
            vec![Edit {
                start: inside,
                end: inside,
                text: "x".into()
            }]
        )
        .is_err()
    );
    assert!(Arc::ptr_eq(&old, &s.current));
    assert!(
        s.edit(
            2,
            vec![
                Edit {
                    start: 0,
                    end: 3,
                    text: "x".into()
                },
                Edit {
                    start: 1,
                    end: 4,
                    text: "y".into()
                }
            ]
        )
        .is_err()
    );
    let a = s.current.text.find("Démø 🦀 pack").unwrap();
    let b = s.current.text.find("Wandering NPC").unwrap();
    let r = s
        .edit(
            2,
            vec![
                Edit {
                    start: b,
                    end: b + 13,
                    text: "Last".into(),
                },
                Edit {
                    start: a,
                    end: a + "Démø 🦀 pack".len(),
                    text: "First".into(),
                },
            ],
        )
        .unwrap();
    assert_eq!(
        &s.current.text[r.ranges[1].result_start..r.ranges[1].result_end],
        "Last"
    );
    assert!(s.current.validation.is_none());
    let current = s.current.clone();
    assert!(
        s.edit(
            3,
            vec![Edit {
                start: 0,
                end: 0,
                text: " ".repeat(2 * 1024 * 1024)
            }]
        )
        .is_err()
    );
    assert!(Arc::ptr_eq(&current, &s.current));
    s.history.policy.limit = 2;
    for i in 0..5 {
        s.edit(
            s.current.revision,
            vec![Edit {
                start: 0,
                end: 0,
                text: format!("# {i}\n"),
            }],
        )
        .unwrap();
    }
    assert_eq!(s.history.len(), 2);
    assert!(s.snapshot(1).is_err());
    assert!(s.undo(s.current.revision, 1).is_err());
    assert!(decode(s.format, &s.current.text).is_ok());
}
#[test]
fn nested_presets_real_validation_and_no_implicit_runtime_change() {
    let f = Fixture::new();
    let loaded = GeneratorPack::load(&f.0).unwrap();
    let mut s = f.session();
    let anchor = "[[generators]]\nid = \"local_map\"";
    let insert = "[[generators.lock_presets]]\nkey=\"mood\"\nlabel=\"Quiet\"\nvalue={type=\"text\",value=\"quiet\"}\n\n";
    replace(&mut s, anchor, &format!("{insert}{anchor}"));
    let manifest = decode(s.format, &s.current.text).unwrap();
    assert_eq!(manifest.generators[0].lock_presets.len(), 2);
    assert!(manifest.generators[1].lock_presets.is_empty());
    assert_eq!(loaded.manifest().generators[0].lock_presets.len(), 1);
    replace(&mut s, insert, "");
    assert_eq!(
        decode(s.format, &s.current.text).unwrap().generators[0]
            .lock_presets
            .len(),
        1
    );
    replace(&mut s, "generators/npc.lua", "../escape.lua");
    assert!(s.current.validation.as_ref().unwrap().contains("path"));
    assert_eq!(loaded.manifest().generators[3].script, "generators/npc.lua");
}
#[test]
fn save_conflicts_failures_permissions_and_source_bytes() {
    let f = Fixture::new();
    let mut s = f.session();
    replace(&mut s, "Demo Generator Pack", "Changed");
    let disk = fs::read(&s.path).unwrap();
    assert!(save::fail_save(&mut s).is_err());
    assert_eq!(fs::read(&s.path).unwrap(), disk);
    assert_eq!(s.saved_revision, 1);
    let lock = f.0.join(".isocosm-pack.toml.author.lock");
    fs::write(&lock, "another writer").unwrap();
    assert!(s.save(2).is_err());
    fs::remove_file(lock).unwrap();
    fs::write(&s.path, "# external\n").unwrap();
    assert!(s.save(2).is_err());
    assert_eq!(fs::read_to_string(&s.path).unwrap(), "# external\n");
    assert_eq!(s.saved_revision, 1);
    f.write(std::str::from_utf8(&disk).unwrap());
    let mut s = f.session();
    replace(&mut s, "Demo Generator Pack", "Unicode 🦀");
    let crlf = s.current.text.replace('\n', "\r\n");
    let len = s.current.text.len();
    s.edit(
        2,
        vec![Edit {
            start: 0,
            end: len,
            text: crlf.clone(),
        }],
    )
    .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&s.path, fs::Permissions::from_mode(0o640)).unwrap();
    }
    s.save(3).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(&s.path).unwrap().permissions().mode() & 0o777,
            0o640
        );
    }
    assert_eq!(fs::read(&s.path).unwrap(), crlf.as_bytes());
    assert_eq!(f.session().current.fingerprint, s.current.fingerprint);
    assert!(s.save(2).is_err());
}

#[test]
fn a_new_second_manifest_refuses_save_without_changing_the_first() {
    let f = Fixture::new();
    let mut s = f.session();
    let before = fs::read(&s.path).unwrap();
    replace(&mut s, "Demo Generator Pack", "Changed");
    fs::write(f.0.join(GeneratorPack::MANIFEST_FILE_JSON), "{}").unwrap();
    assert!(s.save(2).is_err());
    assert_eq!(fs::read(&s.path).unwrap(), before);
}
#[test]
#[cfg(unix)]
fn symlink_input_can_be_checked_but_is_not_replaced_by_save() {
    use std::os::unix::fs::symlink;
    let f = Fixture::new();
    let path = f.0.join(GeneratorPack::MANIFEST_FILE);
    let target = f.0.join("actual.toml");
    fs::rename(&path, &target).unwrap();
    symlink("actual.toml", &path).unwrap();
    let before = fs::read(&target).unwrap();
    let mut s = f.session();
    assert!(s.current.validation.is_none());
    replace(&mut s, "Demo Generator Pack", "Changed");
    assert!(s.save(2).is_err());
    assert!(fs::symlink_metadata(path).unwrap().file_type().is_symlink());
    assert_eq!(fs::read(target).unwrap(), before);
}
