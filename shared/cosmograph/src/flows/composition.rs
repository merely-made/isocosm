// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! What a move carried, by material, and the conversion it made, if any.
//! Absence means the maker reports scalar amounts, not untyped matter.

use crate::matter::{Material, Stock};
use serde::{Deserialize, Serialize};

pub use crate::matter::receipt::Conversion;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Composition {
    pub input: Stock,
    pub output: Stock,
    pub conversion: Option<Conversion>,
}

impl Composition {
    /// Untyped matter in and out, unconverted.
    pub fn untyped(amount: u64) -> Self {
        Self::carried(Stock::single(Material::Untyped, amount))
    }

    /// A stock carried unchanged.
    pub fn carried(stock: Stock) -> Self {
        Self {
            input: stock,
            output: stock,
            conversion: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::matter::receipt::{Address, Book, Receipt, reconcile};
    use crate::schema::PartId;

    #[test]
    fn mixed_reserve_conversion_reconciles_with_the_typed_book() {
        let source = Address::Part(1, PartId(0));
        let target = Address::Reserve(2);
        let input = Stock::from_amounts([3, 5, 7, 11]);
        let output = Stock::single(Material::Untyped, 26);
        let before = Book::from([(source, input)]);
        let after = Book::from([(source, Stock::EMPTY), (target, output)]);
        let receipt = Receipt::Conversion {
            kind: Conversion::Digestion,
            from: source,
            to: target,
            input,
            output,
        };
        assert_eq!(reconcile(&before, &after, &[receipt]), Ok(()));
    }

    #[test]
    fn synthesis_reconciles_untyped_soil_as_producer_tissue() {
        let soil = Address::Soil([0, 0, 0]);
        let tissue = Address::Part(2, PartId(0));
        let before = Book::from([(soil, Stock::single(Material::Untyped, 26))]);
        let after = Book::from([
            (soil, Stock::EMPTY),
            (tissue, Stock::single(Material::Producer, 26)),
        ]);
        let receipt = Receipt::Conversion {
            kind: Conversion::Synthesis,
            from: soil,
            to: tissue,
            input: Stock::single(Material::Untyped, 26),
            output: Stock::single(Material::Producer, 26),
        };
        assert_eq!(reconcile(&before, &after, &[receipt]), Ok(()));
    }
}
