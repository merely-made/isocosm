// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Computed amounts (ruling 268, X3): a fixed number, or a bounded
//! expression tree over what the act reads, with a draw term keyed by the
//! act. An act resolves every amount before staging anything with it.

use super::Binding;
use crate::{Result, schema::Key};
use serde::{Deserialize, Serialize};

/// The most nodes one expression may hold.
pub const MAX_NODES: usize = 64;
/// The widest draw an expression may take, small enough for a crowd to split
/// its members over exactly (ruling 207).
pub const MAX_DRAW: u64 = 64;

/// An effect's amount. A bare number serializes as it always did.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Amount {
    Fixed(u64),
    Computed(Expr),
}

impl From<u64> for Amount {
    fn from(value: u64) -> Self {
        Self::Fixed(value)
    }
}

impl Amount {
    /// The value an act resolved; a computed amount still standing refuses.
    pub fn resolved(&self) -> Result<u64> {
        match self {
            Self::Fixed(value) => Ok(*value),
            Self::Computed(_) => Err("an amount was left unresolved".into()),
        }
    }
    /// Whether resolving it draws, which makes its process act member by
    /// member.
    pub fn draws(&self) -> bool {
        matches!(self, Self::Computed(e) if e.draws())
    }
    /// Whether a cohort's members all resolve it alike: no draw, and
    /// nothing read but the actor's own state.
    pub fn bulk_safe(&self) -> bool {
        match self {
            Self::Fixed(_) => true,
            Self::Computed(e) => {
                !e.draws()
                    && e.reads()
                        .iter()
                        .all(|r| matches!(r.who(), Binding::Actor | Binding::Part))
            },
        }
    }
    pub fn validate(&self) -> Result<()> {
        match self {
            Self::Fixed(_) => Ok(()),
            Self::Computed(e) => e.validate(),
        }
    }
    /// The amount resolved: an expression's value, never below nothing.
    pub fn resolve(
        &self,
        read: &mut impl FnMut(&Reading) -> Result<i64>,
        draw: &mut impl FnMut(u64, u8) -> Result<u64>,
    ) -> Result<Amount> {
        Ok(match self {
            Self::Fixed(value) => Self::Fixed(*value),
            Self::Computed(e) => Self::Fixed(e.eval(read, draw)?.max(0) as u64),
        })
    }
}

/// A bounded expression in signed integers; every operation saturates.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Expr {
    Const(i64),
    Read(Reading),
    Add(Vec<Expr>),
    Mul(Vec<Expr>),
    /// Floor division; a zero divisor gives zero.
    Div(Box<Expr>, Box<Expr>),
    Min(Vec<Expr>),
    Max(Vec<Expr>),
    Clamp {
        value: Box<Expr>,
        lo: i64,
        hi: i64,
    },
    /// A uniform draw in `0..below`, keyed by the act and its slot, so a
    /// slot read twice in one act reads the same draw.
    Draw {
        below: u64,
        slot: u8,
    },
    /// The floor square root of a value, nothing below nothing; twice over a
    /// product it gives a mass to the three-quarter power.
    Sqrt(Box<Expr>),
    /// One where the first value is at least the second, else zero.
    AtLeast(Box<Expr>, Box<Expr>),
}

/// What an expression reads of the act: an account, or one of X2's native
/// readings of a body's living parts, each the sum over those parts, or of
/// the bound part alone where `who` is the part.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Reading {
    /// An account a binding holds, the site's included.
    Account { who: Binding, key: Key },
    /// The longest half-extent of each part expressing a function, summed:
    /// an actuator's or a sensor's span.
    Span { who: Binding, function: Key },
    /// Every part's voxels, `(2|h| + 1)` along each axis, summed.
    Voxels { who: Binding },
    /// The cells each part holds for a function, summed.
    Cells { who: Binding, function: Key },
    /// Those cells' matter: each part's cells for a function times its cell
    /// mass, summed, such as what a body's glands hold.
    CellMass { who: Binding, function: Key },
    /// What one cell weighs, read of the bound part alone.
    CellWeight { who: Binding },
}

impl Reading {
    /// Whose state it reads.
    pub fn who(&self) -> Binding {
        match self {
            Self::Account { who, .. }
            | Self::Span { who, .. }
            | Self::Voxels { who }
            | Self::Cells { who, .. }
            | Self::CellMass { who, .. }
            | Self::CellWeight { who } => *who,
        }
    }
}

