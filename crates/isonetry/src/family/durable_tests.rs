use super::*;

use cosmograph::{Execution, Founding, Session};
use std::{
    fs,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

#[cfg(feature = "family-net")]
use p2panda_core::SigningKey;

struct TestDirectory(PathBuf);

impl TestDirectory {
    fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "isonetry-family-ledger-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir(&path).expect("create unique family ledger test directory");
        Self(path)
    }

    fn ledger(&self) -> PathBuf {
        self.0.join("host.ledger")
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let Ok(entries) = fs::read_dir(&self.0) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let _ = fs::remove_dir(path);
            } else {
                let _ = fs::remove_file(path);
            }
        }
        let _ = fs::remove_dir(&self.0);
    }
}

fn initial_world() -> World {
    let genesis = Founding {
        seed: 816,
        sites: 2,
        population: 4,
        lineages: 2,
        ecology: false,
        ..Default::default()
    }
    .generate()
    .expect("small durable-test genesis");
    let session = Session::new(genesis, Execution::Individuals).expect("test session");
    World::with_sections(
        session,
        BTreeMap::from([("opaque".to_string(), vec![0, 255, 7])]),
    )
}

fn file_limits() -> FamilyLedgerLimits {
    FamilyLedgerLimits {
        max_bytes: 4 * 1_048_576,
    }
}

fn append_join(host: &mut FamilyHost) -> AcceptedRecord {
    let (_, _, record) = host
        .accept(|_, _| {
            Ok(HostAdmission {
                value: (),
                commands: vec![Command::Join],
                final_tick: None,
                section_updates: BTreeMap::from([("opaque".into(), Some(vec![9, 0, 255]))]),
            })
        })
        .expect("persisted host accepts native and section update");
    record.expect("changed host world emits a record")
}

#[test]
fn file_host_restarts_from_verified_ledger_and_returns_suffix_for_republish() {
    let directory = TestDirectory::new();
    let path = directory.ledger();
    let mut host = FamilyHost::create_file(
        &path,
        SessionId([44; 16]),
        initial_world(),
        FamilyLimits::default(),
        file_limits(),
    )
    .expect("create file-backed host");
    let record = append_join(&mut host);
    let expected = host.world().save().expect("accepted world save");
    assert_eq!(
        host.persisted_suffix_after(0).unwrap(),
        vec![record.clone()]
    );
    assert!(
        host.drain_records().len() == 1,
        "transport outbox is independent"
    );
    drop(host);

    let resumed = FamilyHost::resume_file(
        &path,
        Execution::Individuals,
        FamilyLimits::default(),
        file_limits(),
    )
    .expect("resume from persisted checkpoint and accepted suffix");
    assert_eq!(resumed.world().save().unwrap(), expected);
    assert_eq!(resumed.sequence(), 1);
    assert_eq!(resumed.persisted_suffix_after(0).unwrap(), vec![record]);
    assert!(resumed.persisted_suffix_after(1).unwrap().is_empty());
    assert!(resumed.persisted_suffix_after(2).is_err());
    drop(resumed);
}

#[test]
fn durable_host_refuses_a_second_writer_for_the_same_path() {
    let directory = TestDirectory::new();
    let path = directory.ledger();
    let host = FamilyHost::create_file(
        &path,
        SessionId([45; 16]),
        initial_world(),
        FamilyLimits::default(),
        file_limits(),
    )
    .expect("create first host");
    let error = FamilyHost::resume_file(
        &path,
        Execution::Individuals,
        FamilyLimits::default(),
        file_limits(),
    )
    .err()
    .expect("second writer is refused");
    assert!(error.contains("writer"), "got {error}");
    drop(host);
}

