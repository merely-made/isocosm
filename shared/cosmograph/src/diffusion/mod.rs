// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Diffusion over a square grid of columns (ruling 343): Mesocosm's soil
//! transport as a pure kernel, over any column that holds accounts: a
//! ledger's open keys, or a stock's four material channels, which Mesocosm's
//! soil runs on. The caller owns the columns; the kernel keeps neither
//! fields nor history.

use crate::schema::*;
use std::fmt;

/// A column the kernel moves: accounts it holds, each moved alone.
pub trait Holding: Sized {
    type Account;
    /// Each account and what it holds, in a fixed order.
    fn held(&self) -> impl Iterator<Item = (Self::Account, u64)> + '_;
    /// This column less `1 / divisor` of every account, floored.
    fn kept(&self, divisor: u64) -> Self;
    /// Adds `amount` to `account`; false on overflow.
    fn gain(&mut self, account: &Self::Account, amount: u64) -> bool;
}

impl Holding for Ledger {
    type Account = Key;
    fn held(&self) -> impl Iterator<Item = (Key, u64)> + '_ {
        self.iter().map(|(k, v)| (k.clone(), *v))
    }
    fn kept(&self, divisor: u64) -> Self {
        self.iter()
            .map(|(k, v)| (k.clone(), v - v / divisor))
            .collect()
    }
    fn gain(&mut self, account: &Key, amount: u64) -> bool {
        let slot = self.entry(account.clone()).or_default();
        slot.checked_add(amount).map(|v| *slot = v).is_some()
    }
}

/// Why a pass was refused. Every column is left as it was.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal<A> {
    Shape { columns: usize, side: usize },
    ZeroDivisor,
    Overflow { column: usize, account: A },
}

impl<A: fmt::Display> fmt::Display for Refusal<A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Shape { columns, side } => {
                write!(f, "{columns} columns do not make a square of side {side}")
            },
            Self::ZeroDivisor => write!(f, "a zero divisor"),
            Self::Overflow { column, account } => write!(f, "column {column} overflows {account}"),
        }
    }
}

/// One synchronous pass. Each column sheds `1 / divisor` of every account,
/// floored, split among its west, east, north and south neighbours in that
/// order, the units left over going one each to the first. Columns are
/// `side` by `side`, row by row, and each account moves independently of the
/// others. Invalid input or an overflow leaves every column as it was.
pub fn percolate<C: Holding>(
    columns: &mut [C],
    side: usize,
    divisor: u64,
) -> Result<(), Refusal<C::Account>> {
    if side == 0 || side.checked_mul(side) != Some(columns.len()) {
        let columns = columns.len();
        return Err(Refusal::Shape { columns, side });
    }
    if divisor == 0 {
        return Err(Refusal::ZeroDivisor);
    }
    if side == 1 {
        return Ok(());
    }
    let mut next: Vec<C> = columns.iter().map(|c| c.kept(divisor)).collect();
    for (index, column) in columns.iter().enumerate() {
        let (x, z) = (index % side, index / side);
        let neighbours = [
            x.checked_sub(1).map(|nx| z * side + nx),
            (x + 1 < side).then_some(index + 1),
            z.checked_sub(1).map(|nz| nz * side + x),
            (z + 1 < side).then_some(index + side),
        ];
        let count = neighbours.iter().flatten().count() as u64;
        for (account, held) in column.held() {
            let out = held / divisor;
            let (share, extra) = (out / count, out % count);
            for (rank, &neighbour) in neighbours.iter().flatten().enumerate() {
                let moved = share + u64::from((rank as u64) < extra);
                if moved != 0 && !next[neighbour].gain(&account, moved) {
                    return Err(Refusal::Overflow {
                        column: neighbour,
                        account,
                    });
                }
            }
        }
    }
    for (column, kept) in columns.iter_mut().zip(next) {
        *column = kept;
    }
    Ok(())
}

#[cfg(test)]
mod scalar_reference;
#[cfg(test)]
mod tests;
