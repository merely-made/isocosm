// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! TG3a's world-transaction acceptance for bounded disfavoured carries.

use crate::body::Origin;
use crate::discovery::{Evidence, Stress, conditions};
use crate::graft::compatibility::{Compatibility, ConditionAllowance};
use crate::graft::{Crossing, Domain, Verdict};
use crate::matter::Stock;
use crate::rules::WorldRules;
use crate::{OrganismId, Outcome, Rejection, SpeciesId, state_hash};

use super::super::World;
use super::fixture;

fn disfavoured(world: &mut World, donor: SpeciesId) {
    let recipient = world.controlled().expect("embodied").species;
    let lineages = world.lineages_mut();
    lineages.found(donor);
    lineages.set_domain(donor, Domain(2));
    lineages.set_domain(recipient, Domain(1));
    assert_eq!(world.verdict_between(donor, recipient), Verdict::Refused);
}

fn with_allowance(world: World, allowance_mg: u64) -> World {
    world.with_rules(WorldRules {
        graft_compatibility: Compatibility {
            allowance_cells: 0,
            allowance_mg,
            penalty_per_mg: 1,
            raised_by: [None; 2],
        },
        ..WorldRules::native()
    })
}

fn donor_branch_mass(world: &World, donor: OrganismId, part: crate::PartId) -> u64 {
    world
        .organisms
        .iter()
        .find(|organism| organism.id == donor)
        .expect("donor exists")
        .phenotype
        .harvest(part)
        .expect("branch remains whole")
        .mass_mg()
}

fn mixed_branch_stock(
    world: &mut World,
    donor: OrganismId,
    root: crate::PartId,
    tip: crate::PartId,
) -> Stock {
    let root_stock = Stock::from_amounts([0, 100, 150, 150]);
    let tip_stock = Stock::from_amounts([0, 50, 50, 50]);
    let phenotype = &mut world
        .organisms
        .iter_mut()
        .find(|organism| organism.id == donor)
        .expect("donor exists")
        .phenotype;
    phenotype
        .replace_part_stock(root, root_stock)
        .expect("root fits its mass");
    phenotype
        .replace_part_stock(tip, tip_stock)
        .expect("tip fits its mass");
    root_stock
        .checked_add(tip_stock)
        .expect("fixture stock fits")
}

fn assert_arrived_stock(world: &World, root: crate::PartId, parts: &[crate::PartId]) {
    let body = &world.controlled().expect("embodied").phenotype;
    assert_eq!(
        body.part_stock(root),
        Some(&Stock::from_amounts([0, 100, 150, 150]))
    );
    assert_eq!(
        body.part_stock(parts[1]),
        Some(&Stock::from_amounts([0, 50, 50, 50]))
    );
}

#[test]
fn disfavoured_carry_publishes_the_preview_receipt_and_prices_the_reserve() {
    let (world, donor, part, tip) = fixture();
    let incoming_mg = donor_branch_mass(&world, donor, part);
    let mut world = with_allowance(world, incoming_mg);
    let donor_line = SpeciesId(9_700);
    disfavoured(&mut world, donor_line);
    let transferred_stock = mixed_branch_stock(&mut world, donor, part, tip);
    let recipient = world.controlled_id().expect("embodied");
    let reserve_before = world.controlled().expect("embodied").energy_mg;
    let soil_before = world.soil().total_mg();

    let preview = world
        .preview_graft(donor, part, Crossing::Carry)
        .expect("bounded disfavoured carry is feasible");
    let receipt = preview.compatibility.clone().expect("priced receipt");
    assert_eq!(
        (
            receipt.incoming_mg,
            receipt.retained_mg,
            receipt.requested_mg
        ),
        (incoming_mg, 0, incoming_mg)
    );
    assert_eq!(receipt.base_allowance_mg, incoming_mg);
    assert_eq!(receipt.effective_allowance_mg, incoming_mg);
    assert_eq!(receipt.penalty_mg, incoming_mg);
    assert!(receipt.applied.is_empty());
    assert_eq!(preview.cost_mg, incoming_mg);

    world.drain_flows();
    assert!(matches!(
        world.graft(donor, part, Crossing::Carry),
        Outcome::Grafted {
            verdict: Verdict::Refused,
            ..
        }
    ));
    let graft = world.last_graft().expect("published");
    assert_eq!(graft.compatibility, preview.compatibility);
    assert_eq!(graft.compatibility.as_ref(), Some(&receipt));
    assert_eq!(world.controlled_id(), Some(recipient));
    assert_eq!(
        reserve_before - world.controlled().unwrap().energy_mg,
        incoming_mg
    );
    assert_eq!(world.soil().total_mg() - soil_before, incoming_mg);
    assert_arrived_stock(&world, graft.root, &graft.parts);
    let flows = world.drain_flows();
    let transfer = flows
        .iter()
        .find(|flow| flow.record.process == crate::flow::Process::Graft)
        .expect("one exact donor-to-recipient transfer");
    assert_eq!(
        transfer.record.composition.unwrap().input,
        transferred_stock
    );
    assert_eq!(
        transfer.record.composition.unwrap().output,
        transferred_stock
    );
    assert!(flows.iter().any(|flow| {
        flow.record.process == crate::flow::Process::Develop
            && flow.record.source == crate::flow::Account::Reserve
            && flow.record.destination == crate::flow::Account::Soil
            && flow.record.amount_mg == incoming_mg
    }));

    let bytes = crate::snapshot::snapshot(&world).expect("snapshot");
    let restored = crate::snapshot::restore(&bytes).expect("round trip");
    assert_eq!(restored.last_graft(), world.last_graft());
    assert_eq!(state_hash(&restored), state_hash(&world));
}

