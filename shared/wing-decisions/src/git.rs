// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The few git calls the tool needs, with one `cat-file --batch` process for
//! reading many blobs.

use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

pub struct Repo {
    pub root: PathBuf,
}

impl Repo {
    pub fn discover(start: &Path) -> Result<Repo, String> {
        let out = run(start, &["rev-parse", "--show-toplevel"])?;
        Ok(Repo { root: PathBuf::from(out.trim()) })
    }

    pub fn git(&self, args: &[&str]) -> Result<String, String> {
        run(&self.root, args)
    }

    /// Staged blobs: (path, blob id), from the index.
    pub fn index(&self) -> Result<Vec<(String, String)>, String> {
        let out = self.git(&["ls-files", "-s", "-z"])?;
        Ok(out
            .split('\0')
            .filter_map(|rec| {
                let (meta, path) = rec.split_once('\t')?;
                let blob = meta.split(' ').nth(1)?;
                Some((path.to_string(), blob.to_string()))
            })
            .collect())
    }

    /// First-parent commits from the root to HEAD, oldest first, each with the
    /// files it changed against its first parent: one `git log` call.
    pub fn first_parent(&self) -> Result<Vec<(String, Vec<String>)>, String> {
        let out = self.git(&[
            "log",
            "--first-parent",
            "--diff-merges=first-parent",
            "--reverse",
            "--name-only",
            "--format=@@%H",
            "HEAD",
        ])?;
        let mut commits: Vec<(String, Vec<String>)> = Vec::new();
        for line in out.lines() {
            if let Some(h) = line.strip_prefix("@@") {
                commits.push((h.to_string(), Vec::new()));
            } else if let (Some(last), false) = (commits.last_mut(), line.is_empty()) {
                last.1.push(unquote(line));
            }
        }
        Ok(commits)
    }

    pub fn blobs(&self) -> Result<Blobs, String> {
        let mut child = Command::new("git")
            .args(["cat-file", "--batch"])
            .current_dir(&self.root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|e| format!("git cat-file: {e}"))?;
        let stdin = child.stdin.take().ok_or("no stdin")?;
        let stdout = BufReader::new(child.stdout.take().ok_or("no stdout")?);
        Ok(Blobs { child, stdin, stdout })
    }
}

/// git quotes paths with unusual bytes; the repository has none it cares
/// about, so a quoted path is only stripped of its quotes.
fn unquote(p: &str) -> String {
    p.strip_prefix('"').and_then(|p| p.strip_suffix('"')).unwrap_or(p).to_string()
}

fn run(dir: &Path, args: &[&str]) -> Result<String, String> {
    let out = Command::new("git").args(args).current_dir(dir).output().map_err(|e| format!("git: {e}"))?;
    if !out.status.success() {
        return Err(format!("git {}: {}", args.join(" "), String::from_utf8_lossy(&out.stderr).trim()));
    }
    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// A running `git cat-file --batch`.
pub struct Blobs {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl Blobs {
    /// The object named `spec` (a blob id or `rev:path`) as text, or None
    /// when it does not exist.
    pub fn text(&mut self, spec: &str) -> Result<Option<String>, String> {
        writeln!(self.stdin, "{spec}").map_err(|e| e.to_string())?;
        self.stdin.flush().map_err(|e| e.to_string())?;
        let mut header = String::new();
        self.stdout.read_line(&mut header).map_err(|e| e.to_string())?;
        if header.trim_end().ends_with("missing") {
            return Ok(None);
        }
        let size: usize = header
            .split_whitespace()
            .nth(2)
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| format!("cat-file header {header:?}"))?;
        let mut buf = vec![0; size + 1];
        self.stdout.read_exact(&mut buf).map_err(|e| e.to_string())?;
        buf.truncate(size);
        Ok(Some(String::from_utf8_lossy(&buf).into_owned()))
    }
}

impl Drop for Blobs {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
