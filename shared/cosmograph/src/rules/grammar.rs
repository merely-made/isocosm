// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A lineage's growth grammar (cases 1202, 1206): a short list of rules
//! applied in order to a seed of tagmata, which unfolds into the tagmata a
//! body develops from. The grammar is the genotype and the tagmata the plan
//! it unfolds into; nothing that reads the tagmata changes, and a lineage
//! with no grammar is the lineage it always was.
//!
//! A child tagma joins its parent only at the parent's base, middle or tip,
//! so a run bears side runs at the joints of its own pieces: `Branch` cuts
//! the bearer into consecutive tagmata of `every` segments, each chained to
//! the last at its tip, and each whole piece bears one side run at `anchor`.

use super::{Anchor, Development, Facing, Tagma};
use crate::{Result, schema::Key};
use serde::{Deserialize, Serialize};

/// The most tagmata a recipe may unfold to; a parent is named by a byte.
const MOST: usize = 255;

/// The positive direction along each axis: right, above and back.
const ALONG: [Facing; 3] = [Facing::Right, Facing::Above, Facing::Back];

/// A lineage's growth grammar (cases 1202, 1206): rules applied in order to
/// a seed, unfolding into the tagmata a body develops from.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Grammar {
    pub seed: Vec<Tagma>,
    pub rules: Vec<Rule>,
}

/// One growth rule; `tagma` indexes the tagmata as they stand when it applies.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Rule {
    /// Lengthen run `tagma` by `by` segments.
    Repeat { tagma: u8, by: u8 },
    /// Run `tagma` bears side runs like itself: one at every `every`th
    /// segment, each `taper` segments shorter than its bearer, to `depth`
    /// levels, joined at `anchor`, turned to `facing`.
    Branch { tagma: u8, every: u8, depth: u8, taper: u8, anchor: Anchor, facing: Facing },
    /// From segment `at` on, run `tagma`'s segments are of `kind`
    /// (a run splits in two: its near part keeps its kind).
    Differentiate { tagma: u8, at: u8, kind: Key },
    /// Cap the body's segments at `total`, shortening the longest runs first.
    Allocate { total: u16 },
}

/// The tagmata `grammar` unfolds into: its rules applied in order to a copy
/// of its seed (cases 1202, 1206).
///
/// # Errors
/// Fails if a rule names a tagma out of range, a branch has no `every`, a
/// branch's turn runs along its bearer or past two levels, or the tagmata
/// pass 255.
pub fn unfold(grammar: &Grammar) -> Result<Vec<Tagma>> {
    let mut t = grammar.seed.clone();
    if t.len() > MOST {
        return Err(format!("a grammar unfolds to more than {MOST} tagmata"));
    }
    for rule in &grammar.rules {
        match rule {
            Rule::Repeat { tagma, by } => {
                let run = named(&mut t, *tagma)?;
                run.segments = run.segments.saturating_add(*by);
            },
            Rule::Branch { tagma, every, depth, taper, anchor, facing } => {
                branch(&mut t, [*tagma, *every, *depth, *taper], *anchor, *facing)?;
            },
            Rule::Differentiate { tagma, at, kind } => differentiate(&mut t, *tagma, *at, kind)?,
            Rule::Allocate { total } => allocate(&mut t, usize::from(*total)),
        }
    }
    Ok(t)
}

/// `d` with its tagmata unfolded afresh from its grammar, as when a rule has
/// been edited (cases 1202, 1206).
///
/// # Errors
/// Fails if `d` has no grammar or its grammar does not unfold.
pub fn regrown(d: &Development) -> Result<Development> {
    let grammar = d.grammar.as_ref().ok_or("a development with no grammar")?;
    let mut grown = d.clone();
    grown.recipe.tagmata = unfold(grammar)?;
    Ok(grown)
}

fn named(t: &mut [Tagma], tagma: u8) -> Result<&mut Tagma> {
    t.get_mut(usize::from(tagma)).ok_or_else(|| format!("a rule names tagma {tagma}, which is not there"))
}

fn push(t: &mut Vec<Tagma>, tagma: Tagma) -> Result<u8> {
    if t.len() >= MOST {
        return Err(format!("a grammar unfolds to more than {MOST} tagmata"));
    }
    t.push(tagma);
    Ok((t.len() - 1) as u8)
}

/// A run as it stands: the tagma that heads it and its length.
struct Run {
    head: u8,
    len: u8,
}

