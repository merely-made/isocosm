// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Keys, typed links and the fields line (DOC_POLICY, "Decisions and doctrine").

use std::collections::BTreeSet;
use std::fmt;

/// A decision's key: a wing case by number, or a plan's decision by its plan
/// keyword and D-number.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Key {
    Case(u32),
    Plan(String, u32),
    /// A doctrine statement: its area's slug and number, `naming.1`.
    Statement(String, u32),
    /// A key in another repository, by that repository's name: `mere:stack.1`.
    Foreign(String, Box<Key>),
}

impl Key {
    /// Parses `975`, `sim-execution D5`, `naming.1` or `mere:stack.1`.
    pub fn parse(s: &str) -> Option<Key> {
        let s = s.trim();
        if let Some((repo, key)) = s.split_once(':') {
            let inner = Key::parse(key).filter(|k| !matches!(k, Key::Foreign(..)))?;
            return valid_keyword(repo).then(|| Key::Foreign(repo.to_string(), Box::new(inner)));
        }
        if let Ok(n) = s.parse::<u32>() {
            return Some(Key::Case(n));
        }
        if let Some((area, n)) = s.rsplit_once('.') {
            let n = n.parse().ok()?;
            return valid_keyword(area).then(|| Key::Statement(area.to_string(), n));
        }
        let (plan, d) = s.rsplit_once(' ')?;
        let n = d.strip_prefix('D')?.parse().ok()?;
        valid_keyword(plan).then(|| Key::Plan(plan.to_string(), n))
    }

    /// The impresa identity, `case:975` or `decision:sim-execution.D5`.
    pub fn id(&self) -> String {
        match self {
            Key::Case(n) => format!("case:{n}"),
            Key::Plan(p, n) => format!("decision:{p}.D{n}"),
            Key::Statement(a, n) => format!("statement:{a}.{n}"),
            Key::Foreign(r, k) => format!("{r}:{}", k.id()),
        }
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Key::Case(n) => write!(f, "{n}"),
            Key::Plan(p, n) => write!(f, "{p} D{n}"),
            Key::Statement(a, n) => write!(f, "{a}.{n}"),
            Key::Foreign(r, k) => write!(f, "{r}:{k}"),
        }
    }
}

pub fn valid_keyword(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

/// A plan's keyword from its file name: `2026-10-19_sim_execution_plan.md`
/// gives `sim-execution`.
pub fn plan_keyword(path: &str) -> Option<String> {
    let name = path.rsplit('/').next()?;
    let stem = name.strip_suffix("_plan.md")?;
    let rest = stem.get(11..).filter(|_| stem.as_bytes().get(10) == Some(&b'_'))?;
    let k = rest.replace('_', "-");
    valid_keyword(&k).then_some(k)
}

/// The typed links one entry declares in its fields line.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Links {
    pub area: Option<String>,
    pub draws_on: BTreeSet<Key>,
    pub amends: BTreeSet<Key>,
    pub replaces: BTreeSet<Key>,
    pub contradicts: BTreeSet<Key>,
    pub plans: BTreeSet<String>,
    pub governs: BTreeSet<String>,
    pub stack: bool,
    pub summary: Option<String>,
}

pub const PREFIX: &str = "Links:";
const SEP: &str = " · ";

