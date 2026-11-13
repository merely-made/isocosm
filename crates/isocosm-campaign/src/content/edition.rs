// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Edition packs (cases 1064, 1067). An edition's records ship as a runtime
//! pack: a section of the ordinary pack manifest naming record files by kind
//! and the subsystems the pack declares, under the pack's own licence and
//! credit. The rules code reading those records stays compiled.
//!
//! Bundled packs live in the top-level `packs/` folder, found beside the
//! executable when shipped and by source path in development. A campaign's
//! founding pins each pack it plays by id, version and content hash, so a
//! peer can check what it was sent against the pin.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::ContentPackManifest;

/// The manifest file an edition pack is read from.
pub const MANIFEST_FILE: &str = "isocosm-pack.toml";

/// The `[edition]` section of a pack manifest.
///
/// Subsystems are compiled today; a pack's scripted `System` is reserved
/// (case 1064) and will arrive as its own key here, judged by the same
/// registry and conflict rule.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EditionSection {
    /// The system key a campaign founded on this pack plays, such as
    /// `pf2e-srd`. Absent for a pack that only adds records.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    /// Registry keys of the subsystems this pack's rules run as.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub subsystems: Vec<String>,
    /// Record files by kind (`items`, `classes`, `effects`), relative to the
    /// pack root, in the order the adapter reads them.
    #[serde(default)]
    pub records: BTreeMap<String, Vec<String>>,
}

/// What a founding records of a pack: enough to name it and to check that
/// the bytes a peer holds are the ones the campaign was founded on.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PackPin {
    pub id: String,
    pub version: String,
    /// SHA-256 over the manifest and every record file, lowercase hex.
    pub hash: String,
}

/// An edition pack read whole: its manifest, its record files and their hash.
#[derive(Clone, Debug)]
pub struct EditionPack {
    manifest: ContentPackManifest,
    files: BTreeMap<String, String>,
    hash: String,
}

impl EditionPack {
    /// Build a pack from its manifest text and record files, as a directory or
    /// a transferred bundle supplies them. Every declared file must be present.
    pub fn from_files(manifest_text: &str, files: BTreeMap<String, String>) -> Result<Self, String> {
        let manifest: ContentPackManifest =
            toml::from_str(manifest_text).map_err(|e| format!("edition pack manifest: {e}"))?;
        manifest.validate().map_err(|e| format!("edition pack {}: {e}", manifest.id))?;
        let edition = manifest
            .edition
            .as_ref()
            .ok_or_else(|| format!("pack {} declares no edition section", manifest.id))?;
        for file in edition.records.values().flatten() {
            if !files.contains_key(file) {
                return Err(format!("pack {} is missing record file {file}", manifest.id));
            }
        }
        if let Some(extra) = files.keys().find(|f| !edition.records.values().flatten().any(|d| d == *f)) {
            return Err(format!("pack {} carries an undeclared file {extra}", manifest.id));
        }
        let hash = content_hash(manifest_text, &files);
        Ok(Self { manifest, files, hash })
    }

    /// Read a pack directory. Declared files must stay below its root.
    pub fn load(dir: impl AsRef<Path>) -> Result<Self, String> {
        let root = dir
            .as_ref()
            .canonicalize()
            .map_err(|e| format!("open edition pack {}: {e}", dir.as_ref().display()))?;
        let manifest_text = std::fs::read_to_string(root.join(MANIFEST_FILE))
            .map_err(|e| format!("read {}: {e}", root.join(MANIFEST_FILE).display()))?;
        let manifest: ContentPackManifest = toml::from_str(&manifest_text)
            .map_err(|e| format!("parse {}: {e}", root.join(MANIFEST_FILE).display()))?;
        let mut files = BTreeMap::new();
        for file in manifest.edition.iter().flat_map(|e| e.records.values().flatten()) {
            let path = root
                .join(file)
                .canonicalize()
                .map_err(|e| format!("open {} in pack {}: {e}", file, manifest.id))?;
            if !path.starts_with(&root) {
                return Err(format!("pack {} file {file} leaves the pack root", manifest.id));
            }
            let text = std::fs::read_to_string(&path)
                .map_err(|e| format!("read {}: {e}", path.display()))?;
            files.insert(file.clone(), text);
        }
        Self::from_files(&manifest_text, files)
    }

    pub fn manifest(&self) -> &ContentPackManifest {
        &self.manifest
    }

    pub fn id(&self) -> &str {
        &self.manifest.id
    }

    pub fn edition(&self) -> &EditionSection {
        self.manifest.edition.as_ref().expect("checked when built")
    }

    /// The texts of one kind of record, in declared order; none is no error.
    pub fn records(&self, kind: &str) -> Vec<&str> {
        self.edition()
            .records
            .get(kind)
            .into_iter()
            .flatten()
            .map(|file| self.files[file].as_str())
            .collect()
    }

    /// Every file a peer needs to rebuild this pack, manifest excluded.
    pub fn files(&self) -> &BTreeMap<String, String> {
        &self.files
    }

    pub fn hash(&self) -> &str {
        &self.hash
    }

