// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! TG2g's bounded natural typed-soil loop.

use std::collections::BTreeMap;

use super::*;
use crate::flow::{Account, Conversion, Process, RecordedFlow};
use crate::matter::{Material, Stock};
use crate::organism::Kingdom;
use crate::snapshot;

type Key = (Account, Option<OrganismId>);
type Book = BTreeMap<Key, Stock>;

fn add(book: &mut Book, key: Key, stock: Stock) {
    let prior = book.get(&key).copied().unwrap_or(Stock::EMPTY);
    book.insert(key, prior.checked_add(stock).expect("account stock fits"));
}

fn accounts(world: &World) -> Book {
    let mut book = Book::new();
    add(&mut book, (Account::Soil, None), world.soil.total_stock());
    for organism in &world.organisms {
        add(
            &mut book,
            (Account::Substance, Some(organism.id)),
            organism.phenotype.total_stock().expect("body stock fits"),
        );
        add(
            &mut book,
            (Account::Reserve, Some(organism.id)),
            Stock::single(Material::Untyped, organism.energy_mg),
        );
    }
    book
}

fn key(account: Account, subject: Option<crate::flow::Subject>) -> Key {
    if account.is_body() {
        (
            account,
            Some(subject.expect("body account has a subject").organism),
        )
    } else {
        (account, None)
    }
}

fn expected(mut book: Book, flows: &[RecordedFlow]) -> Book {
    for envelope in flows {
        let flow = envelope.record;
        let composition = flow.composition.expect("matter flow is composed");
        let source = key(flow.source, flow.from);
        let destination = key(flow.destination, flow.to);
        let remaining = book
            .get(&source)
            .copied()
            .unwrap_or(Stock::EMPTY)
            .checked_sub(composition.input)
            .expect("flow input is held by its source");
        book.insert(source, remaining);
        add(&mut book, destination, composition.output);
    }
    book
}

fn reconcile(before: &World, after: &World, flows: &[RecordedFlow]) -> Book {
    assert!(!flows.is_empty(), "the accepted ecology tick emitted flows");
    let mut claimed = expected(accounts(before), flows);
    claimed.retain(|_, stock| *stock != Stock::EMPTY);
    let mut actual = accounts(after);
    actual.retain(|_, stock| *stock != Stock::EMPTY);
    assert_eq!(claimed, actual, "typed account receipt reconciles");
    claimed
}

fn producer_world() -> (World, OrganismId) {
    let mut world = World::new(7, 24);
    let producer = world
        .living()
        .find(|organism| organism.kingdom() == Kingdom::Producer && organism.biomass_mg() > 400)
        .map(|organism| organism.id)
        .expect("the generated enclosure has a producer");
    world.organisms.retain(|organism| organism.id == producer);
    world.controlled = Some(producer);
    world.soil = crate::places::Soil::seeded(world.soil.extent(), 0);
    let organism = world
        .organisms
        .iter_mut()
        .find(|organism| organism.id == producer)
        .expect("the producer remains in the roster");
    // An empty reserve makes upkeep return typed body substance. Those
    // nutrients can feed roots immediately; the paired arm distinguishes
    // that return from the later completion of pending soil.
    organism.energy_mg = 0;
    (world, producer)
}