#[test]
fn accepted_carry_replays_from_before_and_after_the_graft_snapshot() {
    let (world, donor, part, tip) = fixture();
    let incoming_mg = donor_branch_mass(&world, donor, part);
    let mut world = with_allowance(world, incoming_mg);
    disfavoured(&mut world, SpeciesId(9_700));
    mixed_branch_stock(&mut world, donor, part, tip);
    let before = crate::snapshot::snapshot(&world).expect("pre-graft snapshot");
    let mut left = crate::snapshot::restore(&before).expect("left restores");
    let mut right = crate::snapshot::restore(&before).expect("right restores");
    let carry = crate::Intent::Graft {
        organism: donor,
        part,
        crossing: Crossing::Carry,
    };

    assert_eq!(left.apply(carry.clone()), right.apply(carry));
    let graft = left.last_graft().expect("accepted carry recorded").clone();
    assert_eq!(
        graft.compatibility,
        right.last_graft().unwrap().compatibility
    );
    assert_arrived_stock(&left, graft.root, &graft.parts);
    assert_arrived_stock(&right, graft.root, &graft.parts);
    assert_eq!(state_hash(&left), state_hash(&right));

    let after = crate::snapshot::snapshot(&left).expect("post-graft snapshot");
    let mut resumed = crate::snapshot::restore(&after).expect("resumed restores");
    for _ in 0..2 {
        assert_eq!(
            left.apply(crate::Intent::Idle),
            resumed.apply(crate::Intent::Idle)
        );
        assert_eq!(left.last_graft(), resumed.last_graft());
        assert_arrived_stock(&left, graft.root, &graft.parts);
        assert_arrived_stock(&resumed, graft.root, &graft.parts);
        assert_eq!(state_hash(&left), state_hash(&resumed));
    }
}

#[test]
fn at_cap_succeeds_and_over_cap_direct_transaction_leaves_the_world_unchanged() {
    let (world, donor, part, _) = fixture();
    let incoming_mg = donor_branch_mass(&world, donor, part);
    let mut at_cap = with_allowance(world, incoming_mg);
    disfavoured(&mut at_cap, SpeciesId(9_700));
    assert!(matches!(
        at_cap.graft(donor, part, Crossing::Carry),
        Outcome::Grafted { .. }
    ));

    let (world, donor, part, _) = fixture();
    let incoming_mg = donor_branch_mass(&world, donor, part);
    let mut over_cap = with_allowance(world, incoming_mg - 1);
    disfavoured(&mut over_cap, SpeciesId(9_700));
    let before = state_hash(&over_cap);
    assert_eq!(
        over_cap.graft(donor, part, Crossing::Carry),
        Outcome::Rejected(Rejection::GraftAllowance {
            requested_mg: incoming_mg,
            allowance_mg: incoming_mg - 1,
        })
    );
    assert_eq!(
        state_hash(&over_cap),
        before,
        "the transaction published nothing"
    );
}

