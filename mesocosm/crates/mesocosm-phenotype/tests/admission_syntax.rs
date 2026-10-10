// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Admission reads TOML as authored and JSON as still accepted (wing design
//! record, ruling 624), and the syntax never reaches the ruleset digest.

use std::path::{Path, PathBuf};

use isocosm::process::Registry;

use mesocosm_phenotype::*;

/// A scratch pack root, unique to one test and cleaned up by it.
struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("processes")).expect("a scratch directory");
        Self(root)
    }

    fn path(&self) -> &Path {
        &self.0
    }

    fn write(&self, relative: &str, body: &str) {
        std::fs::write(self.0.join(relative), body).expect("a scratch file");
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

const GLAND_TOML: &str = r#"namespace = "mesocosm"
name = "secrete"
expressed_by = ["plate"]
seeding = "acquired"
"#;

const GLAND_JSON: &str = r#"{
  "namespace": "mesocosm",
  "name": "secrete",
  "expressed_by": ["plate"],
  "seeding": "acquired"
}"#;

fn manifest_toml(file: &str) -> String {
    format!(
        "pack = \"scratch\"\nversion = \"0.0.1\"\nabi = 1\nlicense = \"MPL-2.0\"\nprocesses = [\"{file}\"]\n"
    )
}

fn manifest_json(file: &str) -> String {
    format!(
        "{{\"pack\": \"scratch\", \"version\": \"0.0.1\", \"abi\": 1, \"license\": \"MPL-2.0\", \"processes\": [\"{file}\"]}}"
    )
}

fn shipped_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("the repository root")
        .join("packs")
        .join("mesocosm")
}

#[test]
fn the_shipped_pack_is_toml_and_lowers_to_the_native_ruleset() {
    assert!(shipped_root().join(MANIFEST).is_file());
    assert!(!shipped_root().join(MANIFEST_JSON).exists());
    let admitted = admit_dir(&shipped_root()).expect("the shipped pack admits");
    assert_eq!(admitted.digest(), Registry::native().digest());
}

#[test]
fn one_pack_in_either_syntax_has_one_digest() {
    let toml = Scratch::new("syntax_toml");
    toml.write("processes/secrete.toml", GLAND_TOML);
    toml.write(MANIFEST, &manifest_toml("processes/secrete.toml"));
    let json = Scratch::new("syntax_json");
    json.write("processes/secrete.json", GLAND_JSON);
    json.write(MANIFEST_JSON, &manifest_json("processes/secrete.json"));
    assert_eq!(
        admit_dir(toml.path())
            .expect("the TOML pack admits")
            .digest(),
        admit_dir(json.path())
            .expect("the JSON pack admits")
            .digest()
    );
}

#[test]
fn a_pack_may_mix_syntaxes_file_by_file() {
    let mixed = Scratch::new("syntax_mixed");
    mixed.write("processes/secrete.json", GLAND_JSON);
    mixed.write(MANIFEST, &manifest_toml("processes/secrete.json"));
    assert!(admit_dir(mixed.path()).is_ok());
}

#[test]
fn a_second_manifest_beside_the_first_is_refused() {
    let both = Scratch::new("syntax_two_manifests");
    both.write("processes/secrete.toml", GLAND_TOML);
    both.write(MANIFEST, &manifest_toml("processes/secrete.toml"));
    both.write(MANIFEST_JSON, &manifest_json("processes/secrete.toml"));
    assert_eq!(
        admit_dir(both.path()).unwrap_err(),
        Admission::UndeclaredFile {
            path: MANIFEST_JSON.to_string()
        }
    );
}

#[test]
fn an_undeclared_toml_file_is_refused() {
    let stray = Scratch::new("syntax_undeclared_toml");
    stray.write("processes/secrete.toml", GLAND_TOML);
    stray.write("processes/stowaway.toml", GLAND_TOML);
    stray.write(MANIFEST, &manifest_toml("processes/secrete.toml"));
    let refusal = admit_dir(stray.path()).unwrap_err();
    assert!(
        matches!(&refusal, Admission::UndeclaredFile { path } if path.ends_with("stowaway.toml")),
        "{refusal:?}"
    );
}

#[test]
fn malformed_toml_is_a_malformed_schema() {
    let broken = Scratch::new("syntax_malformed_toml");
    broken.write(
        "processes/secrete.toml",
        "namespace = \"mesocosm\"\nname = ",
    );
    broken.write(MANIFEST, &manifest_toml("processes/secrete.toml"));
    assert!(matches!(
        admit_dir(broken.path()).unwrap_err(),
        Admission::MalformedSchema { .. }
    ));
}
