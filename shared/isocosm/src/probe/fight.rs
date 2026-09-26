// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! An escalated fight between two contesters (rulings 221 to 223), written
//! once for both runners. Each round is an exchange: the side standing lower
//! loses it, unless an upset turns it. Both sides take the round's strain,
//! and the loser spends reserve, which strains it further. A loser left with
//! no reserve is spent. A side whose strain passes its bearing breaks, once
//! in a fight, up or down by its traits, the moment and a draw; the break
//! moves its standing for the rest of the fight and decides nothing. Then
//! both size each other up again, and one outmatched by more than the margin
//! yields.

use super::{
    Competition, Competitor,
    aggregate::{self, Seen},
    draws::Stream,
};
use crate::{
    Result,
    meaning::value,
    rules::{Mind, Rules},
    schema::*,
    simulation::Work,
};
use std::{cmp::Ordering, collections::BTreeMap};

#[cfg(test)]
mod tests;

/// A fight's two sides, held by whichever runner is fighting.
pub(super) trait Sides {
    /// Takes one act for one side through the interpreter's meanings.
    fn act(&mut self, side: usize, process: &str) -> Result<()>;
    fn member(&self, side: usize) -> &Entity;
}

/// What an act makes of a state, as far as a round has needed to know.
pub(super) type Known = BTreeMap<(Entity, Key), Entity>;

/// Two members fighting as copies of their states at the tick's start
/// (ruling 240), whatever they take settled only at its end. The crowd
/// remembers what each act makes of a state; the exact runner works each
/// act out again.
pub(super) struct Copies<'a> {
    pub states: [Entity; 2],
    pub known: Option<&'a mut Known>,
    pub rules: &'a Rules,
    pub ground: &'a Ground<'a>,
    pub work: &'a mut Work,
}

impl Sides for Copies<'_> {
    fn act(&mut self, side: usize, process: &str) -> Result<()> {
        let key = (self.states[side].clone(), process.to_string());
        if let Some(next) = self.known.as_ref().and_then(|k| k.get(&key)) {
            self.states[side] = next.clone();
            return Ok(());
        }
        let p = &self.rules.processes[process];
        // Only an act whose meaning does not depend on the site can be
        // taken apart from it.
        if !aggregate::site_free(p) {
            return Err(format!("{process} reads the site; a fight cannot take it"));
        }
        self.work.evaluations += 1;
        self.work.represented += 1;
        let mut site = self.ground.site.clone();
        let needs = crate::meaning::needs(self.rules);
        let next = aggregate::apply(p, &key.0, &mut site, 1, self.ground.tick, needs)?
            .ok_or_else(|| format!("{process} was blocked in a fight"))?;
        self.work.accepted += 1;
        self.states[side] = next.clone();
        if let Some(known) = self.known.as_mut() {
            known.insert(key, next);
        }
        Ok(())
    }
    fn member(&self, side: usize) -> &Entity {
        &self.states[side]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Act {
    Round,
    Spend,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fight {
    pub winner: usize,
    /// Each side's acts, in the order it took them.
    pub acts: [Vec<Act>; 2],
    /// Whether each side broke, and which way: `Some(true)` is up.
    pub breaks: [Option<bool>; 2],
}

/// What a mind can bear before it breaks (ruling 164), from its traits.
pub fn bearing(mind: &Mind, e: &Entity) -> i64 {
    let traits: i64 = e
        .traits
        .iter()
        .filter_map(|t| mind.bearing_traits.get(t))
        .sum();
    mind.bearing.saturating_add(traits)
}

/// Whether a mind's kept strain has passed its bearing.
pub fn past_bearing(mind: &Mind, e: &Entity) -> bool {
    let strain = i64::try_from(value(&e.accounts, &mind.strain)).unwrap_or(i64::MAX);
    strain > bearing(mind, e)
}

/// Per mille, the chance a break goes up (ruling 163): what the traits give
/// and what the moment adds, the moment read as the mood.
pub fn rise(mind: &Mind, e: &Entity, mood: i64) -> u64 {
    let traits: i64 = e
        .traits
        .iter()
        .filter_map(|t| mind.rise_traits.get(t))
        .sum();
    mind.rise
        .saturating_add(traits)
        .saturating_add(mind.stake.saturating_mul(mood))
        .clamp(0, 1000) as u64
}

/// The site as it stood when the round began, which both runners read.
pub(super) struct Ground<'a> {
    pub site: &'a Site,
    pub tick: Tick,
}

fn reserve(sides: &impl Sides, kinds: [&Competitor; 2], side: usize) -> u64 {
    value(&sides.member(side).accounts, &kinds[side].body)
}

fn standing(sides: &impl Sides, kinds: [&Competitor; 2], shift: &[i64; 2], side: usize) -> i64 {
    let body = i64::try_from(reserve(sides, kinds, side)).unwrap_or(i64::MAX);
    body.saturating_add(shift[side])
}

pub(super) fn fight(
    c: &Competition,
    mind: &Mind,
    kinds: [&Competitor; 2],
    ground: &Ground,
    sides: &mut impl Sides,
    s: &mut Stream,
) -> Result<Fight> {
    let advantage = i64::try_from(c.advantage).unwrap_or(i64::MAX);
    let mut shift = [0i64; 2];
    let mut breaks = [None; 2];
    let mut acts: [Vec<Act>; 2] = Default::default();
    loop {
        let (a, b) = (
            standing(sides, kinds, &shift, 0),
            standing(sides, kinds, &shift, 1),
        );
        let mut loser = match a.cmp(&b) {
            Ordering::Less => 0,
            Ordering::Greater => 1,
            Ordering::Equal => s.below(2) as usize,
        };
        if s.below(1000) < u64::from(c.upset) {
            loser = 1 - loser;
        }
        for (side, taken) in acts.iter_mut().enumerate() {
            sides.act(side, &c.round)?;
            taken.push(Act::Round);
        }
        for _ in 0..c.cost.min(reserve(sides, kinds, loser)) {
            let before = reserve(sides, kinds, loser);
            sides.act(loser, &kinds[loser].spend)?;
            if reserve(sides, kinds, loser) >= before {
                return Err(format!("{} spent no reserve", kinds[loser].spend));
            }
            acts[loser].push(Act::Spend);
        }
        if reserve(sides, kinds, loser) == 0 {
            return Ok(Fight {
                winner: 1 - loser,
                acts,
                breaks,
            });
        }
        for side in 0..2 {
            let e = sides.member(side);
            if breaks[side].is_some() || !past_bearing(mind, e) {
                continue;
            }
            let seen = Seen {
                member: Some(e),
                site: ground.site,
                tick: ground.tick,
                needs: &mind.needs,
            };
            let up = s.below(1000) < rise(mind, e, seen.mood()?);
            breaks[side] = Some(up);
            shift[side] = if up { advantage } else { -advantage };
        }
        let (a, b) = (
            standing(sides, kinds, &shift, 0),
            standing(sides, kinds, &shift, 1),
        );
        if a.abs_diff(b) > c.margin {
            return Ok(Fight {
                winner: usize::from(b > a),
                acts,
                breaks,
            });
        }
    }
}
