// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A lineage's policy (case 1051; mesocosm-overlay M3.5): a small integer
//! net scoring the processes the deliberative chooser offers, from the
//! chooser's own terms. A direct path carries the hand-written score, so a
//! founding policy chooses as the methodology always has (D2); a hidden
//! layer reads the same terms and each process's shape. A nudge's sway is
//! added outside it (D3). It proposes among the acts the body and the world
//! admit; the resolvers dispose. Each member chooses by its lineage's policy
//! varied by its own draw (Ecology 1's reroll), and at a boundary the
//! variants of members that prospered are weighed like any revision.

use crate::{
    Result, Session,
    history::Command,
    lineage::boundary::Candidate,
    rng::Rng,
    schema::*,
    simulation::Simulation,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// The direct path's weights are in 64ths: 64 carries a term unchanged.
pub const UNIT: i16 = 64;
/// The hidden layer's units.
const HIDDEN: usize = 8;
/// Its inputs: needs, mood, priority, the four shape flags, and a bias.
const INPUTS: usize = 8;
/// Inputs are quantized into `-LIMIT..=LIMIT`; units clamp into `0..=LIMIT`.
const LIMIT: i64 = 32;
/// How much need, in the hand-written score's units, one input step is.
const NEED_STEP: i64 = 100;

/// A lineage's policy: integer weights, so every peer replays it exactly.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LineagePolicy {
    /// Each of the world's needs' weight on the direct path, in the world's
    /// order, in 64ths: the lineage's drive toward serving it.
    pub drives: Vec<i16>,
    /// Priority's weight on the direct path, in 64ths.
    pub priority: i16,
    /// Each hidden unit's weights over the inputs, the last its bias.
    pub hidden: Vec<[i8; INPUTS]>,
    /// Each hidden unit's weight on the score.
    pub out: Vec<i8>,
}

/// What the policy reads of one offered process: the hand-written score's
/// own terms, and the process's shape.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Terms {
    /// Each pressing need the process serves, by its place in the world's
    /// list, at the hand-written weight: its pressure lifted by low mood.
    pub needs: Vec<(usize, i64)>,
    pub mood: i64,
    pub priority: i64,
    /// Whether it moves, hunts, acts on the ground, and has a target.
    pub shape: [bool; 4],
}

impl LineagePolicy {
    /// The policy that scores as the hand-written methodology does (D2).
    pub fn founding(needs: usize) -> Self {
        Self {
            drives: vec![UNIT; needs],
            priority: UNIT,
            hidden: vec![[0; INPUTS]; HIDDEN],
            out: vec![0; HIDDEN],
        }
    }

    /// `self` with every weight moved by up to `size` either way, drawn
    /// from `seed`'s stream in a fixed order.
    pub fn varied(&self, seed: u64, size: i16) -> Self {
        if size == 0 {
            return self.clone();
        }
        let mut rng = Rng::from_seed(seed);
        let s = i32::from(size.abs());
        let mut step = |w: i32, bound: i32| (w + rng.range_i32(-s, s)).clamp(-bound, bound);
        let wide = i32::from(i16::MAX);
        let narrow = i32::from(i8::MAX);
        let drives = self.drives.iter().map(|&w| step(w.into(), wide) as i16).collect();
        let priority = step(self.priority.into(), wide) as i16;
        let hidden = self
            .hidden
            .iter()
            .map(|unit| unit.map(|w| step(w.into(), narrow) as i8))
            .collect();
        let out = self.out.iter().map(|&w| step(w.into(), narrow) as i8).collect();
        Self { drives, priority, hidden, out }
    }

    /// What `t` scores: the direct path, divided once so a founding policy
    /// reproduces the hand-written sum exactly, plus the hidden layer.
    pub fn score(&self, t: &Terms) -> i64 {
        let drive = |k: usize| i64::from(self.drives.get(k).copied().unwrap_or(UNIT));
        let direct: i64 = t.needs.iter().map(|&(k, v)| drive(k) * v).sum::<i64>()
            + i64::from(self.priority) * t.priority;
        direct / i64::from(UNIT) + self.hidden_score(t)
    }

    fn hidden_score(&self, t: &Terms) -> i64 {
        if self.out.iter().all(|&w| w == 0) {
            return 0;
        }
        let needs: i64 = t.needs.iter().map(|&(_, v)| v).sum();
        let flag = |b: bool| if b { LIMIT } else { 0 };
        let [moves, hunts, ground, aimed] = t.shape;
        let x = [
            (needs / NEED_STEP).clamp(-LIMIT, LIMIT),
            t.mood.clamp(-LIMIT, LIMIT),
            t.priority.clamp(-LIMIT, LIMIT),
            flag(moves),
            flag(hunts),
            flag(ground),
            flag(aimed),
            LIMIT,
        ];
        self.hidden
            .iter()
            .zip(&self.out)
            .map(|(unit, &o)| {
                let sum: i64 = unit.iter().zip(&x).map(|(&w, &x)| i64::from(w) * x).sum();
                (sum / LIMIT).clamp(0, LIMIT) * i64::from(o)
            })
            .sum()
    }
}

/// How a lineage's choices went since a reader last took the count: how
/// many a policy made, and how many the hand-written score would not have
/// made, its best not among the chosen act's equals.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Tally {
    pub decisions: u64,
    pub otherwise: u64,
}

