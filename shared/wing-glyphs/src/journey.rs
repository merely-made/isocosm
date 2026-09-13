// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use crate::{
    Canon, DEFAULT_MAX_GRANTS, DEFAULT_MAX_JSON_BYTES, DEFAULT_MAX_TRANSITIONS, GlyphId,
    SCHEMA_VERSION, bounded_text,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ProvenanceKind {
    Ability,
    Trait,
    Technique,
    Item,
    Bond,
    Quest,
    Event,
    Custom(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Provenance {
    pub kind: ProvenanceKind,
    pub evidence: String,
    pub context: Option<String>,
}
impl Provenance {
    fn validate(&self) -> Result<(), String> {
        bounded_text(&self.evidence, "evidence", 4096)?;
        if let Some(context) = &self.context {
            bounded_text(context, "context", 4096)?;
        }
        if let ProvenanceKind::Custom(kind) = &self.kind {
            bounded_text(kind, "custom kind", 256)?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VariantPolicy {
    RequireOwnedBase,
    AcquireBase,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Acquisition {
    pub glyph: GlyphId,
    pub provenance: Provenance,
    /// Caller evidence time. Acquisition order is accepted grant order; this
    /// kernel does not assume all callers share a monotonic simulation clock.
    pub tick: u64,
    pub life: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GrantRecord {
    pub glyph: GlyphId,
    pub provenance: Provenance,
    pub tick: u64,
    pub life: u64,
    pub policy: VariantPolicy,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GrantOutcome {
    /// Zero-based acquisition ordinal.
    Acquired {
        base: GlyphId,
        ordinal: usize,
    },
    Recorded {
        base: GlyphId,
    },
    Duplicate {
        base: GlyphId,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Eligibility {
    pub complete: bool,
    pub can_ascend: bool,
    pub can_wish: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum JourneyTransition {
    Grant { index: usize },
    Experience { amount: u64 },
    Ascend,
    Reincarnate,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JourneyLimits {
    pub grants: usize,
    pub transitions: usize,
}
impl Default for JourneyLimits {
    fn default() -> Self {
        Self {
            grants: DEFAULT_MAX_GRANTS,
            transitions: DEFAULT_MAX_TRANSITIONS,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JourneySnapshot {
    pub version: u32,
    pub individual: String,
    /// Embed the exact immutable correspondence, not only an unchecked name.
    pub canon: Canon,
    pub unlock_thresholds: Vec<u64>,
    pub life: u64,
    pub experience: u64,
    pub divine: bool,
    pub acquisitions: Vec<Acquisition>,
    pub grants: Vec<GrantRecord>,
    pub transitions: Vec<JourneyTransition>,
    #[serde(default)]
    pub limits: JourneyLimits,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "JourneySnapshot", into = "JourneySnapshot")]
pub struct Journey {
    snapshot: JourneySnapshot,
    owned: BTreeMap<GlyphId, usize>,
    evidence: BTreeMap<(GlyphId, String), usize>,
}

/// Groups actual grants in first-seen kind order; no inferred character,
/// obligation, moral judgment, or additional gameplay rule.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct MotifGroup<'a> {
    pub kind: &'a ProvenanceKind,
    pub grants: Vec<&'a GrantRecord>,
}

impl Journey {
    pub fn new(
        individual: String,
        canon: &Canon,
        unlock_thresholds: Vec<u64>,
    ) -> Result<Self, String> {
        Self::new_with_limits(
            individual,
            canon,
            unlock_thresholds,
            JourneyLimits::default(),
        )
    }
    pub fn new_with_limits(
        individual: String,
        canon: &Canon,
        unlock_thresholds: Vec<u64>,
        limits: JourneyLimits,
    ) -> Result<Self, String> {
        bounded_text(&individual, "individual", 256)?;
        if unlock_thresholds.len() != canon.spec().glyphs.len()
            || unlock_thresholds.windows(2).any(|v| v[0] > v[1])
        {
            return Err(
                "unlock thresholds must be nondecreasing and match base glyph count".into(),
            );
        }
        Ok(Self {
            snapshot: JourneySnapshot {
                version: SCHEMA_VERSION,
                individual,
                canon: canon.clone(),
                unlock_thresholds,
                life: 1,
                experience: 0,
                divine: false,
                acquisitions: Vec::new(),
                grants: Vec::new(),
                transitions: Vec::new(),
                limits,
            },
            owned: BTreeMap::new(),
            evidence: BTreeMap::new(),
        })
    }
    pub fn individual(&self) -> &str {
        &self.snapshot.individual
    }
    pub fn canon(&self) -> &Canon {
        &self.snapshot.canon
    }
    pub fn life(&self) -> u64 {
        self.snapshot.life
    }
    pub fn experience(&self) -> u64 {
        self.snapshot.experience
    }
    pub fn is_divine(&self) -> bool {
        self.snapshot.divine
    }
    pub fn acquisitions(&self) -> &[Acquisition] {
        &self.snapshot.acquisitions
    }
    pub fn grants(&self) -> &[GrantRecord] {
        &self.snapshot.grants
    }
    pub fn owns_base(&self, glyph: &str) -> bool {
        self.owned.contains_key(glyph)
    }
    pub fn snapshot(&self) -> JourneySnapshot {
        self.snapshot.clone()
    }

    pub fn grant(
        &mut self,
        glyph: &str,
        provenance: Provenance,
        tick: u64,
        policy: VariantPolicy,
    ) -> Result<GrantOutcome, String> {
        provenance.validate()?;
        let base = self
            .canon()
            .base_id(glyph)
            .ok_or("unknown glyph ID")?
            .to_owned();
        let key = (glyph.to_owned(), provenance.evidence.clone());
        if let Some(&index) = self.evidence.get(&key) {
            let prior = &self.snapshot.grants[index];
            if prior.provenance != provenance {
                return Err("same glyph evidence has conflicting provenance".into());
            }
            // Retried evidence remains idempotent across later ticks/lives.
            return Ok(GrantOutcome::Duplicate { base });
        }
        if self.snapshot.grants.len() >= self.snapshot.limits.grants {
            return Err("journey grant limit reached".into());
        }
        self.check_transition_capacity()?;
        let new = !self.owned.contains_key(&base);
        if new && glyph != base && policy == VariantPolicy::RequireOwnedBase {
            return Err("variant requires its base to be owned".into());
        }
        let ordinal = self.snapshot.acquisitions.len();
        if new {
            self.snapshot.acquisitions.push(Acquisition {
                glyph: base.clone(),
                provenance: provenance.clone(),
                tick,
                life: self.snapshot.life,
            });
            self.owned.insert(base.clone(), ordinal);
        }
        self.evidence.insert(key, self.snapshot.grants.len());
        self.snapshot.transitions.push(JourneyTransition::Grant {
            index: self.snapshot.grants.len(),
        });
        self.snapshot.grants.push(GrantRecord {
            glyph: glyph.to_owned(),
            provenance,
            tick,
            life: self.snapshot.life,
            policy,
        });
        Ok(if new {
            GrantOutcome::Acquired { base, ordinal }
        } else {
            GrantOutcome::Recorded { base }
        })
    }

    pub fn eligibility(&self) -> Eligibility {
        let complete = !self.canon().spec().glyphs.is_empty()
            && self.owned.len() == self.canon().spec().glyphs.len();
        Eligibility {
            complete,
            can_ascend: complete && !self.is_divine(),
            can_wish: complete,
        }
    }
    /// Caller-authorized model transition. Products own admission and effects.
    pub fn ascend(&mut self) -> Result<(), String> {
        if !self.eligibility().can_ascend {
            return Err("journey is not eligible to ascend".into());
        }
        self.check_transition_capacity()?;
        self.snapshot.divine = true;
        self.snapshot.transitions.push(JourneyTransition::Ascend);
        Ok(())
    }
    pub fn add_experience(&mut self, amount: u64) -> Result<(), String> {
        self.check_transition_capacity()?;
        self.snapshot.experience = self
            .snapshot
            .experience
            .checked_add(amount)
            .ok_or("experience overflow")?;
        self.snapshot
            .transitions
            .push(JourneyTransition::Experience { amount });
        Ok(())
    }
    pub fn reincarnate(&mut self) -> Result<(), String> {
        if !self.is_divine() {
            return Err("only a divine journey can reincarnate".into());
        }
        self.check_transition_capacity()?;
        let life = self
            .snapshot
            .life
            .checked_add(1)
            .ok_or("life counter overflow")?;
        self.snapshot.life = life;
        self.snapshot.experience = 0;
        self.snapshot
            .transitions
            .push(JourneyTransition::Reincarnate);
        Ok(())
    }
    /// The acquired prefix admitted by this life's experience thresholds.
    /// This does not itself grant inherent powers or perform product actions.
    pub fn unlocked(&self) -> &[Acquisition] {
        let count = self
            .snapshot
            .unlock_thresholds
            .iter()
            .take(self.acquisitions().len())
            .take_while(|&&threshold| threshold <= self.experience())
            .count();
        &self.acquisitions()[..count]
    }
    /// Reincarnated divine access, separately named from ordinary acquisition.
    pub fn inherent_unlocked(&self) -> &[Acquisition] {
        if self.is_divine() && self.life() > 1 {
            self.unlocked()
        } else {
            &[]
        }
    }
    pub fn motif(&self) -> Vec<MotifGroup<'_>> {
        let mut groups: Vec<MotifGroup<'_>> = Vec::new();
        for grant in self.grants() {
            if let Some(group) = groups.iter_mut().find(|g| *g.kind == grant.provenance.kind) {
                group.grants.push(grant);
            } else {
                groups.push(MotifGroup {
                    kind: &grant.provenance.kind,
                    grants: vec![grant],
                });
            }
        }
        groups
    }
    /// The founding evidence prefix at first ascension. Later grants remain
    /// part of the ongoing journey without rewriting this basis.
    pub fn ascension_basis(&self) -> Option<&[GrantRecord]> {
        let mut count = 0;
        for transition in &self.snapshot.transitions {
            match transition {
                JourneyTransition::Grant { .. } => count += 1,
                JourneyTransition::Ascend => return Some(&self.grants()[..count]),
                _ => {},
            }
        }
        None
    }
    pub fn to_json(&self) -> Result<String, String> {
        self.to_json_with_limit(DEFAULT_MAX_JSON_BYTES)
    }
    pub fn to_json_with_limit(&self, limit: usize) -> Result<String, String> {
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        if json.len() > limit {
            return Err("journey JSON exceeds limit".into());
        }
        Ok(json)
    }
    pub fn from_json(json: &str) -> Result<Self, String> {
        Self::from_json_with_limit(json, DEFAULT_MAX_JSON_BYTES)
    }
    pub fn from_json_with_limit(json: &str, limit: usize) -> Result<Self, String> {
        if json.len() > limit {
            return Err("journey JSON exceeds limit".into());
        }
        serde_json::from_str(json).map_err(|e| e.to_string())
    }
    pub fn from_snapshot(snapshot: JourneySnapshot) -> Result<Self, String> {
        if snapshot.version != SCHEMA_VERSION
            || snapshot.life == 0
            || snapshot.grants.len() > snapshot.limits.grants
            || snapshot.transitions.len() > snapshot.limits.transitions
        {
            return Err("invalid journey version, life or grant count".into());
        }
        let mut rebuilt = Self::new_with_limits(
            snapshot.individual.clone(),
            &snapshot.canon,
            snapshot.unlock_thresholds.clone(),
            snapshot.limits.clone(),
        )?;
        for transition in &snapshot.transitions {
            match *transition {
                JourneyTransition::Grant { index } => {
                    if index != rebuilt.grants().len() {
                        return Err("invalid grant transition order".into());
                    }
                    let grant = snapshot
                        .grants
                        .get(index)
                        .ok_or("grant transition references missing record")?;
                    if grant.life != rebuilt.life() {
                        return Err("grant life disagrees with transitions".into());
                    }
                    if matches!(
                        rebuilt.grant(
                            &grant.glyph,
                            grant.provenance.clone(),
                            grant.tick,
                            grant.policy
                        )?,
                        GrantOutcome::Duplicate { .. }
                    ) {
                        return Err("snapshot contains duplicate grant evidence".into());
                    }
                },
                JourneyTransition::Experience { amount } => rebuilt.add_experience(amount)?,
                JourneyTransition::Ascend => rebuilt.ascend()?,
                JourneyTransition::Reincarnate => rebuilt.reincarnate()?,
            }
        }
        if rebuilt.snapshot != snapshot {
            return Err("journey state disagrees with transition history".into());
        }
        Ok(rebuilt)
    }
    fn check_transition_capacity(&self) -> Result<(), String> {
        if self.snapshot.transitions.len() >= self.snapshot.limits.transitions {
            return Err("journey transition limit reached".into());
        }
        Ok(())
    }
}
impl TryFrom<JourneySnapshot> for Journey {
    type Error = String;
    fn try_from(value: JourneySnapshot) -> Result<Self, String> {
        Self::from_snapshot(value)
    }
}
impl From<Journey> for JourneySnapshot {
    fn from(value: Journey) -> Self {
        value.snapshot
    }
}

#[cfg(test)]
#[path = "journey_tests.rs"]
mod tests;
