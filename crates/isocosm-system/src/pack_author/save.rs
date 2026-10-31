//! Sequential CLI ownership serializes local edits/saves; leases cover cooperating processes.
use super::lease::Lease;
use super::*;
use std::{fs, io::Write, path::Path};
struct Files {
    _lease: Lease,
    temporary: Option<PathBuf>,
}
impl Drop for Files {
    fn drop(&mut self) {
        if let Some(p) = &self.temporary {
            let _ = fs::remove_file(p);
        }
    }
}
/// Test-only interruptions between syncing the temporary file and replacing the target.
#[cfg_attr(not(test), allow(dead_code))]
pub(super) enum Inject {
    Nothing,
    FailBeforeReplace,
    HoldBeforeReplace,
}
impl Session {
    pub(super) fn save(&mut self, base: u64) -> Result<serde_json::Value, String> {
        self.save_inner(base, Inject::Nothing)
    }
    pub(super) fn save_inner(
        &mut self,
        base: u64,
        inject: Inject,
    ) -> Result<serde_json::Value, String> {
        self.check_base(base)?;
        check_manifest_set(&self.path)?;
        let parent = self.path.parent().ok_or("manifest has no parent")?;
        let name = self
            .path
            .file_name()
            .ok_or("manifest has no filename")?
            .to_string_lossy();
        let lock = parent.join(format!(".{name}.author.lock"));
        let mut files = Files {
            _lease: Lease::acquire(lock, &self.disk_fingerprint)?,
            temporary: None,
        };
        let metadata = fs::symlink_metadata(&self.path).map_err(|e| e.to_string())?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err("save requires a regular non-symlink manifest".into());
        }
        check_disk(&self.path, &self.disk_fingerprint)?;
        let temporary = parent.join(format!(
            ".{name}.author-{}-{}",
            std::process::id(),
            INSTANCES.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(|e| e.to_string())?;
        files.temporary = Some(temporary.clone());
        file.write_all(self.current.text.as_bytes())
            .map_err(|e| e.to_string())?;
        file.set_permissions(metadata.permissions())
            .map_err(|e| e.to_string())?;
        file.sync_all().map_err(|e| e.to_string())?;
        drop(file);
        match inject {
            Inject::Nothing => {},
            Inject::FailBeforeReplace => return Err("injected failure before replacement".into()),
            Inject::HoldBeforeReplace => loop {
                std::thread::park();
            },
        }
        check_disk(&self.path, &self.disk_fingerprint)?;
        check_manifest_set(&self.path)?;
        fs::rename(&temporary, &self.path).map_err(|e| format!("replace manifest: {e}"))?;
        files.temporary = None;
        self.disk_fingerprint = self.current.fingerprint.clone();
        self.saved_revision = base;
        Ok(
            serde_json::json!({"saved_revision":base,"source_fingerprint":self.disk_fingerprint,"valid":self.current.validation.is_none(),"admitted":false}),
        )
    }
}
fn check_manifest_set(path: &Path) -> Result<(), String> {
    let parent = path.parent().ok_or("manifest has no parent")?;
    if parent.join(GeneratorPack::MANIFEST_FILE).is_file()
        && parent.join(GeneratorPack::MANIFEST_FILE_JSON).is_file()
    {
        return Err("both TOML and JSON manifests are present; reopen explicitly".into());
    }
    Ok(())
}
fn check_disk(path: &Path, expected: &str) -> Result<(), String> {
    let bytes = fs::read(path).map_err(|e| e.to_string())?;
    if fingerprint(&bytes) != expected {
        return Err("external manifest change; reopen explicitly before saving".into());
    }
    Ok(())
}
#[cfg(test)]
pub(super) fn fail_save(session: &mut Session) -> Result<serde_json::Value, String> {
    session.save_inner(session.current.revision, Inject::FailBeforeReplace)
}
