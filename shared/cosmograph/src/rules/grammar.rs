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

use super::{Anchor, Development, Facing, Hardened, Symmetry, Tagma};
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
    /// Run `tagma` lays down dead matter (D42): see [`Accretion`].
    Accrete { tagma: u8, how: Accretion },
    /// Run `tagma` is copied round the vertical, `arms` of it in all (D45):
    /// arms round a centre, petals. The count is stored here; the lattice
    /// shows at most four, spread as evenly as it can.
    Radiate { tagma: u8, arms: u8 },
    /// The body lays down `account` at its place (D47): each tick it pays
    /// `rate` from its reserve into the place's `account`, which decays at
    /// the place by `decay` a tick (D48). Unfolds to nothing; the world's
    /// processes carry it out (`ecology::deposit`).
    Deposit { account: Key, rate: u64, decay: u64 },
    /// Run `tagma` places what it bears by `symmetry` (D41).
    Symmetry { tagma: u8, symmetry: Symmetry },
    /// The rules in `rules` apply only while `when` holds of the body or its
    /// place (D43); they cannot hold a switch of their own.
    Switch { when: Condition, rules: Vec<Rule> },
    /// Cap the body's segments at `total`, shortening the longest runs first.
    Allocate { total: u16 },
}

/// Where a run lays down matter that holds and does nothing (D42). Its kind
/// is one the world gives no cells.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Accretion {
    /// The run's `oldest` segments are of `kind`: wood, coral, a stalk.
    Harden { oldest: u8, kind: Key },
    /// Each segment bears a covering of `kind` at `socket`: shell, bark,
    /// plate. A run that bears already has no room for one.
    Cover { kind: Key, socket: Facing },
}

/// What a switch reads (D43), of the body or of its place.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Condition {
    /// The body is at least `ticks` old.
    Age { ticks: u64 },
    /// Every store the body has is full.
    FullReserves,
    /// At least `bodies` of the body's lineage share its place.
    Crowding { bodies: u32 },
    /// The world's year is within `from` to `to`, in thousandths of the year
    /// from its start, wrapping past the end where `from` is the greater
    /// (D49): a season as a phase of the wave climate reads.
    Season { from: u16, to: u16 },
}

/// The thousandths in a year, which a season is a range of.
pub const YEAR: u16 = 1000;

/// What a body and its place show a switch to read.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Situation {
    pub age: u64,
    pub reserves_full: bool,
    pub crowd: u32,
    /// Where the world's year stands, in thousandths from its start.
    pub year: u16,
}

impl Condition {
    /// Whether it holds of `now`.
    pub fn holds(&self, now: &Situation) -> bool {
        match *self {
            Self::Age { ticks } => now.age >= ticks,
            Self::FullReserves => now.reserves_full,
            Self::Crowding { bodies } => now.crowd >= bodies,
            Self::Season { from, to } if from <= to => (from..to).contains(&now.year),
            Self::Season { from, to } => now.year >= from || now.year < to,
        }
    }
}

/// The deposits of `grammar`: each one's account, rate and decay.
pub fn deposits(grammar: &Grammar) -> Vec<(&Key, u64, u64)> {
    let found = grammar.rules.iter();
    found.filter_map(|r| if let Rule::Deposit { account, rate, decay } = r { Some((account, *rate, *decay)) } else { None }).collect()
}

/// The switches of `grammar`: each one's index among its rules, with its
/// condition.
pub fn switches(grammar: &Grammar) -> Vec<(usize, Condition)> {
    let found = grammar.rules.iter().enumerate();
    found.filter_map(|(i, r)| if let Rule::Switch { when, .. } = r { Some((i, *when)) } else { None }).collect()
}

/// The switches of `grammar` that hold of `now`, by index among its rules.
pub fn switched_on(grammar: &Grammar, now: &Situation) -> Vec<usize> {
    switches(grammar).into_iter().filter(|(_, when)| when.holds(now)).map(|(i, _)| i).collect()
}

/// The tagmata `grammar` unfolds into: its rules applied in order to a copy
/// of its seed (cases 1202, 1206), every switch off, as stored.
///
/// # Errors
/// Fails if a rule names a tagma out of range, a branch has no `every`, a
/// branch's turn runs along its bearer or past two levels, a radial run has
/// fewer than two arms, a run hardens no segment or is covered where it
/// bears, or the tagmata pass 255.
pub fn unfold(grammar: &Grammar) -> Result<Vec<Tagma>> {
    unfold_on(grammar, &[])
}