#[test]
fn atomic_replacement_failure_leaves_world_revision_and_sequence_unchanged() {
    let directory = TestDirectory::new();
    let path = directory.ledger();
    let host = FamilyHost::create_file(
        &path,
        SessionId([46; 16]),
        initial_world(),
        FamilyLimits::default(),
        file_limits(),
    )
    .expect("create file-backed host");
    let before = host.world().save().unwrap();
    let before_revision = host.revision().unwrap();

    // Force persist() to fail at final replacement after its same-directory
    // stage is written and synced. The host gate must refuse before World or
    // journal state changes.
    fs::remove_file(&path).unwrap();
    fs::create_dir(&path).unwrap();
    let error = host
        .world()
        .put_section("opaque", vec![8, 8, 8])
        .unwrap_err();
    assert!(error.contains("replacement failed"), "got {error}");
    assert_eq!(host.world().save().unwrap(), before);
    assert_eq!(host.revision().unwrap(), before_revision);
    assert_eq!(host.sequence(), 0);
    assert!(host.persisted_suffix_after(0).unwrap().is_empty());
    assert!(
        path.is_dir(),
        "failed replacement leaves its conflicting target intact"
    );
    drop(host);
}

#[test]
fn ledger_byte_cap_refuses_before_replacing_prior_checkpoint() {
    let directory = TestDirectory::new();
    let path = directory.ledger();
    let initial_limit = 128 * 1024;
    let host = FamilyHost::create_file(
        &path,
        SessionId([47; 16]),
        initial_world(),
        FamilyLimits::default(),
        FamilyLedgerLimits {
            max_bytes: initial_limit,
        },
    )
    .expect("initial checkpoint fits configured cap");
    let before = host.world().save().unwrap();
    let bytes = fs::read(&path).unwrap();
    assert!(bytes.len() <= initial_limit);
    let error = host
        .world()
        .put_section("opaque", vec![42; initial_limit])
        .unwrap_err();
    assert!(error.contains("byte limit"), "got {error}");
    assert_eq!(fs::read(&path).unwrap(), bytes);
    assert_eq!(host.world().save().unwrap(), before);
    assert_eq!(host.sequence(), 0);
    assert!(host.persisted_suffix_after(0).unwrap().is_empty());
    drop(host);
}

#[cfg(feature = "family-net")]
#[test]
fn recovered_records_resign_deterministically_and_continue_the_host_log() {
    use crate::family_network::{sign_record, verify_host_record};

    let directory = TestDirectory::new();
    let path = directory.ledger();
    let session = SessionId([48; 16]);
    let key = SigningKey::from_bytes(&[0x48; 32]);
    let author = key.verifying_key();
    let mut host = FamilyHost::create_file(
        &path,
        session,
        initial_world(),
        FamilyLimits::default(),
        file_limits(),
    )
    .expect("create durable signed host");
    let first_record = append_join(&mut host);
    let first_operation = sign_record(&first_record, &key, 0, None, WireLimits::default())
        .expect("sign initial accepted record");
    let original_header = first_operation.header.encode();
    let original_hash = first_operation.hash.clone();
    drop(host);

    let resumed = FamilyHost::resume_file(
        &path,
        Execution::Individuals,
        FamilyLimits::default(),
        file_limits(),
    )
    .expect("resume durable signed host");
    let recovered = resumed.persisted_suffix_after(0).unwrap();
    assert_eq!(recovered, vec![first_record]);
    let restored_operation = sign_record(&recovered[0], &key, 0, None, WireLimits::default())
        .expect("re-sign persisted record at original log position");
    assert_eq!(restored_operation.hash, original_hash);
    assert_eq!(restored_operation.header.encode(), original_header);
    assert_eq!(
        verify_host_record(&restored_operation, &author, session, WireLimits::default()).unwrap(),
        recovered[0]
    );

    resumed
        .world()
        .put_section("opaque", vec![10, 20, 30])
        .expect("append a new post-restart section change");
    let suffix = resumed.persisted_suffix_after(1).unwrap();
    assert_eq!(suffix.len(), 1);
    let second_record = &suffix[0];
    assert_eq!(second_record.sequence, 2);
    assert_eq!(second_record.parent_digest, recovered[0].digest);
    let second_operation = sign_record(
        second_record,
        &key,
        1,
        Some(*restored_operation.hash.as_bytes()),
        WireLimits::default(),
    )
    .expect("continue host log from recovered operation");
    assert_eq!(second_operation.header.seq_num, 1);
    assert_eq!(
        second_operation.header.backlink,
        Some(restored_operation.hash.clone())
    );
    assert_eq!(
        verify_host_record(&second_operation, &author, session, WireLimits::default()).unwrap(),
        *second_record
    );
}