/// One lineage at a reading: its tally, standing and drives.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reading {
    pub lineage: Key,
    pub tally: Tally,
    pub held: u128,
    pub members: u64,
    pub drives: Vec<i16>,
}

impl Simulation {
    /// The policy `actor` chooses by: its lineage's, varied by its own draw
    /// as far as the world's rules say, or none.
    pub(crate) fn expressed(&self, actor: Id, size: i16) -> Option<LineagePolicy> {
        let e = self.body_at_start(actor)?;
        let policy = self.state.lineages.get(&e.lineage)?.policy.as_ref()?;
        let mut policy = match e.drawn {
            Some(drawn) => policy.varied(drawn, size),
            None => policy.clone(),
        };
        // What the member has learned is added to its drives (M3.6).
        if let Some(delta) = self.learned_of(e) {
            for (&k, &w) in &delta.weights {
                if let Some(d) = policy.drives.get_mut(k as usize) {
                    *d = (i32::from(*d) + w).clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16;
                }
            }
        }
        Some(policy)
    }

    /// Commits `policy` as `lineage`'s. Its living members keep their
    /// draws; a cohort never drawn takes one by the world's seed and its
    /// identity, its members sharing it as they share all else.
    pub(crate) fn adopt_policy(&mut self, lineage: &str, policy: &LineagePolicy) -> Result<()> {
        let needs = crate::meaning::needs(&self.genesis.rules).len();
        if policy.drives.len() != needs || policy.hidden.len() != policy.out.len() {
            return Err("a policy's shape does not fit the world's needs".into());
        }
        let seed = self.genesis.dynamics_seed();
        let l = self
            .state
            .lineages
            .get_mut(lineage)
            .ok_or("unknown lineage")?;
        l.policy = Some(policy.clone());
        for (&first, cohort) in &mut self.state.population.groups {
            let e = &cohort.entity;
            if e.alive && e.lineage == lineage && e.drawn.is_none() {
                cohort.entity_mut().drawn = Some(crate::draw(seed, "policy:expression", &[first]));
            }
        }
        Ok(())
    }

    /// Counts a choice a policy made for `actor`'s lineage.
    pub(super) fn tally_choice(&mut self, actor: Id, otherwise: bool) {
        let Some(lineage) = self.body_at_start(actor).map(|e| e.lineage.clone()) else {
            return;
        };
        let t = self.chosen.tally.entry(lineage).or_default();
        t.decisions += 1;
        t.otherwise += u64::from(otherwise);
    }
}

/// The command that founds `lineage`'s policy: the hand-written score,
/// varied by the world's founding size and the lineage's own draw (D2).
pub fn founding(session: &Session, lineage: &str) -> Command {
    let sim = &session.sim;
    let rules = &sim.genesis().rules;
    let size = rules.directing().policy.founding;
    let seed = crate::draw(sim.genesis().dynamics_seed(), &format!("policy:founding:{lineage}"), &[]);
    let needs = crate::meaning::needs(rules).len();
    Command::Policy {
        lineage: lineage.into(),
        policy: LineagePolicy::founding(needs).varied(seed, size),
    }
}

/// The boundary's policy candidates for `lineage`: the variants its living
/// members express, those holding the most matter first, each distinct
/// draw once, at most the world's count (M3.5).
pub fn candidates(session: &Session, lineage: &str) -> Vec<Candidate> {
    let sim = &session.sim;
    let rules = &sim.genesis().rules;
    let v = rules.directing().policy;
    let Some(policy) = sim.state().lineages.get(lineage).and_then(|l| l.policy.as_ref()) else {
        return vec![];
    };
    if v.candidates == 0 || v.expression == 0 {
        return vec![];
    }
    let mut members: Vec<(u128, Id, u64)> = sim
        .state()
        .population
        .groups
        .iter()
        .filter(|(_, g)| g.entity.alive && g.entity.lineage == lineage)
        .filter_map(|(&id, g)| {
            let held = crate::meaning::mass(&crate::anatomy::books(&g.entity), rules);
            Some((held, id, g.entity.drawn?))
        })
        .collect();
    members.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    let mut seen = BTreeSet::new();
    members
        .into_iter()
        .filter(|m| seen.insert(m.2))
        .take(v.candidates as usize)
        .map(|(_, id, drawn)| Candidate {
            name: format!("policy:member-{id}"),
            commands: vec![Command::Policy {
                lineage: lineage.into(),
                policy: policy.varied(drawn, v.expression),
            }],
        })
        .collect()
}

/// Each lineage with a policy: its tally since the last reading, which this
/// takes, its standing and its drives.
pub fn readout(session: &mut Session) -> Vec<Reading> {
    let tallies: BTreeMap<Key, Tally> = std::mem::take(&mut session.sim.chosen.tally);
    let lineages = session.sim.state().lineages.clone();
    lineages
        .iter()
        .filter_map(|(lineage, l)| {
            let policy = l.policy.as_ref()?;
            let (held, members) = crate::lineage::boundary::standing(session, lineage);
            Some(Reading {
                lineage: lineage.clone(),
                tally: tallies.get(lineage).copied().unwrap_or_default(),
                held,
                members,
                drives: policy.drives.clone(),
            })
        })
        .collect()
}
