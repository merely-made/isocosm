//! Bounded revision history. `full` retains every revision's tree; `text`
//! retains trees only for the current revision and snapshots still held by a
//! reader, reparsing older text on demand; `cached` retains trees and parses
//! through one document-scoped Rowan node cache.
use super::*;
use std::{
    collections::HashSet,
    hash::{BuildHasherDefault, Hasher},
    sync::Weak,
};
use taplo::rowan::{GreenNodeData, NodeCache, NodeOrToken};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Mode {
    Full,
    Text,
    Cached,
}
impl Mode {
    /// Measured 2026-10-19 on the annotated pack: at 65 revisions `text` retains
    /// 98,944 requested bytes against 537,240 (`cached`) and 849,544 (`full`),
    /// with publication time within noise of `full`. No command reads an old
    /// revision's tree; undo needs only its text.
    pub(super) const DEFAULT: Self = Self::Text;
}
impl std::str::FromStr for Mode {
    type Err = String;
    fn from_str(name: &str) -> Result<Self, String> {
        match name {
            "full" => Ok(Self::Full),
            "text" => Ok(Self::Text),
            "cached" => Ok(Self::Cached),
            _ => Err("history mode is full, text or cached".into()),
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub(super) struct Policy {
    pub(super) mode: Mode,
    /// Count cap, including the current revision.
    pub(super) limit: usize,
    /// Estimated retained bytes; the oldest revisions go first, never the current one.
    pub(super) budget: usize,
}
impl Policy {
    pub(super) const DEFAULT_LIMIT: usize = 32;
    pub(super) const DEFAULT_BUDGET: usize = 64 * 1024 * 1024;
    pub(super) fn new(mode: Mode, limit: usize) -> Self {
        Self {
            mode,
            limit: limit.clamp(1, 256),
            budget: Self::DEFAULT_BUDGET,
        }
    }
}
pub(super) enum Retained {
    Tree(Arc<Snapshot>),
    Text {
        text: Arc<str>,
        #[cfg_attr(not(test), allow(dead_code))]
        pinned: Weak<Snapshot>,
    },
}
impl Retained {
    fn bytes(&self) -> usize {
        match self {
            Self::Tree(snapshot) => snapshot.bytes,
            Self::Text { text, .. } => text.len(),
        }
    }
}
pub(super) struct History {
    pub(super) policy: Policy,
    pub(super) entries: BTreeMap<u64, Retained>,
    cache: Option<NodeCache>,
    cached_parses: usize,
}
impl History {
    pub(super) fn new(policy: Policy) -> Self {
        Self {
            policy,
            entries: BTreeMap::new(),
            cache: None,
            cached_parses: 0,
        }
    }
    pub(super) fn len(&self) -> usize {
        self.entries.len()
    }
    pub(super) fn retained_bytes(&self) -> usize {
        self.entries.values().map(Retained::bytes).sum()
    }
    /// The cache for the next parse. It never evicts, so it is replaced after
    /// `limit` parses: it then holds at most two history windows' elements.
    pub(super) fn cache(&mut self) -> Option<&mut NodeCache> {
        if self.policy.mode != Mode::Cached {
            return None;
        }
        if self.cached_parses >= self.policy.limit {
            self.cache = None;
            self.cached_parses = 0;
        }
        self.cached_parses += 1;
        Some(self.cache.get_or_insert_with(NodeCache::default))
    }
    pub(super) fn publish(&mut self, previous: Option<&Arc<Snapshot>>, next: Arc<Snapshot>) {
        if let (Mode::Text, Some(previous)) = (self.policy.mode, previous) {
            self.entries.insert(
                previous.revision,
                Retained::Text {
                    text: previous.text.clone(),
                    pinned: Arc::downgrade(previous),
                },
            );
        }
        self.entries.insert(next.revision, Retained::Tree(next));
        while self.entries.len() > self.policy.limit
            || (self.entries.len() > 1 && self.retained_bytes() > self.policy.budget)
        {
            self.entries.pop_first();
        }
    }
    pub(super) fn text(&self, revision: u64) -> Option<Arc<str>> {
        Some(match self.entries.get(&revision)? {
            Retained::Tree(snapshot) => snapshot.text.clone(),
            Retained::Text { text, .. } => text.clone(),
        })
    }
    /// An old revision's tree: a reader's pinned snapshot, else a fresh parse.
    /// No command reads old trees yet; undo needs only [`History::text`].
    #[cfg_attr(not(test), allow(dead_code))]
    pub(super) fn snapshot(&self, revision: u64, format: Format) -> Result<Arc<Snapshot>, String> {
        match self.entries.get(&revision) {
            Some(Retained::Tree(snapshot)) => Ok(snapshot.clone()),
            Some(Retained::Text { text, pinned }) => match pinned.upgrade() {
                Some(snapshot) => Ok(snapshot),
                None => Snapshot::build(revision, text.to_string(), format, None),
            },
            None => Err("revision no longer retained".into()),
        }
    }
    /// Keep only the current revision and drop the node cache.
    #[cfg_attr(not(test), allow(dead_code))]
    pub(super) fn release(&mut self, current: &Arc<Snapshot>) {
        self.entries.clear();
        self.entries
            .insert(current.revision, Retained::Tree(current.clone()));
        self.cache = None;
        self.cached_parses = 0;
    }
}
/// Approximate heap bytes of a revision: its text plus Rowan's green layout
/// (a 24-byte node head with 16 bytes per child; a 24-byte token head plus
/// text). A parse interns tokens and nodes of up to three children, so those
/// are counted once.
pub(super) fn estimate(text: &str, syntax: Option<&taplo::parser::Parse>) -> usize {
    type Seen = HashSet<usize, BuildHasherDefault<Pointer>>;
    fn green(node: &GreenNodeData, seen: &mut Seen) -> usize {
        let children = node.children();
        if children.len() <= 3 && !seen.insert(node as *const _ as usize) {
            return 0;
        }
        24 + children
            .map(|child| {
                16 + match child {
                    NodeOrToken::Node(node) => green(node, seen),
                    NodeOrToken::Token(token) if seen.insert(token as *const _ as usize) => {
                        24 + token.text().len()
                    },
                    NodeOrToken::Token(_) => 0,
                }
            })
            .sum::<usize>()
    }
    text.len() + syntax.map_or(0, |parse| green(&parse.green_node, &mut Seen::default()))
}
/// Hashes element addresses: drops the alignment zeros, spreads by a Fibonacci multiplier.
#[derive(Default)]
struct Pointer(u64);
impl Hasher for Pointer {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, _: &[u8]) {
        unreachable!("only usize keys")
    }
    fn write_usize(&mut self, n: usize) {
        self.0 = (n as u64 >> 3).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    }
}
