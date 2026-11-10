// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! What each kingdom pays for a function's cells (case 1232): a kingdom
//! makes a function cheap or dear and gates none. The weights price what
//! growth and the draft charge; they never decide what a body is or holds.

use super::Rules;
use crate::schema::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// What each kingdom pays for a function's cells (case 1232): a share,
/// numerator over denominator, of the ordinary price. A function a kingdom
/// does not name costs the ordinary price; none is refused.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Kingdoms {
    pub weights: BTreeMap<Key, BTreeMap<Key, [u32; 2]>>,
}

impl Default for Kingdoms {
    /// A first cut, each line to tune in play.
    fn default() -> Self {
        let line = |kingdom: &str, weights: &[(&str, [u32; 2])]| {
            let named = weights.iter().map(|(f, w)| (format!("function:{f}"), *w));
            (kingdom.to_string(), named.collect())
        };
        Kingdoms {
            weights: BTreeMap::from([
                // First cut: plants fix and store cheaply, contract dearly.
                line("kingdom:flora", &[("fix", [1, 2]), ("store", [1, 2]), ("contract", [2, 1])]),
                // First cut: animals contract and sense cheaply, fix dearly.
                line("kingdom:fauna", &[("contract", [1, 2]), ("sense", [1, 2]), ("fix", [2, 1])]),
                // First cut: fungi secrete and conduct cheaply.
                line("kingdom:myco", &[("secrete", [1, 2]), ("conduct", [1, 2])]),
            ]),
        }
    }
}

impl Kingdoms {
    /// Whether every weight has a denominator and names a function the
    /// world's catalogue holds (case 1232).
    ///
    /// # Errors
    /// Fails on a zero denominator or a function outside the catalogue.
    pub fn validate(&self, rules: &Rules) -> crate::Result<()> {
        for (kingdom, named) in &self.weights {
            for (function, [_, of]) in named {
                if *of == 0 {
                    return Err(format!("{kingdom} weighs {function} over nought"));
                }
                if !rules.functions.contains_key(function) {
                    return Err(format!("{kingdom} weighs {function}, outside the catalogue"));
                }
            }
        }
        Ok(())
    }
}

/// Fixed-point scale a function's share is read at.
const SCALE: u128 = 1 << 16;

/// `price` scaled by the mean of the weights `kingdom` pays for `part`'s
/// functions, each by its share of the part's cells (case 1232); a function
/// unweighed counts at one, a part with no cells is unchanged, and so is
/// the price where no rule is set or the kingdom is unknown. Rounds to the
/// nearest, ties down.
pub fn weighed(rules: &Rules, kingdom: &str, part: &Part, price: u128) -> u128 {
    let Some(named) = rules.kingdoms.as_ref().and_then(|k| k.weights.get(kingdom)) else {
        return price;
    };
    let cells: u128 = part.cells.values().map(|n| u128::from(*n)).sum();
    if cells == 0 {
        return price;
    }
    let scaled: u128 = part
        .cells
        .iter()
        .map(|(f, n)| {
            let [top, of] = named.get(f).copied().unwrap_or([1, 1]);
            u128::from(*n) * u128::from(top) * SCALE / u128::from(of.max(1))
        })
        .sum();
    let (top, of) = (price.saturating_mul(scaled), cells * SCALE);
    top.saturating_mul(2).saturating_add(of - 1) / (2 * of)
}

#[cfg(test)]
mod tests;