#[test]
fn pending_typed_soil_mineralizes_then_feeds_on_the_next_tick_and_replays() {
    let (mut world, producer) = producer_world();
    let position = world
        .organisms
        .iter()
        .find(|organism| organism.id == producer)
        .expect("producer exists")
        .position;
    let column = world.soil.column_at(position);
    world
        .soil
        .deposit_stock(column, Stock::from_amounts([0, 4, 4, 0]))
        .expect("pending typed soil fits");
    let mut control = world.clone().with_rules(crate::rules::WorldRules {
        soil_mineralization_mg_per_column_per_tick: 0,
        ..world.rules()
    });
    let mut replay = snapshot::restore_under(
        &snapshot::snapshot(&world).expect("pending soil snapshot encodes"),
        world.admitted(),
    )
    .expect("default typed soil snapshot restores");

    let mut saw_mineralization = false;
    let mut saw_synthesis_after = false;
    let mut saw_body_return = false;
    let mut first_conversion_tick = None;
    for tick in 0..8 {
        let before = world.clone();
        world.apply(Intent::Idle);
        let flows = world.drain_flows();
        reconcile(&before, &world, &flows);
        let replay_outcome = replay.apply(Intent::Idle);
        assert!(matches!(replay_outcome, Outcome::Idled));
        assert_eq!(flows, replay.drain_flows(), "replay preserves flow vectors");
        assert_eq!(world, replay, "replay preserves typed world state");
        let before_control = control.clone();
        control.apply(Intent::Idle);
        let control_flows = control.drain_flows();
        reconcile(&before_control, &control, &control_flows);
        let synthesized_mg = |flows: &[RecordedFlow]| -> u64 {
            flows
                .iter()
                .filter(|flow| {
                    flow.record.composition.is_some_and(|composition| {
                        composition.conversion == Some(Conversion::Synthesis)
                    })
                })
                .map(|flow| flow.record.amount_mg)
                .sum()
        };
        if tick == 0 {
            assert_eq!(synthesized_mg(&flows), synthesized_mg(&control_flows));
            assert_eq!(
                world.organisms, control.organisms,
                "completion cannot alter bodies earlier in the same tick"
            );
        }
        if tick == 1 {
            assert_eq!(
                synthesized_mg(&flows),
                synthesized_mg(&control_flows) + 1,
                "the previous tick's completed milligram feeds roots now"
            );
        }

        let converted = flows.iter().any(|flow| {
            flow.record.process == Process::Decay
                && flow.record.source == Account::Soil
                && flow.record.destination == Account::Soil
                && flow.record.from.is_none()
                && flow.record.to.is_none()
                && flow.record.composition.is_some_and(|composition| {
                    composition.conversion == Some(Conversion::Mineralization)
                        && composition.input.amount(Material::Producer) > 0
                        && composition.output.amount(Material::Untyped) > 0
                })
        });
        let synthesized = flows.iter().any(|flow| {
            flow.record.process == Process::Uptake
                && flow.record.composition.is_some_and(|composition| {
                    composition.conversion == Some(Conversion::Synthesis)
                        && composition.output.amount(Material::Producer) > 0
                })
        });
        saw_mineralization |= converted;
        saw_body_return |= flows.iter().any(|flow| {
            flow.record.process == Process::Upkeep
                && flow.record.source == Account::Substance
                && flow.record.destination == Account::Soil
                && flow.record.composition.is_some_and(|composition| {
                    composition.conversion == Some(Conversion::Mineralization)
                        && composition.input.amount(Material::Producer) > 0
                })
        });
        if converted {
            first_conversion_tick.get_or_insert(tick);
        }
        if synthesized {
            saw_synthesis_after |=
                first_conversion_tick.is_some_and(|conversion| tick > conversion);
        }
    }
    assert_eq!(first_conversion_tick, Some(0));
    assert!(saw_mineralization);
    assert!(
        saw_synthesis_after,
        "newly mineralized stock waits one tick"
    );
    assert!(
        saw_body_return,
        "typed body upkeep returns through mineralization"
    );
}

#[test]
fn zero_budget_is_a_paired_control_and_scalar_equal_channel_swap_is_rejected() {
    let (mut default, producer) = producer_world();
    let position = default
        .organisms
        .iter()
        .find(|organism| organism.id == producer)
        .expect("producer exists")
        .position;
    let column = default.soil.column_at(position);
    default
        .soil
        .deposit_stock(column, Stock::from_amounts([0, 3, 3, 0]))
        .unwrap();
    let mut control = default.clone().with_rules(crate::rules::WorldRules {
        soil_mineralization_mg_per_column_per_tick: 0,
        ..default.rules()
    });

    let before_default = default.clone();
    default.apply(Intent::Idle);
    let default_flows = default.drain_flows();
    let default_claimed = reconcile(&before_default, &default, &default_flows);
    let before_control = control.clone();
    control.apply(Intent::Idle);
    let control_flows = control.drain_flows();
    reconcile(&before_control, &control, &control_flows);
    assert!(
        default_flows
            .iter()
            .any(|flow| flow.record.process == Process::Decay
                && flow.record.source == Account::Soil
                && flow.record.destination == Account::Soil),
        "default budget converts pending soil"
    );
    assert!(!control_flows.iter().any(|flow| {
        flow.record.process == Process::Decay
            && flow.record.source == Account::Soil
            && flow.record.destination == Account::Soil
    }));
    assert_eq!(
        default.total_matter_mg(),
        control.total_matter_mg(),
        "paired controls conserve the same scalar total"
    );

    let soil = default.soil.stock(column);
    assert!(soil.amount(Material::Producer) > 0);
    let held = default
        .soil
        .draw_stock(column, u64::try_from(soil.total()).unwrap());
    default
        .soil
        .deposit_stock(
            column,
            Stock::from_amounts([
                held.amount(Material::Untyped),
                held.amount(Material::Producer) - 1,
                held.amount(Material::Consumer) + 1,
                held.amount(Material::Decomposer),
            ]),
        )
        .unwrap();
    let broken = accounts(&default);
    assert_eq!(
        broken.values().copied().map(Stock::total).sum::<u128>(),
        default_claimed.values().copied().map(Stock::total).sum()
    );
    assert_ne!(
        broken, default_claimed,
        "same scalar with a changed channel is rejected"
    );
}
