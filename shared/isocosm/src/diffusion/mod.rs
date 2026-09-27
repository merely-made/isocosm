// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Diffusion over a square grid of columns (ruling 343): Mesocosm's soil
//! transport, ported as a pure kernel with its tests and left unwired until
//! the places family decides whether columns are sites. The caller owns the
//! columns; the kernel keeps neither fields nor history.

use crate::{Result, schema::*};

/// One synchronous pass. Each column sheds `1 / divisor` of every account,
/// floored, split among its west, east, north and south neighbours in that
/// order, the units left over going one each to the first. Columns are
/// `side` by `side`, row by row, and each account moves independently of the
/// others. Invalid input or an overflow leaves every column as it was.
pub fn percolate(columns: &mut [Ledger], side: usize, divisor: u64) -> Result<()> {
    if side == 0 || side.checked_mul(side) != Some(columns.len()) {
        return Err(format!(
            "{} columns do not make a square of side {side}",
            columns.len()
        ));
    }
    if divisor == 0 {
        return Err("a zero divisor".into());
    }
    if side == 1 {
        return Ok(());
    }
    let mut next: Vec<Ledger> = columns
        .iter()
        .map(|c| {
            c.iter()
                .map(|(k, v)| (k.clone(), v - v / divisor))
                .collect()
        })
        .collect();
    for (index, column) in columns.iter().enumerate() {
        let (x, z) = (index % side, index / side);
        let neighbours = [
            x.checked_sub(1).map(|nx| z * side + nx),
            (x + 1 < side).then_some(index + 1),
            z.checked_sub(1).map(|nz| nz * side + x),
            (z + 1 < side).then_some(index + side),
        ];
        let count = neighbours.iter().flatten().count() as u64;
        for (key, held) in column {
            let out = held / divisor;
            let (share, extra) = (out / count, out % count);
            for (rank, &neighbour) in neighbours.iter().flatten().enumerate() {
                let moved = share + u64::from((rank as u64) < extra);
                if moved == 0 {
                    continue;
                }
                let slot = next[neighbour].entry(key.clone()).or_default();
                *slot = slot
                    .checked_add(moved)
                    .ok_or_else(|| format!("column {neighbour} overflows {key}"))?;
            }
        }
    }
    for (column, ledger) in columns.iter_mut().zip(next) {
        *column = ledger;
    }
    Ok(())
}

#[cfg(test)]
mod scalar_reference;
#[cfg(test)]
mod tests;
