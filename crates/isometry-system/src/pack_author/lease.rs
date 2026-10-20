//! Cooperative save lease. A lease left by a dead process on this host is reclaimed;
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
        })
    }
}
impl Lease {
    pub(super) fn acquire(path: PathBuf, source: &str) -> Result<Self, String> {
        let host = hostname();
        let body = format!(
            "pid={} host={} source={source}\n",
            std::process::id(),
            host.as_deref().unwrap_or("unknown")
        );
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
    if holder.pid == std::process::id() || alive(holder.pid) {
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
// A PID that is reused by an unrelated process reads as alive and keeps refusing.
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
