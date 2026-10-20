//! Stale-lease reclaim: a killed saver's lease is reclaimed, a live one still blocks.
use super::*;
use std::{
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

const HOLDER: &str = "PACK_AUTHOR_LEASE_HOLDER";
fn lock(f: &Fixture) -> PathBuf {
    f.0.join(".isometry-pack.toml.author.lock")
}
// Re-run this test binary as a saver that stops after syncing its temporary file.
fn holder(f: &Fixture) -> Child {
    let child = Command::new(std::env::current_exe().unwrap())
        .args(["lease_holder_child", "--ignored", "--test-threads=1"])
        .env(HOLDER, &f.0)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let temporary = format!(".isometry-pack.toml.author-{}-", child.id());
    let deadline = Instant::now() + Duration::from_secs(60);
    while !(lock(f).exists()
        && fs::read_dir(&f.0)
            .unwrap()
            .flatten()
            .any(|e| e.file_name().to_string_lossy().starts_with(&temporary)))
    {
        assert!(
            Instant::now() < deadline,
            "lease holder did not reach mid-save"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
    child
}
fn dead_pid() -> u32 {
    let mut child = Command::new("true").spawn().unwrap();
    let pid = child.id();
    child.wait().unwrap();
    pid
}
#[test]
#[ignore = "child process for the lease tests"]
fn lease_holder_child() {
    let Some(dir) = std::env::var_os(HOLDER) else {
        return;
    };
    let mut s = Session::open(Path::new(&dir), 32).unwrap();
    replace(&mut s, "Demo Generator Pack", "Child edit");
    let _ = s.save_inner(2, save::Inject::HoldBeforeReplace);
}
#[test]
fn a_killed_savers_lease_is_reclaimed() {
    let f = Fixture::new();
    let before = fs::read(f.0.join(GeneratorPack::MANIFEST_FILE)).unwrap();
    let mut child = holder(&f);
    let held = fs::read_to_string(lock(&f)).unwrap();
    assert!(
        held.starts_with(&format!("pid={} host=", child.id())),
        "{held}"
    );
    assert!(held.contains(&lease::hostname().unwrap()));
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        assert!(
            held.contains(&format!(" boot={} ", lease::boot_id().unwrap())),
            "{held}"
        );
        let start = lease::start_time(child.id()).unwrap();
        assert!(held.contains(&format!(" start={start} ")), "{held}");
    }
    child.kill().unwrap();
    child.wait().unwrap();
    assert!(lock(&f).exists());
    let mut s = f.session();
    assert_eq!(fs::read(&s.path).unwrap(), before);
    replace(&mut s, "Demo Generator Pack", "After reclaim");
    s.save(2).unwrap();
    assert!(
        fs::read_to_string(&s.path)
            .unwrap()
            .contains("After reclaim")
    );
    assert_eq!(s.saved_revision, 2);
    // The lease and the dead saver's synced temporary file are both gone.
    let left = fs::read_dir(&f.0)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.starts_with('.'))
        .collect::<Vec<_>>();
    assert!(left.is_empty(), "{left:?}");
}
#[test]
fn a_live_lease_holder_still_blocks_save() {
    let f = Fixture::new();
    let before = fs::read(f.0.join(GeneratorPack::MANIFEST_FILE)).unwrap();
    let mut child = holder(&f);
    let held = fs::read(lock(&f)).unwrap();
    let mut s = f.session();
    replace(&mut s, "Demo Generator Pack", "Blocked");
    let error = s.save(2).unwrap_err();
    assert!(error.contains("live pid"), "{error}");
    assert_eq!(fs::read(lock(&f)).unwrap(), held);
    assert_eq!(fs::read(&s.path).unwrap(), before);
    assert_eq!(s.saved_revision, 1);
    child.kill().unwrap();
    child.wait().unwrap();
    s.save(2).unwrap();
    assert!(fs::read_to_string(&s.path).unwrap().contains("Blocked"));
}
#[test]
fn foreign_host_and_unrecognized_leases_still_refuse() {
    let f = Fixture::new();
    let mut s = f.session();
    replace(&mut s, "Demo Generator Pack", "Refused");
    let pid = dead_pid();
    for (lease, reason) in [
        (
            format!("pid={pid} host=elsewhere.invalid source=x\n"),
            "on host elsewhere.invalid",
        ),
        (format!("pid={pid} source=x\n"), "unrecognized lease"),
        ("pid=0 host=x source=x\n".to_string(), "unrecognized lease"),
    ] {
        fs::write(lock(&f), &lease).unwrap();
        let error = s.save(2).unwrap_err();
        assert!(error.contains(reason), "{error}");
        assert_eq!(fs::read_to_string(lock(&f)).unwrap(), lease);
        assert_eq!(s.saved_revision, 1);
    }
    let host = lease::hostname().unwrap();
    fs::write(lock(&f), format!("pid={pid} host={host} source=x\n")).unwrap();
    s.save(2).unwrap();
    assert!(!lock(&f).exists());
}
// A live stand-in process plays a holder whose PID was reused, or that predates a reboot.
#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn reused_pids_and_earlier_boots_are_reclaimed_but_a_matching_holder_blocks() {
    let f = Fixture::new();
    let mut s = f.session();
    replace(&mut s, "Demo Generator Pack", "After reuse");
    let mut live = Command::new("sleep").arg("60").spawn().unwrap();
    let pid = live.id();
    let host = lease::hostname().unwrap();
    let boot = lease::boot_id().unwrap();
    let start = lease::start_time(pid).unwrap();
    let matching = format!("pid={pid} host={host} boot={boot} start={start} source=x\n");
    fs::write(lock(&f), &matching).unwrap();
    let error = s.save(2).unwrap_err();
    assert!(error.contains("live pid"), "{error}");
    assert_eq!(fs::read_to_string(lock(&f)).unwrap(), matching);
    assert_eq!(s.saved_revision, 1);
    let mut name = "After reuse".to_string();
    for (case, stale) in [
        format!("pid={pid} host={host} boot={boot} start=0.000001 source=x\n"),
        format!("pid={pid} host={host} boot=0.000001 start={start} source=x\n"),
    ]
    .into_iter()
    .enumerate()
    {
        let next = format!("Reclaimed {case}");
        replace(&mut s, &name, &next);
        name = next;
        fs::write(lock(&f), &stale).unwrap();
        let revision = s.current.revision;
        s.save(revision).unwrap();
        assert!(!lock(&f).exists(), "{stale}");
        assert_eq!(s.saved_revision, revision);
        assert!(fs::read_to_string(&s.path).unwrap().contains(&name));
    }
    // The stand-in itself is untouched: reclaim removes files, never signals processes.
    assert!(live.try_wait().unwrap().is_none());
    live.kill().unwrap();
    live.wait().unwrap();
}
