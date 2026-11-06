// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Bind an observation to immutable input and executable bytes, in addition to
//! the native world's own rule/genesis digests and state witness.

use super::{PHASE, Report, Session};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    io::{Read, Write},
    path::PathBuf,
};

#[derive(Clone, Debug, Serialize)]
pub struct FileIdentity {
    pub path: PathBuf,
    pub sha256: String,
    pub bytes: u64,
}

impl FileIdentity {
    pub fn read(path: PathBuf) -> Self {
        assert!(path.is_absolute(), "identity path must be absolute");
        let mut file = std::fs::File::open(&path)
            .unwrap_or_else(|why| panic!("cannot read {}: {why}", path.display()));
        let mut sha = Sha256::new();
        let mut buffer = [0; 64 * 1024];
        let mut bytes = 0;
        loop {
            let size = file.read(&mut buffer).expect("hash input bytes");
            if size == 0 {
                break;
            }
            bytes += size as u64;
            sha.update(&buffer[..size]);
        }
        Self {
            path,
            sha256: format!("{:x}", sha.finalize()),
            bytes,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Inputs {
    pub phase: &'static str,
    pub executable: FileIdentity,
    pub source_manifest: FileIdentity,
    pub source_patch: FileIdentity,
    pub reference_receipt: FileIdentity,
    pub reference_executor_sha256: String,
    pub matched_reference_fields: Vec<&'static str>,
    pub initial_native_sample_matches_reference: bool,
    pub native_ecology_processes: BTreeMap<String, cosmograph::rules::Process>,
    pub provenance_catalog: super::catalog::Catalog,
    pub protocol: &'static str,
}

#[derive(Clone, Debug, Serialize)]
struct Build {
    executable: FileIdentity,
    source_manifest: FileIdentity,
    source_patch: FileIdentity,
}

impl Build {
    fn from_environment() -> Self {
        Self {
            executable: FileIdentity::read(std::env::current_exe().expect("test executable path")),
            source_manifest: FileIdentity::read(absolute_environment_path(
                "FOOD_DIAGNOSTIC_SOURCE_MANIFEST",
            )),
            source_patch: FileIdentity::read(absolute_environment_path(
                "FOOD_DIAGNOSTIC_SOURCE_PATCH",
            )),
        }
    }
}

fn native_ecology_processes(s: &Session) -> BTreeMap<String, cosmograph::rules::Process> {
    s.sim
        .genesis()
        .rules
        .processes
        .iter()
        .filter(|(id, _)| id.starts_with("ecology:"))
        .map(|(id, p)| (id.clone(), p.clone()))
        .collect()
}

/// Capture only a real, zero-tick native initial state. The parent freezes this
/// build first; the full run must read this exact new reference, not the older
/// unchanged-rule world's genesis or an identity synthesized from its receipt.
pub fn capture_reference(s: &Session, report: &Report) {
    #[derive(Serialize)]
    struct ReferenceIdentity {
        phase: &'static str,
        status: &'static str,
        advanced_ticks: u64,
        build: Build,
        native_ecology_processes: BTreeMap<String, cosmograph::rules::Process>,
        provenance_catalog: super::catalog::Catalog,
        initial_ingesta: super::ingesta::Stores,
    }
    #[derive(Serialize)]
    struct Reference<'a> {
        #[serde(flatten)]
        initial: &'a Report,
        diagnostic_reference: ReferenceIdentity,
    }
    assert_eq!(s.sim.state().tick, report.started_tick);
    assert_eq!(report.samples.len(), 1);
    let path = absolute_environment_path("FOOD_DIAGNOSTIC_REFERENCE");
    assert!(
        !path.exists(),
        "preserve existing reference {}",
        path.display()
    );
    let parent = path.parent().expect("reference parent");
    std::fs::create_dir_all(parent).expect("reference directory");
    let name = path.file_name().unwrap().to_string_lossy();
    let pending = parent.join(format!(".{name}.pending-{}", std::process::id()));
    let mut file = std::fs::File::create(&pending).expect("pending initial reference");
    serde_json::to_writer_pretty(
        &mut file,
        &Reference {
            initial: report,
            diagnostic_reference: ReferenceIdentity {
                phase: PHASE,
                status: "native-initial-reference",
                advanced_ticks: 0,
                build: Build::from_environment(),
                native_ecology_processes: native_ecology_processes(s),
                provenance_catalog: super::catalog::read(s),
                initial_ingesta: super::ingesta::read(s),
            },
        },
    )
    .expect("serialize initial reference");
    file.write_all(b"\n").unwrap();
    file.sync_all().unwrap();
    std::fs::rename(pending, &path).expect("publish initial reference");
    eprintln!("captured {PHASE} initial reference: {}", path.display());
}

pub fn absolute_environment_path(key: &str) -> PathBuf {
    let path = PathBuf::from(
        std::env::var_os(key).unwrap_or_else(|| panic!("{key} must name an absolute path")),
    );
    assert!(path.is_absolute(), "{key} must name an absolute path");
    path
}

impl Inputs {
    pub fn from_environment(s: &Session, report: &Report) -> Self {
        let build = Build::from_environment();
        let reference_receipt =
            FileIdentity::read(absolute_environment_path("FOOD_DIAGNOSTIC_REFERENCE"));
        let reference: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&reference_receipt.path).unwrap()).unwrap();
        let current = serde_json::to_value(report).unwrap();
        let reference_identity = &reference["diagnostic_reference"];
        assert_eq!(
            reference_identity["phase"], PHASE,
            "explicit new reference phase"
        );
        assert_eq!(reference_identity["status"], "native-initial-reference");
        assert_eq!(reference_identity["advanced_ticks"], 0);
        assert_eq!(
            reference_identity["build"],
            serde_json::to_value(&build).unwrap(),
            "reference must be captured by this exact frozen source and executable"
        );
        let native_ecology_processes = native_ecology_processes(s);
        assert_eq!(
            reference_identity["native_ecology_processes"],
            serde_json::to_value(&native_ecology_processes).unwrap(),
            "native definitions match the explicit aligned reference"
        );
        let provenance_catalog = super::catalog::read(s);
        assert_eq!(
            reference_identity["provenance_catalog"],
            serde_json::to_value(&provenance_catalog).unwrap(),
            "actual account and lineage provenance matches the new reference"
        );
        assert_eq!(
            reference_identity["initial_ingesta"],
            serde_json::to_value(super::ingesta::read(s)).unwrap(),
            "actual initial digestion ledgers match the new reference"
        );
        let matched_reference_fields = vec![
            "seed",
            "founding",
            "genesis_digest",
            "rules_digest",
            "initial_state_hash",
            "native_version",
            "tick_microseconds",
            "execution",
            "started_tick",
            "month_ticks",
            "root_store_qualified",
            "root_store_admission",
            "feeding_definitions",
            "region_basis",
        ];
        for key in &matched_reference_fields {
            assert_eq!(current[*key], reference[*key], "qualified reference {key}");
        }
        assert_eq!(
            current["samples"][0], reference["samples"][0],
            "qualified reference initial native sample"
        );
        Self {
            phase: PHASE,
            reference_executor_sha256: build.executable.sha256.clone(),
            executable: build.executable,
            source_manifest: build.source_manifest,
            source_patch: build.source_patch,
            reference_receipt,
            matched_reference_fields,
            initial_native_sample_matches_reference: true,
            native_ecology_processes,
            provenance_catalog,
            protocol: "explicit aligned food rules; own frozen zero-tick initial reference; same bounded seed9 founding parameters and rate family; native Grouped; 131490 minute ticks; three exact monthly samples plus founding; persistence remains unqualified",
        }
    }
}
