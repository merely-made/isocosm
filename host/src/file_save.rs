// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use std::{fs::File, io::{self, Write}, path::Path};

/// Publish complete save bytes by replacing the destination from its own directory.
pub(crate) fn replace(path: &Path, bytes: &[u8]) -> io::Result<()> {
    replace_with(path, bytes, |_, file, bytes| file.write_all(bytes), |_| Ok(()))
}

fn replace_with<W, R>(path: &Path, bytes: &[u8], write: W, ready: R) -> io::Result<()>
where
    W: FnOnce(&Path, &mut File, &[u8]) -> io::Result<()>,
    R: FnOnce(&Path) -> io::Result<()>,
{
    let parent = path.parent().filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let mut stage = tempfile::Builder::new().prefix(".isocosm-save-")
        .suffix(".pending").tempfile_in(parent)?;
    let stage_path = stage.path().to_path_buf();
    write(&stage_path, stage.as_file_mut(), bytes)?;
    stage.as_file_mut().flush()?;
    stage.as_file().sync_all()?;
    // into_temp_path closes the complete, synchronized file before publication.
    let pending = stage.into_temp_path();
    ready(pending.as_ref())?;
    pending.persist(path).map_err(|error| error.error)
}

// Exercise failures at the actual staging phases without replacing filesystem policy.
#[cfg(test)]
pub(crate) fn with_failures<W, R>(path: &Path, bytes: &[u8], write: W, ready: R) -> io::Result<()>
where
    W: FnOnce(&Path, &mut File, &[u8]) -> io::Result<()>,
    R: FnOnce(&Path) -> io::Result<()>,
{
    replace_with(path, bytes, write, ready)
}
