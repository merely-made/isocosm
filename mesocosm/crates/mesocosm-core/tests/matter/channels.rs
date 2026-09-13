// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Per-channel companion to the scalar long-run conservation gate.

use std::collections::BTreeMap;

use mesocosm_core::{
    OrganismId, World,
    flow::{Account, Conversion, RecordedFlow, Subject},
    matter::{Material, Stock},
};

pub(super) type Key = (Account, Option<OrganismId>);
pub(super) type Book = BTreeMap<Key, Stock>;

fn add(book: &mut Book, key: Key, stock: Stock) -> Result<(), String> {
    let held = book.get(&key).copied().unwrap_or(Stock::EMPTY);
    let next = held
        .checked_add(stock)
        .map_err(|error| format!("account {key:?} overflowed: {error:?}"))?;
    book.insert(key, next);
    Ok(())
}

/// The actual state, without a second model: soil plus every body's two accounts.
pub(super) fn books(world: &World) -> Book {
    let mut book = Book::new();
    add(&mut book, (Account::Soil, None), world.soil().total_stock()).expect("finite soil");
    for organism in &world.organisms {
        add(
            &mut book,
            (Account::Substance, Some(organism.id)),
            organism.phenotype.total_stock().expect("valid body stock"),
        )
        .expect("finite body stock");
        add(
            &mut book,
            (Account::Reserve, Some(organism.id)),
            Stock::single(Material::Untyped, organism.energy_mg),
        )
        .expect("finite reserve");
    }
    compact(&mut book);
    book
}

fn side(account: Account, subject: Option<Subject>) -> Result<Key, String> {
    if account.is_body() {
        subject
            .map(|subject| (account, Some(subject.organism)))
            .ok_or_else(|| format!("body account {account:?} omitted its subject"))
    } else {
        Ok((account, None))
    }
}

fn validate(flow: &RecordedFlow) -> Result<(), String> {
    let event = flow.record;
    let composition = event
        .composition
        .ok_or_else(|| format!("uncomposed live flow: {event:?}"))?;
    if composition.input.total() != u128::from(event.amount_mg)
        || composition.output.total() != u128::from(event.amount_mg)
    {
        return Err(format!(
            "flow amount disagrees with its composition: {event:?}"
        ));
    }
    match composition.conversion {
        None if composition.input != composition.output => {
            Err(format!("ordinary transfer changed its stock: {event:?}"))
        },
        Some(Conversion::Synthesis)
            if composition.input.amount(Material::Untyped) == event.amount_mg
                && composition.output.amount(Material::Producer) == event.amount_mg =>
        {
            Ok(())
        },
        Some(Conversion::Digestion | Conversion::Mineralization)
            if composition.output == Stock::single(Material::Untyped, event.amount_mg) =>
        {
            Ok(())
        },
        None => Ok(()),
        _ => Err(format!("invalid conversion vectors: {event:?}")),
    }
}

fn replay(mut before: Book, flows: &[RecordedFlow]) -> Result<Book, String> {
    for flow in flows {
        validate(flow)?;
        let event = flow.record;
        let composition = event.composition.expect("validated");
        let source = side(event.source, event.from)?;
        let destination = side(event.destination, event.to)?;
        let held = before.get(&source).copied().unwrap_or(Stock::EMPTY);
        let remainder = held
            .checked_sub(composition.input)
            .map_err(|_| format!("flow took unavailable stock from {source:?}: {event:?}"))?;
        before.insert(source, remainder);
        add(&mut before, destination, composition.output)?;
    }
    compact(&mut before);
    Ok(before)
}

pub(super) fn matches_actual(expected: &Book, actual: &Book) -> Result<(), String> {
    if expected == actual {
        Ok(())
    } else {
        Err(format!(
            "typed accounts differ: expected {expected:?}, actual {actual:?}"
        ))
    }
}

pub(super) fn reconciles(
    before: &Book,
    after: &World,
    flows: &[RecordedFlow],
) -> Result<(), String> {
    let expected = replay(before.clone(), flows)?;
    matches_actual(&expected, &books(after))
}

fn compact(book: &mut Book) {
    book.retain(|_, stock| *stock != Stock::EMPTY);
}

pub(super) fn swap_one_soil_kind(book: &mut Book) {
    let soil = book.get_mut(&(Account::Soil, None)).expect("soil account");
    let amounts = soil.amounts();
    assert!(amounts[0] > 0, "founding soil holds nutrients");
    *soil = Stock::from_amounts([amounts[0] - 1, amounts[1] + 1, amounts[2], amounts[3]]);
}
