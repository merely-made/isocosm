// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Bounded conversion of retained soil nis into root-accessible stock.

use super::{Column, Soil};
use crate::matter::{Material, Stock};

impl Soil {
    /// Converts at most `dose_mg` of retained nis in each column to untyped
    /// stock, visiting columns in their canonical storage order.
    ///
    /// The callback receives the converted lot and its exact column. It exists
    /// for the caller-owned flow receipt; this store keeps no separate material
    /// authority or per-lot age.
    pub fn mineralize(&mut self, dose_mg: u64, mut converted: impl FnMut(Column, Stock)) {
        if dose_mg == 0 {
            return;
        }

        for (index, held) in self.matter_mg.iter_mut().enumerate() {
            let typed =
                Stock::from_amounts([0, held.amounts()[1], held.amounts()[2], held.amounts()[3]]);
            let (paid, remainder) = typed.take(dose_mg);
            if paid == Stock::EMPTY {
                continue;
            }

            let output = Stock::single(
                Material::Untyped,
                u64::try_from(paid.total()).expect("one conversion stays scalar-bounded"),
            );
            *held = held
                .checked_sub(paid)
                .expect("a paid soil lot came from its column")
                .checked_add(output)
                .expect("conversion replaces paid material in the same column");
            debug_assert_eq!(
                held.amounts()[1..],
                remainder.amounts()[1..],
                "only the paid typed lot may change provenance"
            );

            let column = Column(u32::try_from(index).expect("the finite store fits Column"));
            converted(column, paid);
        }
    }
}

#[cfg(test)]
mod tests;