    pub fn pin(&self) -> PackPin {
        PackPin {
            id: self.manifest.id.clone(),
            version: self.manifest.version.clone(),
            hash: self.hash.clone(),
        }
    }
}

/// Length-prefixed, so no two different file sets share an encoding.
fn content_hash(manifest_text: &str, files: &BTreeMap<String, String>) -> String {
    let mut hasher = Sha256::new();
    let mut part = |bytes: &[u8]| {
        hasher.update((bytes.len() as u64).to_le_bytes());
        hasher.update(bytes);
    };
    part(MANIFEST_FILE.as_bytes());
    part(manifest_text.as_bytes());
    for (path, text) in files {
        part(path.as_bytes());
        part(text.as_bytes());
    }
    hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// The bundled `packs/` folder: beside the executable when shipped, else the
/// source tree's.
pub fn bundled_root() -> Option<PathBuf> {
    let beside = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.join("packs")))
        .filter(|dir| dir.is_dir());
    beside.or_else(|| {
        let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packs");
        source.is_dir().then_some(source)
    })
}

/// Every bundled edition pack, read and hashed once, ordered by id. Packs
/// without an edition section are left to the generator catalogue.
pub fn bundled() -> Result<&'static [EditionPack], String> {
    static PACKS: OnceLock<Result<Vec<EditionPack>, String>> = OnceLock::new();
    PACKS
        .get_or_init(|| {
            let root = bundled_root().ok_or("no bundled packs folder beside the app or in the source tree")?;
            let mut packs = Vec::new();
            let entries = std::fs::read_dir(&root).map_err(|e| format!("read {}: {e}", root.display()))?;
            for entry in entries {
                let dir = entry.map_err(|e| e.to_string())?.path();
                let manifest = dir.join(MANIFEST_FILE);
                if !manifest.is_file() {
                    continue;
                }
                let text = std::fs::read_to_string(&manifest)
                    .map_err(|e| format!("read {}: {e}", manifest.display()))?;
                let declared: ContentPackManifest = toml::from_str(&text)
                    .map_err(|e| format!("parse {}: {e}", manifest.display()))?;
                if declared.edition.is_some() {
                    packs.push(EditionPack::load(&dir)?);
                }
            }
            packs.sort_by(|a, b| a.id().cmp(b.id()));
            if let Some(pair) = packs.windows(2).find(|pair| pair[0].id() == pair[1].id()) {
                return Err(format!("two bundled packs share the id {}", pair[0].id()));
            }
            Ok(packs)
        })
        .as_deref()
        .map_err(Clone::clone)
}

pub fn bundled_pack(id: &str) -> Result<&'static EditionPack, String> {
    bundled()?
        .iter()
        .find(|pack| pack.id() == id)
        .ok_or_else(|| format!("no bundled pack {id}"))
}

/// One kind of record from a bundled pack.
pub fn bundled_records(id: &str, kind: &str) -> Result<Vec<&'static str>, String> {
    Ok(bundled_pack(id)?.records(kind))
}

#[cfg(test)]
mod tests {
    use super::*;

    const MANIFEST: &str = r#"
format = 1
id = "demo-edition"
name = "Demo edition"
version = "1"
license = "CC-BY-4.0"
credit = "Demo credit"
[edition]
system = "demo"
records = { items = ["items.json"] }
"#;

    fn files(text: &str) -> BTreeMap<String, String> {
        BTreeMap::from([("items.json".to_owned(), text.to_owned())])
    }

    #[test]
    fn a_pack_is_pinned_by_its_bytes() {
        let a = EditionPack::from_files(MANIFEST, files("[]")).unwrap();
        let b = EditionPack::from_files(MANIFEST, files("[ ]")).unwrap();
        assert_eq!(a.records("items"), ["[]"]);
        assert!(a.records("spells").is_empty());
        assert_eq!(a.pin().id, "demo-edition");
        assert_ne!(a.hash(), b.hash(), "one changed byte changes the pin");
        assert_eq!(a.hash(), EditionPack::from_files(MANIFEST, files("[]")).unwrap().hash());
    }

    #[test]
    fn a_pack_refuses_missing_extra_and_escaping_files() {
        assert!(EditionPack::from_files(MANIFEST, BTreeMap::new()).is_err());
        let mut extra = files("[]");
        extra.insert("other.json".into(), "[]".into());
        assert!(EditionPack::from_files(MANIFEST, extra).is_err());
        let escaping = MANIFEST.replace("items.json", "../items.json");
        assert!(EditionPack::from_files(&escaping, files("[]")).is_err());
        assert!(EditionPack::from_files(&MANIFEST.replace("[edition]", "[unrelated]"), files("[]")).is_err());
    }

    #[test]
    fn the_bundled_editions_declare_their_terms() {
        let packs = bundled().unwrap();
        for id in ["5e-srd", "pf2e-srd", "ars5"] {
            let pack = packs.iter().find(|p| p.id() == id).unwrap();
            assert!(pack.manifest().license.is_some() && pack.manifest().credit.is_some(), "{id}");
        }
    }
}