impl Expr {
    /// Ruling 268's easy default: `base + rate × reading`, clamped.
    pub fn linear(base: i64, rate: i64, reading: Reading, lo: i64, hi: i64) -> Self {
        Self::Clamp {
            value: Box::new(Self::Add(vec![
                Self::Const(base),
                Self::Mul(vec![Self::Const(rate), Self::Read(reading)]),
            ])),
            lo,
            hi,
        }
    }
    /// The same with a draw in `0..below` added before the clamp.
    pub fn linear_drawn(
        base: i64,
        rate: i64,
        reading: Reading,
        below: u64,
        lo: i64,
        hi: i64,
    ) -> Self {
        Self::Clamp {
            value: Box::new(Self::Add(vec![
                Self::Const(base),
                Self::Mul(vec![Self::Const(rate), Self::Read(reading)]),
                Self::Draw { below, slot: 0 },
            ])),
            lo,
            hi,
        }
    }

    fn children(&self) -> Vec<&Expr> {
        match self {
            Self::Const(_) | Self::Read(_) | Self::Draw { .. } => vec![],
            Self::Add(v) | Self::Mul(v) | Self::Min(v) | Self::Max(v) => v.iter().collect(),
            Self::Div(a, b) | Self::AtLeast(a, b) => vec![a, b],
            Self::Clamp { value, .. } | Self::Sqrt(value) => vec![value],
        }
    }
    pub fn nodes(&self) -> usize {
        1 + self.children().iter().map(|c| c.nodes()).sum::<usize>()
    }
    pub fn draws(&self) -> bool {
        matches!(self, Self::Draw { .. }) || self.children().iter().any(|c| c.draws())
    }
    pub fn reads(&self) -> Vec<&Reading> {
        match self {
            Self::Read(r) => vec![r],
            _ => self.children().into_iter().flat_map(Expr::reads).collect(),
        }
    }
    pub fn validate(&self) -> Result<()> {
        if self.nodes() > MAX_NODES {
            return Err(format!("an amount holds more than {MAX_NODES} nodes"));
        }
        self.check()
    }
    fn check(&self) -> Result<()> {
        match self {
            Self::Add(v) | Self::Mul(v) | Self::Min(v) | Self::Max(v) if v.is_empty() => {
                Err("an amount's operation has no operands".into())
            },
            Self::Clamp { lo, hi, .. } if lo > hi => Err("an amount clamps below its floor".into()),
            Self::Draw { below, .. } if *below == 0 || *below > MAX_DRAW => {
                Err(format!("an amount draws outside 1..={MAX_DRAW}"))
            },
            _ => self.children().iter().try_for_each(|c| c.check()),
        }
    }
    pub fn eval(
        &self,
        read: &mut impl FnMut(&Reading) -> Result<i64>,
        draw: &mut impl FnMut(u64, u8) -> Result<u64>,
    ) -> Result<i64> {
        Ok(match self {
            Self::Const(c) => *c,
            Self::Read(r) => read(r)?,
            Self::Add(v) => all(v, read, draw)?.into_iter().fold(0, i64::saturating_add),
            Self::Mul(v) => all(v, read, draw)?.into_iter().fold(1, i64::saturating_mul),
            Self::Div(a, b) => floor_div(a.eval(read, draw)?, b.eval(read, draw)?),
            Self::Min(v) => all(v, read, draw)?.into_iter().min().unwrap_or(0),
            Self::Max(v) => all(v, read, draw)?.into_iter().max().unwrap_or(0),
            Self::Clamp { value, lo, hi } => value.eval(read, draw)?.max(*lo).min(*hi),
            Self::Draw { below, slot } => {
                i64::try_from(draw(*below, *slot)?).map_err(|e| e.to_string())?
            },
            Self::Sqrt(value) => value.eval(read, draw)?.max(0).unsigned_abs().isqrt() as i64,
            Self::AtLeast(a, b) => i64::from(a.eval(read, draw)? >= b.eval(read, draw)?),
        })
    }
}

/// Division rounding toward negative infinity, a zero divisor giving zero
/// and the one overflow saturating.
fn floor_div(a: i64, b: i64) -> i64 {
    if b == 0 {
        return 0;
    }
    match a.checked_div(b) {
        None => i64::MAX,
        Some(q) if a % b != 0 && (a < 0) != (b < 0) => q - 1,
        Some(q) => q,
    }
}

fn all(
    v: &[Expr],
    read: &mut impl FnMut(&Reading) -> Result<i64>,
    draw: &mut impl FnMut(u64, u8) -> Result<u64>,
) -> Result<Vec<i64>> {
    v.iter().map(|e| e.eval(read, draw)).collect()
}
