// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{
    flow::{Account, Conversion},
    matter::{Material, Stock},
    snapshot,
};

#[test]
fn mixed_soil_reconciles_completed_returns_and_snapshot_replay_per_channel() {
    let mut world = World::new(41, 60);
    let column = world.soil.column_at([0, 0, 0]);
    // Test-only pending material; every completed portion must name its input
    // and output before roots can spend it as nutrients.
    world
        .soil
        .deposit_stock(column, Stock::from_amounts([23, 101, 103, 107]))
        .unwrap();
    let initial = world.total_matter_mg();
    let mut completed = 0;
    let mut resumed = None;
    for tick in 0..120 {
        let before = world.soil.total_stock();
        world.apply(Intent::Idle);
        let mut expected = before.amounts().map(i128::from);
        let flows = world.drain_flows();
        for flow in &flows {
            let event = flow.record;
            let composition = event.composition.expect("every material flow is typed");
            if event.source == Account::Soil {
                for material in Material::ALL {
                    expected[material.index()] -= i128::from(composition.input.amount(material));
                }
            }
            if event.destination == Account::Soil {
                for material in Material::ALL {
                    expected[material.index()] += i128::from(composition.output.amount(material));
                }
            }
            if event.source == Account::Soil && event.destination == Account::Soil {
                assert_eq!(composition.conversion, Some(Conversion::Mineralization));
                assert_eq!((event.from, event.to), (None, None));
                assert_eq!(composition.input.amount(Material::Untyped), 0);
                assert_eq!(
                    composition.output,
                    Stock::single(Material::Untyped, event.amount_mg)
                );
                completed += event.amount_mg;
            }
        }
        let after = world.soil.total_stock();
        assert_eq!(after.amounts().map(i128::from), expected);
        assert_eq!(world.total_matter_mg(), initial);
        if let Some(other) = &mut resumed {
            World::apply(other, Intent::Idle);
            assert_eq!(other.drain_flows(), flows);
            assert_eq!(snapshot::state_hash(&world), snapshot::state_hash(other));
        }
        if tick == 59 {
            resumed = Some(
                snapshot::restore_under(&snapshot::snapshot(&world).unwrap(), world.admitted())
                    .unwrap(),
            );
        }
    }
    assert!(
        completed > 0,
        "the pending typed deposit actually completes"
    );
}

#[test]
fn soil_composition_changes_world_hash_even_when_total_mass_matches() {
    let mut producer = World::new(3, 0);
    let mut consumer = producer.clone();
    let column = producer.soil.column_at([0, 0, 0]);
    producer
        .soil
        .deposit_stock(column, Stock::single(Material::Producer, 7))
        .unwrap();
    consumer
        .soil
        .deposit_stock(column, Stock::single(Material::Consumer, 7))
        .unwrap();
    assert_eq!(producer.total_matter_mg(), consumer.total_matter_mg());
    assert_ne!(
        snapshot::state_hash(&producer),
        snapshot::state_hash(&consumer)
    );
}

#[test]
fn pre_typed_soil_rules_are_refused_at_snapshot_admission() {
    let mut world = World::new(3, 0);
    world.rules.trophic_grammar = 1;
    assert!(matches!(
        snapshot::restore_under(&snapshot::snapshot(&world).unwrap(), world.admitted()),
        Err(snapshot::SnapshotError::Rules { .. })
    ));
}

#[test]
fn pre_completion_grammar_and_different_soil_rates_are_refused() {
    let native = World::new(3, 0);
    let mut old = native.clone();
    old.rules.trophic_grammar = 6;
    let mut other_rate = native.clone();
    other_rate.rules.soil_mineralization_mg_per_column_per_tick = 0;
    for world in [old, other_rate] {
        assert!(matches!(
            snapshot::restore_under(&snapshot::snapshot(&world).unwrap(), native.admitted()),
            Err(snapshot::SnapshotError::Rules { .. })
        ));
    }
}
