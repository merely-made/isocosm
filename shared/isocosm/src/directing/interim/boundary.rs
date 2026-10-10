// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! The interim boundary (ruling 684): a lineage's candidate is worth what a
//! copy of the world grows with it committed, scored over `Session::fork_at`
//! as legacy `adapt_round` scored over a copied world; never by a formula,
//! and the world itself untouched. Lines take their turns from the most
//! numerous down, committing at once, the played line skipped (its turn is
//! the review).

use crate::{Result, Session, schema::*};
use serde::{Deserialize, Serialize};

/// What a copy grew to: the line's living matter and members after `ticks`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Score {
    pub held: u128,
    pub members: u64,
    pub ticks: Tick,
}

impl Score {
    /// Whether this score beats `other`: more matter held, then more members.
    pub fn beats(&self, other: &Score) -> bool {
        (self.held, self.members) > (other.held, other.members)
    }
}

/// A change a line could commit, as the commands that make it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Candidate {
    pub name: Key,
    pub commands: Vec<crate::history::Command>,
}

/// One line's turn: what it weighed and what it committed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Turn {
    pub lineage: Key,
    pub considered: Vec<(Key, Score)>,
    pub committed: Option<Key>,
}

/// The line's living matter and members now.
pub fn standing(session: &Session, lineage: &str) -> (u128, u64) {
    let rules = &session.sim.genesis().rules;
    let living = session.sim.state().population.groups.values();
    living
        .filter(|g| g.entity.alive && g.entity.lineage == lineage)
        .fold((0, 0), |(held, n), g| {
            let mass = crate::meaning::mass(&crate::anatomy::books(&g.entity), rules);
            (held + mass * u128::from(g.count), n + g.count)
        })
}

/// Grows a copy of the world from now with `candidate` committed, nobody
/// at the keyboard, and reads what `lineage` came to.
pub fn score(
    session: &Session,
    lineage: &str,
    candidate: &Candidate,
    ticks: Tick,
) -> Result<Score> {
    let now = session.sim.state().tick;
    let mut copy = session.fork_at(now, format!("branch:score:{}", candidate.name))?;
    for command in &candidate.commands {
        copy.command(command.clone())?;
    }
    copy.advance(ticks)?;
    let (held, members) = standing(&copy, lineage);
    Ok(Score {
        held,
        members,
        ticks,
    })
}

/// One adaptation round: each living line but `played`, most numerous
/// first, weighs its candidates, the status quo among them, and commits
/// the best at once, so later lines answer a world the earlier changed.
pub fn adapt(
    session: &mut Session,
    played: Option<&str>,
    candidates: &dyn Fn(&Session, &str) -> Vec<Candidate>,
    ticks: Tick,
) -> Result<Vec<Turn>> {
    let mut lines: Vec<(u64, Key)> = session
        .sim
        .state()
        .lineages
        .keys()
        .filter(|l| Some(l.as_str()) != played)
        .map(|l| (standing(session, l).1, l.clone()))
        .filter(|(n, _)| *n > 0)
        .collect();
    lines.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
    let mut turns = Vec::new();
    for (_, lineage) in lines {
        let offered = candidates(session, &lineage);
        // A line with nothing but what it has takes no turn.
        if offered.is_empty() {
            continue;
        }
        let stay = Candidate {
            name: "candidate:stay".into(),
            commands: vec![],
        };
        let mut considered = Vec::new();
        let mut best: Option<(Score, &Candidate)> = None;
        for c in std::iter::once(&stay).chain(&offered) {
            let s = score(session, &lineage, c, ticks)?;
            considered.push((c.name.clone(), s));
            if best.is_none_or(|(b, _)| s.beats(&b)) {
                best = Some((s, c));
            }
        }
        let chosen = best.map(|(_, c)| c).filter(|c| !c.commands.is_empty());
        if let Some(c) = chosen {
            for command in &c.commands {
                session.command(command.clone())?;
            }
        }
        turns.push(Turn {
            lineage,
            considered,
            committed: chosen.map(|c| c.name.clone()),
        });
    }
    Ok(turns)
}