/// Cuts `run` into pieces of `every` segments, the first kept in place and
/// the rest appended, each joined to the one before at its tip. Each piece
/// comes back with whether it is whole.
fn cut(t: &mut Vec<Tagma>, run: &Run, every: u8) -> Result<Vec<(u8, bool)>> {
    let mut pieces = vec![(run.head, run.len >= every)];
    if run.len > every {
        let proto = t[usize::from(run.head)].clone();
        t[usize::from(run.head)].segments = every;
        let (mut rest, mut prev) = (run.len - every, run.head);
        while rest > 0 {
            let n = rest.min(every);
            prev = push(t, Tagma { segments: n, parent: Some(prev), anchor: Anchor::Tip, ..proto.clone() })?;
            pieces.push((prev, n == every));
            rest -= n;
        }
    }
    Ok(pieces)
}

/// `socket` if it clears a run along axis `run` and side runs along `next`,
/// else the first axis that does, facing its positive way.
fn clear(socket: Facing, run: usize, next: Option<usize>) -> Facing {
    let free = |a: &usize| *a != run && Some(*a) != next;
    match socket.axis().0 {
        a if free(&a) => socket,
        _ => (0..3).find(free).map_or(socket, |a| ALONG[a]),
    }
}

/// Branches run `[tagma, every, depth, taper]` (cases 1202, 1206).
///
/// A run is cut into pieces of `every` segments and each whole piece bears
/// one side run at `anchor`: at the piece's tip, the segment where
/// `(s + 1) % every == 0`, or at its base or middle. A side run is a copy of
/// its bearer `taper` shorter, itself cut and branched in the next level.
/// Level one turns to `facing`; level two turns to the third axis, the one
/// neither the spine nor level one runs along, so no run meets a sister.
/// Later tagmata that named the bearer keep naming its first piece.
fn branch(t: &mut Vec<Tagma>, [tagma, every, depth, taper]: [u8; 4], anchor: Anchor, facing: Facing) -> Result<()> {
    let bearer = named(t, tagma)?.facing.axis().0;
    if every == 0 {
        return Err("a branch at every 0th segment".into());
    }
    if depth > 2 {
        return Err("a branch deeper than two levels would turn back on its sisters".into());
    }
    let turn = facing.axis().0;
    if turn == bearer {
        return Err("a branch turned along the run that bears it".into());
    }
    let turns = [facing, ALONG[3 - bearer - turn]];
    let mut runs = vec![Run { head: tagma, len: t[usize::from(tagma)].segments }];
    for level in 0..usize::from(depth) {
        let (run_axis, next) = (turns[level].axis().0, (level + 1 < usize::from(depth)).then(|| turns[1].axis().0));
        let mut made = vec![];
        for run in &runs {
            let proto = t[usize::from(run.head)].clone();
            for (piece, whole) in cut(t, run, every)? {
                let len = run.len.saturating_sub(taper);
                if !whole || len == 0 {
                    continue;
                }
                let side = Tagma {
                    segments: len,
                    parent: Some(piece),
                    anchor,
                    facing: turns[level],
                    socket: if proto.bears.is_some() { clear(proto.socket, run_axis, next) } else { proto.socket },
                    ..proto.clone()
                };
                made.push(Run { head: push(t, side)?, len });
            }
        }
        runs = made;
    }
    Ok(())
}

/// Splits run `tagma` at `at` into its near part, kept in place, and a far
/// part of `kind` appended behind it (cases 1202, 1206). A tagma that named
/// the run as parent keeps naming the near part, so a joint at its middle or
/// tip moves back to the near part's; one that follows it in the list
/// continues from the near part's last segment.
fn differentiate(t: &mut Vec<Tagma>, tagma: u8, at_: u8, kind: &Key) -> Result<()> {
    let run = named(t, tagma)?;
    if at_ == 0 || at_ >= run.segments {
        return Ok(());
    }
    let far = Tagma {
        segments: run.segments - at_,
        segment: kind.clone(),
        parent: Some(tagma),
        anchor: Anchor::Tip,
        ..run.clone()
    };
    run.segments = at_;
    push(t, far).map(|_| ())
}

/// Shortens the longest run by one, the highest index of a tie, until the
/// segments come to `total` or every run is down to one.
fn allocate(t: &mut [Tagma], total: usize) {
    while t.iter().map(|r| usize::from(r.segments)).sum::<usize>() > total {
        let Some((longest, _)) = t.iter().enumerate().max_by_key(|(i, r)| (r.segments, *i)) else {
            return;
        };
        if t[longest].segments <= 1 {
            return;
        }
        t[longest].segments -= 1;
    }
}

#[cfg(test)]
mod tests;