#[test]
fn retained_disfavoured_tissue_counts_cumulatively_after_carry_or_regrow() {
    for first_crossing in [Crossing::Carry, Crossing::Regrow] {
        let (world, donor, part, _) = fixture();
        let first_mg = donor_branch_mass(&world, donor, part);
        let mut world = with_allowance(world, first_mg);
        disfavoured(&mut world, SpeciesId(9_700));
        let second = OrganismId(9_701);
        let mut next = world
            .organisms
            .iter()
            .find(|organism| organism.id == donor)
            .expect("original carcass remains")
            .clone();
        next.id = second;
        next.species = SpeciesId(9_701);
        world.organisms.push(next);
        assert!(matches!(
            world.graft(donor, part, first_crossing),
            Outcome::Grafted { .. }
        ));

        let retained: u64 = world
            .controlled()
            .unwrap()
            .phenotype
            .body()
            .living()
            .filter(|part| matches!(part.provenance.origin, Origin::Incorporated { .. }))
            .map(|part| part.mass_mg)
            .sum();
        assert_eq!(
            retained, first_mg,
            "{first_crossing:?} tissue counts toward the cap"
        );

        disfavoured(&mut world, SpeciesId(9_701));
        let second_part = crate::PartId(1);
        let second_mg = donor_branch_mass(&world, second, second_part);
        let before = state_hash(&world);
        assert_eq!(
            world.graft(second, second_part, Crossing::Carry),
            Outcome::Rejected(Rejection::GraftAllowance {
                requested_mg: retained + second_mg,
                allowance_mg: first_mg,
            })
        );
        assert_eq!(state_hash(&world), before);
    }
}

#[test]
fn held_resolved_condition_raises_only_the_declared_allowance() {
    let (world, donor, part, _) = fixture();
    let incoming_mg = donor_branch_mass(&world, donor, part);
    let condition = conditions()
        .into_iter()
        .find(|condition| condition.name == "mesocosm:endured-hunger")
        .expect("native rules name the endured hunger condition")
        .id();
    let mut world = world.with_rules(WorldRules {
        graft_compatibility: Compatibility {
            allowance_cells: 0,
            allowance_mg: incoming_mg - 1,
            penalty_per_mg: 1,
            raised_by: [
                Some(ConditionAllowance {
                    condition,
                    additional_mg: 1,
                }),
                None,
            ],
        },
        ..WorldRules::native()
    });
    disfavoured(&mut world, SpeciesId(9_700));
    assert!(!world.discovered(condition));
    assert!(matches!(
        world.preview_graft(donor, part, Crossing::Carry),
        Err(Rejection::GraftAllowance { .. })
    ));

    world.observe(Evidence::Endured {
        stress: Stress::Hunger,
        ticks: crate::discovery::HUNGER_TICKS,
    });
    assert!(
        world.discovered(condition),
        "real evidence landed the configured condition"
    );
    let preview = world
        .preview_graft(donor, part, Crossing::Carry)
        .expect("one declared bonus closes the one mg gap");
    assert_eq!(preview.compatibility.unwrap().applied, vec![condition]);
}

#[test]
fn unknown_domain_stays_incompatible_and_rules_guard_snapshot_replay() {
    let (world, donor, part, _) = fixture();
    let mut world = with_allowance(world, u64::MAX);
    let recipient = world.controlled().expect("embodied").species;
    let lineages = world.lineages_mut();
    lineages.found(SpeciesId(9_700));
    lineages.set_domain(SpeciesId(9_700), Domain(99));
    lineages.set_domain(recipient, Domain(1));
    assert_eq!(
        world.graft(donor, part, Crossing::Carry),
        Outcome::Rejected(Rejection::Incompatible {
            from: Domain(99),
            into: Domain(1),
        })
    );

    let bytes = crate::snapshot::snapshot(&world).expect("snapshot");
    let restored = crate::snapshot::restore(&bytes).expect("round trip");
    assert_eq!(state_hash(&restored), state_hash(&world));
    assert!(matches!(
        crate::snapshot::restore_under(&bytes, world.admitted()),
        Err(crate::snapshot::SnapshotError::Rules { .. })
    ));
}
