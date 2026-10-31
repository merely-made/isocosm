//! Real terminal authoring followed by the existing pack loader/runtime.
#![cfg(feature = "pack-authoring")]
#![deny(clippy::all)]
use isocosm_system::{GeneratorLimits, GeneratorPack};
use serde_json::{Value, json};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};
fn copy_tree(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for e in fs::read_dir(from).unwrap() {
        let e = e.unwrap();
        let dest = to.join(e.file_name());
        if e.file_type().unwrap().is_dir() {
            copy_tree(&e.path(), &dest);
        } else {
            fs::copy(e.path(), dest).unwrap();
        }
    }
}
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        static N: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        let path = std::env::temp_dir().join(format!(
            "pack-author-cli-{}-{}",
            std::process::id(),
            N.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        copy_tree(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/packs/demo"),
            &path,
        );
        Self(path)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn run(root: &Path, commands: Vec<Value>) -> Vec<Value> {
    let mut p = Command::new(env!("CARGO_BIN_EXE_pack-author"))
        .arg(root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    {
        let mut stdin = p.stdin.take().unwrap();
        for c in commands {
            writeln!(stdin, "{c}").unwrap();
        }
    }
    let out = p.wait_with_output().unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}
#[test]
fn author_save_reopen_and_run_the_real_pack_fixture() {
    let f = Fixture::new();
    let manifest = f.0.join(GeneratorPack::MANIFEST_FILE);
    let original = fs::read_to_string(&manifest).unwrap();
    let first = run(
        &f.0,
        vec![
            json!({"op":"edit","base":1,"edits":[{"start":0,"end":original.len(),"text":"name = \"incomplete"}]}),
            json!({"op":"save","base":2}),
            json!({"op":"quit"}),
        ],
    );
    assert_eq!(first[1]["result"]["state"]["valid"], false);
    assert_eq!(first[2]["result"]["admitted"], false);
    let annotated = include_str!("fixtures/annotated-pack.toml");
    let second = run(
        &f.0,
        vec![
            json!({"op":"check"}),
            json!({"op":"edit","base":1,"edits":[{"start":0,"end":"name = \"incomplete".len(),"text":annotated}]}),
            json!({"op":"inspect"}),
            json!({"op":"save","base":2}),
            json!({"op":"undo","base":2,"target":1}),
            json!({"op":"check"}),
            json!({"op":"undo","base":3,"target":2}),
            json!({"op":"save","base":4}),
            json!({"op":"quit"}),
        ],
    );
    assert_eq!(second[0]["ready"]["valid"], false);
    assert_eq!(second[1]["ok"], false);
    assert_eq!(second[3]["result"]["indexed"], true);
    assert_eq!(second[6]["ok"], false);
    assert_eq!(second[8]["result"]["saved_revision"], 4);
    assert_eq!(fs::read_to_string(&manifest).unwrap(), annotated);
    let third = Command::new(env!("CARGO_BIN_EXE_pack-author"))
        .arg(&f.0)
        .arg("--check")
        .output()
        .unwrap();
    assert!(third.status.success());
    let check: Value = serde_json::from_slice(&third.stdout).unwrap();
    assert_eq!(check["valid"], true);
    assert_eq!(check["admitted"], false);
    assert_eq!(check["choices"].as_array().unwrap().len(), 4);
    let pack = GeneratorPack::load(&f.0).unwrap();
    pack.run_fixture(
        "demo:forge_item",
        "fixtures/forge_item.json",
        GeneratorLimits::default(),
    )
    .unwrap();
}
#[test]
fn stale_commands_refuse_without_changing_disk_or_live_catalogue() {
    let f = Fixture::new();
    let path = f.0.join(GeneratorPack::MANIFEST_FILE);
    let old = fs::read(&path).unwrap();
    let loaded = GeneratorPack::load(&f.0).unwrap();
    let responses = run(
        &f.0,
        vec![
            json!({"op":"edit","base":1,"edits":[{"start":0,"end":0,"text":"# unsaved draft\n"}]}),
            json!({"op":"edit","base":1,"edits":[{"start":0,"end":0,"text":"wrong"}]}),
            json!({"op":"save","base":1}),
            json!({"op":"read","text":true}),
            json!({"op":"quit"}),
        ],
    );
    assert_eq!(responses[2]["ok"], false);
    assert_eq!(responses[3]["ok"], false);
    assert_eq!(responses[4]["result"]["dirty"], true);
    assert_eq!(fs::read(path).unwrap(), old);
    assert_eq!(loaded.manifest().name, "Demo Generator Pack");
}
#[test]
fn structural_commands_edit_by_id_without_caller_offsets() {
    let f = Fixture::new();
    let manifest = f.0.join(GeneratorPack::MANIFEST_FILE);
    fs::write(&manifest, include_str!("fixtures/annotated-pack.toml")).unwrap();
    let preset = json!({"key":"quality","label":"Heirloom quality","value":{"type":"text","value":"heirloom"}});
    let r = run(
        &f.0,
        vec![
            json!({"op":"set_generator_field","base":1,"generator":"forge_item","field":"name","value":"Forge a river heirloom"}),
            json!({"op":"insert_lock_preset","base":2,"generator":"forge_item","preset":preset}),
            json!({"op":"insert_lock_preset","base":2,"generator":"forge_item","preset":preset}),
            json!({"op":"remove_lock_preset","base":3,"generator":"forge_item","key":"culture"}),
            json!({"op":"replace_lock_preset","base":4,"generator":"forge_item","key":"quality",
                "preset":{"key":"quality","label":"Heirloom","value":{"type":"text","value":"old"}}}),
            json!({"op":"remove_lock_preset","base":5,"generator":"npc","key":"culture"}),
            json!({"op":"check"}),
            json!({"op":"save","base":5}),
            json!({"op":"quit"}),
        ],
    );
    for (i, operation) in [
        (1, "set_generator_field"),
        (2, "insert_lock_preset"),
        (4, "remove_lock_preset"),
        (5, "replace_lock_preset"),
    ] {
        let receipt = &r[i]["result"]["receipt"];
        assert_eq!(receipt["operation"], operation, "{}", r[i]);
        assert_eq!(receipt["result"], receipt["base"].as_u64().unwrap() + 1);
        assert!(!r[i]["result"]["edits"].as_array().unwrap().is_empty());
        assert_eq!(r[i]["result"]["state"]["admitted"], false);
    }
    assert!(r[3]["error"].as_str().unwrap().contains("stale"));
    assert!(r[6]["error"].as_str().unwrap().contains("no preset"));
    let forge = &r[7]["result"]["manifest"]["generators"][0];
    assert_eq!(forge["name"], "Forge a river heirloom");
    assert_eq!(forge["lock_presets"][0]["label"], "Heirloom");
    assert_eq!(forge["lock_presets"].as_array().unwrap().len(), 1);
    let saved = fs::read_to_string(&manifest).unwrap();
    assert!(saved.contains("# display name") && saved.contains("literal string"));
    GeneratorPack::load(&f.0)
        .unwrap()
        .run_fixture(
            "demo:forge_item",
            "fixtures/forge_item.json",
            GeneratorLimits::default(),
        )
        .unwrap();
}
