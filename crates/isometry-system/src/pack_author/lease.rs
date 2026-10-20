//! Cooperative save lease. A lease left on this host by a dead process, by a process
//! from an earlier boot, or under a PID now reused by another process is reclaimed;
//! a live holder, another host or an unreadable lease still refuses.
use std::{
    fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

pub(super) struct Lease {
    path: PathBuf,
    identity: Option<(u64, u64)>,
}
impl Drop for Lease {
    fn drop(&mut self) {
        // Remove only the lease this save created, never a successor's.
        let ours = match (self.identity, fs::metadata(&self.path)) {
            (Some(identity), Ok(meta)) => file_identity(&meta) == Some(identity),
            (None, Ok(_)) => true,
            (_, Err(_)) => false,
        };
        if ours {
            let _ = fs::remove_file(&self.path);
        }
    }
}
struct Holder {
    pid: u32,
    host: String,
    // Absent in leases written before boot and start identity were recorded.
    boot: Option<String>,
    start: Option<String>,
}
impl Holder {
    fn parse(text: &str) -> Option<Self> {
        let field = |name: &str| {
            text.split_whitespace()
                .find_map(|part| part.strip_prefix(name)?.strip_prefix('='))
        };
        Some(Self {
            pid: field("pid")?
                .parse()
                .ok()
                .filter(|&pid| pid > 0 && pid <= i32::MAX as u32)?,
            host: field("host")?.to_string(),
            boot: field("boot").map(str::to_string),
            start: field("start").map(str::to_string),
        })
    }
}
impl Lease {
    pub(super) fn acquire(path: PathBuf, source: &str) -> Result<Self, String> {
        let host = hostname();
        let pid = std::process::id();
        let mut body = format!("pid={pid} host={}", host.as_deref().unwrap_or("unknown"));
        for (name, value) in [("boot", boot_id()), ("start", start_time(pid))] {
            if let Some(value) = value {
                body.push_str(&format!(" {name}={value}"));
            }
        }
        body.push_str(&format!(" source={source}\n"));
        match create(&path, &body) {
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
                reclaim(&path, host.as_deref())?;
                create(&path, &body).map_err(|e| unavailable(&e.to_string()))
            },
            other => other.map_err(|e| unavailable(&e.to_string())),
        }
    }
}
fn unavailable(reason: &str) -> String {
    format!("save lease unavailable ({reason}); inspect any stale author lock before removing it")
}
fn create(path: &Path, body: &str) -> io::Result<Lease> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?;
    let lease = Lease {
        path: path.to_path_buf(),
        identity: file_identity(&file.metadata()?),
    };
    file.write_all(body.as_bytes())?;
    file.sync_all()?;
    Ok(lease)
}
fn read(path: &Path) -> io::Result<(Option<(u64, u64)>, String)> {
    let mut file = fs::File::open(path)?;
    let mut text = String::new();
    file.read_to_string(&mut text)?;
    Ok((file_identity(&file.metadata()?), text))
}
fn reclaim(path: &Path, host: Option<&str>) -> Result<(), String> {
    let (identity, text) = read(path).map_err(|e| unavailable(&e.to_string()))?;
    let holder = Holder::parse(&text).ok_or_else(|| unavailable("unrecognized lease"))?;
    if host != Some(holder.host.as_str()) {
        return Err(unavailable(&format!(
            "held by pid {} on host {}",
            holder.pid, holder.host
        )));
    }
    let ours = holder.pid == std::process::id();
    let earlier_boot = matches!((&holder.boot, boot_id()), (Some(then), Some(now)) if *then != now);
    let reused = matches!(
        (&holder.start, start_time(holder.pid)),
        (Some(then), Some(now)) if *then != now
    );
    if ours || !(earlier_boot || reused || !alive(holder.pid)) {
        return Err(unavailable(&format!(
            "held by live pid {} on this host",
            holder.pid
        )));
    }
    let name = path
        .file_name()
        .ok_or("lease has no filename")?
        .to_string_lossy();
    let tombstone = path.with_file_name(format!("{name}.stale-{}", std::process::id()));
    fs::rename(path, &tombstone).map_err(|e| unavailable(&e.to_string()))?;
    let moved = read(&tombstone).ok();
    if moved.as_ref().map(|(i, t)| (*i, t.as_str())) != Some((identity, text.as_str())) {
        // Another saver reclaimed first and this rename took its live lease: put it back.
        let _ = fs::hard_link(&tombstone, path);
        let _ = fs::remove_file(&tombstone);
        return Err(unavailable("another saver reclaimed the stale lease"));
    }
    let _ = fs::remove_file(&tombstone);
    remove_temporaries(path, &name, holder.pid);
    Ok(())
}
// The dead holder's synced temporary files (`.<manifest>.author-<pid>-<n>`) are debris.
fn remove_temporaries(path: &Path, lock_name: &str, pid: u32) {
    let Some(stem) = lock_name.strip_suffix(".author.lock") else {
        return;
    };
    let prefix = format!("{stem}.author-{pid}-");
    let Some(Ok(entries)) = path.parent().map(fs::read_dir) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name
            .strip_prefix(&prefix)
            .is_some_and(|n| !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()))
        {
            let _ = fs::remove_file(entry.path());
        }
    }
}
#[cfg(unix)]
fn file_identity(meta: &fs::Metadata) -> Option<(u64, u64)> {
    use std::os::unix::fs::MetadataExt;
    Some((meta.dev(), meta.ino()))
}
#[cfg(not(unix))]
fn file_identity(_: &fs::Metadata) -> Option<(u64, u64)> {
    None
}
#[cfg(unix)]
pub(super) fn hostname() -> Option<String> {
    let mut buffer = [0u8; 256];
    // SAFETY: the pointer and length describe a writable buffer owned by this frame.
    if unsafe { libc::gethostname(buffer.as_mut_ptr().cast(), buffer.len()) } != 0 {
        return None;
    }
    let end = buffer.iter().position(|&b| b == 0)?;
    let host = std::str::from_utf8(&buffer[..end]).ok()?;
    (!host.is_empty() && !host.contains(char::is_whitespace)).then(|| host.to_string())
}
#[cfg(not(unix))]
pub(super) fn hostname() -> Option<String> {
    None
}
// Alive means the PID exists; `reclaim` uses boot and start identity to tell a reused PID.
#[cfg(unix)]
fn alive(pid: u32) -> bool {
    // SAFETY: signal 0 only checks existence and permission; Holder::parse keeps pid in 1..=i32::MAX.
    let signalled = unsafe { libc::kill(pid as libc::pid_t, 0) } == 0;
    signalled || io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH)
}
#[cfg(not(unix))]
fn alive(_: u32) -> bool {
    true
}
// Identifies this boot, so a lease from before a reboot reads as stale whatever its PID.
#[cfg(target_os = "linux")]
pub(super) fn boot_id() -> Option<String> {
    let id = fs::read_to_string("/proc/sys/kernel/random/boot_id").ok()?;
    token(id.trim())
}
#[cfg(target_os = "macos")]
pub(super) fn boot_id() -> Option<String> {
    let mut boot = libc::timeval {
        tv_sec: 0,
        tv_usec: 0,
    };
    let mut size = std::mem::size_of::<libc::timeval>();
    let mut name = [libc::CTL_KERN, libc::KERN_BOOTTIME];
    // SAFETY: `name` names KERN_BOOTTIME, whose value is one timeval; `boot` and `size`
    // describe a writable timeval owned by this frame, and no new value is supplied.
    let status = unsafe {
        libc::sysctl(
            name.as_mut_ptr(),
            name.len() as libc::c_uint,
            (&raw mut boot).cast(),
            &mut size,
            std::ptr::null_mut(),
            0,
        )
    };
    (status == 0 && size == std::mem::size_of::<libc::timeval>())
        .then(|| format!("{}.{:06}", boot.tv_sec, boot.tv_usec))
}
#[cfg(not(any(target_os = "linux", target_os = "macos")))]
pub(super) fn boot_id() -> Option<String> {
    None
}
// When a process started; a different value under the same PID means the PID was reused.
#[cfg(target_os = "linux")]
pub(super) fn start_time(pid: u32) -> Option<String> {
    let stat = fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    // Field 22, starttime; fields after the parenthesized command name start at field 3.
    let rest = &stat[stat.rfind(')')? + 1..];
    token(rest.split_whitespace().nth(19)?)
}
#[cfg(target_os = "macos")]
pub(super) fn start_time(pid: u32) -> Option<String> {
    let mut info = std::mem::MaybeUninit::<libc::proc_bsdinfo>::zeroed();
    let size = std::mem::size_of::<libc::proc_bsdinfo>() as libc::c_int;
    // SAFETY: PROC_PIDTBSDINFO writes at most `size` bytes into `info`, a zeroed
    // proc_bsdinfo owned by this frame; the struct is plain data, so zeroed is valid.
    let written = unsafe {
        libc::proc_pidinfo(
            pid as libc::c_int,
            libc::PROC_PIDTBSDINFO,
            0,
            info.as_mut_ptr().cast(),
            size,
        )
    };
    if written != size {
        return None;
    }
    // SAFETY: proc_pidinfo filled all `size` bytes, and zeroed bytes were already valid.
    let info = unsafe { info.assume_init() };
    Some(format!(
        "{}.{:06}",
        info.pbi_start_tvsec, info.pbi_start_tvusec
    ))
}
#[cfg(not(any(target_os = "linux", target_os = "macos")))]
pub(super) fn start_time(_: u32) -> Option<String> {
    None
}
#[cfg(target_os = "linux")]
fn token(value: &str) -> Option<String> {
    (!value.is_empty() && !value.contains(char::is_whitespace)).then(|| value.to_string())
}
