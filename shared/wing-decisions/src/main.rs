// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! `wing-decisions regen [--stage]` rebuilds the manifest, the register (once
//! the fields lines carry areas and summaries) and the journal cache.
//! `wing-decisions lookup <key | path | --area NAME | --history KEY>`.
//! `wing-decisions anchors (--index FILE.scip | --scip CRATE_DIR)...`.
//! `wing-decisions check` fails when the committed manifest is stale, and
//! reports the doctrine's budget overruns without failing.

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use wing_decisions::anchors::{Found, Index};
use wing_decisions::git::Repo;
use wing_decisions::lookup::Lookup;
use wing_decisions::budget::{self, BUDGET, Budget};
use wing_decisions::model::Key;
use wing_decisions::{DOCTRINE, MANIFEST, Snapshot, journal, scan};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("wing-decisions: {e}");
            ExitCode::from(2)
        }
    }
}

fn run(args: &[String]) -> Result<ExitCode, String> {
    let repo = Repo::discover(&std::env::current_dir().map_err(|e| e.to_string())?)?;
    let rest = &args[args.len().min(1)..];
    match args.first().map(String::as_str) {
        Some("regen") => regen(&repo, rest.iter().any(|a| a == "--stage")),
        Some("check") => check(&repo),
        Some("lookup") => lookup(&repo, rest),
        Some("anchors") => anchors(&repo, rest),
        _ => Err("usage: wing-decisions regen [--stage] | check | lookup <key|path|--area NAME|--history KEY> | anchors (--index FILE | --scip DIR)...".into()),
    }
}

fn register_path(snap: &Snapshot) -> Option<String> {
    let record = snap.files.iter().find(|(p, _)| scan::is_record(p))?;
    let dir = record.0.rsplit_once('/').map_or("", |(d, _)| d);
    Some(format!("{dir}/case_register.md"))
}

fn regen(repo: &Repo, stage: bool) -> Result<ExitCode, String> {
    let snap = Snapshot::from_index(repo)?;
    let m = snap.manifest();
    let mut written = vec![MANIFEST.to_string()];
    write(&repo.root.join(MANIFEST), &m.to_toml(true))?;
    let cache = repo.root.join("target").join("wing-decisions");
    std::fs::create_dir_all(&cache).map_err(|e| e.to_string())?;
    write(&cache.join("derived.toml"), &m.to_toml(false))?;
    let reg_path = register_path(&snap);
    let reg_dir = reg_path.as_deref().and_then(|p| p.rsplit_once('/')).map_or("", |(d, _)| d);
    match (m.register(&snap.areas(), reg_dir), reg_path.clone()) {
        (Some(reg), Some(path)) => {
            write(&repo.root.join(&path), &reg)?;
            written.push(path);
        }
        _ => println!("register: left as it is until every live case carries an area and a summary (L3)"),
    }
    let live = m.items.values().filter(|i| i.entry.is_some()).count();
    println!("manifest: {} decisions ({live} live), {} files read", m.items.len(), snap.files.len());
    for e in &m.errors {
        eprintln!("  {e}");
    }
    match journal::update(repo, &snap.citer()) {
        Ok((_, u)) => println!("journal: {} commits folded in, {} records added, {} in all", u.commits, u.records, u.total),
        Err(e) => eprintln!("journal: not updated: {e}"),
    }
    if stage {
        let mut a = vec!["add", "--"];
        a.extend(written.iter().map(String::as_str));
        repo.git(&a)?;
    }
    Ok(ExitCode::SUCCESS)
}

fn check(repo: &Repo) -> Result<ExitCode, String> {
    let snap = Snapshot::from_index(repo)?;
    let fresh = snap.manifest().to_toml(true);
    let staged = repo.git(&["show", &format!(":{MANIFEST}")]).unwrap_or_default();
    let current = staged.replace("\r\n", "\n") == fresh;
    if current {
        println!("manifest is current");
    } else {
        println!("manifest is stale: run `wing-decisions regen --stage`");
    }
    let doctrine = snap.files.iter().find(|(p, _)| p == DOCTRINE).map_or("", |(_, t)| t.as_str());
    match repo.git(&["show", &format!(":{BUDGET}")]) {
        Ok(text) => {
            let over = Budget::parse(&text)?.overruns(&budget::measure(doctrine));
            if over.is_empty() {
                println!("doctrine within budget");
            } else {
                println!("over budget, for the next round touching each area:");
                over.iter().for_each(|o| println!("  {o}"));
            }
        }
        Err(_) => println!("no {BUDGET} staged; budget not checked"),
    }
    Ok(if current { ExitCode::SUCCESS } else { ExitCode::from(1) })
}