/// Parses one fields line:
/// `Links: area Execution · draws on 975, 338 · plans sim-execution ·
/// governs isocosm::stage::Stage · stack · summary <text>`.
/// `summary` is last and takes the rest of the line.
pub fn parse_links(line: &str) -> Result<Links, String> {
    let body = line.trim().strip_prefix(PREFIX).ok_or("not a fields line")?.trim();
    let mut links = Links::default();
    let mut parts = body.split(SEP);
    while let Some(part) = parts.next() {
        let part = part.trim();
        if let Some(rest) = part.strip_prefix("summary ") {
            let tail: Vec<&str> = parts.by_ref().collect();
            let mut s = rest.to_string();
            for t in tail {
                s.push_str(SEP);
                s.push_str(t);
            }
            links.summary = Some(s.trim().to_string());
            break;
        }
        if part == "stack" {
            links.stack = true;
        } else if let Some(v) = part.strip_prefix("area ") {
            links.area = Some(v.trim().to_string());
        } else if let Some(v) = part.strip_prefix("draws on ") {
            links.draws_on = keys(v)?;
        } else if let Some(v) = part.strip_prefix("amends ") {
            links.amends = keys(v)?;
        } else if let Some(v) = part.strip_prefix("replaces ") {
            links.replaces = keys(v)?;
        } else if let Some(v) = part.strip_prefix("contradicts ") {
            links.contradicts = keys(v)?;
        } else if let Some(v) = part.strip_prefix("plans ") {
            links.plans = list(v).map(str::to_string).collect();
            if let Some(bad) = links.plans.iter().find(|p| !valid_keyword(p)) {
                return Err(format!("bad plan keyword {bad:?}"));
            }
        } else if let Some(v) = part.strip_prefix("governs ") {
            links.governs = list(v).map(str::to_string).collect();
        } else if !part.is_empty() {
            return Err(format!("unknown link {part:?}"));
        }
    }
    Ok(links)
}

fn list(v: &str) -> impl Iterator<Item = &str> {
    v.split(',').map(str::trim).filter(|s| !s.is_empty())
}

fn keys(v: &str) -> Result<BTreeSet<Key>, String> {
    list(v).map(|k| Key::parse(k).ok_or_else(|| format!("bad key {k:?}"))).collect()
}

/// An anchor (`isocosm::stage::Simulation::stage`) as an impresa identity.
pub fn anchor_id(anchor: &str) -> String {
    format!("anchor:{}", anchor.replace("::", "/"))
}

pub fn slug(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if !out.ends_with('-') && !out.is_empty() {
            out.push('-');
        }
    }
    out.trim_end_matches('-').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_parse_both_forms() {
        assert_eq!(Key::parse("975"), Some(Key::Case(975)));
        assert_eq!(Key::parse("sim-execution D5"), Some(Key::Plan("sim-execution".into(), 5)));
        assert_eq!(Key::parse("Sim D5"), None);
        let across = Key::Foreign("mere".into(), Box::new(Key::Statement("stack".into(), 1)));
        assert_eq!(Key::parse("mere:stack.1"), Some(across.clone()));
        assert_eq!(across.to_string(), "mere:stack.1");
        assert_eq!(Key::parse("isocosm:600").map(|k| k.id()).as_deref(), Some("isocosm:case:600"));
        assert_eq!(Key::parse("a:b:stack.1"), None);
        assert_eq!(Key::parse("naming.1"), Some(Key::Statement("naming".into(), 1)));
        assert_eq!(Key::Statement("naming".into(), 1).id(), "statement:naming.1");
        assert_eq!(Key::Plan("sim-execution".into(), 5).id(), "decision:sim-execution.D5");
    }

    #[test]
    fn plan_keywords_come_from_file_names() {
        assert_eq!(plan_keyword("a/2026-10-19_sim_execution_plan.md").as_deref(), Some("sim-execution"));
        assert_eq!(plan_keyword("a/2026-09-18_sim_prior_art_brief.md"), None);
    }

    #[test]
    fn a_fields_line_parses_with_summary_last() {
        let l = parse_links(
            "Links: area Execution · draws on 975, 338 · contradicts sim-execution D3 · plans sim-execution · governs isocosm::stage::Stage · stack · summary Bodies by Arc · copy on write",
        )
        .unwrap();
        assert_eq!(l.area.as_deref(), Some("Execution"));
        assert_eq!(l.draws_on.len(), 2);
        assert!(l.contradicts.contains(&Key::Plan("sim-execution".into(), 3)));
        assert!(l.stack);
        assert_eq!(l.summary.as_deref(), Some("Bodies by Arc · copy on write"));
        assert!(parse_links("Links: drawn from 3").is_err());
    }
}