/// The tagmata `grammar` unfolds into while the switches at `on` (indices
/// among its rules) hold: their rules apply where they stand in the order.
///
/// # Errors
/// As [`unfold`], and where a switch holds a switch.
pub fn unfold_on(grammar: &Grammar, on: &[usize]) -> Result<Vec<Tagma>> {
    let mut t = grammar.seed.clone();
    if t.len() > MOST {
        return Err(format!("a grammar unfolds to more than {MOST} tagmata"));
    }
    for (i, rule) in grammar.rules.iter().enumerate() {
        match rule {
            Rule::Switch { rules, .. } => {
                if rules.iter().any(|r| matches!(r, Rule::Switch { .. } | Rule::Deposit { .. })) {
                    return Err("a switch that holds a switch or a deposit".into());
                }
                if on.contains(&i) {
                    for rule in rules {
                        apply(&mut t, rule)?;
                    }
                }
            },
            rule => apply(&mut t, rule)?,
        }
    }
    Ok(t)
}

/// Applies one rule that is no switch.
fn apply(t: &mut Vec<Tagma>, rule: &Rule) -> Result<()> {
    match rule {
        Rule::Switch { .. } => return Err("a switch that holds a switch".into()),
        Rule::Repeat { tagma, by } => {
            let run = named(t, *tagma)?;
            run.segments = run.segments.saturating_add(*by);
        },
        Rule::Branch { tagma, every, depth, taper, anchor, facing } => {
            branch(t, [*tagma, *every, *depth, *taper], *anchor, *facing)?;
        },
        Rule::Differentiate { tagma, at, kind } => differentiate(t, *tagma, *at, kind)?,
        Rule::Accrete { tagma, how } => accrete(named(t, *tagma)?, how)?,
        Rule::Symmetry { tagma, symmetry } => {
            if matches!(symmetry, Symmetry::Radial { arms } if *arms < 2) {
                return Err("a radial run of fewer than two arms".into());
            }
            named(t, *tagma)?.symmetry = Some(*symmetry);
        },
        Rule::Radiate { tagma, arms } => radiate(t, *tagma, *arms)?,
        Rule::Deposit { rate, decay, .. } if *rate == 0 || *decay == 0 => {
            return Err("a deposit that pays or decays nothing".into());
        },
        Rule::Deposit { .. } => {},
        Rule::Allocate { total } => allocate(t, usize::from(*total)),
    }
    Ok(())
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
        let (mut rest, mut prev, mut done) = (run.len - every, run.head, every);
        while rest > 0 {
            let n = rest.min(every);
            // What hardened of the run beyond the pieces before stays so.
            let harden = proto.harden.as_ref().and_then(|h| {
                Some(Hardened { oldest: h.oldest.checked_sub(done).filter(|o| *o > 0)?, ..h.clone() })
            });
            let piece = Tagma { segments: n, parent: Some(prev), anchor: Anchor::Tip, harden, ..proto.clone() };
            prev = push(t, piece)?;
            pieces.push((prev, n == every));
            rest -= n;
            done += n;
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
                    harden: None,
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

/// The quarters round the vertical between the arms of a whole-run radial
/// the lattice shows: all of two, three or four, none beyond four.
const QUARTERS: usize = 4;

/// Copies run `tagma` round the vertical, `arms` in all (D45), each at the
/// joint the run joins, turned a quarter or two or three from it with its
/// socket. Two arms are opposite; three take the next two quarters; four
/// and more fill the ring, and a fifth arm is stored, not shown. A run
/// that joins nothing or runs up or down has no ring to be copied round.
fn radiate(t: &mut Vec<Tagma>, tagma: u8, arms: u8) -> Result<()> {
    if arms < 2 {
        return Err("a radial run of fewer than two arms".into());
    }
    let run = named(t, tagma)?.clone();
    if run.parent.is_none() || !run.facing.horizontal() {
        return Err("a run copied round the vertical must join a parent and run level".into());
    }
    let shown = usize::from(arms).min(QUARTERS);
    for arm in 1..shown {
        let places = arm * (QUARTERS / shown);
        let copy = Tagma { facing: run.facing.turned(places), socket: run.socket.turned(places), ..run.clone() };
        push(t, copy)?;
    }
    Ok(())
}

/// Lays `how` down on `run`.
fn accrete(run: &mut Tagma, how: &Accretion) -> Result<()> {
    match how {
        Accretion::Harden { oldest: 0, .. } => return Err("a run that hardens none of its segments".into()),
        Accretion::Harden { oldest, kind } => run.harden = Some(Hardened { oldest: *oldest, kind: kind.clone() }),
        Accretion::Cover { kind, socket } => {
            if run.bears.is_some() {
                return Err("a covering on a run that already bears a kind".into());
            }
            (run.bears, run.per_segment, run.socket) = (Some(kind.clone()), 1, *socket);
        },
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
        harden: run.harden.as_ref().and_then(|h| Some(Hardened { oldest: h.oldest.checked_sub(at_).filter(|o| *o > 0)?, ..h.clone() })),
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
