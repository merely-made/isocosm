//! Atlas budget is a local machine preference, outside campaign/network data.

use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{self, Read, Write},
    path::PathBuf,
};

#[derive(Serialize, Deserialize)]
struct FileValue {
    version: u32,
    atlas_budget_mib: u32,
}

pub(crate) struct Preferences {
    path: Option<PathBuf>,
    attempted: u32,
}

fn config_path() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    let root = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)?
        .join("Merely")
        .join("Isometry");
    #[cfg(target_os = "macos")]
    let root = std::env::var_os("HOME")
        .map(PathBuf::from)?
        .join("Library/Application Support/Isometry");
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let root = xdg_root(
        std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from),
        std::env::var_os("HOME").map(PathBuf::from),
    )?
    .join("isometry");
    root.is_absolute().then(|| root.join("terrain.json"))
}

// Pure resolver also tested on Windows; invalid XDG values use HOME.
#[cfg(any(test, not(any(target_os = "windows", target_os = "macos"))))]
fn xdg_root(xdg: Option<PathBuf>, home: Option<PathBuf>) -> Option<PathBuf> {
    xdg.filter(|path| path.is_absolute()).or_else(|| {
        home.filter(|path| path.is_absolute())
            .map(|path| path.join(".config"))
    })
}

impl Preferences {
    pub(crate) fn local(ui: &mut isometry_views::UiState) -> Self {
        Self::open(config_path(), ui)
    }

    fn open(path: Option<PathBuf>, ui: &mut isometry_views::UiState) -> Self {
        let mut store = Self {
            path,
            attempted: ui.terrain_settings.budget_mib,
        };
        match store.load() {
            Ok(Some(value)) => ui.terrain_settings.budget_mib = value,
            Ok(None) => {},
            Err(error) => {
                eprintln!("[isometry] terrain preference unavailable; using default: {error}")
            },
        }
        store.attempted = ui.terrain_settings.budget_mib;
        store
    }

    fn load(&self) -> io::Result<Option<u32>> {
        let Some(path) = &self.path else {
            return Ok(None);
        };
        let file = match fs::File::open(path) {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error),
        };
        let mut bytes = Vec::new();
        file.take(4097).read_to_end(&mut bytes)?;
        if bytes.len() > 4096 {
            return Err(io::Error::other("terrain preference exceeds 4 KiB"));
        }
        let value: FileValue = serde_json::from_slice(&bytes).map_err(io::Error::other)?;
        if value.version != 1 || value.atlas_budget_mib == 0 {
            return Err(io::Error::other(
                "unsupported or invalid terrain preference",
            ));
        }
        Ok(Some(value.atlas_budget_mib))
    }

    /// Once per changed request, including a failed attempt. No per-frame I/O
    /// retry loop; changing the setting again retries and reports any failure.
    pub(crate) fn save_changed(&mut self, ui: &isometry_views::UiState) -> io::Result<()> {
        let value = ui.terrain_settings.budget_mib;
        if self.attempted == value {
            return Ok(());
        }
        self.attempted = value;
        let path = self
            .path
            .as_ref()
            .ok_or_else(|| io::Error::other("no local config directory"))?;
        let parent = path
            .parent()
            .ok_or_else(|| io::Error::other("no preference parent directory"))?;
        fs::create_dir_all(parent)?;
        // Each writer owns its temporary file. Rename replaces the old value
        // only after the complete new JSON has reached the file.
        static SERIAL: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let serial = SERIAL.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let temporary = path.with_extension(format!("json.{}.{}.tmp", std::process::id(), serial));
        let mut file = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temporary)?;
        let result = (|| {
            let bytes = serde_json::to_vec_pretty(&FileValue {
                version: 1,
                atlas_budget_mib: value,
            })
            .map_err(io::Error::other)?;
            file.write_all(&bytes)?;
            file.sync_all()?;
            drop(file);
            fs::rename(&temporary, path)
        })();
        if result.is_err() {
            let _ = fs::remove_file(&temporary);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use isometry_views::{UiState, demo_map};

    #[test]
    fn xdg_empty_relative_and_unset_fall_back_to_home() {
        let absolute = std::env::temp_dir();
        for xdg in [None, Some(PathBuf::new()), Some(PathBuf::from("relative"))] {
            assert_eq!(
                xdg_root(xdg, Some(absolute.clone())),
                Some(absolute.join(".config"))
            );
        }
        assert_eq!(xdg_root(Some(absolute.clone()), None), Some(absolute));
        assert_eq!(xdg_root(None, None), None);
        assert_eq!(xdg_root(None, Some(PathBuf::from("relative"))), None);
    }

    struct Scratch(PathBuf);
    impl Scratch {
        fn new() -> Self {
            static SERIAL: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
            let id = SERIAL.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let path = std::env::temp_dir()
                .join(format!("isometry-preferences-{}-{id}", std::process::id()));
            fs::create_dir(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for Scratch {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }

    #[test]
    fn terrain_preference_roundtrip_preserves_request_not_device_cap() {
        let scratch = Scratch::new();
        let path = scratch.0.join("terrain.json");
        let mut ui = UiState::new(demo_map());
        let mut store = Preferences::open(Some(path.clone()), &mut ui);
        assert_eq!(ui.terrain_settings.budget_mib, 8);
        assert!(!path.exists());
        ui.terrain_settings.budget_mib = 64;
        ui.terrain_settings.report(32 * 1024 * 1024, 12);
        store.save_changed(&ui).unwrap();
        let mut restarted = UiState::new(demo_map());
        let mut restored = Preferences::open(Some(path.clone()), &mut restarted);
        assert_eq!(restarted.terrain_settings.budget_mib, 64);
        assert_eq!(restarted.terrain_settings.device_budget, None);
        assert_eq!(restarted.terrain_settings.omitted, 0);
        restarted.terrain_settings.budget_mib = 4;
        restored.save_changed(&restarted).unwrap();
        assert_eq!(restored.load().unwrap(), Some(4), "replace existing file");
    }

    #[test]
    fn terrain_preference_bad_files_and_write_failure_leave_session_usable() {
        let scratch = Scratch::new();
        let path = scratch.0.join("terrain.json");
        for bytes in [
            "not json",
            "{\"version\":2,\"atlas_budget_mib\":4}",
            "{\"version\":1,\"atlas_budget_mib\":0}",
        ] {
            fs::write(&path, bytes).unwrap();
            let mut ui = UiState::new(demo_map());
            let mut store = Preferences::open(Some(path.clone()), &mut ui);
            assert_eq!(ui.terrain_settings.budget_mib, 8);
            store.save_changed(&ui).unwrap();
            assert_eq!(
                fs::read_to_string(&path).unwrap(),
                bytes,
                "no boot overwrite"
            );
        }
        let mut ui = UiState::new(demo_map());
        let mut store = Preferences::open(Some(path.join("impossible.json")), &mut ui);
        ui.terrain_settings.budget_mib = 4;
        assert!(store.save_changed(&ui).is_err());
        assert_eq!(
            ui.terrain_settings.budget_mib, 4,
            "session setting survives failed persistence"
        );
        assert!(
            store.save_changed(&ui).is_ok(),
            "same failed value does not retry every frame"
        );
    }
}