fn lookup(repo: &Repo, rest: &[String]) -> Result<ExitCode, String> {
    // The derived view, rebuilt from the index so it is never stale.
    let l = Lookup::parse(&Snapshot::from_index(repo)?.manifest().to_toml(false))?;
    let out = match rest {
        [flag, name @ ..] if flag == "--area" => l.area(&name.join(" ")),
        [flag, key @ ..] if flag == "--history" => history(repo, &key.join(" "))?,
        q if !q.is_empty() => {
            let q = q.join(" ");
            match Key::parse(&q) {
                Some(k) => l.decision(&k).unwrap_or_else(|| format!("{k}: neither declared nor cited\n")),
                None => l.path(&q),
            }
        }
        _ => return Err("lookup what?".into()),
    };
    print!("{out}");
    Ok(ExitCode::SUCCESS)
}

fn history(repo: &Repo, key: &str) -> Result<String, String> {
    let k = Key::parse(key).ok_or("not a key")?;
    let snap = Snapshot::from_index(repo)?;
    let (j, _) = journal::update(repo, &snap.citer())?;
    let id = k.id();
    let mut s = String::new();
    for r in j.entries().iter().filter(|r| r.subject == id || r.object == id) {
        let stance = if r.stance == wing_impresa::Stance::Lapse { "lapse" } else { "assoc" };
        s.push_str(&format!("{} {stance} {} {} {} (rev {})\n", &r.cause[..10], r.subject, r.kind, r.object, r.canon_revision));
    }
    Ok(if s.is_empty() { format!("no records for {k}\n") } else { s })
}

fn anchors(repo: &Repo, rest: &[String]) -> Result<ExitCode, String> {
    let mut index = Index::default();
    let mut it = rest.iter();
    while let Some(flag) = it.next() {
        let v = it.next().ok_or("flag without a value")?;
        let file = match flag.as_str() {
            "--index" => PathBuf::from(v),
            "--scip" => scip(repo, Path::new(v))?,
            _ => return Err(format!("unknown flag {flag}")),
        };
        let bytes = std::fs::read(&file).map_err(|e| format!("{}: {e}", file.display()))?;
        let one = Index::load(&bytes)?;
        for (k, v) in one.by_path {
            index.by_path.entry(k).or_default().extend(v);
        }
    }
    let m = Snapshot::from_index(repo)?.manifest();
    let mut obviated = Vec::new();
    for (k, item) in &m.items {
        let Some(e) = &item.entry else { continue };
        if e.links.governs.is_empty() {
            continue;
        }
        let mut gone = 0;
        for a in &e.links.governs {
            let found = index.resolve(a);
            match &found {
                Found::Present(at) => println!("{k}: {a} present at {}:{}", at[0].0, at[0].1),
                Found::Moved(to) => println!("{k}: {a} moved? {}", to.join(", ")),
                Found::Gone => {
                    gone += 1;
                    println!("{k}: {a} GONE");
                }
            }
        }
        if gone == e.links.governs.len() {
            obviated.push(k.to_string());
        }
    }
    if !obviated.is_empty() {
        println!("obviated (every anchor gone): {}", obviated.join(", "));
    }
    Ok(ExitCode::SUCCESS)
}

/// Runs rust-analyzer's SCIP indexer over one crate directory.
fn scip(repo: &Repo, dir: &Path) -> Result<PathBuf, String> {
    let name = dir.file_name().and_then(|n| n.to_str()).unwrap_or("crate");
    let out = repo.root.join("target").join("wing-decisions").join(format!("{name}.scip"));
    std::fs::create_dir_all(out.parent().unwrap()).map_err(|e| e.to_string())?;
    let status = std::process::Command::new("rust-analyzer")
        .arg("scip")
        .arg(repo.root.join(dir))
        .arg("--output")
        .arg(&out)
        .status()
        .map_err(|e| format!("rust-analyzer: {e}"))?;
    if !status.success() {
        return Err(format!("rust-analyzer scip failed on {}", dir.display()));
    }
    Ok(out)
}

fn write(path: &Path, text: &str) -> Result<(), String> {
    let old = std::fs::read_to_string(path).unwrap_or_default();
    if old.replace("\r\n", "\n") != text {
        std::fs::write(path, text).map_err(|e| format!("{}: {e}", path.display()))?;
    }
    Ok(())
}
