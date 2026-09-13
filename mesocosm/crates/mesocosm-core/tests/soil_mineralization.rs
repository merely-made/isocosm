// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! TG2g's soil-side acceptance seam.
//!
//! The world test owns tick ordering and natural bodies. This file keeps the
//! public soil contract small and observable: one bounded dose per column,
//! typed stock only, canonical order, and an exact subjectless flow shape.

use mesocosm_core::flow::{Account, Carrier, Conversion, FlowEvent, Process};
use mesocosm_core::matter::{Material, Stock};
use mesocosm_core::places::Soil;

#[test]
fn mineralization_completes_typed_stock_in_column_order_and_preserves_scalar_mass() {
    let mut soil = Soil::seeded(1, 10);
    let first = [-1, 0, -1];
    let second = [0, 0, -1];
    let first_column = soil.column_at(first);
    let second_column = soil.column_at(second);
    soil.deposit_stock(first_column, Stock::from_amounts([0, 4, 3, 0]))
        .expect("first typed deposit fits");
    soil.deposit_stock(second_column, Stock::from_amounts([0, 0, 0, 5]))
        .expect("second typed deposit fits");
    let before_total = soil.total_mg();
    let before_untyped = soil.total_stock().amount(Material::Untyped);
    let mut converted = Vec::new();

    soil.mineralize(2, |column, paid| converted.push((column, paid)));

    assert_eq!(
        soil.total_mg(),
        before_total,
        "conversion changes kind only"
    );
    assert_eq!(
        soil.total_stock().amount(Material::Untyped),
        before_untyped + 4,
        "two typed milligrams complete in each of the two populated columns"
    );
    assert_eq!(converted.len(), 2);
    assert_eq!(
        converted[0],
        (first_column, Stock::from_amounts([0, 1, 1, 0]))
    );
    assert_eq!(
        converted[1],
        (second_column, Stock::from_amounts([0, 0, 0, 2]))
    );
    assert_eq!(soil.stock(first_column), Stock::from_amounts([12, 3, 2, 0]));
    assert_eq!(
        soil.stock(second_column),
        Stock::from_amounts([12, 0, 0, 3])
    );
}

#[test]
fn zero_budget_leaves_pending_typed_soil_untouched() {
    let mut soil = Soil::seeded(0, 10);
    let column = soil.column_at([0, 0, 0]);
    soil.deposit_stock(column, Stock::from_amounts([0, 4, 0, 3]))
        .expect("typed deposit fits");
    let before = soil.clone();
    let mut callbacks = 0;

    soil.mineralize(0, |_, _| callbacks += 1);

    assert_eq!(callbacks, 0);
    assert_eq!(soil, before);
}

#[test]
fn mineralization_receipt_is_subjectless_soil_to_soil_decay_with_exact_vectors() {
    let input = Stock::from_amounts([0, 2, 0, 1]);
    let flow = FlowEvent {
        process: Process::Decay,
        carrier: Carrier::Matter,
        source: Account::Soil,
        destination: Account::Soil,
        amount_mg: 3,
        composition: None,
        from: None,
        to: None,
    }
    .mineralized(input);
    let composition = flow.composition.expect("mineralization is composed");
    assert_eq!(flow.from, None);
    assert_eq!(flow.to, None);
    assert_eq!(flow.source, Account::Soil);
    assert_eq!(flow.destination, Account::Soil);
    assert_eq!(composition.input, input);
    assert_eq!(composition.output, Stock::single(Material::Untyped, 3));
    assert_eq!(composition.conversion, Some(Conversion::Mineralization));

    let untyped = FlowEvent::soil_mineralization(Stock::single(Material::Untyped, 3));
    assert_eq!(untyped.composition.unwrap().conversion, None);
}
