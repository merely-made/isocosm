// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use crate::matter::{Material, Stock};
use crate::places::Soil;

#[test]
fn mixed_stock_converts_only_a_bounded_typed_lot() {
    let mut soil = Soil::seeded(0, 0);
    let column = soil.column_at([0, 0, 0]);
    soil.deposit_stock(column, Stock::from_amounts([11, 4, 6, 8]))
        .unwrap();
    let mut receipts = Vec::new();

    soil.mineralize(9, |column, paid| receipts.push((column, paid)));

    assert_eq!(soil.stock(column).amounts(), [20, 2, 3, 4]);
    assert_eq!(receipts, vec![(column, Stock::from_amounts([0, 2, 3, 4]))]);
    assert_eq!(soil.total_mg(), 29);
}

#[test]
fn untyped_only_and_zero_dose_leave_soil_unchanged() {
    let mut soil = Soil::seeded(0, 17);
    let column = soil.column_at([0, 0, 0]);
    let before = soil.clone();
    soil.mineralize(3, |_, _| panic!("untyped stock cannot mineralize"));
    assert_eq!(soil, before);

    soil.deposit_stock(column, Stock::single(Material::Producer, 1))
        .unwrap();
    let before = soil.clone();
    soil.mineralize(0, |_, _| panic!("zero dose cannot mineralize"));
    assert_eq!(soil, before);
}

#[test]
fn tiny_typed_residue_is_fully_converted_without_exceeding_the_dose() {
    let mut soil = Soil::seeded(0, 0);
    let column = soil.column_at([0, 0, 0]);
    soil.deposit_stock(column, Stock::single(Material::Producer, 1))
        .unwrap();

    soil.mineralize(10, |_, paid| {
        assert_eq!(paid, Stock::single(Material::Producer, 1))
    });

    assert_eq!(soil.stock(column), Stock::single(Material::Untyped, 1));
}

#[test]
fn conversion_reaches_the_scalar_column_ceiling_exactly() {
    let mut soil = Soil::seeded(0, u64::MAX - 2);
    let column = soil.column_at([0, 0, 0]);
    soil.deposit_stock(column, Stock::single(Material::Producer, 2))
        .unwrap();

    soil.mineralize(2, |_, paid| {
        assert_eq!(paid, Stock::single(Material::Producer, 2))
    });

    assert_eq!(
        soil.stock(column),
        Stock::single(Material::Untyped, u64::MAX)
    );
    assert_eq!(soil.total_mg(), u64::MAX);
}
